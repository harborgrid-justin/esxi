//! Tile cache with LRU eviction policy for efficient memory management.

use super::{TileCoord, TileData};
use lru::LruCache;
use parking_lot::RwLock;
use std::num::NonZeroUsize;
use std::sync::Arc;

/// Tile cache configuration.
#[derive(Debug, Clone)]
pub struct TileCacheConfig {
    /// Maximum number of tiles to cache.
    pub max_tiles: usize,
    /// Maximum cache size in bytes.
    pub max_bytes: usize,
    /// Enable persistent disk cache.
    pub disk_cache_enabled: bool,
    /// Disk cache directory.
    pub disk_cache_dir: Option<String>,
}

impl Default for TileCacheConfig {
    fn default() -> Self {
        Self {
            max_tiles: 256,
            max_bytes: 50 * 1024 * 1024, // 50 MB
            disk_cache_enabled: false,
            disk_cache_dir: None,
        }
    }
}

/// LRU tile cache for efficient memory management.
pub struct TileCache {
    /// Configuration.
    config: TileCacheConfig,
    /// LRU cache of tiles.
    cache: Arc<RwLock<LruCache<TileCoord, TileData>>>,
    /// Current cache size in bytes.
    cache_size_bytes: Arc<RwLock<usize>>,
    /// Cache statistics.
    stats: Arc<RwLock<CacheStats>>,
}

impl TileCache {
    /// Create a new tile cache.
    pub fn new(config: TileCacheConfig) -> Self {
        let cache_capacity = NonZeroUsize::new(config.max_tiles).unwrap_or(NonZeroUsize::new(256).unwrap());

        Self {
            cache: Arc::new(RwLock::new(LruCache::new(cache_capacity))),
            cache_size_bytes: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            config,
        }
    }

    /// Get a tile from the cache.
    pub fn get(&self, coord: &TileCoord) -> Option<TileData> {
        let mut stats = self.stats.write();
        stats.total_requests += 1;

        let mut cache = self.cache.write();
        if let Some(tile) = cache.get(coord) {
            stats.cache_hits += 1;
            Some(tile.clone())
        } else {
            stats.cache_misses += 1;
            None
        }
    }

    /// Put a tile into the cache.
    pub fn put(&self, tile: TileData) -> Option<TileData> {
        let tile_size = tile.size();
        let coord = tile.coord;

        let mut cache = self.cache.write();
        let mut cache_size = self.cache_size_bytes.write();

        // Check if we need to evict tiles to stay under memory limit
        while *cache_size + tile_size > self.config.max_bytes && !cache.is_empty() {
            if let Some((_, evicted_tile)) = cache.pop_lru() {
                *cache_size = cache_size.saturating_sub(evicted_tile.size());
                self.stats.write().evictions += 1;
            } else {
                break;
            }
        }

        // Add the new tile
        let evicted = cache.put(coord, tile);
        if evicted.is_none() {
            *cache_size += tile_size;
        }

        evicted
    }

    /// Check if a tile is in the cache.
    pub fn contains(&self, coord: &TileCoord) -> bool {
        self.cache.read().contains(coord)
    }

    /// Remove a tile from the cache.
    pub fn remove(&self, coord: &TileCoord) -> Option<TileData> {
        let mut cache = self.cache.write();
        if let Some(tile) = cache.pop(coord) {
            let mut cache_size = self.cache_size_bytes.write();
            *cache_size = cache_size.saturating_sub(tile.size());
            Some(tile)
        } else {
            None
        }
    }

    /// Clear all tiles from the cache.
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();

        let mut cache_size = self.cache_size_bytes.write();
        *cache_size = 0;

