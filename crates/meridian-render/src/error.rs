//! Error types for the rendering engine

use thiserror::Error;

/// Result type for rendering operations
pub type RenderResult<T> = Result<T, RenderError>;

/// Errors that can occur during rendering operations
#[derive(Error, Debug)]
pub enum RenderError {
    /// Tile coordinate is invalid
    #[error("Invalid tile coordinate: z={z}, x={x}, y={y}")]
    InvalidTileCoordinate { z: u8, x: u32, y: u32 },

    /// Zoom level out of range
    #[error("Zoom level {0} out of range (0-22)")]
    InvalidZoomLevel(u8),

    /// Image encoding/decoding error
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    /// SVG rendering error
    #[error("SVG rendering error: {0}")]
    SvgError(String),

    /// MVT encoding/decoding error
    #[error("MVT error: {0}")]
    MvtError(String),

    /// Style parsing error
    #[error("Style error: {0}")]
    StyleError(String),

    /// Cache error
    #[error("Cache error: {0}")]
    CacheError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Compression error
    #[error("Compression error: {0}")]
    CompressionError(String),

    /// Geometry error
    #[error("Geometry error: {0}")]
    GeometryError(String),

    /// Rendering pipeline error
    #[error("Pipeline error: {0}")]
    PipelineError(String),

    /// Symbol not found
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Feature limit exceeded
    #[error("Feature limit exceeded: {actual} > {limit}")]
    FeatureLimitExceeded { actual: usize, limit: usize },

    /// Memory limit exceeded
    #[error("Memory limit exceeded")]
    MemoryLimitExceeded,

    /// Timeout error
    #[error("Rendering timeout after {0}ms")]
    Timeout(u64),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<prost::DecodeError> for RenderError {
    fn from(err: prost::DecodeError) -> Self {
        RenderError::MvtError(err.to_string())
    }
}

impl From<prost::EncodeError> for RenderError {
    fn from(err: prost::EncodeError) -> Self {
        RenderError::MvtError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RenderError::InvalidTileCoordinate { z: 10, x: 1024, y: 512 };
        assert_eq!(
            err.to_string(),
            "Invalid tile coordinate: z=10, x=1024, y=512"
        );
    }

    #[test]
    fn test_zoom_level_error() {
        let err = RenderError::InvalidZoomLevel(25);
        assert_eq!(err.to_string(), "Zoom level 25 out of range (0-22)");
    }
}
