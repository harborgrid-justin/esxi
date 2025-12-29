//! Error types for the Meridian Data Pipeline.
//!
//! This module provides comprehensive error handling for all pipeline operations,
//! including source reading, transformations, sink writing, and orchestration.

use std::fmt;
use thiserror::Error;

/// Result type alias for pipeline operations.
pub type Result<T> = std::result::Result<T, PipelineError>;

/// Main error type for the Meridian Data Pipeline.
#[derive(Error, Debug)]
pub enum PipelineError {
    /// Error occurred while reading from a data source.
    #[error("Source error: {0}")]
    Source(#[from] SourceError),

    /// Error occurred during data transformation.
    #[error("Transform error: {0}")]
    Transform(#[from] TransformError),

    /// Error occurred while writing to a data sink.
    #[error("Sink error: {0}")]
    Sink(#[from] SinkError),

    /// Error in pipeline configuration.
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Error in pipeline execution.
    #[error("Execution error: {0}")]
    Execution(String),

    /// Error in data validation.
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Error in checkpoint/resume operations.
    #[error("Checkpoint error: {0}")]
    Checkpoint(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Arrow error.
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    /// DataFusion error.
    #[error("DataFusion error: {0}")]
    DataFusion(#[from] datafusion::error::DataFusionError),

    /// SQL error.
    #[error("SQL error: {0}")]
    Sql(#[from] sqlx::Error),

    /// HTTP/API error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Geometry processing error.
    #[error("Geometry error: {0}")]
    Geometry(String),

    /// Projection error.
    #[error("Projection error: {0}")]
    Projection(String),

    /// Schema mismatch error.
    #[error("Schema mismatch: expected {expected}, found {found}")]
    SchemaMismatch { expected: String, found: String },

    /// Resource not found error.
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Timeout error.
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Concurrency error.
    #[error("Concurrency error: {0}")]
    Concurrency(String),

    /// Generic error.
    #[error("{0}")]
    Other(String),
}

/// Errors related to data sources.
#[derive(Error, Debug)]
pub enum SourceError {
    /// File not found or inaccessible.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Database connection error.
    #[error("Database connection error: {0}")]
    DatabaseConnection(String),

    /// API endpoint error.
    #[error("API error: {endpoint} - {message}")]
    ApiError { endpoint: String, message: String },

    /// Streaming source error.
    #[error("Stream error: {0}")]
    Stream(String),

    /// Unsupported format.
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Parse error.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Authentication error.
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Authorization error.
    #[error("Authorization failed: {0}")]
    Authorization(String),
}

/// Errors related to data transformations.
#[derive(Error, Debug)]
pub enum TransformError {
    /// Invalid geometry.
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    /// Projection transformation failed.
    #[error("Projection failed: from {from_srs} to {to_srs} - {reason}")]
    ProjectionFailed {
        from_srs: String,
        to_srs: String,
        reason: String,
    },

    /// Filter operation failed.
    #[error("Filter failed: {0}")]
    FilterFailed(String),

    /// Aggregation operation failed.
    #[error("Aggregation failed: {0}")]
    AggregationFailed(String),

    /// Join operation failed.
    #[error("Join failed: {0}")]
    JoinFailed(String),

    /// Data type conversion error.
    #[error("Type conversion error: cannot convert {from} to {to}")]
    TypeConversion { from: String, to: String },

    /// Expression evaluation error.
    #[error("Expression evaluation error: {0}")]
    ExpressionEvaluation(String),

    /// Missing required field.
    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Errors related to data sinks.
#[derive(Error, Debug)]
pub enum SinkError {
    /// File write error.
    #[error("File write error: {0}")]
    FileWrite(String),

    /// Database write error.
    #[error("Database write error: {0}")]
    DatabaseWrite(String),

    /// Vector tile generation error.
    #[error("Vector tile generation error: {0}")]
    VectorTile(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Compression error.
    #[error("Compression error: {0}")]
    Compression(String),

    /// Buffer overflow.
    #[error("Buffer overflow: {0}")]
    BufferOverflow(String),
}

/// Errors related to pipeline configuration.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Invalid YAML configuration.
    #[error("Invalid YAML: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),

    /// Invalid JSON configuration.
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    /// Invalid TOML configuration.
    #[error("Invalid TOML: {0}")]
    InvalidToml(#[from] toml::de::Error),

    /// Missing required configuration.
    #[error("Missing required config: {0}")]
    MissingConfig(String),

    /// Invalid configuration value.
    #[error("Invalid config value for {key}: {value}")]
    InvalidValue { key: String, value: String },

    /// Configuration version mismatch.
    #[error("Config version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },
}

/// Errors related to data validation.
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Geometry validation failed.
    #[error("Geometry validation failed: {0}")]
    GeometryValidation(String),

    /// Schema validation failed.
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    /// Data quality check failed.
    #[error("Data quality check failed: {check} - {reason}")]
    QualityCheck { check: String, reason: String },

    /// Constraint violation.
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    /// Out of range value.
    #[error("Value out of range: {field} - expected {expected}, found {found}")]
    OutOfRange {
        field: String,
        expected: String,
        found: String,
    },
}

// Implement conversion from serde_json::Error
impl From<serde_json::Error> for PipelineError {
    fn from(err: serde_json::Error) -> Self {
        PipelineError::Serialization(err.to_string())
    }
}

// Implement conversion from serde_yaml::Error
impl From<serde_yaml::Error> for PipelineError {
    fn from(err: serde_yaml::Error) -> Self {
        PipelineError::Config(ConfigError::InvalidYaml(err))
    }
}

// Implement conversion from String for convenience
impl From<String> for PipelineError {
    fn from(s: String) -> Self {
        PipelineError::Other(s)
    }
}

impl From<&str> for PipelineError {
    fn from(s: &str) -> Self {
        PipelineError::Other(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PipelineError::Source(SourceError::FileNotFound("test.geojson".to_string()));
        assert!(err.to_string().contains("test.geojson"));
    }

    #[test]
    fn test_schema_mismatch_error() {
        let err = PipelineError::SchemaMismatch {
            expected: "Point".to_string(),
            found: "LineString".to_string(),
        };
        assert!(err.to_string().contains("Point"));
        assert!(err.to_string().contains("LineString"));
    }

    #[test]
    fn test_transform_error() {
        let err = TransformError::ProjectionFailed {
            from_srs: "EPSG:4326".to_string(),
            to_srs: "EPSG:3857".to_string(),
            reason: "Invalid coordinates".to_string(),
        };
        assert!(err.to_string().contains("EPSG:4326"));
        assert!(err.to_string().contains("EPSG:3857"));
    }
}
