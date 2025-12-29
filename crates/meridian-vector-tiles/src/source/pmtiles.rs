//! PMTiles reader source

use crate::error::{Error, Result};
use crate::generation::SourceFeature;
use crate::source::{TileSource, SourceMetadata};
use crate::tile::bounds::MercatorBounds;
use crate::tile::coordinate::TileCoordinate;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// PMTiles source for reading existing PMTiles archives
pub struct PMTilesSource {
    path: PathBuf,
}

impl PMTilesSource {
    /// Create a new PMTiles source
    pub async fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(Error::source(format!(
                "PMTiles file not found: {}",
                path.display()
            )));
        }

        Ok(Self { path })
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[async_trait]
impl TileSource for PMTilesSource {
    async fn get_features(
        &self,
        tile: TileCoordinate,
        _bounds: &MercatorBounds,
    ) -> Result<Vec<SourceFeature>> {
        // PMTiles sources return pre-generated tiles, not raw features
        // This would need to decode the MVT tile and extract features
        // For now, return empty - this is typically used for serving, not generation
        Ok(Vec::new())
    }

    async fn layers(&self) -> Result<Vec<String>> {
        // Would need to read PMTiles metadata
        Ok(Vec::new())
    }

    async fn metadata(&self) -> Result<SourceMetadata> {
        // Would need to read PMTiles header and metadata
        Ok(SourceMetadata {
            name: self
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("pmtiles")
                .to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pmtiles_path() {
        // This test would require a real PMTiles file
    }
}
