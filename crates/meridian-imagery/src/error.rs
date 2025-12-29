//! Error types for imagery processing operations

use std::fmt;

/// Result type for imagery operations
pub type Result<T> = std::result::Result<T, ImageryError>;

/// Errors that can occur during imagery processing
#[derive(Debug, thiserror::Error)]
pub enum ImageryError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid image format
    #[error("Invalid image format: {0}")]
    InvalidFormat(String),

    /// Unsupported format
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Invalid band index
    #[error("Invalid band {band}, total bands: {total}")]
    InvalidBand {
        /// Requested band index
        band: usize,
        /// Total number of bands
        total: usize,
    },

    /// Invalid dimensions
    #[error("Invalid dimensions: {0}")]
    InvalidDimensions(String),

    /// Metadata error
    #[error("Metadata error: {0}")]
    Metadata(String),

    /// Projection error
    #[error("Projection error: {0}")]
    Projection(String),

    /// Processing error
    #[error("Processing error: {0}")]
    Processing(String),

    /// Classification error
    #[error("Classification error: {0}")]
    Classification(String),

    /// STAC catalog error
    #[error("STAC error: {0}")]
    Stac(String),

    /// Compression error
    #[error("Compression error: {0}")]
    Compression(String),

    /// GDAL error
    #[cfg(feature = "gdal")]
    #[error("GDAL error: {0}")]
    Gdal(#[from] gdal::errors::GdalError),

    /// Image crate error
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl ImageryError {
    /// Create a new invalid format error
    pub fn invalid_format(msg: impl Into<String>) -> Self {
        Self::InvalidFormat(msg.into())
    }

    /// Create a new unsupported format error
    pub fn unsupported_format(msg: impl Into<String>) -> Self {
        Self::UnsupportedFormat(msg.into())
    }

    /// Create a new metadata error
    pub fn metadata(msg: impl Into<String>) -> Self {
        Self::Metadata(msg.into())
    }

    /// Create a new processing error
    pub fn processing(msg: impl Into<String>) -> Self {
        Self::Processing(msg.into())
    }
}
