//! Error types for vector tile operations

use std::fmt;

/// Result type alias for vector tile operations
pub type Result<T> = std::result::Result<T, Error>;

/// Vector tile error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Encoding error
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Decoding error
    #[error("Decoding error: {0}")]
    Decoding(String),

    /// Protocol buffer error
    #[error("Protocol buffer error: {0}")]
    Protobuf(#[from] prost::DecodeError),

    /// Geometry error
    #[error("Geometry error: {0}")]
    Geometry(String),

    /// Tile coordinate error
    #[error("Invalid tile coordinate: {0}")]
    InvalidCoordinate(String),

    /// Tile bounds error
    #[error("Invalid tile bounds: {0}")]
    InvalidBounds(String),

    /// Compression error
    #[error("Compression error: {0}")]
    Compression(String),

    /// Decompression error
    #[error("Decompression error: {0}")]
    Decompression(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Source error
    #[error("Source error: {0}")]
    Source(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),

    /// Style error
    #[error("Style error: {0}")]
    Style(String),

    /// TileJSON error
    #[error("TileJSON error: {0}")]
    TileJson(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Feature too large
    #[error("Feature exceeds maximum size: {size} bytes (max: {max})")]
    FeatureTooLarge { size: usize, max: usize },

    /// Layer not found
    #[error("Layer not found: {0}")]
    LayerNotFound(String),

    /// Tile not found
    #[error("Tile not found: {0}")]
    TileNotFound(String),

    /// Invalid zoom level
    #[error("Invalid zoom level: {zoom} (min: {min}, max: {max})")]
    InvalidZoom { zoom: u8, min: u8, max: u8 },

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// SQLite error (for MBTiles)
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// S3 error
    #[error("S3 error: {0}")]
    S3(String),

    /// HTTP error
    #[error("HTTP error: {0}")]
    Http(String),

    /// PMTiles error
    #[error("PMTiles error: {0}")]
    PMTiles(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a new encoding error
    pub fn encoding<S: Into<String>>(msg: S) -> Self {
        Error::Encoding(msg.into())
    }

    /// Create a new decoding error
    pub fn decoding<S: Into<String>>(msg: S) -> Self {
        Error::Decoding(msg.into())
    }

    /// Create a new geometry error
    pub fn geometry<S: Into<String>>(msg: S) -> Self {
        Error::Geometry(msg.into())
    }

    /// Create a new storage error
    pub fn storage<S: Into<String>>(msg: S) -> Self {
        Error::Storage(msg.into())
    }

    /// Create a new source error
    pub fn source<S: Into<String>>(msg: S) -> Self {
        Error::Source(msg.into())
    }

    /// Create a new compression error
    pub fn compression<S: Into<String>>(msg: S) -> Self {
        Error::Compression(msg.into())
    }

    /// Create a new S3 error
    pub fn s3<S: Into<String>>(msg: S) -> Self {
        Error::S3(msg.into())
    }

    /// Create a new PMTiles error
    pub fn pmtiles<S: Into<String>>(msg: S) -> Self {
        Error::PMTiles(msg.into())
    }
}

// Convert anyhow errors
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::encoding("test error");
        assert_eq!(err.to_string(), "Encoding error: test error");

        let err = Error::InvalidZoom {
            zoom: 30,
            min: 0,
            max: 24,
        };
        assert_eq!(
            err.to_string(),
            "Invalid zoom level: 30 (min: 0, max: 24)"
        );
    }
}
