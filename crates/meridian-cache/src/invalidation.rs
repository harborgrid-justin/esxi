//! Cache invalidation strategies and policies

use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::backend::CacheBackend;
use crate::error::{CacheError, CacheResult};

/// Cache invalidation strategy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidationStrategy {
    /// Time-to-live based invalidation
    TTL,
    /// Dependency-based invalidation
    Dependency,
    /// Event-based invalidation
    Event,
    /// Manual invalidation
    Manual,
}

/// Invalidation event type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InvalidationEvent {
    /// Data updated
    DataUpdated(String),
    /// Data deleted
    DataDeleted(String),
    /// Schema changed
    SchemaChanged(String),
    /// Custom event
    Custom(String),
}

/// Dependency tracking for cache entries
#[derive(Debug, Clone)]
pub struct CacheDependency {
    /// The cache key
    pub key: String,
    /// Dependencies (other keys this depends on)
    pub depends_on: Vec<String>,
    /// Dependents (keys that depend on this)
    pub dependents: Vec<String>,
}

/// Invalidation policy
pub struct InvalidationPolicy {
    /// Strategy
    strategy: InvalidationStrategy,
    /// Default TTL for TTL strategy
    default_ttl: Option<Duration>,
    /// Dependency graph
    dependencies: Arc<DashMap<String, CacheDependency>>,
    /// Event subscriptions
    event_subscriptions: Arc<RwLock<HashMap<InvalidationEvent, HashSet<String>>>>,
}

