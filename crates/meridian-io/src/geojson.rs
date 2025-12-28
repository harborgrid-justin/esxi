//! GeoJSON reading and writing support

use crate::error::{IoError, Result};
use crate::traits::{Feature, FeatureCollection, FeatureStream, Metadata, Reader, Writer};
use futures::{stream, Stream, StreamExt};
use geo_types::Geometry;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write as _};
use std::path::Path;

/// GeoJSON reader
pub struct GeoJsonReader;

impl GeoJsonReader {
    /// Create a new GeoJSON reader
    pub fn new() -> Self {
        Self
    }

    /// Convert geojson::Feature to our Feature type
    fn convert_feature(gj_feature: geojson::Feature) -> Result<Feature> {
        let id = gj_feature.id.map(|id| match id {
            geojson::feature::Id::String(s) => s,
            geojson::feature::Id::Number(n) => n.to_string(),
        });

        let geometry = if let Some(gj_geom) = gj_feature.geometry {
            Some(Self::convert_geometry(&gj_geom)?)
        } else {
            None
        };

        let properties = gj_feature.properties
            .unwrap_or_default()
            .into_iter()
            .collect();

        Ok(Feature {
            id,
            geometry,
            properties,
            crs: None,
        })
    }

    /// Convert geojson::Geometry to geo_types::Geometry
    fn convert_geometry(gj_geom: &geojson::Geometry) -> Result<Geometry<f64>> {
        let geom: Geometry<f64> = gj_geom.try_into()
            .map_err(|e: geojson::Error| IoError::InvalidGeometry(e.to_string()))?;

        Ok(geom)
    }

    /// Parse CRS from GeoJSON
    fn parse_crs(gj_crs: &Option<geojson::Crs>) -> Option<String> {
        gj_crs.as_ref().and_then(|crs| {
            if let geojson::Crs::Named { name } = crs {
                Some(name.clone())
            } else {
                None
            }
        })
    }
}

impl Default for GeoJsonReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for GeoJsonReader {
    fn read(&self, path: &Path) -> Result<FeatureCollection> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let gj: geojson::GeoJson = serde_json::from_reader(reader)?;

        match gj {
            geojson::GeoJson::FeatureCollection(fc) => {
                let features: Result<Vec<Feature>> = fc.features
                    .into_iter()
                    .map(Self::convert_feature)
                    .collect();

                let crs = Self::parse_crs(&fc.foreign_members
                    .and_then(|fm| fm.get("crs")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())));

                let bbox = fc.bbox.map(|b| b.iter().copied().collect());

                Ok(FeatureCollection {
                    features: features?,
                    crs,
                    bbox,
                })
            }
            geojson::GeoJson::Feature(f) => {
                let feature = Self::convert_feature(f)?;
                Ok(FeatureCollection::from_features(vec![feature]))
            }
            geojson::GeoJson::Geometry(g) => {
                let geometry = Self::convert_geometry(&g)?;
                let feature = Feature::new(Some(geometry));
                Ok(FeatureCollection::from_features(vec![feature]))
            }
        }
    }

    fn read_stream(&self, path: &Path) -> Result<FeatureStream> {
        // For streaming, we need to parse line-by-line GeoJSON (newline-delimited)
        // or load the whole file for standard GeoJSON
        let collection = self.read(path)?;
        let stream = stream::iter(collection.features.into_iter().map(Ok));
        Ok(Box::pin(stream))
    }

    fn read_crs(&self, path: &Path) -> Result<Option<String>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let gj: geojson::GeoJson = serde_json::from_reader(reader)?;

        match gj {
            geojson::GeoJson::FeatureCollection(fc) => {
                Ok(Self::parse_crs(&fc.foreign_members
                    .and_then(|fm| fm.get("crs")
                        .and_then(|v| serde_json::from_value(v.clone()).ok()))))
            }
            _ => Ok(None),
        }
    }

    fn read_metadata(&self, path: &Path) -> Result<Metadata> {
        let collection = self.read(path)?;

        let mut metadata = Metadata::new();
        metadata.layers.push("default".to_string());
        metadata.feature_counts.insert("default".to_string(), collection.features.len());
        metadata.crs = collection.crs;
        metadata.bbox = collection.bbox;

        // Determine geometry types
        let mut geom_types = std::collections::HashSet::new();
        for feature in &collection.features {
            if let Some(ref geom) = feature.geometry {
                geom_types.insert(geometry_type_name(geom));
            }
        }
        metadata.geometry_types = geom_types.into_iter().collect();

        // Build schema from first feature
        if let Some(first) = collection.features.first() {
            for (key, value) in &first.properties {
                let type_name = match value {
                    Value::String(_) => "String",
                    Value::Number(_) => "Number",
                    Value::Bool(_) => "Boolean",
                    Value::Array(_) => "Array",
                    Value::Object(_) => "Object",
                    Value::Null => "Null",
                };
                metadata.schema.insert(key.clone(), type_name.to_string());
            }
        }

        Ok(metadata)
    }
}

