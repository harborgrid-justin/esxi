//! # Meridian Security
//!
//! Enterprise-grade security module for $983M SaaS Platform v0.5
//!
//! ## Features
//!
//! - **Encryption**: AES-256-GCM, ChaCha20-Poly1305, Envelope encryption
//! - **Key Management**: Secure keyring, key rotation, key derivation (KDF)
//! - **Hashing**: Argon2id password hashing, HMAC message authentication
//! - **Tokens**: JWT creation/validation, refresh token management
//! - **Zero Trust**: Policy-based access control, context-aware security
//! - **Audit**: Comprehensive security event logging
//! - **Secrets**: Secure secrets management and vault operations
//!
//! ## OWASP Compliance
//!
//! This module implements security controls aligned with:
//! - OWASP Top 10 2021
//! - OWASP ASVS (Application Security Verification Standard) Level 3
//! - NIST Cryptographic Standards
//! - SOC 2 Type II requirements
//!
//! ## Usage
//!
//! ```rust,no_run
//! use meridian_security::{
//!     encryption::aes::AesGcmEncryptor,
//!     hashing::password::PasswordHasher,
//!     tokens::jwt::JwtManager,
//!     zero_trust::policy::PolicyEngine,
//! };
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Encryption algorithms and envelope encryption
pub mod encryption;

/// Key Management System (KMS) - storage, rotation, derivation
pub mod kms;

/// Secure hashing - passwords, HMAC
pub mod hashing;

/// Token management - JWT, refresh tokens
pub mod tokens;

/// Zero-trust security architecture
pub mod zero_trust;

/// Security audit logging
pub mod audit;

/// Secrets management and vault operations
pub mod secrets;

/// Common error types for the security module
pub mod error;

/// Security-related constants and configurations
pub mod config;

pub use error::{SecurityError, SecurityResult};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::encryption::{aes::AesGcmEncryptor, chacha::ChaChaEncryptor, envelope::EnvelopeEncryption};
    pub use crate::error::{SecurityError, SecurityResult};
    pub use crate::hashing::{hmac::HmacVerifier, password::PasswordHasher};
    pub use crate::kms::{derivation::KeyDerivation, keyring::Keyring};
    pub use crate::secrets::vault::SecretsVault;
    pub use crate::tokens::{jwt::JwtManager, refresh::RefreshTokenManager};
    pub use crate::zero_trust::{context::SecurityContext, policy::PolicyEngine};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_structure() {
        // Ensure all modules compile
        assert!(true);
    }
}
