//! Comprehensive error types for meridian-compression
//!
//! Enterprise-grade error handling with detailed context and metrics.

use std::io;
use thiserror::Error;

/// Result type alias for compression operations
pub type Result<T> = std::result::Result<T, CompressionError>;

/// Comprehensive error types for all compression operations
#[derive(Error, Debug)]
pub enum CompressionError {
    /// I/O operation failed
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// LZ4 compression/decompression error
    #[error("LZ4 error: {0}")]
    Lz4(String),

    /// Zstandard compression/decompression error
    #[error("Zstandard error: {0}")]
    Zstd(String),

    /// Brotli compression/decompression error
    #[error("Brotli error: {0}")]
    Brotli(String),

    /// Gzip compression/decompression error
    #[error("Gzip error: {0}")]
    Gzip(String),

    /// Snappy compression/decompression error
    #[error("Snappy error: {0}")]
    Snappy(String),

    /// Dictionary training error
    #[error("Dictionary training error: {0}")]
    DictionaryTraining(String),

    /// Dictionary not found or invalid
    #[error("Dictionary error: {0}")]
    Dictionary(String),

    /// Invalid compression level
    #[error("Invalid compression level {level}: {reason}")]
    InvalidCompressionLevel { level: i32, reason: String },

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Buffer size error
    #[error("Buffer size error: {0}")]
    BufferSize(String),

    /// Decompression error - corrupted data
    #[error("Decompression failed: data may be corrupted - {0}")]
    CorruptedData(String),

    /// Checksum mismatch
    #[error("Checksum mismatch: expected {expected:x}, got {actual:x}")]
    ChecksumMismatch { expected: u64, actual: u64 },

    /// Unsupported compression algorithm
    #[error("Unsupported compression algorithm: {0}")]
    UnsupportedAlgorithm(String),

    /// Pipeline error
    #[error("Pipeline error at stage {stage}: {error}")]
    Pipeline { stage: String, error: String },

    /// Adaptive compression error
    #[error("Adaptive compression error: {0}")]
    Adaptive(String),

    /// Delta compression error
    #[error("Delta compression error: {0}")]
    Delta(String),

    /// Streaming operation error
    #[error("Streaming error: {0}")]
    Streaming(String),

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource} - {limit}")]
    ResourceLimit { resource: String, limit: String },

    /// Timeout error
    #[error("Operation timed out after {seconds}s")]
    Timeout { seconds: u64 },

    /// Concurrent operation error
    #[error("Concurrent operation error: {0}")]
    Concurrency(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Custom error with context
    #[error("Compression error: {message} (context: {context})")]
    Custom { message: String, context: String },
}

impl CompressionError {
    /// Create a custom error with context
    pub fn custom(message: impl Into<String>, context: impl Into<String>) -> Self {
        Self::Custom {
            message: message.into(),
            context: context.into(),
        }
    }

    /// Create an LZ4 error
    pub fn lz4(msg: impl Into<String>) -> Self {
        Self::Lz4(msg.into())
    }

    /// Create a Zstandard error
    pub fn zstd(msg: impl Into<String>) -> Self {
        Self::Zstd(msg.into())
    }

    /// Create a Brotli error
    pub fn brotli(msg: impl Into<String>) -> Self {
        Self::Brotli(msg.into())
    }

    /// Create a Gzip error
    pub fn gzip(msg: impl Into<String>) -> Self {
        Self::Gzip(msg.into())
    }

    /// Create a Snappy error
    pub fn snappy(msg: impl Into<String>) -> Self {
        Self::Snappy(msg.into())
    }

    /// Create a dictionary error
    pub fn dictionary(msg: impl Into<String>) -> Self {
        Self::Dictionary(msg.into())
    }

    /// Create a corrupted data error
    pub fn corrupted(msg: impl Into<String>) -> Self {
        Self::CorruptedData(msg.into())
    }

    /// Create a pipeline error
    pub fn pipeline(stage: impl Into<String>, error: impl Into<String>) -> Self {
        Self::Pipeline {
            stage: stage.into(),
            error: error.into(),
        }
    }

    /// Create a resource limit error
    pub fn resource_limit(resource: impl Into<String>, limit: impl Into<String>) -> Self {
        Self::ResourceLimit {
            resource: resource.into(),
            limit: limit.into(),
        }
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. }
                | Self::ResourceLimit { .. }
                | Self::Concurrency(_)
                | Self::BufferSize(_)
        )
    }

    /// Check if error indicates data corruption
    pub fn is_corruption(&self) -> bool {
        matches!(
            self,
            Self::CorruptedData(_) | Self::ChecksumMismatch { .. }
        )
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::CorruptedData(_) | Self::ChecksumMismatch { .. } => ErrorSeverity::Critical,
            Self::Io(_) | Self::ResourceLimit { .. } => ErrorSeverity::High,
            Self::Timeout { .. } | Self::Concurrency(_) => ErrorSeverity::Medium,
            _ => ErrorSeverity::Low,
        }
    }
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ErrorSeverity {
    /// Check if error requires immediate attention
    pub fn requires_alert(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = CompressionError::custom("test error", "test context");
        assert!(matches!(err, CompressionError::Custom { .. }));
    }

    #[test]
    fn test_error_recoverable() {
        let timeout = CompressionError::Timeout { seconds: 30 };
        assert!(timeout.is_recoverable());

        let corrupted = CompressionError::corrupted("bad data");
        assert!(!corrupted.is_recoverable());
    }

    #[test]
    fn test_error_severity() {
        let corrupted = CompressionError::corrupted("bad data");
        assert_eq!(corrupted.severity(), ErrorSeverity::Critical);

        let timeout = CompressionError::Timeout { seconds: 30 };
        assert_eq!(timeout.severity(), ErrorSeverity::Medium);
    }

    #[test]
    fn test_corruption_detection() {
        let corrupted = CompressionError::corrupted("bad data");
        assert!(corrupted.is_corruption());

        let checksum = CompressionError::ChecksumMismatch {
            expected: 123,
            actual: 456,
        };
        assert!(checksum.is_corruption());
    }
}
