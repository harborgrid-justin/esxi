//! Event archival and compaction for managing event store growth.

use crate::error::{EventError, Result};
use crate::event::{StoredEvent, StreamId};
use crate::snapshot::Snapshot;
use crate::store::{EventStore, ReadOptions};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Archival policy for determining which events to archive.
pub trait ArchivalPolicy: Send + Sync {
    /// Determine if an event should be archived.
    fn should_archive(&self, event: &StoredEvent) -> bool;
}

/// Time-based archival policy.
pub struct TimeBasedPolicy {
    age_threshold: Duration,
}

impl TimeBasedPolicy {
    /// Create a new time-based policy.
    pub fn new(age_threshold: Duration) -> Self {
        Self { age_threshold }
    }
}

impl ArchivalPolicy for TimeBasedPolicy {
    fn should_archive(&self, event: &StoredEvent) -> bool {
        Utc::now() - event.metadata.timestamp > self.age_threshold
    }
}

/// Version-based archival policy.
pub struct VersionBasedPolicy {
    version_threshold: u64,
}

impl VersionBasedPolicy {
    /// Create a new version-based policy.
    pub fn new(version_threshold: u64) -> Self {
        Self { version_threshold }
    }
}

impl ArchivalPolicy for VersionBasedPolicy {
    fn should_archive(&self, event: &StoredEvent) -> bool {
        event.metadata.sequence < self.version_threshold
    }
}

/// Combined archival policy.
pub struct CombinedPolicy {
    policies: Vec<Box<dyn ArchivalPolicy>>,
    require_all: bool,
}

impl CombinedPolicy {
    /// Create a new combined policy that requires all policies to match.
    pub fn all(policies: Vec<Box<dyn ArchivalPolicy>>) -> Self {
        Self {
            policies,
            require_all: true,
        }
    }

    /// Create a new combined policy that requires any policy to match.
    pub fn any(policies: Vec<Box<dyn ArchivalPolicy>>) -> Self {
        Self {
            policies,
            require_all: false,
        }
    }
}

impl ArchivalPolicy for CombinedPolicy {
    fn should_archive(&self, event: &StoredEvent) -> bool {
        if self.require_all {
            self.policies.iter().all(|p| p.should_archive(event))
        } else {
            self.policies.iter().any(|p| p.should_archive(event))
        }
    }
}

/// Archival destination for storing archived events.
#[async_trait]
pub trait ArchivalDestination: Send + Sync {
    /// Archive events.
    async fn archive(&self, events: Vec<StoredEvent>) -> Result<()>;

    /// Retrieve archived events.
    async fn retrieve(&self, stream_id: &StreamId) -> Result<Vec<StoredEvent>>;

    /// Delete archived events.
    async fn delete(&self, stream_id: &StreamId) -> Result<()>;
}

/// File-based archival destination.
pub struct FileArchive {
    base_path: PathBuf,
}

impl FileArchive {
    /// Create a new file archive.
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Get the archive file path for a stream.
    fn archive_path(&self, stream_id: &StreamId) -> PathBuf {
        self.base_path.join(format!("{}.archive", stream_id))
    }
}

#[async_trait]
impl ArchivalDestination for FileArchive {
    async fn archive(&self, events: Vec<StoredEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let stream_id = &events[0].metadata.stream_id;
        let path = self.archive_path(stream_id);

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Serialize events
        let serialized = serde_json::to_vec(&events)
            .map_err(|e| EventError::Serialization(e.to_string()))?;

        // Write to file
        tokio::fs::write(&path, serialized).await?;

        Ok(())
    }

    async fn retrieve(&self, stream_id: &StreamId) -> Result<Vec<StoredEvent>> {
        let path = self.archive_path(stream_id);

        // Read from file
        let data = tokio::fs::read(&path).await?;

        // Deserialize events
        let events = serde_json::from_slice(&data)
            .map_err(|e| EventError::Deserialization(e.to_string()))?;

        Ok(events)
    }

    async fn delete(&self, stream_id: &StreamId) -> Result<()> {
        let path = self.archive_path(stream_id);
        tokio::fs::remove_file(&path).await?;
        Ok(())
    }
}

/// Archival metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivalMetadata {
    /// Stream ID
    pub stream_id: StreamId,
    /// Number of events archived
    pub events_archived: usize,
    /// Timestamp of archival
    pub archived_at: DateTime<Utc>,
    /// Version range archived
    pub version_range: (u64, u64),
}

