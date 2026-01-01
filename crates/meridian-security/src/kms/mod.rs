//! Key Management System (KMS)
//!
//! Enterprise-grade key management including:
//! - Secure key storage (keyring)
//! - Key rotation with versioning
//! - Key derivation functions (KDF)
//! - Key lifecycle management
//!
//! ## NIST SP 800-57 Compliance
//! - Key generation using approved algorithms
//! - Key storage with proper access controls
//! - Key rotation (90-day intervals recommended)
//! - Key destruction with secure erasure

pub mod derivation;
pub mod keyring;

pub use derivation::KeyDerivation;
pub use keyring::{KeyMetadata, Keyring};
