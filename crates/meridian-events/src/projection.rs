//! Event projections for building read models (CQRS read side).

use crate::error::{EventError, Result};
use crate::event::{DomainEvent, StoredEvent};
use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Trait for event projections.
#[async_trait]
pub trait Projection: Send + Sync {
    /// Get the projection name.
    fn name(&self) -> &str;

    /// Handle an event.
    async fn handle(&self, event: &StoredEvent) -> Result<()>;

    /// Reset the projection.
    async fn reset(&self) -> Result<()>;

    /// Get the current position of the projection.
    async fn position(&self) -> Result<u64>;

    /// Set the position of the projection.
    async fn set_position(&self, position: u64) -> Result<()>;
}

/// Typed projection that handles specific event types.
#[async_trait]
pub trait TypedProjection<E: DomainEvent>: Send + Sync {
    /// Handle a typed event.
    async fn handle_event(&self, event: &E) -> Result<()>;
}

/// Projection manager for coordinating multiple projections.
pub struct ProjectionManager {
    projections: Arc<RwLock<HashMap<String, Arc<dyn Projection>>>>,
}

impl Default for ProjectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectionManager {
    /// Create a new projection manager.
    pub fn new() -> Self {
        Self {
            projections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a projection.
    pub fn register(&self, projection: Arc<dyn Projection>) {
        let name = projection.name().to_string();
        self.projections.write().insert(name, projection);
    }

    /// Get a projection by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Projection>> {
        self.projections.read().get(name).cloned()
    }

    /// Handle an event with all registered projections.
    pub async fn handle_event(&self, event: &StoredEvent) -> Result<()> {
        let projections = self.projections.read().clone();

        for projection in projections.values() {
            projection.handle(event).await?;
        }

        Ok(())
    }

    /// Reset all projections.
    pub async fn reset_all(&self) -> Result<()> {
        let projections = self.projections.read().clone();

        for projection in projections.values() {
            projection.reset().await?;
        }

        Ok(())
    }

    /// Get all projection names.
    pub fn projection_names(&self) -> Vec<String> {
        self.projections.read().keys().cloned().collect()
    }
}

/// In-memory projection state store.
#[derive(Debug, Clone)]
pub struct InMemoryProjectionState {
    state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    position: Arc<RwLock<u64>>,
}

impl Default for InMemoryProjectionState {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryProjectionState {
    /// Create a new in-memory projection state.
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
            position: Arc::new(RwLock::new(0)),
        }
    }

    /// Set a value in the state.
    pub fn set(&self, key: String, value: serde_json::Value) {
        self.state.write().insert(key, value);
    }

    /// Get a value from the state.
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.state.read().get(key).cloned()
    }

    /// Remove a value from the state.
    pub fn remove(&self, key: &str) -> Option<serde_json::Value> {
        self.state.write().remove(key)
    }

    /// Clear all state.
    pub fn clear(&self) {
        self.state.write().clear();
        *self.position.write() = 0;
    }

    /// Get all keys.
    pub fn keys(&self) -> Vec<String> {
        self.state.read().keys().cloned().collect()
    }

    /// Get the current position.
    pub fn position(&self) -> u64 {
        *self.position.read()
    }

    /// Set the current position.
    pub fn set_position(&self, pos: u64) {
        *self.position.write() = pos;
    }
}

/// Base projection implementation.
pub struct BaseProjection<H>
where
    H: Fn(&StoredEvent) -> Result<()> + Send + Sync,
{
    name: String,
    state: InMemoryProjectionState,
    handler: H,
}

impl<H> BaseProjection<H>
where
    H: Fn(&StoredEvent) -> Result<()> + Send + Sync,
{
    /// Create a new base projection.
    pub fn new(name: impl Into<String>, handler: H) -> Self {
        Self {
            name: name.into(),
            state: InMemoryProjectionState::new(),
            handler,
        }
    }

    /// Get the projection state.
    pub fn state(&self) -> &InMemoryProjectionState {
        &self.state
    }
}

#[async_trait]
impl<H> Projection for BaseProjection<H>
where
    H: Fn(&StoredEvent) -> Result<()> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    async fn handle(&self, event: &StoredEvent) -> Result<()> {
        (self.handler)(event)?;
        self.state.set_position(event.metadata.sequence);
        Ok(())
    }

    async fn reset(&self) -> Result<()> {
        self.state.clear();
        Ok(())
    }

    async fn position(&self) -> Result<u64> {
        Ok(self.state.position())
    }

    async fn set_position(&self, position: u64) -> Result<()> {
        self.state.set_position(position);
        Ok(())
    }
}

/// Projection builder for creating projections fluently.
pub struct ProjectionBuilder {
    name: String,
    state: InMemoryProjectionState,
}

impl ProjectionBuilder {
    /// Create a new projection builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: InMemoryProjectionState::new(),
        }
    }

    /// Build the projection with a handler function.
    pub fn build<H>(self, handler: H) -> BaseProjection<H>
    where
        H: Fn(&StoredEvent) -> Result<()> + Send + Sync,
    {
        BaseProjection {
            name: self.name,
            state: self.state,
            handler,
        }
    }
}

/// Catch-up subscription for replaying events to projections.
pub struct CatchUpSubscription<S>
where
    S: crate::store::EventStore,
{
    event_store: Arc<S>,
    projection: Arc<dyn Projection>,
    batch_size: usize,
}