/// Archival service for managing event archival.
pub struct ArchivalService<S, D>
where
    S: EventStore,
    D: ArchivalDestination,
{
    event_store: S,
    destination: D,
    policy: Box<dyn ArchivalPolicy>,
}

impl<S, D> ArchivalService<S, D>
where
    S: EventStore,
    D: ArchivalDestination,
{
    /// Create a new archival service.
    pub fn new(event_store: S, destination: D, policy: Box<dyn ArchivalPolicy>) -> Self {
        Self {
            event_store,
            destination,
            policy,
        }
    }

    /// Archive events from a stream.
    pub async fn archive_stream(&self, stream_id: &StreamId) -> Result<ArchivalMetadata> {
        // Read all events from the stream
        let events = self
            .event_store
            .read(stream_id, ReadOptions::default())
            .await?;

        // Filter events to archive
        let to_archive: Vec<_> = events
            .into_iter()
            .filter(|e| self.policy.should_archive(e))
            .collect();

        if to_archive.is_empty() {
            return Err(EventError::Archival("No events to archive".to_string()));
        }

        let version_range = (
            to_archive.first().unwrap().metadata.sequence,
            to_archive.last().unwrap().metadata.sequence,
        );
        let events_archived = to_archive.len();

        // Archive the events
        self.destination.archive(to_archive).await?;

        Ok(ArchivalMetadata {
            stream_id: stream_id.clone(),
            events_archived,
            archived_at: Utc::now(),
            version_range,
        })
    }

    /// Restore archived events.
    pub async fn restore_stream(&self, stream_id: &StreamId) -> Result<Vec<StoredEvent>> {
        self.destination.retrieve(stream_id).await
    }
}

/// Compaction strategy for reducing event store size.
pub trait CompactionStrategy: Send + Sync {
    /// Determine if a stream should be compacted.
    fn should_compact(&self, event_count: usize, last_snapshot_version: Option<u64>) -> bool;
}

/// Snapshot-based compaction strategy.
pub struct SnapshotCompaction {
    min_events: usize,
}

impl SnapshotCompaction {
    /// Create a new snapshot compaction strategy.
    pub fn new(min_events: usize) -> Self {
        Self { min_events }
    }
}

impl CompactionStrategy for SnapshotCompaction {
    fn should_compact(&self, event_count: usize, last_snapshot_version: Option<u64>) -> bool {
        if let Some(snap_version) = last_snapshot_version {
            event_count > self.min_events && snap_version > 0
        } else {
            false
        }
    }
}

/// Compaction service for reducing event store size.
pub struct CompactionService<S>
where
    S: EventStore,
{
    event_store: S,
    strategy: Box<dyn CompactionStrategy>,
}

impl<S> CompactionService<S>
where
    S: EventStore,
{
    /// Create a new compaction service.
    pub fn new(event_store: S, strategy: Box<dyn CompactionStrategy>) -> Self {
        Self {
            event_store,
            strategy,
        }
    }

    /// Compact a stream by removing events before the last snapshot.
    pub async fn compact_stream<A>(
        &self,
        stream_id: &StreamId,
        snapshot: Option<Snapshot<A>>,
    ) -> Result<CompactionResult> {
        // Read all events
        let events = self
            .event_store
            .read(stream_id, ReadOptions::default())
            .await?;

        let event_count = events.len();
        let last_snapshot_version = snapshot.as_ref().map(|s| s.version());

        if !self.strategy.should_compact(event_count, last_snapshot_version) {
            return Ok(CompactionResult {
                events_removed: 0,
                events_retained: event_count,
            });
        }

        // In a real implementation, you would remove events before the snapshot
        // This is a placeholder since our event store doesn't support deletion
        Ok(CompactionResult {
            events_removed: 0,
            events_retained: event_count,
        })
    }
}

/// Result of a compaction operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionResult {
    /// Number of events removed
    pub events_removed: usize,
    /// Number of events retained
    pub events_retained: usize,
}

impl CompactionResult {
    /// Get the total number of events processed.
    pub fn total_events(&self) -> usize {
        self.events_removed + self.events_retained
    }

