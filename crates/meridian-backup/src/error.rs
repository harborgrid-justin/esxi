//! Error types for the Meridian backup system.

use thiserror::Error;

/// Main error type for backup operations.
#[derive(Error, Debug)]
pub enum BackupError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Encryption error: {0}")]
    Encryption(#[from] EncryptionError),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Verification failed: {0}")]
    VerificationFailed(String),

    #[error("Backup not found: {0}")]
    BackupNotFound(String),

    #[error("Invalid backup state: {0}")]
    InvalidState(String),

    #[error("Scheduling error: {0}")]
    Scheduling(String),

    #[error("Replication error: {0}")]
    Replication(String),

    #[error("Recovery error: {0}")]
    Recovery(String),

    #[error("Retention policy violation: {0}")]
    RetentionViolation(String),

    #[error("Failover error: {0}")]
    Failover(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Time parsing error: {0}")]
    TimeParsing(String),

    #[error("Invalid backup format: {0}")]
    InvalidFormat(String),

    #[error("Deduplication error: {0}")]
    Deduplication(String),

    #[error("RTO/RPO violation: metric={0}, expected={1}, actual={2}")]
    SlaViolation(String, String, String),

    #[error("Concurrent operation conflict: {0}")]
    ConcurrencyConflict(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Storage backend specific errors.
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("S3 error: {0}")]
    S3(String),

    #[error("GCS error: {0}")]
    Gcs(String),

    #[error("Azure error: {0}")]
    Azure(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("Bucket not found: {0}")]
    BucketNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Encryption related errors.
#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Invalid nonce: {0}")]
    InvalidNonce(String),

    #[error("Authentication tag mismatch")]
    AuthenticationFailed,

    #[error("Unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
}

/// Result type alias for backup operations.
pub type Result<T> = std::result::Result<T, BackupError>;

/// Result type alias for storage operations.
pub type StorageResult<T> = std::result::Result<T, StorageError>;

/// Result type alias for encryption operations.
pub type EncryptionResult<T> = std::result::Result<T, EncryptionError>;
