//! Channel management for pub/sub

use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::pubsub::PubSubMessage;

/// Channel ID type
pub type ChannelId = String;

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Maximum subscribers
    pub max_subscribers: usize,

    /// Message buffer size
    pub buffer_size: usize,

    /// Persistent (messages stored)
    pub persistent: bool,

    /// Require authentication
    pub require_auth: bool,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            max_subscribers: 1000,
            buffer_size: 1000,
            persistent: false,
            require_auth: false,
        }
    }
}

/// Pub/sub channel
pub struct Channel {
    /// Channel ID
    id: ChannelId,

    /// Channel name
    name: String,

    /// Configuration
    config: ChannelConfig,

    /// Broadcast sender
    tx: broadcast::Sender<PubSubMessage>,

    /// Subscriber count tracker
    subscribers: Arc<std::sync::atomic::AtomicUsize>,

    /// Message history (if persistent)
    history: Arc<parking_lot::RwLock<Vec<PubSubMessage>>>,
}

impl Channel {
    /// Create new channel
    pub fn new(id: ChannelId, name: String, config: ChannelConfig) -> Self {
        let (tx, _) = broadcast::channel(config.buffer_size);

        Self {
            id,
            name,
            config,
            tx,
            subscribers: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            history: Arc::new(parking_lot::RwLock::new(Vec::new())),
        }
    }

    /// Get channel ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get channel name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get configuration
    pub fn config(&self) -> &ChannelConfig {
        &self.config
    }

    /// Subscribe to channel
    pub fn subscribe(&self) -> Result<broadcast::Receiver<PubSubMessage>> {
        let current_subscribers = self.subscribers.load(std::sync::atomic::Ordering::Relaxed);

        if current_subscribers >= self.config.max_subscribers {
            return Err(Error::Internal(format!(
                "Channel {} has reached maximum subscribers ({})",
                self.id, self.config.max_subscribers
            )));
        }

        self.subscribers
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(self.tx.subscribe())
    }

    /// Publish message to channel
    pub async fn publish(&self, message: PubSubMessage) -> Result<()> {
        // Store in history if persistent
        if self.config.persistent {
            let mut history = self.history.write();
            history.push(message.clone());

            // Keep only last N messages
            if history.len() > self.config.buffer_size {
                let excess = history.len() - self.config.buffer_size;
                history.drain(0..excess);
            }
        }

        // Broadcast to subscribers
        self.tx
            .send(message)
            .map_err(|e| Error::Internal(format!("Failed to publish message: {}", e)))?;

        Ok(())
    }

    /// Get message history
    pub fn get_history(&self) -> Vec<PubSubMessage> {
        self.history.read().clone()
    }

    /// Get subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Clear history
    pub fn clear_history(&self) {
        self.history.write().clear();
    }
}

/// Channel manager
pub struct ChannelManager {
    /// Channels by ID
    channels: Arc<DashMap<ChannelId, Arc<Channel>>>,

    /// Default channel config
    default_config: ChannelConfig,
}

impl ChannelManager {
    /// Create new channel manager
    pub fn new() -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
            default_config: ChannelConfig::default(),
        }
    }

    /// Create with default config
    pub fn with_default_config(config: ChannelConfig) -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
            default_config: config,
        }
    }

    /// Get or create channel
    pub fn get_or_create(&self, channel_id: &str) -> Arc<Channel> {
        if let Some(channel) = self.channels.get(channel_id) {
            return channel.value().clone();
        }

        let channel = Arc::new(Channel::new(
            channel_id.to_string(),
            channel_id.to_string(),
            self.default_config.clone(),
        ));

        self.channels.insert(channel_id.to_string(), channel.clone());
        channel
    }

    /// Create channel with custom config
    pub fn create_channel(
        &self,
        channel_id: ChannelId,
        name: String,
        config: ChannelConfig,
    ) -> Result<Arc<Channel>> {
        if self.channels.contains_key(&channel_id) {
            return Err(Error::Internal(format!(
                "Channel {} already exists",
                channel_id
            )));
        }

        let channel = Arc::new(Channel::new(channel_id.clone(), name, config));
        self.channels.insert(channel_id, channel.clone());

        Ok(channel)
    }

    /// Get channel
    pub fn get_channel(&self, channel_id: &str) -> Option<Arc<Channel>> {
        self.channels.get(channel_id).map(|c| c.value().clone())
    }

    /// Remove channel
    pub fn remove_channel(&self, channel_id: &str) -> Option<Arc<Channel>> {
        self.channels.remove(channel_id).map(|(_, c)| c)
    }

    /// Get all channel IDs
    pub fn channel_ids(&self) -> Vec<ChannelId> {
        self.channels.iter().map(|e| e.key().clone()).collect()
    }

    /// Get channel count
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Get total subscriber count across all channels
    pub fn total_subscribers(&self) -> usize {
        self.channels
            .iter()
            .map(|e| e.value().subscriber_count())
            .sum()
    }

    /// Publish to channel
    pub async fn publish(&self, channel_id: &str, message: PubSubMessage) -> Result<()> {
        let channel = self.get_or_create(channel_id);
        channel.publish(message).await
    }

    /// Subscribe to channel
    pub fn subscribe(&self, channel_id: &str) -> Result<broadcast::Receiver<PubSubMessage>> {
        let channel = self.get_or_create(channel_id);
        channel.subscribe()
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

    #[test]
    fn test_channel_creation() {
        let config = ChannelConfig::default();
        let channel = Channel::new("test".to_string(), "Test Channel".to_string(), config);

        assert_eq!(channel.id(), "test");
        assert_eq!(channel.name(), "Test Channel");
        assert_eq!(channel.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_channel_publish_subscribe() {
        let config = ChannelConfig::default();
        let channel = Channel::new("test".to_string(), "Test".to_string(), config);

        let mut rx = channel.subscribe().unwrap();

        let msg = PubSubMessage::new("test".to_string(), vec![1, 2, 3]);
        channel.publish(msg.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_channel_manager() {
        let manager = ChannelManager::new();
        assert_eq!(manager.channel_count(), 0);

        let channel = manager.get_or_create("test");
        assert_eq!(channel.id(), "test");
        assert_eq!(manager.channel_count(), 1);

        let same_channel = manager.get_or_create("test");
        assert_eq!(manager.channel_count(), 1);
        assert_eq!(channel.id(), same_channel.id());
    }

    #[tokio::test]
    async fn test_persistent_channel() {
        let mut config = ChannelConfig::default();
        config.persistent = true;

        let channel = Channel::new("test".to_string(), "Test".to_string(), config);

        let msg1 = PubSubMessage::new("test".to_string(), vec![1]);
        let msg2 = PubSubMessage::new("test".to_string(), vec![2]);

        channel.publish(msg1).await.unwrap();
        channel.publish(msg2).await.unwrap();

        let history = channel.get_history();
        assert_eq!(history.len(), 2);
    }
}
