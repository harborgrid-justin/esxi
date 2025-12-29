//! Event sourcing for audit trails and state replay

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;
use crate::streaming::Stream;

/// Event type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Feature created
    FeatureCreated,

    /// Feature updated
    FeatureUpdated,

    /// Feature deleted
    FeatureDeleted,

    /// Layer created
    LayerCreated,

    /// Layer updated
    LayerUpdated,

    /// Layer deleted
    LayerDeleted,

    /// User joined
    UserJoined,

    /// User left
    UserLeft,

    /// Permission changed
    PermissionChanged,

    /// Custom event
    Custom(String),
}

/// Event severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum EventSeverity {
    /// Debug level
    Debug,

    /// Info level
    Info,

    /// Warning level
    Warning,

    /// Error level
    Error,

    /// Critical level
    Critical,
}

/// Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event ID
    pub id: String,

    /// Event type
    pub event_type: EventType,

    /// Event severity
    pub severity: EventSeverity,

    /// User ID who triggered the event
    pub user_id: Option<String>,

    /// Entity ID (feature, layer, etc.)
    pub entity_id: Option<String>,

    /// Entity type
    pub entity_type: Option<String>,

    /// Event data
    pub data: serde_json::Value,

    /// Previous state (for updates)
    pub previous_state: Option<serde_json::Value>,

    /// New state (for updates)
    pub new_state: Option<serde_json::Value>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Tags
    pub tags: Vec<String>,

    /// Metadata
    pub metadata: serde_json::Value,
}

impl Event {
    /// Create new event
    pub fn new(event_type: EventType, severity: EventSeverity) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            severity,
            user_id: None,
            entity_id: None,
            entity_type: None,
            data: serde_json::json!({}),
            previous_state: None,
            new_state: None,
            timestamp: Utc::now(),
            tags: vec![],
            metadata: serde_json::json!({}),
        }
    }

    /// With user ID
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// With entity
    pub fn with_entity(mut self, entity_id: String, entity_type: String) -> Self {
        self.entity_id = Some(entity_id);
        self.entity_type = Some(entity_type);
        self
    }

    /// With data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// With state change
    pub fn with_state_change(
        mut self,
        previous: serde_json::Value,
        new: serde_json::Value,
    ) -> Self {
        self.previous_state = Some(previous);
        self.new_state = Some(new);
        self
    }

    /// Add tag
    pub fn add_tag(mut self, tag: String) -> Self {
        self.tags.push(tag);
        self
    }
}

/// Event stream
pub struct EventStream {
    /// Broadcast channel
    tx: broadcast::Sender<Event>,

    /// Message counter
    message_count: Arc<AtomicU64>,
}

