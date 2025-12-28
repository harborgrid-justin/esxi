//! Error types for the streaming system.

use thiserror::Error;

/// Result type for streaming operations.
pub type Result<T> = std::result::Result<T, StreamError>;

/// Errors that can occur in the streaming system.
#[derive(Error, Debug)]
pub enum StreamError {
    /// WebSocket connection error
    #[error("WebSocket connection error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Connection closed unexpectedly
    #[error("Connection closed unexpectedly")]
    ConnectionClosed,

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    /// Channel not found
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    /// Room not found
    #[error("Room not found: {0}")]
    RoomNotFound(String),

    /// Client not found
    #[error("Client not found: {0}")]
    ClientNotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Maximum connections exceeded
    #[error("Maximum connections exceeded")]
    MaxConnectionsExceeded,

    /// Invalid subscription
    #[error("Invalid subscription: {0}")]
    InvalidSubscription(String),

    /// Sync conflict
    #[error("Sync conflict: {0}")]
    SyncConflict(String),

    /// Invalid viewport
    #[error("Invalid viewport: {0}")]
    InvalidViewport(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Generic error
    #[error("{0}")]
    Generic(String),
}

impl StreamError {
    /// Create a generic error with a custom message.
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    /// Create an invalid message error.
    pub fn invalid_message(msg: impl Into<String>) -> Self {
        Self::InvalidMessage(msg.into())
    }

    /// Create a permission denied error.
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create a sync conflict error.
    pub fn sync_conflict(msg: impl Into<String>) -> Self {
        Self::SyncConflict(msg.into())
    }

    /// Returns true if this error is recoverable.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            StreamError::Timeout
                | StreamError::ConnectionClosed
                | StreamError::WebSocket(_)
        )
    }

    /// Returns true if this error indicates a client error (4xx equivalent).
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            StreamError::InvalidMessage(_)
                | StreamError::PermissionDenied(_)
                | StreamError::InvalidSubscription(_)
                | StreamError::InvalidViewport(_)
        )
    }

    /// Returns true if this error indicates a server error (5xx equivalent).
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            StreamError::MaxConnectionsExceeded | StreamError::Generic(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = StreamError::generic("test error");
        assert!(matches!(err, StreamError::Generic(_)));
    }

    #[test]
    fn test_error_is_recoverable() {
        assert!(StreamError::Timeout.is_recoverable());
        assert!(StreamError::ConnectionClosed.is_recoverable());
        assert!(!StreamError::PermissionDenied("test".into()).is_recoverable());
    }

    #[test]
    fn test_error_classification() {
        let client_err = StreamError::InvalidMessage("test".into());
        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());

        let server_err = StreamError::MaxConnectionsExceeded;
        assert!(!server_err.is_client_error());
        assert!(server_err.is_server_error());
    }
}
