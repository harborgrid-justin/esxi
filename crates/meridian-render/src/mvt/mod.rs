//! Mapbox Vector Tile (MVT) encoding and decoding
//!
//! Implements the MVT specification for efficient vector tile generation

use crate::error::{RenderError, RenderResult};
use crate::tile::{TileBounds, TileCoord};
use flate2::write::GzEncoder;
use flate2::Compression;
use geo_types::{Geometry, LineString, Point, Polygon};
use std::collections::HashMap;
use std::io::Write;

/// MVT extent (standard is 4096)
pub const MVT_EXTENT: u32 = 4096;

/// Feature in a vector tile layer
#[derive(Debug, Clone)]
pub struct Feature {
    /// Feature ID
    pub id: Option<u64>,
    /// Geometry
    pub geometry: Geometry<f64>,
    /// Properties
    pub properties: HashMap<String, Value>,
}

impl Feature {
    /// Create a new feature
    pub fn new(geometry: Geometry<f64>) -> Self {
        Feature {
            id: None,
            geometry,
            properties: HashMap::new(),
        }
    }

    /// Set feature ID
    pub fn with_id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    /// Add a property
    pub fn with_property(mut self, key: String, value: Value) -> Self {
        self.properties.insert(key, value);
        self
    }

    /// Add multiple properties
    pub fn with_properties(mut self, properties: HashMap<String, Value>) -> Self {
        self.properties.extend(properties);
        self
    }
}

/// Property value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Float(f64),
    Double(f64),
    Int(i64),
    UInt(u64),
    SInt(i64),
    Bool(bool),
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Double(f)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

/// Layer in a vector tile
#[derive(Debug, Clone)]
pub struct Layer {
    /// Layer name
    pub name: String,
    /// Features in this layer
    pub features: Vec<Feature>,
    /// Extent (default 4096)
    pub extent: u32,
}

impl Layer {
    /// Create a new layer
    pub fn new(name: String) -> Self {
        Layer {
            name,
            features: Vec::new(),
            extent: MVT_EXTENT,
        }
    }

    /// Add a feature
    pub fn add_feature(&mut self, feature: Feature) {
        self.features.push(feature);
    }

    /// Set extent
    pub fn with_extent(mut self, extent: u32) -> Self {
        self.extent = extent;
        self
    }

    /// Get number of features
    pub fn feature_count(&self) -> usize {
        self.features.len()
    }
}

/// Vector tile containing multiple layers
#[derive(Debug, Clone)]
pub struct VectorTile {
    /// Tile coordinate
    pub coord: TileCoord,
    /// Layers
    pub layers: Vec<Layer>,
}

impl VectorTile {
    /// Create a new vector tile
    pub fn new(coord: TileCoord) -> Self {
        VectorTile {
            coord,
            layers: Vec::new(),
        }
    }

    /// Add a layer
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// Get a layer by name
    pub fn get_layer(&self, name: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.name == name)
    }

    /// Get a mutable layer by name
    pub fn get_layer_mut(&mut self, name: &str) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.name == name)
    }

    /// Encode to MVT protobuf format
    pub fn encode(&self) -> RenderResult<Vec<u8>> {
        // For now, return a simple encoded representation
        // In a real implementation, this would use prost to encode to protobuf
        let mut output = Vec::new();

        // Write a simple header
        output.extend_from_slice(b"MVT");
        output.push(self.layers.len() as u8);

        for layer in &self.layers {
            // Write layer name length and name
            output.push(layer.name.len() as u8);
            output.extend_from_slice(layer.name.as_bytes());

            // Write feature count
            let feature_count = layer.features.len() as u32;
            output.extend_from_slice(&feature_count.to_le_bytes());

            // Simplified encoding of features
            for feature in &layer.features {
                encode_feature(&mut output, feature, &self.coord.bounds(), layer.extent)?;
            }
        }

        Ok(output)
    }

    /// Encode and compress with gzip
    pub fn encode_compressed(&self) -> RenderResult<Vec<u8>> {
        let data = self.encode()?;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&data)
            .map_err(|e| RenderError::CompressionError(e.to_string()))?;
        encoder
            .finish()
            .map_err(|e| RenderError::CompressionError(e.to_string()))
    }
}

