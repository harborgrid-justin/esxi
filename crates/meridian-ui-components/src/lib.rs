//! Meridian UI Components - Enterprise React/TypeScript UI Library
//!
//! This crate provides WASM bindings for high-performance GIS operations
//! that complement the React UI components.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod bindings;

/// Initialize panic hook for better error messages in the browser
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Geographic coordinates in WGS84
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Coordinate {
    pub lon: f64,
    pub lat: f64,
}

#[wasm_bindgen]
impl Coordinate {
    #[wasm_bindgen(constructor)]
    pub fn new(lon: f64, lat: f64) -> Self {
        Self { lon, lat }
    }

    /// Convert to Web Mercator projection
    pub fn to_web_mercator(&self) -> WebMercatorCoordinate {
        let x = self.lon * 20037508.34 / 180.0;
        let y = (std::f64::consts::PI * self.lat / 180.0).tan()
            .atanh()
            * 20037508.34
            / std::f64::consts::PI;

        WebMercatorCoordinate { x, y }
    }
}

/// Web Mercator projected coordinates (EPSG:3857)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct WebMercatorCoordinate {
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
impl WebMercatorCoordinate {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Convert to WGS84 geographic coordinates
    pub fn to_geographic(&self) -> Coordinate {
        let lon = (self.x / 20037508.34) * 180.0;
        let lat = (self.y / 20037508.34 * std::f64::consts::PI).tanh().atan() * 180.0 / std::f64::consts::PI;

        Coordinate { lon, lat }
    }
}

/// Bounding box for spatial extents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct BoundingBox {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
}

#[wasm_bindgen]
impl BoundingBox {
    #[wasm_bindgen(constructor)]
    pub fn new(min_lon: f64, min_lat: f64, max_lon: f64, max_lat: f64) -> Self {
        Self {
            min_lon,
            min_lat,
            max_lon,
            max_lat,
        }
    }

    /// Calculate center point
    pub fn center(&self) -> Coordinate {
        Coordinate {
            lon: (self.min_lon + self.max_lon) / 2.0,
            lat: (self.min_lat + self.max_lat) / 2.0,
        }
    }

    /// Check if point is within bounds
    pub fn contains(&self, coord: &Coordinate) -> bool {
        coord.lon >= self.min_lon
            && coord.lon <= self.max_lon
            && coord.lat >= self.min_lat
            && coord.lat <= self.max_lat
    }

    /// Calculate area in square degrees
    pub fn area(&self) -> f64 {
        (self.max_lon - self.min_lon).abs() * (self.max_lat - self.min_lat).abs()
    }
}

/// Calculate distance between two points using Haversine formula
#[wasm_bindgen]
pub fn calculate_distance(coord1: &Coordinate, coord2: &Coordinate) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = coord1.lat.to_radians();
    let lat2_rad = coord2.lat.to_radians();
    let delta_lat = (coord2.lat - coord1.lat).to_radians();
    let delta_lon = (coord2.lon - coord1.lon).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS_KM * c
}

/// Calculate area of a polygon using Shoelace formula
#[wasm_bindgen]
pub fn calculate_polygon_area(coordinates: JsValue) -> Result<f64, JsValue> {
    let coords: Vec<Coordinate> = serde_wasm_bindgen::from_value(coordinates)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse coordinates: {}", e)))?;

    if coords.len() < 3 {
        return Err(JsValue::from_str("Polygon must have at least 3 points"));
    }

    // Convert to Web Mercator for accurate area calculation
    let projected: Vec<_> = coords.iter().map(|c| c.to_web_mercator()).collect();

    let mut area = 0.0;
    for i in 0..projected.len() {
        let j = (i + 1) % projected.len();
        area += projected[i].x * projected[j].y;
        area -= projected[j].x * projected[i].y;
    }

    Ok((area / 2.0).abs() / 1_000_000.0) // Convert to square kilometers
}

/// Create buffer around a point
#[wasm_bindgen]
pub fn create_point_buffer(center: &Coordinate, radius_km: f64, segments: usize) -> JsValue {
    let mut points = Vec::new();
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat_rad = center.lat.to_radians();

    for i in 0..segments {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (segments as f64);

        let delta_lat = (radius_km / EARTH_RADIUS_KM) * angle.cos();
        let delta_lon = (radius_km / EARTH_RADIUS_KM) * angle.sin() / lat_rad.cos();

        points.push(Coordinate {
            lon: center.lon + delta_lon.to_degrees(),
            lat: center.lat + delta_lat.to_degrees(),
        });
    }

    serde_wasm_bindgen::to_value(&points).unwrap_or(JsValue::NULL)
}

/// Performance metrics for rendering operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct RenderMetrics {
    pub frames_rendered: u32,
    pub average_frame_time_ms: f64,
    pub features_rendered: usize,
    pub tiles_loaded: usize,
}

#[wasm_bindgen]
impl RenderMetrics {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            frames_rendered: 0,
            average_frame_time_ms: 0.0,
            features_rendered: 0,
            tiles_loaded: 0,
        }
    }

    /// Get metrics as JSON
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(self)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }
}

/// Spatial index for efficient feature lookups
#[wasm_bindgen]
pub struct SpatialIndex {
    features: HashMap<u32, BoundingBox>,
}

#[wasm_bindgen]
impl SpatialIndex {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            features: HashMap::new(),
        }
    }

    /// Insert a feature with its bounding box
    pub fn insert(&mut self, feature_id: u32, bbox: BoundingBox) {
        self.features.insert(feature_id, bbox);
    }

    /// Query features intersecting with bounding box
    pub fn query(&self, query_bbox: &BoundingBox) -> Vec<u32> {
        self.features
            .iter()
            .filter(|(_, bbox)| {
                // Simple bounding box intersection test
                !(bbox.max_lon < query_bbox.min_lon
                    || bbox.min_lon > query_bbox.max_lon
                    || bbox.max_lat < query_bbox.min_lat
                    || bbox.min_lat > query_bbox.max_lat)
            })
            .map(|(id, _)| *id)
            .collect()
    }

    /// Clear all features
    pub fn clear(&mut self) {
        self.features.clear();
    }

    /// Get feature count
    pub fn size(&self) -> usize {
        self.features.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_conversion() {
        let coord = Coordinate::new(0.0, 0.0);
        let mercator = coord.to_web_mercator();
        assert_eq!(mercator.x, 0.0);
        assert_eq!(mercator.y, 0.0);
    }

    #[test]
    fn test_distance_calculation() {
        let coord1 = Coordinate::new(0.0, 0.0);
        let coord2 = Coordinate::new(1.0, 1.0);
        let distance = calculate_distance(&coord1, &coord2);
        assert!(distance > 0.0);
        assert!(distance < 200.0); // Approximately 157 km
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(-180.0, -90.0, 180.0, 90.0);
        let center = bbox.center();
        assert_eq!(center.lon, 0.0);
        assert_eq!(center.lat, 0.0);
    }
}
