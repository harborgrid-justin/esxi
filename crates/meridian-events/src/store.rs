//! Event store implementation with append-only log semantics.

use crate::error::{EventError, Result};
use crate::event::{EventEnvelope, StoredEvent, StreamId};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Expected version for optimistic concurrency control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedVersion {
    /// Any version is acceptable (no concurrency check)
    Any,
    /// Stream must not exist
    NoStream,
    /// Stream must exist
    StreamExists,
    /// Specific version expected
    Exact(u64),
}

impl ExpectedVersion {
    /// Check if the expected version matches the actual version.
    pub fn matches(&self, actual: Option<u64>) -> bool {
        match (self, actual) {
            (ExpectedVersion::Any, _) => true,
            (ExpectedVersion::NoStream, None) => true,
            (ExpectedVersion::NoStream, Some(_)) => false,
            (ExpectedVersion::StreamExists, Some(_)) => true,
            (ExpectedVersion::StreamExists, None) => false,
            (ExpectedVersion::Exact(expected), Some(actual)) => expected == &actual,
            (ExpectedVersion::Exact(_), None) => false,
        }
    }
}

/// Options for reading events from a stream.
#[derive(Debug, Clone)]
pub struct ReadOptions {
    /// Start reading from this version (inclusive)
    pub from_version: u64,
    /// Maximum number of events to read
    pub max_count: Option<usize>,
    /// Read direction
    pub direction: ReadDirection,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            from_version: 0,
            max_count: None,
            direction: ReadDirection::Forward,
        }
    }
}

/// Direction for reading events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadDirection {
    /// Read forward from the start version
    Forward,
    /// Read backward from the start version
    Backward,
}

/// Result of appending events.
#[derive(Debug, Clone)]
pub struct AppendResult {
    /// Stream ID
    pub stream_id: StreamId,
    /// New version after append
    pub new_version: u64,
    /// Number of events appended
    pub events_appended: usize,
}

/// Trait for event store implementations.
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Append events to a stream.
    async fn append(
        &self,
        stream_id: &StreamId,
        expected_version: ExpectedVersion,
        events: Vec<EventEnvelope>,
    ) -> Result<AppendResult>;

    /// Read events from a stream.
    async fn read(
        &self,
        stream_id: &StreamId,
        options: ReadOptions,
    ) -> Result<Vec<StoredEvent>>;

    /// Get the current version of a stream.
    async fn get_version(&self, stream_id: &StreamId) -> Result<Option<u64>>;

    /// Check if a stream exists.
    async fn stream_exists(&self, stream_id: &StreamId) -> Result<bool>;

    /// Delete a stream (soft delete - marks as deleted).
    async fn delete_stream(&self, stream_id: &StreamId) -> Result<()>;

    /// List all streams.
    async fn list_streams(&self) -> Result<Vec<StreamId>>;

    /// Get global event sequence for subscriptions.
    async fn read_all(&self, from_position: u64, max_count: usize) -> Result<Vec<StoredEvent>>;
}

/// In-memory event store implementation (for testing and development).
#[derive(Debug)]
pub struct InMemoryEventStore {
    /// Streams indexed by stream ID
    streams: Arc<RwLock<HashMap<StreamId, StreamData>>>,
    /// Global event log for subscriptions
    global_log: Arc<RwLock<Vec<StoredEvent>>>,
}

