//! Dead letter queue for failed tasks and workflows.

use crate::dag::TaskId;
use crate::error::{WorkflowError, WorkflowResult};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// A unique identifier for a dead letter entry.
pub type DeadLetterId = Uuid;

/// Reason for a task/workflow being sent to the dead letter queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeadLetterReason {
    /// Maximum retries exhausted.
    RetriesExhausted,

    /// Execution timeout exceeded.
    TimeoutExceeded,

    /// Fatal error that cannot be retried.
    FatalError { error: String },

    /// Invalid configuration or definition.
    InvalidConfiguration { error: String },

    /// Dependency failure.
    DependencyFailed { dependency: String },

    /// Manual intervention required.
    ManualIntervention { reason: String },

    /// Resource limit exceeded.
    ResourceLimitExceeded { resource: String },

    /// Other reason.
    Other { reason: String },
}

impl DeadLetterReason {
    /// Returns true if the failure is potentially recoverable.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            DeadLetterReason::RetriesExhausted
                | DeadLetterReason::TimeoutExceeded
                | DeadLetterReason::DependencyFailed { .. }
                | DeadLetterReason::ResourceLimitExceeded { .. }
        )
    }
}

/// A dead letter entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetter<T> {
    /// Unique identifier for this dead letter.
    pub id: DeadLetterId,

    /// The failed payload.
    pub payload: T,

    /// Reason for failure.
    pub reason: DeadLetterReason,

    /// Original error message.
    pub error_message: String,

    /// Number of attempts made.
    pub attempt_count: u32,

    /// Timestamp when added to DLQ.
    pub dead_lettered_at: DateTime<Utc>,

    /// Original enqueue timestamp.
    pub original_enqueued_at: DateTime<Utc>,

    /// Stack trace or additional debug information.
    pub debug_info: Option<String>,

    /// Custom metadata.
    pub metadata: HashMap<String, String>,

    /// Whether this item has been processed/acknowledged from DLQ.
    pub processed: bool,

    /// Processing notes or resolution.
    pub processing_notes: Option<String>,
}

impl<T> DeadLetter<T> {
    /// Creates a new dead letter entry.
    pub fn new(
        payload: T,
        reason: DeadLetterReason,
        error_message: String,
        attempt_count: u32,
        original_enqueued_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            payload,
            reason,
            error_message,
            attempt_count,
            dead_lettered_at: Utc::now(),
            original_enqueued_at,
            debug_info: None,
            metadata: HashMap::new(),
            processed: false,
            processing_notes: None,
        }
    }

    /// Adds debug information.
    pub fn with_debug_info(mut self, debug_info: String) -> Self {
        self.debug_info = Some(debug_info);
        self
    }

    /// Adds metadata.
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Gets the total time from original enqueue to dead letter.
    pub fn total_duration(&self) -> Duration {
        self.dead_lettered_at
            .signed_duration_since(self.original_enqueued_at)
    }
}

/// Filter criteria for querying dead letters.
#[derive(Debug, Clone, Default)]
pub struct DeadLetterFilter {
    /// Filter by reason type.
    pub reason_type: Option<String>,

    /// Filter by processed status.
    pub processed: Option<bool>,

    /// Filter by date range (from).
    pub from_date: Option<DateTime<Utc>>,

    /// Filter by date range (to).
    pub to_date: Option<DateTime<Utc>>,

    /// Filter by minimum attempt count.
    pub min_attempts: Option<u32>,
}

impl DeadLetterFilter {
    /// Creates a new filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters for unprocessed items.
    pub fn unprocessed() -> Self {
        Self {
            processed: Some(false),
            ..Default::default()
        }
    }

    /// Filters for processed items.
    pub fn processed_items() -> Self {
        Self {
            processed: Some(true),
            ..Default::default()
        }
    }

    /// Filters by date range.
    pub fn date_range(from: DateTime<Utc>, to: DateTime<Utc>) -> Self {
        Self {
            from_date: Some(from),
            to_date: Some(to),
            ..Default::default()
        }
    }

