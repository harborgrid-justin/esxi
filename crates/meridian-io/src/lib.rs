//! Meridian I/O - Format import/export for the Meridian GIS Platform
//!
//! This crate provides comprehensive support for reading and writing various
//! geospatial file formats including:
//!
//! - **Vector formats**: Shapefile, GeoJSON, KML/KMZ, GeoPackage, WKT/WKB, CSV
//! - **Raster formats**: GeoTIFF
//!
//! # Features
//!
//! - Streaming support for large files
//! - Automatic format detection
//! - CRS/projection handling
//! - Async I/O support via Tokio
//! - Extensible reader/writer traits
//!
//! # Examples
//!
//! ## Reading a GeoJSON file
//!
//! ```no_run
//! use meridian_io::geojson::GeoJsonReader;
//! use meridian_io::traits::Reader;
//! use std::path::Path;
//!
//! let reader = GeoJsonReader::new();
//! let collection = reader.read(Path::new("data.geojson")).unwrap();
//! println!("Read {} features", collection.features.len());
//! ```
//!
//! ## Auto-detecting format
//!
//! ```no_run
//! use meridian_io::FormatRegistry;
//! use std::path::Path;
//!
//! let path = Path::new("data.shp");
//! let collection = FormatRegistry::read_auto(path).unwrap();
//! println!("Detected format and read {} features", collection.features.len());
//! ```
//!
//! ## Streaming large files
//!
//! ```no_run
//! use meridian_io::geojson::GeoJsonReader;
//! use meridian_io::traits::Reader;
//! use futures::StreamExt;
//! use std::path::Path;
//!
//! # async fn example() {
//! let reader = GeoJsonReader::new();
//! let mut stream = reader.read_stream(Path::new("large.geojson")).unwrap();
//!
//! while let Some(feature) = stream.next().await {
//!     let feature = feature.unwrap();
//!     // Process feature
//! }
//! # }
//! ```

pub mod csv;
pub mod detection;
pub mod error;
pub mod geojson;
pub mod geotiff;
pub mod gpkg;
pub mod kml;
pub mod shapefile;
pub mod traits;
pub mod wkt;

// Re-export commonly used types
pub use error::{IoError, Result};
pub use traits::{Feature, FeatureCollection, Format, Metadata, Reader, Writer};

use csv::{CsvReader, CsvWriter};
use detection::detect_format;
use geojson::{GeoJsonReader, GeoJsonWriter};
use gpkg::GeoPackageReader;
use kml::{KmlReader, KmlWriter, KmzReader};
use shapefile::{ShapefileReader, ShapefileWriter};
use std::path::Path;
use wkt::{WktReader, WktWriter};

/// Central registry for all supported formats
pub struct FormatRegistry;

impl FormatRegistry {
    /// Auto-detect format and read file
    pub fn read_auto(path: &Path) -> Result<FeatureCollection> {
        let format = detect_format(path)?;
        Self::read_with_format(path, format)
    }

    /// Read file with specified format
    pub fn read_with_format(path: &Path, format: Format) -> Result<FeatureCollection> {
        match format {
            Format::GeoJson => GeoJsonReader::new().read(path),
            Format::Shapefile => ShapefileReader::new().read(path),
            Format::Kml => KmlReader::new().read(path),
            Format::Kmz => KmzReader::new().read(path),
            Format::GeoPackage => GeoPackageReader::new().read(path),
            Format::Wkt => WktReader::new().read(path),
            Format::Csv => CsvReader::new().read(path),
            _ => Err(IoError::UnsupportedFormat(format!("{:?}", format))),
        }
    }

    /// Write file with specified format
    pub fn write_with_format(
        path: &Path,
        collection: &FeatureCollection,
        format: Format,
    ) -> Result<()> {
        match format {
            Format::GeoJson => GeoJsonWriter::new().write(path, collection),
            Format::Shapefile => ShapefileWriter::new().write(path, collection),
            Format::Kml => KmlWriter::new().write(path, collection),
            Format::Wkt => WktWriter::new().write(path, collection),
            Format::Csv => CsvWriter::new().write(path, collection),
            _ => Err(IoError::UnsupportedFormat(format!("{:?}", format))),
        }
    }