impl InvalidationPolicy {
    /// Create a new invalidation policy with the given strategy
    pub fn new(strategy: InvalidationStrategy) -> Self {
        Self {
            strategy,
            default_ttl: None,
            dependencies: Arc::new(DashMap::new()),
            event_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a TTL-based invalidation policy
    pub fn ttl(default_ttl: Duration) -> Self {
        Self {
            strategy: InvalidationStrategy::TTL,
            default_ttl: Some(default_ttl),
            dependencies: Arc::new(DashMap::new()),
            event_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a dependency-based invalidation policy
    pub fn dependency() -> Self {
        Self::new(InvalidationStrategy::Dependency)
    }

    /// Create an event-based invalidation policy
    pub fn event() -> Self {
        Self::new(InvalidationStrategy::Event)
    }

    /// Add a dependency relationship
    pub fn add_dependency(&self, key: &str, depends_on: Vec<String>) {
        let mut dep = self
            .dependencies
            .entry(key.to_string())
            .or_insert_with(|| CacheDependency {
                key: key.to_string(),
                depends_on: Vec::new(),
                dependents: Vec::new(),
            });

        dep.depends_on = depends_on.clone();

        // Update dependents
        for parent in depends_on {
            let mut parent_dep = self
                .dependencies
                .entry(parent.clone())
                .or_insert_with(|| CacheDependency {
                    key: parent.clone(),
                    depends_on: Vec::new(),
                    dependents: Vec::new(),
                });

            if !parent_dep.dependents.contains(&key.to_string()) {
                parent_dep.dependents.push(key.to_string());
            }
        }
    }

    /// Subscribe a key to an invalidation event
    pub async fn subscribe(&self, event: InvalidationEvent, key: String) {
        let mut subs = self.event_subscriptions.write().await;
        subs.entry(event).or_insert_with(HashSet::new).insert(key);
    }

    /// Unsubscribe a key from an invalidation event
    pub async fn unsubscribe(&self, event: &InvalidationEvent, key: &str) {
        let mut subs = self.event_subscriptions.write().await;
        if let Some(keys) = subs.get_mut(event) {
            keys.remove(key);
        }
    }

    /// Get all keys that should be invalidated when a key is updated
    pub fn get_cascade_keys(&self, key: &str) -> Vec<String> {
        let mut to_invalidate = vec![key.to_string()];
        let mut visited = HashSet::new();
        visited.insert(key.to_string());

        // BFS to find all dependents
        let mut queue = vec![key.to_string()];

        while let Some(current) = queue.pop() {
            if let Some(dep) = self.dependencies.get(&current) {
                for dependent in &dep.dependents {
                    if !visited.contains(dependent) {
                        visited.insert(dependent.clone());
                        to_invalidate.push(dependent.clone());
                        queue.push(dependent.clone());
                    }
                }
            }
        }

        to_invalidate
    }

    /// Get keys subscribed to an event
    pub async fn get_event_keys(&self, event: &InvalidationEvent) -> Vec<String> {
        let subs = self.event_subscriptions.read().await;
        subs.get(event)
            .map(|keys| keys.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Invalidate cache entries based on an event
    pub async fn invalidate_event<B: CacheBackend>(
        &self,
        backend: &B,
        event: InvalidationEvent,
    ) -> CacheResult<usize> {
        let keys = self.get_event_keys(&event).await;
        let mut count = 0;

        info!("Invalidating {} keys for event {:?}", keys.len(), event);

        for key in keys {
            if backend.delete(&key).await? {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Invalidate a key and all its dependents
    pub async fn invalidate_cascade<B: CacheBackend>(
        &self,
        backend: &B,
        key: &str,
    ) -> CacheResult<usize> {
        let keys = self.get_cascade_keys(key);
        let mut count = 0;

        debug!("Cascade invalidating {} keys from '{}'", keys.len(), key);

        for key in keys {
            if backend.delete(&key).await? {
                count += 1;
            }
        }

        Ok(count)
    }
}

/// Invalidation manager for coordinating cache invalidation
pub struct InvalidationManager {
    policies: Vec<Arc<InvalidationPolicy>>,
    background_task: Option<tokio::task::JoinHandle<()>>,
}

impl InvalidationManager {
    /// Create a new invalidation manager
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            background_task: None,
        }
    }

    /// Add an invalidation policy
    pub fn add_policy(&mut self, policy: Arc<InvalidationPolicy>) {
        self.policies.push(policy);
    }

    /// Start background invalidation tasks
    pub fn start_background_tasks<B: CacheBackend + 'static>(
        &mut self,
        backend: Arc<B>,
        check_interval: Duration,
    ) {
        let policies = self.policies.clone();

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);

            loop {
                interval.tick().await;

                // Check TTL-based policies
                for policy in &policies {
                    if policy.strategy == InvalidationStrategy::TTL {
                        // Backend handles TTL automatically, but we can add additional logic here
                        debug!("Checking TTL-based invalidations");
                    }
                }
            }
        });

        self.background_task = Some(task);
    }

    /// Stop background tasks
    pub fn stop_background_tasks(&mut self) {
        if let Some(task) = self.background_task.take() {
            task.abort();
        }
    }

    /// Invalidate based on all policies
    pub async fn invalidate<B: CacheBackend>(
        &self,
        backend: &B,
        key: &str,
    ) -> CacheResult<usize> {
        let mut total = 0;

        for policy in &self.policies {
            match policy.strategy {
                InvalidationStrategy::Dependency => {
                    total += policy.invalidate_cascade(backend, key).await?;
                }
                InvalidationStrategy::Manual => {
                    if backend.delete(key).await? {
                        total += 1;
                    }
                }
                _ => {}
            }
        }

        Ok(total)
    }

    /// Trigger an invalidation event
    pub async fn trigger_event<B: CacheBackend>(
        &self,
        backend: &B,
        event: InvalidationEvent,
    ) -> CacheResult<usize> {
        let mut total = 0;

        for policy in &self.policies {
            if policy.strategy == InvalidationStrategy::Event {
                total += policy.invalidate_event(backend, event.clone()).await?;
            }
        }

        Ok(total)
    }
}

impl Default for InvalidationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Smart invalidation tracker for query results
pub struct QueryInvalidationTracker {
    /// Map of table/entity names to affected query patterns
    table_queries: Arc<DashMap<String, HashSet<String>>>,
    /// Map of query patterns to cache keys
    query_keys: Arc<DashMap<String, HashSet<String>>>,
}

impl QueryInvalidationTracker {
    /// Create a new query invalidation tracker
    pub fn new() -> Self {
        Self {
            table_queries: Arc::new(DashMap::new()),
            query_keys: Arc::new(DashMap::new()),
        }
    }

    /// Track a query pattern for a cache key
    pub fn track_query(&self, cache_key: &str, query_pattern: &str, tables: Vec<String>) {
        // Store query pattern -> cache key mapping
        self.query_keys
            .entry(query_pattern.to_string())
            .or_insert_with(HashSet::new)
            .insert(cache_key.to_string());

        // Store table -> query pattern mapping
        for table in tables {
            self.table_queries
                .entry(table)
                .or_insert_with(HashSet::new)
                .insert(query_pattern.to_string());
        }
    }

    /// Get all cache keys affected by changes to a table
    pub fn get_affected_keys(&self, table: &str) -> Vec<String> {
        let mut keys = HashSet::new();

        if let Some(queries) = self.table_queries.get(table) {
            for query_pattern in queries.iter() {
                if let Some(query_keys) = self.query_keys.get(query_pattern) {
                    keys.extend(query_keys.iter().cloned());
                }
            }
        }

        keys.into_iter().collect()
    }

    /// Invalidate all queries affected by a table change
    pub async fn invalidate_table<B: CacheBackend>(
        &self,
        backend: &B,
        table: &str,
    ) -> CacheResult<usize> {
        let keys = self.get_affected_keys(table);
        let mut count = 0;

        info!("Invalidating {} query cache keys for table '{}'", keys.len(), table);

        for key in keys {
            if backend.delete(&key).await? {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Clear all tracking data
    pub fn clear(&self) {
        self.table_queries.clear();
        self.query_keys.clear();
    }
}

impl Default for QueryInvalidationTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::memory::MemoryCache;
    use bytes::Bytes;
    use crate::backend::CacheOptions;

    #[tokio::test]
    async fn test_dependency_invalidation() -> CacheResult<()> {
        let cache = MemoryCache::lru(100);
        let policy = InvalidationPolicy::dependency();

        // Set up dependency graph
        policy.add_dependency("user:1:profile", vec!["user:1".to_string()]);
        policy.add_dependency("user:1:posts", vec!["user:1".to_string()]);

        // Add some cache entries
        cache.set("user:1", Bytes::from("user data"), CacheOptions::default()).await?;
        cache.set("user:1:profile", Bytes::from("profile data"), CacheOptions::default()).await?;
        cache.set("user:1:posts", Bytes::from("posts data"), CacheOptions::default()).await?;

        // Invalidate the parent
        let count = policy.invalidate_cascade(&cache, "user:1").await?;
        assert_eq!(count, 3); // Should invalidate all three

        Ok(())
    }

    #[tokio::test]
    async fn test_event_invalidation() -> CacheResult<()> {
        let cache = MemoryCache::lru(100);
        let policy = Arc::new(InvalidationPolicy::event());

        // Subscribe keys to events
        policy.subscribe(
            InvalidationEvent::DataUpdated("users".to_string()),
            "user:list".to_string(),
        ).await;
        policy.subscribe(
            InvalidationEvent::DataUpdated("users".to_string()),
            "user:count".to_string(),
        ).await;

        // Add cache entries
        cache.set("user:list", Bytes::from("list data"), CacheOptions::default()).await?;
        cache.set("user:count", Bytes::from("count data"), CacheOptions::default()).await?;

        // Trigger event
        let count = policy
            .invalidate_event(&cache, InvalidationEvent::DataUpdated("users".to_string()))
            .await?;
        assert_eq!(count, 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_query_invalidation_tracker() -> CacheResult<()> {
        let cache = MemoryCache::lru(100);
        let tracker = QueryInvalidationTracker::new();

        // Track some queries
        tracker.track_query(
            "query:users:all",
            "SELECT * FROM users",
            vec!["users".to_string()],
        );
        tracker.track_query(
            "query:users:active",
            "SELECT * FROM users WHERE active=true",
            vec!["users".to_string()],
        );

        // Add cache entries
        cache.set("query:users:all", Bytes::from("all users"), CacheOptions::default()).await?;
        cache.set("query:users:active", Bytes::from("active users"), CacheOptions::default()).await?;

        // Invalidate by table
        let count = tracker.invalidate_table(&cache, "users").await?;
        assert_eq!(count, 2);

        Ok(())
    }
}
