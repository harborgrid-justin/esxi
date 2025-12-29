//! Event bus with publish-subscribe pattern for event distribution.

use crate::error::{EventError, Result};
use crate::event::StoredEvent;
use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Subscriber identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriberId(Uuid);

impl SubscriberId {
    /// Generate a new subscriber ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SubscriberId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SubscriberId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Trait for event subscribers.
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Handle an event.
    async fn handle(&self, event: StoredEvent) -> Result<()>;

    /// Get the subscriber name.
    fn name(&self) -> &str;

    /// Filter events (return true to handle the event).
    fn filter(&self, event: &StoredEvent) -> bool {
        let _ = event;
        true // By default, handle all events
    }
}

/// Subscription handle for managing subscriptions.
pub struct Subscription {
    id: SubscriberId,
    name: String,
    receiver: mpsc::UnboundedReceiver<StoredEvent>,
}

impl Subscription {
    /// Create a new subscription.
    fn new(
        id: SubscriberId,
        name: String,
        receiver: mpsc::UnboundedReceiver<StoredEvent>,
    ) -> Self {
        Self { id, name, receiver }
    }

    /// Get the subscription ID.
    pub fn id(&self) -> SubscriberId {
        self.id
    }

    /// Get the subscription name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Receive the next event.
    pub async fn next(&mut self) -> Option<StoredEvent> {
        self.receiver.recv().await
    }

    /// Try to receive an event without blocking.
    pub fn try_next(&mut self) -> Option<StoredEvent> {
        self.receiver.try_recv().ok()
    }
}

/// Event bus for publishing and subscribing to events.
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<SubscriberId, mpsc::UnboundedSender<StoredEvent>>>>,
    subscriber_names: Arc<RwLock<HashMap<SubscriberId, String>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Create a new event bus.
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            subscriber_names: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to events.
    pub fn subscribe(&self, name: impl Into<String>) -> Subscription {
        let id = SubscriberId::new();
        let name = name.into();
        let (tx, rx) = mpsc::unbounded_channel();

        self.subscribers.write().insert(id, tx);
        self.subscriber_names.write().insert(id, name.clone());

        Subscription::new(id, name, rx)
    }

    /// Unsubscribe from events.
    pub fn unsubscribe(&self, id: SubscriberId) {
        self.subscribers.write().remove(&id);
        self.subscriber_names.write().remove(&id);
    }

    /// Publish an event to all subscribers.
    pub async fn publish(&self, event: StoredEvent) -> Result<()> {
        let subscribers = self.subscribers.read();
        let mut failed = Vec::new();

        for (id, sender) in subscribers.iter() {
            if sender.send(event.clone()).is_err() {
                failed.push(*id);
            }
        }

        // Clean up failed subscribers
        drop(subscribers);
        for id in failed {
            self.unsubscribe(id);
        }

        Ok(())
    }

    /// Get the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.read().len()
    }

    /// Get subscriber names.
    pub fn subscriber_names(&self) -> Vec<String> {
        self.subscriber_names.read().values().cloned().collect()
    }
}

/// Filtered event bus that only publishes matching events.
pub struct FilteredEventBus<F>
where
    F: Fn(&StoredEvent) -> bool + Send + Sync,
{
    bus: EventBus,
    filter: F,
}

impl<F> FilteredEventBus<F>
where
    F: Fn(&StoredEvent) -> bool + Send + Sync,
{
    /// Create a new filtered event bus.
    pub fn new(bus: EventBus, filter: F) -> Self {
        Self { bus, filter }
    }

    /// Subscribe to filtered events.
    pub fn subscribe(&self, name: impl Into<String>) -> Subscription {
        self.bus.subscribe(name)
    }

    /// Publish an event if it matches the filter.
    pub async fn publish(&self, event: StoredEvent) -> Result<()> {
        if (self.filter)(&event) {
            self.bus.publish(event).await
        } else {
            Ok(())
        }
    }

    /// Get the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.bus.subscriber_count()
    }
}

