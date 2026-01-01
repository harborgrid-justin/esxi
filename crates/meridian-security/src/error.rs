//! Error types for the security module

use thiserror::Error;

/// Result type alias for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Comprehensive error types for security operations
#[derive(Error, Debug)]
pub enum SecurityError {
    /// Encryption or decryption failed
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// Decryption failed
    #[error("Decryption error: {0}")]
    DecryptionError(String),

    /// Key derivation failed
    #[error("Key derivation error: {0}")]
    KeyDerivationError(String),

    /// Invalid key size or format
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Key not found in keyring
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Password hashing failed
    #[error("Password hashing error: {0}")]
    PasswordHashError(String),

    /// Password verification failed
    #[error("Password verification failed")]
    PasswordVerificationFailed,

    /// HMAC verification failed
    #[error("HMAC verification failed")]
    HmacVerificationFailed,

    /// JWT token error
    #[error("JWT error: {0}")]
    JwtError(String),

    /// Token expired
    #[error("Token expired")]
    TokenExpired,

    /// Invalid token
    #[error("Invalid token: {0}")]
    InvalidToken(String),

    /// Refresh token error
    #[error("Refresh token error: {0}")]
    RefreshTokenError(String),

    /// Policy evaluation failed
    #[error("Policy evaluation error: {0}")]
    PolicyError(String),

    /// Access denied by policy
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// Security context invalid
    #[error("Invalid security context: {0}")]
    InvalidContext(String),

    /// Audit logging failed
    #[error("Audit error: {0}")]
    AuditError(String),

    /// Secrets vault error
    #[error("Vault error: {0}")]
    VaultError(String),

    /// Secret not found
    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Invalid configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Cryptographic operation failed
    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Internal error
    #[error("Internal security error: {0}")]
    InternalError(String),
}

// Convert from common error types
impl From<ring::error::Unspecified> for SecurityError {
    fn from(e: ring::error::Unspecified) -> Self {
        SecurityError::CryptoError(format!("Ring error: {:?}", e))
    }
}

impl From<aes_gcm::Error> for SecurityError {
    fn from(e: aes_gcm::Error) -> Self {
        SecurityError::EncryptionError(format!("AES-GCM error: {:?}", e))
    }
}

impl From<chacha20poly1305::Error> for SecurityError {
    fn from(e: chacha20poly1305::Error) -> Self {
        SecurityError::EncryptionError(format!("ChaCha20 error: {:?}", e))
    }
}

impl From<argon2::Error> for SecurityError {
    fn from(e: argon2::Error) -> Self {
        SecurityError::PasswordHashError(format!("Argon2 error: {:?}", e))
    }
}

impl From<jsonwebtoken::errors::Error> for SecurityError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        SecurityError::JwtError(format!("{}", e))
    }
}

impl From<serde_json::Error> for SecurityError {
    fn from(e: serde_json::Error) -> Self {
        SecurityError::SerializationError(format!("{}", e))
    }
}

impl From<base64::DecodeError> for SecurityError {
    fn from(e: base64::DecodeError) -> Self {
        SecurityError::CryptoError(format!("Base64 decode error: {}", e))
    }
}
