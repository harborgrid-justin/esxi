//! Multi-tier cache management system

use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

use crate::backend::{CacheBackend, CacheEntry, CacheOptions, WritePolicy};
use crate::error::{CacheError, CacheResult};
use crate::stats::CacheStats;

/// Cache tier configuration
#[derive(Debug, Clone)]
pub struct TierConfig {
    /// Tier name
    pub name: String,
    /// Tier priority (lower = higher priority, checked first)
    pub priority: u8,
    /// Whether to promote cache hits to higher tiers
    pub promote_on_hit: bool,
    /// Write policy
    pub write_policy: WritePolicy,
}

/// A single cache tier
pub struct CacheTier {
    /// Tier configuration
    pub config: TierConfig,
    /// The backend implementation
    pub backend: Arc<dyn CacheBackend>,
}

/// Multi-tier cache manager
pub struct TieredCache {
    /// Cache tiers, sorted by priority
    tiers: Vec<CacheTier>,
    /// Global write policy
    write_policy: WritePolicy,
}

impl TieredCache {
    /// Create a new tiered cache
    pub fn new(write_policy: WritePolicy) -> Self {
        Self {
            tiers: Vec::new(),
            write_policy,
        }
    }

    /// Add a cache tier
    pub fn add_tier(&mut self, config: TierConfig, backend: Arc<dyn CacheBackend>) {
        let tier = CacheTier { config, backend };
        self.tiers.push(tier);
        // Sort by priority
        self.tiers.sort_by_key(|t| t.config.priority);
    }

    /// Get the number of tiers
    pub fn tier_count(&self) -> usize {
        self.tiers.len()
    }

    /// Get a tier by index
    pub fn get_tier(&self, index: usize) -> Option<&CacheTier> {
        self.tiers.get(index)
    }

    /// Promote a value to higher priority tiers
    async fn promote(&self, key: &str, entry: &CacheEntry, current_tier: usize) -> CacheResult<()> {
        debug!("Promoting key '{}' from tier {}", key, current_tier);

        // Promote to all higher priority tiers
        for tier_idx in 0..current_tier {
            let tier = &self.tiers[tier_idx];
            if tier.config.promote_on_hit {
                let options = CacheOptions {
                    ttl: entry.metadata.ttl,
                    tags: entry.metadata.tags.clone(),
                    compress: false,
                };

                if let Err(e) = tier.backend.set(key, entry.value.clone(), options).await {
                    warn!(
                        "Failed to promote key '{}' to tier {}: {}",
                        key, tier_idx, e
                    );
                }
            }
        }

        Ok(())
    }

    /// Write to all tiers based on write policy
    async fn write_all_tiers(&self, key: &str, value: Bytes, options: CacheOptions) -> CacheResult<()> {
        match self.write_policy {
            WritePolicy::WriteThrough => {
                // Write to all tiers synchronously
                for tier in &self.tiers {
                    tier.backend.set(key, value.clone(), options.clone()).await?;
                }
            }
            WritePolicy::WriteBack => {
                // Write to first tier synchronously, others asynchronously
                if let Some(first_tier) = self.tiers.first() {
                    first_tier.backend.set(key, value.clone(), options.clone()).await?;

                    // Spawn background tasks for other tiers
                    for tier in self.tiers.iter().skip(1) {
                        let backend = tier.backend.clone();
                        let key = key.to_string();
                        let value = value.clone();
                        let options = options.clone();

                        tokio::spawn(async move {
                            if let Err(e) = backend.set(&key, value, options).await {
                                warn!("Background write failed: {}", e);
                            }
                        });
                    }
                }
            }
            WritePolicy::WriteAround => {
                // Write only to lowest priority tier (typically disk/persistent storage)
                if let Some(last_tier) = self.tiers.last() {
                    last_tier.backend.set(key, value, options).await?;
                }
            }
        }

        Ok(())
    }

    /// Get the effective write policy for a tier
    fn get_tier_write_policy(&self, tier: &CacheTier) -> WritePolicy {
        if tier.config.write_policy == WritePolicy::WriteThrough {
            tier.config.write_policy
        } else {
            self.write_policy
        }
    }
}

#[async_trait]
impl CacheBackend for TieredCache {
    async fn get(&self, key: &str) -> CacheResult<Option<CacheEntry>> {
        // Try each tier in priority order
        for (idx, tier) in self.tiers.iter().enumerate() {
            match tier.backend.get(key).await? {
                Some(entry) => {
                    // Found in this tier, promote to higher tiers if configured
                    if idx > 0 {
                        self.promote(key, &entry, idx).await?;
                    }
                    return Ok(Some(entry));
                }
                None => continue,
            }
        }

        // Not found in any tier
        Ok(None)
    }

    async fn set(&self, key: &str, value: Bytes, options: CacheOptions) -> CacheResult<()> {
        self.write_all_tiers(key, value, options).await
    }

