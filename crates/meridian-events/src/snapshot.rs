//! Snapshotting for aggregate performance optimization.

use crate::aggregate::{AggregateId, AggregateRoot};
use crate::error::{EventError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Snapshot metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Aggregate ID
    pub aggregate_id: AggregateId,
    /// Aggregate type
    pub aggregate_type: String,
    /// Version at which the snapshot was taken
    pub version: u64,
    /// Timestamp of the snapshot
    pub timestamp: DateTime<Utc>,
}

impl SnapshotMetadata {
    /// Create new snapshot metadata.
    pub fn new(aggregate_id: AggregateId, aggregate_type: String, version: u64) -> Self {
        Self {
            aggregate_id,
            aggregate_type,
            version,
            timestamp: Utc::now(),
        }
    }
}

/// Snapshot container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot<A> {
    /// Snapshot metadata
    pub metadata: SnapshotMetadata,
    /// The aggregate state
    pub state: A,
}

impl<A> Snapshot<A> {
    /// Create a new snapshot.
    pub fn new(metadata: SnapshotMetadata, state: A) -> Self {
        Self { metadata, state }
    }

    /// Get the version.
    pub fn version(&self) -> u64 {
        self.metadata.version
    }

    /// Get the aggregate ID.
    pub fn aggregate_id(&self) -> &AggregateId {
        &self.metadata.aggregate_id
    }

    /// Get the state.
    pub fn state(&self) -> &A {
        &self.state
    }
}

/// Trait for snapshot stores.
#[async_trait]
pub trait SnapshotStore<A: AggregateRoot>: Send + Sync {
    /// Save a snapshot.
    async fn save(&self, aggregate: &A) -> Result<()>;

    /// Load a snapshot.
    async fn load(&self, id: &AggregateId) -> Result<Snapshot<A>>;

    /// Delete a snapshot.
    async fn delete(&self, id: &AggregateId) -> Result<()>;

    /// Check if a snapshot exists.
    async fn exists(&self, id: &AggregateId) -> Result<bool>;

    /// Get snapshot metadata without loading the full state.
    async fn get_metadata(&self, id: &AggregateId) -> Result<SnapshotMetadata>;
}

/// In-memory snapshot store implementation.
pub struct InMemorySnapshotStore<A>
where
    A: AggregateRoot + Clone,
{
    snapshots: Arc<RwLock<HashMap<AggregateId, Snapshot<A>>>>,
}

impl<A> Default for InMemorySnapshotStore<A>
where
    A: AggregateRoot + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<A> InMemorySnapshotStore<A>
where
    A: AggregateRoot + Clone,
{
    /// Create a new in-memory snapshot store.
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the number of snapshots.
    pub fn len(&self) -> usize {
        self.snapshots.read().len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.snapshots.read().is_empty()
    }

    /// Clear all snapshots.
    pub fn clear(&self) {
        self.snapshots.write().clear();
    }
}

#[async_trait]
impl<A> SnapshotStore<A> for InMemorySnapshotStore<A>
where
    A: AggregateRoot + Clone,
{
    async fn save(&self, aggregate: &A) -> Result<()> {
        let metadata = SnapshotMetadata::new(
            aggregate.aggregate_id().clone(),
            A::aggregate_type().to_string(),
            aggregate.version(),
        );

        let snapshot = Snapshot::new(metadata, aggregate.clone());

        self.snapshots
            .write()
            .insert(aggregate.aggregate_id().clone(), snapshot);

        Ok(())
    }

    async fn load(&self, id: &AggregateId) -> Result<Snapshot<A>> {
        self.snapshots
            .read()
            .get(id)
            .cloned()
            .ok_or_else(|| EventError::SnapshotNotFound(id.to_string()))
    }

    async fn delete(&self, id: &AggregateId) -> Result<()> {
        self.snapshots.write().remove(id);
        Ok(())
    }

    async fn exists(&self, id: &AggregateId) -> Result<bool> {
        Ok(self.snapshots.read().contains_key(id))
    }

    async fn get_metadata(&self, id: &AggregateId) -> Result<SnapshotMetadata> {
        let snapshot = self.load(id).await?;
        Ok(snapshot.metadata)
    }
}

/// Snapshot strategy for determining when to take snapshots.
pub trait SnapshotStrategy: Send + Sync {
    /// Determine if a snapshot should be taken.
    fn should_snapshot(&self, current_version: u64, last_snapshot_version: u64) -> bool;
}

/// Frequency-based snapshot strategy.
pub struct FrequencyStrategy {
    frequency: u64,
}

impl FrequencyStrategy {
    /// Create a new frequency strategy.
    pub fn new(frequency: u64) -> Self {
        Self { frequency }
    }
}

impl SnapshotStrategy for FrequencyStrategy {
    fn should_snapshot(&self, current_version: u64, last_snapshot_version: u64) -> bool {
        current_version - last_snapshot_version >= self.frequency
    }
}

/// Always snapshot strategy (for testing).
pub struct AlwaysStrategy;

impl SnapshotStrategy for AlwaysStrategy {
    fn should_snapshot(&self, _current_version: u64, _last_snapshot_version: u64) -> bool {
        true
    }
}

/// Never snapshot strategy.
pub struct NeverStrategy;

impl SnapshotStrategy for NeverStrategy {
    fn should_snapshot(&self, _current_version: u64, _last_snapshot_version: u64) -> bool {
        false
    }
}

/// Snapshot manager for coordinating snapshot operations.
pub struct SnapshotManager<A, S>
where
    A: AggregateRoot,
    S: SnapshotStore<A>,
{
    store: Arc<S>,
    strategy: Arc<dyn SnapshotStrategy>,
    _phantom: std::marker::PhantomData<A>,
}

