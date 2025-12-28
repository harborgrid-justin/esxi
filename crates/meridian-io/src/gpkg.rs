//! GeoPackage (SQLite-based) support

use crate::error::{IoError, Result};
use crate::traits::{Feature, FeatureCollection, FeatureStream, Metadata, Reader};
use futures::stream;
use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// GeoPackage reader
pub struct GeoPackageReader;

impl GeoPackageReader {
    /// Create a new GeoPackage reader
    pub fn new() -> Self {
        Self
    }

    /// List all feature layers in the GeoPackage
    pub fn list_layers(&self, path: &Path) -> Result<Vec<String>> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        let mut stmt = conn.prepare(
            "SELECT table_name FROM gpkg_contents WHERE data_type = 'features'"
        )?;

        let layers = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;

        Ok(layers)
    }

    /// Read a specific layer from the GeoPackage
    pub fn read_layer(&self, path: &Path, layer_name: &str) -> Result<FeatureCollection> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        // Get geometry column name
        let geom_column = self.get_geometry_column(&conn, layer_name)?;

        // Get all column names
        let columns = self.get_columns(&conn, layer_name)?;

        // Build query
        let query = format!("SELECT * FROM \"{}\"", layer_name);

        let mut stmt = conn.prepare(&query)?;
        let column_count = stmt.column_count();

        let mut features = Vec::new();

        let rows = stmt.query_map([], |row| {
            let mut properties = HashMap::new();
            let mut geometry = None;

            for i in 0..column_count {
                let col_name = columns.get(i).map(|s| s.as_str()).unwrap_or("");

                if col_name == geom_column {
                    // Parse geometry (stored as WKB in GeoPackage)
                    if let Ok(wkb_data) = row.get::<_, Vec<u8>>(i) {
                        geometry = Self::parse_gpkg_geometry(&wkb_data).ok();
                    }
                } else {
                    // Store property
                    let value = Self::get_column_value(row, i);
                    if col_name != "fid" && col_name != "id" {
                        properties.insert(col_name.to_string(), value);
                    }
                }
            }

            Ok(Feature {
                id: None,
                geometry,
                properties,
                crs: None,
            })
        })?;

        for row in rows {
            features.push(row?);
        }

        // Get CRS information
        let crs = self.get_layer_crs(&conn, layer_name)?;

        Ok(FeatureCollection {
            features,
            crs,
            bbox: None,
        })
    }

    /// Get geometry column name for a layer
    fn get_geometry_column(&self, conn: &Connection, layer_name: &str) -> Result<String> {
        let mut stmt = conn.prepare(
            "SELECT column_name FROM gpkg_geometry_columns WHERE table_name = ?"
        )?;

        let geom_column = stmt.query_row([layer_name], |row| row.get::<_, String>(0))
            .map_err(|_| IoError::GeoPackage(format!("No geometry column found for layer: {}", layer_name)))?;

        Ok(geom_column)
    }

    /// Get all column names for a table
    fn get_columns(&self, conn: &Connection, table_name: &str) -> Result<Vec<String>> {
        let query = format!("PRAGMA table_info(\"{}\")", table_name);
        let mut stmt = conn.prepare(&query)?;

        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))? // Column 1 is the name
            .collect::<rusqlite::Result<Vec<String>>>()?;

        Ok(columns)
    }

    /// Get CRS for a layer
    fn get_layer_crs(&self, conn: &Connection, layer_name: &str) -> Result<Option<String>> {
        let mut stmt = conn.prepare(
            "SELECT srs_id FROM gpkg_geometry_columns WHERE table_name = ?"
        )?;

        if let Ok(srs_id) = stmt.query_row([layer_name], |row| row.get::<_, i32>(0)) {
            // Convert SRS ID to EPSG code
            if srs_id > 0 {
                return Ok(Some(format!("EPSG:{}", srs_id)));
            }
        }

        Ok(None)
    }

    /// Parse GeoPackage geometry (WKB with GeoPackage header)
    fn parse_gpkg_geometry(data: &[u8]) -> Result<geo_types::Geometry<f64>> {
        if data.len() < 8 {
            return Err(IoError::GeoPackage("Invalid GeoPackage geometry: too short".to_string()));
        }

        // GeoPackage Binary Format has a header:
        // Bytes 0-1: Magic number (0x47, 0x50) = "GP"
        // Byte 2: Version
        // Byte 3: Flags
        // Bytes 4-7: SRS ID
        // Then WKB data follows

        // Verify magic number
        if data[0] != 0x47 || data[1] != 0x50 {
            return Err(IoError::GeoPackage("Invalid GeoPackage geometry magic number".to_string()));
        }

        // Extract flags to determine envelope size
        let flags = data[3];
        let envelope_type = (flags >> 1) & 0x07;

        // Calculate envelope size
        let envelope_size = match envelope_type {
            0 => 0,   // No envelope
            1 => 32,  // XY
            2 => 48,  // XYZ
            3 => 48,  // XYM
            4 => 64,  // XYZM
            _ => return Err(IoError::GeoPackage("Invalid envelope type".to_string())),
        };

        // WKB data starts after header (8 bytes) + envelope
        let wkb_offset = 8 + envelope_size;

        if data.len() < wkb_offset {
            return Err(IoError::GeoPackage("Invalid GeoPackage geometry: header overflow".to_string()));
        }

        let _wkb_data = &data[wkb_offset..];

        // Parse WKB using wkt crate (it also supports WKB)
        // For now, we'll use a simplified approach
        // TODO: Implement proper WKB parsing
        Err(IoError::GeoPackage("WKB parsing not yet implemented".to_string()))
    }

    /// Get value from SQLite row
    fn get_column_value(row: &rusqlite::Row, index: usize) -> Value {
        use rusqlite::types::ValueRef;

        match row.get_ref(index).ok() {
            Some(ValueRef::Null) => Value::Null,
            Some(ValueRef::Integer(i)) => Value::from(i),
            Some(ValueRef::Real(f)) => Value::from(f),
            Some(ValueRef::Text(s)) => {
                let text = String::from_utf8_lossy(s).to_string();
                Value::String(text)
            }
            Some(ValueRef::Blob(_)) => Value::Null, // Skip blobs
            None => Value::Null,
        }
    }
}

