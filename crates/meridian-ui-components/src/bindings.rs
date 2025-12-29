//! TypeScript/JavaScript bindings for Meridian UI Components
//!
//! Provides ergonomic interfaces for calling Rust/WASM functions from TypeScript

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Style configuration for map layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStyle {
    pub fill_color: String,
    pub stroke_color: String,
    pub stroke_width: f32,
    pub opacity: f32,
    pub z_index: i32,
}

impl Default for LayerStyle {
    fn default() -> Self {
        Self {
            fill_color: "#3b82f6".to_string(),
            stroke_color: "#1e40af".to_string(),
            stroke_width: 2.0,
            opacity: 0.8,
            z_index: 0,
        }
    }
}

/// Feature properties for GIS data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureProperties {
    pub id: u32,
    pub name: String,
    pub layer_id: String,
    pub attributes: HashMap<String, String>,
    pub visible: bool,
    pub selected: bool,
}

/// Map view state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapViewState {
    pub center_lon: f64,
    pub center_lat: f64,
    pub zoom: f64,
    pub rotation: f64,
    pub pitch: f64,
}

impl Default for MapViewState {
    fn default() -> Self {
        Self {
            center_lon: 0.0,
            center_lat: 0.0,
            zoom: 2.0,
            rotation: 0.0,
            pitch: 0.0,
        }
    }
}

/// Layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfig {
    pub id: String,
    pub name: String,
    pub layer_type: String, // "vector", "raster", "tile"
    pub source_url: Option<String>,
    pub style: LayerStyle,
    pub visible: bool,
    pub opacity: f32,
    pub min_zoom: f64,
    pub max_zoom: f64,
}

/// Measurement result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct MeasurementResult {
    pub measurement_type: String, // "distance", "area", "perimeter"
    pub value: f64,
    pub unit: String,
}

#[wasm_bindgen]
impl MeasurementResult {
    #[wasm_bindgen(constructor)]
    pub fn new(measurement_type: String, value: f64, unit: String) -> Self {
        Self {
            measurement_type,
            value,
            unit,
        }
    }

    /// Export as JSON
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(self)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }
}

/// Query builder for spatial queries
#[wasm_bindgen]
pub struct QueryBuilder {
    layer_id: Option<String>,
    geometry_type: Option<String>,
    spatial_relation: Option<String>,
    attributes: HashMap<String, String>,
}

#[wasm_bindgen]
impl QueryBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            layer_id: None,
            geometry_type: None,
            spatial_relation: None,
            attributes: HashMap::new(),
        }
    }

    /// Set layer to query
    pub fn set_layer(&mut self, layer_id: String) {
        self.layer_id = Some(layer_id);
    }

    /// Set geometry type filter
    pub fn set_geometry_type(&mut self, geometry_type: String) {
        self.geometry_type = Some(geometry_type);
    }

    /// Set spatial relationship (intersects, contains, within, etc.)
    pub fn set_spatial_relation(&mut self, relation: String) {
        self.spatial_relation = Some(relation);
    }

    /// Add attribute filter
    pub fn add_attribute_filter(&mut self, key: String, value: String) {
        self.attributes.insert(key, value);
    }

    /// Build query as JSON
    pub fn build(&self) -> Result<JsValue, JsValue> {
        let query = serde_json::json!({
            "layer_id": self.layer_id,
            "geometry_type": self.geometry_type,
            "spatial_relation": self.spatial_relation,
            "attributes": self.attributes,
        });

        serde_wasm_bindgen::to_value(&query)
            .map_err(|e| JsValue::from_str(&format!("Query build failed: {}", e)))
    }
}

/// Style rules for dynamic styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRule {
    pub property: String,
    pub operator: String, // "==", "!=", ">", "<", ">=", "<=", "contains"
    pub value: String,
    pub style: LayerStyle,
}

/// Theme configuration for UI components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub primary_color: String,
    pub secondary_color: String,
    pub background_color: String,
    pub text_color: String,
    pub border_color: String,
    pub accent_color: String,
    pub dark_mode: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary_color: "#3b82f6".to_string(),
            secondary_color: "#8b5cf6".to_string(),
            background_color: "#ffffff".to_string(),
            text_color: "#1f2937".to_string(),
            border_color: "#e5e7eb".to_string(),
            accent_color: "#10b981".to_string(),
            dark_mode: false,
        }
    }
}

/// Export configuration for data export
#[wasm_bindgen]
pub struct ExportConfig {
    format: String, // "geojson", "csv", "kml", "shapefile"
    crs: String,    // Coordinate reference system
    include_attributes: bool,
}

#[wasm_bindgen]
impl ExportConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(format: String, crs: String, include_attributes: bool) -> Self {
        Self {
            format,
            crs,
            include_attributes,
        }
    }

    /// Get format
    pub fn get_format(&self) -> String {
        self.format.clone()
    }

    /// Get CRS
    pub fn get_crs(&self) -> String {
        self.crs.clone()
    }

    /// Check if attributes should be included
    pub fn should_include_attributes(&self) -> bool {
        self.include_attributes
    }
}

/// Validate GeoJSON structure
#[wasm_bindgen]
pub fn validate_geojson(geojson: JsValue) -> Result<bool, JsValue> {
    let json: serde_json::Value = serde_wasm_bindgen::from_value(geojson)
        .map_err(|e| JsValue::from_str(&format!("Invalid JSON: {}", e)))?;

    // Basic GeoJSON validation
    if !json.is_object() {
        return Ok(false);
    }

    let obj = json.as_object().unwrap();

    // Must have type field
    if !obj.contains_key("type") {
        return Ok(false);
    }

    let geojson_type = obj["type"].as_str().unwrap_or("");

    // Validate based on type
    match geojson_type {
        "Feature" => Ok(obj.contains_key("geometry") && obj.contains_key("properties")),
        "FeatureCollection" => Ok(obj.contains_key("features")),
        "Point" | "LineString" | "Polygon" | "MultiPoint" | "MultiLineString" | "MultiPolygon" => {
            Ok(obj.contains_key("coordinates"))
        }
        _ => Ok(false),
    }
}

/// Parse color string to RGBA
#[wasm_bindgen]
pub fn parse_color(color: &str) -> Result<JsValue, JsValue> {
    let color = color.trim();

    // Handle hex colors
    if color.starts_with('#') {
        let hex = &color[1..];
        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| JsValue::from_str("Invalid hex color"))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| JsValue::from_str("Invalid hex color"))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| JsValue::from_str("Invalid hex color"))?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| JsValue::from_str("Invalid hex color"))?
        } else {
            255
        };

        let rgba = serde_json::json!({
            "r": r,
            "g": g,
            "b": b,
            "a": a as f32 / 255.0,
        });

        return serde_wasm_bindgen::to_value(&rgba)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)));
    }

    Err(JsValue::from_str("Unsupported color format"))
}

/// Generate unique ID for features
#[wasm_bindgen]
pub fn generate_feature_id() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_style_default() {
        let style = LayerStyle::default();
        assert_eq!(style.fill_color, "#3b82f6");
        assert_eq!(style.opacity, 0.8);
    }

    #[test]
    fn test_map_view_state_default() {
        let state = MapViewState::default();
        assert_eq!(state.center_lon, 0.0);
        assert_eq!(state.zoom, 2.0);
    }
}