    /// Auto-detect format from extension and write
    pub fn write_auto(path: &Path, collection: &FeatureCollection) -> Result<()> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| IoError::UnknownFormat(path.to_path_buf()))?;

        let format = Format::from_extension(ext)
            .ok_or_else(|| IoError::UnknownFormat(path.to_path_buf()))?;

        Self::write_with_format(path, collection, format)
    }

    /// Get reader for format
    pub fn get_reader(format: Format) -> Box<dyn Reader> {
        match format {
            Format::GeoJson => Box::new(GeoJsonReader::new()),
            Format::Shapefile => Box::new(ShapefileReader::new()),
            Format::Kml => Box::new(KmlReader::new()),
            Format::Kmz => Box::new(KmzReader::new()),
            Format::GeoPackage => Box::new(GeoPackageReader::new()),
            Format::Wkt => Box::new(WktReader::new()),
            Format::Csv => Box::new(CsvReader::new()),
            _ => Box::new(GeoJsonReader::new()), // Default fallback
        }
    }

    /// Get writer for format
    /// Note: This function is commented out because Writer trait is not object-safe
    /// due to the generic write_stream method. Use concrete writer types instead.
    // pub fn get_writer(format: Format) -> Box<dyn Writer> {
    //     match format {
    //         Format::GeoJson => Box::new(GeoJsonWriter::new()),
    //         Format::Shapefile => Box::new(ShapefileWriter::new()),
    //         Format::Kml => Box::new(KmlWriter::new()),
    //         Format::Wkt => Box::new(WktWriter::new()),
    //         Format::Csv => Box::new(CsvWriter::new()),
    //         _ => Box::new(GeoJsonWriter::new()), // Default fallback
    //     }
    // }

    /// List all supported formats
    pub fn supported_formats() -> Vec<Format> {
        vec![
            Format::GeoJson,
            Format::Shapefile,
            Format::Kml,
            Format::Kmz,
            Format::GeoTiff,
            Format::GeoPackage,
            Format::Wkt,
            Format::Wkb,
            Format::Csv,
            Format::Gml,
        ]
    }

    /// Check if a format is supported for reading
    pub fn supports_reading(format: Format) -> bool {
        matches!(
            format,
            Format::GeoJson
                | Format::Shapefile
                | Format::Kml
                | Format::Kmz
                | Format::GeoPackage
                | Format::Wkt
                | Format::Csv
        )
    }

    /// Check if a format is supported for writing
    pub fn supports_writing(format: Format) -> bool {
        matches!(
            format,
            Format::GeoJson | Format::Kml | Format::Wkt | Format::Csv
        )
    }
}

/// Convenience function to read any supported format
pub fn read(path: &Path) -> Result<FeatureCollection> {
    FormatRegistry::read_auto(path)
}

/// Convenience function to write any supported format
pub fn write(path: &Path, collection: &FeatureCollection) -> Result<()> {
    FormatRegistry::write_auto(path, collection)
}

/// Convenience function to detect file format
pub fn detect(path: &Path) -> Result<Format> {
    detect_format(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_registry() {
        let formats = FormatRegistry::supported_formats();
        assert!(!formats.is_empty());
    }

    #[test]
    fn test_supports_reading() {
        assert!(FormatRegistry::supports_reading(Format::GeoJson));
        assert!(FormatRegistry::supports_reading(Format::Shapefile));
        assert!(FormatRegistry::supports_reading(Format::Kml));
    }

    #[test]
    fn test_supports_writing() {
        assert!(FormatRegistry::supports_writing(Format::GeoJson));
        assert!(FormatRegistry::supports_writing(Format::Kml));
        assert!(FormatRegistry::supports_writing(Format::Wkt));
    }

    #[test]
    fn test_format_from_extension() {
        assert_eq!(Format::from_extension("geojson"), Some(Format::GeoJson));
        assert_eq!(Format::from_extension("shp"), Some(Format::Shapefile));
        assert_eq!(Format::from_extension("kml"), Some(Format::Kml));
        assert_eq!(Format::from_extension("csv"), Some(Format::Csv));
    }
}
