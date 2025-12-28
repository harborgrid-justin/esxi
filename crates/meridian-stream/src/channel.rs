//! Publish/subscribe channel system for message broadcasting.

use crate::error::{Result, StreamError};
use crate::messages::{ChannelId, ClientId, StreamMessage};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

/// Default channel capacity for broadcast channels.
const DEFAULT_CHANNEL_CAPACITY: usize = 1024;

/// Maximum number of channels allowed.
const MAX_CHANNELS: usize = 10000;

/// A subscription to a channel.
pub struct Subscription {
    /// Channel identifier
    pub channel_id: ChannelId,
    /// Receiver for messages
    receiver: broadcast::Receiver<Arc<StreamMessage>>,
}

impl Subscription {
    /// Receive the next message from the channel.
    pub async fn recv(&mut self) -> Result<Arc<StreamMessage>> {
        self.receiver
            .recv()
            .await
            .map_err(|e| match e {
                broadcast::error::RecvError::Closed => StreamError::ConnectionClosed,
                broadcast::error::RecvError::Lagged(n) => {
                    warn!("Subscription lagged by {} messages", n);
                    StreamError::generic(format!("Subscription lagged by {} messages", n))
                }
            })
    }

    /// Try to receive a message without blocking.
    pub fn try_recv(&mut self) -> Result<Arc<StreamMessage>> {
        self.receiver
            .try_recv()
            .map_err(|e| match e {
                broadcast::error::TryRecvError::Empty => {
                    StreamError::generic("No messages available")
                }
                broadcast::error::TryRecvError::Closed => StreamError::ConnectionClosed,
                broadcast::error::TryRecvError::Lagged(n) => {
                    warn!("Subscription lagged by {} messages", n);
                    StreamError::generic(format!("Subscription lagged by {} messages", n))
                }
            })
    }
}

/// Internal channel state.
struct ChannelState {
    /// Broadcast sender
    sender: broadcast::Sender<Arc<StreamMessage>>,
    /// Number of active subscribers
    subscriber_count: usize,
    /// Channel metadata
    metadata: serde_json::Value,
}

/// Channel manager for pub/sub operations.
#[derive(Clone)]
pub struct ChannelManager {
    /// Active channels
    channels: Arc<DashMap<ChannelId, ChannelState>>,
    /// Client subscriptions
    subscriptions: Arc<DashMap<ClientId, Vec<ChannelId>>>,
    /// Default channel capacity
    capacity: usize,
}

