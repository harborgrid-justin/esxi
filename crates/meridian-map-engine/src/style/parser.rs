//! Style specification parser for loading and validating styles.

use super::{StyleSpec, SourceSpec, LayerStyleSpec};
use crate::error::{MapEngineError, Result};
use std::path::Path;

/// Style parser for loading style specifications.
pub struct StyleParser;

impl StyleParser {
    /// Parse a style from JSON string.
    pub fn from_json(json: &str) -> Result<StyleSpec> {
        serde_json::from_str(json).map_err(|e| {
            MapEngineError::StyleParse {
                location: "root".to_string(),
                message: e.to_string(),
            }
        })
    }

    /// Load a style from a JSON file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<StyleSpec> {
        let json = std::fs::read_to_string(path).map_err(|e| {
            MapEngineError::StyleParse {
                location: "file".to_string(),
                message: e.to_string(),
            }
        })?;

        Self::from_json(&json)
    }

    /// Validate a style specification.
    pub fn validate(style: &StyleSpec) -> Result<()> {
        // Check version
        if style.version != 8 {
            return Err(MapEngineError::StyleParse {
                location: "version".to_string(),
                message: format!("Unsupported style version: {}", style.version),
            });
        }

        // Validate sources
        for source in &style.sources {
            Self::validate_source(source)?;
        }

        // Validate layers
        for layer in &style.layers {
            Self::validate_layer(layer, style)?;
        }

        Ok(())
    }

    /// Validate a source specification.
    fn validate_source(source: &SourceSpec) -> Result<()> {
        // Check that source has either URL or tiles
        if source.url.is_none() && source.tiles.is_none() {
            return Err(MapEngineError::StyleParse {
                location: format!("source.{}", source.id),
                message: "Source must have either 'url' or 'tiles'".to_string(),
            });
        }

        Ok(())
    }

    /// Validate a layer specification.
    fn validate_layer(layer: &LayerStyleSpec, style: &StyleSpec) -> Result<()> {
        // Check that source exists
        if !style.sources.iter().any(|s| s.id == layer.source) {
            return Err(MapEngineError::StyleParse {
                location: format!("layer.{}", layer.id),
                message: format!("Source '{}' not found", layer.source),
            });
        }

        Ok(())
    }

    /// Convert style to JSON string.
    pub fn to_json(style: &StyleSpec) -> Result<String> {
        serde_json::to_string_pretty(style).map_err(|e| {
            MapEngineError::StyleParse {
                location: "serialization".to_string(),
                message: e.to_string(),
            }
        })
    }

    /// Save style to a JSON file.
    pub fn to_file(style: &StyleSpec, path: impl AsRef<Path>) -> Result<()> {
        let json = Self::to_json(style)?;
        std::fs::write(path, json).map_err(|e| {
            MapEngineError::StyleParse {
                location: "file".to_string(),
                message: e.to_string(),
            }
        })
    }
}

/// Style builder for programmatically creating styles.
pub struct StyleBuilder {
    style: StyleSpec,
}

impl StyleBuilder {
    /// Create a new style builder.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            style: StyleSpec::new(name),
        }
    }

    /// Add a source to the style.
    pub fn add_source(mut self, source: SourceSpec) -> Self {
        self.style.add_source(source);
        self
    }

    /// Add a layer to the style.
    pub fn add_layer(mut self, layer: LayerStyleSpec) -> Self {
        self.style.add_layer(layer);
        self
    }

    /// Build the final style specification.
    pub fn build(self) -> StyleSpec {
        self.style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::{SourceType, StyleLayerType};

    #[test]
    fn test_style_builder() {
        let style = StyleBuilder::new("test_style")
            .add_source(SourceSpec {
                id: "osm".to_string(),
                source_type: SourceType::Raster,
                url: Some("https://tile.openstreetmap.org/{z}/{x}/{y}.png".to_string()),
                tiles: None,
                minzoom: Some(0),
                maxzoom: Some(18),
            })
            .build();

        assert_eq!(style.name, "test_style");
        assert_eq!(style.sources.len(), 1);
    }

    #[test]
    fn test_json_serialization() {
        let style = StyleSpec::new("test");
        let json = StyleParser::to_json(&style).unwrap();
        assert!(json.contains("test"));

        let parsed = StyleParser::from_json(&json).unwrap();
        assert_eq!(parsed.name, "test");
    }

    #[test]
    fn test_style_validation() {
        let mut style = StyleSpec::new("test");
        style.add_source(SourceSpec {
            id: "test_source".to_string(),
            source_type: SourceType::Vector,
            url: Some("https://example.com/tiles".to_string()),
            tiles: None,
            minzoom: None,
            maxzoom: None,
        });

        assert!(StyleParser::validate(&style).is_ok());
    }
}
