//! Error types for spatial analysis operations

use thiserror::Error;

/// Result type for analysis operations
pub type Result<T> = std::result::Result<T, AnalysisError>;

/// Errors that can occur during spatial analysis
#[derive(Error, Debug)]
pub enum AnalysisError {
    /// Invalid geometry provided
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    /// Invalid parameters provided
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    /// Topology error
    #[error("Topology error: {0}")]
    TopologyError(String),

    /// Network analysis error
    #[error("Network analysis error: {0}")]
    NetworkError(String),

    /// Surface analysis error
    #[error("Surface analysis error: {0}")]
    SurfaceError(String),

    /// Buffer operation error
    #[error("Buffer operation error: {0}")]
    BufferError(String),

    /// Overlay operation error
    #[error("Overlay operation error: {0}")]
    OverlayError(String),

    /// Proximity analysis error
    #[error("Proximity analysis error: {0}")]
    ProximityError(String),

    /// Statistical analysis error
    #[error("Statistical analysis error: {0}")]
    StatisticsError(String),

    /// Transformation error
    #[error("Transformation error: {0}")]
    TransformationError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Computation error
    #[error("Computation error: {0}")]
    ComputationError(String),

    /// Empty geometry collection
    #[error("Empty geometry collection")]
    EmptyGeometry,

    /// Insufficient data points
    #[error("Insufficient data points: required {required}, got {actual}")]
    InsufficientData { required: usize, actual: usize },

    /// Operation not supported
    #[error("Operation not supported: {0}")]
    UnsupportedOperation(String),

    /// Core library error
    #[error("Core library error: {0}")]
    CoreError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

impl AnalysisError {
    /// Create a new invalid geometry error
    pub fn invalid_geometry(msg: impl Into<String>) -> Self {
        Self::InvalidGeometry(msg.into())
    }

    /// Create a new invalid parameters error
    pub fn invalid_parameters(msg: impl Into<String>) -> Self {
        Self::InvalidParameters(msg.into())
    }

    /// Create a new topology error
    pub fn topology_error(msg: impl Into<String>) -> Self {
        Self::TopologyError(msg.into())
    }

    /// Create a new network error
    pub fn network_error(msg: impl Into<String>) -> Self {
        Self::NetworkError(msg.into())
    }

    /// Create a new computation error
    pub fn computation_error(msg: impl Into<String>) -> Self {
        Self::ComputationError(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AnalysisError::invalid_geometry("invalid polygon");
        assert_eq!(err.to_string(), "Invalid geometry: invalid polygon");
    }

    #[test]
    fn test_insufficient_data_error() {
        let err = AnalysisError::InsufficientData {
            required: 10,
            actual: 5,
        };
        assert_eq!(
            err.to_string(),
            "Insufficient data points: required 10, got 5"
        );
    }
}
