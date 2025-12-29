//! In-memory cache backend with multiple eviction strategies

use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::{CacheBackend, CacheEntry, CacheMetadata, CacheOptions};
use crate::error::{CacheError, CacheResult};
use crate::stats::CacheStats;

/// Eviction strategy for memory cache
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionStrategy {
    /// Least Recently Used
    LRU,
    /// Least Frequently Used
    LFU,
    /// Adaptive Replacement Cache
    ARC,
}

/// Configuration for memory cache
#[derive(Debug, Clone)]
pub struct MemoryCacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,
    /// Maximum size in bytes
    pub max_size_bytes: usize,
    /// Eviction strategy
    pub eviction_strategy: EvictionStrategy,
    /// Default TTL for entries
    pub default_ttl: Option<Duration>,
}

impl Default for MemoryCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_size_bytes: 100 * 1024 * 1024, // 100 MB
            eviction_strategy: EvictionStrategy::LRU,
            default_ttl: Some(Duration::from_secs(3600)), // 1 hour
        }
    }
}

/// LRU-based memory cache
struct LruMemoryCache {
    cache: RwLock<LruCache<String, CacheEntry>>,
    config: MemoryCacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    current_size: Arc<parking_lot::Mutex<usize>>,
}

impl LruMemoryCache {
    fn new(config: MemoryCacheConfig) -> Self {
        let capacity = NonZeroUsize::new(config.max_entries).unwrap_or(NonZeroUsize::new(1).unwrap());
        Self {
            cache: RwLock::new(LruCache::new(capacity)),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            current_size: Arc::new(parking_lot::Mutex::new(0)),
        }
    }

    fn evict_if_needed(&self, new_entry_size: usize) -> CacheResult<()> {
        let mut size = self.current_size.lock();
        let mut cache = self.cache.write();

        // Evict entries until we have enough space
        while *size + new_entry_size > self.config.max_size_bytes && !cache.is_empty() {
            if let Some((_, entry)) = cache.pop_lru() {
                *size = size.saturating_sub(entry.metadata.size);
                self.stats.write().evictions += 1;
            } else {
                break;
            }
        }

        if *size + new_entry_size > self.config.max_size_bytes {
            return Err(CacheError::CapacityExceeded);
        }

        Ok(())
    }

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        if let Some(ttl) = entry.metadata.ttl {
            let age = chrono::Utc::now()
                .signed_duration_since(entry.metadata.created_at)
                .to_std()
                .unwrap_or(Duration::from_secs(0));
            age > ttl
        } else {
            false
        }
    }
}

/// LFU entry with frequency tracking
struct LfuEntry {
    entry: CacheEntry,
    frequency: u64,
    last_access: Instant,
}

/// LFU-based memory cache
struct LfuMemoryCache {
    cache: DashMap<String, LfuEntry>,
    config: MemoryCacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    current_size: Arc<parking_lot::Mutex<usize>>,
}

impl LfuMemoryCache {
    fn new(config: MemoryCacheConfig) -> Self {
        Self {
            cache: DashMap::new(),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            current_size: Arc::new(parking_lot::Mutex::new(0)),
        }
    }

    fn evict_if_needed(&self, new_entry_size: usize) -> CacheResult<()> {
        let mut size = self.current_size.lock();

        // Evict least frequently used entries
        while *size + new_entry_size > self.config.max_size_bytes && !self.cache.is_empty() {
            // Find entry with lowest frequency
            let min_key = self
                .cache
                .iter()
                .min_by_key(|entry| entry.value().frequency)
                .map(|entry| entry.key().clone());

            if let Some(key) = min_key {
                if let Some((_, entry)) = self.cache.remove(&key) {
                    *size = size.saturating_sub(entry.entry.metadata.size);
                    self.stats.write().evictions += 1;
                }
            } else {
                break;
            }
        }

        if *size + new_entry_size > self.config.max_size_bytes {
            return Err(CacheError::CapacityExceeded);
        }

        Ok(())
    }

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        if let Some(ttl) = entry.metadata.ttl {
            let age = chrono::Utc::now()
                .signed_duration_since(entry.metadata.created_at)
                .to_std()
                .unwrap_or(Duration::from_secs(0));
            age > ttl
        } else {
            false
        }
    }
}

