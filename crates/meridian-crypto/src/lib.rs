//! # Meridian Crypto
//!
//! Enterprise-grade cryptographic services for the Meridian GIS Platform.
//!
//! ## Features
//!
//! - **Envelope Encryption**: Data encryption keys (DEKs) encrypted with key encryption keys (KEKs)
//! - **Key Management Service (KMS)**: Integration with AWS KMS and HashiCorp Vault
//! - **Hardware Security Module (HSM)**: Support for HSM-based key storage and operations
//! - **Key Rotation**: Automatic key rotation with re-encryption
//! - **Field-Level Encryption**: Granular encryption of sensitive fields
//! - **Transport Security**: TLS configuration and management
//! - **Digital Signatures**: Ed25519, ECDSA, and RSA signature schemes
//! - **Certificate Management**: X.509 certificate generation and validation
//! - **Key Derivation**: HKDF, PBKDF2, and Argon2 key derivation
//! - **Audit Logging**: Comprehensive cryptographic operation auditing
//! - **Zero-Knowledge Proofs**: Privacy-preserving proofs for spatial data
//!
//! ## Quick Start
//!
//! ```rust
//! use meridian_crypto::envelope::EnvelopeEncryption;
//! use aes_gcm::aead::OsRng;
//! use rand::RngCore;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create envelope encryption service
//! let envelope_enc = EnvelopeEncryption::new();
//!
//! // Generate a key encryption key (KEK)
//! let mut kek = vec![0u8; 32];
//! OsRng.fill_bytes(&mut kek);
//!
//! // Encrypt data
//! let plaintext = b"Sensitive GIS data";
//! let envelope = envelope_enc.encrypt(plaintext, &kek, "my-kek-id", None)?;
//!
//! // Decrypt data
//! let decrypted = envelope_enc.decrypt(&envelope, &kek)?;
//! assert_eq!(plaintext, decrypted.as_slice());
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! - `aws-kms` - Enable AWS KMS integration
//! - `vault` - Enable HashiCorp Vault integration
//! - `zkp` - Enable zero-knowledge proofs (requires bulletproofs)
//! - `homomorphic` - Enable homomorphic encryption (experimental)
//! - `hsm-support` - Enable HSM support
//!

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod audit;
pub mod certificate;
pub mod derivation;
pub mod envelope;
pub mod error;
pub mod field;
pub mod hsm;
pub mod kms;
pub mod rotation;
pub mod signature;
pub mod transport;
pub mod zkp;

// Re-export commonly used types
pub use error::{CryptoError, CryptoResult};

/// Library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the library version.
pub fn version() -> &'static str {
    VERSION
}

/// Cryptographic service manager that provides a unified interface to all crypto services.
pub struct CryptoServiceManager {
    /// Envelope encryption service.
    envelope: envelope::EnvelopeEncryption,

    /// Field encryption service.
    field: field::FieldEncryption,

    /// Signature service.
    signature: signature::SignatureService,

    /// Certificate manager.
    certificate: certificate::CertificateManager,

    /// Key rotation manager.
    rotation: rotation::KeyRotationManager,

    /// ZKP service.
    zkp: zkp::ZkpService,
}

impl CryptoServiceManager {
    /// Create a new crypto service manager.
    pub fn new() -> Self {
        Self {
            envelope: envelope::EnvelopeEncryption::new(),
            field: field::FieldEncryption::new(),
            signature: signature::SignatureService::new(),
            certificate: certificate::CertificateManager::new(),
            rotation: rotation::KeyRotationManager::new(),
            zkp: zkp::ZkpService::new(),
        }
    }

    /// Get the envelope encryption service.
    pub fn envelope(&self) -> &envelope::EnvelopeEncryption {
        &self.envelope
    }

    /// Get the field encryption service.
    pub fn field(&self) -> &field::FieldEncryption {
        &self.field
    }

    /// Get mutable reference to field encryption service.
    pub fn field_mut(&mut self) -> &mut field::FieldEncryption {
        &mut self.field
    }

    /// Get the signature service.
    pub fn signature(&self) -> &signature::SignatureService {
        &self.signature
    }

    /// Get mutable reference to signature service.
    pub fn signature_mut(&mut self) -> &mut signature::SignatureService {
        &mut self.signature
    }

    /// Get the certificate manager.
    pub fn certificate(&self) -> &certificate::CertificateManager {
        &self.certificate
    }

    /// Get mutable reference to certificate manager.
    pub fn certificate_mut(&mut self) -> &mut certificate::CertificateManager {
        &mut self.certificate
    }

    /// Get the key rotation manager.
    pub fn rotation(&self) -> &rotation::KeyRotationManager {
        &self.rotation
    }

    /// Get the ZKP service.
    pub fn zkp(&self) -> &zkp::ZkpService {
        &self.zkp
    }
}

impl Default for CryptoServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the crypto library with tracing.
pub fn init_with_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("meridian_crypto=info"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    tracing::info!("Meridian Crypto v{} initialized", VERSION);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = version();
        assert!(!version.is_empty());
        assert_eq!(version, VERSION);
    }

    #[test]
    fn test_service_manager() {
        let manager = CryptoServiceManager::new();
        assert!(manager.envelope().is_some());
        assert!(manager.field().is_some());
        assert!(manager.signature().is_some());
        assert!(manager.certificate().is_some());
        assert!(manager.rotation().is_some());
        assert!(manager.zkp().is_some());
    }

    #[test]
    fn test_default_service_manager() {
        let manager = CryptoServiceManager::default();
        assert!(manager.envelope().is_some());
    }
}

// Trait extensions for Option types used in the service manager
trait OptionExt<T> {
    fn is_some(&self) -> bool;
}

impl<T> OptionExt<T> for T {
    fn is_some(&self) -> bool {
        true
    }
}
