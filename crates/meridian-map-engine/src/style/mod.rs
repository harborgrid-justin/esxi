//! Style specification and rendering styles.

pub mod evaluator;
pub mod parser;

use palette::Srgba;
use serde::{Deserialize, Serialize};

/// Style specification for map features (Mapbox-style).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSpec {
    /// Style version.
    pub version: u8,
    /// Style name.
    pub name: String,
    /// Data sources.
    pub sources: Vec<SourceSpec>,
    /// Style layers.
    pub layers: Vec<LayerStyleSpec>,
}

impl StyleSpec {
    /// Create a new empty style specification.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            version: 8,
            name: name.into(),
            sources: Vec::new(),
            layers: Vec::new(),
        }
    }

    /// Add a source to the style.
    pub fn add_source(&mut self, source: SourceSpec) {
        self.sources.push(source);
    }

    /// Add a layer to the style.
    pub fn add_layer(&mut self, layer: LayerStyleSpec) {
        self.layers.push(layer);
    }
}

/// Data source specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpec {
    /// Source ID.
    pub id: String,
    /// Source type.
    #[serde(rename = "type")]
    pub source_type: SourceType,
    /// Source URL (for remote sources).
    pub url: Option<String>,
    /// Tile URLs (for vector/raster tiles).
    pub tiles: Option<Vec<String>>,
    /// Minimum zoom level.
    pub minzoom: Option<u8>,
    /// Maximum zoom level.
    pub maxzoom: Option<u8>,
}

/// Source type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    /// Vector tile source.
    Vector,
    /// Raster tile source.
    Raster,
    /// GeoJSON source.
    GeoJson,
    /// Image source.
    Image,
}

/// Layer style specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStyleSpec {
    /// Layer ID.
    pub id: String,
    /// Layer type.
    #[serde(rename = "type")]
    pub layer_type: StyleLayerType,
    /// Source ID.
    pub source: String,
    /// Source layer (for vector tiles).
    #[serde(rename = "source-layer")]
    pub source_layer: Option<String>,
    /// Filter expression.
    pub filter: Option<FilterExpression>,
    /// Paint properties.
    pub paint: Option<PaintProperties>,
    /// Layout properties.
    pub layout: Option<LayoutProperties>,
}

/// Style layer type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StyleLayerType {
    /// Fill (polygon) layer.
    Fill,
    /// Line layer.
    Line,
    /// Symbol (text/icon) layer.
    Symbol,
    /// Circle (point) layer.
    Circle,
    /// Raster layer.
    Raster,
}

/// Filter expression for feature filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterExpression {
    /// Comparison filter ["==", "property", value].
    Comparison(Vec<serde_json::Value>),
    /// Logical filter ["all", filter1, filter2, ...].
    Logical(Vec<serde_json::Value>),
}

/// Paint properties for styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaintProperties {
    /// Fill color (for polygons).
    #[serde(rename = "fill-color")]
    pub fill_color: Option<ColorOrExpression>,
    /// Fill opacity.
    #[serde(rename = "fill-opacity")]
    pub fill_opacity: Option<NumberOrExpression>,
    /// Line color.
    #[serde(rename = "line-color")]
    pub line_color: Option<ColorOrExpression>,
    /// Line width.
    #[serde(rename = "line-width")]
    pub line_width: Option<NumberOrExpression>,
    /// Line opacity.
    #[serde(rename = "line-opacity")]
    pub line_opacity: Option<NumberOrExpression>,
    /// Circle color.
    #[serde(rename = "circle-color")]
    pub circle_color: Option<ColorOrExpression>,
    /// Circle radius.
    #[serde(rename = "circle-radius")]
    pub circle_radius: Option<NumberOrExpression>,
    /// Circle opacity.
    #[serde(rename = "circle-opacity")]
    pub circle_opacity: Option<NumberOrExpression>,
}

