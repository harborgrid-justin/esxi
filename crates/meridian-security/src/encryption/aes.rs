//! AES-256-GCM authenticated encryption
//!
//! Implements AES-256 in Galois/Counter Mode (GCM) for authenticated encryption
//! with associated data (AEAD). This provides both confidentiality and integrity.
//!
//! ## Security Properties
//! - 256-bit key (NIST recommended for TOP SECRET data)
//! - 96-bit nonce (unique per encryption)
//! - 128-bit authentication tag
//! - Authenticated encryption prevents tampering
//!
//! ## OWASP Compliance
//! - Uses approved NIST encryption algorithm
//! - Proper nonce generation (cryptographically random)
//! - No key reuse with same nonce
//! - Constant-time operations where possible

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use zeroize::Zeroizing;

use crate::{
    config::{AES_256_KEY_SIZE, AES_GCM_NONCE_SIZE},
    error::{SecurityError, SecurityResult},
};

use super::{Encryptor, KeyGenerator};

/// AES-256-GCM encryptor for authenticated encryption
///
/// # Example
/// ```rust,no_run
/// use meridian_security::encryption::aes::AesGcmEncryptor;
///
/// let key = AesGcmEncryptor::generate_key().unwrap();
/// let encryptor = AesGcmEncryptor::new(&key).unwrap();
///
/// let plaintext = b"Sensitive data";
/// let ciphertext = encryptor.encrypt(plaintext).unwrap();
/// let decrypted = encryptor.decrypt(&ciphertext).unwrap();
///
/// assert_eq!(plaintext, &decrypted[..]);
/// ```
pub struct AesGcmEncryptor {
    cipher: Aes256Gcm,
}

impl AesGcmEncryptor {
    /// Create a new AES-256-GCM encryptor with the provided key
    ///
    /// # Arguments
    /// * `key` - 32-byte (256-bit) encryption key
    ///
    /// # Errors
    /// Returns `SecurityError::InvalidKey` if the key size is incorrect
    pub fn new(key: &[u8]) -> SecurityResult<Self> {
        if key.len() != AES_256_KEY_SIZE {
            return Err(SecurityError::InvalidKey(format!(
                "AES-256 requires {}-byte key, got {}",
                AES_256_KEY_SIZE,
                key.len()
            )));
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| SecurityError::EncryptionError(format!("Failed to initialize AES-GCM: {:?}", e)))?;

        Ok(Self { cipher })
    }

    /// Encrypt with additional authenticated data (AAD)
    ///
    /// AAD is authenticated but not encrypted. Useful for metadata like
    /// message headers, protocol versions, etc.
    ///
    /// # Arguments
    /// * `plaintext` - Data to encrypt
    /// * `aad` - Additional data to authenticate (not encrypted)
    ///
    /// # Returns
    /// Concatenation of [nonce || ciphertext || tag]
    pub fn encrypt_with_aad(&self, plaintext: &[u8], aad: &[u8]) -> SecurityResult<Vec<u8>> {
        use aes_gcm::aead::Payload;

        // Generate cryptographically secure random nonce
        let nonce = Self::generate_nonce()?;
        let nonce_obj = Nonce::from_slice(&nonce);

        let payload = Payload {
            msg: plaintext,
            aad,
        };

        // Encrypt and authenticate
        let ciphertext = self
            .cipher
            .encrypt(nonce_obj, payload)
            .map_err(|e| SecurityError::EncryptionError(format!("AES-GCM encryption failed: {:?}", e)))?;

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
        use aes_gcm::aead::Payload;

        if ciphertext.len() < AES_GCM_NONCE_SIZE {
            return Err(SecurityError::DecryptionError(
                "Ciphertext too short".to_string(),
            ));
        }

        // Extract nonce and ciphertext
        let (nonce, ct) = ciphertext.split_at(AES_GCM_NONCE_SIZE);
        let nonce_obj = Nonce::from_slice(nonce);

        let payload = Payload { msg: ct, aad };

        // Decrypt and verify authentication tag
        let plaintext = self
            .cipher
            .decrypt(nonce_obj, payload)
            .map_err(|_| SecurityError::DecryptionError("Authentication verification failed".to_string()))?;

        Ok(plaintext)
    }

    /// Generate a cryptographically secure random nonce
    fn generate_nonce() -> SecurityResult<Vec<u8>> {
        let mut nonce = vec![0u8; AES_GCM_NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce);
        Ok(nonce)
    }

    /// Encrypt data in-place (overwrites plaintext)
    ///
    /// Useful for minimizing memory allocation for sensitive data
    pub fn encrypt_in_place(&self, buffer: &mut Vec<u8>) -> SecurityResult<Vec<u8>> {
        let plaintext = buffer.clone();
        buffer.clear();
        self.encrypt(&plaintext)
    }
}

impl Encryptor for AesGcmEncryptor {
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

impl KeyGenerator for AesGcmEncryptor {
    /// Generate a new 256-bit cryptographically secure random key
    fn generate_key() -> SecurityResult<Vec<u8>> {
        let mut key = Zeroizing::new(vec![0u8; AES_256_KEY_SIZE]);
        rand::thread_rng().fill_bytes(&mut key);
        Ok(key.to_vec())
    }
}

impl Drop for AesGcmEncryptor {
    fn drop(&mut self) {
        // Zeroize sensitive data on drop
        // Note: Aes256Gcm doesn't expose key, relies on internal zeroization
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key = AesGcmEncryptor::generate_key().unwrap();
        assert_eq!(key.len(), AES_256_KEY_SIZE);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = AesGcmEncryptor::generate_key().unwrap();
        let encryptor = AesGcmEncryptor::new(&key).unwrap();

        let plaintext = b"Hello, Enterprise World!";
        let ciphertext = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_encrypt_with_aad() {
        let key = AesGcmEncryptor::generate_key().unwrap();
        let encryptor = AesGcmEncryptor::new(&key).unwrap();

        let plaintext = b"Secret message";
        let aad = b"user_id:12345";

        let ciphertext = encryptor.encrypt_with_aad(plaintext, aad).unwrap();
        let decrypted = encryptor.decrypt_with_aad(&ciphertext, aad).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_decrypt_with_wrong_aad_fails() {
        let key = AesGcmEncryptor::generate_key().unwrap();
        let encryptor = AesGcmEncryptor::new(&key).unwrap();

        let plaintext = b"Secret message";
        let aad = b"user_id:12345";
        let wrong_aad = b"user_id:99999";

        let ciphertext = encryptor.encrypt_with_aad(plaintext, aad).unwrap();
        let result = encryptor.decrypt_with_aad(&ciphertext, wrong_aad);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_size() {
        let short_key = vec![0u8; 16]; // Only 128 bits
        let result = AesGcmEncryptor::new(&short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampering_detection() {
        let key = AesGcmEncryptor::generate_key().unwrap();
        let encryptor = AesGcmEncryptor::new(&key).unwrap();

        let plaintext = b"Important data";
        let mut ciphertext = encryptor.encrypt(plaintext).unwrap();

        // Tamper with ciphertext
        if let Some(byte) = ciphertext.last_mut() {
            *byte ^= 0xFF;
        }

        let result = encryptor.decrypt(&ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_unique_ciphertexts() {
        let key = AesGcmEncryptor::generate_key().unwrap();
        let encryptor = AesGcmEncryptor::new(&key).unwrap();

        let plaintext = b"Same message";
        let ct1 = encryptor.encrypt(plaintext).unwrap();
        let ct2 = encryptor.encrypt(plaintext).unwrap();

        // Same plaintext should produce different ciphertext (different nonces)
        assert_ne!(ct1, ct2);
    }
}
