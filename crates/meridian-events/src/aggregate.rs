//! Aggregate root pattern for domain-driven design.

use crate::error::{EventError, Result};
use crate::event::{DomainEvent, EventEnvelope, EventMetadata, EventVersion, StoredEvent, StreamId};
use crate::store::ExpectedVersion;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Aggregate ID type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AggregateId(String);

impl AggregateId {
    /// Create a new aggregate ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to a stream ID.
    pub fn to_stream_id(&self, aggregate_type: &str) -> StreamId {
        StreamId::new(format!("{}-{}", aggregate_type, self.0))
    }
}

impl fmt::Display for AggregateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AggregateId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AggregateId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Trait for aggregate roots.
#[async_trait]
pub trait AggregateRoot: Send + Sync + Sized {
    /// The type of events this aggregate produces.
    type Event: DomainEvent + Clone;

    /// The aggregate type name.
    fn aggregate_type() -> &'static str;

    /// Get the aggregate ID.
    fn aggregate_id(&self) -> &AggregateId;

    /// Get the current version.
    fn version(&self) -> u64;

    /// Apply an event to the aggregate.
    fn apply(&mut self, event: &Self::Event) -> Result<()>;

    /// Create a new aggregate from an ID.
    fn new(id: AggregateId) -> Self;

    /// Validate the aggregate state.
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Container for an aggregate with its uncommitted events.
#[derive(Debug)]
pub struct Aggregate<A: AggregateRoot> {
    /// The aggregate root
    root: A,
    /// Uncommitted events
    uncommitted_events: Vec<A::Event>,
    /// Original version when loaded
    original_version: u64,
}

impl<A: AggregateRoot> Aggregate<A> {
    /// Create a new aggregate.
    pub fn new(id: AggregateId) -> Self {
        Self {
            root: A::new(id),
            uncommitted_events: Vec::new(),
            original_version: 0,
        }
    }

    /// Get a reference to the aggregate root.
    pub fn root(&self) -> &A {
        &self.root
    }

    /// Get a mutable reference to the aggregate root.
    pub fn root_mut(&mut self) -> &mut A {
        &mut self.root
    }

    /// Apply an event and add it to uncommitted events.
    pub fn apply_new(&mut self, event: A::Event) -> Result<()> {
        self.root.apply(&event)?;
        self.uncommitted_events.push(event);
        Ok(())
    }

    /// Get uncommitted events.
    pub fn uncommitted_events(&self) -> &[A::Event] {
        &self.uncommitted_events
    }

    /// Clear uncommitted events (after persisting).
    pub fn commit(&mut self) {
        self.original_version = self.root.version();
        self.uncommitted_events.clear();
    }

    /// Get the original version.
    pub fn original_version(&self) -> u64 {
        self.original_version
    }

    /// Load aggregate from event history.
    pub fn from_history(id: AggregateId, events: Vec<A::Event>) -> Result<Self> {
        let mut aggregate = Self::new(id);

        for event in events {
            aggregate.root.apply(&event)?;
        }

        aggregate.original_version = aggregate.root.version();
        Ok(aggregate)
    }

    /// Get the stream ID for this aggregate.
    pub fn stream_id(&self) -> StreamId {
        self.root.aggregate_id().to_stream_id(A::aggregate_type())
    }

    /// Validate the aggregate state.
    pub fn validate(&self) -> Result<()> {
        self.root.validate()
    }
}

/// Repository for loading and saving aggregates.
#[async_trait]
pub trait AggregateRepository<A: AggregateRoot>: Send + Sync {
    /// Load an aggregate by ID.
    async fn load(&self, id: &AggregateId) -> Result<Aggregate<A>>;

    /// Save an aggregate.
    async fn save(&self, aggregate: &mut Aggregate<A>) -> Result<()>;