impl<S> CatchUpSubscription<S>
where
    S: crate::store::EventStore,
{
    /// Create a new catch-up subscription.
    pub fn new(event_store: Arc<S>, projection: Arc<dyn Projection>, batch_size: usize) -> Self {
        Self {
            event_store,
            projection,
            batch_size,
        }
    }

    /// Run the catch-up subscription.
    pub async fn run(&self) -> Result<()> {
        let mut position = self.projection.position().await?;

        loop {
            let events = self
                .event_store
                .read_all(position, self.batch_size)
                .await?;

            if events.is_empty() {
                break;
            }

            for event in &events {
                self.projection.handle(event).await?;
                position = event.metadata.sequence;
            }

            if events.len() < self.batch_size {
                break;
            }
        }

        Ok(())
    }
}

/// Live subscription for processing new events in real-time.
pub struct LiveSubscription<S>
where
    S: crate::store::EventStore,
{
    event_store: Arc<S>,
    projection: Arc<dyn Projection>,
    poll_interval: std::time::Duration,
}

impl<S> LiveSubscription<S>
where
    S: crate::store::EventStore,
{
    /// Create a new live subscription.
    pub fn new(
        event_store: Arc<S>,
        projection: Arc<dyn Projection>,
        poll_interval: std::time::Duration,
    ) -> Self {
        Self {
            event_store,
            projection,
            poll_interval,
        }
    }

    /// Start the live subscription.
    pub async fn start(self) -> Result<()> {
        let mut position = self.projection.position().await?;

        loop {
            tokio::time::sleep(self.poll_interval).await;

            let events = self.event_store.read_all(position, 100).await?;

            for event in &events {
                self.projection.handle(event).await?;
                position = event.metadata.sequence;
            }
        }
    }
}

/// Read model trait for querying projected data.
#[async_trait]
pub trait ReadModel: Send + Sync {
    /// The type of the model.
    type Model;

    /// Get a model by ID.
    async fn get(&self, id: &str) -> Result<Option<Self::Model>>;

    /// List all models.
    async fn list(&self) -> Result<Vec<Self::Model>>;

    /// Query models with a filter.
    async fn query(&self, filter: &dyn std::any::Any) -> Result<Vec<Self::Model>>;
}

/// In-memory read model implementation.
pub struct InMemoryReadModel<T> {
    models: Arc<RwLock<HashMap<String, T>>>,
}

impl<T> Default for InMemoryReadModel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> InMemoryReadModel<T> {
    /// Create a new in-memory read model.
    pub fn new() -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert or update a model.
    pub fn upsert(&self, id: String, model: T) {
        self.models.write().insert(id, model);
    }

    /// Remove a model.
    pub fn remove(&self, id: &str) -> Option<T> {
        self.models.write().remove(id)
    }

    /// Clear all models.
    pub fn clear(&self) {
        self.models.write().clear();
    }
}

#[async_trait]
impl<T: Clone + Send + Sync> ReadModel for InMemoryReadModel<T> {
    type Model = T;

    async fn get(&self, id: &str) -> Result<Option<Self::Model>> {
        Ok(self.models.read().get(id).cloned())
    }

    async fn list(&self) -> Result<Vec<Self::Model>> {
        Ok(self.models.read().values().cloned().collect())
    }

    async fn query(&self, _filter: &dyn std::any::Any) -> Result<Vec<Self::Model>> {
        // Simple implementation - return all
        self.list().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventMetadata, EventVersion, StreamId};
    use serde_json::json;

    #[tokio::test]
    async fn test_projection_manager() {
        let manager = ProjectionManager::new();

        let projection = Arc::new(
            ProjectionBuilder::new("test-projection")
                .build(|_event| Ok(())),
        );

        manager.register(projection.clone());

        assert!(manager.get("test-projection").is_some());
        assert_eq!(manager.projection_names().len(), 1);
    }

    #[tokio::test]
    async fn test_in_memory_projection_state() {
        let state = InMemoryProjectionState::new();

        state.set("key1".to_string(), json!({"value": 123}));
        assert!(state.get("key1").is_some());

        state.set_position(42);
        assert_eq!(state.position(), 42);

        state.clear();
        assert!(state.get("key1").is_none());
        assert_eq!(state.position(), 0);
    }

    #[tokio::test]
    async fn test_base_projection() {
        let projection = BaseProjection::new("test", |event| {
            assert_eq!(event.metadata.sequence, 1);
            Ok(())
        });

        let event = StoredEvent::new(
            EventMetadata::new(
                StreamId::new("test"),
                1,
                EventVersion::new("TestEvent", semver::Version::new(1, 0, 0)),
            ),
            vec![],
        );

        projection.handle(&event).await.unwrap();
        assert_eq!(projection.position().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_in_memory_read_model() {
        let read_model = InMemoryReadModel::<String>::new();

        read_model.upsert("id1".to_string(), "value1".to_string());
        read_model.upsert("id2".to_string(), "value2".to_string());

        let result = read_model.get("id1").await.unwrap();
        assert_eq!(result, Some("value1".to_string()));

        let all = read_model.list().await.unwrap();
        assert_eq!(all.len(), 2);

        read_model.remove("id1");
        let result = read_model.get("id1").await.unwrap();
        assert!(result.is_none());
    }
}
