//! Encryption and decryption for backup data.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use bytes::Bytes;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{EncryptionError, EncryptionResult};

/// Encryption algorithm types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

/// Encryption key derivation method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyDerivation {
    Argon2,
    Pbkdf2,
    Scrypt,
}

/// Encryption configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub key_derivation: KeyDerivation,
    pub key_size: usize,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            key_derivation: KeyDerivation::Argon2,
            key_size: 32,
        }
    }
}

/// Encrypted data container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub algorithm: EncryptionAlgorithm,
    pub metadata: Vec<u8>,
}

/// Encryption key manager.
pub struct EncryptionManager {
    config: EncryptionConfig,
    master_key: Vec<u8>,
}

impl EncryptionManager {
    /// Create a new encryption manager with a master key.
    pub fn new(config: EncryptionConfig, master_key: Vec<u8>) -> EncryptionResult<Self> {
        if master_key.len() != config.key_size {
            return Err(EncryptionError::InvalidKey(format!(
                "Expected key size {}, got {}",
                config.key_size,
                master_key.len()
            )));
        }

        Ok(Self { config, master_key })
    }

    /// Create a new encryption manager from a password.
    pub fn from_password(config: EncryptionConfig, password: &str) -> EncryptionResult<Self> {
        let key = Self::derive_key_from_password(password, &config)?;
        Self::new(config, key)
    }

    /// Generate a random encryption key.
    pub fn generate_key(size: usize) -> Vec<u8> {
        let mut key = vec![0u8; size];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Derive a key from a password using the configured method.
    fn derive_key_from_password(
        password: &str,
        config: &EncryptionConfig,
    ) -> EncryptionResult<Vec<u8>> {
        match config.key_derivation {
            KeyDerivation::Argon2 => {
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();

                let password_hash = argon2
                    .hash_password(password.as_bytes(), &salt)
                    .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;

                let hash = password_hash.hash.ok_or_else(|| {
                    EncryptionError::KeyDerivationFailed("No hash generated".to_string())
                })?;

                let hash_bytes = hash.as_bytes();
                if hash_bytes.len() >= config.key_size {
                    Ok(hash_bytes[..config.key_size].to_vec())
                } else {
                    // Extend with SHA-256 if needed
                    let mut hasher = Sha256::new();
                    hasher.update(hash_bytes);
                    let extended = hasher.finalize();
                    Ok(extended[..config.key_size].to_vec())
                }
            }
            KeyDerivation::Pbkdf2 | KeyDerivation::Scrypt => {
                // Simplified implementation - use Argon2 as fallback
                Self::derive_key_from_password(
                    password,
                    &EncryptionConfig {
                        key_derivation: KeyDerivation::Argon2,
                        ..config.clone()
                    },
                )
            }
        }
    }

    /// Encrypt data.
    pub fn encrypt(&self, plaintext: &[u8]) -> EncryptionResult<EncryptedData> {
        match self.config.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.encrypt_aes_gcm(plaintext),
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                // Fallback to AES-GCM for now
                self.encrypt_aes_gcm(plaintext)
            }
        }
    }

    /// Decrypt data.
    pub fn decrypt(&self, encrypted: &EncryptedData) -> EncryptionResult<Vec<u8>> {
        match encrypted.algorithm {
            EncryptionAlgorithm::Aes256Gcm => self.decrypt_aes_gcm(encrypted),
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                // Fallback to AES-GCM for now
                self.decrypt_aes_gcm(encrypted)
            }
        }
    }

    /// Encrypt using AES-256-GCM.
    fn encrypt_aes_gcm(&self, plaintext: &[u8]) -> EncryptionResult<EncryptedData> {
        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedData {
            nonce: nonce_bytes.to_vec(),
            ciphertext,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
            metadata: Vec::new(),
        })
    }

    /// Decrypt using AES-256-GCM.
    fn decrypt_aes_gcm(&self, encrypted: &EncryptedData) -> EncryptionResult<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        if encrypted.nonce.len() != 12 {
            return Err(EncryptionError::InvalidNonce(
                "Nonce must be 12 bytes".to_string(),
            ));
        }

        let nonce = Nonce::from_slice(&encrypted.nonce);

        // Decrypt the data
        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|_| EncryptionError::AuthenticationFailed)?;

        Ok(plaintext)
    }

    /// Calculate SHA-256 hash of data.
    pub fn hash(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Verify data against a hash.
    pub fn verify_hash(&self, data: &[u8], expected_hash: &[u8]) -> bool {
        let actual_hash = self.hash(data);
        actual_hash == expected_hash
    }
}

/// Encrypt bytes with a given key.
pub fn encrypt_bytes(key: &[u8], data: &[u8]) -> EncryptionResult<EncryptedData> {
    let config = EncryptionConfig::default();
    let manager = EncryptionManager::new(config, key.to_vec())?;
    manager.encrypt(data)
}

/// Decrypt bytes with a given key.
pub fn decrypt_bytes(key: &[u8], encrypted: &EncryptedData) -> EncryptionResult<Vec<u8>> {
    let config = EncryptionConfig::default();
    let manager = EncryptionManager::new(config, key.to_vec())?;
    manager.decrypt(encrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let config = EncryptionConfig::default();
        let key = EncryptionManager::generate_key(32);
        let manager = EncryptionManager::new(config, key).unwrap();

        let plaintext = b"Hello, Meridian!";
        let encrypted = manager.encrypt(plaintext).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_password_derivation() {
        let config = EncryptionConfig::default();
        let manager = EncryptionManager::from_password(config, "test-password").unwrap();

        let plaintext = b"Secret data";
        let encrypted = manager.encrypt(plaintext).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_hash_verification() {
        let config = EncryptionConfig::default();
        let key = EncryptionManager::generate_key(32);
        let manager = EncryptionManager::new(config, key).unwrap();

        let data = b"Data to hash";
        let hash = manager.hash(data);

        assert!(manager.verify_hash(data, &hash));
        assert!(!manager.verify_hash(b"Different data", &hash));
    }
}
