//! Shapefile reading and writing support

use crate::error::{IoError, Result};
use crate::traits::{Feature, FeatureCollection, FeatureStream, Metadata, Reader, Writer};
use futures::{stream, Stream, StreamExt};
use geo_types::Geometry;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Shapefile reader
pub struct ShapefileReader;

impl ShapefileReader {
    /// Create a new shapefile reader
    pub fn new() -> Self {
        Self
    }

    /// Convert shapefile shape to geo_types::Geometry
    fn convert_shape(shape: shapefile::Shape) -> Result<Option<Geometry<f64>>> {
        use shapefile::Shape;

        match shape {
            Shape::Point(p) => {
                Ok(Some(Geometry::Point(geo_types::Point::new(p.x, p.y))))
            }
            Shape::PointM(p) => {
                Ok(Some(Geometry::Point(geo_types::Point::new(p.x, p.y))))
            }
            Shape::PointZ(p) => {
                Ok(Some(Geometry::Point(geo_types::Point::new(p.x, p.y))))
            }
            Shape::Polyline(pl) => {
                if pl.parts().len() == 1 {
                    // Single LineString
                    let coords: Vec<_> = pl.points()
                        .iter()
                        .map(|p| geo_types::Coord { x: p.x, y: p.y })
                        .collect();
                    Ok(Some(Geometry::LineString(geo_types::LineString::from(coords))))
                } else {
                    // MultiLineString
                    let mut lines = Vec::new();
                    for part in pl.parts() {
                        let coords: Vec<_> = part
                            .iter()
                            .map(|p| geo_types::Coord { x: p.x, y: p.y })
                            .collect();
                        lines.push(geo_types::LineString::from(coords));
                    }
                    Ok(Some(Geometry::MultiLineString(geo_types::MultiLineString::from(lines))))
                }
            }
            Shape::PolylineM(pl) => {
                if pl.parts().len() == 1 {
                    let coords: Vec<_> = pl.points()
                        .iter()
                        .map(|p| geo_types::Coord { x: p.x, y: p.y })
                        .collect();
                    Ok(Some(Geometry::LineString(geo_types::LineString::from(coords))))
                } else {
                    let mut lines = Vec::new();
                    for part in pl.parts() {
                        let coords: Vec<_> = part
                            .iter()
                            .map(|p| geo_types::Coord { x: p.x, y: p.y })
                            .collect();
                        lines.push(geo_types::LineString::from(coords));
                    }
                    Ok(Some(Geometry::MultiLineString(geo_types::MultiLineString::from(lines))))
                }
            }
            Shape::PolylineZ(pl) => {
                if pl.parts().len() == 1 {
                    let coords: Vec<_> = pl.points()
                        .iter()
                        .map(|p| geo_types::Coord { x: p.x, y: p.y })
                        .collect();
                    Ok(Some(Geometry::LineString(geo_types::LineString::from(coords))))
                } else {
                    let mut lines = Vec::new();
                    for part in pl.parts() {
                        let coords: Vec<_> = part
                            .iter()
                            .map(|p| geo_types::Coord { x: p.x, y: p.y })
                            .collect();
                        lines.push(geo_types::LineString::from(coords));
                    }
                    Ok(Some(Geometry::MultiLineString(geo_types::MultiLineString::from(lines))))
                }
            }
            Shape::Polygon(pg) => {
                if pg.parts().len() == 1 {
                    // Single Polygon
                    let exterior: Vec<_> = pg.parts()[0]
                        .iter()
                        .map(|p| geo_types::Coord { x: p.x, y: p.y })
                        .collect();
                    let polygon = geo_types::Polygon::new(
                        geo_types::LineString::from(exterior),
                        vec![],
                    );
                    Ok(Some(Geometry::Polygon(polygon)))
                } else {
                    // MultiPolygon or Polygon with holes
                    // Note: Shapefile polygon parts can be exterior rings or holes
                    // This is a simplified conversion
                    let mut polygons = Vec::new();
                    for part in pg.parts() {
                        let coords: Vec<_> = part
                            .iter()
                            .map(|p| geo_types::Coord { x: p.x, y: p.y })
                            .collect();
                        let polygon = geo_types::Polygon::new(
                            geo_types::LineString::from(coords),
                            vec![],
                        );
                        polygons.push(polygon);
                    }
                    Ok(Some(Geometry::MultiPolygon(geo_types::MultiPolygon::from(polygons))))
                }
            }
            Shape::PolygonM(pg) => {
                let mut polygons = Vec::new();
                for part in pg.parts() {
                    let coords: Vec<_> = part
                        .iter()
                        .map(|p| geo_types::Coord { x: p.x, y: p.y })
                        .collect();
                    let polygon = geo_types::Polygon::new(
                        geo_types::LineString::from(coords),
                        vec![],
                    );
                    polygons.push(polygon);
                }
                if polygons.len() == 1 {
                    Ok(Some(Geometry::Polygon(polygons.into_iter().next().unwrap())))
                } else {
                    Ok(Some(Geometry::MultiPolygon(geo_types::MultiPolygon::from(polygons))))
                }
            }
            Shape::PolygonZ(pg) => {
                let mut polygons = Vec::new();
                for part in pg.parts() {
                    let coords: Vec<_> = part
                        .iter()
                        .map(|p| geo_types::Coord { x: p.x, y: p.y })
                        .collect();
                    let polygon = geo_types::Polygon::new(
                        geo_types::LineString::from(coords),
                        vec![],
                    );
                    polygons.push(polygon);
                }
                if polygons.len() == 1 {
                    Ok(Some(Geometry::Polygon(polygons.into_iter().next().unwrap())))
                } else {
                    Ok(Some(Geometry::MultiPolygon(geo_types::MultiPolygon::from(polygons))))
                }
            }
            Shape::Multipoint(mp) => {
                let points: Vec<_> = mp.points()
                    .iter()
                    .map(|p| geo_types::Point::new(p.x, p.y))
                    .collect();
                Ok(Some(Geometry::MultiPoint(geo_types::MultiPoint::from(points))))
            }
            Shape::MultipointM(mp) => {
                let points: Vec<_> = mp.points()
                    .iter()
                    .map(|p| geo_types::Point::new(p.x, p.y))
                    .collect();
                Ok(Some(Geometry::MultiPoint(geo_types::MultiPoint::from(points))))
            }
            Shape::MultipointZ(mp) => {
                let points: Vec<_> = mp.points()
                    .iter()
                    .map(|p| geo_types::Point::new(p.x, p.y))
                    .collect();
                Ok(Some(Geometry::MultiPoint(geo_types::MultiPoint::from(points))))
            }
            Shape::NullShape => Ok(None),
            _ => Err(IoError::InvalidGeometry("Unsupported shapefile geometry type".to_string())),
        }
    }