    /// Checks if a dead letter matches this filter.
    pub fn matches<T>(&self, dl: &DeadLetter<T>) -> bool {
        if let Some(processed) = self.processed {
            if dl.processed != processed {
                return false;
            }
        }

        if let Some(ref from) = self.from_date {
            if dl.dead_lettered_at < *from {
                return false;
            }
        }

        if let Some(ref to) = self.to_date {
            if dl.dead_lettered_at > *to {
                return false;
            }
        }

        if let Some(min_attempts) = self.min_attempts {
            if dl.attempt_count < min_attempts {
                return false;
            }
        }

        true
    }
}

/// Statistics for the dead letter queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterStats {
    /// Total number of items in DLQ.
    pub total_items: usize,

    /// Number of unprocessed items.
    pub unprocessed_items: usize,

    /// Number of processed items.
    pub processed_items: usize,

    /// Items by reason type.
    pub items_by_reason: HashMap<String, usize>,

    /// Oldest item timestamp.
    pub oldest_item_at: Option<DateTime<Utc>>,

    /// Newest item timestamp.
    pub newest_item_at: Option<DateTime<Utc>>,
}

/// Dead letter queue for failed tasks and workflows.
pub struct DeadLetterQueue<T: Clone> {
    /// Dead letter entries.
    entries: Arc<RwLock<HashMap<DeadLetterId, DeadLetter<T>>>>,

    /// Maximum number of entries to retain.
    max_entries: Option<usize>,

    /// Time-to-live for entries (auto-delete after this duration).
    ttl: Option<Duration>,

    /// Queue name for identification.
    name: String,
}

