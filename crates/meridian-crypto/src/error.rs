//! Error types for the Meridian cryptographic library.
//!
//! This module provides comprehensive error handling for all cryptographic operations,
//! including encryption, key management, HSM operations, and audit logging.

use std::fmt;
use thiserror::Error;

/// Result type alias for cryptographic operations.
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Main error type for all cryptographic operations.
#[derive(Error, Debug)]
pub enum CryptoError {
    /// Encryption operation failed.
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    /// Decryption operation failed.
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Key generation failed.
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    /// Key derivation failed.
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    /// Invalid key format or size.
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Key not found in the key store.
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Key rotation operation failed.
    #[error("Key rotation failed: {0}")]
    KeyRotationFailed(String),

    /// KMS operation failed.
    #[error("KMS operation failed: {0}")]
    KmsOperationFailed(String),

    /// AWS KMS specific error.
    #[error("AWS KMS error: {0}")]
    AwsKmsError(String),

    /// HashiCorp Vault specific error.
    #[error("Vault error: {0}")]
    VaultError(String),

    /// HSM operation failed.
    #[error("HSM operation failed: {0}")]
    HsmOperationFailed(String),

    /// HSM not available or not configured.
    #[error("HSM not available: {0}")]
    HsmNotAvailable(String),

    /// Digital signature generation failed.
    #[error("Signature generation failed: {0}")]
    SignatureFailed(String),

    /// Signature verification failed.
    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),

    /// Certificate operation failed.
    #[error("Certificate error: {0}")]
    CertificateError(String),

    /// Certificate validation failed.
    #[error("Certificate validation failed: {0}")]
    CertificateValidationFailed(String),

    /// Certificate has expired.
    #[error("Certificate expired: {0}")]
    CertificateExpired(String),

    /// Invalid certificate chain.
    #[error("Invalid certificate chain: {0}")]
    InvalidCertificateChain(String),

    /// TLS/Transport encryption error.
    #[error("Transport encryption error: {0}")]
    TransportError(String),

    /// Field-level encryption error.
    #[error("Field encryption error: {0}")]
    FieldEncryptionError(String),

    /// Audit logging failed.
    #[error("Audit logging failed: {0}")]
    AuditLogError(String),

    /// Zero-knowledge proof generation failed.
    #[error("ZKP generation failed: {0}")]
    ZkpGenerationFailed(String),

    /// Zero-knowledge proof verification failed.
    #[error("ZKP verification failed: {0}")]
    ZkpVerificationFailed(String),

    /// Homomorphic encryption operation failed.
    #[error("Homomorphic encryption error: {0}")]
    HomomorphicError(String),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Invalid input data.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Encoding/decoding error.
    #[error("Encoding error: {0}")]
    EncodingError(String),

    /// I/O operation failed.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Network/HTTP error.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Timeout occurred.
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Permission denied.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Resource not available.
    #[error("Resource unavailable: {0}")]
    ResourceUnavailable(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Unsupported operation.
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

impl CryptoError {
    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            CryptoError::NetworkError(_)
                | CryptoError::Timeout(_)
                | CryptoError::ResourceUnavailable(_)
                | CryptoError::KmsOperationFailed(_)
        )
    }

    /// Check if this error is related to authentication/authorization.
    pub fn is_auth_error(&self) -> bool {
        matches!(self, CryptoError::PermissionDenied(_))
    }

    /// Check if this error indicates a key-related problem.
    pub fn is_key_error(&self) -> bool {
        matches!(
            self,
            CryptoError::InvalidKey(_)
                | CryptoError::KeyNotFound(_)
                | CryptoError::KeyGenerationFailed(_)
                | CryptoError::KeyDerivationFailed(_)
        )
    }
}

impl From<ring::error::Unspecified> for CryptoError {
    fn from(err: ring::error::Unspecified) -> Self {
        CryptoError::InternalError(format!("Ring cryptography error: {:?}", err))
    }
}

impl From<aes_gcm::Error> for CryptoError {
    fn from(err: aes_gcm::Error) -> Self {
        CryptoError::EncryptionFailed(format!("AES-GCM error: {:?}", err))
    }
}

impl From<base64::DecodeError> for CryptoError {
    fn from(err: base64::DecodeError) -> Self {
        CryptoError::EncodingError(format!("Base64 decode error: {}", err))
    }
}

impl From<hex::FromHexError> for CryptoError {
    fn from(err: hex::FromHexError) -> Self {
        CryptoError::EncodingError(format!("Hex decode error: {}", err))
    }
}

impl From<serde_json::Error> for CryptoError {
    fn from(err: serde_json::Error) -> Self {
        CryptoError::SerializationError(format!("JSON error: {}", err))
    }
}

impl From<reqwest::Error> for CryptoError {
    fn from(err: reqwest::Error) -> Self {
        CryptoError::NetworkError(format!("HTTP request error: {}", err))
    }
}

#[cfg(feature = "aws-kms")]
impl<E> From<aws_sdk_kms::error::SdkError<E>> for CryptoError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: aws_sdk_kms::error::SdkError<E>) -> Self {
        CryptoError::AwsKmsError(format!("AWS SDK error: {}", err))
    }
}

#[cfg(feature = "vault")]
impl From<vaultrs::error::ClientError> for CryptoError {
    fn from(err: vaultrs::error::ClientError) -> Self {
        CryptoError::VaultError(format!("Vault client error: {}", err))
    }
}

/// Convenience macro for creating crypto errors.
#[macro_export]
macro_rules! crypto_error {
    ($kind:ident, $msg:expr) => {
        $crate::error::CryptoError::$kind($msg.to_string())
    };
    ($kind:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::error::CryptoError::$kind(format!($fmt, $($arg)*))
    };
}
