//! CSV file support with coordinate columns

use crate::error::{IoError, Result};
use crate::traits::{Feature, FeatureCollection, FeatureStream, Metadata, Reader, Writer};
use crate::wkt::WktReader;
use futures::stream;
use geo_types::{Geometry, Point};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// CSV coordinate column detection configuration
#[derive(Debug, Clone)]
pub struct CsvConfig {
    /// X/Longitude column name (auto-detected if None)
    pub x_column: Option<String>,

    /// Y/Latitude column name (auto-detected if None)
    pub y_column: Option<String>,

    /// WKT geometry column name (auto-detected if None)
    pub wkt_column: Option<String>,

    /// CSV delimiter
    pub delimiter: u8,

    /// Has header row
    pub has_headers: bool,

    /// Skip empty lines
    pub skip_empty: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        Self {
            x_column: None,
            y_column: None,
            wkt_column: None,
            delimiter: b',',
            has_headers: true,
            skip_empty: true,
        }
    }
}

impl CsvConfig {
    /// Create a new CSV config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set X column name
    pub fn with_x_column(mut self, name: impl Into<String>) -> Self {
        self.x_column = Some(name.into());
        self
    }

    /// Set Y column name
    pub fn with_y_column(mut self, name: impl Into<String>) -> Self {
        self.y_column = Some(name.into());
        self
    }

    /// Set WKT column name
    pub fn with_wkt_column(mut self, name: impl Into<String>) -> Self {
        self.wkt_column = Some(name.into());
        self
    }

    /// Set delimiter
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Set tab delimiter
    pub fn with_tab_delimiter(mut self) -> Self {
        self.delimiter = b'\t';
        self
    }
}

/// CSV reader with spatial support
pub struct CsvReader {
    config: CsvConfig,
}

impl CsvReader {
    /// Create a new CSV reader with default config
    pub fn new() -> Self {
        Self {
            config: CsvConfig::default(),
        }
    }

    /// Create a CSV reader with custom config
    pub fn with_config(config: CsvConfig) -> Self {
        Self { config }
    }

    /// Auto-detect coordinate columns from headers
    fn detect_coordinate_columns(headers: &csv::StringRecord) -> (Option<usize>, Option<usize>, Option<usize>) {
        let mut x_col = None;
        let mut y_col = None;
        let mut wkt_col = None;

        for (i, header) in headers.iter().enumerate() {
            let header_lower = header.to_lowercase();

            // Check for WKT column
            if header_lower == "wkt" || header_lower == "geometry" || header_lower == "geom" {
                wkt_col = Some(i);
            }

            // Check for X/Longitude columns
            if x_col.is_none() && (
                header_lower == "x" || header_lower == "lon" || header_lower == "longitude" ||
                header_lower == "long" || header_lower == "lng"
            ) {
                x_col = Some(i);
            }

            // Check for Y/Latitude columns
            if y_col.is_none() && (
                header_lower == "y" || header_lower == "lat" || header_lower == "latitude"
            ) {
                y_col = Some(i);
            }
        }

        (x_col, y_col, wkt_col)
    }

    /// Parse a CSV row into a Feature
    fn parse_record(
        record: &csv::StringRecord,
        headers: &csv::StringRecord,
        x_col: Option<usize>,
        y_col: Option<usize>,
        wkt_col: Option<usize>,
    ) -> Result<Feature> {
        let mut properties = HashMap::new();

        // Extract properties
        for (i, (header, value)) in headers.iter().zip(record.iter()).enumerate() {
            // Skip coordinate columns in properties
            if Some(i) == x_col || Some(i) == y_col || Some(i) == wkt_col {
                continue;
            }

            // Try to parse as number, otherwise store as string
            let json_value = if let Ok(num) = value.parse::<f64>() {
                Value::from(num)
            } else if let Ok(int) = value.parse::<i64>() {
                Value::from(int)
            } else if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
                Value::from(value.eq_ignore_ascii_case("true"))
            } else if value.is_empty() {
                Value::Null
            } else {
                Value::String(value.to_string())
            };

            properties.insert(header.to_string(), json_value);
        }

        // Create geometry
        let geometry = if let Some(wkt_idx) = wkt_col {
            // Parse WKT geometry
            if let Some(wkt_str) = record.get(wkt_idx) {
                if !wkt_str.is_empty() {
                    Some(WktReader::parse_wkt(wkt_str)?)
                } else {
                    None
                }
            } else {
                None
            }
        } else if let (Some(x_idx), Some(y_idx)) = (x_col, y_col) {
            // Create point from X/Y coordinates
            if let (Some(x_str), Some(y_str)) = (record.get(x_idx), record.get(y_idx)) {
                let x = x_str.parse::<f64>()
                    .map_err(|_| IoError::Csv(format!("Invalid X coordinate: {}", x_str)))?;
                let y = y_str.parse::<f64>()
                    .map_err(|_| IoError::Csv(format!("Invalid Y coordinate: {}", y_str)))?;

                Some(Geometry::Point(Point::new(x, y)))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Feature {
            id: None,
            geometry,
            properties,
            crs: None,
        })
    }
}

