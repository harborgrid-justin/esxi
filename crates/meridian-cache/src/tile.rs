//! GIS tile caching optimization

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::sync::Arc;
use std::time::Duration;

use crate::backend::{CacheBackend, CacheOptions};
use crate::compression::{AdaptiveCompressor, CompressionAlgorithm};
use crate::error::{CacheError, CacheResult};

/// Tile coordinate in a tiled map system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoord {
    /// Zoom level
    pub z: u8,
    /// X coordinate
    pub x: u32,
    /// Y coordinate
    pub y: u32,
}

impl TileCoord {
    /// Create a new tile coordinate
    pub fn new(z: u8, x: u32, y: u32) -> Self {
        Self { z, x, y }
    }

    /// Generate a cache key for this tile
    pub fn cache_key(&self, layer: &str) -> String {
        format!("tile:{}:{}:{}:{}", layer, self.z, self.x, self.y)
    }

    /// Get the parent tile (one zoom level up)
    pub fn parent(&self) -> Option<Self> {
        if self.z == 0 {
            None
        } else {
            Some(Self {
                z: self.z - 1,
                x: self.x / 2,
                y: self.y / 2,
            })
        }
    }

    /// Get the four child tiles (one zoom level down)
    pub fn children(&self) -> [Self; 4] {
        let base_x = self.x * 2;
        let base_y = self.y * 2;
        let child_z = self.z + 1;

        [
            Self::new(child_z, base_x, base_y),
            Self::new(child_z, base_x + 1, base_y),
            Self::new(child_z, base_x, base_y + 1),
            Self::new(child_z, base_x + 1, base_y + 1),
        ]
    }

    /// Get neighboring tiles
    pub fn neighbors(&self) -> [Self; 8] {
        [
            Self::new(self.z, self.x - 1, self.y - 1), // NW
            Self::new(self.z, self.x, self.y - 1),     // N
            Self::new(self.z, self.x + 1, self.y - 1), // NE
            Self::new(self.z, self.x - 1, self.y),     // W
            Self::new(self.z, self.x + 1, self.y),     // E
            Self::new(self.z, self.x - 1, self.y + 1), // SW
            Self::new(self.z, self.x, self.y + 1),     // S
            Self::new(self.z, self.x + 1, self.y + 1), // SE
        ]
    }

    /// Get the bounding box for a tile pyramid
    pub fn pyramid_bounds(z: u8) -> (u32, u32) {
        let max = 2u32.pow(z as u32);
        (max, max)
    }

    /// Check if tile coordinates are valid for the zoom level
    pub fn is_valid(&self) -> bool {
        let (max_x, max_y) = Self::pyramid_bounds(self.z);
        self.x < max_x && self.y < max_y
    }
}

/// Tile format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileFormat {
    /// PNG image
    PNG,
    /// JPEG image
    JPEG,
    /// WebP image
    WebP,
    /// Protocol Buffers (vector tiles)
    PBF,
    /// GeoJSON
    GeoJSON,
    /// TopoJSON
    TopoJSON,
}

impl TileFormat {
    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            TileFormat::PNG => "image/png",
            TileFormat::JPEG => "image/jpeg",
            TileFormat::WebP => "image/webp",
            TileFormat::PBF => "application/x-protobuf",
            TileFormat::GeoJSON => "application/geo+json",
            TileFormat::TopoJSON => "application/json",
        }
    }

    /// Check if this format should be compressed
    pub fn should_compress(&self) -> bool {
        match self {
            TileFormat::PNG | TileFormat::JPEG | TileFormat::WebP => false, // Already compressed
            TileFormat::PBF | TileFormat::GeoJSON | TileFormat::TopoJSON => true,
        }
    }
}

/// Tile metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileMetadata {
    /// Tile format
    pub format: String,
    /// Layer name
    pub layer: String,
    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// ETag for cache validation
    pub etag: Option<String>,
    /// Whether the tile is compressed
    pub compressed: bool,
}

