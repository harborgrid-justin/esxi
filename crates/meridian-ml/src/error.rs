//! Error types for meridian-ml

use thiserror::Error;

/// Result type alias for meridian-ml operations
pub type Result<T> = std::result::Result<T, MlError>;

/// Machine learning error types
#[derive(Error, Debug)]
pub enum MlError {
    /// Model errors
    #[error("Model error: {0}")]
    Model(String),

    /// Model not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Model already exists
    #[error("Model already exists: {0}")]
    ModelAlreadyExists(String),

    /// Invalid model format
    #[error("Invalid model format: {0}")]
    InvalidModelFormat(String),

    /// Feature extraction error
    #[error("Feature extraction error: {0}")]
    FeatureExtraction(String),

    /// Invalid feature dimensions
    #[error("Invalid feature dimensions: expected {expected}, got {actual}")]
    InvalidFeatureDimensions { expected: usize, actual: usize },

    /// Training error
    #[error("Training error: {0}")]
    Training(String),

    /// Training not converged
    #[error("Training did not converge after {0} iterations")]
    NotConverged(usize),

    /// Inference error
    #[error("Inference error: {0}")]
    Inference(String),

    /// Invalid input data
    #[error("Invalid input data: {0}")]
    InvalidInput(String),

    /// Invalid output data
    #[error("Invalid output data: {0}")]
    InvalidOutput(String),

    /// Evaluation error
    #[error("Evaluation error: {0}")]
    Evaluation(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// ONNX error
    #[cfg(feature = "onnx")]
    #[error("ONNX error: {0}")]
    Onnx(String),

    /// GPU error
    #[cfg(feature = "gpu")]
    #[error("GPU error: {0}")]
    Gpu(String),

    /// GPU not available
    #[error("GPU not available")]
    GpuNotAvailable,

    /// AutoML error
    #[cfg(feature = "automl")]
    #[error("AutoML error: {0}")]
    AutoML(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Missing required parameter
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    /// Invalid parameter value
    #[error("Invalid parameter value for {param}: {reason}")]
    InvalidParameter { param: String, reason: String },

    /// Dimension mismatch
    #[error("Dimension mismatch: {0}")]
    DimensionMismatch(String),

    /// Empty dataset
    #[error("Empty dataset provided")]
    EmptyDataset,

    /// Insufficient data
    #[error("Insufficient data: need at least {required}, got {actual}")]
    InsufficientData { required: usize, actual: usize },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serde JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Bincode error
    #[error("Bincode error: {0}")]
    Bincode(String),

    /// Array shape error
    #[error("Array shape error: {0}")]
    ShapeError(String),

    /// Numerical error
    #[error("Numerical error: {0}")]
    Numerical(String),

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Singular matrix
    #[error("Singular matrix encountered")]
    SingularMatrix,

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

impl From<bincode::Error> for MlError {
    fn from(err: bincode::Error) -> Self {
        MlError::Bincode(err.to_string())
    }
}

impl From<ndarray::ShapeError> for MlError {
    fn from(err: ndarray::ShapeError) -> Self {
        MlError::ShapeError(err.to_string())
    }
}

#[cfg(feature = "onnx")]
impl From<tract_core::anyhow::Error> for MlError {
    fn from(err: tract_core::anyhow::Error) -> Self {
        MlError::Onnx(err.to_string())
    }
}

impl MlError {
    /// Create a model error
    pub fn model<S: Into<String>>(msg: S) -> Self {
        MlError::Model(msg.into())
    }

    /// Create a feature extraction error
    pub fn feature_extraction<S: Into<String>>(msg: S) -> Self {
        MlError::FeatureExtraction(msg.into())
    }

    /// Create a training error
    pub fn training<S: Into<String>>(msg: S) -> Self {
        MlError::Training(msg.into())
    }

    /// Create an inference error
    pub fn inference<S: Into<String>>(msg: S) -> Self {
        MlError::Inference(msg.into())
    }

    /// Create an invalid input error
    pub fn invalid_input<S: Into<String>>(msg: S) -> Self {
        MlError::InvalidInput(msg.into())
    }

    /// Create a numerical error
    pub fn numerical<S: Into<String>>(msg: S) -> Self {
        MlError::Numerical(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = MlError::model("test error");
        assert_eq!(err.to_string(), "Model error: test error");
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let ml_err: MlError = io_err.into();
        assert!(matches!(ml_err, MlError::Io(_)));
    }
}
