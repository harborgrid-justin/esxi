//! Mapbox Style Specification types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mapbox Style Specification (v8)
/// https://docs.mapbox.com/mapbox-gl-js/style-spec/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    /// Style version (always 8)
    pub version: u8,

    /// Style name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Style metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,

    /// Center point [longitude, latitude]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center: Option<[f64; 2]>,

    /// Zoom level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoom: Option<f64>,

    /// Bearing (rotation) in degrees
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearing: Option<f64>,

    /// Pitch in degrees
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f64>,

    /// Data sources
    pub sources: HashMap<String, StyleSource>,

    /// Sprite URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprite: Option<String>,

    /// Glyphs URL template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glyphs: Option<String>,

    /// Style layers
    pub layers: Vec<StyleLayer>,

    /// Transition properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,
}

/// Style source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StyleSource {
    /// Vector tile source
    Vector {
        /// TileJSON URL or inline TileJSON
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,

        /// Tile URLs
        #[serde(skip_serializing_if = "Option::is_none")]
        tiles: Option<Vec<String>>,

        /// Bounds [west, south, east, north]
        #[serde(skip_serializing_if = "Option::is_none")]
        bounds: Option<[f64; 4]>,

        /// Scheme (xyz or tms)
        #[serde(skip_serializing_if = "Option::is_none")]
        scheme: Option<String>,

        /// Min zoom
        #[serde(skip_serializing_if = "Option::is_none")]
        minzoom: Option<u8>,

        /// Max zoom
        #[serde(skip_serializing_if = "Option::is_none")]
        maxzoom: Option<u8>,

        /// Attribution
        #[serde(skip_serializing_if = "Option::is_none")]
        attribution: Option<String>,
    },

    /// Raster tile source
    Raster {
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,

        #[serde(skip_serializing_if = "Option::is_none")]
        tiles: Option<Vec<String>>,

        #[serde(skip_serializing_if = "Option::is_none")]
        tileSize: Option<u32>,
    },

    /// GeoJSON source
    GeoJSON {
        data: serde_json::Value,
    },
}

/// Style layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleLayer {
    /// Layer ID
    pub id: String,

    /// Layer type
    #[serde(rename = "type")]
    pub layer_type: LayerType,

    /// Source ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Source layer (for vector tiles)
    #[serde(skip_serializing_if = "Option::is_none", rename = "source-layer")]
    pub source_layer: Option<String>,

    /// Min zoom
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<u8>,

    /// Max zoom
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<u8>,

    /// Filter expression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<serde_json::Value>,

    /// Layout properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<HashMap<String, serde_json::Value>>,

    /// Paint properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paint: Option<HashMap<String, serde_json::Value>>,
}

/// Layer type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayerType {
    Fill,
    Line,
    Symbol,
    Circle,
    Heatmap,
    #[serde(rename = "fill-extrusion")]
    FillExtrusion,
    Raster,
    Hillshade,
    Background,
}

/// Transition properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Duration in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u32>,

    /// Delay in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<u32>,
}

impl Style {
    /// Create a new empty style
    pub fn new() -> Self {
        Self {
            version: 8,
            name: None,
            metadata: None,
            center: None,
            zoom: None,
            bearing: None,
            pitch: None,
            sources: HashMap::new(),
            sprite: None,
            glyphs: None,
            layers: Vec::new(),
            transition: None,
        }
    }

    /// Add a source
    pub fn add_source(mut self, id: String, source: StyleSource) -> Self {
        self.sources.insert(id, source);
        self
    }

    /// Add a layer
    pub fn add_layer(mut self, layer: StyleLayer) -> Self {
        self.layers.push(layer);
        self
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_creation() {
        let style = Style::new();
        assert_eq!(style.version, 8);
        assert!(style.sources.is_empty());
        assert!(style.layers.is_empty());
    }

    #[test]
    fn test_style_serialization() {
        let style = Style::new();
        let json = style.to_json().unwrap();
        assert!(json.contains("\"version\":8"));
    }

    #[test]
    fn test_layer_type() {
        let layer = StyleLayer {
            id: "test".to_string(),
            layer_type: LayerType::Fill,
            source: Some("source".to_string()),
            source_layer: None,
            minzoom: None,
            maxzoom: None,
            filter: None,
            layout: None,
            paint: None,
        };

        assert_eq!(layer.layer_type, LayerType::Fill);
    }
}
