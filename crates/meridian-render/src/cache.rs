//! Tile caching system with in-memory and disk-based storage

use crate::error::RenderResult;
use crate::tile::TileCoord;
use lru::LruCache;
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Cached tile data
#[derive(Clone, Debug)]
pub struct CachedTile {
    /// Tile coordinate
    pub coord: TileCoord,
    /// Tile data (encoded image or MVT)
    pub data: Vec<u8>,
    /// Content type (e.g., "image/png", "application/x-protobuf")
    pub content_type: String,
    /// Cache timestamp
    pub cached_at: SystemTime,
    /// Optional ETag for HTTP caching
    pub etag: Option<String>,
}

impl CachedTile {
    /// Create a new cached tile
    pub fn new(coord: TileCoord, data: Vec<u8>, content_type: String) -> Self {
        CachedTile {
            coord,
            data,
            content_type,
            cached_at: SystemTime::now(),
            etag: None,
        }
    }

    /// Set ETag
    pub fn with_etag(mut self, etag: String) -> Self {
        self.etag = Some(etag);
        self
    }

    /// Check if tile is expired
    pub fn is_expired(&self, max_age: Duration) -> bool {
        match SystemTime::now().duration_since(self.cached_at) {
            Ok(age) => age > max_age,
            Err(_) => true,
        }
    }

    /// Get age of cached tile
    pub fn age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.cached_at)
            .unwrap_or(Duration::from_secs(0))
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Total evictions
    pub evictions: u64,
    /// Total size in bytes
    pub size_bytes: u64,
    /// Number of cached items
    pub item_count: usize,
}

impl CacheStats {
    /// Calculate hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// In-memory LRU cache for tiles
pub struct MemoryCache {
    cache: Arc<Mutex<LruCache<TileCoord, CachedTile>>>,
    stats: Arc<Mutex<CacheStats>>,
    max_age: Duration,
}

impl MemoryCache {
    /// Create a new memory cache with given capacity
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1000).unwrap());
        MemoryCache {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            stats: Arc::new(Mutex::new(CacheStats::default())),
            max_age: Duration::from_secs(3600), // 1 hour default
        }
    }

    /// Set maximum age for cached items
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }

    /// Get a tile from cache
    pub fn get(&self, coord: &TileCoord) -> Option<CachedTile> {
        let mut cache = self.cache.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        if let Some(tile) = cache.get(coord) {
            if !tile.is_expired(self.max_age) {
                stats.hits += 1;
                return Some(tile.clone());
            }
            // Expired, remove it
            cache.pop(coord);
        }

        stats.misses += 1;
        None
    }

    /// Put a tile into cache
    pub fn put(&self, tile: CachedTile) {
        let mut cache = self.cache.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let tile_size = tile.data.len() as u64;

        if let Some(old_tile) = cache.put(tile.coord, tile) {
            stats.evictions += 1;
            stats.size_bytes = stats.size_bytes.saturating_sub(old_tile.data.len() as u64);
        }

        stats.size_bytes += tile_size;
        stats.item_count = cache.len();
    }

    /// Clear all cached tiles
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        cache.clear();
        stats.size_bytes = 0;
        stats.item_count = 0;
    }

    /// Remove expired tiles
    pub fn purge_expired(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let expired: Vec<TileCoord> = cache
            .iter()
            .filter(|(_, tile)| tile.is_expired(self.max_age))
            .map(|(coord, _)| *coord)
            .collect();

        for coord in expired {
            if let Some(tile) = cache.pop(&coord) {
                stats.size_bytes = stats.size_bytes.saturating_sub(tile.data.len() as u64);
                stats.evictions += 1;
            }
        }

        stats.item_count = cache.len();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.hits = 0;
        stats.misses = 0;
        stats.evictions = 0;
    }
}

/// Disk-based cache for persistent tile storage
pub struct DiskCache {
    base_path: PathBuf,
    stats: Arc<Mutex<CacheStats>>,
    max_age: Duration,
}