impl EventStream {
    /// Create new event stream
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);

        Self {
            tx,
            message_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get message count
    pub fn message_count(&self) -> u64 {
        self.message_count.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl Stream for EventStream {
    type Item = Event;

    fn subscribe(&self) -> broadcast::Receiver<Self::Item> {
        self.tx.subscribe()
    }

    async fn publish(&self, item: Self::Item) -> Result<()> {
        self.tx
            .send(item)
            .map_err(|e| crate::error::Error::Internal(format!("Failed to publish event: {}", e)))?;

        self.message_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

/// Event store for persistence and replay
pub struct EventStore {
    /// All events
    events: Arc<parking_lot::RwLock<Vec<Event>>>,

    /// Stream
    stream: Arc<EventStream>,

    /// Maximum events to keep in memory
    max_events: usize,
}

impl EventStore {
    /// Create new event store
    pub fn new(stream_capacity: usize, max_events: usize) -> Self {
        Self {
            events: Arc::new(parking_lot::RwLock::new(Vec::new())),
            stream: Arc::new(EventStream::new(stream_capacity)),
            max_events,
        }
    }

    /// Append event
    pub async fn append(&self, event: Event) -> Result<()> {
        // Publish to stream
        self.stream.publish(event.clone()).await?;

        // Store event
        let mut events = self.events.write();
        events.push(event);

        // Trim if necessary
        if events.len() > self.max_events {
            let excess = events.len() - self.max_events;
            events.drain(0..excess);
        }

        Ok(())
    }

    /// Get all events
    pub fn get_all_events(&self) -> Vec<Event> {
        self.events.read().clone()
    }

    /// Get events by type
    pub fn get_events_by_type(&self, event_type: EventType) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get events by user
    pub fn get_events_by_user(&self, user_id: &str) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| e.user_id.as_ref().map_or(false, |id| id == user_id))
            .cloned()
            .collect()
    }

    /// Get events by entity
    pub fn get_events_by_entity(&self, entity_id: &str) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| e.entity_id.as_ref().map_or(false, |id| id == entity_id))
            .cloned()
            .collect()
    }

    /// Get events by severity
    pub fn get_events_by_severity(&self, severity: EventSeverity) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| e.severity == severity)
            .cloned()
            .collect()
    }

    /// Get events by tag
    pub fn get_events_by_tag(&self, tag: &str) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| e.tags.iter().any(|t| t == tag))
            .cloned()
            .collect()
    }

    /// Get events in time range
    pub fn get_events_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<Event> {
        self.events
            .read()
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get latest N events
    pub fn get_latest_events(&self, count: usize) -> Vec<Event> {
        let events = self.events.read();
        let start = events.len().saturating_sub(count);
        events[start..].to_vec()
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.events.read().len()
    }

    /// Get stream
    pub fn stream(&self) -> &EventStream {
        &self.stream
    }

    /// Clear all events
    pub fn clear(&self) {
        self.events.write().clear();
    }

    /// Get event statistics
    pub fn get_statistics(&self) -> EventStatistics {
        let events = self.events.read();

        let mut stats = EventStatistics::default();
        stats.total_events = events.len();

        for event in events.iter() {
            match event.severity {
                EventSeverity::Debug => stats.debug_count += 1,
                EventSeverity::Info => stats.info_count += 1,
                EventSeverity::Warning => stats.warning_count += 1,
                EventSeverity::Error => stats.error_count += 1,
                EventSeverity::Critical => stats.critical_count += 1,
            }
        }

        stats
    }
}

/// Event statistics
#[derive(Debug, Clone, Default)]
pub struct EventStatistics {
    /// Total events
    pub total_events: usize,

    /// Debug events
    pub debug_count: usize,

    /// Info events
    pub info_count: usize,

    /// Warning events
    pub warning_count: usize,

    /// Error events
    pub error_count: usize,

    /// Critical events
    pub critical_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new(EventType::FeatureCreated, EventSeverity::Info)
            .with_user("user1".to_string())
            .with_entity("feature123".to_string(), "Feature".to_string())
            .add_tag("important".to_string());

        assert_eq!(event.event_type, EventType::FeatureCreated);
        assert_eq!(event.severity, EventSeverity::Info);
        assert_eq!(event.user_id, Some("user1".to_string()));
        assert_eq!(event.tags.len(), 1);
    }

    #[tokio::test]
    async fn test_event_stream() {
        let stream = EventStream::new(100);
        let mut rx = stream.subscribe();

        let event = Event::new(EventType::FeatureCreated, EventSeverity::Info);
        stream.publish(event.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.event_type, EventType::FeatureCreated);
        assert_eq!(stream.message_count(), 1);
    }

    #[tokio::test]
    async fn test_event_store() {
        let store = EventStore::new(100, 1000);

        let event1 = Event::new(EventType::FeatureCreated, EventSeverity::Info)
            .with_user("user1".to_string());
        let event2 = Event::new(EventType::FeatureUpdated, EventSeverity::Info)
            .with_user("user1".to_string());
        let event3 = Event::new(EventType::FeatureCreated, EventSeverity::Warning);

        store.append(event1).await.unwrap();
        store.append(event2).await.unwrap();
        store.append(event3).await.unwrap();

        assert_eq!(store.event_count(), 3);

        let user1_events = store.get_events_by_user("user1");
        assert_eq!(user1_events.len(), 2);

        let created_events = store.get_events_by_type(EventType::FeatureCreated);
        assert_eq!(created_events.len(), 2);

        let stats = store.get_statistics();
        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.info_count, 2);
        assert_eq!(stats.warning_count, 1);
    }
}
