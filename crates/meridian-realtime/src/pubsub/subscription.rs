//! Subscription handling for pub/sub system

use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::pubsub::PubSubMessage;

/// Subscription ID type
pub type SubscriptionId = String;

/// Subscription filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionFilter {
    /// No filter (receive all messages)
    All,

    /// Filter by sender ID
    Sender(String),

    /// Filter by message pattern (regex)
    Pattern(String),

    /// Custom filter function
    Custom(String),
}

impl SubscriptionFilter {
    /// Check if message matches filter
    pub fn matches(&self, message: &PubSubMessage) -> bool {
        match self {
            SubscriptionFilter::All => true,
            SubscriptionFilter::Sender(sender_id) => {
                message.sender_id.as_ref().map_or(false, |id| id == sender_id)
            }
            SubscriptionFilter::Pattern(_pattern) => {
                // TODO: Implement regex matching on message data
                true
            }
            SubscriptionFilter::Custom(_) => {
                // TODO: Implement custom filter logic
                true
            }
        }
    }
}

/// Subscription
pub struct Subscription {
    /// Subscription ID
    id: SubscriptionId,

    /// Channel ID
    channel_id: String,

    /// Subscriber ID (user or connection)
    subscriber_id: String,

    /// Filter
    filter: SubscriptionFilter,

    /// Receiver
    rx: broadcast::Receiver<PubSubMessage>,

    /// Active flag
    active: Arc<std::sync::atomic::AtomicBool>,
}

impl Subscription {
    /// Create new subscription
    pub fn new(
        channel_id: String,
        subscriber_id: String,
        filter: SubscriptionFilter,
        rx: broadcast::Receiver<PubSubMessage>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            channel_id,
            subscriber_id,
            filter,
            rx,
            active: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }

    /// Get subscription ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get channel ID
    pub fn channel_id(&self) -> &str {
        &self.channel_id
    }

    /// Get subscriber ID
    pub fn subscriber_id(&self) -> &str {
        &self.subscriber_id
    }

    /// Get filter
    pub fn filter(&self) -> &SubscriptionFilter {
        &self.filter
    }

    /// Check if active
    pub fn is_active(&self) -> bool {
        self.active.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Deactivate subscription
    pub fn deactivate(&self) {
        self.active.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// Receive next message (blocking)
    pub async fn recv(&mut self) -> Result<PubSubMessage> {
        if !self.is_active() {
            return Err(Error::Internal("Subscription is not active".to_string()));
        }

        loop {
            match self.rx.recv().await {
                Ok(msg) => {
                    if self.filter.matches(&msg) {
                        return Ok(msg);
                    }
                    // Message doesn't match filter, continue waiting
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("Subscription lagged, skipped {} messages", skipped);
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(Error::ChannelClosed);
                }
            }
        }
    }

    /// Try to receive message (non-blocking)
    pub fn try_recv(&mut self) -> Option<PubSubMessage> {
        if !self.is_active() {
            return None;
        }

        loop {
            match self.rx.try_recv() {
                Ok(msg) => {
                    if self.filter.matches(&msg) {
                        return Some(msg);
                    }
                    // Message doesn't match filter, try next
                }
                Err(broadcast::error::TryRecvError::Empty) => {
                    return None;
                }
                Err(broadcast::error::TryRecvError::Lagged(skipped)) => {
                    tracing::warn!("Subscription lagged, skipped {} messages", skipped);
                    continue;
                }
                Err(broadcast::error::TryRecvError::Closed) => {
                    return None;
                }
            }
        }
    }
}

/// Subscription manager
pub struct SubscriptionManager {
    /// Subscriptions by ID
    subscriptions: Arc<DashMap<SubscriptionId, Arc<Subscription>>>,

    /// Subscriptions by subscriber ID
    subscriber_subscriptions: Arc<DashMap<String, Vec<SubscriptionId>>>,

    /// Subscriptions by channel ID
    channel_subscriptions: Arc<DashMap<String, Vec<SubscriptionId>>>,
}

impl SubscriptionManager {
    /// Create new subscription manager
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(DashMap::new()),
            subscriber_subscriptions: Arc::new(DashMap::new()),
            channel_subscriptions: Arc::new(DashMap::new()),
        }
    }

    /// Add subscription
    pub fn add_subscription(&self, subscription: Subscription) -> SubscriptionId {
        let id = subscription.id().to_string();
        let subscriber_id = subscription.subscriber_id().to_string();
        let channel_id = subscription.channel_id().to_string();

        let subscription = Arc::new(subscription);
        self.subscriptions.insert(id.clone(), subscription);

        // Add to subscriber index
        self.subscriber_subscriptions
            .entry(subscriber_id)
            .or_insert_with(Vec::new)
            .push(id.clone());

        // Add to channel index
        self.channel_subscriptions
            .entry(channel_id)
            .or_insert_with(Vec::new)
            .push(id.clone());

        id
    }

