//! Cache warming and preloading strategies

use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

use crate::backend::{CacheBackend, CacheOptions};
use crate::error::{CacheError, CacheResult};

/// Data source for cache warming
#[async_trait]
pub trait WarmupSource: Send + Sync {
    /// Get a list of keys to warm up
    async fn get_keys(&self) -> CacheResult<Vec<String>>;

    /// Fetch data for a specific key
    async fn fetch(&self, key: &str) -> CacheResult<Bytes>;

    /// Get cache options for a key
    fn get_options(&self, key: &str) -> CacheOptions {
        CacheOptions::default()
    }
}

/// Warmup strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarmupStrategy {
    /// Load all keys at once
    Eager,
    /// Load keys on-demand with prioritization
    Lazy,
    /// Load most frequently accessed keys first
    Frequency,
    /// Load most recently accessed keys first
    Recency,
    /// Custom priority-based loading
    Priority,
}

/// Warmup configuration
#[derive(Debug, Clone)]
pub struct WarmupConfig {
    /// Warmup strategy
    pub strategy: WarmupStrategy,
    /// Maximum concurrent warmup operations
    pub concurrency: usize,
    /// Timeout for each warmup operation
    pub timeout: Duration,
    /// Whether to continue on errors
    pub continue_on_error: bool,
    /// Batch size for warmup operations
    pub batch_size: usize,
    /// Delay between batches
    pub batch_delay: Duration,
}

impl Default for WarmupConfig {
    fn default() -> Self {
        Self {
            strategy: WarmupStrategy::Eager,
            concurrency: 10,
            timeout: Duration::from_secs(30),
            continue_on_error: true,
            batch_size: 100,
            batch_delay: Duration::from_millis(100),
        }
    }
}

/// Cache warmup manager
pub struct CacheWarmer<B: CacheBackend> {
    backend: Arc<B>,
    config: WarmupConfig,
    semaphore: Arc<Semaphore>,
}

impl<B: CacheBackend + 'static> CacheWarmer<B> {
    /// Create a new cache warmer
    pub fn new(backend: Arc<B>, config: WarmupConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.concurrency));

        Self {
            backend,
            config,
            semaphore,
        }
    }

    /// Warm up the cache using the given source
    pub async fn warmup<S: WarmupSource + 'static>(&self, source: Arc<S>) -> CacheResult<WarmupStats> {
        let start = Instant::now();
        let mut stats = WarmupStats::default();

        info!("Starting cache warmup with strategy: {:?}", self.config.strategy);

        // Get all keys to warm up
        let keys = source.get_keys().await?;
        stats.total_keys = keys.len();

        info!("Warming up {} keys", keys.len());

        // Process keys based on strategy
        match self.config.strategy {
            WarmupStrategy::Eager => {
                self.warmup_eager(source, keys, &mut stats).await?;
            }
            WarmupStrategy::Lazy | WarmupStrategy::Frequency | WarmupStrategy::Recency => {
                self.warmup_prioritized(source, keys, &mut stats).await?;
            }
            WarmupStrategy::Priority => {
                self.warmup_priority(source, keys, &mut stats).await?;
            }
        }

        stats.duration = start.elapsed();
        info!(
            "Cache warmup completed: {} succeeded, {} failed in {:?}",
            stats.succeeded, stats.failed, stats.duration
        );

        Ok(stats)
    }

    /// Eager warmup - load all keys as fast as possible
    async fn warmup_eager<S: WarmupSource + 'static>(
        &self,
        source: Arc<S>,
        keys: Vec<String>,
        stats: &mut WarmupStats,
    ) -> CacheResult<()> {
        // Process in batches
        for chunk in keys.chunks(self.config.batch_size) {
            let mut tasks = Vec::new();

            for key in chunk {
                let source = source.clone();
                let backend = self.backend.clone();
                let semaphore = self.semaphore.clone();
                let key = key.clone();
                let timeout = self.config.timeout;
                let continue_on_error = self.config.continue_on_error;

                let task = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();

                    let result = tokio::time::timeout(timeout, async {
                        let data = source.fetch(&key).await?;
                        let options = source.get_options(&key);
                        backend.set(&key, data, options).await?;
                        Ok::<_, CacheError>(())
                    })
                    .await;

                    match result {
                        Ok(Ok(())) => Ok(()),
                        Ok(Err(e)) => {
                            if continue_on_error {
                                warn!("Failed to warm up key '{}': {}", key, e);
                                Err(e)
                            } else {
                                Err(e)
                            }
                        }
                        Err(_) => {
                            if continue_on_error {
                                warn!("Timeout warming up key '{}'", key);
                                Err(CacheError::Timeout)
                            } else {
                                Err(CacheError::Timeout)
                            }
                        }
                    }
                });

                tasks.push(task);
            }

            // Wait for batch to complete
            for task in tasks {
                match task.await {
                    Ok(Ok(())) => stats.succeeded += 1,
                    Ok(Err(_)) => stats.failed += 1,
                    Err(_) => stats.failed += 1,
                }
            }

            // Delay between batches
            if self.config.batch_delay > Duration::from_millis(0) {
                tokio::time::sleep(self.config.batch_delay).await;
            }

            debug!(
                "Warmup progress: {}/{} keys processed",
                stats.succeeded + stats.failed,
                stats.total_keys
            );
        }

        Ok(())
    }

    /// Prioritized warmup - load keys based on priority
    async fn warmup_prioritized<S: WarmupSource + 'static>(
        &self,
        source: Arc<S>,
        keys: Vec<String>,
        stats: &mut WarmupStats,
    ) -> CacheResult<()> {
        // For now, use eager strategy
        // In a real implementation, we would sort keys by priority
        self.warmup_eager(source, keys, stats).await
    }

    /// Priority-based warmup with custom ordering
    async fn warmup_priority<S: WarmupSource + 'static>(
        &self,
        source: Arc<S>,
        keys: Vec<String>,
        stats: &mut WarmupStats,
    ) -> CacheResult<()> {
        // For now, use eager strategy
        // In a real implementation, we would allow custom priority functions
        self.warmup_eager(source, keys, stats).await
    }

    /// Refresh a single key in the cache
    pub async fn refresh<S: WarmupSource>(
        &self,
        source: Arc<S>,
        key: &str,
    ) -> CacheResult<()> {
        let data = source.fetch(key).await?;
        let options = source.get_options(key);
        self.backend.set(key, data, options).await?;
        Ok(())
    }

    /// Start background warmup task
    pub fn start_background_warmup<S: WarmupSource + 'static>(
        &self,
        source: Arc<S>,
        interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        let backend = self.backend.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                info!("Starting scheduled cache warmup");

                let warmer = CacheWarmer::new(backend.clone(), config.clone());
                match warmer.warmup(source.clone()).await {
                    Ok(stats) => {
                        info!("Scheduled warmup completed: {:?}", stats);
                    }
                    Err(e) => {
                        warn!("Scheduled warmup failed: {}", e);
                    }
                }
            }
        })
    }
}