/// Tile cache configuration
#[derive(Debug, Clone)]
pub struct TileCacheConfig {
    /// Default TTL for tiles
    pub default_ttl: Duration,
    /// Enable compression for vector tiles
    pub compress_vector_tiles: bool,
    /// Enable pre-fetching of neighboring tiles
    pub prefetch_neighbors: bool,
    /// Enable pre-fetching of child tiles
    pub prefetch_children: bool,
    /// Maximum zoom level to cache
    pub max_zoom: u8,
    /// Minimum zoom level to cache
    pub min_zoom: u8,
}

impl Default for TileCacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600 * 24 * 7), // 1 week
            compress_vector_tiles: true,
            prefetch_neighbors: true,
            prefetch_children: false,
            max_zoom: 18,
            min_zoom: 0,
        }
    }
}

/// Optimized tile cache for GIS data
pub struct TileCache<B: CacheBackend> {
    backend: Arc<B>,
    config: TileCacheConfig,
    compressor: AdaptiveCompressor,
}

impl<B: CacheBackend + 'static> TileCache<B> {
    /// Create a new tile cache
    pub fn new(backend: Arc<B>, config: TileCacheConfig) -> Self {
        Self {
            backend,
            config,
            compressor: AdaptiveCompressor::default(),
        }
    }

    /// Get a tile from the cache
    pub async fn get_tile(
        &self,
        layer: &str,
        coord: TileCoord,
    ) -> CacheResult<Option<Bytes>> {
        if !coord.is_valid() {
            return Err(CacheError::InvalidKey(format!(
                "Invalid tile coordinates: {:?}",
                coord
            )));
        }

        if coord.z > self.config.max_zoom || coord.z < self.config.min_zoom {
            return Err(CacheError::InvalidKey(format!(
                "Zoom level {} out of range [{}, {}]",
                coord.z, self.config.min_zoom, self.config.max_zoom
            )));
        }

        let key = coord.cache_key(layer);

        if let Some(entry) = self.backend.get(&key).await? {
            // Decompress if needed
            let metadata: TileMetadata = serde_json::from_slice(
                entry.metadata.tags.first()
                    .and_then(|tag| tag.strip_prefix("metadata:"))
                    .map(|s| s.as_bytes())
                    .unwrap_or(b"{}"),
            )
            .unwrap_or_else(|_| TileMetadata {
                format: "unknown".to_string(),
                layer: layer.to_string(),
                generated_at: chrono::Utc::now(),
                etag: None,
                compressed: false,
            });

            if metadata.compressed {
                let compressed_data = crate::compression::CompressedData {
                    data: entry.value,
                    algorithm: CompressionAlgorithm::LZ4,
                    original_size: 0, // Not stored
                };
                let decompressed = self.compressor.decompress(&compressed_data)?;
                Ok(Some(decompressed))
            } else {
                Ok(Some(entry.value))
            }
        } else {
            Ok(None)
        }
    }

    /// Set a tile in the cache
    pub async fn set_tile(
        &self,
        layer: &str,
        coord: TileCoord,
        data: Bytes,
        format: TileFormat,
    ) -> CacheResult<()> {
        if !coord.is_valid() {
            return Err(CacheError::InvalidKey(format!(
                "Invalid tile coordinates: {:?}",
                coord
            )));
        }

        let key = coord.cache_key(layer);

        // Compress if beneficial
        let (data, compressed) = if self.config.compress_vector_tiles && format.should_compress() {
            let compressed = self.compressor.compress(&data)?;
            if compressed.data.len() < data.len() {
                (compressed.data, true)
            } else {
                (data, false)
            }
        } else {
            (data, false)
        };

        // Generate ETag
        let etag = {
            let mut hasher = sha2::Sha256::new();
            hasher.update(&data);
            hex::encode(hasher.finalize())
        };

        let metadata = TileMetadata {
            format: format.mime_type().to_string(),
            layer: layer.to_string(),
            generated_at: chrono::Utc::now(),
            etag: Some(etag),
            compressed,
        };

        let metadata_json = serde_json::to_string(&metadata)
            .map_err(|e| CacheError::Backend(format!("Failed to serialize metadata: {}", e)))?;

        let options = CacheOptions {
            ttl: Some(self.config.default_ttl),
            tags: vec![format!("metadata:{}", metadata_json)],
            compress: false, // We handle compression manually
        };

        self.backend.set(&key, data, options).await?;

        // Pre-fetch neighbors if enabled
        if self.config.prefetch_neighbors {
            // This would be implemented by the caller
        }

        Ok(())
    }

    /// Delete a tile from the cache
    pub async fn delete_tile(&self, layer: &str, coord: TileCoord) -> CacheResult<bool> {
        let key = coord.cache_key(layer);
        self.backend.delete(&key).await
    }

    /// Get multiple tiles at once
    pub async fn get_tiles(
        &self,
        layer: &str,
        coords: &[TileCoord],
    ) -> CacheResult<Vec<Option<Bytes>>> {
        let keys: Vec<String> = coords
            .iter()
            .map(|coord| coord.cache_key(layer))
            .collect();

        let entries = self.backend.mget(&keys).await?;

        let mut results = Vec::with_capacity(entries.len());
        for entry in entries {
            if let Some(entry) = entry {
                results.push(Some(entry.value));
            } else {
                results.push(None);
            }
        }

        Ok(results)
    }

    /// Invalidate all tiles for a layer
    pub async fn invalidate_layer(&self, layer: &str) -> CacheResult<usize> {
        let pattern = format!("tile:{}:*", layer);
        self.backend.delete_pattern(&pattern).await
    }

    /// Invalidate tiles within a zoom range
    pub async fn invalidate_zoom_range(
        &self,
        layer: &str,
        min_zoom: u8,
        max_zoom: u8,
    ) -> CacheResult<usize> {
        let mut total = 0;

        for z in min_zoom..=max_zoom {
            let pattern = format!("tile:{}:{}:*", layer, z);
            total += self.backend.delete_pattern(&pattern).await?;
        }

        Ok(total)
    }

    /// Pre-warm tiles for a bounding box at specific zoom levels
    pub async fn prewarm_bbox<F>(
        &self,
        layer: &str,
        min_coord: TileCoord,
        max_coord: TileCoord,
        tile_generator: F,
    ) -> CacheResult<usize>
    where
        F: Fn(TileCoord) -> futures::future::BoxFuture<'static, CacheResult<(Bytes, TileFormat)>>,
    {
        if min_coord.z != max_coord.z {
            return Err(CacheError::InvalidKey(
                "Min and max coordinates must have the same zoom level".to_string(),
            ));
        }

        let mut count = 0;
        let z = min_coord.z;

        for x in min_coord.x..=max_coord.x {
            for y in min_coord.y..=max_coord.y {
                let coord = TileCoord::new(z, x, y);

                // Check if tile already exists
                if self.get_tile(layer, coord).await?.is_some() {
                    continue;
                }

                // Generate and cache tile
                match tile_generator(coord).await {
                    Ok((data, format)) => {
                        self.set_tile(layer, coord, data, format).await?;
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to generate tile {:?}: {}", coord, e);
                    }
                }
            }
        }

        Ok(count)
    }

    /// Get cache statistics for a specific layer
    pub async fn get_layer_stats(&self, layer: &str) -> CacheResult<LayerStats> {
        let pattern = format!("tile:{}:*", layer);
        let keys = self.backend.keys(&pattern).await?;

        let mut stats = LayerStats {
            layer: layer.to_string(),
            total_tiles: keys.len(),
            zoom_distribution: std::collections::HashMap::new(),
            total_size: 0,
        };

        for key in keys {
            // Parse zoom level from key
            if let Some(parts) = key.split(':').nth(2) {
                if let Ok(zoom) = parts.parse::<u8>() {
                    *stats.zoom_distribution.entry(zoom).or_insert(0) += 1;
                }
            }
        }

        Ok(stats)
    }
}

