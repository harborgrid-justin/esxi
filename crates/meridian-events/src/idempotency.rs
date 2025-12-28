//! Idempotency and deduplication for ensuring operations are executed exactly once.

use crate::error::{EventError, Result};
use blake3::Hasher;
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

/// Idempotency key for identifying operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Create a new idempotency key.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// Generate a key from a UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid.to_string())
    }

    /// Generate a key from data using content hashing.
    pub fn from_data(data: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(data);
        let hash = hasher.finalize();
        Self(hash.to_hex().to_string())
    }

    /// Get the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IdempotencyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for IdempotencyKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for IdempotencyKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Status of an idempotent operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    /// Operation is in progress
    InProgress,
    /// Operation completed successfully
    Completed,
    /// Operation failed
    Failed,
}

/// Record of an idempotent operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    /// Idempotency key
    pub key: IdempotencyKey,
    /// Operation status
    pub status: OperationStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Result data (serialized)
    pub result: Option<Vec<u8>>,
    /// Error message (if failed)
    pub error: Option<String>,
}

impl OperationRecord {
    /// Create a new operation record.
    pub fn new(key: IdempotencyKey) -> Self {
        Self {
            key,
            status: OperationStatus::InProgress,
            created_at: Utc::now(),
            completed_at: None,
            result: None,
            error: None,
        }
    }

    /// Mark the operation as completed.
    pub fn complete(&mut self, result: Vec<u8>) {
        self.status = OperationStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.result = Some(result);
    }

