//! Error types for the Meridian event sourcing system.

use thiserror::Error;

/// Result type alias for event sourcing operations.
pub type Result<T> = std::result::Result<T, EventError>;

/// Comprehensive error types for event sourcing operations.
#[derive(Error, Debug)]
pub enum EventError {
    /// Event store errors
    #[error("Event store error: {0}")]
    Store(String),

    /// Event not found
    #[error("Event not found: stream={stream}, version={version}")]
    EventNotFound { stream: String, version: u64 },

    /// Stream not found
    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    /// Concurrency conflict (optimistic locking)
    #[error("Concurrency conflict: expected version {expected}, got {actual}")]
    ConcurrencyConflict { expected: u64, actual: u64 },

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Schema evolution error
    #[error("Schema evolution error: {0}")]
    SchemaEvolution(String),

    /// Invalid event version
    #[error("Invalid event version: expected {expected}, got {actual}")]
    InvalidVersion { expected: String, actual: String },

    /// Aggregate error
    #[error("Aggregate error: {0}")]
    Aggregate(String),

    /// Aggregate not found
    #[error("Aggregate not found: {0}")]
    AggregateNotFound(String),

    /// Invalid aggregate state
    #[error("Invalid aggregate state: {0}")]
    InvalidAggregateState(String),

    /// Command validation error
    #[error("Command validation error: {0}")]
    CommandValidation(String),

    /// Command handler error
    #[error("Command handler error: {0}")]
    CommandHandler(String),

    /// Projection error
    #[error("Projection error: {0}")]
    Projection(String),

    /// Snapshot error
    #[error("Snapshot error: {0}")]
    Snapshot(String),

    /// Snapshot not found
    #[error("Snapshot not found for aggregate: {0}")]
    SnapshotNotFound(String),

    /// Replay error
    #[error("Replay error: {0}")]
    Replay(String),

    /// Saga error
    #[error("Saga error: {0}")]
    Saga(String),

    /// Saga timeout
    #[error("Saga timeout: {0}")]
    SagaTimeout(String),

    /// Event bus error
    #[error("Event bus error: {0}")]
    EventBus(String),

    /// Subscriber error
    #[error("Subscriber error: {0}")]
    Subscriber(String),

    /// Idempotency violation
    #[error("Idempotency violation: duplicate operation {0}")]
    IdempotencyViolation(String),

    /// Archival error
    #[error("Archival error: {0}")]
    Archival(String),

    /// Causation error
    #[error("Causation error: {0}")]
    Causation(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Database error
    #[cfg(feature = "rocksdb-backend")]
    #[error("Database error: {0}")]
    Database(#[from] rocksdb::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    Configuration(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Custom error
    #[error("{0}")]
    Custom(String),
}

impl EventError {
    /// Create a store error.
    pub fn store(msg: impl Into<String>) -> Self {
        Self::Store(msg.into())
    }

    /// Create a serialization error.
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    /// Create a deserialization error.
    pub fn deserialization(msg: impl Into<String>) -> Self {
        Self::Deserialization(msg.into())
    }

    /// Create an aggregate error.
    pub fn aggregate(msg: impl Into<String>) -> Self {
        Self::Aggregate(msg.into())
    }

    /// Create a command validation error.
    pub fn command_validation(msg: impl Into<String>) -> Self {
        Self::CommandValidation(msg.into())
    }

    /// Create a projection error.
    pub fn projection(msg: impl Into<String>) -> Self {
        Self::Projection(msg.into())
    }

    /// Create a saga error.
    pub fn saga(msg: impl Into<String>) -> Self {
        Self::Saga(msg.into())
    }

    /// Create an internal error.
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    /// Create a custom error.
    pub fn custom(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }
}

/// Extension trait for converting bincode errors to EventError.
impl From<Box<bincode::ErrorKind>> for EventError {
    fn from(err: Box<bincode::ErrorKind>) -> Self {
        EventError::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EventError::ConcurrencyConflict {
            expected: 5,
            actual: 3,
        };
        assert_eq!(
            err.to_string(),
            "Concurrency conflict: expected version 5, got 3"
        );
    }

    #[test]
    fn test_error_construction() {
        let err = EventError::store("Database connection failed");
        assert!(matches!(err, EventError::Store(_)));
    }
}
