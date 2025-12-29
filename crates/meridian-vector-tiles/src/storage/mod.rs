//! Tile storage backends

pub mod directory;
pub mod mbtiles;
pub mod pmtiles;
pub mod s3;

pub use directory::DirectoryStorage;
pub use mbtiles::MBTilesStorage;
pub use pmtiles::PMTilesStorage;
pub use s3::S3Storage;

use crate::error::Result;
use crate::tile::coordinate::TileCoordinate;
use async_trait::async_trait;

/// Trait for tile storage backends
#[async_trait]
pub trait TileStorage: Send + Sync {
    /// Store a tile
    async fn put_tile(&self, tile: TileCoordinate, data: Vec<u8>) -> Result<()>;

    /// Retrieve a tile
    async fn get_tile(&self, tile: TileCoordinate) -> Result<Option<Vec<u8>>>;

    /// Check if a tile exists
    async fn has_tile(&self, tile: TileCoordinate) -> Result<bool>;

    /// Delete a tile
    async fn delete_tile(&self, tile: TileCoordinate) -> Result<()>;

    /// List all tiles (for iteration)
    async fn list_tiles(&self) -> Result<Vec<TileCoordinate>> {
        Ok(Vec::new())
    }

    /// Get storage metadata
    async fn metadata(&self) -> Result<StorageMetadata> {
        Ok(StorageMetadata::default())
    }

    /// Flush any pending writes
    async fn flush(&self) -> Result<()> {
        Ok(())
    }

    /// Close the storage
    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

/// Storage metadata
#[derive(Debug, Clone, Default)]
pub struct StorageMetadata {
    /// Total number of tiles
    pub tile_count: Option<u64>,
    /// Total size in bytes
    pub total_size: Option<u64>,
    /// Minimum zoom level
    pub min_zoom: Option<u8>,
    /// Maximum zoom level
    pub max_zoom: Option<u8>,
    /// Bounds [west, south, east, north]
    pub bounds: Option<[f64; 4]>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_metadata() {
        let metadata = StorageMetadata::default();
        assert!(metadata.tile_count.is_none());
    }
}
