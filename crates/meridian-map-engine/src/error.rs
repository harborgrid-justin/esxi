//! Error types for the Meridian Map Engine.

use thiserror::Error;

/// Result type alias for map engine operations.
pub type Result<T> = std::result::Result<T, MapEngineError>;

/// Errors that can occur in the map rendering engine.
#[derive(Error, Debug)]
pub enum MapEngineError {
    /// WebGPU adapter not found or unavailable.
    #[error("WebGPU adapter not available: {0}")]
    AdapterNotAvailable(String),

    /// Failed to request WebGPU device.
    #[error("Failed to request WebGPU device: {0}")]
    DeviceRequest(String),

    /// Shader compilation error.
    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),

    /// Pipeline creation error.
    #[error("Failed to create render pipeline: {0}")]
    PipelineCreation(String),

    /// Texture creation or loading error.
    #[error("Texture error: {0}")]
    Texture(String),

    /// Buffer creation or management error.
    #[error("Buffer error: {0}")]
    Buffer(String),

    /// Tile loading error.
    #[error("Failed to load tile {z}/{x}/{y}: {error}")]
    TileLoad {
        x: u32,
        y: u32,
        z: u32,
        error: String,
    },

    /// Tile cache error.
    #[error("Tile cache error: {0}")]
    TileCache(String),

    /// Style parsing error.
    #[error("Style parsing error at {location}: {message}")]
    StyleParse { location: String, message: String },

    /// Style evaluation error.
    #[error("Style evaluation error: {0}")]
    StyleEvaluation(String),

    /// Camera projection error.
    #[error("Camera projection error: {0}")]
    Projection(String),

    /// Feature picking error.
    #[error("Feature picking error: {0}")]
    Picking(String),

    /// Font loading or rendering error.
    #[error("Font error: {0}")]
    Font(String),

    /// Image decoding error.
    #[error("Image decoding error: {0}")]
    ImageDecode(#[from] image::ImageError),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error.
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// Network error (tile fetching).
    #[error("Network error: {0}")]
    Network(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Out of memory.
    #[error("Out of memory: {0}")]
    OutOfMemory(String),

    /// Operation not supported.
    #[error("Operation not supported: {0}")]
    NotSupported(String),

    /// Generic rendering error.
    #[error("Rendering error: {0}")]
    Rendering(String),
}

impl MapEngineError {
    /// Check if the error is recoverable.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            MapEngineError::TileLoad { .. }
                | MapEngineError::Network(_)
                | MapEngineError::ResourceNotFound(_)
        )
    }

    /// Check if the error is fatal and requires engine restart.
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            MapEngineError::AdapterNotAvailable(_)
                | MapEngineError::DeviceRequest(_)
                | MapEngineError::OutOfMemory(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recoverability() {
        let recoverable = MapEngineError::TileLoad {
            x: 0,
            y: 0,
            z: 0,
            error: "timeout".to_string(),
        };
        assert!(recoverable.is_recoverable());
        assert!(!recoverable.is_fatal());

        let fatal = MapEngineError::OutOfMemory("GPU memory exhausted".to_string());
        assert!(!fatal.is_recoverable());
        assert!(fatal.is_fatal());
    }
}