impl ChannelManager {
    /// Create a new channel manager.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHANNEL_CAPACITY)
    }

    /// Create a new channel manager with specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
            subscriptions: Arc::new(DashMap::new()),
            capacity,
        }
    }

    /// Create a new channel.
    pub fn create_channel(&self, channel_id: ChannelId) -> Result<()> {
        self.create_channel_with_metadata(channel_id, serde_json::Value::Null)
    }

    /// Create a new channel with metadata.
    pub fn create_channel_with_metadata(
        &self,
        channel_id: ChannelId,
        metadata: serde_json::Value,
    ) -> Result<()> {
        if self.channels.len() >= MAX_CHANNELS {
            return Err(StreamError::generic("Maximum number of channels exceeded"));
        }

        if self.channels.contains_key(&channel_id) {
            return Ok(()); // Channel already exists
        }

        let (sender, _) = broadcast::channel(self.capacity);
        let state = ChannelState {
            sender,
            subscriber_count: 0,
            metadata,
        };

        self.channels.insert(channel_id.clone(), state);
        info!("Created channel: {}", channel_id);
        Ok(())
    }

    /// Subscribe a client to a channel.
    pub fn subscribe(&self, client_id: ClientId, channel_id: ChannelId) -> Result<Subscription> {
        // Create channel if it doesn't exist
        if !self.channels.contains_key(&channel_id) {
            self.create_channel(channel_id.clone())?;
        }

        // Get receiver from channel
        let receiver = {
            let mut channel = self
                .channels
                .get_mut(&channel_id)
                .ok_or_else(|| StreamError::ChannelNotFound(channel_id.clone()))?;
            channel.subscriber_count += 1;
            channel.sender.subscribe()
        };

        // Track subscription for client
        self.subscriptions
            .entry(client_id)
            .or_insert_with(Vec::new)
            .push(channel_id.clone());

        debug!(
            "Client {} subscribed to channel {}",
            client_id, channel_id
        );

        Ok(Subscription {
            channel_id,
            receiver,
        })
    }

    /// Unsubscribe a client from a channel.
    pub fn unsubscribe(&self, client_id: ClientId, channel_id: &ChannelId) -> Result<()> {
        // Remove from client subscriptions
        if let Some(mut subs) = self.subscriptions.get_mut(&client_id) {
            subs.retain(|c| c != channel_id);
        }

        // Decrement subscriber count
        if let Some(mut channel) = self.channels.get_mut(channel_id) {
            channel.subscriber_count = channel.subscriber_count.saturating_sub(1);
            debug!(
                "Client {} unsubscribed from channel {}",
                client_id, channel_id
            );
        }

        Ok(())
    }

    /// Unsubscribe a client from all channels.
    pub fn unsubscribe_all(&self, client_id: ClientId) -> Result<()> {
        if let Some((_, channels)) = self.subscriptions.remove(&client_id) {
            for channel_id in channels {
                if let Some(mut channel) = self.channels.get_mut(&channel_id) {
                    channel.subscriber_count = channel.subscriber_count.saturating_sub(1);
                }
            }
            debug!("Client {} unsubscribed from all channels", client_id);
        }
        Ok(())
    }

    /// Publish a message to a channel.
    pub fn publish(&self, channel_id: &ChannelId, message: StreamMessage) -> Result<usize> {
        let channel = self
            .channels
            .get(channel_id)
            .ok_or_else(|| StreamError::ChannelNotFound(channel_id.clone()))?;

        let arc_message = Arc::new(message);
        let receiver_count = channel
            .sender
            .send(arc_message)
            .map_err(|_| StreamError::generic("Failed to send message"))?;

        debug!(
            "Published message to channel {} ({} receivers)",
            channel_id, receiver_count
        );

        Ok(receiver_count)
    }

    /// Broadcast a message to multiple channels.
    pub fn broadcast(&self, channel_ids: &[ChannelId], message: StreamMessage) -> Result<()> {
        let arc_message = Arc::new(message);
        for channel_id in channel_ids {
            if let Some(channel) = self.channels.get(channel_id) {
                let _ = channel.sender.send(arc_message.clone());
            }
        }
        Ok(())
    }

    /// Get the number of subscribers for a channel.
    pub fn subscriber_count(&self, channel_id: &ChannelId) -> usize {
        self.channels
            .get(channel_id)
            .map(|c| c.subscriber_count)
            .unwrap_or(0)
    }

    /// Get all channels a client is subscribed to.
    pub fn get_client_channels(&self, client_id: &ClientId) -> Vec<ChannelId> {
        self.subscriptions
            .get(client_id)
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Delete a channel.
    pub fn delete_channel(&self, channel_id: &ChannelId) -> Result<()> {
        if let Some((_, channel)) = self.channels.remove(channel_id) {
            info!(
                "Deleted channel {} ({} subscribers)",
                channel_id, channel.subscriber_count
            );
        }
        Ok(())
    }

    /// Clean up channels with no subscribers.
    pub fn cleanup_empty_channels(&self) -> usize {
        let mut removed = 0;
        self.channels.retain(|channel_id, channel| {
            if channel.subscriber_count == 0 {
                info!("Removing empty channel: {}", channel_id);
                removed += 1;
                false
            } else {
                true
            }
        });
        removed
    }

    /// Get the total number of active channels.
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Get the total number of active clients.
    pub fn client_count(&self) -> usize {
        self.subscriptions.len()
    }

    /// Get channel metadata.
    pub fn get_channel_metadata(&self, channel_id: &ChannelId) -> Option<serde_json::Value> {
        self.channels
            .get(channel_id)
            .map(|c| c.metadata.clone())
    }

    /// Update channel metadata.
    pub fn update_channel_metadata(
        &self,
        channel_id: &ChannelId,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let mut channel = self
            .channels
            .get_mut(channel_id)
            .ok_or_else(|| StreamError::ChannelNotFound(channel_id.clone()))?;
        channel.metadata = metadata;
        Ok(())
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_channel_creation() {
        let manager = ChannelManager::new();
        let channel_id = "test-channel".to_string();

        manager.create_channel(channel_id.clone()).unwrap();
        assert_eq!(manager.channel_count(), 1);
    }

    #[tokio::test]
    async fn test_subscribe_and_publish() {
        let manager = ChannelManager::new();
        let channel_id = "test-channel".to_string();
        let client_id = Uuid::new_v4();

        let mut sub = manager
            .subscribe(client_id, channel_id.clone())
            .unwrap();

        let msg = StreamMessage::ping();
        manager.publish(&channel_id, msg).unwrap();

        let received = sub.recv().await.unwrap();
        assert!(matches!(*received, StreamMessage::Ping { .. }));
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let manager = ChannelManager::new();
        let channel_id = "test-channel".to_string();
        let client_id = Uuid::new_v4();

        manager.subscribe(client_id, channel_id.clone()).unwrap();
        assert_eq!(manager.subscriber_count(&channel_id), 1);

        manager.unsubscribe(client_id, &channel_id).unwrap();
        assert_eq!(manager.subscriber_count(&channel_id), 0);
    }

    #[tokio::test]
    async fn test_cleanup_empty_channels() {
        let manager = ChannelManager::new();
        manager.create_channel("channel1".to_string()).unwrap();
        manager.create_channel("channel2".to_string()).unwrap();

        assert_eq!(manager.channel_count(), 2);

        let removed = manager.cleanup_empty_channels();
        assert_eq!(removed, 2);
        assert_eq!(manager.channel_count(), 0);
    }
}