/// ARC (Adaptive Replacement Cache) implementation
struct ArcMemoryCache {
    // T1: Recent cache entries (LRU)
    t1: RwLock<LruCache<String, CacheEntry>>,
    // T2: Frequent cache entries (LFU)
    t2: DashMap<String, LfuEntry>,
    // B1: Ghost entries evicted from T1
    b1: RwLock<LruCache<String, usize>>,
    // B2: Ghost entries evicted from T2
    b2: RwLock<LruCache<String, usize>>,
    // Target size for T1
    p: Arc<parking_lot::Mutex<usize>>,
    config: MemoryCacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    current_size: Arc<parking_lot::Mutex<usize>>,
}

impl ArcMemoryCache {
    fn new(config: MemoryCacheConfig) -> Self {
        let capacity = NonZeroUsize::new(config.max_entries / 2).unwrap_or(NonZeroUsize::new(1).unwrap());
        Self {
            t1: RwLock::new(LruCache::new(capacity)),
            t2: DashMap::new(),
            b1: RwLock::new(LruCache::new(capacity)),
            b2: RwLock::new(LruCache::new(capacity)),
            p: Arc::new(parking_lot::Mutex::new(0)),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            current_size: Arc::new(parking_lot::Mutex::new(0)),
        }
    }

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        if let Some(ttl) = entry.metadata.ttl {
            let age = chrono::Utc::now()
                .signed_duration_since(entry.metadata.created_at)
                .to_std()
                .unwrap_or(Duration::from_secs(0));
            age > ttl
        } else {
            false
        }
    }
}

/// In-memory cache backend
pub struct MemoryCache {
    inner: MemoryCacheInner,
}

enum MemoryCacheInner {
    Lru(LruMemoryCache),
    Lfu(LfuMemoryCache),
    Arc(ArcMemoryCache),
}

impl MemoryCache {
    /// Create a new memory cache with the given configuration
    pub fn new(config: MemoryCacheConfig) -> Self {
        let inner = match config.eviction_strategy {
            EvictionStrategy::LRU => MemoryCacheInner::Lru(LruMemoryCache::new(config)),
            EvictionStrategy::LFU => MemoryCacheInner::Lfu(LfuMemoryCache::new(config)),
            EvictionStrategy::ARC => MemoryCacheInner::Arc(ArcMemoryCache::new(config)),
        };

        Self { inner }
    }

    /// Create a new LRU memory cache
    pub fn lru(max_entries: usize) -> Self {
        Self::new(MemoryCacheConfig {
            max_entries,
            eviction_strategy: EvictionStrategy::LRU,
            ..Default::default()
        })
    }

    /// Create a new LFU memory cache
    pub fn lfu(max_entries: usize) -> Self {
        Self::new(MemoryCacheConfig {
            max_entries,
            eviction_strategy: EvictionStrategy::LFU,
            ..Default::default()
        })
    }

    /// Create a new ARC memory cache
    pub fn arc(max_entries: usize) -> Self {
        Self::new(MemoryCacheConfig {
            max_entries,
            eviction_strategy: EvictionStrategy::ARC,
            ..Default::default()
        })
    }
}

