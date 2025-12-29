//! S3 storage backend

use crate::error::{Error, Result};
use crate::storage::{TileStorage, StorageMetadata};
use crate::tile::coordinate::TileCoordinate;
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_s3::{PutObjectRequest, GetObjectRequest, DeleteObjectRequest, S3Client, S3};
use std::str::FromStr;
use tokio::io::AsyncReadExt;

/// S3 storage configuration
#[derive(Debug, Clone)]
pub struct S3Config {
    /// S3 bucket name
    pub bucket: String,
    /// Key prefix (e.g., "tiles/")
    pub prefix: String,
    /// AWS region
    pub region: Region,
    /// Content type for tiles
    pub content_type: String,
    /// Cache control header
    pub cache_control: Option<String>,
}

impl Default for S3Config {
    fn default() -> Self {
        Self {
            bucket: String::new(),
            prefix: String::new(),
            region: Region::UsEast1,
            content_type: "application/vnd.mapbox-vector-tile".to_string(),
            cache_control: Some("public, max-age=86400".to_string()),
        }
    }
}

/// S3 tile storage
pub struct S3Storage {
    client: S3Client,
    config: S3Config,
}

impl S3Storage {
    /// Create a new S3 storage
    pub fn new(config: S3Config) -> Self {
        let client = S3Client::new(config.region.clone());
        Self { client, config }
    }

    /// Create with custom region
    pub fn with_region(mut self, region: &str) -> Result<Self> {
        self.config.region = Region::from_str(region)
            .map_err(|e| Error::s3(format!("Invalid region: {}", e)))?;
        self.client = S3Client::new(self.config.region.clone());
        Ok(self)
    }

    /// Get S3 key for a tile
    fn tile_key(&self, tile: TileCoordinate) -> String {
        format!(
            "{}{}/{}/{}.mvt",
            self.config.prefix, tile.z, tile.x, tile.y
        )
    }
}

#[async_trait]
impl TileStorage for S3Storage {
    async fn put_tile(&self, tile: TileCoordinate, data: Vec<u8>) -> Result<()> {
        let key = self.tile_key(tile);

        let mut request = PutObjectRequest {
            bucket: self.config.bucket.clone(),
            key,
            body: Some(data.into()),
            content_type: Some(self.config.content_type.clone()),
            ..Default::default()
        };

        if let Some(ref cache_control) = self.config.cache_control {
            request.cache_control = Some(cache_control.clone());
        }

        self.client
            .put_object(request)
            .await
            .map_err(|e| Error::s3(format!("Failed to put object: {}", e)))?;

        Ok(())
    }

    async fn get_tile(&self, tile: TileCoordinate) -> Result<Option<Vec<u8>>> {
        let key = self.tile_key(tile);

        let request = GetObjectRequest {
            bucket: self.config.bucket.clone(),
            key,
            ..Default::default()
        };

        match self.client.get_object(request).await {
            Ok(output) => {
                if let Some(body) = output.body {
                    let mut data = Vec::new();
                    body.into_async_read().read_to_end(&mut data).await?;
                    Ok(Some(data))
                } else {
                    Ok(None)
                }
            }
            Err(rusoto_core::RusotoError::Service(rusoto_s3::GetObjectError::NoSuchKey(_))) => {
                Ok(None)
            }
            Err(e) => Err(Error::s3(format!("Failed to get object: {}", e))),
        }
    }

    async fn has_tile(&self, tile: TileCoordinate) -> Result<bool> {
        // Use HEAD request to check existence
        self.get_tile(tile).await.map(|opt| opt.is_some())
    }

    async fn delete_tile(&self, tile: TileCoordinate) -> Result<()> {
        let key = self.tile_key(tile);

        let request = DeleteObjectRequest {
            bucket: self.config.bucket.clone(),
            key,
            ..Default::default()
        };

        self.client
            .delete_object(request)
            .await
            .map_err(|e| Error::s3(format!("Failed to delete object: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_config() {
        let config = S3Config::default();
        assert_eq!(config.content_type, "application/vnd.mapbox-vector-tile");
    }

    #[test]
    fn test_tile_key() {
        let config = S3Config {
            prefix: "tiles/".to_string(),
            ..Default::default()
        };
        let storage = S3Storage::new(config);

        let tile = TileCoordinate::new(10, 512, 384);
        let key = storage.tile_key(tile);

        assert_eq!(key, "tiles/10/512/384.mvt");
    }
}