#[derive(Debug, Clone)]
struct StreamData {
    events: Vec<StoredEvent>,
    version: u64,
    deleted: bool,
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryEventStore {
    /// Create a new in-memory event store.
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            global_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get stream data (internal helper).
    fn get_stream_data(&self, stream_id: &StreamId) -> Option<StreamData> {
        self.streams.read().get(stream_id).cloned()
    }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append(
        &self,
        stream_id: &StreamId,
        expected_version: ExpectedVersion,
        events: Vec<EventEnvelope>,
    ) -> Result<AppendResult> {
        if events.is_empty() {
            return Ok(AppendResult {
                stream_id: stream_id.clone(),
                new_version: self.get_version(stream_id).await?.unwrap_or(0),
                events_appended: 0,
            });
        }

        let mut streams = self.streams.write();
        let stream_data = streams.get_mut(stream_id);

        // Check expected version
        let current_version = stream_data.as_ref().map(|s| s.version);
        if !expected_version.matches(current_version) {
            return Err(EventError::ConcurrencyConflict {
                expected: match expected_version {
                    ExpectedVersion::Exact(v) => v,
                    ExpectedVersion::NoStream => 0,
                    ExpectedVersion::StreamExists => current_version.unwrap_or(0),
                    ExpectedVersion::Any => current_version.unwrap_or(0),
                },
                actual: current_version.unwrap_or(0),
            });
        }

        // Convert envelopes to stored events
        let mut stored_events = Vec::new();
        let mut next_version = current_version.unwrap_or(0) + 1;

        for envelope in events {
            let payload = serde_json::to_vec(&envelope.payload)
                .map_err(|e| EventError::Serialization(e.to_string()))?;

            let mut metadata = envelope.metadata;
            metadata.sequence = next_version;

            let stored_event = StoredEvent::new(metadata, payload);
            stored_events.push(stored_event.clone());

            // Add to global log
            self.global_log.write().push(stored_event);

            next_version += 1;
        }

        let new_version = next_version - 1;
        let events_appended = stored_events.len();

        // Update or create stream
        if let Some(data) = stream_data {
            data.events.extend(stored_events);
            data.version = new_version;
        } else {
            streams.insert(
                stream_id.clone(),
                StreamData {
                    events: stored_events,
                    version: new_version,
                    deleted: false,
                },
            );
        }

        Ok(AppendResult {
            stream_id: stream_id.clone(),
            new_version,
            events_appended,
        })
    }

    async fn read(
        &self,
        stream_id: &StreamId,
        options: ReadOptions,
    ) -> Result<Vec<StoredEvent>> {
        let streams = self.streams.read();
        let stream_data = streams.get(stream_id).ok_or_else(|| {
            EventError::StreamNotFound(stream_id.to_string())
        })?;

        if stream_data.deleted {
            return Err(EventError::StreamNotFound(stream_id.to_string()));
        }

        let mut events: Vec<StoredEvent> = stream_data
            .events
            .iter()
            .filter(|e| match options.direction {
                ReadDirection::Forward => e.metadata.sequence >= options.from_version,
                ReadDirection::Backward => e.metadata.sequence <= options.from_version,
            })
            .cloned()
            .collect();

        if options.direction == ReadDirection::Backward {
            events.reverse();
        }

        if let Some(max_count) = options.max_count {
            events.truncate(max_count);
        }

        Ok(events)
    }

    async fn get_version(&self, stream_id: &StreamId) -> Result<Option<u64>> {
        Ok(self.get_stream_data(stream_id).map(|s| s.version))
    }

    async fn stream_exists(&self, stream_id: &StreamId) -> Result<bool> {
        Ok(self
            .get_stream_data(stream_id)
            .map(|s| !s.deleted)
            .unwrap_or(false))
    }

    async fn delete_stream(&self, stream_id: &StreamId) -> Result<()> {
        let mut streams = self.streams.write();
        if let Some(stream) = streams.get_mut(stream_id) {
            stream.deleted = true;
        }
        Ok(())
    }

    async fn list_streams(&self) -> Result<Vec<StreamId>> {
        let streams = self.streams.read();
        Ok(streams
            .iter()
            .filter(|(_, data)| !data.deleted)
            .map(|(id, _)| id.clone())
            .collect())
    }

    async fn read_all(&self, from_position: u64, max_count: usize) -> Result<Vec<StoredEvent>> {
        let global_log = self.global_log.read();
        Ok(global_log
            .iter()
            .skip(from_position as usize)
            .take(max_count)
            .cloned()
            .collect())
    }
}

#[cfg(feature = "rocksdb-backend")]
pub mod rocksdb_store {
    use super::*;
    use rocksdb::{Options, DB};
    use std::path::Path;

    /// RocksDB-based event store implementation.
    pub struct RocksDbEventStore {
        db: Arc<DB>,
    }