#[async_trait]
impl CacheBackend for MemoryCache {
    async fn get(&self, key: &str) -> CacheResult<Option<CacheEntry>> {
        match &self.inner {
            MemoryCacheInner::Lru(cache) => {
                let mut guard = cache.cache.write();
                if let Some(entry) = guard.get_mut(key) {
                    if cache.is_expired(entry) {
                        guard.pop(key);
                        cache.stats.write().misses += 1;
                        return Ok(None);
                    }
                    entry.metadata.last_accessed = chrono::Utc::now();
                    entry.metadata.access_count += 1;
                    cache.stats.write().hits += 1;
                    Ok(Some(entry.clone()))
                } else {
                    cache.stats.write().misses += 1;
                    Ok(None)
                }
            }
            MemoryCacheInner::Lfu(cache) => {
                if let Some(mut entry) = cache.cache.get_mut(key) {
                    if cache.is_expired(&entry.entry) {
                        drop(entry);
                        cache.cache.remove(key);
                        cache.stats.write().misses += 1;
                        return Ok(None);
                    }
                    entry.frequency += 1;
                    entry.last_access = Instant::now();
                    entry.entry.metadata.last_accessed = chrono::Utc::now();
                    entry.entry.metadata.access_count += 1;
                    cache.stats.write().hits += 1;
                    Ok(Some(entry.entry.clone()))
                } else {
                    cache.stats.write().misses += 1;
                    Ok(None)
                }
            }
            MemoryCacheInner::Arc(cache) => {
                // Check T1 (recent)
                let mut t1 = cache.t1.write();
                if let Some(entry) = t1.get_mut(key) {
                    if cache.is_expired(entry) {
                        t1.pop(key);
                        cache.stats.write().misses += 1;
                        return Ok(None);
                    }
                    let entry_clone = entry.clone();
                    // Promote to T2
                    t1.pop(key);
                    drop(t1);
                    cache.t2.insert(
                        key.to_string(),
                        LfuEntry {
                            entry: entry_clone.clone(),
                            frequency: 1,
                            last_access: Instant::now(),
                        },
                    );
                    cache.stats.write().hits += 1;
                    return Ok(Some(entry_clone));
                }
                drop(t1);

                // Check T2 (frequent)
                if let Some(mut entry) = cache.t2.get_mut(key) {
                    if cache.is_expired(&entry.entry) {
                        drop(entry);
                        cache.t2.remove(key);
                        cache.stats.write().misses += 1;
                        return Ok(None);
                    }
                    entry.frequency += 1;
                    entry.last_access = Instant::now();
                    entry.entry.metadata.last_accessed = chrono::Utc::now();
                    entry.entry.metadata.access_count += 1;
                    cache.stats.write().hits += 1;
                    Ok(Some(entry.entry.clone()))
                } else {
                    cache.stats.write().misses += 1;
                    Ok(None)
                }
            }
        }
    }

    async fn set(&self, key: &str, value: Bytes, options: CacheOptions) -> CacheResult<()> {
        let metadata = CacheMetadata {
            ttl: options.ttl,
            size: value.len(),
            tags: options.tags,
            ..Default::default()
        };

        let entry = CacheEntry {
            value,
            metadata: metadata.clone(),
        };

        match &self.inner {
            MemoryCacheInner::Lru(cache) => {
                cache.evict_if_needed(entry.metadata.size)?;
                let mut guard = cache.cache.write();

                // Remove old entry if exists
                if let Some(old) = guard.pop(key) {
                    let mut size = cache.current_size.lock();
                    *size = size.saturating_sub(old.metadata.size);
                }

                guard.put(key.to_string(), entry);
                let mut size = cache.current_size.lock();
                *size += metadata.size;
                cache.stats.write().sets += 1;
                Ok(())
            }
            MemoryCacheInner::Lfu(cache) => {
                cache.evict_if_needed(entry.metadata.size)?;

                if let Some((_, old)) = cache.cache.remove(key) {
                    let mut size = cache.current_size.lock();
                    *size = size.saturating_sub(old.entry.metadata.size);
                }

                cache.cache.insert(
                    key.to_string(),
                    LfuEntry {
                        entry,
                        frequency: 0,
                        last_access: Instant::now(),
                    },
                );
                let mut size = cache.current_size.lock();
                *size += metadata.size;
                cache.stats.write().sets += 1;
                Ok(())
            }
            MemoryCacheInner::Arc(cache) => {
                // Insert into T1 (recent)
                let mut t1 = cache.t1.write();
                if let Some(old) = t1.pop(key) {
                    let mut size = cache.current_size.lock();
                    *size = size.saturating_sub(old.metadata.size);
                }
                t1.put(key.to_string(), entry);
                let mut size = cache.current_size.lock();
                *size += metadata.size;
                cache.stats.write().sets += 1;
                Ok(())
            }
        }
    }

