//! Error types for the I/O layer

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Result type for I/O operations
pub type Result<T> = std::result::Result<T, IoError>;

/// Errors that can occur during format I/O operations
#[derive(Debug, Error)]
pub enum IoError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Shapefile error
    #[error("Shapefile error: {0}")]
    Shapefile(String),

    /// GeoJSON error
    #[error("GeoJSON error: {0}")]
    GeoJson(String),

    /// KML parsing error
    #[error("KML parsing error: {0}")]
    Kml(String),

    /// GeoTIFF error
    #[error("GeoTIFF error: {0}")]
    GeoTiff(String),

    /// GeoPackage error
    #[error("GeoPackage error: {0}")]
    GeoPackage(String),

    /// WKT/WKB error
    #[error("WKT/WKB error: {0}")]
    Wkt(String),

    /// CSV parsing error
    #[error("CSV error: {0}")]
    Csv(String),

    /// Format detection error
    #[error("Could not detect format for file: {0}")]
    UnknownFormat(PathBuf),

    /// Unsupported format
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid geometry
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    /// Coordinate reference system error
    #[error("CRS error: {0}")]
    Crs(String),

    /// Encoding error
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Compression/decompression error
    #[error("Compression error: {0}")]
    Compression(String),

    /// Database error (for GeoPackage)
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// XML parsing error
    #[error("XML parsing error: {0}")]
    Xml(String),

    /// Feature parsing error
    #[error("Feature parsing error: {0}")]
    Feature(String),

    /// Attribute error
    #[error("Attribute error: {0}")]
    Attribute(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<shapefile::Error> for IoError {
    fn from(err: shapefile::Error) -> Self {
        IoError::Shapefile(err.to_string())
    }
}

impl From<geojson::Error> for IoError {
    fn from(err: geojson::Error) -> Self {
        IoError::GeoJson(err.to_string())
    }
}

impl From<quick_xml::Error> for IoError {
    fn from(err: quick_xml::Error) -> Self {
        IoError::Xml(err.to_string())
    }
}

impl From<tiff::TiffError> for IoError {
    fn from(err: tiff::TiffError) -> Self {
        IoError::GeoTiff(err.to_string())
    }
}

impl From<csv::Error> for IoError {
    fn from(err: csv::Error) -> Self {
        IoError::Csv(err.to_string())
    }
}

impl From<zip::result::ZipError> for IoError {
    fn from(err: zip::result::ZipError) -> Self {
        IoError::Compression(err.to_string())
    }
}

impl From<wkt::WktError> for IoError {
    fn from(err: wkt::WktError) -> Self {
        IoError::Wkt(err.to_string())
    }
}

impl From<serde_json::Error> for IoError {
    fn from(err: serde_json::Error) -> Self {
        IoError::GeoJson(err.to_string())
    }
}