    impl RocksDbEventStore {
        /// Create a new RocksDB event store.
        pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
            let mut opts = Options::default();
            opts.create_if_missing(true);

            let db = DB::open(&opts, path)?;

            Ok(Self { db: Arc::new(db) })
        }

        fn stream_key(stream_id: &StreamId) -> Vec<u8> {
            format!("stream:{}", stream_id).into_bytes()
        }

        fn event_key(stream_id: &StreamId, sequence: u64) -> Vec<u8> {
            format!("event:{}:{:020}", stream_id, sequence).into_bytes()
        }

        fn version_key(stream_id: &StreamId) -> Vec<u8> {
            format!("version:{}", stream_id).into_bytes()
        }
    }

    #[async_trait]
    impl EventStore for RocksDbEventStore {
        async fn append(
            &self,
            stream_id: &StreamId,
            expected_version: ExpectedVersion,
            events: Vec<EventEnvelope>,
        ) -> Result<AppendResult> {
            if events.is_empty() {
                return Ok(AppendResult {
                    stream_id: stream_id.clone(),
                    new_version: self.get_version(stream_id).await?.unwrap_or(0),
                    events_appended: 0,
                });
            }

            // Get current version
            let current_version = self.get_version(stream_id).await?;

            // Check expected version
            if !expected_version.matches(current_version) {
                return Err(EventError::ConcurrencyConflict {
                    expected: match expected_version {
                        ExpectedVersion::Exact(v) => v,
                        ExpectedVersion::NoStream => 0,
                        ExpectedVersion::StreamExists => current_version.unwrap_or(0),
                        ExpectedVersion::Any => current_version.unwrap_or(0),
                    },
                    actual: current_version.unwrap_or(0),
                });
            }

            let mut next_version = current_version.unwrap_or(0) + 1;
            let mut events_appended = 0;

            for envelope in events {
                let payload = serde_json::to_vec(&envelope.payload)
                    .map_err(|e| EventError::Serialization(e.to_string()))?;

                let mut metadata = envelope.metadata;
                metadata.sequence = next_version;

                let stored_event = StoredEvent::new(metadata, payload);
                let serialized = bincode::serialize(&stored_event)?;

                self.db.put(
                    Self::event_key(stream_id, next_version),
                    serialized,
                )?;

                next_version += 1;
                events_appended += 1;
            }

            let new_version = next_version - 1;

            // Update version
            self.db.put(
                Self::version_key(stream_id),
                new_version.to_le_bytes(),
            )?;

            // Mark stream as existing
            self.db.put(Self::stream_key(stream_id), b"1")?;

            Ok(AppendResult {
                stream_id: stream_id.clone(),
                new_version,
                events_appended,
            })
        }

        async fn read(
            &self,
            stream_id: &StreamId,
            options: ReadOptions,
        ) -> Result<Vec<StoredEvent>> {
            let exists = self.stream_exists(stream_id).await?;
            if !exists {
                return Err(EventError::StreamNotFound(stream_id.to_string()));
            }

            let mut events = Vec::new();
            let version = self.get_version(stream_id).await?.unwrap_or(0);

            let range = if options.direction == ReadDirection::Forward {
                options.from_version..=version
            } else {
                0..=options.from_version
            };

            for seq in range {
                let key = Self::event_key(stream_id, seq);
                if let Some(data) = self.db.get(&key)? {
                    let event: StoredEvent = bincode::deserialize(&data)?;
                    events.push(event);

                    if let Some(max_count) = options.max_count {
                        if events.len() >= max_count {
                            break;
                        }
                    }
                }
            }

            if options.direction == ReadDirection::Backward {
                events.reverse();
            }

            Ok(events)
        }

        async fn get_version(&self, stream_id: &StreamId) -> Result<Option<u64>> {
            let key = Self::version_key(stream_id);
            match self.db.get(&key)? {
                Some(bytes) => {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(&bytes);
                    Ok(Some(u64::from_le_bytes(arr)))
                }
                None => Ok(None),
            }
        }

