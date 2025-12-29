//! Mapbox Vector Tile (MVT) encoding
//!
//! Implements the MVT 2.0 specification:
//! https://github.com/mapbox/vector-tile-spec/tree/master/2.1

use crate::error::{Error, Result};
use crate::tile::extent::TileExtent;
use bytes::{BufMut, BytesMut};
use geo_types::{Coord, Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
use prost::Message;
use std::collections::HashMap;

/// MVT tile data structure
#[derive(Debug, Clone)]
pub struct MvtTile {
    /// Tile layers
    pub layers: Vec<MvtLayer>,
}

/// MVT layer
#[derive(Debug, Clone)]
pub struct MvtLayer {
    /// Layer name
    pub name: String,
    /// Layer extent
    pub extent: u32,
    /// Layer version (default: 2)
    pub version: u32,
    /// Features in this layer
    pub features: Vec<MvtFeature>,
}

/// MVT feature
#[derive(Debug, Clone)]
pub struct MvtFeature {
    /// Feature ID (optional)
    pub id: Option<u64>,
    /// Feature geometry
    pub geometry: Geometry,
    /// Feature properties
    pub properties: HashMap<String, MvtValue>,
}

/// MVT value types
#[derive(Debug, Clone, PartialEq)]
pub enum MvtValue {
    String(String),
    Float(f32),
    Double(f64),
    Int(i64),
    UInt(u64),
    SInt(i64),
    Bool(bool),
}

/// MVT encoder
pub struct MvtEncoder {
    extent: TileExtent,
}

impl MvtEncoder {
    /// Create a new MVT encoder
    pub fn new() -> Self {
        Self {
            extent: TileExtent::default_mvt(),
        }
    }

    /// Create with custom extent
    pub fn with_extent(extent: TileExtent) -> Self {
        Self { extent }
    }

    /// Encode a tile to MVT format
    pub fn encode(&self, tile: &MvtTile) -> Result<Vec<u8>> {
        let mut proto_tile = proto::Tile::default();

        for layer in &tile.layers {
            let proto_layer = self.encode_layer(layer)?;
            proto_tile.layers.push(proto_layer);
        }

        let mut buf = BytesMut::new();
        proto_tile
            .encode(&mut buf)
            .map_err(|e| Error::encoding(format!("Failed to encode MVT: {}", e)))?;

        Ok(buf.to_vec())
    }

    /// Encode a single layer
    fn encode_layer(&self, layer: &MvtLayer) -> Result<proto::tile::Layer> {
        let mut proto_layer = proto::tile::Layer {
            version: layer.version,
            name: layer.name.clone(),
            extent: Some(layer.extent),
            ..Default::default()
        };

        // Build key and value tables
        let mut keys = Vec::new();
        let mut values = Vec::new();
        let mut key_map: HashMap<String, u32> = HashMap::new();
        let mut value_map: HashMap<MvtValue, u32> = HashMap::new();

        // Collect all unique keys and values
        for feature in &layer.features {
            for (key, value) in &feature.properties {
                if !key_map.contains_key(key) {
                    key_map.insert(key.clone(), keys.len() as u32);
                    keys.push(key.clone());
                }

                if !value_map.contains_key(value) {
                    value_map.insert(value.clone(), values.len() as u32);
                    values.push(value.clone());
                }
            }
        }

        proto_layer.keys = keys;
        proto_layer.values = values
            .into_iter()
            .map(|v| self.encode_value(v))
            .collect();

        // Encode features
        for feature in &layer.features {
            let proto_feature = self.encode_feature(feature, &key_map, &value_map)?;
            proto_layer.features.push(proto_feature);
        }

        Ok(proto_layer)
    }

    /// Encode a single feature
    fn encode_feature(
        &self,
        feature: &MvtFeature,
        key_map: &HashMap<String, u32>,
        value_map: &HashMap<MvtValue, u32>,
    ) -> Result<proto::tile::Feature> {
        let mut proto_feature = proto::tile::Feature::default();

        if let Some(id) = feature.id {
            proto_feature.id = Some(id);
        }

        // Encode properties as tag pairs
        for (key, value) in &feature.properties {
            let key_idx = *key_map.get(key).unwrap();
            let value_idx = *value_map.get(value).unwrap();
            proto_feature.tags.push(key_idx);
            proto_feature.tags.push(value_idx);
        }

        // Encode geometry
        let (geom_type, geom_data) = self.encode_geometry(&feature.geometry)?;
        proto_feature.r#type = Some(geom_type as i32);
        proto_feature.geometry = geom_data;

        Ok(proto_feature)
    }

    /// Encode geometry to command integers
    fn encode_geometry(&self, geometry: &Geometry) -> Result<(proto::tile::GeomType, Vec<u32>)> {
        match geometry {
            Geometry::Point(point) => Ok((proto::tile::GeomType::Point, self.encode_point(point))),
            Geometry::MultiPoint(mp) => Ok((proto::tile::GeomType::Point, self.encode_multipoint(mp))),
            Geometry::LineString(ls) => Ok((proto::tile::GeomType::Linestring, self.encode_linestring(ls))),
            Geometry::MultiLineString(mls) => Ok((proto::tile::GeomType::Linestring, self.encode_multilinestring(mls))),
            Geometry::Polygon(poly) => Ok((proto::tile::GeomType::Polygon, self.encode_polygon(poly))),
            Geometry::MultiPolygon(mp) => Ok((proto::tile::GeomType::Polygon, self.encode_multipolygon(mp))),
            _ => Err(Error::geometry("Unsupported geometry type")),
        }
    }

    /// Encode a point
    fn encode_point(&self, point: &Point) -> Vec<u32> {
        let x = point.x().round() as i32;
        let y = point.y().round() as i32;
        vec![9, Self::zigzag(x), Self::zigzag(y)] // MoveTo(1)
    }

    /// Encode multipoint
    fn encode_multipoint(&self, mp: &MultiPoint) -> Vec<u32> {
        let mut geom = Vec::new();
        geom.push(Self::command(1, mp.0.len() as u32)); // MoveTo(count)

        for point in &mp.0 {
            geom.push(Self::zigzag(point.x().round() as i32));
            geom.push(Self::zigzag(point.y().round() as i32));
        }

        geom
    }

    /// Encode linestring
    fn encode_linestring(&self, ls: &LineString) -> Vec<u32> {
        if ls.0.is_empty() {
            return Vec::new();
        }

        let mut geom = Vec::new();
        let mut cursor = (0i32, 0i32);

        // MoveTo first point
        let first = &ls.0[0];
        let x = first.x.round() as i32;
        let y = first.y.round() as i32;
        geom.push(9); // MoveTo(1)
        geom.push(Self::zigzag(x - cursor.0));
        geom.push(Self::zigzag(y - cursor.1));
        cursor = (x, y);

        // LineTo remaining points
        if ls.0.len() > 1 {
            geom.push(Self::command(2, (ls.0.len() - 1) as u32));
            for coord in &ls.0[1..] {
                let x = coord.x.round() as i32;
                let y = coord.y.round() as i32;
                geom.push(Self::zigzag(x - cursor.0));
                geom.push(Self::zigzag(y - cursor.1));
                cursor = (x, y);
            }
        }

        geom
    }

    /// Encode multilinestring
    fn encode_multilinestring(&self, mls: &MultiLineString) -> Vec<u32> {
        let mut geom = Vec::new();
        for ls in &mls.0 {
            geom.extend(self.encode_linestring(ls));
        }
        geom
    }

    /// Encode polygon
    fn encode_polygon(&self, poly: &Polygon) -> Vec<u32> {
        let mut geom = Vec::new();

        // Encode exterior ring
        geom.extend(self.encode_ring(poly.exterior(), true));

        // Encode interior rings
        for interior in poly.interiors() {
            geom.extend(self.encode_ring(interior, false));
        }

        geom
    }

    /// Encode multipolygon
    fn encode_multipolygon(&self, mp: &MultiPolygon) -> Vec<u32> {
        let mut geom = Vec::new();
        for poly in &mp.0 {
            geom.extend(self.encode_polygon(poly));
        }
        geom
    }

    /// Encode a polygon ring
    fn encode_ring(&self, ring: &LineString, _is_exterior: bool) -> Vec<u32> {
        if ring.0.len() < 3 {
            return Vec::new();
        }

        let mut geom = Vec::new();
        let mut cursor = (0i32, 0i32);

        // MoveTo first point
        let first = &ring.0[0];
        let x = first.x.round() as i32;
        let y = first.y.round() as i32;
        geom.push(9); // MoveTo(1)
        geom.push(Self::zigzag(x - cursor.0));
        geom.push(Self::zigzag(y - cursor.1));
        cursor = (x, y);

        // LineTo remaining points (excluding closing point)
        let count = ring.0.len() - 1;
        geom.push(Self::command(2, count as u32));
        for coord in &ring.0[1..count] {
            let x = coord.x.round() as i32;
            let y = coord.y.round() as i32;
            geom.push(Self::zigzag(x - cursor.0));
            geom.push(Self::zigzag(y - cursor.1));
            cursor = (x, y);
        }

        // ClosePath
        geom.push(15); // ClosePath(1)

        geom
    }

    /// Encode a value
    fn encode_value(&self, value: MvtValue) -> proto::tile::Value {
        let mut v = proto::tile::Value::default();

        match value {
            MvtValue::String(s) => v.string_value = Some(s),
            MvtValue::Float(f) => v.float_value = Some(f),
            MvtValue::Double(d) => v.double_value = Some(d),
            MvtValue::Int(i) => v.int_value = Some(i),
            MvtValue::UInt(u) => v.uint_value = Some(u),
            MvtValue::SInt(s) => v.sint_value = Some(s),
            MvtValue::Bool(b) => v.bool_value = Some(b),
        }

        v
    }

    /// Create command integer
    fn command(id: u32, count: u32) -> u32 {
        (id & 0x7) | (count << 3)
    }

    /// ZigZag encode a signed integer
    fn zigzag(n: i32) -> u32 {
        ((n << 1) ^ (n >> 31)) as u32
    }
}

impl Default for MvtEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Protocol buffer definitions
mod proto {
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    pub struct Tile {
        #[prost(message, repeated, tag = "3")]
        pub layers: Vec<tile::Layer>,
    }

    pub mod tile {
        use prost::Message;

        #[derive(Clone, PartialEq, Message)]
        pub struct Layer {
            #[prost(uint32, required, tag = "15")]
            pub version: u32,
            #[prost(string, required, tag = "1")]
            pub name: String,
            #[prost(message, repeated, tag = "2")]
            pub features: Vec<Feature>,
            #[prost(string, repeated, tag = "3")]
            pub keys: Vec<String>,
            #[prost(message, repeated, tag = "4")]
            pub values: Vec<Value>,
            #[prost(uint32, optional, tag = "5")]
            pub extent: Option<u32>,
        }

        #[derive(Clone, PartialEq, Message)]
        pub struct Feature {
            #[prost(uint64, optional, tag = "1")]
            pub id: Option<u64>,
            #[prost(uint32, repeated, packed = "true", tag = "2")]
            pub tags: Vec<u32>,
            #[prost(enumeration = "GeomType", optional, tag = "3")]
            pub r#type: Option<i32>,
            #[prost(uint32, repeated, packed = "true", tag = "4")]
            pub geometry: Vec<u32>,
        }

        #[derive(Clone, PartialEq, Message)]
        pub struct Value {
            #[prost(string, optional, tag = "1")]
            pub string_value: Option<String>,
            #[prost(float, optional, tag = "2")]
            pub float_value: Option<f32>,
            #[prost(double, optional, tag = "3")]
            pub double_value: Option<f64>,
            #[prost(int64, optional, tag = "4")]
            pub int_value: Option<i64>,
            #[prost(uint64, optional, tag = "5")]
            pub uint_value: Option<u64>,
            #[prost(sint64, optional, tag = "6")]
            pub sint_value: Option<i64>,
            #[prost(bool, optional, tag = "7")]
            pub bool_value: Option<bool>,
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(i32)]
        pub enum GeomType {
            Unknown = 0,
            Point = 1,
            Linestring = 2,
            Polygon = 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag_encoding() {
        assert_eq!(MvtEncoder::zigzag(0), 0);
        assert_eq!(MvtEncoder::zigzag(-1), 1);
        assert_eq!(MvtEncoder::zigzag(1), 2);
        assert_eq!(MvtEncoder::zigzag(-2), 3);
    }

    #[test]
    fn test_command_encoding() {
        assert_eq!(MvtEncoder::command(1, 1), 9);
        assert_eq!(MvtEncoder::command(2, 3), 26);
        assert_eq!(MvtEncoder::command(7, 1), 15);
    }
}
