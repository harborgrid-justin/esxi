//! Error types for the real-time system

use std::fmt;

/// Result type for real-time operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the real-time system
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Redis error
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// CRDT error
    #[error("CRDT error: {0}")]
    Crdt(String),

    /// Synchronization error
    #[error("Synchronization error: {0}")]
    Sync(String),

    /// Conflict resolution error
    #[error("Conflict resolution error: {0}")]
    Conflict(String),

    /// State error
    #[error("State error: {0}")]
    State(String),

    /// Room not found
    #[error("Room not found: {0}")]
    RoomNotFound(String),

    /// User not found
    #[error("User not found: {0}")]
    UserNotFound(String),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Invalid message
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Encryption error
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Decryption error
    #[error("Decryption error: {0}")]
    Decryption(String),

    /// Key exchange error
    #[error("Key exchange error: {0}")]
    KeyExchange(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Channel closed
    #[error("Channel closed")]
    ChannelClosed,

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}

impl Error {
    /// Check if error is recoverable (client should retry)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::Timeout
                | Error::Connection(_)
                | Error::WebSocket(_)
                | Error::Redis(_)
                | Error::ChannelClosed
        )
    }

    /// Check if error is a client error (4xx equivalent)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Error::InvalidMessage(_)
                | Error::PermissionDenied(_)
                | Error::RoomNotFound(_)
                | Error::UserNotFound(_)
                | Error::Protocol(_)
        )
    }

    /// Check if error is a server error (5xx equivalent)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Error::Internal(_) | Error::State(_) | Error::Config(_)
        )
    }

    /// Get error code for protocol
    pub fn error_code(&self) -> u16 {
        match self {
            Error::InvalidMessage(_) => 400,
            Error::PermissionDenied(_) => 403,
            Error::RoomNotFound(_) | Error::UserNotFound(_) => 404,
            Error::Timeout => 408,
            Error::Conflict(_) => 409,
            Error::RateLimitExceeded => 429,
            Error::Internal(_) | Error::State(_) | Error::Config(_) => 500,
            Error::Connection(_) | Error::WebSocket(_) => 503,
            _ => 500,
        }
    }
}

// Conversion from axum::Error
impl From<axum::Error> for Error {
    fn from(err: axum::Error) -> Self {
        Error::WebSocket(err.to_string())
    }
}

// Conversion from serde_json::Error
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

// Conversion from rmp_serde::encode::Error
impl From<rmp_serde::encode::Error> for Error {
    fn from(err: rmp_serde::encode::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

// Conversion from rmp_serde::decode::Error
impl From<rmp_serde::decode::Error> for Error {
    fn from(err: rmp_serde::decode::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

// Conversion from tungstenite::Error
impl From<tungstenite::Error> for Error {
    fn from(err: tungstenite::Error) -> Self {
        Error::WebSocket(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recoverability() {
        assert!(Error::Timeout.is_recoverable());
        assert!(Error::Connection("test".to_string()).is_recoverable());
        assert!(!Error::PermissionDenied("test".to_string()).is_recoverable());
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(Error::InvalidMessage("test".to_string()).error_code(), 400);
        assert_eq!(Error::PermissionDenied("test".to_string()).error_code(), 403);
        assert_eq!(Error::RoomNotFound("test".to_string()).error_code(), 404);
        assert_eq!(Error::Timeout.error_code(), 408);
        assert_eq!(Error::RateLimitExceeded.error_code(), 429);
    }

    #[test]
    fn test_error_classification() {
        let client_err = Error::InvalidMessage("test".to_string());
        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());

        let server_err = Error::Internal("test".to_string());
        assert!(server_err.is_server_error());
        assert!(!server_err.is_client_error());
    }
}
