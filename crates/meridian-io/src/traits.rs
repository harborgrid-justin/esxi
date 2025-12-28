//! I/O traits for reading and writing geospatial data

use crate::error::Result;
use futures::Stream;
use geo_types::Geometry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::pin::Pin;

/// Represents a geospatial feature with geometry and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// Feature ID (optional)
    pub id: Option<String>,

    /// Geometry
    pub geometry: Option<Geometry<f64>>,

    /// Properties (attributes)
    pub properties: HashMap<String, serde_json::Value>,

    /// Coordinate reference system (EPSG code or WKT)
    pub crs: Option<String>,
}

impl Feature {
    /// Create a new feature
    pub fn new(geometry: Option<Geometry<f64>>) -> Self {
        Self {
            id: None,
            geometry,
            properties: HashMap::new(),
            crs: None,
        }
    }

    /// Create a feature with ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Add a property
    pub fn with_property(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    /// Set CRS
    pub fn with_crs(mut self, crs: impl Into<String>) -> Self {
        self.crs = Some(crs.into());
        self
    }

    /// Get a property value
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// Get a string property
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.properties.get(key)?.as_str()
    }

    /// Get a numeric property
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.properties.get(key)?.as_f64()
    }

    /// Get an integer property
    pub fn get_integer(&self, key: &str) -> Option<i64> {
        self.properties.get(key)?.as_i64()
    }

    /// Get a boolean property
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.properties.get(key)?.as_bool()
    }
}

/// Feature collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCollection {
    /// Features in the collection
    pub features: Vec<Feature>,

    /// Coordinate reference system
    pub crs: Option<String>,

    /// Bounding box [min_x, min_y, max_x, max_y]
    pub bbox: Option<Vec<f64>>,
}

impl FeatureCollection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
            crs: None,
            bbox: None,
        }
    }

    /// Create a collection with features
    pub fn from_features(features: Vec<Feature>) -> Self {
        Self {
            features,
            crs: None,
            bbox: None,
        }
    }

    /// Add a feature
    pub fn add_feature(&mut self, feature: Feature) {
        self.features.push(feature);
    }

    /// Get number of features
    pub fn len(&self) -> usize {
        self.features.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }
}

impl Default for FeatureCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for feature stream
pub type FeatureStream = Pin<Box<dyn Stream<Item = Result<Feature>> + Send>>;

/// Trait for reading geospatial data
pub trait Reader: Send + Sync {
    /// Read all features from a file
    fn read(&self, path: &Path) -> Result<FeatureCollection>;

    /// Read features as a stream (for large files)
    fn read_stream(&self, path: &Path) -> Result<FeatureStream>;

    /// Get the CRS from the file
    fn read_crs(&self, path: &Path) -> Result<Option<String>>;

    /// Get metadata (layer names, feature count, etc.)
    fn read_metadata(&self, path: &Path) -> Result<Metadata>;
}

/// Trait for writing geospatial data
pub trait Writer: Send + Sync {
    /// Write features to a file
    fn write(&self, path: &Path, collection: &FeatureCollection) -> Result<()>;

    /// Write features as a stream (for large datasets)
    fn write_stream<S>(&self, path: &Path, stream: S) -> Result<()>
    where
        S: Stream<Item = Result<Feature>> + Send + 'static;

    /// Append features to an existing file
    fn append(&self, path: &Path, collection: &FeatureCollection) -> Result<()>;
}

/// Trait for streaming large files
pub trait StreamingReader: Send + Sync {
    /// Read features in chunks
    fn read_chunked(&self, path: &Path, chunk_size: usize) -> Result<FeatureStream>;

    /// Read features with a filter
    fn read_filtered<F>(&self, path: &Path, filter: F) -> Result<FeatureStream>
    where
        F: Fn(&Feature) -> bool + Send + Sync + 'static;
}

/// Metadata about a geospatial file or layer
#[derive(Debug, Clone)]
pub struct Metadata {
    /// Layer name(s)
    pub layers: Vec<String>,

    /// Feature count (per layer)
    pub feature_counts: HashMap<String, usize>,

    /// Coordinate reference system
    pub crs: Option<String>,

    /// Bounding box [min_x, min_y, max_x, max_y]
    pub bbox: Option<Vec<f64>>,

    /// Geometry type(s)
    pub geometry_types: Vec<String>,

    /// Attribute schema (field names and types)
    pub schema: HashMap<String, String>,

    /// Additional metadata
    pub additional: HashMap<String, String>,
}

impl Metadata {
    /// Create new metadata
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            feature_counts: HashMap::new(),
            crs: None,
            bbox: None,
            geometry_types: Vec::new(),
            schema: HashMap::new(),
            additional: HashMap::new(),
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Geospatial file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// GeoJSON (.geojson, .json)
    GeoJson,

    /// Shapefile (.shp)
    Shapefile,

    /// KML (.kml)
    Kml,

    /// KMZ (.kmz)
    Kmz,

    /// GeoTIFF (.tif, .tiff)
    GeoTiff,

    /// GeoPackage (.gpkg)
    GeoPackage,

    /// WKT text file
    Wkt,

    /// WKB binary file
    Wkb,

    /// CSV with coordinates
    Csv,

    /// GML (.gml)
    Gml,
}

impl Format {
    /// Get format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "geojson" | "json" => Some(Format::GeoJson),
            "shp" => Some(Format::Shapefile),
            "kml" => Some(Format::Kml),
            "kmz" => Some(Format::Kmz),
            "tif" | "tiff" | "geotiff" => Some(Format::GeoTiff),
            "gpkg" => Some(Format::GeoPackage),
            "wkt" => Some(Format::Wkt),
            "wkb" => Some(Format::Wkb),
            "csv" => Some(Format::Csv),
            "gml" => Some(Format::Gml),
            _ => None,
        }
    }

    /// Get common file extensions for this format
    pub fn extensions(&self) -> &[&str] {
        match self {
            Format::GeoJson => &["geojson", "json"],
            Format::Shapefile => &["shp"],
            Format::Kml => &["kml"],
            Format::Kmz => &["kmz"],
            Format::GeoTiff => &["tif", "tiff"],
            Format::GeoPackage => &["gpkg"],
            Format::Wkt => &["wkt"],
            Format::Wkb => &["wkb"],
            Format::Csv => &["csv"],
            Format::Gml => &["gml"],
        }
    }

    /// Get MIME type for this format
    pub fn mime_type(&self) -> &str {
        match self {
            Format::GeoJson => "application/geo+json",
            Format::Shapefile => "application/x-shapefile",
            Format::Kml => "application/vnd.google-earth.kml+xml",
            Format::Kmz => "application/vnd.google-earth.kmz",
            Format::GeoTiff => "image/tiff",
            Format::GeoPackage => "application/geopackage+sqlite3",
            Format::Wkt => "text/plain",
            Format::Wkb => "application/octet-stream",
            Format::Csv => "text/csv",
            Format::Gml => "application/gml+xml",
        }
    }
}