/// Statistics from a warmup operation
#[derive(Debug, Clone, Default)]
pub struct WarmupStats {
    /// Total number of keys to warm up
    pub total_keys: usize,
    /// Number of keys successfully warmed up
    pub succeeded: usize,
    /// Number of keys that failed to warm up
    pub failed: usize,
    /// Total duration of warmup
    pub duration: Duration,
}

impl WarmupStats {
    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_keys == 0 {
            0.0
        } else {
            (self.succeeded as f64 / self.total_keys as f64) * 100.0
        }
    }

    /// Get the throughput (keys per second)
    pub fn throughput(&self) -> f64 {
        if self.duration.as_secs() == 0 {
            0.0
        } else {
            self.succeeded as f64 / self.duration.as_secs_f64()
        }
    }
}

/// Preloader for commonly accessed data patterns
pub struct CachePreloader<B: CacheBackend> {
    backend: Arc<B>,
    patterns: Arc<parking_lot::RwLock<Vec<PreloadPattern>>>,
}

/// A preload pattern definition
#[derive(Debug, Clone)]
pub struct PreloadPattern {
    /// Pattern name
    pub name: String,
    /// Key pattern or template
    pub key_pattern: String,
    /// Whether this pattern is enabled
    pub enabled: bool,
    /// Priority (higher = loaded first)
    pub priority: u8,
}

impl<B: CacheBackend + 'static> CachePreloader<B> {
    /// Create a new cache preloader
    pub fn new(backend: Arc<B>) -> Self {
        Self {
            backend,
            patterns: Arc::new(parking_lot::RwLock::new(Vec::new())),
        }
    }

    /// Add a preload pattern
    pub fn add_pattern(&self, pattern: PreloadPattern) {
        let mut patterns = self.patterns.write();
        patterns.push(pattern);
        patterns.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove a preload pattern
    pub fn remove_pattern(&self, name: &str) {
        let mut patterns = self.patterns.write();
        patterns.retain(|p| p.name != name);
    }

    /// Get all patterns
    pub fn get_patterns(&self) -> Vec<PreloadPattern> {
        self.patterns.read().clone()
    }

    /// Preload data based on registered patterns
    pub async fn preload<S: WarmupSource + 'static>(&self, source: Arc<S>) -> CacheResult<WarmupStats> {
        let patterns = self.get_patterns();
        let mut all_keys = Vec::new();

        // Collect keys matching patterns
        for pattern in patterns {
            if !pattern.enabled {
                continue;
            }

            let keys = self.backend.keys(&pattern.key_pattern).await?;
            all_keys.extend(keys);
        }

        // Remove duplicates
        all_keys.sort();
        all_keys.dedup();

        // Warm up the cache
        let warmer = CacheWarmer::new(self.backend.clone(), WarmupConfig::default());
        warmer.warmup(source).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::memory::MemoryCache;

    struct MockWarmupSource {
        data: Arc<parking_lot::RwLock<std::collections::HashMap<String, Bytes>>>,
    }

    impl MockWarmupSource {
        fn new() -> Self {
            let mut data = std::collections::HashMap::new();
            data.insert("key1".to_string(), Bytes::from("value1"));
            data.insert("key2".to_string(), Bytes::from("value2"));
            data.insert("key3".to_string(), Bytes::from("value3"));

            Self {
                data: Arc::new(parking_lot::RwLock::new(data)),
            }
        }
    }

    #[async_trait]
    impl WarmupSource for MockWarmupSource {
        async fn get_keys(&self) -> CacheResult<Vec<String>> {
            Ok(self.data.read().keys().cloned().collect())
        }

        async fn fetch(&self, key: &str) -> CacheResult<Bytes> {
            self.data
                .read()
                .get(key)
                .cloned()
                .ok_or_else(|| CacheError::KeyNotFound(key.to_string()))
        }
    }

    #[tokio::test]
    async fn test_cache_warmup() -> CacheResult<()> {
        let cache = Arc::new(MemoryCache::lru(100));
        let source = Arc::new(MockWarmupSource::new());
        let warmer = CacheWarmer::new(cache.clone(), WarmupConfig::default());

        let stats = warmer.warmup(source).await?;

        assert_eq!(stats.total_keys, 3);
        assert_eq!(stats.succeeded, 3);
        assert_eq!(stats.failed, 0);

        // Verify data was cached
        assert!(cache.exists("key1").await?);
        assert!(cache.exists("key2").await?);
        assert!(cache.exists("key3").await?);

        Ok(())
    }
}
