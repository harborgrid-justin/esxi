//! Event replay and reconstruction for debugging and auditing.

use crate::aggregate::{Aggregate, AggregateId, AggregateRoot};
use crate::error::Result;
use crate::event::{DomainEvent, StoredEvent, StreamId};
use crate::store::{EventStore, ReadOptions};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Replay options for controlling event replay.
#[derive(Debug, Clone)]
pub struct ReplayOptions {
    /// Start from this version
    pub from_version: u64,
    /// End at this version (inclusive, None for all)
    pub to_version: Option<u64>,
    /// Start from this timestamp
    pub from_timestamp: Option<DateTime<Utc>>,
    /// End at this timestamp (inclusive, None for all)
    pub to_timestamp: Option<DateTime<Utc>>,
    /// Batch size for processing
    pub batch_size: usize,
}

impl Default for ReplayOptions {
    fn default() -> Self {
        Self {
            from_version: 0,
            to_version: None,
            from_timestamp: None,
            to_timestamp: None,
            batch_size: 100,
        }
    }
}

impl ReplayOptions {
    /// Create new replay options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the starting version.
    pub fn from_version(mut self, version: u64) -> Self {
        self.from_version = version;
        self
    }

    /// Set the ending version.
    pub fn to_version(mut self, version: u64) -> Self {
        self.to_version = Some(version);
        self
    }

    /// Set the starting timestamp.
    pub fn from_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.from_timestamp = Some(timestamp);
        self
    }

    /// Set the ending timestamp.
    pub fn to_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.to_timestamp = Some(timestamp);
        self
    }

    /// Set the batch size.
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Check if an event matches the replay criteria.
    pub fn matches(&self, event: &StoredEvent) -> bool {
        // Check version range
        if event.metadata.sequence < self.from_version {
            return false;
        }
        if let Some(to) = self.to_version {
            if event.metadata.sequence > to {
                return false;
            }
        }

        // Check timestamp range
        if let Some(from) = self.from_timestamp {
            if event.metadata.timestamp < from {
                return false;
            }
        }
        if let Some(to) = self.to_timestamp {
            if event.metadata.timestamp > to {
                return false;
            }
        }

        true
    }
}

/// Trait for handling replayed events.
#[async_trait]
pub trait ReplayHandler: Send + Sync {
    /// Handle a replayed event.
    async fn handle(&self, event: &StoredEvent) -> Result<()>;

    /// Called before replay starts.
    async fn before_replay(&self) -> Result<()> {
        Ok(())
    }

    /// Called after replay completes.
    async fn after_replay(&self, events_replayed: usize) -> Result<()> {
        let _ = events_replayed;
        Ok(())
    }
}

/// Event replay service.
pub struct ReplayService<S>
where
    S: EventStore,
{
    event_store: S,
}

impl<S> ReplayService<S>
where
    S: EventStore,
{
    /// Create a new replay service.
    pub fn new(event_store: S) -> Self {
        Self { event_store }
    }

    /// Replay events from a stream.
    pub async fn replay_stream(
        &self,
        stream_id: &StreamId,
        options: ReplayOptions,
        handler: &dyn ReplayHandler,
    ) -> Result<ReplayResult> {
        handler.before_replay().await?;

        let events = self
            .event_store
            .read(
                stream_id,
                ReadOptions {
                    from_version: options.from_version,
                    max_count: None,
                    direction: crate::store::ReadDirection::Forward,
                },
            )
            .await?;

        let mut events_replayed = 0;
        let mut events_skipped = 0;

        for event in events {
            if options.matches(&event) {
                handler.handle(&event).await?;
                events_replayed += 1;
            } else {
                events_skipped += 1;
            }
        }

        handler.after_replay(events_replayed).await?;

        Ok(ReplayResult {
            events_replayed,
            events_skipped,
        })
    }

    /// Replay all events across all streams.
    pub async fn replay_all(
        &self,
        options: ReplayOptions,
        handler: &dyn ReplayHandler,
    ) -> Result<ReplayResult> {
        handler.before_replay().await?;

        let mut position = options.from_version;
        let mut events_replayed = 0;
        let mut events_skipped = 0;

        loop {
            let events = self
                .event_store
                .read_all(position, options.batch_size)
                .await?;

            if events.is_empty() {
                break;
            }

            for event in &events {
                if options.matches(event) {
                    handler.handle(event).await?;
                    events_replayed += 1;
                } else {
                    events_skipped += 1;
                }

                position = event.metadata.sequence;
            }

            if events.len() < options.batch_size {
                break;
            }

            position += 1;
        }

        handler.after_replay(events_replayed).await?;

        Ok(ReplayResult {
            events_replayed,
            events_skipped,
        })
    }

    /// Reconstruct an aggregate from its event history.
    pub async fn reconstruct_aggregate<A>(
        &self,
        id: &AggregateId,
    ) -> Result<Aggregate<A>>
    where
        A: AggregateRoot,
        A::Event: for<'de> Deserialize<'de>,
    {
        let stream_id = id.to_stream_id(A::aggregate_type());

        let events = self
            .event_store
            .read(&stream_id, ReadOptions::default())
            .await?;

        let mut domain_events = Vec::new();
        for stored in events {
            let event: A::Event = stored.deserialize_payload()?;
            domain_events.push(event);
        }

        Aggregate::from_history(id.clone(), domain_events)
    }
}

