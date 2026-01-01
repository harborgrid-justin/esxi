//! Query Plan Caching and Reuse
//!
//! Caches compiled query plans to avoid re-optimization for repeated queries.

use crate::plan::{LogicalPlan, PhysicalPlan};
use dashmap::DashMap;
use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::Arc;

/// Plan cache configuration
#[derive(Debug, Clone)]
pub struct PlanCacheConfig {
    /// Maximum number of cached plans
    pub max_size: usize,
    /// Enable statistics-based cache invalidation
    pub enable_stats_invalidation: bool,
    /// Maximum age of cached plans (seconds)
    pub max_age_seconds: u64,
}

impl Default for PlanCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            enable_stats_invalidation: true,
            max_age_seconds: 3600, // 1 hour
        }
    }
}

/// Cached plan entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPlan {
    pub query_hash: u64,
    pub physical_plan: PhysicalPlan,
    pub created_at: i64,
    pub last_used: i64,
    pub use_count: u64,
    pub avg_execution_time_ms: f64,
}

impl CachedPlan {
    pub fn new(query_hash: u64, physical_plan: PhysicalPlan) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            query_hash,
            physical_plan,
            created_at: now,
            last_used: now,
            use_count: 0,
            avg_execution_time_ms: 0.0,
        }
    }

    pub fn record_usage(&mut self, execution_time_ms: f64) {
        self.last_used = chrono::Utc::now().timestamp();
        self.use_count += 1;

        // Update moving average
        if self.use_count == 1 {
            self.avg_execution_time_ms = execution_time_ms;
        } else {
            let alpha = 0.2; // Exponential moving average factor
            self.avg_execution_time_ms = alpha * execution_time_ms
                + (1.0 - alpha) * self.avg_execution_time_ms;
        }
    }

    pub fn is_expired(&self, max_age_seconds: u64) -> bool {
        let now = chrono::Utc::now().timestamp();
        (now - self.created_at) as u64 > max_age_seconds
    }
}

/// Thread-safe plan cache with LRU eviction
pub struct PlanCache {
    config: PlanCacheConfig,
    cache: Arc<RwLock<LruCache<u64, CachedPlan>>>,
    stats: Arc<CacheStatistics>,
}

impl PlanCache {
    pub fn new(config: PlanCacheConfig) -> Self {
        let cache_size = NonZeroUsize::new(config.max_size).unwrap();
        Self {
            config,
            cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
            stats: Arc::new(CacheStatistics::default()),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(PlanCacheConfig::default())
    }

    /// Get cached plan for query
    pub fn get(&self, query: &str) -> Option<PhysicalPlan> {
        let query_hash = self.hash_query(query);
        let mut cache = self.cache.write();

        if let Some(cached) = cache.get_mut(&query_hash) {
            // Check if expired
            if cached.is_expired(self.config.max_age_seconds) {
                cache.pop(&query_hash);
                self.stats.record_miss();
                return None;
            }

            self.stats.record_hit();
            Some(cached.physical_plan.clone())
        } else {
            self.stats.record_miss();
            None
        }
    }

    /// Put plan in cache
    pub fn put(&self, query: &str, plan: PhysicalPlan) {
        let query_hash = self.hash_query(query);
        let cached_plan = CachedPlan::new(query_hash, plan);

        let mut cache = self.cache.write();
        cache.put(query_hash, cached_plan);
        self.stats.record_insertion();
    }

    /// Record execution time for cached plan
    pub fn record_execution(&self, query: &str, execution_time_ms: f64) {
        let query_hash = self.hash_query(query);
        let mut cache = self.cache.write();

        if let Some(cached) = cache.get_mut(&query_hash) {
            cached.record_usage(execution_time_ms);
        }
    }

    /// Invalidate cache for specific table
    pub fn invalidate_table(&self, table: &str) {
        // For now, clear entire cache
        // In production, would track table dependencies
        self.clear();
        tracing::info!("Cache invalidated for table: {}", table);
    }

    /// Clear entire cache
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        self.stats.record_clear();
    }

    /// Get cache statistics
    pub fn statistics(&self) -> CacheStats {
        self.stats.get_stats()
    }

    /// Hash query to cache key
    fn hash_query(&self, query: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        hasher.finish()
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.cache.read().len()
    }

    /// Get cache capacity
    pub fn capacity(&self) -> usize {
        self.config.max_size
    }
}

/// Cache statistics
#[derive(Debug, Default)]
struct CacheStatistics {
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
    insertions: Arc<RwLock<u64>>,
    evictions: Arc<RwLock<u64>>,
    clears: Arc<RwLock<u64>>,
}

impl CacheStatistics {
    fn record_hit(&self) {
        *self.hits.write() += 1;
    }

    fn record_miss(&self) {
        *self.misses.write() += 1;
    }

    fn record_insertion(&self) {
        *self.insertions.write() += 1;
    }

    fn record_eviction(&self) {
        *self.evictions.write() += 1;
    }

    fn record_clear(&self) {
        *self.clears.write() += 1;
    }