        let mut stats = self.stats.write();
        stats.evictions += cache.len() as u64;
    }

    /// Prune tiles older than the specified duration.
    pub fn prune_older_than(&self, max_age: std::time::Duration) {
        let mut cache = self.cache.write();
        let mut cache_size = self.cache_size_bytes.write();
        let mut stats = self.stats.write();

        let mut to_remove = Vec::new();

        // Collect old tiles (LRU cache doesn't expose iterator, so we use peek_lru)
        while let Some((coord, tile)) = cache.peek_lru() {
            if tile.age() > max_age {
                to_remove.push(*coord);
            }
            break; // Only check the least recently used
        }

        // Remove old tiles
        for coord in to_remove {
            if let Some(tile) = cache.pop(&coord) {
                *cache_size = cache_size.saturating_sub(tile.size());
                stats.evictions += 1;
            }
        }
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        *self.stats.read()
    }

    /// Get current cache size in bytes.
    pub fn size_bytes(&self) -> usize {
        *self.cache_size_bytes.read()
    }

    /// Get current number of cached tiles.
    pub fn len(&self) -> usize {
        self.cache.read().len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.read().is_empty()
    }

    /// Get cache capacity.
    pub fn capacity(&self) -> usize {
        self.config.max_tiles
    }

    /// Get cache fill percentage.
    pub fn fill_percentage(&self) -> f32 {
        (self.len() as f32 / self.capacity() as f32) * 100.0
    }

    /// Get memory usage percentage.
    pub fn memory_usage_percentage(&self) -> f32 {
        (self.size_bytes() as f32 / self.config.max_bytes as f32) * 100.0
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    /// Total cache requests.
    pub total_requests: u64,
    /// Cache hits.
    pub cache_hits: u64,
    /// Cache misses.
    pub cache_misses: u64,
    /// Number of evictions.
    pub evictions: u64,
}

impl CacheStats {
    /// Get cache hit rate as a percentage.
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Get cache miss rate as a percentage.
    pub fn miss_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.cache_misses as f64 / self.total_requests as f64) * 100.0
        }
    }
}

/// Disk cache for persistent tile storage.
#[cfg(feature = "tile-cache")]
pub struct DiskCache {
    cache_dir: std::path::PathBuf,
}

#[cfg(feature = "tile-cache")]
impl DiskCache {
    /// Create a new disk cache.
    pub fn new(cache_dir: impl Into<std::path::PathBuf>) -> std::io::Result<Self> {
        let cache_dir = cache_dir.into();
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self { cache_dir })
    }

    /// Get the cache file path for a tile.
    fn get_tile_path(&self, coord: &TileCoord) -> std::path::PathBuf {
        self.cache_dir
            .join(format!("{}", coord.z))
            .join(format!("{}", coord.x))
            .join(format!("{}.tile", coord.y))
    }

    /// Get a tile from disk cache.
    pub fn get(&self, coord: &TileCoord) -> std::io::Result<Option<TileData>> {
        let path = self.get_tile_path(coord);

        if path.exists() {
            let data = std::fs::read(&path)?;
            Ok(Some(TileData::new(*coord, data)))
        } else {
            Ok(None)
        }
    }

    /// Put a tile into disk cache.
    pub fn put(&self, tile: &TileData) -> std::io::Result<()> {
        let path = self.get_tile_path(&tile.coord);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&path, &tile.data)?;
        Ok(())
    }

    /// Clear the entire disk cache.
    pub fn clear(&self) -> std::io::Result<()> {
        std::fs::remove_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let config = TileCacheConfig::default();
        let cache = TileCache::new(config);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_put_get() {
        let config = TileCacheConfig::default();
        let cache = TileCache::new(config);

        let coord = TileCoord::new(1, 2, 3);
        let tile = TileData::new(coord, vec![1, 2, 3, 4]);

        cache.put(tile);
        assert!(cache.contains(&coord));

        let retrieved = cache.get(&coord);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_cache_stats() {
        let config = TileCacheConfig::default();
        let cache = TileCache::new(config);

        let coord = TileCoord::new(1, 2, 3);
        let tile = TileData::new(coord, vec![1, 2, 3, 4]);

        cache.put(tile);
        cache.get(&coord);
        cache.get(&TileCoord::new(9, 9, 9));

        let stats = cache.stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_rate(), 50.0);
    }

    #[test]
    fn test_cache_eviction() {
        let mut config = TileCacheConfig::default();
        config.max_tiles = 2;
        config.max_bytes = 100;

        let cache = TileCache::new(config);

        cache.put(TileData::new(TileCoord::new(1, 1, 1), vec![0; 30]));
        cache.put(TileData::new(TileCoord::new(2, 2, 2), vec![0; 30]));
        cache.put(TileData::new(TileCoord::new(3, 3, 3), vec![0; 30]));

        // First tile should be evicted
        assert!(!cache.contains(&TileCoord::new(1, 1, 1)));
        assert!(cache.contains(&TileCoord::new(2, 2, 2)));
        assert!(cache.contains(&TileCoord::new(3, 3, 3)));
    }
}
