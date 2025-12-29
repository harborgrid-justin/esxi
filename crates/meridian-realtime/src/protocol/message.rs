//! Protocol message types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Message type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageType {
    // Connection lifecycle
    /// Authentication request
    Auth = 0x01,
    /// Authentication success
    AuthSuccess = 0x02,
    /// Authentication failure
    AuthFailure = 0x03,

    // Room management
    /// Join room
    JoinRoom = 0x10,
    /// Leave room
    LeaveRoom = 0x11,
    /// Room joined
    RoomJoined = 0x12,
    /// Room left
    RoomLeft = 0x13,

    // User events
    /// User joined
    UserJoined = 0x20,
    /// User left
    UserLeft = 0x21,
    /// User updated
    UserUpdated = 0x22,

    // Data synchronization
    /// Data message
    Data = 0x30,
    /// Sync request
    Sync = 0x31,
    /// Sync response
    SyncResponse = 0x32,
    /// State update
    StateUpdate = 0x33,

    // CRDT operations
    /// CRDT operation
    CrdtOperation = 0x40,
    /// CRDT merge
    CrdtMerge = 0x41,

    // Collaboration
    /// Cursor update
    Cursor = 0x50,
    /// Selection update
    Selection = 0x51,
    /// Presence update
    Presence = 0x52,
    /// Annotation
    Annotation = 0x53,

    // Streaming
    /// GPS update
    GpsUpdate = 0x60,
    /// Sensor reading
    SensorReading = 0x61,
    /// Event
    Event = 0x62,

    // Control messages
    /// Ping
    Ping = 0xF0,
    /// Pong
    Pong = 0xF1,
    /// Error
    Error = 0xF2,
    /// Ack
    Ack = 0xF3,

    // Custom
    /// Custom message
    Custom = 0xFF,
}

impl MessageType {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::Auth),
            0x02 => Some(Self::AuthSuccess),
            0x03 => Some(Self::AuthFailure),
            0x10 => Some(Self::JoinRoom),
            0x11 => Some(Self::LeaveRoom),
            0x12 => Some(Self::RoomJoined),
            0x13 => Some(Self::RoomLeft),
            0x20 => Some(Self::UserJoined),
            0x21 => Some(Self::UserLeft),
            0x22 => Some(Self::UserUpdated),
            0x30 => Some(Self::Data),
            0x31 => Some(Self::Sync),
            0x32 => Some(Self::SyncResponse),
            0x33 => Some(Self::StateUpdate),
            0x40 => Some(Self::CrdtOperation),
            0x41 => Some(Self::CrdtMerge),
            0x50 => Some(Self::Cursor),
            0x51 => Some(Self::Selection),
            0x52 => Some(Self::Presence),
            0x53 => Some(Self::Annotation),
            0x60 => Some(Self::GpsUpdate),
            0x61 => Some(Self::SensorReading),
            0x62 => Some(Self::Event),
            0xF0 => Some(Self::Ping),
            0xF1 => Some(Self::Pong),
            0xF2 => Some(Self::Error),
            0xF3 => Some(Self::Ack),
            0xFF => Some(Self::Custom),
            _ => None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Message priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessagePriority {
    /// Low priority
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority
    High = 2,
    /// Critical priority
    Critical = 3,
}

impl MessagePriority {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Low),
            1 => Some(Self::Normal),
            2 => Some(Self::High),
            3 => Some(Self::Critical),
            _ => None,
        }
    }

    /// Convert to u8
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Protocol message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID
    pub id: String,

    /// Message type
    pub msg_type: MessageType,

    /// Priority
    pub priority: MessagePriority,

    /// Payload
    pub payload: Vec<u8>,

    /// Timestamp (milliseconds since epoch)
    pub timestamp: i64,

    /// Sequence number (for ordering)
    pub sequence: Option<u64>,

    /// Requires acknowledgment
    pub requires_ack: bool,

    /// Metadata
    pub metadata: serde_json::Value,
}

impl Message {
    /// Create new message
    pub fn new(msg_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            msg_type,
            priority: MessagePriority::Normal,
            payload,
            timestamp: chrono::Utc::now().timestamp_millis(),
            sequence: None,
            requires_ack: false,
            metadata: serde_json::json!({}),
        }
    }

    /// With priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// With sequence number
    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.sequence = Some(sequence);
        self
    }

    /// Require acknowledgment
    pub fn require_ack(mut self) -> Self {
        self.requires_ack = true;
        self
    }

    /// With metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Create acknowledgment message
    pub fn create_ack(&self) -> Self {
        Self::new(MessageType::Ack, self.id.as_bytes().to_vec())
            .with_metadata(serde_json::json!({
                "ack_for": self.id,
                "ack_type": format!("{:?}", self.msg_type)
            }))
    }

    /// Create error message
    pub fn create_error(error: &str) -> Self {
        Self::new(MessageType::Error, error.as_bytes().to_vec())
            .with_priority(MessagePriority::High)
    }

    /// Create ping message
    pub fn create_ping() -> Self {
        Self::new(MessageType::Ping, vec![])
    }

    /// Create pong message
    pub fn create_pong() -> Self {
        Self::new(MessageType::Pong, vec![])
    }

    /// Get payload size
    pub fn payload_size(&self) -> usize {
        self.payload.len()
    }

    /// Check if payload is empty
    pub fn is_empty(&self) -> bool {
        self.payload.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_conversion() {
        assert_eq!(MessageType::Auth.to_u8(), 0x01);
        assert_eq!(MessageType::from_u8(0x01), Some(MessageType::Auth));
        assert_eq!(MessageType::from_u8(0x99), None);
    }

    #[test]
    fn test_message_priority() {
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::new(MessageType::Data, vec![1, 2, 3])
            .with_priority(MessagePriority::High)
            .with_sequence(42)
            .require_ack();

        assert_eq!(msg.msg_type, MessageType::Data);
        assert_eq!(msg.priority, MessagePriority::High);
        assert_eq!(msg.sequence, Some(42));
        assert!(msg.requires_ack);
        assert_eq!(msg.payload_size(), 3);
    }

    #[test]
    fn test_message_ack() {
        let msg = Message::new(MessageType::Data, vec![1, 2, 3]);
        let ack = msg.create_ack();

        assert_eq!(ack.msg_type, MessageType::Ack);
        assert_eq!(ack.metadata["ack_for"], msg.id);
    }

    #[test]
    fn test_control_messages() {
        let ping = Message::create_ping();
        assert_eq!(ping.msg_type, MessageType::Ping);
        assert!(ping.is_empty());

        let pong = Message::create_pong();
        assert_eq!(pong.msg_type, MessageType::Pong);

        let error = Message::create_error("Test error");
        assert_eq!(error.msg_type, MessageType::Error);
        assert_eq!(error.priority, MessagePriority::High);
    }
}