        async fn stream_exists(&self, stream_id: &StreamId) -> Result<bool> {
            let key = Self::stream_key(stream_id);
            Ok(self.db.get(&key)?.is_some())
        }

        async fn delete_stream(&self, stream_id: &StreamId) -> Result<()> {
            let key = Self::stream_key(stream_id);
            self.db.delete(&key)?;
            Ok(())
        }

        async fn list_streams(&self) -> Result<Vec<StreamId>> {
            let mut streams = Vec::new();
            let prefix = b"stream:";

            let iter = self.db.prefix_iterator(prefix);
            for item in iter {
                let (key, _) = item?;
                if let Ok(key_str) = std::str::from_utf8(&key) {
                    if let Some(stream_id) = key_str.strip_prefix("stream:") {
                        streams.push(StreamId::new(stream_id));
                    }
                }
            }

            Ok(streams)
        }

        async fn read_all(&self, from_position: u64, max_count: usize) -> Result<Vec<StoredEvent>> {
            // For RocksDB, we'd need a separate global log structure
            // This is a simplified implementation
            let streams = self.list_streams().await?;
            let mut all_events = Vec::new();

            for stream_id in streams {
                let events = self.read(&stream_id, ReadOptions::default()).await?;
                all_events.extend(events);
            }

            // Sort by sequence
            all_events.sort_by_key(|e| e.metadata.sequence);

            Ok(all_events
                .into_iter()
                .skip(from_position as usize)
                .take(max_count)
                .collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventMetadata, EventVersion, StreamId};

    #[tokio::test]
    async fn test_expected_version_matching() {
        assert!(ExpectedVersion::Any.matches(None));
        assert!(ExpectedVersion::Any.matches(Some(5)));
        assert!(ExpectedVersion::NoStream.matches(None));
        assert!(!ExpectedVersion::NoStream.matches(Some(1)));
        assert!(ExpectedVersion::Exact(5).matches(Some(5)));
        assert!(!ExpectedVersion::Exact(5).matches(Some(3)));
    }

    #[tokio::test]
    async fn test_in_memory_store_append() {
        let store = InMemoryEventStore::new();
        let stream_id = StreamId::new("test-stream");

        let event = EventEnvelope::new(
            EventMetadata::new(
                stream_id.clone(),
                0,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            serde_json::json!({"data": "test"}),
        );

        let result = store
            .append(&stream_id, ExpectedVersion::NoStream, vec![event])
            .await
            .unwrap();

        assert_eq!(result.new_version, 1);
        assert_eq!(result.events_appended, 1);
    }

    #[tokio::test]
    async fn test_concurrency_conflict() {
        let store = InMemoryEventStore::new();
        let stream_id = StreamId::new("test-stream");

        let event1 = EventEnvelope::new(
            EventMetadata::new(
                stream_id.clone(),
                0,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            serde_json::json!({"data": "test1"}),
        );

        store
            .append(&stream_id, ExpectedVersion::NoStream, vec![event1])
            .await
            .unwrap();

        let event2 = EventEnvelope::new(
            EventMetadata::new(
                stream_id.clone(),
                0,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            serde_json::json!({"data": "test2"}),
        );

        // This should fail because we expect version 0 but it's now 1
        let result = store
            .append(&stream_id, ExpectedVersion::Exact(0), vec![event2])
            .await;

        assert!(matches!(result, Err(EventError::ConcurrencyConflict { .. })));
    }

    #[tokio::test]
    async fn test_read_events() {
        let store = InMemoryEventStore::new();
        let stream_id = StreamId::new("test-stream");

        let events: Vec<_> = (0..5)
            .map(|i| {
                EventEnvelope::new(
                    EventMetadata::new(
                        stream_id.clone(),
                        0,
                        EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
                    ),
                    serde_json::json!({"data": i}),
                )
            })
            .collect();

        store
            .append(&stream_id, ExpectedVersion::NoStream, events)
            .await
            .unwrap();

        let read_events = store
            .read(&stream_id, ReadOptions::default())
            .await
            .unwrap();

        assert_eq!(read_events.len(), 5);
    }
}
