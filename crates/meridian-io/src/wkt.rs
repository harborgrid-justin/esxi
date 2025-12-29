//! WKT (Well-Known Text) and WKB (Well-Known Binary) support

use crate::error::{IoError, Result};
use crate::traits::{Feature, FeatureCollection, FeatureStream, Metadata, Reader, Writer};
use futures::stream;
use geo_types::Geometry;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write as _};
use std::path::Path;
use wkt::Wkt;

/// WKT reader
pub struct WktReader;

impl WktReader {
    /// Create a new WKT reader
    pub fn new() -> Self {
        Self
    }

    /// Parse WKT string to geometry
    pub fn parse_wkt(wkt_str: &str) -> Result<Geometry<f64>> {
        let wkt: Wkt<f64> = wkt_str.parse()
            .map_err(|e| IoError::Wkt(format!("WKT parse error: {:?}", e)))?;

        // Convert WKT to geo_types
        let geom: Geometry<f64> = wkt.try_into()
            .map_err(|e| IoError::InvalidGeometry(format!("WKT conversion error: {:?}", e)))?;

        Ok(geom)
    }

    /// Parse WKT from a line (may include properties)
    fn parse_line(line: &str) -> Result<Feature> {
        let line = line.trim();

        // Check if line contains tabs or commas (might be CSV-like with WKT)
        if line.contains('\t') || (line.contains(',') && !line.starts_with("POINT") &&
           !line.starts_with("LINESTRING") && !line.starts_with("POLYGON") &&
           !line.starts_with("MULTIPOINT") && !line.starts_with("MULTILINESTRING") &&
           !line.starts_with("MULTIPOLYGON")) {
            // Parse as CSV-like format
            Self::parse_wkt_csv_line(line)
        } else {
            // Parse as pure WKT
            let geometry = Self::parse_wkt(line)?;
            Ok(Feature::new(Some(geometry)))
        }
    }

    /// Parse CSV line with WKT geometry column
    fn parse_wkt_csv_line(line: &str) -> Result<Feature> {
        // Simple CSV parsing (assumes WKT is in first column)
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.is_empty() {
            return Err(IoError::Wkt("Empty line".to_string()));
        }

        let geometry = Self::parse_wkt(parts[0])?;
        let mut properties = HashMap::new();

        // Additional columns as properties
        for (i, part) in parts.iter().enumerate().skip(1) {
            properties.insert(
                format!("column_{}", i),
                serde_json::Value::String(part.to_string()),
            );
        }

        Ok(Feature {
            id: None,
            geometry: Some(geometry),
            properties,
            crs: None,
        })
    }
}

impl Default for WktReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for WktReader {
    fn read(&self, path: &Path) -> Result<FeatureCollection> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut features = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue; // Skip empty lines and comments
            }

            match Self::parse_line(line) {
                Ok(feature) => features.push(feature),
                Err(e) => {
                    // Log error but continue parsing
                    eprintln!("Warning: Failed to parse line: {}", e);
                }
            }
        }

        Ok(FeatureCollection::from_features(features))
    }

    fn read_stream(&self, path: &Path) -> Result<FeatureStream> {
        let collection = self.read(path)?;
        let stream = stream::iter(collection.features.into_iter().map(Ok));
        Ok(Box::pin(stream))
    }

    fn read_crs(&self, _path: &Path) -> Result<Option<String>> {
        // WKT files don't typically include CRS information
        Ok(None)
    }

    fn read_metadata(&self, path: &Path) -> Result<Metadata> {
        let collection = self.read(path)?;

        let mut metadata = Metadata::new();
        metadata.layers.push("default".to_string());
        metadata.feature_counts.insert("default".to_string(), collection.features.len());

        // Determine geometry types
        let mut geom_types = std::collections::HashSet::new();
        for feature in &collection.features {
            if let Some(ref geom) = feature.geometry {
                geom_types.insert(geometry_type_name(geom));
            }
        }
        metadata.geometry_types = geom_types.into_iter().collect();

        Ok(metadata)
    }
}

/// WKT writer
pub struct WktWriter {
    /// Include properties in output
    pub include_properties: bool,
}

impl WktWriter {
    /// Create a new WKT writer
    pub fn new() -> Self {
        Self {
            include_properties: false,
        }
    }

    /// Include properties in output (tab-separated)
    pub fn with_properties(mut self) -> Self {
        self.include_properties = true;
        self
    }

    /// Convert geometry to WKT string
    pub fn geometry_to_wkt(geom: &Geometry<f64>) -> Result<String> {
        // Use the wkt crate's conversion from geo_types
        use wkt::ToWkt;
        Ok(geom.wkt_string())
    }
}

impl Default for WktWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for WktWriter {
    fn write(&self, path: &Path, collection: &FeatureCollection) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        for feature in &collection.features {
            if let Some(ref geom) = feature.geometry {
                let wkt = Self::geometry_to_wkt(geom)?;

                if self.include_properties && !feature.properties.is_empty() {
                    // Write WKT + properties
                    write!(writer, "{}", wkt)?;
                    for (_, value) in &feature.properties {
                        write!(writer, "\t{}", value)?;
                    }
                    writeln!(writer)?;
                } else {
                    // Write pure WKT
                    writeln!(writer, "{}", wkt)?;
                }
            }
        }

        writer.flush()?;
        Ok(())
    }

    fn write_stream<S>(&self, path: &Path, stream: S) -> Result<()>
    where
        S: futures::Stream<Item = Result<Feature>> + Send + 'static,
    {
        use futures::StreamExt;

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| IoError::Other(format!("Failed to create runtime: {}", e)))?;

        rt.block_on(async {
            let mut stream = Box::pin(stream);
            while let Some(result) = stream.next().await {
                let feature = result?;
                if let Some(ref geom) = feature.geometry {
                    let wkt = Self::geometry_to_wkt(geom)?;
                    writeln!(writer, "{}", wkt)?;
                }
            }
            Ok::<_, IoError>(())
        })?;

        writer.flush()?;
        Ok(())
    }

    fn append(&self, path: &Path, collection: &FeatureCollection) -> Result<()> {
        use std::fs::OpenOptions;

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        let mut writer = BufWriter::new(file);

        for feature in &collection.features {
            if let Some(ref geom) = feature.geometry {
                let wkt = Self::geometry_to_wkt(geom)?;
                writeln!(writer, "{}", wkt)?;
            }
        }

        writer.flush()?;
        Ok(())
    }
}

/// WKB (Well-Known Binary) support
pub struct WkbReader;

impl WkbReader {
    /// Create a new WKB reader
    pub fn new() -> Self {
        Self
    }
}

impl Default for WkbReader {
    fn default() -> Self {
        Self::new()
    }
}

/// WKB writer
pub struct WkbWriter;

impl WkbWriter {
    /// Create a new WKB writer
    pub fn new() -> Self {
        Self
    }
}

impl Default for WkbWriter {
    fn default() -> Self {
        Self::new()
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
    fn test_parse_wkt_point() {
        let result = WktReader::parse_wkt("POINT(1 2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_wkt_linestring() {
        let result = WktReader::parse_wkt("LINESTRING(0 0, 1 1, 2 2)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_geometry_to_wkt() {
        use geo_types::Point;
        let point = Geometry::Point(Point::new(1.0, 2.0));
        let wkt = WktWriter::geometry_to_wkt(&point).unwrap();
        assert!(wkt.contains("POINT"));
    }
}
