//! Response Caching Module
//!
//! Enterprise response caching with TTL, LRU eviction, and cache policies.

pub mod policy;

use bytes::Bytes;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Cached response
#[derive(Debug, Clone)]
pub struct CachedResponse {
    /// Response body
    pub body: Bytes,

    /// Status code
    pub status: u16,

    /// Headers
    pub headers: Vec<(String, String)>,

    /// When this entry was cached
    pub cached_at: Instant,

    /// Time to live
    pub ttl: Duration,

    /// Number of hits
    pub hits: u64,
}

impl CachedResponse {
    /// Check if cache entry is expired
    pub fn is_expired(&self) -> bool {
        self.cached_at.elapsed() >= self.ttl
    }

    /// Get remaining TTL
    pub fn remaining_ttl(&self) -> Duration {
        self.ttl.saturating_sub(self.cached_at.elapsed())
    }
}

/// Cache key
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CacheKey {
    /// Request path
    pub path: String,

    /// Query string
    pub query: Option<String>,

    /// Vary headers
    pub vary: Vec<(String, String)>,
}

impl CacheKey {
    /// Create a cache key from path
    pub fn from_path(path: String) -> Self {
        Self {
            path,
            query: None,
            vary: vec![],
        }
    }

    /// Create a cache key from path and query
    pub fn from_path_query(path: String, query: Option<String>) -> Self {
        Self {
            path,
            query,
            vary: vec![],
        }
    }

    /// Add vary header
    pub fn with_vary(mut self, header: String, value: String) -> Self {
        self.vary.push((header, value));
        self.vary.sort(); // Ensure consistent ordering
        self
    }

    /// Generate cache key string
    pub fn to_string(&self) -> String {
        let mut key = self.path.clone();
        if let Some(query) = &self.query {
            key.push('?');
            key.push_str(query);
        }
        for (header, value) in &self.vary {
            key.push('|');
            key.push_str(header);
            key.push(':');
            key.push_str(value);
        }
        key
    }
}

/// Response Cache
///
/// Thread-safe in-memory cache with TTL and LRU eviction.
pub struct ResponseCache {
    /// Cache storage
    cache: Arc<DashMap<String, CachedResponse>>,

    /// Maximum cache size in bytes
    max_size: usize,

    /// Current cache size in bytes
    current_size: Arc<parking_lot::RwLock<usize>>,

    /// Default TTL
    default_ttl: Duration,
}

