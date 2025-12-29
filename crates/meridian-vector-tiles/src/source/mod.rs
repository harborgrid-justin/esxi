//! Data sources for tile generation

pub mod file;
pub mod pmtiles;
pub mod postgis;

pub use file::FileSource;
pub use pmtiles::PMTilesSource;
pub use postgis::PostGISSource;

use crate::error::Result;
use crate::generation::SourceFeature;
use crate::tile::bounds::MercatorBounds;
use crate::tile::coordinate::TileCoordinate;
use async_trait::async_trait;

/// Trait for tile data sources
#[async_trait]
pub trait TileSource: Send + Sync {
    /// Get features for a tile
    async fn get_features(
        &self,
        tile: TileCoordinate,
        bounds: &MercatorBounds,
    ) -> Result<Vec<SourceFeature>>;

    /// Get the maximum zoom level for this source
    fn max_zoom(&self) -> u8 {
        crate::MAX_ZOOM_LEVEL
    }

    /// Get the minimum zoom level for this source
    fn min_zoom(&self) -> u8 {
        crate::MIN_ZOOM_LEVEL
    }

    /// Get layer names provided by this source
    async fn layers(&self) -> Result<Vec<String>>;

    /// Get metadata about the source
    async fn metadata(&self) -> Result<SourceMetadata> {
        Ok(SourceMetadata::default())
    }
}

/// Source metadata
#[derive(Debug, Clone)]
pub struct SourceMetadata {
    /// Source name
    pub name: String,
    /// Source description
    pub description: Option<String>,
    /// Attribution text
    pub attribution: Option<String>,
    /// Bounds in lon/lat [west, south, east, north]
    pub bounds: Option<[f64; 4]>,
    /// Center point and zoom [lon, lat, zoom]
    pub center: Option<[f64; 3]>,
    /// Minimum zoom
    pub min_zoom: u8,
    /// Maximum zoom
    pub max_zoom: u8,
    /// Layer metadata
    pub layers: Vec<LayerMetadata>,
}

impl Default for SourceMetadata {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            attribution: None,
            bounds: None,
            center: None,
            min_zoom: crate::MIN_ZOOM_LEVEL,
            max_zoom: crate::MAX_ZOOM_LEVEL,
            layers: Vec::new(),
        }
    }
}

/// Layer metadata
#[derive(Debug, Clone)]
pub struct LayerMetadata {
    /// Layer name
    pub name: String,
    /// Layer description
    pub description: Option<String>,
    /// Minimum zoom
    pub min_zoom: u8,
    /// Maximum zoom
    pub max_zoom: u8,
    /// Geometry type
    pub geometry_type: Option<String>,
    /// Field definitions
    pub fields: Vec<FieldMetadata>,
}

/// Field metadata
#[derive(Debug, Clone)]
pub struct FieldMetadata {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: FieldType,
    /// Field description
    pub description: Option<String>,
}

/// Field type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Unknown,
}

impl FieldType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FieldType::String => "string",
            FieldType::Integer => "integer",
            FieldType::Float => "float",
            FieldType::Boolean => "boolean",
            FieldType::Unknown => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_metadata() {
        let metadata = SourceMetadata::default();
        assert_eq!(metadata.min_zoom, 0);
        assert_eq!(metadata.max_zoom, 24);
    }

    #[test]
    fn test_field_type() {
        let ft = FieldType::String;
        assert_eq!(ft.as_str(), "string");
    }
}
