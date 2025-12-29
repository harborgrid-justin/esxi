//! MBTiles storage backend

use crate::error::{Error, Result};
use crate::storage::{TileStorage, StorageMetadata};
use crate::tile::coordinate::TileCoordinate;
use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// MBTiles storage implementation
///
/// MBTiles is a specification for storing tiled map data in SQLite databases.
/// https://github.com/mapbox/mbtiles-spec
pub struct MBTilesStorage {
    path: PathBuf,
    conn: Arc<Mutex<Connection>>,
}

impl MBTilesStorage {
    /// Create or open an MBTiles database
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let conn = Connection::open(&path)?;

        // Create tables if they don't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                name TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tiles (
                zoom_level INTEGER,
                tile_column INTEGER,
                tile_row INTEGER,
                tile_data BLOB,
                PRIMARY KEY (zoom_level, tile_column, tile_row)
            )",
            [],
        )?;

        // Create index for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS tiles_idx ON tiles (zoom_level, tile_column, tile_row)",
            [],
        )?;

        Ok(Self {
            path,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Set metadata value
    pub async fn set_metadata(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO metadata (name, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    /// Get metadata value
    pub async fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT value FROM metadata WHERE name = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?;
        Ok(result)
    }

    /// Initialize with standard metadata
    pub async fn initialize(&self, name: &str, format: &str) -> Result<()> {
        self.set_metadata("name", name).await?;
        self.set_metadata("type", "baselayer").await?;
        self.set_metadata("version", "1.0.0").await?;
        self.set_metadata("description", "").await?;
        self.set_metadata("format", format).await?;
        Ok(())
    }

    /// Convert ZXY to TMS coordinates (Y axis is flipped in MBTiles)
    fn zxy_to_tms(tile: TileCoordinate) -> (u8, u32, u32) {
        let tms = tile.to_tms();
        (tms.z, tms.x, tms.y)
    }
}

#[async_trait]
impl TileStorage for MBTilesStorage {
    async fn put_tile(&self, tile: TileCoordinate, data: Vec<u8>) -> Result<()> {
        let (z, x, y) = Self::zxy_to_tms(tile);
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO tiles (zoom_level, tile_column, tile_row, tile_data)
             VALUES (?1, ?2, ?3, ?4)",
            params![z, x, y, data],
        )?;

        Ok(())
    }

    async fn get_tile(&self, tile: TileCoordinate) -> Result<Option<Vec<u8>>> {
        let (z, x, y) = Self::zxy_to_tms(tile);
        let conn = self.conn.lock().unwrap();

        let result = conn
            .query_row(
                "SELECT tile_data FROM tiles WHERE zoom_level = ?1 AND tile_column = ?2 AND tile_row = ?3",
                params![z, x, y],
                |row| row.get(0),
            )
            .optional()?;

        Ok(result)
    }

    async fn has_tile(&self, tile: TileCoordinate) -> Result<bool> {
        let (z, x, y) = Self::zxy_to_tms(tile);
        let conn = self.conn.lock().unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tiles WHERE zoom_level = ?1 AND tile_column = ?2 AND tile_row = ?3",
            params![z, x, y],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    async fn delete_tile(&self, tile: TileCoordinate) -> Result<()> {
        let (z, x, y) = Self::zxy_to_tms(tile);
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "DELETE FROM tiles WHERE zoom_level = ?1 AND tile_column = ?2 AND tile_row = ?3",
            params![z, x, y],
        )?;

        Ok(())
    }

    async fn list_tiles(&self) -> Result<Vec<TileCoordinate>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT zoom_level, tile_column, tile_row FROM tiles")?;

        let tiles = stmt
            .query_map([], |row| {
                let z: u8 = row.get(0)?;
                let x: u32 = row.get(1)?;
                let y: u32 = row.get(2)?;
                Ok(TileCoordinate::new(z, x, y).to_tms()) // Convert back from TMS
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tiles)
    }

    async fn metadata(&self) -> Result<StorageMetadata> {
        let conn = self.conn.lock().unwrap();

        let tile_count: Option<u64> = conn
            .query_row("SELECT COUNT(*) FROM tiles", [], |row| row.get(0))
            .ok();

        let min_zoom: Option<u8> = conn
            .query_row("SELECT MIN(zoom_level) FROM tiles", [], |row| row.get(0))
            .ok();

        let max_zoom: Option<u8> = conn
            .query_row("SELECT MAX(zoom_level) FROM tiles", [], |row| row.get(0))
            .ok();

        Ok(StorageMetadata {
            tile_count,
            min_zoom,
            max_zoom,
            total_size: None,
            bounds: None,
        })
    }

    async fn flush(&self) -> Result<()> {
        // SQLite auto-flushes, but we can execute a checkpoint
        let conn = self.conn.lock().unwrap();
        conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_mbtiles_storage() {
        let temp = NamedTempFile::new().unwrap();
        let storage = MBTilesStorage::new(temp.path()).await.unwrap();

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

    #[tokio::test]
    async fn test_mbtiles_metadata() {
        let temp = NamedTempFile::new().unwrap();
        let storage = MBTilesStorage::new(temp.path()).await.unwrap();

        storage.set_metadata("test_key", "test_value").await.unwrap();
        let value = storage.get_metadata("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }
}
