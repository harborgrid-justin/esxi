//! Error types for Meridian 3D

use thiserror::Error;

/// Result type alias for Meridian 3D operations
pub type Result<T> = std::result::Result<T, Error>;

/// Comprehensive error type for 3D rendering operations
#[derive(Error, Debug)]
pub enum Error {
    /// GPU adapter not found
    #[error("No suitable GPU adapter found")]
    NoAdapter,

    /// WebGPU device error
    #[error("WebGPU device error: {0}")]
    DeviceError(#[from] wgpu::RequestDeviceError),

    /// Surface error
    #[error("Surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),

    /// Texture creation error
    #[error("Failed to create texture: {0}")]
    TextureError(String),

    /// Shader compilation error
    #[error("Shader compilation failed: {0}")]
    ShaderError(String),

    /// Image loading error
    #[error("Failed to load image: {0}")]
    ImageError(#[from] image::ImageError),

    /// glTF model loading error
    #[error("Failed to load glTF model: {0}")]
    GltfError(#[from] gltf::Error),

    /// Heightmap processing error
    #[error("Heightmap error: {0}")]
    HeightmapError(String),

    /// Mesh generation error
    #[error("Mesh generation failed: {0}")]
    MeshError(String),

    /// Terrain rendering error
    #[error("Terrain rendering error: {0}")]
    TerrainError(String),

    /// Building rendering error
    #[error("Building rendering error: {0}")]
    BuildingError(String),

    /// Scene graph error
    #[error("Scene graph error: {0}")]
    SceneError(String),

    /// LOD system error
    #[error("LOD system error: {0}")]
    LodError(String),

    /// Physics error
    #[error("Physics error: {0}")]
    PhysicsError(String),

    /// Camera navigation error
    #[error("Navigation error: {0}")]
    NavigationError(String),

    /// Picking/selection error
    #[error("Picking error: {0}")]
    PickingError(String),

    /// Lighting system error
    #[error("Lighting error: {0}")]
    LightingError(String),

    /// Shadow mapping error
    #[error("Shadow mapping error: {0}")]
    ShadowError(String),

    /// Atmospheric effects error
    #[error("Atmosphere error: {0}")]
    AtmosphereError(String),

    /// Analysis error (viewshed, shadow analysis, etc.)
    #[error("Analysis error: {0}")]
    AnalysisError(String),

    /// Export error (screenshot, video)
    #[error("Export error: {0}")]
    ExportError(String),

    /// Video encoding error
    #[cfg(feature = "video-export")]
    #[error("Video encoding error: {0}")]
    VideoError(String),

    /// File I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Out of bounds access
    #[error("Index out of bounds: {0}")]
    OutOfBounds(String),

    /// Generic error with context
    #[error("Error: {0}")]
    Generic(String),

    /// Wrapped anyhow error for flexibility
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Error {
    /// Create a terrain error
    pub fn terrain(msg: impl Into<String>) -> Self {
        Self::TerrainError(msg.into())
    }

    /// Create a building error
    pub fn building(msg: impl Into<String>) -> Self {
        Self::BuildingError(msg.into())
    }

    /// Create a scene error
    pub fn scene(msg: impl Into<String>) -> Self {
        Self::SceneError(msg.into())
    }

    /// Create a mesh error
    pub fn mesh(msg: impl Into<String>) -> Self {
        Self::MeshError(msg.into())
    }

    /// Create a shader error
    pub fn shader(msg: impl Into<String>) -> Self {
        Self::ShaderError(msg.into())
    }

    /// Create a lighting error
    pub fn lighting(msg: impl Into<String>) -> Self {
        Self::LightingError(msg.into())
    }

    /// Create an analysis error
    pub fn analysis(msg: impl Into<String>) -> Self {
        Self::AnalysisError(msg.into())
    }

    /// Create an export error
    pub fn export(msg: impl Into<String>) -> Self {
        Self::ExportError(msg.into())
    }

    /// Create a not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create an invalid operation error
    pub fn invalid_op(msg: impl Into<String>) -> Self {
        Self::InvalidOperation(msg.into())
    }
}

/// Extension trait for Result types to add context
pub trait ResultExt<T> {
    /// Add context to an error
    fn context(self, msg: impl Into<String>) -> Result<T>;

    /// Add context from a closure (lazy evaluation)
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E: Into<Error>> ResultExt<T> for std::result::Result<T, E> {
    fn context(self, msg: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            let err: Error = e.into();
            Error::Generic(format!("{}: {}", msg.into(), err))
        })
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let err: Error = e.into();
            Error::Generic(format!("{}: {}", f(), err))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_constructors() {
        let err = Error::terrain("test terrain error");
        assert!(matches!(err, Error::TerrainError(_)));

        let err = Error::building("test building error");
        assert!(matches!(err, Error::BuildingError(_)));

        let err = Error::not_found("resource.obj");
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[test]
    fn test_result_ext() {
        let result: std::result::Result<i32, std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));

        let err = result.context("Failed to load file").unwrap_err();
        assert!(err.to_string().contains("Failed to load file"));
    }
}