    /// Check if an aggregate exists.
    async fn exists(&self, id: &AggregateId) -> Result<bool>;
}

/// Base implementation of aggregate repository using event store.
pub struct EventSourcedRepository<A: AggregateRoot, S> {
    event_store: S,
    _phantom: std::marker::PhantomData<A>,
}

impl<A: AggregateRoot, S> EventSourcedRepository<A, S> {
    /// Create a new repository.
    pub fn new(event_store: S) -> Self {
        Self {
            event_store,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<A, S> AggregateRepository<A> for EventSourcedRepository<A, S>
where
    A: AggregateRoot,
    A::Event: for<'de> Deserialize<'de> + Serialize,
    S: crate::store::EventStore + Send + Sync,
{
    async fn load(&self, id: &AggregateId) -> Result<Aggregate<A>> {
        let stream_id = id.to_stream_id(A::aggregate_type());

        // Read all events from the stream
        let stored_events = self
            .event_store
            .read(
                &stream_id,
                crate::store::ReadOptions::default(),
            )
            .await?;

        if stored_events.is_empty() {
            return Err(EventError::AggregateNotFound(id.to_string()));
        }

        // Deserialize events
        let mut events = Vec::new();
        for stored in stored_events {
            let event: A::Event = stored.deserialize_payload()?;
            events.push(event);
        }

        // Reconstruct aggregate from history
        Aggregate::from_history(id.clone(), events)
    }

    async fn save(&self, aggregate: &mut Aggregate<A>) -> Result<()> {
        let uncommitted = aggregate.uncommitted_events();
        if uncommitted.is_empty() {
            return Ok(());
        }

        // Validate before saving
        aggregate.validate()?;

        let stream_id = aggregate.stream_id();

        // Convert events to envelopes
        let mut envelopes = Vec::new();
        for event in uncommitted {
            let event_version = EventVersion::new(
                event.event_type(),
                event.event_version(),
            );

            let metadata = EventMetadata::new(
                stream_id.clone(),
                0, // Will be set by the store
                event_version,
            );

            let payload = event.to_json()?;
            envelopes.push(EventEnvelope::new(metadata, payload));
        }

        // Append to event store with optimistic concurrency check
        let expected_version = if aggregate.original_version() == 0 {
            crate::store::ExpectedVersion::NoStream
        } else {
            crate::store::ExpectedVersion::Exact(aggregate.original_version())
        };

        self.event_store
            .append(&stream_id, expected_version, envelopes)
            .await?;

        // Commit the aggregate
        aggregate.commit();

        Ok(())
    }

    async fn exists(&self, id: &AggregateId) -> Result<bool> {
        let stream_id = id.to_stream_id(A::aggregate_type());
        self.event_store.stream_exists(&stream_id).await
    }
}

/// Snapshot-aware repository that uses snapshots for performance.
pub struct SnapshotRepository<A, S, SS>
where
    A: AggregateRoot,
    S: crate::store::EventStore,
    SS: crate::snapshot::SnapshotStore<A>,
{
    event_store: S,
    snapshot_store: SS,
    snapshot_frequency: u64,
    _phantom: std::marker::PhantomData<A>,
}

impl<A, S, SS> SnapshotRepository<A, S, SS>
where
    A: AggregateRoot,
    S: crate::store::EventStore,
    SS: crate::snapshot::SnapshotStore<A>,
{
    /// Create a new snapshot repository.
    pub fn new(event_store: S, snapshot_store: SS, snapshot_frequency: u64) -> Self {
        Self {
            event_store,
            snapshot_store,
            snapshot_frequency,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the snapshot frequency.
    pub fn snapshot_frequency(&self) -> u64 {
        self.snapshot_frequency
    }
}

#[async_trait]
impl<A, S, SS> AggregateRepository<A> for SnapshotRepository<A, S, SS>
where
    A: AggregateRoot + Clone + for<'de> Deserialize<'de> + Serialize,
    A::Event: for<'de> Deserialize<'de> + Serialize,
    S: crate::store::EventStore + Send + Sync,
    SS: crate::snapshot::SnapshotStore<A> + Send + Sync,
{
    async fn load(&self, id: &AggregateId) -> Result<Aggregate<A>> {
        let stream_id = id.to_stream_id(A::aggregate_type());

        // Try to load from snapshot
        let (mut aggregate, from_version) = match self.snapshot_store.load(id).await {
            Ok(snapshot) => {
                let version = snapshot.version();
                (Aggregate::<A>::from_history(id.clone(), vec![])?, version)
            }
            Err(_) => (Aggregate::new(id.clone()), 0),
        };

        // Load events after snapshot
        let stored_events = self
            .event_store
            .read(
                &stream_id,
                crate::store::ReadOptions {
                    from_version: from_version + 1,
                    max_count: None,
                    direction: crate::store::ReadDirection::Forward,
                },
            )
            .await?;

        if from_version == 0 && stored_events.is_empty() {
            return Err(EventError::AggregateNotFound(id.to_string()));
        }

        // Apply events
        for stored in stored_events {
            let event: A::Event = stored.deserialize_payload()?;
            aggregate.root.apply(&event)?;
        }

        aggregate.original_version = aggregate.root.version();
        Ok(aggregate)
    }

    async fn save(&self, aggregate: &mut Aggregate<A>) -> Result<()> {
        // Save events first
        let uncommitted = aggregate.uncommitted_events();
        if !uncommitted.is_empty() {
            // Validate before saving
            aggregate.validate()?;

            let stream_id = aggregate.stream_id();

            // Convert events to envelopes
            let mut envelopes = Vec::new();
            for event in uncommitted {
                let event_version = EventVersion::new(
                    event.event_type(),
                    event.event_version(),
                );

                let metadata = EventMetadata::new(
                    stream_id.clone(),
                    0, // Will be set by the store
                    event_version,
                );

                let payload = event.to_json()?;
                envelopes.push(EventEnvelope::new(metadata, payload));
            }

            // Append to event store with optimistic concurrency check
            let expected_version = if aggregate.original_version() == 0 {
                ExpectedVersion::NoStream
            } else {
                ExpectedVersion::Exact(aggregate.original_version())
            };

            self.event_store
                .append(&stream_id, expected_version, envelopes)
                .await?;

            aggregate.commit();
        }

        // Check if we should create a snapshot
        let version = aggregate.root().version();
        if version % self.snapshot_frequency == 0 {
            self.snapshot_store.save(aggregate.root()).await?;
        }

        Ok(())
    }

    async fn exists(&self, id: &AggregateId) -> Result<bool> {
        let stream_id = id.to_stream_id(A::aggregate_type());
        self.event_store.stream_exists(&stream_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        value: i32,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &str {
            "TestEvent"
        }

        fn event_version(&self) -> semver::Version {
            semver::Version::new(1, 0, 0)
        }

        fn to_json(&self) -> Result<serde_json::Value> {
            Ok(serde_json::to_value(self)
                .map_err(|e| EventError::Serialization(e.to_string()))?)
        }

        fn from_json(value: serde_json::Value) -> Result<Self> {
            Ok(serde_json::from_value(value)
                .map_err(|e| EventError::Deserialization(e.to_string()))?)
        }
    }

    #[derive(Debug)]
    struct TestAggregate {
        id: AggregateId,
        version: u64,
        total: i32,
    }

    #[async_trait]
    impl AggregateRoot for TestAggregate {
        type Event = TestEvent;

        fn aggregate_type() -> &'static str {
            "TestAggregate"
        }

        fn aggregate_id(&self) -> &AggregateId {
            &self.id
        }

        fn version(&self) -> u64 {
            self.version
        }

        fn apply(&mut self, event: &Self::Event) -> Result<()> {
            self.total += event.value;
            self.version += 1;
            Ok(())
        }

        fn new(id: AggregateId) -> Self {
            Self {
                id,
                version: 0,
                total: 0,
            }
        }
    }

    #[test]
    fn test_aggregate_creation() {
        let id = AggregateId::new("test-1");
        let aggregate = Aggregate::<TestAggregate>::new(id);

        assert_eq!(aggregate.root().version(), 0);
        assert_eq!(aggregate.uncommitted_events().len(), 0);
    }

    #[test]
    fn test_aggregate_apply_events() {
        let id = AggregateId::new("test-1");
        let mut aggregate = Aggregate::<TestAggregate>::new(id);

        aggregate.apply_new(TestEvent { value: 10 }).unwrap();
        aggregate.apply_new(TestEvent { value: 20 }).unwrap();

        assert_eq!(aggregate.root().version(), 2);
        assert_eq!(aggregate.root().total, 30);
        assert_eq!(aggregate.uncommitted_events().len(), 2);
    }

    #[test]
    fn test_aggregate_from_history() {
        let id = AggregateId::new("test-1");
        let events = vec![
            TestEvent { value: 10 },
            TestEvent { value: 20 },
            TestEvent { value: 30 },
        ];

        let aggregate = Aggregate::<TestAggregate>::from_history(id, events).unwrap();

        assert_eq!(aggregate.root().version(), 3);
        assert_eq!(aggregate.root().total, 60);
        assert_eq!(aggregate.uncommitted_events().len(), 0);
    }
}
