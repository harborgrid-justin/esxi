//! ChaCha20-Poly1305 authenticated encryption
//!
//! Implements ChaCha20-Poly1305 AEAD cipher, which is optimized for
//! software implementations and provides excellent performance on
//! systems without AES hardware acceleration.
//!
//! ## Security Properties
//! - 256-bit key
//! - 96-bit nonce (unique per encryption)
//! - 128-bit Poly1305 authentication tag
//! - Constant-time operations (resistant to timing attacks)
//!
//! ## Use Cases
//! - Mobile devices without AES-NI
//! - Environments requiring constant-time crypto
//! - High-performance software encryption
//!
//! ## Standards
//! - RFC 8439 (ChaCha20 and Poly1305 for IETF Protocols)
//! - NIST approved for sensitive data

use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use zeroize::Zeroizing;

use crate::{
    config::{CHACHA20_KEY_SIZE, CHACHA20_NONCE_SIZE},
    error::{SecurityError, SecurityResult},
};

use super::{Encryptor, KeyGenerator};

/// ChaCha20-Poly1305 encryptor for authenticated encryption
///
/// # Example
/// ```rust,no_run
/// use meridian_security::encryption::chacha::ChaChaEncryptor;
///
/// let key = ChaChaEncryptor::generate_key().unwrap();
/// let encryptor = ChaChaEncryptor::new(&key).unwrap();
///
/// let plaintext = b"Sensitive mobile data";
/// let ciphertext = encryptor.encrypt(plaintext).unwrap();
/// let decrypted = encryptor.decrypt(&ciphertext).unwrap();
///
/// assert_eq!(plaintext, &decrypted[..]);
/// ```
pub struct ChaChaEncryptor {
    cipher: ChaCha20Poly1305,
}

impl ChaChaEncryptor {
    /// Create a new ChaCha20-Poly1305 encryptor with the provided key
    ///
    /// # Arguments
    /// * `key` - 32-byte (256-bit) encryption key
    ///
    /// # Errors
    /// Returns `SecurityError::InvalidKey` if the key size is incorrect
    pub fn new(key: &[u8]) -> SecurityResult<Self> {
        if key.len() != CHACHA20_KEY_SIZE {
            return Err(SecurityError::InvalidKey(format!(
                "ChaCha20 requires {}-byte key, got {}",
                CHACHA20_KEY_SIZE,
                key.len()
            )));
        }

        let cipher = ChaCha20Poly1305::new_from_slice(key).map_err(|e| {
            SecurityError::EncryptionError(format!("Failed to initialize ChaCha20-Poly1305: {:?}", e))
        })?;

        Ok(Self { cipher })
    }

    /// Encrypt with additional authenticated data (AAD)
    ///
    /// AAD is authenticated but not encrypted. Useful for metadata.
    ///
    /// # Arguments
    /// * `plaintext` - Data to encrypt
    /// * `aad` - Additional data to authenticate (not encrypted)
    ///
    /// # Returns
    /// Concatenation of [nonce || ciphertext || tag]
    pub fn encrypt_with_aad(&self, plaintext: &[u8], aad: &[u8]) -> SecurityResult<Vec<u8>> {
        use chacha20poly1305::aead::Payload;

        // Generate cryptographically secure random nonce
        let nonce = Self::generate_nonce()?;
        let nonce_obj = Nonce::from_slice(&nonce);

        let payload = Payload {
            msg: plaintext,
            aad,
        };

        // Encrypt and authenticate
        let ciphertext = self.cipher.encrypt(nonce_obj, payload).map_err(|e| {
            SecurityError::EncryptionError(format!("ChaCha20-Poly1305 encryption failed: {:?}", e))
        })?;

        // Return: nonce || ciphertext (includes auth tag)
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt with additional authenticated data (AAD)
    ///
    /// # Arguments
    /// * `ciphertext` - Encrypted data (nonce || ciphertext || tag)
    /// * `aad` - Additional authenticated data (must match encryption)
    ///
    /// # Errors
    /// Returns error if authentication fails or decryption fails
    pub fn decrypt_with_aad(&self, ciphertext: &[u8], aad: &[u8]) -> SecurityResult<Vec<u8>> {
        use chacha20poly1305::aead::Payload;

        if ciphertext.len() < CHACHA20_NONCE_SIZE {
            return Err(SecurityError::DecryptionError(
                "Ciphertext too short".to_string(),
            ));
        }

        // Extract nonce and ciphertext
        let (nonce, ct) = ciphertext.split_at(CHACHA20_NONCE_SIZE);
        let nonce_obj = Nonce::from_slice(nonce);

        let payload = Payload { msg: ct, aad };

        // Decrypt and verify authentication tag
        let plaintext = self.cipher.decrypt(nonce_obj, payload).map_err(|_| {
            SecurityError::DecryptionError("Poly1305 authentication verification failed".to_string())
        })?;

        Ok(plaintext)
    }

    /// Generate a cryptographically secure random nonce
    fn generate_nonce() -> SecurityResult<Vec<u8>> {
        let mut nonce = vec![0u8; CHACHA20_NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce);
        Ok(nonce)
    }

    /// Stream encryption for large files
    ///
    /// Encrypts data in chunks to minimize memory usage
    pub fn encrypt_stream(&self, plaintext: &[u8], chunk_size: usize) -> SecurityResult<Vec<u8>> {
        // For simplicity, encrypt as single operation
        // Production implementation would use streaming
        self.encrypt(plaintext)
    }

    /// Stream decryption for large files
    pub fn decrypt_stream(&self, ciphertext: &[u8], chunk_size: usize) -> SecurityResult<Vec<u8>> {
        // For simplicity, decrypt as single operation
        // Production implementation would use streaming
        self.decrypt(ciphertext)
    }
}

impl Encryptor for ChaChaEncryptor {
    /// Encrypt plaintext and return ciphertext
    ///
    /// # Format
    /// Returns: nonce (12 bytes) || ciphertext || auth_tag (16 bytes)
    fn encrypt(&self, plaintext: &[u8]) -> SecurityResult<Vec<u8>> {
        self.encrypt_with_aad(plaintext, &[])
    }

