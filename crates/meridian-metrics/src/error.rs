//! Error types for the Meridian Metrics system.

use std::fmt;

/// Result type alias for Meridian Metrics operations.
pub type Result<T> = std::result::Result<T, MetricsError>;

/// Primary error type for the metrics system.
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    /// Error during metric collection
    #[error("Metric collection error: {0}")]
    Collection(String),

    /// Error during metric export
    #[error("Export error: {0}")]
    Export(String),

    /// Error in health check system
    #[error("Health check error: {0}")]
    HealthCheck(String),

    /// Error in profiling system
    #[error("Profiling error: {0}")]
    Profiling(String),

    /// SLA threshold violation
    #[error("SLA violation: {metric} {comparison} threshold {threshold}, got {actual}")]
    SlaViolation {
        metric: String,
        comparison: String,
        threshold: f64,
        actual: f64,
    },

    /// Error in streaming subsystem
    #[error("Streaming error: {0}")]
    Streaming(String),

    /// Error in aggregation
    #[error("Aggregation error: {0}")]
    Aggregation(String),

    /// Invalid metric name or label
    #[error("Invalid metric: {0}")]
    InvalidMetric(String),

    /// Invalid configuration
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// OpenTelemetry error
    #[error("OpenTelemetry error: {0}")]
    OpenTelemetry(String),

    /// Prometheus error
    #[error("Prometheus error: {0}")]
    Prometheus(String),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Metric not found
    #[error("Metric not found: {0}")]
    NotFound(String),

    /// Lock acquisition failed
    #[error("Failed to acquire lock: {0}")]
    LockError(String),

    /// Generic error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl MetricsError {
    /// Create a collection error
    pub fn collection<S: Into<String>>(msg: S) -> Self {
        Self::Collection(msg.into())
    }

    /// Create an export error
    pub fn export<S: Into<String>>(msg: S) -> Self {
        Self::Export(msg.into())
    }

    /// Create a health check error
    pub fn health_check<S: Into<String>>(msg: S) -> Self {
        Self::HealthCheck(msg.into())
    }

    /// Create a profiling error
    pub fn profiling<S: Into<String>>(msg: S) -> Self {
        Self::Profiling(msg.into())
    }

    /// Create a streaming error
    pub fn streaming<S: Into<String>>(msg: S) -> Self {
        Self::Streaming(msg.into())
    }

    /// Create an aggregation error
    pub fn aggregation<S: Into<String>>(msg: S) -> Self {
        Self::Aggregation(msg.into())
    }

    /// Create an invalid metric error
    pub fn invalid_metric<S: Into<String>>(msg: S) -> Self {
        Self::InvalidMetric(msg.into())
    }

    /// Create a configuration error
    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(msg: S) -> Self {
        Self::Timeout(msg.into())
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}

impl From<opentelemetry::metrics::MetricsError> for MetricsError {
    fn from(err: opentelemetry::metrics::MetricsError) -> Self {
        Self::OpenTelemetry(err.to_string())
    }
}

impl From<prometheus::Error> for MetricsError {
    fn from(err: prometheus::Error) -> Self {
        Self::Prometheus(err.to_string())
    }
}

impl From<tungstenite::Error> for MetricsError {
    fn from(err: tungstenite::Error) -> Self {
        Self::WebSocket(err.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for MetricsError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::LockError(err.to_string())
    }
}

/// Extension trait for converting Results
pub trait ResultExt<T> {
    /// Convert to MetricsError with context
    fn context<S: Into<String>>(self, msg: S) -> Result<T>;
}

impl<T, E: fmt::Display> ResultExt<T> for std::result::Result<T, E> {
    fn context<S: Into<String>>(self, msg: S) -> Result<T> {
        self.map_err(|e| MetricsError::Internal(format!("{}: {}", msg.into(), e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = MetricsError::collection("test error");
        assert!(err.to_string().contains("test error"));

        let err = MetricsError::SlaViolation {
            metric: "query_latency".to_string(),
            comparison: "exceeds".to_string(),
            threshold: 100.0,
            actual: 150.0,
        };
        assert!(err.to_string().contains("query_latency"));
        assert!(err.to_string().contains("150"));
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let metrics_err: MetricsError = io_err.into();
        assert!(matches!(metrics_err, MetricsError::Io(_)));
    }
}