impl ResponseCache {
    /// Create a new response cache
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            max_size,
            current_size: Arc::new(parking_lot::RwLock::new(0)),
            default_ttl,
        }
    }

    /// Get cached response
    pub fn get(&self, key: &CacheKey) -> Option<CachedResponse> {
        let key_str = key.to_string();

        self.cache.get_mut(&key_str).and_then(|mut entry| {
            if entry.is_expired() {
                // Remove expired entry
                drop(entry);
                self.remove(key);
                None
            } else {
                // Increment hit counter
                entry.hits += 1;
                Some(entry.clone())
            }
        })
    }

    /// Put response in cache
    pub fn put(
        &self,
        key: CacheKey,
        body: Bytes,
        status: u16,
        headers: Vec<(String, String)>,
        ttl: Option<Duration>,
    ) -> bool {
        let ttl = ttl.unwrap_or(self.default_ttl);
        let key_str = key.to_string();
        let size = body.len();

        // Check if we need to evict entries
        self.evict_if_needed(size);

        let cached = CachedResponse {
            body,
            status,
            headers,
            cached_at: Instant::now(),
            ttl,
            hits: 0,
        };

        self.cache.insert(key_str, cached);

        // Update current size
        let mut current_size = self.current_size.write();
        *current_size += size;

        true
    }

    /// Remove from cache
    pub fn remove(&self, key: &CacheKey) -> Option<CachedResponse> {
        let key_str = key.to_string();

        self.cache.remove(&key_str).map(|(_, cached)| {
            let mut current_size = self.current_size.write();
            *current_size = current_size.saturating_sub(cached.body.len());
            cached
        })
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        self.cache.clear();
        let mut current_size = self.current_size.write();
        *current_size = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_entries = self.cache.len();
        let current_size = *self.current_size.read();

        let mut total_hits = 0;
        let mut expired_entries = 0;

        for entry in self.cache.iter() {
            total_hits += entry.hits;
            if entry.is_expired() {
                expired_entries += 1;
            }
        }

        CacheStats {
            total_entries,
            current_size,
            max_size: self.max_size,
            total_hits,
            expired_entries,
        }
    }

    /// Evict entries if needed to make space
    fn evict_if_needed(&self, needed_size: usize) {
        let mut current_size = *self.current_size.read();

        if current_size + needed_size <= self.max_size {
            return;
        }

        // Collect expired entries first
        let expired: Vec<String> = self
            .cache
            .iter()
            .filter(|entry| entry.is_expired())
            .map(|entry| entry.key().clone())
            .collect();

        for key in expired {
            if let Some((_, cached)) = self.cache.remove(&key) {
                current_size = current_size.saturating_sub(cached.body.len());
            }
        }

        // If still need space, evict LRU entries
        if current_size + needed_size > self.max_size {
            let mut entries: Vec<(String, Instant, usize)> = self
                .cache
                .iter()
                .map(|entry| {
                    (
                        entry.key().clone(),
                        entry.cached_at,
                        entry.body.len(),
                    )
                })
                .collect();

            // Sort by cached_at (oldest first)
            entries.sort_by_key(|(_, cached_at, _)| *cached_at);

            for (key, _, size) in entries {
                if current_size + needed_size <= self.max_size {
                    break;
                }

                self.cache.remove(&key);
                current_size = current_size.saturating_sub(size);
            }
        }

        // Update current size
        let mut size_guard = self.current_size.write();
        *size_guard = current_size;
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        let expired: Vec<String> = self
            .cache
            .iter()
            .filter(|entry| entry.is_expired())
            .map(|entry| entry.key().clone())
            .collect();

        for key in expired {
            self.cache.remove(&key);
        }
    }
}

impl Clone for ResponseCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            max_size: self.max_size,
            current_size: Arc::clone(&self.current_size),
            default_ttl: self.default_ttl,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of cache entries
    pub total_entries: usize,
    /// Current cache size in bytes
    pub current_size: usize,
    /// Maximum cache size in bytes
    pub max_size: usize,
    /// Total number of cache hits
    pub total_hits: u64,
    /// Number of expired entries
    pub expired_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_put_get() {
        let cache = ResponseCache::new(1024 * 1024, Duration::from_secs(60));

        let key = CacheKey::from_path("/api/users".to_string());
        let body = Bytes::from("test response");

        cache.put(
            key.clone(),
            body.clone(),
            200,
            vec![],
            None,
        );

        let cached = cache.get(&key).unwrap();
        assert_eq!(cached.body, body);
        assert_eq!(cached.status, 200);
        assert_eq!(cached.hits, 1);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = ResponseCache::new(1024 * 1024, Duration::from_millis(50));

        let key = CacheKey::from_path("/api/users".to_string());
        let body = Bytes::from("test response");

        cache.put(key.clone(), body, 200, vec![], None);

        // Should be in cache
        assert!(cache.get(&key).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(100));

        // Should be expired and removed
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_key_with_query() {
        let key1 = CacheKey::from_path_query(
            "/api/users".to_string(),
            Some("page=1".to_string()),
        );
        let key2 = CacheKey::from_path_query(
            "/api/users".to_string(),
            Some("page=2".to_string()),
        );

        assert_ne!(key1.to_string(), key2.to_string());
    }

    #[test]
    fn test_cache_stats() {
        let cache = ResponseCache::new(1024 * 1024, Duration::from_secs(60));

        let key = CacheKey::from_path("/api/users".to_string());
        cache.put(
            key.clone(),
            Bytes::from("test"),
            200,
            vec![],
            None,
        );

        let stats = cache.stats();
        assert_eq!(stats.total_entries, 1);
        assert!(stats.current_size > 0);
    }
}
