//! Message types for real-time streaming.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a client connection.
pub type ClientId = Uuid;

/// Unique identifier for a channel.
pub type ChannelId = String;

/// Unique identifier for a room.
pub type RoomId = String;

/// Unique identifier for a feature.
pub type FeatureId = String;

/// Unique identifier for a layer.
pub type LayerId = String;

/// Timestamp in milliseconds since Unix epoch.
pub type Timestamp = u64;

/// Top-level streaming message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamMessage {
    /// Feature update message
    FeatureUpdate(FeatureUpdateMessage),

    /// Layer update message
    LayerUpdate(LayerUpdateMessage),

    /// Viewport update message
    ViewportUpdate(ViewportUpdateMessage),

    /// Presence update message
    PresenceUpdate(PresenceUpdateMessage),

    /// Room message
    Room(RoomMessage),

    /// Channel subscription message
    Subscribe(SubscribeMessage),

    /// Channel unsubscribe message
    Unsubscribe(UnsubscribeMessage),

    /// Sync message
    Sync(SyncMessage),

    /// Ping message for keepalive
    Ping { timestamp: Timestamp },

    /// Pong response to ping
    Pong { timestamp: Timestamp },

    /// Error message
    Error(ErrorMessage),

    /// Custom message
    Custom {
        channel: ChannelId,
        data: serde_json::Value,
    },
}

/// Feature update operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureOperation {
    /// Feature created
    Create,
    /// Feature updated
    Update,
    /// Feature deleted
    Delete,
}

/// Feature update message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureUpdateMessage {
    /// Feature identifier
    pub feature_id: FeatureId,

    /// Layer identifier
    pub layer_id: LayerId,

    /// Operation type
    pub operation: FeatureOperation,

    /// Feature data (GeoJSON or custom format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Client that initiated the update
    pub client_id: ClientId,

    /// Timestamp of the update
    pub timestamp: Timestamp,

    /// Version number for conflict resolution
    pub version: u64,

    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Layer update message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerUpdateMessage {
    /// Layer identifier
    pub layer_id: LayerId,

    /// Operation type
    pub operation: LayerOperation,

    /// Layer configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,

    /// Client that initiated the update
    pub client_id: ClientId,

    /// Timestamp of the update
    pub timestamp: Timestamp,
}

/// Layer operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerOperation {
    /// Layer created
    Create,
    /// Layer updated
    Update,
    /// Layer deleted
    Delete,
    /// Layer visibility changed
    VisibilityChanged { visible: bool },
    /// Layer style changed
    StyleChanged,
}

/// Viewport update message for spatial subscriptions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportUpdateMessage {
    /// Client identifier
    pub client_id: ClientId,

    /// Viewport bounds [min_lon, min_lat, max_lon, max_lat]
    pub bounds: [f64; 4],

    /// Zoom level
    pub zoom: f64,

    /// Center point [lon, lat]
    pub center: [f64; 2],

    /// Timestamp of the update
    pub timestamp: Timestamp,

    /// Layers to subscribe to in this viewport
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layers: Option<Vec<LayerId>>,
}

/// Presence update message for user awareness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdateMessage {
    /// Client identifier
    pub client_id: ClientId,

    /// User identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// User display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,

    /// Presence status
    pub status: PresenceStatus,

    /// Current cursor position [lon, lat]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<[f64; 2]>,

    /// Current viewport
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<[f64; 4]>,

    /// Custom presence data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Timestamp of the update
    pub timestamp: Timestamp,
}

/// Presence status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    /// User is online
    Online,
    /// User is idle
    Idle,
    /// User is offline
    Offline,
}

/// Room-related messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RoomMessage {
    /// Join a room
    Join {
        room_id: RoomId,
        client_id: ClientId,
        user_data: Option<serde_json::Value>,
    },

    /// Leave a room
    Leave {
        room_id: RoomId,
        client_id: ClientId,
    },

    /// Room state update
    StateUpdate {
        room_id: RoomId,
        state: serde_json::Value,
        version: u64,
    },

    /// Participant joined
    ParticipantJoined {
        room_id: RoomId,
        client_id: ClientId,
        user_data: Option<serde_json::Value>,
    },

    /// Participant left
    ParticipantLeft {
        room_id: RoomId,
        client_id: ClientId,
    },
}

/// Subscribe to a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeMessage {
    /// Channel identifier
    pub channel: ChannelId,

    /// Client identifier
    pub client_id: ClientId,

    /// Optional filter parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<serde_json::Value>,
}

/// Unsubscribe from a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeMessage {
    /// Channel identifier
    pub channel: ChannelId,

    /// Client identifier
    pub client_id: ClientId,
}

/// Synchronization message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "sync_type", rename_all = "snake_case")]
pub enum SyncMessage {
    /// Request full state
    RequestState {
        entity_type: String,
        entity_id: String,
    },

    /// Full state response
    StateResponse {
        entity_type: String,
        entity_id: String,
        state: serde_json::Value,
        version: u64,
    },

    /// Operational transform
    Operation {
        entity_type: String,
        entity_id: String,
        operation: serde_json::Value,
        version: u64,
        client_id: ClientId,
    },

    /// Acknowledgment
    Ack {
        entity_id: String,
        version: u64,
    },

    /// Conflict detected
    Conflict {
        entity_id: String,
        local_version: u64,
        remote_version: u64,
    },
}

/// Error message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// Optional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl StreamMessage {
    /// Create a ping message.
    pub fn ping() -> Self {
        Self::Ping {
            timestamp: current_timestamp(),
        }
    }

    /// Create a pong message.
    pub fn pong() -> Self {
        Self::Pong {
            timestamp: current_timestamp(),
        }
    }

    /// Create an error message.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Error(ErrorMessage {
            code: code.into(),
            message: message.into(),
            details: None,
        })
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }
}

/// Get current timestamp in milliseconds.
pub fn current_timestamp() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as Timestamp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = StreamMessage::ping();
        let json = msg.to_json().unwrap();
        let parsed = StreamMessage::from_json(&json).unwrap();

        match parsed {
            StreamMessage::Ping { .. } => (),
            _ => panic!("Expected Ping message"),
        }
    }

    #[test]
    fn test_feature_update_message() {
        let msg = FeatureUpdateMessage {
            feature_id: "feature-1".to_string(),
            layer_id: "layer-1".to_string(),
            operation: FeatureOperation::Create,
            data: Some(serde_json::json!({"type": "Point"})),
            client_id: Uuid::new_v4(),
            timestamp: current_timestamp(),
            version: 1,
            metadata: None,
        };

        let envelope = StreamMessage::FeatureUpdate(msg);
        let json = envelope.to_json().unwrap();
        let parsed = StreamMessage::from_json(&json).unwrap();

        match parsed {
            StreamMessage::FeatureUpdate(_) => (),
            _ => panic!("Expected FeatureUpdate message"),
        }
    }

    #[test]
    fn test_current_timestamp() {
        let ts1 = current_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = current_timestamp();
        assert!(ts2 > ts1);
    }
}