/// Event dispatcher that routes events to subscribers.
pub struct EventDispatcher {
    bus: Arc<EventBus>,
    handlers: Arc<RwLock<HashMap<SubscriberId, Arc<dyn EventSubscriber>>>>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventDispatcher {
    /// Create a new event dispatcher.
    pub fn new() -> Self {
        Self {
            bus: Arc::new(EventBus::new()),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a subscriber.
    pub fn register(&self, subscriber: Arc<dyn EventSubscriber>) -> SubscriberId {
        let id = SubscriberId::new();
        self.handlers.write().insert(id, subscriber);
        id
    }

    /// Unregister a subscriber.
    pub fn unregister(&self, id: SubscriberId) {
        self.handlers.write().remove(&id);
    }

    /// Dispatch an event to all registered subscribers.
    pub async fn dispatch(&self, event: StoredEvent) -> Result<()> {
        let handlers = self.handlers.read();

        for handler in handlers.values() {
            if handler.filter(&event) {
                handler.handle(event.clone()).await?;
            }
        }

        Ok(())
    }

    /// Get the number of registered subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.handlers.read().len()
    }

    /// Get the event bus.
    pub fn bus(&self) -> Arc<EventBus> {
        Arc::clone(&self.bus)
    }
}

/// Event stream for consuming events asynchronously.
pub struct EventStream {
    subscription: Subscription,
}

impl EventStream {
    /// Create a new event stream.
    pub fn new(subscription: Subscription) -> Self {
        Self { subscription }
    }

    /// Get the next event from the stream.
    pub async fn next(&mut self) -> Option<StoredEvent> {
        self.subscription.next().await
    }

    /// Get the subscription ID.
    pub fn id(&self) -> SubscriberId {
        self.subscription.id()
    }
}

/// Multi-bus router for routing events to multiple buses.
pub struct EventRouter {
    buses: Vec<Arc<EventBus>>,
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventRouter {
    /// Create a new event router.
    pub fn new() -> Self {
        Self { buses: Vec::new() }
    }

    /// Add a bus to the router.
    pub fn add_bus(&mut self, bus: Arc<EventBus>) {
        self.buses.push(bus);
    }

    /// Route an event to all buses.
    pub async fn route(&self, event: StoredEvent) -> Result<()> {
        for bus in &self.buses {
            bus.publish(event.clone()).await?;
        }
        Ok(())
    }

    /// Get the number of buses.
    pub fn bus_count(&self) -> usize {
        self.buses.len()
    }
}

/// Simple subscriber implementation.
pub struct SimpleSubscriber<F>
where
    F: Fn(StoredEvent) -> Result<()> + Send + Sync,
{
    name: String,
    handler: F,
}

impl<F> SimpleSubscriber<F>
where
    F: Fn(StoredEvent) -> Result<()> + Send + Sync,
{
    /// Create a new simple subscriber.
    pub fn new(name: impl Into<String>, handler: F) -> Self {
        Self {
            name: name.into(),
            handler,
        }
    }
}

#[async_trait]
impl<F> EventSubscriber for SimpleSubscriber<F>
where
    F: Fn(StoredEvent) -> Result<()> + Send + Sync,
{
    async fn handle(&self, event: StoredEvent) -> Result<()> {
        (self.handler)(event)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventMetadata, EventVersion, StreamId};

    #[test]
    fn test_subscriber_id_generation() {
        let id1 = SubscriberId::new();
        let id2 = SubscriberId::new();
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_event_bus_subscribe() {
        let bus = EventBus::new();
        let mut sub = bus.subscribe("test-subscriber");

        assert_eq!(bus.subscriber_count(), 1);
        assert_eq!(sub.name(), "test-subscriber");

        bus.unsubscribe(sub.id());
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_event_bus_publish() {
        let bus = EventBus::new();
        let mut sub = bus.subscribe("test-subscriber");

        let event = StoredEvent::new(
            EventMetadata::new(
                StreamId::new("test"),
                1,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            vec![],
        );

        bus.publish(event.clone()).await.unwrap();

        let received = sub.next().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap().metadata.sequence, 1);
    }

    #[tokio::test]
    async fn test_filtered_event_bus() {
        let bus = EventBus::new();
        let filtered = FilteredEventBus::new(bus, |event| event.metadata.sequence > 5);

        let mut sub = filtered.subscribe("filtered-subscriber");

        let event1 = StoredEvent::new(
            EventMetadata::new(
                StreamId::new("test"),
                3,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            vec![],
        );

        let event2 = StoredEvent::new(
            EventMetadata::new(
                StreamId::new("test"),
                10,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            vec![],
        );

        filtered.publish(event1).await.unwrap();
        filtered.publish(event2.clone()).await.unwrap();

        let received = sub.next().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap().metadata.sequence, 10);
    }

    #[tokio::test]
    async fn test_event_dispatcher() {
        let dispatcher = EventDispatcher::new();

        let subscriber = Arc::new(SimpleSubscriber::new("test", |event| {
            assert_eq!(event.metadata.sequence, 1);
            Ok(())
        }));

        dispatcher.register(subscriber);
        assert_eq!(dispatcher.subscriber_count(), 1);

        let event = StoredEvent::new(
            EventMetadata::new(
                StreamId::new("test"),
                1,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            vec![],
        );

        dispatcher.dispatch(event).await.unwrap();
    }

    #[tokio::test]
    async fn test_event_router() {
        let mut router = EventRouter::new();

        let bus1 = Arc::new(EventBus::new());
        let bus2 = Arc::new(EventBus::new());

        let mut sub1 = bus1.subscribe("subscriber-1");
        let mut sub2 = bus2.subscribe("subscriber-2");

        router.add_bus(bus1);
        router.add_bus(bus2);

        assert_eq!(router.bus_count(), 2);

        let event = StoredEvent::new(
            EventMetadata::new(
                StreamId::new("test"),
                1,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            vec![],
        );

        router.route(event).await.unwrap();

        assert!(sub1.next().await.is_some());
        assert!(sub2.next().await.is_some());
    }
}