    /// Get the compression ratio.
    pub fn compression_ratio(&self) -> f64 {
        if self.total_events() == 0 {
            0.0
        } else {
            self.events_removed as f64 / self.total_events() as f64
        }
    }
}

/// Archive manager for coordinating archival and compaction.
pub struct ArchiveManager {
    archival_metadata: HashMap<StreamId, ArchivalMetadata>,
    compaction_results: HashMap<StreamId, CompactionResult>,
}

impl Default for ArchiveManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchiveManager {
    /// Create a new archive manager.
    pub fn new() -> Self {
        Self {
            archival_metadata: HashMap::new(),
            compaction_results: HashMap::new(),
        }
    }

    /// Record archival metadata.
    pub fn record_archival(&mut self, metadata: ArchivalMetadata) {
        self.archival_metadata
            .insert(metadata.stream_id.clone(), metadata);
    }

    /// Record compaction result.
    pub fn record_compaction(&mut self, stream_id: StreamId, result: CompactionResult) {
        self.compaction_results.insert(stream_id, result);
    }

    /// Get archival metadata for a stream.
    pub fn get_archival_metadata(&self, stream_id: &StreamId) -> Option<&ArchivalMetadata> {
        self.archival_metadata.get(stream_id)
    }

    /// Get compaction result for a stream.
    pub fn get_compaction_result(&self, stream_id: &StreamId) -> Option<&CompactionResult> {
        self.compaction_results.get(stream_id)
    }

    /// Get total events archived.
    pub fn total_events_archived(&self) -> usize {
        self.archival_metadata
            .values()
            .map(|m| m.events_archived)
            .sum()
    }

    /// Get total events removed by compaction.
    pub fn total_events_removed(&self) -> usize {
        self.compaction_results
            .values()
            .map(|r| r.events_removed)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventMetadata, EventVersion};

    fn create_test_event(sequence: u64, days_old: i64) -> StoredEvent {
        let mut metadata = EventMetadata::new(
            StreamId::new("test"),
            sequence,
            EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
        );
        metadata.timestamp = Utc::now() - Duration::days(days_old);

        StoredEvent::new(metadata, vec![])
    }

    #[test]
    fn test_time_based_policy() {
        let policy = TimeBasedPolicy::new(Duration::days(30));

        let recent_event = create_test_event(1, 10);
        let old_event = create_test_event(2, 60);

        assert!(!policy.should_archive(&recent_event));
        assert!(policy.should_archive(&old_event));
    }

    #[test]
    fn test_version_based_policy() {
        let policy = VersionBasedPolicy::new(50);

        let event1 = create_test_event(25, 0);
        let event2 = create_test_event(75, 0);

        assert!(policy.should_archive(&event1));
        assert!(!policy.should_archive(&event2));
    }

    #[test]
    fn test_combined_policy_all() {
        let policies: Vec<Box<dyn ArchivalPolicy>> = vec![
            Box::new(TimeBasedPolicy::new(Duration::days(30))),
            Box::new(VersionBasedPolicy::new(50)),
        ];

        let policy = CombinedPolicy::all(policies);

        let event1 = create_test_event(25, 60); // Old and low version
        let event2 = create_test_event(75, 60); // Old but high version
        let event3 = create_test_event(25, 10); // Recent but low version

        assert!(policy.should_archive(&event1));
        assert!(!policy.should_archive(&event2));
        assert!(!policy.should_archive(&event3));
    }

    #[test]
    fn test_snapshot_compaction_strategy() {
        let strategy = SnapshotCompaction::new(100);

        assert!(!strategy.should_compact(50, Some(50)));
        assert!(strategy.should_compact(150, Some(50)));
        assert!(!strategy.should_compact(150, None));
    }

    #[test]
    fn test_compaction_result() {
        let result = CompactionResult {
            events_removed: 30,
            events_retained: 70,
        };

        assert_eq!(result.total_events(), 100);
        assert_eq!(result.compression_ratio(), 0.3);
    }

    #[test]
    fn test_archive_manager() {
        let mut manager = ArchiveManager::new();

        let metadata = ArchivalMetadata {
            stream_id: StreamId::new("test"),
            events_archived: 50,
            archived_at: Utc::now(),
            version_range: (1, 50),
        };

        manager.record_archival(metadata.clone());

        assert_eq!(manager.total_events_archived(), 50);
        assert!(manager.get_archival_metadata(&metadata.stream_id).is_some());
    }
}