    /// Mark the operation as failed.
    pub fn fail(&mut self, error: String) {
        self.status = OperationStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error);
    }

    /// Check if the operation is complete.
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            OperationStatus::Completed | OperationStatus::Failed
        )
    }

    /// Get the result if available.
    pub fn get_result<T: for<'de> Deserialize<'de>>(&self) -> Result<Option<T>> {
        match &self.result {
            Some(data) => {
                let value = serde_json::from_slice(data)
                    .map_err(|e| EventError::Deserialization(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

/// Trait for idempotency stores.
pub trait IdempotencyStore: Send + Sync {
    /// Record an operation.
    fn record(&self, key: IdempotencyKey) -> Result<OperationRecord>;

    /// Get an operation record.
    fn get(&self, key: &IdempotencyKey) -> Result<Option<OperationRecord>>;

    /// Update an operation record.
    fn update(&self, record: OperationRecord) -> Result<()>;

    /// Delete an operation record.
    fn delete(&self, key: &IdempotencyKey) -> Result<()>;

    /// Check if an operation exists.
    fn exists(&self, key: &IdempotencyKey) -> Result<bool>;
}

/// In-memory idempotency store.
pub struct InMemoryIdempotencyStore {
    records: Arc<RwLock<HashMap<IdempotencyKey, OperationRecord>>>,
}

impl Default for InMemoryIdempotencyStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryIdempotencyStore {
    /// Create a new in-memory idempotency store.
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the number of records.
    pub fn len(&self) -> usize {
        self.records.read().len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.records.read().is_empty()
    }

    /// Clear all records.
    pub fn clear(&self) {
        self.records.write().clear();
    }
}

impl IdempotencyStore for InMemoryIdempotencyStore {
    fn record(&self, key: IdempotencyKey) -> Result<OperationRecord> {
        let mut records = self.records.write();

        if let Some(existing) = records.get(&key) {
            return Err(EventError::IdempotencyViolation(key.to_string()));
        }

        let record = OperationRecord::new(key.clone());
        records.insert(key, record.clone());

        Ok(record)
    }

    fn get(&self, key: &IdempotencyKey) -> Result<Option<OperationRecord>> {
        Ok(self.records.read().get(key).cloned())
    }

    fn update(&self, record: OperationRecord) -> Result<()> {
        self.records
            .write()
            .insert(record.key.clone(), record);
        Ok(())
    }

    fn delete(&self, key: &IdempotencyKey) -> Result<()> {
        self.records.write().remove(key);
        Ok(())
    }

    fn exists(&self, key: &IdempotencyKey) -> Result<bool> {
        Ok(self.records.read().contains_key(key))
    }
}

/// Idempotency manager for coordinating idempotent operations.
pub struct IdempotencyManager<S: IdempotencyStore> {
    store: Arc<S>,
    ttl: Option<Duration>,
}

impl<S: IdempotencyStore> IdempotencyManager<S> {
    /// Create a new idempotency manager.
    pub fn new(store: Arc<S>) -> Self {
        Self { store, ttl: None }
    }

    /// Set the time-to-live for operation records.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Execute an operation idempotently.
    pub async fn execute<F, T>(&self, key: IdempotencyKey, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
        T: Serialize + for<'de> Deserialize<'de>,
    {
        // Check if operation already exists
        if let Some(record) = self.store.get(&key)? {
            match record.status {
                OperationStatus::Completed => {
                    // Return cached result
                    if let Some(result) = record.get_result()? {
                        return Ok(result);
                    }
                    return Err(EventError::Internal(
                        "Completed operation has no result".to_string(),
                    ));
                }
                OperationStatus::Failed => {
                    return Err(EventError::Internal(format!(
                        "Operation previously failed: {}",
                        record.error.unwrap_or_default()
                    )));
                }
                OperationStatus::InProgress => {
                    return Err(EventError::Internal(
                        "Operation already in progress".to_string(),
                    ));
                }
            }
        }

        // Record the operation
        let mut record = self.store.record(key)?;

        // Execute the operation
        match operation() {
            Ok(result) => {
                let serialized = serde_json::to_vec(&result)
                    .map_err(|e| EventError::Serialization(e.to_string()))?;
                record.complete(serialized);
                self.store.update(record)?;
                Ok(result)
            }
            Err(err) => {
                record.fail(err.to_string());
                self.store.update(record)?;
                Err(err)
            }
        }
    }

    /// Clean up expired records.
    pub fn cleanup(&self) -> Result<usize> {
        if let Some(ttl) = self.ttl {
            let cutoff = Utc::now() - ttl;
            let mut removed = 0;

            // This is a simplified implementation
            // In a real implementation, you'd need to iterate through all records
            let _ = cutoff;
            let _ = removed;

            Ok(0)
        } else {
            Ok(0)
        }
    }
}

/// Deduplication tracker for detecting duplicate events.
pub struct DeduplicationTracker {
    seen: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    ttl: Duration,
}

impl DeduplicationTracker {
    /// Create a new deduplication tracker.
    pub fn new(ttl: Duration) -> Self {
        Self {
            seen: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    /// Check if an item has been seen before.
    pub fn has_seen(&self, id: &str) -> bool {
        let seen = self.seen.read();
        if let Some(timestamp) = seen.get(id) {
            Utc::now() - *timestamp < self.ttl
        } else {
            false
        }
    }

    /// Mark an item as seen.
    pub fn mark_seen(&self, id: String) {
        self.seen.write().insert(id, Utc::now());
    }

    /// Check and mark an item (returns true if it was already seen).
    pub fn check_and_mark(&self, id: String) -> bool {
        if self.has_seen(&id) {
            true
        } else {
            self.mark_seen(id);
            false
        }
    }

    /// Clean up expired entries.
    pub fn cleanup(&self) -> usize {
        let mut seen = self.seen.write();
        let cutoff = Utc::now() - self.ttl;
        let before = seen.len();

        seen.retain(|_, timestamp| *timestamp > cutoff);

        before - seen.len()
    }

    /// Get the number of tracked items.
    pub fn len(&self) -> usize {
        self.seen.read().len()
    }

    /// Check if the tracker is empty.
    pub fn is_empty(&self) -> bool {
        self.seen.read().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idempotency_key_from_data() {
        let data1 = b"test data";
        let data2 = b"test data";
        let data3 = b"different data";

        let key1 = IdempotencyKey::from_data(data1);
        let key2 = IdempotencyKey::from_data(data2);
        let key3 = IdempotencyKey::from_data(data3);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_operation_record() {
        let mut record = OperationRecord::new(IdempotencyKey::new("test"));

        assert_eq!(record.status, OperationStatus::InProgress);
        assert!(!record.is_complete());

        record.complete(vec![1, 2, 3]);
        assert_eq!(record.status, OperationStatus::Completed);
        assert!(record.is_complete());
    }

    #[test]
    fn test_in_memory_idempotency_store() {
        let store = InMemoryIdempotencyStore::new();
        let key = IdempotencyKey::new("test-key");

        assert!(!store.exists(&key).unwrap());

        let record = store.record(key.clone()).unwrap();
        assert!(store.exists(&key).unwrap());

        let retrieved = store.get(&key).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().key, record.key);

        store.delete(&key).unwrap();
        assert!(!store.exists(&key).unwrap());
    }

    #[test]
    fn test_idempotency_violation() {
        let store = InMemoryIdempotencyStore::new();
        let key = IdempotencyKey::new("test-key");

        store.record(key.clone()).unwrap();

        let result = store.record(key);
        assert!(matches!(result, Err(EventError::IdempotencyViolation(_))));
    }

    #[tokio::test]
    async fn test_idempotency_manager() {
        let store = Arc::new(InMemoryIdempotencyStore::new());
        let manager = IdempotencyManager::new(store);

        let key = IdempotencyKey::new("test-op");
        let mut counter = 0;

        let result1 = manager
            .execute(key.clone(), || {
                counter += 1;
                Ok(42)
            })
            .await
            .unwrap();

        assert_eq!(result1, 42);
        assert_eq!(counter, 1);

        // Second execution should return cached result
        let result2 = manager
            .execute(key, || {
                counter += 1;
                Ok(100)
            })
            .await
            .unwrap();

        assert_eq!(result2, 42);
        assert_eq!(counter, 1); // Counter shouldn't increase
    }

    #[test]
    fn test_deduplication_tracker() {
        let tracker = DeduplicationTracker::new(Duration::seconds(5));

        assert!(!tracker.has_seen("id1"));

        tracker.mark_seen("id1".to_string());
        assert!(tracker.has_seen("id1"));
        assert!(!tracker.has_seen("id2"));
    }

    #[test]
    fn test_deduplication_check_and_mark() {
        let tracker = DeduplicationTracker::new(Duration::seconds(5));

        let is_duplicate1 = tracker.check_and_mark("id1".to_string());
        assert!(!is_duplicate1);

        let is_duplicate2 = tracker.check_and_mark("id1".to_string());
        assert!(is_duplicate2);
    }
}