/// Statistics for a tile layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStats {
    /// Layer name
    pub layer: String,
    /// Total number of cached tiles
    pub total_tiles: usize,
    /// Distribution of tiles by zoom level
    pub zoom_distribution: std::collections::HashMap<u8, usize>,
    /// Total cache size in bytes
    pub total_size: usize,
}

/// Tile pyramid generator for pre-caching
pub struct TilePyramidGenerator {
    /// Base zoom level
    pub base_zoom: u8,
    /// Maximum zoom level
    pub max_zoom: u8,
    /// Tile size in pixels
    pub tile_size: u32,
}

impl TilePyramidGenerator {
    /// Create a new pyramid generator
    pub fn new(base_zoom: u8, max_zoom: u8, tile_size: u32) -> Self {
        Self {
            base_zoom,
            max_zoom,
            tile_size,
        }
    }

    /// Calculate the number of tiles at a zoom level
    pub fn tiles_at_zoom(&self, zoom: u8) -> u64 {
        let count = 2u64.pow(zoom as u32);
        count * count
    }

    /// Calculate total tiles in the pyramid
    pub fn total_tiles(&self) -> u64 {
        (self.base_zoom..=self.max_zoom)
            .map(|z| self.tiles_at_zoom(z))
            .sum()
    }