    /// Convert dbase field value to JSON value
    fn convert_field_value(value: &shapefile::dbase::FieldValue) -> Value {
        use shapefile::dbase::FieldValue;

        match value {
            FieldValue::Character(s) => s.as_ref()
                .map(|s| Value::String(s.clone()))
                .unwrap_or(Value::Null),
            FieldValue::Numeric(n) => n.map(Value::from).unwrap_or(Value::Null),
            FieldValue::Logical(b) => b.map(Value::from).unwrap_or(Value::Null),
            FieldValue::Date(d) => d.as_ref()
                .map(|d| Value::String(d.to_string()))
                .unwrap_or(Value::Null),
            FieldValue::Float(f) => f.map(Value::from).unwrap_or(Value::Null),
            FieldValue::Integer(i) => Value::from(*i),
            FieldValue::Double(d) => Value::from(*d),
            FieldValue::DateTime(_) => Value::Null, // TODO: proper datetime conversion
            FieldValue::Currency(_) => Value::Null,
            FieldValue::Memo(_) => Value::Null,
        }
    }

    /// Read projection file (.prj)
    fn read_prj(shp_path: &Path) -> Result<Option<String>> {
        let prj_path = shp_path.with_extension("prj");
        if prj_path.exists() {
            let file = File::open(prj_path)?;
            let reader = BufReader::new(file);
            let mut wkt = String::new();
            for line in reader.lines() {
                wkt.push_str(&line?);
            }
            Ok(Some(wkt.trim().to_string()))
        } else {
            Ok(None)
        }
    }
}

impl Default for ShapefileReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for ShapefileReader {
    fn read(&self, path: &Path) -> Result<FeatureCollection> {
        let mut reader = shapefile::Reader::from_path(path)?;

        let crs = Self::read_prj(path)?;
        let mut features = Vec::new();

        for result in reader.iter_shapes_and_records() {
            let (shape, record) = result?;

            let geometry = Self::convert_shape(shape)?;

            let mut properties = HashMap::new();
            for (name, value) in record {
                properties.insert(name, Self::convert_field_value(&value));
            }

            features.push(Feature {
                id: None,
                geometry,
                properties,
                crs: crs.clone(),
            });
        }

        Ok(FeatureCollection {
            features,
            crs,
            bbox: None,
        })
    }

    fn read_stream(&self, path: &Path) -> Result<FeatureStream> {
        let collection = self.read(path)?;
        let stream = stream::iter(collection.features.into_iter().map(Ok));
        Ok(Box::pin(stream))
    }

    fn read_crs(&self, path: &Path) -> Result<Option<String>> {
        Self::read_prj(path)
    }

    fn read_metadata(&self, path: &Path) -> Result<Metadata> {
        let reader = shapefile::Reader::from_path(path)?;
        let header = reader.header();

        let mut metadata = Metadata::new();
        metadata.layers.push("default".to_string());
        metadata.crs = Self::read_prj(path)?;

        // Bounding box from header
        metadata.bbox = Some(vec![
            header.bbox.min.x,
            header.bbox.min.y,
            header.bbox.max.x,
            header.bbox.max.y,
        ]);

        // Geometry type
        let geom_type = format!("{:?}", header.shape_type);
        metadata.geometry_types.push(geom_type);

        // Feature count (requires reading all records)
        let count = shapefile::Reader::from_path(path)?.iter_shapes().count();
        metadata.feature_counts.insert("default".to_string(), count);

        Ok(metadata)
    }
}

/// Shapefile writer
pub struct ShapefileWriter;

impl ShapefileWriter {
    /// Create a new shapefile writer
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShapefileWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for ShapefileWriter {
    fn write(&self, _path: &Path, _collection: &FeatureCollection) -> Result<()> {
        // TODO: Implement shapefile writing
        // This requires determining shapefile type, creating .shp, .shx, .dbf files
        Err(IoError::UnsupportedFormat("Shapefile writing not yet implemented".to_string()))
    }

    fn write_stream<S>(&self, _path: &Path, _stream: S) -> Result<()>
    where
        S: Stream<Item = Result<Feature>> + Send + 'static,
    {
        Err(IoError::UnsupportedFormat("Shapefile streaming write not yet implemented".to_string()))
    }

    fn append(&self, _path: &Path, _collection: &FeatureCollection) -> Result<()> {
        Err(IoError::UnsupportedFormat("Shapefile append not yet implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shapefile_reader_creation() {
        let _reader = ShapefileReader::new();
    }
}