    /// Get subscription
    pub fn get_subscription(&self, id: &str) -> Option<Arc<Subscription>> {
        self.subscriptions.get(id).map(|s| s.value().clone())
    }

    /// Remove subscription
    pub fn remove_subscription(&self, id: &str) -> Option<Arc<Subscription>> {
        let subscription = self.subscriptions.remove(id).map(|(_, s)| s)?;

        // Remove from subscriber index
        if let Some(mut subs) = self.subscriber_subscriptions.get_mut(subscription.subscriber_id()) {
            subs.retain(|sub_id| sub_id != id);
        }

        // Remove from channel index
        if let Some(mut subs) = self.channel_subscriptions.get_mut(subscription.channel_id()) {
            subs.retain(|sub_id| sub_id != id);
        }

        subscription.deactivate();

        Some(subscription)
    }

    /// Get subscriptions for subscriber
    pub fn get_subscriber_subscriptions(&self, subscriber_id: &str) -> Vec<Arc<Subscription>> {
        self.subscriber_subscriptions
            .get(subscriber_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.subscriptions.get(id).map(|s| s.value().clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get subscriptions for channel
    pub fn get_channel_subscriptions(&self, channel_id: &str) -> Vec<Arc<Subscription>> {
        self.channel_subscriptions
            .get(channel_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.subscriptions.get(id).map(|s| s.value().clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Remove all subscriptions for subscriber
    pub fn remove_subscriber_subscriptions(&self, subscriber_id: &str) {
        if let Some((_, sub_ids)) = self.subscriber_subscriptions.remove(subscriber_id) {
            for sub_id in sub_ids {
                self.remove_subscription(&sub_id);
            }
        }
    }

    /// Get subscription count
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }

    /// Get subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.subscriber_subscriptions.len()
    }

    /// Clear all subscriptions
    pub fn clear(&self) {
        self.subscriptions.clear();
        self.subscriber_subscriptions.clear();
        self.channel_subscriptions.clear();
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[test]
    fn test_subscription_filter() {
        let filter_all = SubscriptionFilter::All;
        let filter_sender = SubscriptionFilter::Sender("user1".to_string());

        let msg1 = PubSubMessage::new("channel1".to_string(), vec![1, 2, 3]);
        let msg2 = PubSubMessage::new("channel1".to_string(), vec![4, 5, 6])
            .with_sender("user1".to_string());

        assert!(filter_all.matches(&msg1));
        assert!(filter_all.matches(&msg2));
        assert!(!filter_sender.matches(&msg1));
        assert!(filter_sender.matches(&msg2));
    }

    #[test]
    fn test_subscription_manager() {
        let manager = SubscriptionManager::new();
        assert_eq!(manager.subscription_count(), 0);

        let (tx, rx) = broadcast::channel(100);
        let subscription = Subscription::new(
            "channel1".to_string(),
            "user1".to_string(),
            SubscriptionFilter::All,
            rx,
        );

        let id = manager.add_subscription(subscription);
        assert_eq!(manager.subscription_count(), 1);

        let retrieved = manager.get_subscription(&id).unwrap();
        assert_eq!(retrieved.channel_id(), "channel1");
        assert_eq!(retrieved.subscriber_id(), "user1");

        manager.remove_subscription(&id);
        assert_eq!(manager.subscription_count(), 0);
    }

    #[test]
    fn test_subscription_indices() {
        let manager = SubscriptionManager::new();

        let (_, rx1) = broadcast::channel(100);
        let (_, rx2) = broadcast::channel(100);
        let (_, rx3) = broadcast::channel(100);

        let sub1 = Subscription::new(
            "channel1".to_string(),
            "user1".to_string(),
            SubscriptionFilter::All,
            rx1,
        );
        let sub2 = Subscription::new(
            "channel1".to_string(),
            "user1".to_string(),
            SubscriptionFilter::All,
            rx2,
        );
        let sub3 = Subscription::new(
            "channel2".to_string(),
            "user2".to_string(),
            SubscriptionFilter::All,
            rx3,
        );

        manager.add_subscription(sub1);
        manager.add_subscription(sub2);
        manager.add_subscription(sub3);

        let user1_subs = manager.get_subscriber_subscriptions("user1");
        assert_eq!(user1_subs.len(), 2);

        let channel1_subs = manager.get_channel_subscriptions("channel1");
        assert_eq!(channel1_subs.len(), 2);

        manager.remove_subscriber_subscriptions("user1");
        assert_eq!(manager.subscription_count(), 1);
    }
}