impl Default for CsvReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for CsvReader {
    fn read(&self, path: &Path) -> Result<FeatureCollection> {
        let file = File::open(path)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(self.config.delimiter)
            .has_headers(self.config.has_headers)
            .from_reader(file);

        let headers = rdr.headers()?.clone();

        // Detect or use configured coordinate columns
        let (x_col, y_col, wkt_col) = if let Some(ref wkt) = self.config.wkt_column {
            let wkt_idx = headers.iter().position(|h| h == wkt);
            (None, None, wkt_idx)
        } else if let (Some(ref x), Some(ref y)) = (&self.config.x_column, &self.config.y_column) {
            let x_idx = headers.iter().position(|h| h == x);
            let y_idx = headers.iter().position(|h| h == y);
            (x_idx, y_idx, None)
        } else {
            Self::detect_coordinate_columns(&headers)
        };

        let mut features = Vec::new();

        for result in rdr.records() {
            let record = result?;

            if self.config.skip_empty && record.iter().all(|f| f.is_empty()) {
                continue;
            }

            match Self::parse_record(&record, &headers, x_col, y_col, wkt_col) {
                Ok(feature) => features.push(feature),
                Err(e) => {
                    eprintln!("Warning: Failed to parse CSV record: {}", e);
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
        // CSV files don't typically include CRS information
        Ok(None)
    }

    fn read_metadata(&self, path: &Path) -> Result<Metadata> {
        let collection = self.read(path)?;

        let mut metadata = Metadata::new();
        metadata.layers.push("default".to_string());
        metadata.feature_counts.insert("default".to_string(), collection.features.len());

        // Build schema from first feature
        if let Some(first) = collection.features.first() {
            for (key, value) in &first.properties {
                let type_name = match value {
                    Value::String(_) => "String",
                    Value::Number(_) => "Number",
                    Value::Bool(_) => "Boolean",
                    _ => "Unknown",
                };
                metadata.schema.insert(key.clone(), type_name.to_string());
            }
        }

        // Determine geometry types
        let mut geom_types = std::collections::HashSet::new();
        for feature in &collection.features {
            if let Some(ref geom) = feature.geometry {
                geom_types.insert(match geom {
                    Geometry::Point(_) => "Point".to_string(),
                    _ => "Other".to_string(),
                });
            }
        }
        metadata.geometry_types = geom_types.into_iter().collect();

        Ok(metadata)
    }
}

/// CSV writer with spatial support
pub struct CsvWriter {
    config: CsvConfig,
    /// Write geometry as WKT
    pub write_wkt: bool,
    /// Write geometry as separate X/Y columns
    pub write_xy: bool,
}

impl CsvWriter {
    /// Create a new CSV writer
    pub fn new() -> Self {
        Self {
            config: CsvConfig::default(),
            write_wkt: true,
            write_xy: false,
        }
    }

    /// Enable WKT output
    pub fn with_wkt(mut self) -> Self {
        self.write_wkt = true;
        self.write_xy = false;
        self
    }

    /// Enable X/Y column output
    pub fn with_xy(mut self) -> Self {
        self.write_xy = true;
        self.write_wkt = false;
        self
    }

    /// Set delimiter
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.config.delimiter = delimiter;
        self
    }
}

impl Default for CsvWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for CsvWriter {
    fn write(&self, path: &Path, collection: &FeatureCollection) -> Result<()> {
        let file = File::create(path)?;
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(self.config.delimiter)
            .from_writer(BufWriter::new(file));

        if collection.features.is_empty() {
            return Ok(());
        }

        // Build headers
        let mut headers = Vec::new();

        if self.write_wkt {
            headers.push("wkt".to_string());
        } else if self.write_xy {
            headers.push("x".to_string());
            headers.push("y".to_string());
        }

        // Add property headers from first feature
        if let Some(first) = collection.features.first() {
            for key in first.properties.keys() {
                headers.push(key.clone());
            }
        }

        wtr.write_record(&headers)?;

        // Write features
        for feature in &collection.features {
            let mut record = Vec::new();

            // Write geometry
            if self.write_wkt {
                if let Some(ref geom) = feature.geometry {
                    let wkt = crate::wkt::WktWriter::geometry_to_wkt(geom)?;
                    record.push(wkt);
                } else {
                    record.push(String::new());
                }
            } else if self.write_xy {
                if let Some(Geometry::Point(pt)) = feature.geometry {
                    record.push(pt.x().to_string());
                    record.push(pt.y().to_string());
                } else {
                    record.push(String::new());
                    record.push(String::new());
                }
            }

            // Write properties
            for header in headers.iter().skip(if self.write_wkt { 1 } else if self.write_xy { 2 } else { 0 }) {
                let value = feature.properties.get(header)
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        Value::Null => String::new(),
                        _ => v.to_string(),
                    })
                    .unwrap_or_default();
                record.push(value);
            }

            wtr.write_record(&record)?;
        }

        wtr.flush()?;
        Ok(())
    }

    fn write_stream<S>(&self, _path: &Path, _stream: S) -> Result<()>
    where
        S: futures::Stream<Item = Result<Feature>> + Send + 'static,
    {
        Err(IoError::UnsupportedFormat("CSV streaming write not yet implemented".to_string()))
    }

    fn append(&self, _path: &Path, _collection: &FeatureCollection) -> Result<()> {
        Err(IoError::UnsupportedFormat("CSV append not yet implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_config() {
        let config = CsvConfig::new()
            .with_x_column("lon")
            .with_y_column("lat");

        assert_eq!(config.x_column, Some("lon".to_string()));
        assert_eq!(config.y_column, Some("lat".to_string()));
    }
}
