//! Publish-Subscribe system for message routing

pub mod channel;
pub mod subscription;

pub use channel::{Channel, ChannelId, ChannelManager};
pub use subscription::{Subscription, SubscriptionId, SubscriptionManager};

use serde::{Deserialize, Serialize};

/// Message for pub/sub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubMessage {
    /// Channel ID
    pub channel: String,

    /// Message data
    pub data: Vec<u8>,

    /// Message timestamp
    pub timestamp: i64,

    /// Sender ID
    pub sender_id: Option<String>,

    /// Message ID
    pub message_id: String,
}

impl PubSubMessage {
    /// Create new pub/sub message
    pub fn new(channel: String, data: Vec<u8>) -> Self {
        Self {
            channel,
            data,
            timestamp: chrono::Utc::now().timestamp_millis(),
            sender_id: None,
            message_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// With sender ID
    pub fn with_sender(mut self, sender_id: String) -> Self {
        self.sender_id = Some(sender_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pubsub_message() {
        let msg = PubSubMessage::new("channel1".to_string(), vec![1, 2, 3])
            .with_sender("user1".to_string());

        assert_eq!(msg.channel, "channel1");
        assert_eq!(msg.data, vec![1, 2, 3]);
        assert_eq!(msg.sender_id, Some("user1".to_string()));
        assert!(!msg.message_id.is_empty());
    }
}