/// GeoJSON writer
pub struct GeoJsonWriter {
    /// Pretty print JSON
    pub pretty: bool,
}

impl GeoJsonWriter {
    /// Create a new GeoJSON writer
    pub fn new() -> Self {
        Self { pretty: false }
    }

    /// Enable pretty printing
    pub fn with_pretty(mut self) -> Self {
        self.pretty = true;
        self
    }

    /// Convert our Feature to geojson::Feature
    fn convert_feature(feature: &Feature) -> Result<geojson::Feature> {
        let id = feature.id.as_ref().map(|s| geojson::feature::Id::String(s.clone()));

        let geometry = if let Some(ref geom) = feature.geometry {
            Some(Self::convert_geometry(geom)?)
        } else {
            None
        };

        let properties = Some(feature.properties.clone());

        Ok(geojson::Feature {
            bbox: None,
            geometry,
            id,
            properties,
            foreign_members: None,
        })
    }

    /// Convert geo_types::Geometry to geojson::Geometry
    fn convert_geometry(geom: &Geometry<f64>) -> Result<geojson::Geometry> {
        let gj_value = match geom {
            Geometry::Point(p) => geojson::Value::from(p),
            Geometry::Line(l) => geojson::Value::from(l),
            Geometry::LineString(ls) => geojson::Value::from(ls),
            Geometry::Polygon(p) => geojson::Value::from(p),
            Geometry::MultiPoint(mp) => geojson::Value::from(mp),
            Geometry::MultiLineString(mls) => geojson::Value::from(mls),
            Geometry::MultiPolygon(mp) => geojson::Value::from(mp),
            Geometry::GeometryCollection(gc) => {
                let geometries: Result<Vec<geojson::Geometry>> = gc.iter()
                    .map(Self::convert_geometry)
                    .collect();
                geojson::Value::GeometryCollection(geometries?)
            }
            _ => return Err(IoError::InvalidGeometry("Unsupported geometry type".to_string())),
        };

        Ok(geojson::Geometry::new(gj_value))
    }
}

impl Default for GeoJsonWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for GeoJsonWriter {
    fn write(&self, path: &Path, collection: &FeatureCollection) -> Result<()> {
        let features: Result<Vec<geojson::Feature>> = collection.features
            .iter()
            .map(Self::convert_feature)
            .collect();

        let gj_fc = geojson::FeatureCollection {
            bbox: collection.bbox.clone(),
            features: features?,
            foreign_members: None,
        };

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        if self.pretty {
            serde_json::to_writer_pretty(&mut writer, &gj_fc)?;
        } else {
            serde_json::to_writer(&mut writer, &gj_fc)?;
        }

        writer.flush()?;
        Ok(())
    }

    fn write_stream<S>(&self, path: &Path, mut stream: S) -> Result<()>
    where
        S: Stream<Item = Result<Feature>> + Send + 'static,
    {
        // For streaming, write newline-delimited GeoJSON
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Use tokio runtime for async stream processing
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| IoError::Other(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            while let Some(result) = stream.next().await {
                let feature = result?;
                let gj_feature = Self::convert_feature(&feature)?;
                let json = serde_json::to_string(&gj_feature)?;
                writeln!(writer, "{}", json)?;
            }
            Ok::<_, IoError>(())
        })?;

        writer.flush()?;
        Ok(())
    }

    fn append(&self, path: &Path, collection: &FeatureCollection) -> Result<()> {
        // Read existing features
        let reader = GeoJsonReader::new();
        let mut existing = reader.read(path)?;

        // Append new features
        existing.features.extend(collection.features.iter().cloned());

        // Write back
        self.write(path, &existing)
    }
}

/// Get geometry type name
fn geometry_type_name(geom: &Geometry<f64>) -> String {
    match geom {
        Geometry::Point(_) => "Point",
        Geometry::Line(_) => "Line",
        Geometry::LineString(_) => "LineString",
        Geometry::Polygon(_) => "Polygon",
        Geometry::MultiPoint(_) => "MultiPoint",
        Geometry::MultiLineString(_) => "MultiLineString",
        Geometry::MultiPolygon(_) => "MultiPolygon",
        Geometry::GeometryCollection(_) => "GeometryCollection",
        _ => "Unknown",
    }.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geometry_type_name() {
        use geo_types::Point;
        let point = Geometry::Point(Point::new(1.0, 2.0));
        assert_eq!(geometry_type_name(&point), "Point");
    }
}
