use thiserror::Error;

/// Result type for SDK operations
pub type Result<T> = std::result::Result<T, Error>;

/// SDK error types
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// URL parsing error
    #[error("Invalid URL: {0}")]
    UrlError(#[from] url::ParseError),

    /// API returned an error response
    #[error("API error: {status} - {message}")]
    ApiError {
        status: u16,
        message: String,
    },

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// Authorization error
    #[error("Authorization failed: {0}")]
    AuthorizationError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Connection timeout
    #[error("Connection timeout")]
    Timeout,

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a new API error
    pub fn api_error(status: u16, message: impl Into<String>) -> Self {
        Error::ApiError {
            status,
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Error::NotFound(resource.into())
    }

    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Error::ValidationError(message.into())
    }

    /// Check if error is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::NotFound(_))
    }

    /// Check if error is an authentication error
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::AuthenticationError(_) | Error::AuthorizationError(_))
    }
}
