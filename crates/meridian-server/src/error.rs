//! Error types and handling for the Meridian server
//!
//! Provides comprehensive error types with proper HTTP status codes
//! and JSON error responses for API clients.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Result type alias for server operations
pub type ServerResult<T> = Result<T, ServerError>;

/// Main error type for the Meridian server
#[derive(Debug, Error)]
pub enum ServerError {
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Database errors
    #[error("Database error: {0}")]
    Database(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Authorization errors
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Resource already exists
    #[error("Resource already exists: {0}")]
    Conflict(String),

    /// Bad request
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    IoError(String),

    /// Serialization/Deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Timeout error
    #[error("Request timeout: {0}")]
    Timeout(String),

    /// Geometry errors
    #[error("Geometry error: {0}")]
    GeometryError(String),

    /// OGC service errors
    #[error("OGC service error: {0}")]
    OgcError(String),

    /// External service errors
    #[error("External service error: {0}")]
    ExternalService(String),
}

/// JSON error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Optional detailed error information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,

    /// Request ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// Timestamp of the error
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ServerError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ServerError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::Authentication(_) => StatusCode::UNAUTHORIZED,
            ServerError::Authorization(_) => StatusCode::FORBIDDEN,
            ServerError::Validation(_) => StatusCode::BAD_REQUEST,
            ServerError::NotFound(_) => StatusCode::NOT_FOUND,
            ServerError::Conflict(_) => StatusCode::CONFLICT,
            ServerError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServerError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::Serialization(_) => StatusCode::BAD_REQUEST,
            ServerError::RateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,
            ServerError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ServerError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            ServerError::GeometryError(_) => StatusCode::BAD_REQUEST,
            ServerError::OgcError(_) => StatusCode::BAD_REQUEST,
            ServerError::ExternalService(_) => StatusCode::BAD_GATEWAY,
        }
    }

    /// Get the error code string for this error
    pub fn error_code(&self) -> &'static str {
        match self {
            ServerError::Configuration(_) => "CONFIGURATION_ERROR",
            ServerError::Database(_) => "DATABASE_ERROR",
            ServerError::Authentication(_) => "AUTHENTICATION_ERROR",
            ServerError::Authorization(_) => "AUTHORIZATION_ERROR",
            ServerError::Validation(_) => "VALIDATION_ERROR",
            ServerError::NotFound(_) => "NOT_FOUND",
            ServerError::Conflict(_) => "CONFLICT",
            ServerError::BadRequest(_) => "BAD_REQUEST",
            ServerError::Internal(_) => "INTERNAL_ERROR",
            ServerError::IoError(_) => "IO_ERROR",
            ServerError::Serialization(_) => "SERIALIZATION_ERROR",
            ServerError::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            ServerError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            ServerError::Timeout(_) => "TIMEOUT",
            ServerError::GeometryError(_) => "GEOMETRY_ERROR",
            ServerError::OgcError(_) => "OGC_ERROR",
            ServerError::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
        }
    }

    /// Convert the error to an ErrorResponse
    pub fn to_response(&self, request_id: Option<String>) -> ErrorResponse {
        ErrorResponse {
            code: self.error_code().to_string(),
            message: self.to_string(),
            details: None,
            request_id,
            timestamp: chrono::Utc::now(),
        }
    }
}

// Implement IntoResponse for ServerError to use with Axum
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let response = self.to_response(None);

        // Log the error
        match status {
            StatusCode::INTERNAL_SERVER_ERROR => {
                tracing::error!("Server error: {}", self);
            }
            StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND => {
                tracing::debug!("Client error: {}", self);
            }
            _ => {
                tracing::warn!("Error: {}", self);
            }
        }

        (status, Json(response)).into_response()
    }
}

// Implement conversions from common error types

impl From<serde_json::Error> for ServerError {
    fn from(err: serde_json::Error) -> Self {
        ServerError::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> Self {
        ServerError::IoError(err.to_string())
    }
}

impl From<validator::ValidationErrors> for ServerError {
    fn from(err: validator::ValidationErrors) -> Self {
        ServerError::Validation(err.to_string())
    }
}

impl From<anyhow::Error> for ServerError {
    fn from(err: anyhow::Error) -> Self {
        ServerError::Internal(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            ServerError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ServerError::Authentication("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ServerError::Authorization("test".to_string()).status_code(),
            StatusCode::FORBIDDEN
        );
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            ServerError::NotFound("test".to_string()).error_code(),
            "NOT_FOUND"
        );
        assert_eq!(
            ServerError::Database("test".to_string()).error_code(),
            "DATABASE_ERROR"
        );
    }

    #[test]
    fn test_error_response() {
        let error = ServerError::NotFound("Layer 123".to_string());
        let response = error.to_response(Some("req-123".to_string()));

        assert_eq!(response.code, "NOT_FOUND");
        assert_eq!(response.request_id, Some("req-123".to_string()));
    }
}