    /// Estimate storage size in bytes
    pub fn estimate_storage(&self, avg_tile_size: usize) -> usize {
        self.total_tiles() as usize * avg_tile_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_coord() {
        let coord = TileCoord::new(10, 512, 256);
        assert_eq!(coord.z, 10);
        assert_eq!(coord.x, 512);
        assert_eq!(coord.y, 256);
        assert!(coord.is_valid());

        let key = coord.cache_key("osm");
        assert_eq!(key, "tile:osm:10:512:256");
    }

    #[test]
    fn test_tile_parent() {
        let coord = TileCoord::new(10, 512, 256);
        let parent = coord.parent().unwrap();

        assert_eq!(parent.z, 9);
        assert_eq!(parent.x, 256);
        assert_eq!(parent.y, 128);
    }

    #[test]
    fn test_tile_children() {
        let coord = TileCoord::new(10, 512, 256);
        let children = coord.children();

        assert_eq!(children.len(), 4);
        assert_eq!(children[0], TileCoord::new(11, 1024, 512));
        assert_eq!(children[1], TileCoord::new(11, 1025, 512));
        assert_eq!(children[2], TileCoord::new(11, 1024, 513));
        assert_eq!(children[3], TileCoord::new(11, 1025, 513));
    }

    #[test]
    fn test_tile_pyramid() {
        let pyramid = TilePyramidGenerator::new(0, 5, 256);

        assert_eq!(pyramid.tiles_at_zoom(0), 1);
        assert_eq!(pyramid.tiles_at_zoom(1), 4);
        assert_eq!(pyramid.tiles_at_zoom(2), 16);

        // Total: 1 + 4 + 16 + 64 + 256 + 1024 = 1365
        assert_eq!(pyramid.total_tiles(), 1365);
    }

    #[tokio::test]
    async fn test_tile_cache() -> CacheResult<()> {
        use crate::backend::memory::MemoryCache;

        let backend = Arc::new(MemoryCache::lru(1000));
        let cache = TileCache::new(backend, TileCacheConfig::default());

        let coord = TileCoord::new(10, 512, 256);
        let data = Bytes::from("tile data");

        cache
            .set_tile("osm", coord, data.clone(), TileFormat::PNG)
            .await?;

        let result = cache.get_tile("osm", coord).await?;
        assert!(result.is_some());

        Ok(())
    }
}
