//! Encryption algorithms and envelope encryption
//!
//! This module provides enterprise-grade encryption capabilities:
//! - AES-256-GCM for high-performance authenticated encryption
//! - ChaCha20-Poly1305 for software-optimized authenticated encryption
//! - Envelope encryption with key hierarchy for data-at-rest protection
//!
//! All implementations follow NIST and OWASP cryptographic standards.

pub mod aes;
pub mod chacha;
pub mod envelope;

use crate::error::SecurityResult;

/// Trait for symmetric encryption operations
pub trait Encryptor {
    /// Encrypt plaintext and return ciphertext with authentication tag
    fn encrypt(&self, plaintext: &[u8]) -> SecurityResult<Vec<u8>>;

    /// Decrypt ciphertext and verify authentication tag
    fn decrypt(&self, ciphertext: &[u8]) -> SecurityResult<Vec<u8>>;
}

/// Trait for key generation
pub trait KeyGenerator {
    /// Generate a new cryptographically secure random key
    fn generate_key() -> SecurityResult<Vec<u8>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_module_compiles() {
        // Module structure test
        assert!(true);
    }
}
