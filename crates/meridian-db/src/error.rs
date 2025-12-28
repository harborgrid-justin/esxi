//! Database error types for Meridian GIS Platform

use thiserror::Error;

/// Database errors
#[derive(Error, Debug)]
pub enum DbError {
    /// SQL execution error
    #[error("SQL error: {0}")]
    SqlError(#[from] sqlx::Error),

    /// Migration error
    #[error("Migration error: {0}")]
    MigrationError(String),

    /// Connection pool error
    #[error("Connection pool error: {0}")]
    PoolError(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Not found error
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Geometry error
    #[error("Geometry error: {0}")]
    GeometryError(String),

    /// PostGIS extension error
    #[error("PostGIS error: {0}")]
    PostGisError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// UUID error
    #[error("UUID error: {0}")]
    UuidError(#[from] uuid::Error),

    /// Connection timeout
    #[error("Connection timeout")]
    Timeout,

    /// Generic error
    #[error("Database error: {0}")]
    Generic(String),
}

/// Result type alias for database operations
pub type DbResult<T> = Result<T, DbError>;

impl DbError {
    /// Check if error is a connection error
    pub fn is_connection_error(&self) -> bool {
        matches!(
            self,
            DbError::PoolError(_) | DbError::Timeout | DbError::SqlError(_)
        )
    }

    /// Check if error is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, DbError::NotFound(_))
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            DbError::SqlError(err) => {
                // Connection errors are retryable
                err.as_database_error().is_none()
            }
            DbError::PoolError(_) | DbError::Timeout => true,
            _ => false,
        }
    }
}