/// Encode a feature to bytes
fn encode_feature(
    output: &mut Vec<u8>,
    feature: &Feature,
    bounds: &TileBounds,
    extent: u32,
) -> RenderResult<()> {
    // Write feature ID
    let has_id = feature.id.is_some();
    output.push(if has_id { 1 } else { 0 });
    if let Some(id) = feature.id {
        output.extend_from_slice(&id.to_le_bytes());
    }

    // Write geometry type
    let geom_type = match &feature.geometry {
        Geometry::Point(_) | Geometry::MultiPoint(_) => 1u8,
        Geometry::LineString(_) | Geometry::MultiLineString(_) => 2u8,
        Geometry::Polygon(_) | Geometry::MultiPolygon(_) => 3u8,
        _ => 0u8,
    };
    output.push(geom_type);

    // Encode geometry
    encode_geometry(output, &feature.geometry, bounds, extent)?;

    // Write property count
    output.push(feature.properties.len() as u8);

    // Encode properties (simplified)
    for (key, value) in &feature.properties {
        output.push(key.len() as u8);
        output.extend_from_slice(key.as_bytes());
        encode_value(output, value);
    }

    Ok(())
}

/// Encode a geometry to MVT coordinates
fn encode_geometry(
    output: &mut Vec<u8>,
    geometry: &Geometry<f64>,
    bounds: &TileBounds,
    extent: u32,
) -> RenderResult<()> {
    match geometry {
        Geometry::Point(point) => {
            let (x, y) = project_point(point, bounds, extent);
            output.extend_from_slice(&x.to_le_bytes());
            output.extend_from_slice(&y.to_le_bytes());
        }
        Geometry::MultiPoint(multi_point) => {
            output.push(multi_point.0.len() as u8);
            for point in &multi_point.0 {
                let (x, y) = project_point(point, bounds, extent);
                output.extend_from_slice(&x.to_le_bytes());
                output.extend_from_slice(&y.to_le_bytes());
            }
        }
        Geometry::LineString(line) => {
            encode_linestring(output, line, bounds, extent);
        }
        Geometry::MultiLineString(multi_line) => {
            output.push(multi_line.0.len() as u8);
            for line in &multi_line.0 {
                encode_linestring(output, line, bounds, extent);
            }
        }
        Geometry::Polygon(polygon) => {
            encode_polygon(output, polygon, bounds, extent);
        }
        Geometry::MultiPolygon(multi_polygon) => {
            output.push(multi_polygon.0.len() as u8);
            for polygon in &multi_polygon.0 {
                encode_polygon(output, polygon, bounds, extent);
            }
        }
        _ => {}
    }

    Ok(())
}

/// Project a point to tile coordinates
fn project_point(point: &Point<f64>, bounds: &TileBounds, extent: u32) -> (u32, u32) {
    let x = ((point.x() - bounds.min_x) / bounds.width() * f64::from(extent)) as u32;
    let y = ((bounds.max_y - point.y()) / bounds.height() * f64::from(extent)) as u32;
    (x.min(extent), y.min(extent))
}

/// Encode a linestring
fn encode_linestring(
    output: &mut Vec<u8>,
    line: &LineString<f64>,
    bounds: &TileBounds,
    extent: u32,
) {
    output.push(line.0.len() as u8);
    for coord in &line.0 {
        let x = ((coord.x - bounds.min_x) / bounds.width() * f64::from(extent)) as u32;
        let y = ((bounds.max_y - coord.y) / bounds.height() * f64::from(extent)) as u32;
        output.extend_from_slice(&x.min(extent).to_le_bytes());
        output.extend_from_slice(&y.min(extent).to_le_bytes());
    }
}

/// Encode a polygon
fn encode_polygon(
    output: &mut Vec<u8>,
    polygon: &Polygon<f64>,
    bounds: &TileBounds,
    extent: u32,
) {
    // Exterior ring
    encode_linestring(output, polygon.exterior(), bounds, extent);

    // Interior rings (holes)
    output.push(polygon.interiors().len() as u8);
    for interior in polygon.interiors() {
        encode_linestring(output, interior, bounds, extent);
    }
}

/// Encode a property value
fn encode_value(output: &mut Vec<u8>, value: &Value) {
    match value {
        Value::String(s) => {
            output.push(1); // String type
            let bytes = s.as_bytes();
            output.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
            output.extend_from_slice(bytes);
        }
        Value::Int(i) | Value::SInt(i) => {
            output.push(2); // Int type
            output.extend_from_slice(&i.to_le_bytes());
        }
        Value::UInt(u) => {
            output.push(3); // UInt type
            output.extend_from_slice(&u.to_le_bytes());
        }
        Value::Double(d) | Value::Float(d) => {
            output.push(4); // Double type
            output.extend_from_slice(&d.to_le_bytes());
        }
        Value::Bool(b) => {
            output.push(5); // Bool type
            output.push(if *b { 1 } else { 0 });
        }
    }
}

