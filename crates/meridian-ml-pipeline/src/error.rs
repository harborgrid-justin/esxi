//! Error types for the ML pipeline engine

use thiserror::Error;

/// Result type for ML pipeline operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for ML pipeline operations
#[derive(Error, Debug)]
pub enum Error {
    /// Pipeline construction error
    #[error("Pipeline error: {0}")]
    Pipeline(String),

    /// Model loading error
    #[error("Model loading error: {0}")]
    ModelLoad(String),

    /// Model inference error
    #[error("Inference error: {0}")]
    Inference(String),

    /// Data transformation error
    #[error("Transform error: {0}")]
    Transform(String),

    /// Model not found in registry
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Invalid model format
    #[error("Invalid model format: {0}")]
    InvalidModel(String),

    /// Data validation error
    #[error("Data validation error: {0}")]
    Validation(String),

    /// Drift detection error
    #[error("Drift detection error: {0}")]
    Drift(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// ONNX runtime error
    #[error("ONNX runtime error: {0}")]
    Onnx(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Resource exhausted
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Create a pipeline error
    pub fn pipeline(msg: impl Into<String>) -> Self {
        Self::Pipeline(msg.into())
    }

    /// Create a model loading error
    pub fn model_load(msg: impl Into<String>) -> Self {
        Self::ModelLoad(msg.into())
    }

    /// Create an inference error
    pub fn inference(msg: impl Into<String>) -> Self {
        Self::Inference(msg.into())
    }

    /// Create a transform error
    pub fn transform(msg: impl Into<String>) -> Self {
        Self::Transform(msg.into())
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a drift error
    pub fn drift(msg: impl Into<String>) -> Self {
        Self::Drift(msg.into())
    }

    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create an invalid model error
    pub fn invalid_model(msg: impl Into<String>) -> Self {
        Self::InvalidModel(msg.into())
    }

    /// Create an ONNX error
    pub fn onnx(msg: impl Into<String>) -> Self {
        Self::Onnx(msg.into())
    }

    /// Create an IO error
    pub fn io(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}