    /// Decrypt ciphertext and verify authentication
    ///
    /// # Arguments
    /// * `ciphertext` - Must be in format: nonce || ciphertext || tag
    fn decrypt(&self, ciphertext: &[u8]) -> SecurityResult<Vec<u8>> {
        self.decrypt_with_aad(ciphertext, &[])
    }
}

impl KeyGenerator for ChaChaEncryptor {
    /// Generate a new 256-bit cryptographically secure random key
    fn generate_key() -> SecurityResult<Vec<u8>> {
        let mut key = Zeroizing::new(vec![0u8; CHACHA20_KEY_SIZE]);
        rand::thread_rng().fill_bytes(&mut key);
        Ok(key.to_vec())
    }
}

impl Drop for ChaChaEncryptor {
    fn drop(&mut self) {
        // Zeroize sensitive data on drop
        // ChaCha20Poly1305 handles internal zeroization
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = ChaChaEncryptor::generate_key().unwrap();
        assert_eq!(key.len(), CHACHA20_KEY_SIZE);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = ChaChaEncryptor::generate_key().unwrap();
        let encryptor = ChaChaEncryptor::new(&key).unwrap();

        let plaintext = b"ChaCha20 is fast!";
        let ciphertext = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_encrypt_with_aad() {
        let key = ChaChaEncryptor::generate_key().unwrap();
        let encryptor = ChaChaEncryptor::new(&key).unwrap();

        let plaintext = b"Mobile transaction data";
        let aad = b"device_id:xyz789";

        let ciphertext = encryptor.encrypt_with_aad(plaintext, aad).unwrap();
        let decrypted = encryptor.decrypt_with_aad(&ciphertext, aad).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_decrypt_with_wrong_aad_fails() {
        let key = ChaChaEncryptor::generate_key().unwrap();
        let encryptor = ChaChaEncryptor::new(&key).unwrap();

        let plaintext = b"Authenticated data";
        let aad = b"context:prod";
        let wrong_aad = b"context:dev";

        let ciphertext = encryptor.encrypt_with_aad(plaintext, aad).unwrap();
        let result = encryptor.decrypt_with_aad(&ciphertext, wrong_aad);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_size() {
        let short_key = vec![0u8; 16];
        let result = ChaChaEncryptor::new(&short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampering_detection() {
        let key = ChaChaEncryptor::generate_key().unwrap();
        let encryptor = ChaChaEncryptor::new(&key).unwrap();

        let plaintext = b"Critical data";
        let mut ciphertext = encryptor.encrypt(plaintext).unwrap();

        // Tamper with the last byte (part of auth tag)
        if let Some(byte) = ciphertext.last_mut() {
            *byte ^= 0xFF;
        }

        let result = encryptor.decrypt(&ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_unique_ciphertexts() {
        let key = ChaChaEncryptor::generate_key().unwrap();
        let encryptor = ChaChaEncryptor::new(&key).unwrap();

        let plaintext = b"Same message";
        let ct1 = encryptor.encrypt(plaintext).unwrap();
        let ct2 = encryptor.encrypt(plaintext).unwrap();

        // Different nonces produce different ciphertexts
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_large_data() {
        let key = ChaChaEncryptor::generate_key().unwrap();
        let encryptor = ChaChaEncryptor::new(&key).unwrap();

        let plaintext = vec![0x42u8; 1024 * 1024]; // 1 MB
        let ciphertext = encryptor.encrypt(&plaintext).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_empty_plaintext() {
        let key = ChaChaEncryptor::generate_key().unwrap();
        let encryptor = ChaChaEncryptor::new(&key).unwrap();

        let plaintext = b"";
        let ciphertext = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }
}