/// Clip geometry to tile bounds
pub fn clip_geometry(geometry: &Geometry<f64>, bounds: &TileBounds) -> Option<Geometry<f64>> {
    // Simplified clipping - check if geometry intersects bounds
    match geometry {
        Geometry::Point(point) => {
            if bounds.contains(point.x(), point.y()) {
                Some(geometry.clone())
            } else {
                None
            }
        }
        _ => {
            // For complex geometries, we'd need proper clipping algorithms
            // For now, just return the geometry as-is
            Some(geometry.clone())
        }
    }
}

/// Simplify geometry for a given zoom level
pub fn simplify_geometry(geometry: &Geometry<f64>, _zoom: u8, _tolerance: f64) -> Geometry<f64> {
    // Simplified version - in production, use Douglas-Peucker or similar
    // For now, just return the original geometry
    geometry.clone()
}

/// MVT encoder builder
pub struct MvtEncoder {
    tile: VectorTile,
    simplify: bool,
    clip: bool,
    tolerance: f64,
}

impl MvtEncoder {
    /// Create a new encoder for a tile
    pub fn new(coord: TileCoord) -> Self {
        MvtEncoder {
            tile: VectorTile::new(coord),
            simplify: true,
            clip: true,
            tolerance: 1.0,
        }
    }

    /// Enable or disable simplification
    pub fn with_simplify(mut self, simplify: bool) -> Self {
        self.simplify = simplify;
        self
    }

    /// Enable or disable clipping
    pub fn with_clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Set simplification tolerance
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Add a layer with features
    pub fn add_layer(&mut self, mut layer: Layer) -> RenderResult<()> {
        let bounds = self.tile.coord.bounds();

        // Process features
        if self.clip || self.simplify {
            let mut processed_features = Vec::new();

            for feature in layer.features {
                let mut geometry = feature.geometry;

                // Clip to tile bounds
                if self.clip {
                    if let Some(clipped) = clip_geometry(&geometry, &bounds) {
                        geometry = clipped;
                    } else {
                        continue; // Skip features outside tile
                    }
                }

                // Simplify geometry
                if self.simplify {
                    geometry = simplify_geometry(&geometry, self.tile.coord.z, self.tolerance);
                }

                processed_features.push(Feature {
                    id: feature.id,
                    geometry,
                    properties: feature.properties,
                });
            }

            layer.features = processed_features;
        }

        self.tile.add_layer(layer);
        Ok(())
    }

    /// Build the vector tile
    pub fn build(self) -> VectorTile {
        self.tile
    }

    /// Encode and return bytes
    pub fn encode(self) -> RenderResult<Vec<u8>> {
        self.tile.encode()
    }

    /// Encode with compression
    pub fn encode_compressed(self) -> RenderResult<Vec<u8>> {
        self.tile.encode_compressed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_creation() {
        let point = Point::new(0.0, 0.0);
        let feature = Feature::new(Geometry::Point(point))
            .with_id(1)
            .with_property("name".to_string(), Value::String("Test".to_string()));

        assert_eq!(feature.id, Some(1));
        assert_eq!(feature.properties.len(), 1);
    }

    #[test]
    fn test_layer_creation() {
        let mut layer = Layer::new("test".to_string());
        let point = Point::new(0.0, 0.0);
        let feature = Feature::new(Geometry::Point(point));

        layer.add_feature(feature);

        assert_eq!(layer.name, "test");
        assert_eq!(layer.feature_count(), 1);
    }

    #[test]
    fn test_vector_tile() {
        let coord = TileCoord::new(10, 512, 384).unwrap();
        let mut tile = VectorTile::new(coord);

        let mut layer = Layer::new("poi".to_string());
        let point = Point::new(0.0, 0.0);
        let feature = Feature::new(Geometry::Point(point));
        layer.add_feature(feature);

        tile.add_layer(layer);

        assert_eq!(tile.layers.len(), 1);
        assert!(tile.get_layer("poi").is_some());
    }

    #[test]
    fn test_mvt_encoder() {
        let coord = TileCoord::new(10, 512, 384).unwrap();
        let mut encoder = MvtEncoder::new(coord);

        let mut layer = Layer::new("test".to_string());
        let point = Point::new(0.0, 0.0);
        let feature = Feature::new(Geometry::Point(point));
        layer.add_feature(feature);

        encoder.add_layer(layer).unwrap();
        let tile = encoder.build();

        assert_eq!(tile.layers.len(), 1);
    }

    #[test]
    fn test_value_conversions() {
        let v1: Value = "test".into();
        assert!(matches!(v1, Value::String(_)));

        let v2: Value = 42i64.into();
        assert!(matches!(v2, Value::Int(42)));

        let v3: Value = 3.14.into();
        assert!(matches!(v3, Value::Double(_)));

        let v4: Value = true.into();
        assert!(matches!(v4, Value::Bool(true)));
    }
}