impl DiskCache {
    /// Create a new disk cache at the given path
    pub fn new<P: AsRef<Path>>(base_path: P) -> RenderResult<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path)?;

        Ok(DiskCache {
            base_path,
            stats: Arc::new(Mutex::new(CacheStats::default())),
            max_age: Duration::from_secs(86400), // 24 hours default
        })
    }

    /// Set maximum age for cached items
    pub fn with_max_age(mut self, max_age: Duration) -> Self {
        self.max_age = max_age;
        self
    }

    /// Get the file path for a tile
    fn tile_path(&self, coord: &TileCoord) -> PathBuf {
        self.base_path
            .join(format!("{}", coord.z))
            .join(format!("{}", coord.x))
            .join(format!("{}.tile", coord.y))
    }

    /// Get a tile from disk cache
    pub fn get(&self, coord: &TileCoord) -> RenderResult<Option<CachedTile>> {
        let path = self.tile_path(coord);
        let mut stats = self.stats.lock().unwrap();

        if !path.exists() {
            stats.misses += 1;
            return Ok(None);
        }

        // Check file age
        let metadata = fs::metadata(&path)?;
        let modified = metadata.modified()?;
        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or(Duration::from_secs(u64::MAX));

        if age > self.max_age {
            // Expired, remove it
            let _ = fs::remove_file(&path);
            stats.misses += 1;
            return Ok(None);
        }

        // Read tile data
        let data = fs::read(&path)?;

        // Read metadata file if it exists
        let meta_path = path.with_extension("meta");
        let content_type = if meta_path.exists() {
            fs::read_to_string(&meta_path).unwrap_or_else(|_| "application/octet-stream".to_string())
        } else {
            "application/octet-stream".to_string()
        };

        stats.hits += 1;

        Ok(Some(CachedTile {
            coord: *coord,
            data,
            content_type,
            cached_at: modified,
            etag: None,
        }))
    }

    /// Put a tile into disk cache
    pub fn put(&self, tile: &CachedTile) -> RenderResult<()> {
        let path = self.tile_path(&tile.coord);

        // Create parent directories
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write tile data
        fs::write(&path, &tile.data)?;

        // Write metadata
        let meta_path = path.with_extension("meta");
        fs::write(&meta_path, &tile.content_type)?;

        let mut stats = self.stats.lock().unwrap();
        stats.size_bytes += tile.data.len() as u64;
        stats.item_count += 1;

        Ok(())
    }

    /// Remove a tile from cache
    pub fn remove(&self, coord: &TileCoord) -> RenderResult<bool> {
        let path = self.tile_path(coord);

        if !path.exists() {
            return Ok(false);
        }

        let size = fs::metadata(&path)?.len();
        fs::remove_file(&path)?;

        // Remove metadata file
        let meta_path = path.with_extension("meta");
        if meta_path.exists() {
            let _ = fs::remove_file(&meta_path);
        }

        let mut stats = self.stats.lock().unwrap();
        stats.size_bytes = stats.size_bytes.saturating_sub(size);
        stats.item_count = stats.item_count.saturating_sub(1);

        Ok(true)
    }

    /// Clear entire cache
    pub fn clear(&self) -> RenderResult<()> {
        fs::remove_dir_all(&self.base_path)?;
        fs::create_dir_all(&self.base_path)?;

        let mut stats = self.stats.lock().unwrap();
        stats.size_bytes = 0;
        stats.item_count = 0;
        stats.evictions = 0;

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }
}

/// Two-tier cache with memory and disk backing
pub struct TileCache {
    memory: MemoryCache,
    disk: Option<DiskCache>,
}

impl TileCache {
    /// Create a new tile cache with memory only
    pub fn memory_only(capacity: usize) -> Self {
        TileCache {
            memory: MemoryCache::new(capacity),
            disk: None,
        }
    }

    /// Create a new tile cache with memory and disk backing
    pub fn with_disk<P: AsRef<Path>>(
        memory_capacity: usize,
        disk_path: P,
    ) -> RenderResult<Self> {
        Ok(TileCache {
            memory: MemoryCache::new(memory_capacity),
            disk: Some(DiskCache::new(disk_path)?),
        })
    }

    /// Get a tile from cache (checks memory, then disk)
    pub fn get(&self, coord: &TileCoord) -> RenderResult<Option<CachedTile>> {
        // Check memory first
        if let Some(tile) = self.memory.get(coord) {
            return Ok(Some(tile));
        }

        // Check disk if available
        if let Some(disk) = &self.disk {
            if let Some(tile) = disk.get(coord)? {
                // Promote to memory cache
                self.memory.put(tile.clone());
                return Ok(Some(tile));
            }
        }

        Ok(None)
    }

    /// Put a tile into cache (stores in both memory and disk)
    pub fn put(&self, tile: CachedTile) -> RenderResult<()> {
        self.memory.put(tile.clone());

        if let Some(disk) = &self.disk {
            disk.put(&tile)?;
        }

        Ok(())
    }

    /// Clear all caches
    pub fn clear(&self) -> RenderResult<()> {
        self.memory.clear();

        if let Some(disk) = &self.disk {
            disk.clear()?;
        }

        Ok(())
    }

    /// Get combined statistics
    pub fn stats(&self) -> CacheStats {
        let mut stats = self.memory.stats();

        if let Some(disk) = &self.disk {
            let disk_stats = disk.stats();
            stats.size_bytes += disk_stats.size_bytes;
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_cache() {
        let cache = MemoryCache::new(10);
        let coord = TileCoord::new(10, 512, 384).unwrap();
        let tile = CachedTile::new(coord, vec![1, 2, 3], "image/png".to_string());

        cache.put(tile.clone());
        let cached = cache.get(&coord).unwrap();
        assert_eq!(cached.data, vec![1, 2, 3]);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_miss() {
        let cache = MemoryCache::new(10);
        let coord = TileCoord::new(10, 512, 384).unwrap();

        assert!(cache.get(&coord).is_none());

        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_expiry() {
        let cache = MemoryCache::new(10).with_max_age(Duration::from_millis(10));
        let coord = TileCoord::new(10, 512, 384).unwrap();
        let tile = CachedTile::new(coord, vec![1, 2, 3], "image/png".to_string());

        cache.put(tile);

        // Sleep to let it expire
        std::thread::sleep(Duration::from_millis(20));

        assert!(cache.get(&coord).is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = MemoryCache::new(2);

        let tile1 = CachedTile::new(
            TileCoord::new(10, 0, 0).unwrap(),
            vec![1; 100],
            "image/png".to_string(),
        );
        let tile2 = CachedTile::new(
            TileCoord::new(10, 1, 0).unwrap(),
            vec![2; 200],
            "image/png".to_string(),
        );
        let tile3 = CachedTile::new(
            TileCoord::new(10, 2, 0).unwrap(),
            vec![3; 300],
            "image/png".to_string(),
        );

        cache.put(tile1);
        cache.put(tile2);
        cache.put(tile3); // This should evict tile1

        let stats = cache.stats();
        assert_eq!(stats.item_count, 2);
        assert_eq!(stats.evictions, 1);
        assert_eq!(stats.size_bytes, 500); // 200 + 300
    }
}