impl<A, S> SnapshotManager<A, S>
where
    A: AggregateRoot,
    S: SnapshotStore<A>,
{
    /// Create a new snapshot manager.
    pub fn new(store: Arc<S>, strategy: Arc<dyn SnapshotStrategy>) -> Self {
        Self {
            store,
            strategy,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Check if a snapshot should be taken.
    pub async fn should_snapshot(&self, id: &AggregateId, current_version: u64) -> Result<bool> {
        let last_version = match self.store.get_metadata(id).await {
            Ok(metadata) => metadata.version,
            Err(_) => 0,
        };

        Ok(self
            .strategy
            .should_snapshot(current_version, last_version))
    }

    /// Save a snapshot if the strategy allows.
    pub async fn save_if_needed(&self, aggregate: &A) -> Result<bool> {
        let should_save = self
            .should_snapshot(aggregate.aggregate_id(), aggregate.version())
            .await?;

        if should_save {
            self.store.save(aggregate).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Load a snapshot.
    pub async fn load(&self, id: &AggregateId) -> Result<Snapshot<A>> {
        self.store.load(id).await
    }

    /// Delete a snapshot.
    pub async fn delete(&self, id: &AggregateId) -> Result<()> {
        self.store.delete(id).await
    }
}

/// Snapshot cleanup policy.
pub trait SnapshotCleanupPolicy: Send + Sync {
    /// Determine if a snapshot should be deleted.
    fn should_delete(&self, metadata: &SnapshotMetadata) -> bool;
}

/// Time-based cleanup policy.
pub struct TimeBasedCleanup {
    max_age: chrono::Duration,
}

impl TimeBasedCleanup {
    /// Create a new time-based cleanup policy.
    pub fn new(max_age: chrono::Duration) -> Self {
        Self { max_age }
    }
}

impl SnapshotCleanupPolicy for TimeBasedCleanup {
    fn should_delete(&self, metadata: &SnapshotMetadata) -> bool {
        let age = Utc::now() - metadata.timestamp;
        age > self.max_age
    }
}

/// Version-based cleanup policy.
pub struct VersionBasedCleanup {
    keep_last_n: usize,
}

impl VersionBasedCleanup {
    /// Create a new version-based cleanup policy.
    pub fn new(keep_last_n: usize) -> Self {
        Self { keep_last_n }
    }

    /// Get the number of versions to keep.
    pub fn keep_last_n(&self) -> usize {
        self.keep_last_n
    }
}

impl SnapshotCleanupPolicy for VersionBasedCleanup {
    fn should_delete(&self, _metadata: &SnapshotMetadata) -> bool {
        // This would need access to all snapshots to implement properly
        // For now, return false
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::AggregateRoot;
    use async_trait::async_trait;

    #[derive(Debug, Clone)]
    struct TestAggregate {
        id: AggregateId,
        version: u64,
        value: i32,
    }

    #[async_trait]
    impl AggregateRoot for TestAggregate {
        type Event = ();

        fn aggregate_type() -> &'static str {
            "TestAggregate"
        }

        fn aggregate_id(&self) -> &AggregateId {
            &self.id
        }

        fn version(&self) -> u64 {
            self.version
        }

        fn apply(&mut self, _event: &Self::Event) -> Result<()> {
            self.version += 1;
            Ok(())
        }

        fn new(id: AggregateId) -> Self {
            Self {
                id,
                version: 0,
                value: 0,
            }
        }
    }

    #[tokio::test]
    async fn test_in_memory_snapshot_store() {
        let store = InMemorySnapshotStore::<TestAggregate>::new();

        let aggregate = TestAggregate {
            id: AggregateId::new("test-1"),
            version: 5,
            value: 42,
        };

        store.save(&aggregate).await.unwrap();
        assert!(store.exists(&aggregate.id).await.unwrap());

        let snapshot = store.load(&aggregate.id).await.unwrap();
        assert_eq!(snapshot.version(), 5);
        assert_eq!(snapshot.state().value, 42);

        store.delete(&aggregate.id).await.unwrap();
        assert!(!store.exists(&aggregate.id).await.unwrap());
    }

    #[test]
    fn test_frequency_strategy() {
        let strategy = FrequencyStrategy::new(10);

        assert!(!strategy.should_snapshot(5, 0));
        assert!(strategy.should_snapshot(10, 0));
        assert!(strategy.should_snapshot(15, 5));
        assert!(!strategy.should_snapshot(14, 5));
    }

    #[test]
    fn test_always_strategy() {
        let strategy = AlwaysStrategy;
        assert!(strategy.should_snapshot(1, 0));
        assert!(strategy.should_snapshot(100, 99));
    }

    #[test]
    fn test_never_strategy() {
        let strategy = NeverStrategy;
        assert!(!strategy.should_snapshot(1, 0));
        assert!(!strategy.should_snapshot(100, 0));
    }

    #[tokio::test]
    async fn test_snapshot_manager() {
        let store = Arc::new(InMemorySnapshotStore::<TestAggregate>::new());
        let strategy = Arc::new(FrequencyStrategy::new(5));
        let manager = SnapshotManager::new(store, strategy);

        let aggregate = TestAggregate {
            id: AggregateId::new("test-1"),
            version: 5,
            value: 42,
        };

        let saved = manager.save_if_needed(&aggregate).await.unwrap();
        assert!(saved);

        let loaded = manager.load(&aggregate.id).await.unwrap();
        assert_eq!(loaded.version(), 5);
    }
}
