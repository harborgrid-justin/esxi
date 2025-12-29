//! Directory-based tile storage

use crate::error::{Error, Result};
use crate::storage::{TileStorage, StorageMetadata};
use crate::tile::coordinate::TileCoordinate;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Directory storage layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryLayout {
    /// ZXY layout: z/x/y.ext
    ZXY,
    /// TMS layout: z/x/y.ext (with Y flipped)
    TMS,
    /// Quadkey layout: q/u/a/d/key.ext
    Quadkey,
}

/// Directory-based tile storage
pub struct DirectoryStorage {
    root: PathBuf,
    layout: DirectoryLayout,
    extension: String,
}

impl DirectoryStorage {
    /// Create a new directory storage
    pub async fn new<P: AsRef<Path>>(root: P) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root).await?;

        Ok(Self {
            root,
            layout: DirectoryLayout::ZXY,
            extension: "mvt".to_string(),
        })
    }

    /// Create with custom layout
    pub async fn with_layout<P: AsRef<Path>>(
        root: P,
        layout: DirectoryLayout,
    ) -> Result<Self> {
        let mut storage = Self::new(root).await?;
        storage.layout = layout;
        Ok(storage)
    }

    /// Set file extension
    pub fn with_extension(mut self, ext: &str) -> Self {
        self.extension = ext.to_string();
        self
    }

    /// Get the file path for a tile
    fn tile_path(&self, tile: TileCoordinate) -> PathBuf {
        match self.layout {
            DirectoryLayout::ZXY => {
                let mut path = self.root.clone();
                path.push(tile.z.to_string());
                path.push(tile.x.to_string());
                path.push(format!("{}.{}", tile.y, self.extension));
                path
            }
            DirectoryLayout::TMS => {
                let tms = tile.to_tms();
                let mut path = self.root.clone();
                path.push(tms.z.to_string());
                path.push(tms.x.to_string());
                path.push(format!("{}.{}", tms.y, self.extension));
                path
            }
            DirectoryLayout::Quadkey => {
                let quadkey = tile.to_quadkey();
                let mut path = self.root.clone();

                // Split quadkey into directory levels for better distribution
                // e.g., "0123" -> 0/1/2/3.ext
                for c in quadkey.chars() {
                    path.push(c.to_string());
                }
                path.set_extension(&self.extension);
                path
            }
        }
    }

    /// Ensure parent directory exists
    async fn ensure_parent_dir(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl TileStorage for DirectoryStorage {
    async fn put_tile(&self, tile: TileCoordinate, data: Vec<u8>) -> Result<()> {
        let path = self.tile_path(tile);
        self.ensure_parent_dir(&path).await?;
        fs::write(&path, data).await?;
        Ok(())
    }

    async fn get_tile(&self, tile: TileCoordinate) -> Result<Option<Vec<u8>>> {
        let path = self.tile_path(tile);

        match fs::read(&path).await {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn has_tile(&self, tile: TileCoordinate) -> Result<bool> {
        let path = self.tile_path(tile);
        Ok(path.exists())
    }

    async fn delete_tile(&self, tile: TileCoordinate) -> Result<()> {
        let path = self.tile_path(tile);

        match fs::remove_file(&path).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn list_tiles(&self) -> Result<Vec<TileCoordinate>> {
        // Recursively scan directory for tiles
        // This is a simplified implementation
        let mut tiles = Vec::new();

        // Walk the directory tree
        // This would need a proper recursive implementation
        // For now, return empty

        Ok(tiles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_directory_storage() {
        let temp = tempdir().unwrap();
        let storage = DirectoryStorage::new(temp.path()).await.unwrap();

        let tile = TileCoordinate::new(10, 512, 384);
        let data = vec![1, 2, 3, 4, 5];

        // Put tile
        storage.put_tile(tile, data.clone()).await.unwrap();

        // Get tile
        let retrieved = storage.get_tile(tile).await.unwrap();
        assert_eq!(retrieved, Some(data));

        // Has tile
        assert!(storage.has_tile(tile).await.unwrap());

        // Delete tile
        storage.delete_tile(tile).await.unwrap();
        assert!(!storage.has_tile(tile).await.unwrap());
    }

    #[test]
    fn test_tile_path_zxy() {
        let temp = PathBuf::from("/tmp/tiles");
        let storage = DirectoryStorage {
            root: temp.clone(),
            layout: DirectoryLayout::ZXY,
            extension: "mvt".to_string(),
        };

        let tile = TileCoordinate::new(10, 512, 384);
        let path = storage.tile_path(tile);

        assert_eq!(path, temp.join("10/512/384.mvt"));
    }

    #[test]
    fn test_tile_path_quadkey() {
        let temp = PathBuf::from("/tmp/tiles");
        let storage = DirectoryStorage {
            root: temp.clone(),
            layout: DirectoryLayout::Quadkey,
            extension: "mvt".to_string(),
        };

        let tile = TileCoordinate::new(3, 3, 5);
        let path = storage.tile_path(tile);

        // Quadkey for this tile should create nested directories
        assert!(path.starts_with(&temp));
    }
}