impl<T: Clone> DeadLetterQueue<T> {
    /// Creates a new dead letter queue.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_entries: None,
            ttl: Some(Duration::days(30)), // Default 30 days retention
            name: name.into(),
        }
    }

    /// Sets the maximum number of entries.
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = Some(max);
        self
    }

    /// Sets the time-to-live for entries.
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Adds an entry to the dead letter queue.
    pub async fn add(&self, entry: DeadLetter<T>) -> WorkflowResult<DeadLetterId> {
        let mut entries = self.entries.write().await;

        // Check max entries limit
        if let Some(max) = self.max_entries {
            if entries.len() >= max {
                warn!(
                    "Dead letter queue {} is at capacity ({}), removing oldest entry",
                    self.name, max
                );
                // Remove oldest entry
                if let Some(oldest_id) = entries
                    .iter()
                    .min_by_key(|(_, e)| e.dead_lettered_at)
                    .map(|(id, _)| *id)
                {
                    entries.remove(&oldest_id);
                }
            }
        }

        let entry_id = entry.id;
        info!(
            "Added entry {} to dead letter queue {} (reason: {:?})",
            entry_id, self.name, entry.reason
        );

        entries.insert(entry_id, entry);
        Ok(entry_id)
    }

    /// Retrieves an entry by ID.
    pub async fn get(&self, id: DeadLetterId) -> Option<DeadLetter<T>> {
        let entries = self.entries.read().await;
        entries.get(&id).cloned()
    }

    /// Lists entries matching the filter.
    pub async fn list(&self, filter: DeadLetterFilter) -> Vec<DeadLetter<T>> {
        let entries = self.entries.read().await;

        entries
            .values()
            .filter(|entry| filter.matches(entry))
            .cloned()
            .collect()
    }

    /// Marks an entry as processed.
    pub async fn mark_processed(
        &self,
        id: DeadLetterId,
        notes: Option<String>,
    ) -> WorkflowResult<()> {
        let mut entries = self.entries.write().await;

        let entry = entries
            .get_mut(&id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Dead letter {} not found", id)))?;

        entry.processed = true;
        entry.processing_notes = notes;

        info!("Marked dead letter {} as processed", id);
        Ok(())
    }

    /// Removes an entry from the DLQ.
    pub async fn remove(&self, id: DeadLetterId) -> WorkflowResult<DeadLetter<T>> {
        let mut entries = self.entries.write().await;

        entries
            .remove(&id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Dead letter {} not found", id)))
    }

    /// Retries a dead lettered item by returning it.
    pub async fn retry(&self, id: DeadLetterId) -> WorkflowResult<T> {
        let entry = self.remove(id).await?;
        info!("Retrying dead letter {}", entry.id);
        Ok(entry.payload)
    }

    /// Clears all processed entries.
    pub async fn clear_processed(&self) -> usize {
        let mut entries = self.entries.write().await;

        let initial_count = entries.len();
        entries.retain(|_, entry| !entry.processed);
        let removed_count = initial_count - entries.len();

        info!("Cleared {} processed entries from dead letter queue {}", removed_count, self.name);
        removed_count
    }

    /// Clears entries older than the TTL.
    pub async fn clear_expired(&self) -> usize {
        if let Some(ttl) = self.ttl {
            let mut entries = self.entries.write().await;
            let now = Utc::now();
            let cutoff = now - ttl;

            let initial_count = entries.len();
            entries.retain(|_, entry| entry.dead_lettered_at > cutoff);
            let removed_count = initial_count - entries.len();

            info!("Cleared {} expired entries from dead letter queue {}", removed_count, self.name);
            removed_count
        } else {
            0
        }
    }

    /// Gets statistics for the DLQ.
    pub async fn stats(&self) -> DeadLetterStats {
        let entries = self.entries.read().await;

        let total_items = entries.len();
        let unprocessed_items = entries.values().filter(|e| !e.processed).count();
        let processed_items = total_items - unprocessed_items;

        let mut items_by_reason: HashMap<String, usize> = HashMap::new();
        for entry in entries.values() {
            let reason_name = format!("{:?}", entry.reason);
            *items_by_reason.entry(reason_name).or_insert(0) += 1;
        }

        let oldest_item_at = entries
            .values()
            .map(|e| e.dead_lettered_at)
            .min();

        let newest_item_at = entries
            .values()
            .map(|e| e.dead_lettered_at)
            .max();

        DeadLetterStats {
            total_items,
            unprocessed_items,
            processed_items,
            items_by_reason,
            oldest_item_at,
            newest_item_at,
        }
    }

    /// Clears all entries from the DLQ.
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
        info!("Cleared all entries from dead letter queue {}", self.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dlq_add_and_retrieve() {
        let dlq = DeadLetterQueue::new("test");

        let entry = DeadLetter::new(
            "test_payload".to_string(),
            DeadLetterReason::RetriesExhausted,
            "Test error".to_string(),
            3,
            Utc::now(),
        );

        let entry_id = dlq.add(entry.clone()).await.unwrap();

        let retrieved = dlq.get(entry_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().payload, "test_payload");
    }

    #[tokio::test]
    async fn test_dlq_mark_processed() {
        let dlq = DeadLetterQueue::new("test");

        let entry = DeadLetter::new(
            "test".to_string(),
            DeadLetterReason::RetriesExhausted,
            "Error".to_string(),
            1,
            Utc::now(),
        );

        let entry_id = dlq.add(entry).await.unwrap();

        dlq.mark_processed(entry_id, Some("Resolved manually".to_string()))
            .await
            .unwrap();

        let retrieved = dlq.get(entry_id).await.unwrap();
        assert!(retrieved.processed);
        assert_eq!(retrieved.processing_notes, Some("Resolved manually".to_string()));
    }

    #[tokio::test]
    async fn test_dlq_filter() {
        let dlq = DeadLetterQueue::new("test");

        let entry1 = DeadLetter::new(
            "test1".to_string(),
            DeadLetterReason::RetriesExhausted,
            "Error".to_string(),
            1,
            Utc::now(),
        );

        let mut entry2 = DeadLetter::new(
            "test2".to_string(),
            DeadLetterReason::FatalError {
                error: "Fatal".to_string(),
            },
            "Error".to_string(),
            1,
            Utc::now(),
        );
        entry2.processed = true;

        dlq.add(entry1).await.unwrap();
        dlq.add(entry2).await.unwrap();

        let unprocessed = dlq.list(DeadLetterFilter::unprocessed()).await;
        assert_eq!(unprocessed.len(), 1);

        let processed = dlq.list(DeadLetterFilter::processed_items()).await;
        assert_eq!(processed.len(), 1);
    }

    #[tokio::test]
    async fn test_dlq_stats() {
        let dlq = DeadLetterQueue::new("test");

        let entry = DeadLetter::new(
            "test".to_string(),
            DeadLetterReason::RetriesExhausted,
            "Error".to_string(),
            1,
            Utc::now(),
        );

        dlq.add(entry).await.unwrap();

        let stats = dlq.stats().await;
        assert_eq!(stats.total_items, 1);
        assert_eq!(stats.unprocessed_items, 1);
    }
}
