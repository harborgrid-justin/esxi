//! Tile response caching

use crate::tile::coordinate::TileCoordinate;
use moka::future::Cache;
use std::sync::Arc;

/// Tile cache using LRU eviction
pub struct TileCache {
    cache: Cache<TileCoordinate, Arc<Vec<u8>>>,
}

impl TileCache {
    /// Create a new tile cache
    pub fn new(max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .build();

        Self { cache }
    }

    /// Get a cached tile
    pub async fn get(&self, tile: &TileCoordinate) -> Option<Vec<u8>> {
        self.cache.get(tile).await.map(|arc| (*arc).clone())
    }

    /// Put a tile in the cache
    pub async fn put(&self, tile: TileCoordinate, data: Vec<u8>) {
        self.cache.insert(tile, Arc::new(data)).await;
    }

    /// Remove a tile from the cache
    pub async fn remove(&self, tile: &TileCoordinate) {
        self.cache.invalidate(tile).await;
    }

    /// Clear the entire cache
    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.cache.entry_count(),
            weighted_size: self.cache.weighted_size(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries in cache
    pub entry_count: u64,
    /// Total weighted size
    pub weighted_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tile_cache() {
        let cache = TileCache::new(100);
        let tile = TileCoordinate::new(10, 512, 384);
        let data = vec![1, 2, 3, 4, 5];

        // Put and get
        cache.put(tile, data.clone()).await;
        let retrieved = cache.get(&tile).await;
        assert_eq!(retrieved, Some(data));

        // Remove
        cache.remove(&tile).await;
        assert!(cache.get(&tile).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = TileCache::new(100);
        let tile = TileCoordinate::new(10, 512, 384);
        cache.put(tile, vec![1, 2, 3]).await;

        cache.clear().await;
        assert!(cache.get(&tile).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = TileCache::new(100);
        let tile = TileCoordinate::new(10, 512, 384);
        cache.put(tile, vec![1, 2, 3]).await;

        let stats = cache.stats().await;
        assert!(stats.entry_count > 0);
    }
}