    async fn delete(&self, key: &str) -> CacheResult<bool> {
        let mut deleted = false;

        // Delete from all tiers
        for tier in &self.tiers {
            if tier.backend.delete(key).await? {
                deleted = true;
            }
        }

        Ok(deleted)
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        // Check all tiers
        for tier in &self.tiers {
            if tier.backend.exists(key).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn clear(&self) -> CacheResult<()> {
        // Clear all tiers
        for tier in &self.tiers {
            tier.backend.clear().await?;
        }
        Ok(())
    }

    async fn stats(&self) -> CacheResult<CacheStats> {
        // Aggregate stats from all tiers
        let mut total_stats = CacheStats::default();

        for tier in &self.tiers {
            let tier_stats = tier.backend.stats().await?;
            total_stats.hits += tier_stats.hits;
            total_stats.misses += tier_stats.misses;
            total_stats.sets += tier_stats.sets;
            total_stats.deletes += tier_stats.deletes;
            total_stats.evictions += tier_stats.evictions;
        }

        Ok(total_stats)
    }

    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>> {
        // Collect keys from all tiers and deduplicate
        let mut all_keys = std::collections::HashSet::new();

        for tier in &self.tiers {
            let keys = tier.backend.keys(pattern).await?;
            all_keys.extend(keys);
        }

        Ok(all_keys.into_iter().collect())
    }

    async fn delete_pattern(&self, pattern: &str) -> CacheResult<usize> {
        let mut total = 0;

        for tier in &self.tiers {
            total += tier.backend.delete_pattern(pattern).await?;
        }

        Ok(total)
    }

    async fn delete_by_tags(&self, tags: &[String]) -> CacheResult<usize> {
        let mut total = 0;

        for tier in &self.tiers {
            total += tier.backend.delete_by_tags(tags).await?;
        }

        Ok(total)
    }

    async fn size(&self) -> CacheResult<usize> {
        let mut total = 0;

        for tier in &self.tiers {
            total += tier.backend.size().await?;
        }

        Ok(total)
    }

    async fn len(&self) -> CacheResult<usize> {
        // Count unique keys across all tiers
        let all_keys = self.keys("*").await?;
        Ok(all_keys.len())
    }

    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
        let mut updated = false;

        for tier in &self.tiers {
            if tier.backend.expire(key, ttl).await? {
                updated = true;
            }
        }

        Ok(updated)
    }

    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        // Return TTL from first tier that has the key
        for tier in &self.tiers {
            if let Some(ttl) = tier.backend.ttl(key).await? {
                return Ok(Some(ttl));
            }
        }
        Ok(None)
    }

    async fn mget(&self, keys: &[String]) -> CacheResult<Vec<Option<CacheEntry>>> {
        let mut results = vec![None; keys.len()];
        let mut missing_indices: Vec<usize> = (0..keys.len()).collect();

        // Try each tier in order
        for tier in &self.tiers {
            if missing_indices.is_empty() {
                break;
            }

            let missing_keys: Vec<String> = missing_indices
                .iter()
                .map(|&i| keys[i].clone())
                .collect();

            let tier_results = tier.backend.mget(&missing_keys).await?;

            // Update results and track what's still missing
            let mut new_missing = Vec::new();
            for (i, result) in tier_results.into_iter().enumerate() {
                let original_idx = missing_indices[i];
                if let Some(entry) = result {
                    results[original_idx] = Some(entry);
                } else {
                    new_missing.push(original_idx);
                }
            }

            missing_indices = new_missing;
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
        for tier in &self.tiers {
            tier.backend.flush().await?;
        }
        Ok(())
    }

    async fn close(&self) -> CacheResult<()> {
        for tier in &self.tiers {
            tier.backend.close().await?;
        }
        Ok(())
    }
}

/// Builder for creating tiered caches
pub struct TieredCacheBuilder {
    write_policy: WritePolicy,
    tiers: Vec<(TierConfig, Arc<dyn CacheBackend>)>,
}

impl TieredCacheBuilder {
    /// Create a new builder with the specified write policy
    pub fn new(write_policy: WritePolicy) -> Self {
        Self {
            write_policy,
            tiers: Vec::new(),
        }
    }

    /// Add a tier to the cache
    pub fn add_tier(mut self, config: TierConfig, backend: Arc<dyn CacheBackend>) -> Self {
        self.tiers.push((config, backend));
        self
    }

    /// Build the tiered cache
    pub fn build(self) -> TieredCache {
        let mut cache = TieredCache::new(self.write_policy);
        for (config, backend) in self.tiers {
            cache.add_tier(config, backend);
        }
        cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::memory::{MemoryCache, MemoryCacheConfig, EvictionStrategy};

    #[tokio::test]
    async fn test_tiered_cache() -> CacheResult<()> {
        let l1 = Arc::new(MemoryCache::new(MemoryCacheConfig {
            max_entries: 100,
            eviction_strategy: EvictionStrategy::LRU,
            ..Default::default()
        }));

        let l2 = Arc::new(MemoryCache::new(MemoryCacheConfig {
            max_entries: 1000,
            eviction_strategy: EvictionStrategy::LRU,
            ..Default::default()
        }));

        let cache = TieredCacheBuilder::new(WritePolicy::WriteThrough)
            .add_tier(
                TierConfig {
                    name: "L1".to_string(),
                    priority: 0,
                    promote_on_hit: true,
                    write_policy: WritePolicy::WriteThrough,
                },
                l1,
            )
            .add_tier(
                TierConfig {
                    name: "L2".to_string(),
                    priority: 1,
                    promote_on_hit: false,
                    write_policy: WritePolicy::WriteThrough,
                },
                l2,
            )
            .build();

        let key = "test_key";
        let value = Bytes::from("test_value");

        cache.set(key, value.clone(), CacheOptions::default()).await?;
        let result = cache.get(key).await?;

        assert!(result.is_some());
        assert_eq!(result.unwrap().value, value);

        Ok(())
    }
}
