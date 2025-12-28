//! Error types for the Meridian cache system

use thiserror::Error;

/// Main error type for cache operations
#[derive(Error, Debug)]
pub enum CacheError {
    /// Key not found in cache
    #[error("Cache key not found: {0}")]
    KeyNotFound(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    /// Redis connection error
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Compression error
    #[error("Compression error: {0}")]
    Compression(String),

    /// Decompression error
    #[error("Decompression error: {0}")]
    Decompression(String),

    /// Cache backend error
    #[error("Backend error: {0}")]
    Backend(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Cache coherence error
    #[error("Cache coherence error: {0}")]
    Coherence(String),

    /// Invalidation error
    #[error("Invalidation error: {0}")]
    Invalidation(String),

    /// Warmup error
    #[error("Cache warmup error: {0}")]
    Warmup(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Capacity exceeded
    #[error("Cache capacity exceeded")]
    CapacityExceeded,

    /// Invalid key format
    #[error("Invalid key format: {0}")]
    InvalidKey(String),

    /// Invalid value format
    #[error("Invalid value format: {0}")]
    InvalidValue(String),

    /// Connection pool exhausted
    #[error("Connection pool exhausted")]
    PoolExhausted,

    /// Generic error
    #[error("Cache error: {0}")]
    Other(String),
}

/// Result type alias for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

impl CacheError {
    /// Check if the error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            CacheError::Timeout
                | CacheError::PoolExhausted
                | CacheError::Redis(_)
                | CacheError::Backend(_)
        )
    }

    /// Check if the error indicates a missing key
    pub fn is_not_found(&self) -> bool {
        matches!(self, CacheError::KeyNotFound(_))
    }
}
