//! TileJSON metadata specification
//!
//! TileJSON is a format for describing tiled map data.
//! https://github.com/mapbox/tilejson-spec

use serde::{Deserialize, Serialize};

/// TileJSON specification (version 3.0.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileJSON {
    /// TileJSON version (always "3.0.0")
    pub tilejson: String,

    /// Tile URLs template
    pub tiles: Vec<String>,

    /// Name of the tileset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Description of the tileset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Version of the tileset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Attribution text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,

    /// Template URL for retrieving TileJSON
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,

    /// Legend URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legend: Option<String>,

    /// Tile scheme (xyz or tms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,

    /// Bounds [west, south, east, north]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounds: Option<[f64; 4]>,

    /// Center [longitude, latitude, zoom]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center: Option<[f64; 3]>,

    /// Minimum zoom level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<u8>,

    /// Maximum zoom level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<u8>,

    /// Vector layers (for vector tiles)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_layers: Option<Vec<VectorLayer>>,

    /// Data files (alternative to tiles)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<String>>,

    /// Tile size (default: 512)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilesize: Option<u32>,

    /// Fill zoom (overzoom)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fillzoom: Option<u8>,
}

/// Vector layer description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorLayer {
    /// Layer ID
    pub id: String,

    /// Layer description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Minimum zoom level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minzoom: Option<u8>,

    /// Maximum zoom level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<u8>,

    /// Field definitions (name -> type)
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty", default)]
    pub fields: std::collections::HashMap<String, String>,
}

impl TileJSON {
    /// Create a new TileJSON with required fields
    pub fn new(tiles: Vec<String>) -> Self {
        Self {
            tilejson: "3.0.0".to_string(),
            tiles,
            name: None,
            description: None,
            version: None,
            attribution: None,
            template: None,
            legend: None,
            scheme: Some("xyz".to_string()),
            bounds: None,
            center: None,
            minzoom: None,
            maxzoom: None,
            vector_layers: None,
            data: None,
            tilesize: None,
            fillzoom: None,
        }
    }

    /// Set name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set bounds
    pub fn with_bounds(mut self, bounds: [f64; 4]) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Set center
    pub fn with_center(mut self, center: [f64; 3]) -> Self {
        self.center = Some(center);
        self
    }

    /// Set zoom range
    pub fn with_zoom_range(mut self, minzoom: u8, maxzoom: u8) -> Self {
        self.minzoom = Some(minzoom);
        self.maxzoom = Some(maxzoom);
        self
    }

    /// Add vector layer
    pub fn add_vector_layer(mut self, layer: VectorLayer) -> Self {
        self.vector_layers
            .get_or_insert_with(Vec::new)
            .push(layer);
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

impl VectorLayer {
    /// Create a new vector layer
    pub fn new(id: String) -> Self {
        Self {
            id,
            description: None,
            minzoom: None,
            maxzoom: None,
            fields: std::collections::HashMap::new(),
        }
    }

    /// Add a field
    pub fn add_field(mut self, name: String, field_type: String) -> Self {
        self.fields.insert(name, field_type);
        self
    }

    /// Set zoom range
    pub fn with_zoom_range(mut self, minzoom: u8, maxzoom: u8) -> Self {
        self.minzoom = Some(minzoom);
        self.maxzoom = Some(maxzoom);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tilejson_creation() {
        let tilejson = TileJSON::new(vec!["http://localhost/tiles/{z}/{x}/{y}.mvt".to_string()]);
        assert_eq!(tilejson.tilejson, "3.0.0");
        assert_eq!(tilejson.tiles.len(), 1);
    }

    #[test]
    fn test_tilejson_builder() {
        let tilejson = TileJSON::new(vec!["http://localhost/tiles/{z}/{x}/{y}.mvt".to_string()])
            .with_name("Test Tileset".to_string())
            .with_bounds([-180.0, -85.0, 180.0, 85.0])
            .with_zoom_range(0, 14);

        assert_eq!(tilejson.name, Some("Test Tileset".to_string()));
        assert_eq!(tilejson.minzoom, Some(0));
        assert_eq!(tilejson.maxzoom, Some(14));
    }

    #[test]
    fn test_vector_layer() {
        let layer = VectorLayer::new("roads".to_string())
            .add_field("name".to_string(), "string".to_string())
            .add_field("lanes".to_string(), "integer".to_string())
            .with_zoom_range(0, 14);

        assert_eq!(layer.id, "roads");
        assert_eq!(layer.fields.len(), 2);
        assert_eq!(layer.minzoom, Some(0));
    }

    #[test]
    fn test_tilejson_serialization() {
        let tilejson = TileJSON::new(vec!["http://localhost/tiles/{z}/{x}/{y}.mvt".to_string()])
            .with_name("Test".to_string());

        let json = tilejson.to_json().unwrap();
        assert!(json.contains("\"tilejson\":\"3.0.0\""));
        assert!(json.contains("\"name\":\"Test\""));

        let parsed = TileJSON::from_json(&json).unwrap();
        assert_eq!(parsed.name, Some("Test".to_string()));
    }
}
