//! Message broadcasting with Redis pub/sub for horizontal scaling

use std::sync::Arc;

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::error::{Error, Result};
use crate::protocol::Message;

/// Broadcast message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastEnvelope {
    /// Target room ID (None = all rooms)
    pub room_id: Option<String>,

    /// Sender instance ID
    pub sender_instance: String,

    /// Message payload
    pub message: Message,

    /// Timestamp
    pub timestamp: i64,
}

impl BroadcastEnvelope {
    /// Create new broadcast envelope
    pub fn new(room_id: Option<String>, message: Message) -> Self {
        Self {
            room_id,
            sender_instance: instance_id(),
            message,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// Broadcaster handles Redis pub/sub for multi-instance deployments
pub struct Broadcaster {
    redis_pool: deadpool_redis::Pool,
    local_tx: broadcast::Sender<BroadcastEnvelope>,
    instance_id: String,
}

impl Broadcaster {
    /// Create new broadcaster
    pub fn new(redis_pool: deadpool_redis::Pool) -> Self {
        let (local_tx, _) = broadcast::channel(10000);
        let instance_id = instance_id();

        Self {
            redis_pool,
            local_tx,
            instance_id,
        }
    }

    /// Start listening for Redis pub/sub messages
    pub async fn start(&self) -> Result<()> {
        let mut conn = self.redis_pool.get().await?;
        let mut pubsub = conn.as_pubsub();

        // Subscribe to global channel
        pubsub.subscribe("meridian:broadcast").await?;

        info!("Broadcaster started for instance {}", self.instance_id);

        let local_tx = self.local_tx.clone();
        let instance_id = self.instance_id.clone();

        tokio::spawn(async move {
            loop {
                match pubsub.on_message().next().await {
                    Some(msg) => {
                        let payload = msg.get_payload_bytes();
                        match rmp_serde::from_slice::<BroadcastEnvelope>(payload) {
                            Ok(envelope) => {
                                // Ignore messages from this instance
                                if envelope.sender_instance != instance_id {
                                    if let Err(e) = local_tx.send(envelope) {
                                        error!("Failed to send to local channel: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize broadcast envelope: {}", e);
                            }
                        }
                    }
                    None => {
                        error!("Redis pub/sub connection closed");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Broadcast message to all instances
    pub async fn broadcast(&self, room_id: Option<String>, message: Message) -> Result<()> {
        let envelope = BroadcastEnvelope::new(room_id, message);
        let data = rmp_serde::to_vec(&envelope)?;

        let mut conn = self.redis_pool.get().await?;
        conn.publish::<_, _, ()>("meridian:broadcast", data).await?;

        Ok(())
    }

    /// Broadcast to specific room
    pub async fn broadcast_to_room(&self, room_id: String, message: Message) -> Result<()> {
        self.broadcast(Some(room_id), message).await
    }

    /// Broadcast to all rooms
    pub async fn broadcast_global(&self, message: Message) -> Result<()> {
        self.broadcast(None, message).await
    }

    /// Subscribe to local broadcast events
    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastEnvelope> {
        self.local_tx.subscribe()
    }

    /// Get instance ID
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }
}

/// Generate unique instance ID
fn instance_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());

    format!("{}-{}-{}", hostname, std::process::id(), count)
}

/// Broadcast statistics
#[derive(Debug, Clone, Default)]
pub struct BroadcastStats {
    /// Messages sent
    pub messages_sent: u64,

    /// Messages received
    pub messages_received: u64,

    /// Bytes sent
    pub bytes_sent: u64,

    /// Bytes received
    pub bytes_received: u64,

    /// Errors
    pub errors: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_id() {
        let id1 = instance_id();
        let id2 = instance_id();

        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2); // Should be unique
    }

    #[test]
    fn test_broadcast_envelope() {
        let msg = Message::new(crate::protocol::MessageType::Data, vec![1, 2, 3]);
        let envelope = BroadcastEnvelope::new(Some("room1".to_string()), msg);

        assert_eq!(envelope.room_id, Some("room1".to_string()));
        assert!(envelope.timestamp > 0);
    }
}
