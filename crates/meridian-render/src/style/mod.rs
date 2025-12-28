//! Map styling system supporting Mapbox GL style specification

use crate::error::{RenderError, RenderResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Map style definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    /// Style version
    pub version: u8,
    /// Style name
    pub name: String,
    /// Data sources
    #[serde(default)]
    pub sources: HashMap<String, Source>,
    /// Layers
    pub layers: Vec<Layer>,
    /// Sprite URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprite: Option<String>,
    /// Glyphs URL template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glyphs: Option<String>,
}

impl Style {
    /// Create a new style
    pub fn new(name: String) -> Self {
        Style {
            version: 8,
            name,
            sources: HashMap::new(),
            layers: Vec::new(),
            sprite: None,
            glyphs: None,
        }
    }

    /// Add a source
    pub fn add_source(&mut self, id: String, source: Source) {
        self.sources.insert(id, source);
    }

    /// Add a layer
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// Get layers for a specific source
    pub fn layers_for_source(&self, source_id: &str) -> Vec<&Layer> {
        self.layers
            .iter()
            .filter(|layer| layer.source.as_ref() == Some(&source_id.to_string()))
            .collect()
    }

    /// Load style from JSON
    pub fn from_json(json: &str) -> RenderResult<Self> {
        serde_json::from_str(json).map_err(|e| RenderError::StyleError(format!("Invalid style JSON: {}", e)))
    }

    /// Convert style to JSON
    pub fn to_json(&self) -> RenderResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| RenderError::StyleError(e.to_string()))
    }
}

/// Data source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Source {
    /// Vector tile source
    #[serde(rename = "vector")]
    Vector {
        /// Tile URLs
        tiles: Vec<String>,
        /// Min zoom
        #[serde(skip_serializing_if = "Option::is_none")]
        minzoom: Option<u8>,
        /// Max zoom
        #[serde(skip_serializing_if = "Option::is_none")]
        maxzoom: Option<u8>,
    },
    /// Raster tile source
    #[serde(rename = "raster")]
    Raster {
        /// Tile URLs
        tiles: Vec<String>,
        /// Tile size (default 256)
        #[serde(skip_serializing_if = "Option::is_none")]
        tileSize: Option<u32>,
    },
    /// GeoJSON source
    #[serde(rename = "geojson")]
    GeoJson {
        /// GeoJSON data
        data: serde_json::Value,
    },
}

/// Layer definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
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
    pub minzoom: Option<f64>,
    /// Max zoom
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maxzoom: Option<f64>,
    /// Filter expression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,
    /// Paint properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paint: Option<PaintProperties>,
    /// Layout properties
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutProperties>,
}

impl Layer {
    /// Check if layer is visible at zoom level
    pub fn is_visible_at_zoom(&self, zoom: f64) -> bool {
        if let Some(minzoom) = self.minzoom {
            if zoom < minzoom {
                return false;
            }
        }
        if let Some(maxzoom) = self.maxzoom {
            if zoom >= maxzoom {
                return false;
            }
        }
        true
    }
}

/// Layer type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LayerType {
    Fill,
    Line,
    Symbol,
    Circle,
    Raster,
    Background,
    Heatmap,
    Hillshade,
}

/// Filter expression for features
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Filter {
    /// Array-based filter (legacy)
    Array(Vec<serde_json::Value>),
    /// Expression-based filter
    Expression(Expression),
}

/// Style expression
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Expression {
    /// Literal value
    Literal(serde_json::Value),
    /// Array expression
    Array(Vec<serde_json::Value>),
}

/// Paint properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaintProperties {
    // Fill properties
    #[serde(skip_serializing_if = "Option::is_none", rename = "fill-color")]
    pub fill_color: Option<PropertyValue<Color>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "fill-opacity")]
    pub fill_opacity: Option<PropertyValue<f64>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "fill-outline-color")]
    pub fill_outline_color: Option<PropertyValue<Color>>,

    // Line properties
    #[serde(skip_serializing_if = "Option::is_none", rename = "line-color")]
    pub line_color: Option<PropertyValue<Color>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "line-width")]
    pub line_width: Option<PropertyValue<f64>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "line-opacity")]
    pub line_opacity: Option<PropertyValue<f64>>,

    // Circle properties
    #[serde(skip_serializing_if = "Option::is_none", rename = "circle-radius")]
    pub circle_radius: Option<PropertyValue<f64>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "circle-color")]
    pub circle_color: Option<PropertyValue<Color>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "circle-opacity")]
    pub circle_opacity: Option<PropertyValue<f64>>,

    // Text properties
    #[serde(skip_serializing_if = "Option::is_none", rename = "text-color")]
    pub text_color: Option<PropertyValue<Color>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "text-halo-color")]
    pub text_halo_color: Option<PropertyValue<Color>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "text-halo-width")]
    pub text_halo_width: Option<PropertyValue<f64>>,
}

/// Layout properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,

    // Symbol layout
    #[serde(skip_serializing_if = "Option::is_none", rename = "text-field")]
    pub text_field: Option<PropertyValue<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "text-font")]
    pub text_font: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "text-size")]
    pub text_size: Option<PropertyValue<f64>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "icon-image")]
    pub icon_image: Option<PropertyValue<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "icon-size")]
    pub icon_size: Option<PropertyValue<f64>>,

    // Line layout
    #[serde(skip_serializing_if = "Option::is_none", rename = "line-cap")]
    pub line_cap: Option<LineCap>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "line-join")]
    pub line_join: Option<LineJoin>,
}

/// Visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Visible,
    None,
}

/// Line cap style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

/// Line join style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineJoin {
    Bevel,
    Round,
    Miter,
}

/// Property value with zoom-dependent styling
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue<T> {
    /// Constant value
    Constant(T),
    /// Expression
    Expression(Expression),
}

impl<T> PropertyValue<T>
where
    T: Clone,
{
    /// Get the constant value if available
    pub fn constant(&self) -> Option<&T> {
        match self {
            PropertyValue::Constant(v) => Some(v),
            PropertyValue::Expression(_) => None,
        }
    }
}

/// Color representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Color {
    /// RGB string (e.g., "#ff0000" or "rgb(255, 0, 0)")
    String(String),
    /// RGBA array [r, g, b, a]
    Array([u8; 4]),
}

impl Color {
    /// Parse color to RGBA
    pub fn to_rgba(&self) -> RenderResult<[u8; 4]> {
        match self {
            Color::Array(rgba) => Ok(*rgba),
            Color::String(s) => parse_color(s),
        }
    }

    /// Create from RGB
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::Array([r, g, b, 255])
    }

    /// Create from RGBA
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color::Array([r, g, b, a])
    }
}

/// Parse color string to RGBA
fn parse_color(s: &str) -> RenderResult<[u8; 4]> {
    let s = s.trim();

    // Hex color
    if let Some(hex) = s.strip_prefix('#') {
        let hex = hex.trim();
        match hex.len() {
            3 => {
                // #RGB
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                    .map_err(|_| RenderError::StyleError(format!("Invalid color: {}", s)))?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                    .map_err(|_| RenderError::StyleError(format!("Invalid color: {}", s)))?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                    .map_err(|_| RenderError::StyleError(format!("Invalid color: {}", s)))?;
                Ok([r, g, b, 255])
            }
            6 => {
                // #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| RenderError::StyleError(format!("Invalid color: {}", s)))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| RenderError::StyleError(format!("Invalid color: {}", s)))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| RenderError::StyleError(format!("Invalid color: {}", s)))?;
                Ok([r, g, b, 255])
            }
            8 => {
                // #RRGGBBAA
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| RenderError::StyleError(format!("Invalid color: {}", s)))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| RenderError::StyleError(format!("Invalid color: {}", s)))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| RenderError::StyleError(format!("Invalid color: {}", s)))?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|_| RenderError::StyleError(format!("Invalid color: {}", s)))?;
                Ok([r, g, b, a])
            }
            _ => Err(RenderError::StyleError(format!("Invalid hex color: {}", s))),
        }
    } else {
        // Named colors or RGB/RGBA
        Err(RenderError::StyleError(format!(
            "Color format not supported: {}",
            s
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_parsing() {
        assert_eq!(parse_color("#ff0000").unwrap(), [255, 0, 0, 255]);
        assert_eq!(parse_color("#00ff00").unwrap(), [0, 255, 0, 255]);
        assert_eq!(parse_color("#0000ff").unwrap(), [0, 0, 255, 255]);
        assert_eq!(parse_color("#f00").unwrap(), [255, 0, 0, 255]);
        assert_eq!(parse_color("#ff0000ff").unwrap(), [255, 0, 0, 255]);
        assert_eq!(parse_color("#ff000080").unwrap(), [255, 0, 0, 128]);
    }

    #[test]
    fn test_style_creation() {
        let mut style = Style::new("test".to_string());
        style.add_layer(Layer {
            id: "background".to_string(),
            layer_type: LayerType::Background,
            source: None,
            source_layer: None,
            minzoom: None,
            maxzoom: None,
            filter: None,
            paint: Some(PaintProperties {
                fill_color: Some(PropertyValue::Constant(Color::rgb(255, 255, 255))),
                fill_opacity: None,
                fill_outline_color: None,
                line_color: None,
                line_width: None,
                line_opacity: None,
                circle_radius: None,
                circle_color: None,
                circle_opacity: None,
                text_color: None,
                text_halo_color: None,
                text_halo_width: None,
            }),
            layout: None,
        });

        assert_eq!(style.layers.len(), 1);
        assert_eq!(style.layers[0].id, "background");
    }

    #[test]
    fn test_layer_zoom_visibility() {
        let layer = Layer {
            id: "test".to_string(),
            layer_type: LayerType::Fill,
            source: None,
            source_layer: None,
            minzoom: Some(5.0),
            maxzoom: Some(10.0),
            filter: None,
            paint: None,
            layout: None,
        };

        assert!(!layer.is_visible_at_zoom(4.0));
        assert!(layer.is_visible_at_zoom(5.0));
        assert!(layer.is_visible_at_zoom(7.0));
        assert!(layer.is_visible_at_zoom(9.9));
        assert!(!layer.is_visible_at_zoom(10.0));
    }

    #[test]
    fn test_color_to_rgba() {
        let color = Color::rgb(255, 128, 64);
        assert_eq!(color.to_rgba().unwrap(), [255, 128, 64, 255]);

        let color = Color::String("#ff8040".to_string());
        assert_eq!(color.to_rgba().unwrap(), [255, 128, 64, 255]);
    }
}
