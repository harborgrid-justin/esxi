//! Event types and versioning for event sourcing.

use crate::causation::{CausationId, CorrelationId};
use crate::error::{EventError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Event identifier (UUID v4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    /// Generate a new event ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for EventId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Stream identifier for grouping related events.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StreamId(String);

impl StreamId {
    /// Create a new stream ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StreamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for StreamId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for StreamId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Event version for schema evolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventVersion {
    /// Event type name
    pub event_type: String,
    /// Schema version (semantic versioning)
    pub version: semver::Version,
}

impl EventVersion {
    /// Create a new event version.
    pub fn new(event_type: impl Into<String>, version: semver::Version) -> Self {
        Self {
            event_type: event_type.into(),
            version,
        }
    }

    /// Check if this version is compatible with another.
    pub fn is_compatible_with(&self, other: &EventVersion) -> bool {
        if self.event_type != other.event_type {
            return false;
        }

        // Major version must match for compatibility
        self.version.major == other.version.major
    }
}

impl fmt::Display for EventVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.event_type, self.version)
    }
}

/// Metadata associated with an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Event ID
    pub event_id: EventId,
    /// Stream ID
    pub stream_id: StreamId,
    /// Sequence number within the stream
    pub sequence: u64,
    /// Event version
    pub event_version: EventVersion,
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
    /// Causation ID (what caused this event)
    pub causation_id: Option<CausationId>,
    /// Correlation ID (for tracking related events)
    pub correlation_id: Option<CorrelationId>,
    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

impl EventMetadata {
    /// Create new event metadata.
    pub fn new(
        stream_id: StreamId,
        sequence: u64,
        event_version: EventVersion,
    ) -> Self {
        Self {
            event_id: EventId::new(),
            stream_id,
            sequence,
            event_version,
            timestamp: Utc::now(),
            causation_id: None,
            correlation_id: None,
            custom: HashMap::new(),
        }
    }

    /// Set causation ID.
    pub fn with_causation(mut self, causation_id: CausationId) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    /// Set correlation ID.
    pub fn with_correlation(mut self, correlation_id: CorrelationId) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Add custom metadata.
    pub fn with_custom(mut self, key: String, value: serde_json::Value) -> Self {
        self.custom.insert(key, value);
        self
    }
}

/// Trait for domain events.
pub trait DomainEvent: Send + Sync + fmt::Debug {
    /// Get the event type name.
    fn event_type(&self) -> &str;

    /// Get the event version.
    fn event_version(&self) -> semver::Version;

    /// Serialize the event to JSON.
    fn to_json(&self) -> Result<serde_json::Value>;

    /// Deserialize the event from JSON.
    fn from_json(value: serde_json::Value) -> Result<Self>
    where
        Self: Sized;
}

/// Event envelope containing metadata and payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Event payload (serialized)
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    /// Create a new event envelope.
    pub fn new(metadata: EventMetadata, payload: serde_json::Value) -> Self {
        Self { metadata, payload }
    }

    /// Deserialize the payload into a specific event type.
    pub fn deserialize_payload<E: DomainEvent>(&self) -> Result<E> {
        E::from_json(self.payload.clone())
    }
}

/// Event store entry with serialized data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Serialized payload
    pub payload: Vec<u8>,
}

impl StoredEvent {
    /// Create a new stored event.
    pub fn new(metadata: EventMetadata, payload: Vec<u8>) -> Self {
        Self { metadata, payload }
    }

    /// Deserialize the payload.
    pub fn deserialize_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        serde_json::from_slice(&self.payload)
            .map_err(|e| EventError::Deserialization(e.to_string()))
    }

    /// Get the event as an envelope.
    pub fn to_envelope(&self) -> Result<EventEnvelope> {
        let payload: serde_json::Value = serde_json::from_slice(&self.payload)
            .map_err(|e| EventError::Deserialization(e.to_string()))?;
        Ok(EventEnvelope::new(self.metadata.clone(), payload))
    }
}

/// Event stream containing multiple events.
#[derive(Debug, Clone)]
pub struct EventStream {
    /// Stream ID
    pub stream_id: StreamId,
    /// Events in the stream
    pub events: Vec<StoredEvent>,
    /// Current version (sequence number of last event)
    pub version: u64,
}

impl EventStream {
    /// Create a new event stream.
    pub fn new(stream_id: StreamId) -> Self {
        Self {
            stream_id,
            events: Vec::new(),
            version: 0,
        }
    }

    /// Add an event to the stream.
    pub fn append(&mut self, event: StoredEvent) {
        self.version = event.metadata.sequence;
        self.events.push(event);
    }

    /// Get events after a specific version.
    pub fn events_after(&self, version: u64) -> Vec<&StoredEvent> {
        self.events
            .iter()
            .filter(|e| e.metadata.sequence > version)
            .collect()
    }
}

/// Schema evolution handler for upgrading events.
pub trait EventUpgrader: Send + Sync {
    /// Upgrade an event from one version to another.
    fn upgrade(
        &self,
        from_version: &EventVersion,
        to_version: &EventVersion,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value>;

    /// Check if this upgrader can handle the given version pair.
    fn can_upgrade(&self, from: &EventVersion, to: &EventVersion) -> bool;
}

/// Registry for event upgraders.
#[derive(Default)]
pub struct EventUpgraderRegistry {
    upgraders: Vec<Box<dyn EventUpgrader>>,
}

impl EventUpgraderRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        Self {
            upgraders: Vec::new(),
        }
    }

    /// Register an upgrader.
    pub fn register(&mut self, upgrader: Box<dyn EventUpgrader>) {
        self.upgraders.push(upgrader);
    }

    /// Upgrade an event to a target version.
    pub fn upgrade(
        &self,
        from_version: &EventVersion,
        to_version: &EventVersion,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Find an upgrader that can handle this version pair
        for upgrader in &self.upgraders {
            if upgrader.can_upgrade(from_version, to_version) {
                return upgrader.upgrade(from_version, to_version, payload);
            }
        }

        Err(EventError::SchemaEvolution(format!(
            "No upgrader found for {} -> {}",
            from_version, to_version
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id_generation() {
        let id1 = EventId::new();
        let id2 = EventId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_stream_id() {
        let stream_id = StreamId::new("test-stream");
        assert_eq!(stream_id.as_str(), "test-stream");
    }

    #[test]
    fn test_event_version_compatibility() {
        let v1 = EventVersion::new(
            "TestEvent",
            semver::Version::new(1, 0, 0),
        );
        let v2 = EventVersion::new(
            "TestEvent",
            semver::Version::new(1, 1, 0),
        );
        let v3 = EventVersion::new(
            "TestEvent",
            semver::Version::new(2, 0, 0),
        );

        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_event_metadata() {
        let stream_id = StreamId::new("test");
        let version = EventVersion::new("TestEvent", semver::Version::new(1, 0, 0));
        let metadata = EventMetadata::new(stream_id, 1, version);

        assert_eq!(metadata.sequence, 1);
        assert_eq!(metadata.stream_id.as_str(), "test");
    }
}
