//! Error types for the Meridian Core library.
//!
//! This module provides comprehensive error handling for all spatial operations,
//! coordinate transformations, and data validation operations in the Meridian GIS platform.

use thiserror::Error;

/// The main error type for Meridian Core operations.
///
/// This enum encompasses all possible errors that can occur during geometry processing,
/// coordinate transformations, spatial indexing, and feature manipulation.
#[derive(Error, Debug)]
pub enum MeridianError {
    /// Error during coordinate reference system transformation
    #[error("CRS transformation error: {0}")]
    TransformError(String),

    /// Invalid coordinate reference system specification
    #[error("Invalid CRS: {0}")]
    InvalidCrs(String),

    /// Error during projection operations
    #[error("Projection error: {0}")]
    ProjectionError(String),

    /// Invalid geometry encountered
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    /// Bounding box error
    #[error("Bounding box error: {0}")]
    BoundingBoxError(String),

    /// Spatial index error
    #[error("Spatial index error: {0}")]
    SpatialIndexError(String),

    /// Feature property error
    #[error("Feature property error: {0}")]
    PropertyError(String),

    /// Layer operation error
    #[error("Layer error: {0}")]
    LayerError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Input/Output error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Generic error with custom message
    #[error("{0}")]
    Custom(String),
}

/// A specialized Result type for Meridian Core operations.
///
/// This type alias simplifies error handling throughout the library by providing
/// a consistent return type for fallible operations.
pub type Result<T> = std::result::Result<T, MeridianError>;

impl From<proj::ProjError> for MeridianError {
    fn from(err: proj::ProjError) -> Self {
        MeridianError::ProjectionError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = MeridianError::InvalidGeometry("Point cannot be empty".to_string());
        assert_eq!(err.to_string(), "Invalid geometry: Point cannot be empty");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: MeridianError = io_err.into();
        assert!(matches!(err, MeridianError::IoError(_)));
    }
}