/// Result of a replay operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    /// Number of events replayed
    pub events_replayed: usize,
    /// Number of events skipped
    pub events_skipped: usize,
}

impl ReplayResult {
    /// Get total events processed.
    pub fn total_events(&self) -> usize {
        self.events_replayed + self.events_skipped
    }
}

/// Replay handler that collects statistics.
pub struct StatisticsReplayHandler {
    stats: std::sync::Arc<parking_lot::RwLock<ReplayStatistics>>,
}

impl Default for StatisticsReplayHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl StatisticsReplayHandler {
    /// Create a new statistics handler.
    pub fn new() -> Self {
        Self {
            stats: std::sync::Arc::new(parking_lot::RwLock::new(ReplayStatistics::default())),
        }
    }

    /// Get the collected statistics.
    pub fn statistics(&self) -> ReplayStatistics {
        self.stats.read().clone()
    }
}

#[async_trait]
impl ReplayHandler for StatisticsReplayHandler {
    async fn handle(&self, event: &StoredEvent) -> Result<()> {
        let mut stats = self.stats.write();
        stats.total_events += 1;

        let event_type = &event.metadata.event_version.event_type;
        *stats.events_by_type.entry(event_type.clone()).or_insert(0) += 1;

        Ok(())
    }

    async fn before_replay(&self) -> Result<()> {
        let mut stats = self.stats.write();
        stats.start_time = Some(Utc::now());
        Ok(())
    }

    async fn after_replay(&self, _events_replayed: usize) -> Result<()> {
        let mut stats = self.stats.write();
        stats.end_time = Some(Utc::now());
        Ok(())
    }
}

/// Statistics collected during replay.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayStatistics {
    /// Total number of events
    pub total_events: usize,
    /// Events by type
    pub events_by_type: HashMap<String, usize>,
    /// Start time
    pub start_time: Option<DateTime<Utc>>,
    /// End time
    pub end_time: Option<DateTime<Utc>>,
}

impl ReplayStatistics {
    /// Get the duration of the replay.
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Get events per second.
    pub fn events_per_second(&self) -> Option<f64> {
        self.duration().map(|d| {
            let seconds = d.num_milliseconds() as f64 / 1000.0;
            if seconds > 0.0 {
                self.total_events as f64 / seconds
            } else {
                0.0
            }
        })
    }
}

/// Replay handler for debugging (logs events).
pub struct DebugReplayHandler;

#[async_trait]
impl ReplayHandler for DebugReplayHandler {
    async fn handle(&self, event: &StoredEvent) -> Result<()> {
        tracing::debug!(
            stream = %event.metadata.stream_id,
            sequence = event.metadata.sequence,
            event_type = %event.metadata.event_version.event_type,
            timestamp = %event.metadata.timestamp,
            "Replaying event"
        );
        Ok(())
    }

    async fn before_replay(&self) -> Result<()> {
        tracing::info!("Starting event replay");
        Ok(())
    }

    async fn after_replay(&self, events_replayed: usize) -> Result<()> {
        tracing::info!(events = events_replayed, "Completed event replay");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventMetadata, EventVersion};
    use crate::store::InMemoryEventStore;

    #[test]
    fn test_replay_options() {
        let options = ReplayOptions::new()
            .from_version(5)
            .to_version(10)
            .batch_size(50);

        assert_eq!(options.from_version, 5);
        assert_eq!(options.to_version, Some(10));
        assert_eq!(options.batch_size, 50);
    }

    #[test]
    fn test_replay_options_matching() {
        let options = ReplayOptions::new().from_version(5).to_version(10);

        let event1 = StoredEvent::new(
            EventMetadata::new(
                StreamId::new("test"),
                3,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            vec![],
        );
        assert!(!options.matches(&event1));

        let event2 = StoredEvent::new(
            EventMetadata::new(
                StreamId::new("test"),
                7,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            vec![],
        );
        assert!(options.matches(&event2));

        let event3 = StoredEvent::new(
            EventMetadata::new(
                StreamId::new("test"),
                15,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            vec![],
        );
        assert!(!options.matches(&event3));
    }

    #[tokio::test]
    async fn test_statistics_handler() {
        let handler = StatisticsReplayHandler::new();

        handler.before_replay().await.unwrap();

        let event = StoredEvent::new(
            EventMetadata::new(
                StreamId::new("test"),
                1,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            vec![],
        );

        handler.handle(&event).await.unwrap();
        handler.handle(&event).await.unwrap();

        handler.after_replay(2).await.unwrap();

        let stats = handler.statistics();
        assert_eq!(stats.total_events, 2);
        assert!(stats.start_time.is_some());
        assert!(stats.end_time.is_some());
    }

    #[tokio::test]
    async fn test_replay_service() {
        let store = InMemoryEventStore::new();
        let service = ReplayService::new(store);

        let handler = StatisticsReplayHandler::new();
        let result = service
            .replay_all(ReplayOptions::default(), &handler)
            .await
            .unwrap();

        assert_eq!(result.events_replayed, 0);
        assert_eq!(result.events_skipped, 0);
    }
}