impl Default for GeoPackageReader {
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for GeoPackageReader {
    fn read(&self, path: &Path) -> Result<FeatureCollection> {
        // Read the first layer by default
        let layers = self.list_layers(path)?;

        if layers.is_empty() {
            return Err(IoError::GeoPackage("No feature layers found".to_string()));
        }

        self.read_layer(path, &layers[0])
    }

    fn read_stream(&self, path: &Path) -> Result<FeatureStream> {
        let collection = self.read(path)?;
        let stream = stream::iter(collection.features.into_iter().map(Ok));
        Ok(Box::pin(stream))
    }

    fn read_crs(&self, path: &Path) -> Result<Option<String>> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let layers = self.list_layers(path)?;

        if layers.is_empty() {
            return Ok(None);
        }

        self.get_layer_crs(&conn, &layers[0])
    }

    fn read_metadata(&self, path: &Path) -> Result<Metadata> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let layers = self.list_layers(path)?;

        let mut metadata = Metadata::new();
        metadata.layers = layers.clone();

        // Get feature counts for each layer
        for layer in &layers {
            let query = format!("SELECT COUNT(*) FROM \"{}\"", layer);
            if let Ok(count) = conn.query_row(&query, [], |row| row.get::<_, usize>(0)) {
                metadata.feature_counts.insert(layer.clone(), count);
            }

            // Get CRS for layer
            if metadata.crs.is_none() {
                metadata.crs = self.get_layer_crs(&conn, layer)?;
            }

            // Get geometry type
            if let Ok(_geom_col) = self.get_geometry_column(&conn, layer) {
                let query = format!(
                    "SELECT geometry_type_name FROM gpkg_geometry_columns WHERE table_name = ?",
                );
                if let Ok(geom_type) = conn.query_row(&query, [layer], |row| row.get::<_, String>(0)) {
                    if !metadata.geometry_types.contains(&geom_type) {
                        metadata.geometry_types.push(geom_type);
                    }
                }
            }
        }

        Ok(metadata)
    }
}

/// GeoPackage writer
pub struct GeoPackageWriter;

impl GeoPackageWriter {
    /// Create a new GeoPackage writer
    pub fn new() -> Self {
        Self
    }
}

impl Default for GeoPackageWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geopackage_reader_creation() {
        let _reader = GeoPackageReader::new();
    }

    #[test]
    fn test_geopackage_writer_creation() {
        let _writer = GeoPackageWriter::new();
    }
}
