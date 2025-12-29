//! Tile generation modules

pub mod attribute;
pub mod clipper;
pub mod overzooming;
pub mod simplifier;

pub use attribute::AttributeFilter;
pub use clipper::GeometryClipper;
pub use overzooming::OverzoomHandler;
pub use simplifier::GeometrySimplifier;

use crate::encoding::mvt::{MvtFeature, MvtLayer, MvtTile, MvtValue};
use crate::error::Result;
use crate::source::TileSource;
use crate::tile::bounds::MercatorBounds;
use crate::tile::coordinate::TileCoordinate;
use crate::tile::extent::{ExtentConverter, TileExtent};
use std::collections::HashMap;
use std::sync::Arc;

/// Tile generation configuration
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    /// Tile extent (default: 4096)
    pub extent: u32,
    /// Buffer size around tile (in pixels)
    pub buffer: u32,
    /// Enable geometry simplification
    pub simplify: bool,
    /// Simplification tolerance (in tile coordinates)
    pub simplify_tolerance: f64,
    /// Enable geometry clipping
    pub clip: bool,
    /// Maximum features per tile
    pub max_features: Option<usize>,
    /// Maximum tile size in bytes
    pub max_tile_size: Option<usize>,
    /// Enable overzoom
    pub enable_overzoom: bool,
    /// Maximum overzoom level
    pub max_overzoom: u8,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            extent: crate::DEFAULT_EXTENT,
            buffer: 64,
            simplify: true,
            simplify_tolerance: 1.0,
            clip: true,
            max_features: Some(10000),
            max_tile_size: Some(500_000), // 500KB
            enable_overzoom: true,
            max_overzoom: 5,
        }
    }
}

/// Tile generator
pub struct TileGenerator {
    config: GenerationConfig,
    simplifier: GeometrySimplifier,
    clipper: GeometryClipper,
    overzoom: OverzoomHandler,
}

impl TileGenerator {
    /// Create a new tile generator with default configuration
    pub fn new() -> Self {
        Self::with_config(GenerationConfig::default())
    }

    /// Create a tile generator with custom configuration
    pub fn with_config(config: GenerationConfig) -> Self {
        Self {
            simplifier: GeometrySimplifier::new(config.simplify_tolerance),
            clipper: GeometryClipper::new(config.buffer),
            overzoom: OverzoomHandler::new(config.max_overzoom),
            config,
        }
    }

    /// Generate a tile from a source
    pub async fn generate<S: TileSource>(
        &self,
        source: &S,
        tile: TileCoordinate,
    ) -> Result<Option<MvtTile>> {
        // Get tile bounds
        let bounds = MercatorBounds::from_tile(&tile);
        let extent = TileExtent::new(self.config.extent);

        // Create extent converter
        let converter = ExtentConverter::new(
            extent,
            bounds.min_x,
            bounds.min_y,
            bounds.max_x,
            bounds.max_y,
        );

        // Fetch features from source
        let features = source.get_features(tile, &bounds).await?;

        if features.is_empty() {
            return Ok(None);
        }

        // Group features by layer
        let mut layers: HashMap<String, Vec<MvtFeature>> = HashMap::new();

        for feature in features {
            let mut geometry = feature.geometry;

            // Simplify geometry if enabled
            if self.config.simplify {
                geometry = self.simplifier.simplify(&geometry, tile.z);
            }

            // Clip geometry if enabled
            if self.config.clip {
                if let Some(clipped) = self.clipper.clip(&geometry, &bounds, &converter) {
                    geometry = clipped;
                } else {
                    continue; // Skip features completely outside tile
                }
            }

            // Create MVT feature
            let mvt_feature = MvtFeature {
                id: feature.id,
                geometry,
                properties: feature.properties,
            };

            layers
                .entry(feature.layer)
                .or_insert_with(Vec::new)
                .push(mvt_feature);
        }

        // Create MVT layers
        let mvt_layers: Vec<MvtLayer> = layers
            .into_iter()
            .map(|(name, features)| {
                // Apply max features limit
                let features = if let Some(max) = self.config.max_features {
                    features.into_iter().take(max).collect()
                } else {
                    features
                };

                MvtLayer {
                    name,
                    extent: self.config.extent,
                    version: 2,
                    features,
                }
            })
            .collect();

        Ok(Some(MvtTile {
            layers: mvt_layers,
        }))
    }

    /// Get the configuration
    pub fn config(&self) -> &GenerationConfig {
        &self.config
    }
}

impl Default for TileGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Feature data from a source
#[derive(Debug, Clone)]
pub struct SourceFeature {
    /// Feature ID
    pub id: Option<u64>,
    /// Layer name
    pub layer: String,
    /// Geometry
    pub geometry: geo_types::Geometry,
    /// Properties
    pub properties: HashMap<String, MvtValue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_config() {
        let config = GenerationConfig::default();
        assert_eq!(config.extent, 4096);
        assert_eq!(config.buffer, 64);
        assert!(config.simplify);
        assert!(config.clip);
    }

    #[test]
    fn test_tile_generator() {
        let generator = TileGenerator::new();
        assert_eq!(generator.config().extent, 4096);
    }
}