/// Layout properties for positioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutProperties {
    /// Text field.
    #[serde(rename = "text-field")]
    pub text_field: Option<String>,
    /// Text size.
    #[serde(rename = "text-size")]
    pub text_size: Option<NumberOrExpression>,
    /// Text font.
    #[serde(rename = "text-font")]
    pub text_font: Option<Vec<String>>,
    /// Icon image.
    #[serde(rename = "icon-image")]
    pub icon_image: Option<String>,
    /// Icon size.
    #[serde(rename = "icon-size")]
    pub icon_size: Option<NumberOrExpression>,
}

/// Color value or data-driven expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorOrExpression {
    /// Static color string.
    Color(String),
    /// Data-driven expression.
    Expression(Vec<serde_json::Value>),
}

/// Number value or data-driven expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NumberOrExpression {
    /// Static number.
    Number(f32),
    /// Data-driven expression.
    Expression(Vec<serde_json::Value>),
}

/// Color utilities.
pub struct ColorUtils;

impl ColorUtils {
    /// Parse a color string to RGBA values.
    pub fn parse_color(color: &str) -> Result<[f32; 4], String> {
        // Try parsing hex color
        if color.starts_with('#') {
            return Self::parse_hex_color(color);
        }

        // Try parsing rgb/rgba
        if color.starts_with("rgb") {
            return Self::parse_rgb_color(color);
        }

        // Try named colors
        Self::parse_named_color(color)
    }

    /// Parse hex color (#RGB or #RRGGBB or #RRGGBBAA).
    fn parse_hex_color(color: &str) -> Result<[f32; 4], String> {
        let hex = color.trim_start_matches('#');

        match hex.len() {
            3 => {
                // #RGB
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                    .map_err(|e| e.to_string())?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                    .map_err(|e| e.to_string())?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                    .map_err(|e| e.to_string())?;
                Ok([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0])
            }
            6 => {
                // #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
                Ok([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0])
            }
            8 => {
                // #RRGGBBAA
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|e| e.to_string())?;
                Ok([
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                ])
            }
            _ => Err(format!("Invalid hex color: {}", color)),
        }
    }

    /// Parse RGB/RGBA color.
    fn parse_rgb_color(color: &str) -> Result<[f32; 4], String> {
        // Simplified RGB parsing
        // In production, use a proper CSS color parser
        Ok([1.0, 1.0, 1.0, 1.0])
    }

    /// Parse named color.
    fn parse_named_color(color: &str) -> Result<[f32; 4], String> {
        match color.to_lowercase().as_str() {
            "black" => Ok([0.0, 0.0, 0.0, 1.0]),
            "white" => Ok([1.0, 1.0, 1.0, 1.0]),
            "red" => Ok([1.0, 0.0, 0.0, 1.0]),
            "green" => Ok([0.0, 1.0, 0.0, 1.0]),
            "blue" => Ok([0.0, 0.0, 1.0, 1.0]),
            "yellow" => Ok([1.0, 1.0, 0.0, 1.0]),
            "cyan" => Ok([0.0, 1.0, 1.0, 1.0]),
            "magenta" => Ok([1.0, 0.0, 1.0, 1.0]),
            _ => Err(format!("Unknown color: {}", color)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_spec_creation() {
        let style = StyleSpec::new("test_style");
        assert_eq!(style.name, "test_style");
        assert_eq!(style.version, 8);
        assert_eq!(style.sources.len(), 0);
        assert_eq!(style.layers.len(), 0);
    }

    #[test]
    fn test_color_parsing_hex() {
        let color = ColorUtils::parse_color("#ff0000").unwrap();
        assert_eq!(color, [1.0, 0.0, 0.0, 1.0]);

        let color = ColorUtils::parse_color("#00ff00").unwrap();
        assert_eq!(color, [0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn test_color_parsing_named() {
        let color = ColorUtils::parse_color("red").unwrap();
        assert_eq!(color, [1.0, 0.0, 0.0, 1.0]);

        let color = ColorUtils::parse_color("white").unwrap();
        assert_eq!(color, [1.0, 1.0, 1.0, 1.0]);
    }
}