    async fn delete(&self, key: &str) -> CacheResult<bool> {
        match &self.inner {
            MemoryCacheInner::Lru(cache) => {
                let mut guard = cache.cache.write();
                if let Some(entry) = guard.pop(key) {
                    let mut size = cache.current_size.lock();
                    *size = size.saturating_sub(entry.metadata.size);
                    cache.stats.write().deletes += 1;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            MemoryCacheInner::Lfu(cache) => {
                if let Some((_, entry)) = cache.cache.remove(key) {
                    let mut size = cache.current_size.lock();
                    *size = size.saturating_sub(entry.entry.metadata.size);
                    cache.stats.write().deletes += 1;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            MemoryCacheInner::Arc(cache) => {
                let mut t1 = cache.t1.write();
                if let Some(entry) = t1.pop(key) {
                    let mut size = cache.current_size.lock();
                    *size = size.saturating_sub(entry.metadata.size);
                    cache.stats.write().deletes += 1;
                    return Ok(true);
                }
                drop(t1);

                if let Some((_, entry)) = cache.t2.remove(key) {
                    let mut size = cache.current_size.lock();
                    *size = size.saturating_sub(entry.entry.metadata.size);
                    cache.stats.write().deletes += 1;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        match &self.inner {
            MemoryCacheInner::Lru(cache) => {
                let guard = cache.cache.read();
                Ok(guard.contains(key))
            }
            MemoryCacheInner::Lfu(cache) => Ok(cache.cache.contains_key(key)),
            MemoryCacheInner::Arc(cache) => {
                let t1 = cache.t1.read();
                if t1.contains(key) {
                    return Ok(true);
                }
                drop(t1);
                Ok(cache.t2.contains_key(key))
            }
        }
    }

    async fn clear(&self) -> CacheResult<()> {
        match &self.inner {
            MemoryCacheInner::Lru(cache) => {
                let mut guard = cache.cache.write();
                guard.clear();
                *cache.current_size.lock() = 0;
                Ok(())
            }
            MemoryCacheInner::Lfu(cache) => {
                cache.cache.clear();
                *cache.current_size.lock() = 0;
                Ok(())
            }
            MemoryCacheInner::Arc(cache) => {
                cache.t1.write().clear();
                cache.t2.clear();
                cache.b1.write().clear();
                cache.b2.write().clear();
                *cache.current_size.lock() = 0;
                Ok(())
            }
        }
    }

    async fn stats(&self) -> CacheResult<CacheStats> {
        match &self.inner {
            MemoryCacheInner::Lru(cache) => Ok(*cache.stats.read()),
            MemoryCacheInner::Lfu(cache) => Ok(*cache.stats.read()),
            MemoryCacheInner::Arc(cache) => Ok(*cache.stats.read()),
        }
    }

    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>> {
        // Simple glob pattern matching
        let regex_pattern = pattern
            .replace(".", "\\.")
            .replace("*", ".*")
            .replace("?", ".");
        let regex = regex::Regex::new(&format!("^{}$", regex_pattern))
            .map_err(|e| CacheError::InvalidKey(e.to_string()))?;

        match &self.inner {
            MemoryCacheInner::Lru(cache) => {
                let guard = cache.cache.read();
                let keys: Vec<String> = guard
                    .iter()
                    .map(|(k, _)| k.clone())
                    .filter(|k| regex.is_match(k))
                    .collect();
                Ok(keys)
            }
            MemoryCacheInner::Lfu(cache) => {
                let keys: Vec<String> = cache
                    .cache
                    .iter()
                    .map(|entry| entry.key().clone())
                    .filter(|k| regex.is_match(k))
                    .collect();
                Ok(keys)
            }
            MemoryCacheInner::Arc(cache) => {
                let mut keys: Vec<String> = Vec::new();
                let t1 = cache.t1.read();
                keys.extend(
                    t1.iter()
                        .map(|(k, _)| k.clone())
                        .filter(|k| regex.is_match(k)),
                );
                drop(t1);
                keys.extend(
                    cache
                        .t2
                        .iter()
                        .map(|entry| entry.key().clone())
                        .filter(|k| regex.is_match(k)),
                );
                Ok(keys)
            }
        }
    }

    async fn delete_pattern(&self, pattern: &str) -> CacheResult<usize> {
        let keys = self.keys(pattern).await?;
        let mut count = 0;
        for key in keys {
            if self.delete(&key).await? {
                count += 1;
            }
        }
        Ok(count)
    }

    async fn delete_by_tags(&self, tags: &[String]) -> CacheResult<usize> {
        let mut count = 0;
        let all_keys = self.keys("*").await?;

        for key in all_keys {
            if let Some(entry) = self.get(&key).await? {
                if tags.iter().any(|tag| entry.metadata.tags.contains(tag))
                    && self.delete(&key).await? {
                        count += 1;
                    }
            }
        }
        Ok(count)
    }

    async fn size(&self) -> CacheResult<usize> {
        match &self.inner {
            MemoryCacheInner::Lru(cache) => Ok(*cache.current_size.lock()),
            MemoryCacheInner::Lfu(cache) => Ok(*cache.current_size.lock()),
            MemoryCacheInner::Arc(cache) => Ok(*cache.current_size.lock()),
        }
    }

    async fn len(&self) -> CacheResult<usize> {
        match &self.inner {
            MemoryCacheInner::Lru(cache) => Ok(cache.cache.read().len()),
            MemoryCacheInner::Lfu(cache) => Ok(cache.cache.len()),
            MemoryCacheInner::Arc(cache) => {
                Ok(cache.t1.read().len() + cache.t2.len())
            }
        }
    }

    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
        match &self.inner {
            MemoryCacheInner::Lru(cache) => {
                let mut guard = cache.cache.write();
                if let Some(entry) = guard.get_mut(key) {
                    entry.metadata.ttl = Some(ttl);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            MemoryCacheInner::Lfu(cache) => {
                if let Some(mut entry) = cache.cache.get_mut(key) {
                    entry.entry.metadata.ttl = Some(ttl);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            MemoryCacheInner::Arc(cache) => {
                let mut t1 = cache.t1.write();
                if let Some(entry) = t1.get_mut(key) {
                    entry.metadata.ttl = Some(ttl);
                    return Ok(true);
                }
                drop(t1);

                if let Some(mut entry) = cache.t2.get_mut(key) {
                    entry.entry.metadata.ttl = Some(ttl);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        if let Some(entry) = self.get(key).await? {
            Ok(entry.metadata.ttl)
        } else {
            Ok(None)
        }
    }

    async fn mget(&self, keys: &[String]) -> CacheResult<Vec<Option<CacheEntry>>> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.get(key).await?);
        }
        Ok(results)
    }

    async fn mset(&self, entries: Vec<(String, Bytes, CacheOptions)>) -> CacheResult<()> {
        for (key, value, options) in entries {
            self.set(&key, value, options).await?;
        }
        Ok(())
    }

    async fn flush(&self) -> CacheResult<()> {
        // No-op for memory cache
        Ok(())
    }

    async fn close(&self) -> CacheResult<()> {
        self.clear().await
    }
}