    fn get_stats(&self) -> CacheStats {
        let hits = *self.hits.read();
        let misses = *self.misses.read();
        let total_requests = hits + misses;

        CacheStats {
            hits,
            misses,
            hit_rate: if total_requests > 0 {
                hits as f64 / total_requests as f64
            } else {
                0.0
            },
            insertions: *self.insertions.read(),
            evictions: *self.evictions.read(),
            clears: *self.clears.read(),
        }
    }
}

/// Cache statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub insertions: u64,
    pub evictions: u64,
    pub clears: u64,
}

/// Prepared statement cache for parameterized queries
pub struct PreparedStatementCache {
    cache: DashMap<String, PreparedStatement>,
    config: PlanCacheConfig,
}

impl PreparedStatementCache {
    pub fn new(config: PlanCacheConfig) -> Self {
        Self {
            cache: DashMap::new(),
            config,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(PlanCacheConfig::default())
    }

    /// Prepare a statement
    pub fn prepare(&self, name: String, query: String, plan: PhysicalPlan) {
        let prepared = PreparedStatement {
            name: name.clone(),
            query,
            plan,
            created_at: chrono::Utc::now().timestamp(),
            use_count: 0,
        };

        self.cache.insert(name, prepared);
    }

    /// Get prepared statement
    pub fn get(&self, name: &str) -> Option<PhysicalPlan> {
        self.cache.get_mut(name).map(|mut entry| {
            entry.use_count += 1;
            entry.plan.clone()
        })
    }

    /// Deallocate prepared statement
    pub fn deallocate(&self, name: &str) -> bool {
        self.cache.remove(name).is_some()
    }

    /// Clear all prepared statements
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get count of prepared statements
    pub fn count(&self) -> usize {
        self.cache.len()
    }
}

/// Prepared statement
#[derive(Debug, Clone)]
struct PreparedStatement {
    name: String,
    query: String,
    plan: PhysicalPlan,
    created_at: i64,
    use_count: u64,
}

/// Query result cache for materialized results
pub struct ResultCache {
    cache: DashMap<u64, CachedResult>,
    config: PlanCacheConfig,
}

impl ResultCache {
    pub fn new(config: PlanCacheConfig) -> Self {
        Self {
            cache: DashMap::new(),
            config,
        }
    }

    /// Cache query results
    pub fn put(&self, query: &str, result: Vec<u8>) {
        let query_hash = self.hash_query(query);
        let cached = CachedResult {
            query_hash,
            result,
            created_at: chrono::Utc::now().timestamp(),
            size_bytes: 0, // Would calculate actual size
        };

        self.cache.insert(query_hash, cached);
    }

    /// Get cached results
    pub fn get(&self, query: &str) -> Option<Vec<u8>> {
        let query_hash = self.hash_query(query);

        self.cache.get(&query_hash).and_then(|entry| {
            if self.is_expired(&entry.value()) {
                drop(entry);
                self.cache.remove(&query_hash);
                None
            } else {
                Some(entry.result.clone())
            }
        })
    }

    /// Check if cached result is expired
    fn is_expired(&self, result: &CachedResult) -> bool {
        let now = chrono::Utc::now().timestamp();
        (now - result.created_at) as u64 > self.config.max_age_seconds
    }

    fn hash_query(&self, query: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        hasher.finish()
    }

    /// Clear cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

/// Cached query result
#[derive(Debug, Clone)]
struct CachedResult {
    query_hash: u64,
    result: Vec<u8>,
    created_at: i64,
    size_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{Cost, PhysicalNode, PhysicalOp, Schema, Cardinality};

    fn create_test_plan() -> PhysicalPlan {
        let node = PhysicalNode::new(
            PhysicalOp::SeqScan {
                table: "test".to_string(),
                alias: None,
                predicates: vec![],
                projection: None,
            },
            vec![],
            Schema::empty(),
            Cost::zero(),
            Cardinality::new(100.0),
        );

        PhysicalPlan::new(node, Cost::zero())
    }

    #[test]
    fn test_plan_cache() {
        let cache = PlanCache::with_default_config();
        let query = "SELECT * FROM users WHERE id = 1";
        let plan = create_test_plan();

        // Cache miss
        assert!(cache.get(query).is_none());

        // Put plan
        cache.put(query, plan.clone());

        // Cache hit
        assert!(cache.get(query).is_some());

        // Statistics
        let stats = cache.statistics();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.insertions, 1);
    }

    #[test]
    fn test_prepared_statement_cache() {
        let cache = PreparedStatementCache::with_default_config();
        let plan = create_test_plan();

        cache.prepare("stmt1".to_string(), "SELECT * FROM users".to_string(), plan);

        assert!(cache.get("stmt1").is_some());
        assert!(cache.get("nonexistent").is_none());

        assert!(cache.deallocate("stmt1"));
        assert!(cache.get("stmt1").is_none());
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = PlanCache::with_default_config();
        let query = "SELECT * FROM users";
        let plan = create_test_plan();

        cache.put(query, plan);
        assert_eq!(cache.size(), 1);

        cache.invalidate_table("users");
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_result_cache() {
        let cache = ResultCache::new(PlanCacheConfig::default());
        let query = "SELECT * FROM users";
        let result = vec![1, 2, 3, 4];

        cache.put(query, result.clone());
        assert_eq!(cache.get(query), Some(result));
    }
}
