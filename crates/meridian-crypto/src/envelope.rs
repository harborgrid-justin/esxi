//! Envelope encryption implementation for data encryption keys (DEKs).
//!
//! Envelope encryption is a practice of encrypting plaintext data with a data encryption key (DEK)
//! and then encrypting the DEK with a key encryption key (KEK). This provides an additional layer
//! of security and enables key rotation without re-encrypting all data.

use crate::error::{CryptoError, CryptoResult};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Size of the data encryption key in bytes (256 bits).
const DEK_SIZE: usize = 32;

/// Size of the nonce for AES-GCM (96 bits).
const NONCE_SIZE: usize = 12;

/// Data encryption key (DEK) used for encrypting actual data.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct DataEncryptionKey {
    #[zeroize(skip)]
    pub id: String,
    pub key_material: Vec<u8>,
    #[zeroize(skip)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl DataEncryptionKey {
    /// Generate a new random data encryption key.
    pub fn generate() -> CryptoResult<Self> {
        let mut key_material = vec![0u8; DEK_SIZE];
        OsRng.fill_bytes(&mut key_material);

        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            key_material,
            created_at: chrono::Utc::now(),
        })
    }

    /// Create a DEK from existing key material.
    pub fn from_bytes(id: String, key_material: Vec<u8>) -> CryptoResult<Self> {
        if key_material.len() != DEK_SIZE {
            return Err(CryptoError::InvalidKey(format!(
                "DEK must be {} bytes, got {}",
                DEK_SIZE,
                key_material.len()
            )));
        }

        Ok(Self {
            id,
            key_material,
            created_at: chrono::Utc::now(),
        })
    }

    /// Get the key ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the key material (use with caution).
    pub fn key_material(&self) -> &[u8] {
        &self.key_material
    }
}

/// Encrypted data envelope containing ciphertext and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    /// Unique identifier for this envelope.
    pub envelope_id: String,

    /// ID of the DEK used to encrypt the data.
    pub dek_id: String,

    /// Encrypted data encryption key (encrypted with KEK).
    pub encrypted_dek: Vec<u8>,

    /// Nonce used for DEK encryption.
    pub dek_nonce: Vec<u8>,

    /// Encrypted plaintext data.
    pub ciphertext: Vec<u8>,

    /// Nonce used for data encryption.
    pub data_nonce: Vec<u8>,

    /// Additional authenticated data (AAD) used in encryption.
    #[serde(default)]
    pub aad: Vec<u8>,

    /// Timestamp when the envelope was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Algorithm used for encryption.
    pub algorithm: String,

    /// Key encryption key ID (reference to KMS or HSM).
    pub kek_id: String,
}

/// Envelope encryption service for managing data encryption.
pub struct EnvelopeEncryption {
    /// Default algorithm for encryption.
    algorithm: String,
}

impl EnvelopeEncryption {
    /// Create a new envelope encryption service.
    pub fn new() -> Self {
        Self {
            algorithm: "AES-256-GCM".to_string(),
        }
    }

    /// Encrypt data using envelope encryption.
    ///
    /// This method:
    /// 1. Generates a new random DEK
    /// 2. Encrypts the plaintext with the DEK
    /// 3. Encrypts the DEK with the provided KEK
    /// 4. Returns an encrypted envelope containing all components
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        kek: &[u8],
        kek_id: &str,
        aad: Option<&[u8]>,
    ) -> CryptoResult<EncryptedEnvelope> {
        // Generate a new DEK
        let dek = DataEncryptionKey::generate()?;

        // Create cipher for data encryption
        let data_cipher = Aes256Gcm::new_from_slice(&dek.key_material)
            .map_err(|e| CryptoError::EncryptionFailed(format!("Failed to create cipher: {}", e)))?;

        // Generate random nonce for data encryption
        let mut data_nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut data_nonce_bytes);
        let data_nonce = Nonce::from_slice(&data_nonce_bytes);

        // Encrypt the plaintext with the DEK
        let ciphertext = if let Some(aad_data) = aad {
            data_cipher
                .encrypt(data_nonce, aes_gcm::aead::Payload {
                    msg: plaintext,
                    aad: aad_data,
                })
                .map_err(|e| CryptoError::EncryptionFailed(format!("Data encryption failed: {}", e)))?
        } else {
            data_cipher
                .encrypt(data_nonce, plaintext)
                .map_err(|e| CryptoError::EncryptionFailed(format!("Data encryption failed: {}", e)))?
        };

        // Encrypt the DEK with the KEK
        let kek_cipher = Aes256Gcm::new_from_slice(kek)
            .map_err(|e| CryptoError::EncryptionFailed(format!("Failed to create KEK cipher: {}", e)))?;

        let mut dek_nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut dek_nonce_bytes);
        let dek_nonce = Nonce::from_slice(&dek_nonce_bytes);

        let encrypted_dek = kek_cipher
            .encrypt(dek_nonce, dek.key_material.as_ref())
            .map_err(|e| CryptoError::EncryptionFailed(format!("DEK encryption failed: {}", e)))?;

        Ok(EncryptedEnvelope {
            envelope_id: uuid::Uuid::new_v4().to_string(),
            dek_id: dek.id.clone(),
            encrypted_dek,
            dek_nonce: dek_nonce_bytes.to_vec(),
            ciphertext,
            data_nonce: data_nonce_bytes.to_vec(),
            aad: aad.map(|a| a.to_vec()).unwrap_or_default(),
            created_at: chrono::Utc::now(),
            algorithm: self.algorithm.clone(),
            kek_id: kek_id.to_string(),
        })
    }

    /// Decrypt data from an encrypted envelope.
    ///
    /// This method:
    /// 1. Decrypts the DEK using the provided KEK
    /// 2. Decrypts the ciphertext using the DEK
    /// 3. Returns the original plaintext
    pub fn decrypt(&self, envelope: &EncryptedEnvelope, kek: &[u8]) -> CryptoResult<Vec<u8>> {
        // Decrypt the DEK using the KEK
        let kek_cipher = Aes256Gcm::new_from_slice(kek)
            .map_err(|e| CryptoError::DecryptionFailed(format!("Failed to create KEK cipher: {}", e)))?;

        let dek_nonce = Nonce::from_slice(&envelope.dek_nonce);

        let dek_material = kek_cipher
            .decrypt(dek_nonce, envelope.encrypted_dek.as_ref())
            .map_err(|e| CryptoError::DecryptionFailed(format!("DEK decryption failed: {}", e)))?;

        // Create DEK
        let dek = DataEncryptionKey::from_bytes(envelope.dek_id.clone(), dek_material)?;

        // Decrypt the ciphertext using the DEK
        let data_cipher = Aes256Gcm::new_from_slice(&dek.key_material)
            .map_err(|e| CryptoError::DecryptionFailed(format!("Failed to create data cipher: {}", e)))?;

        let data_nonce = Nonce::from_slice(&envelope.data_nonce);

        let plaintext = if !envelope.aad.is_empty() {
            data_cipher
                .decrypt(data_nonce, aes_gcm::aead::Payload {
                    msg: &envelope.ciphertext,
                    aad: &envelope.aad,
                })
                .map_err(|e| CryptoError::DecryptionFailed(format!("Data decryption failed: {}", e)))?
        } else {
            data_cipher
                .decrypt(data_nonce, envelope.ciphertext.as_ref())
                .map_err(|e| CryptoError::DecryptionFailed(format!("Data decryption failed: {}", e)))?
        };

        Ok(plaintext)
    }

    /// Rotate the encryption key by re-encrypting the DEK with a new KEK.
    ///
    /// This allows key rotation without re-encrypting the actual data.
    pub fn rotate_kek(
        &self,
        envelope: &EncryptedEnvelope,
        old_kek: &[u8],
        new_kek: &[u8],
        new_kek_id: &str,
    ) -> CryptoResult<EncryptedEnvelope> {
        // Decrypt the DEK with the old KEK
        let old_kek_cipher = Aes256Gcm::new_from_slice(old_kek)
            .map_err(|e| CryptoError::KeyRotationFailed(format!("Failed to create old KEK cipher: {}", e)))?;

        let dek_nonce = Nonce::from_slice(&envelope.dek_nonce);

        let dek_material = old_kek_cipher
            .decrypt(dek_nonce, envelope.encrypted_dek.as_ref())
            .map_err(|e| CryptoError::KeyRotationFailed(format!("Failed to decrypt DEK with old KEK: {}", e)))?;

        // Encrypt the DEK with the new KEK
        let new_kek_cipher = Aes256Gcm::new_from_slice(new_kek)
            .map_err(|e| CryptoError::KeyRotationFailed(format!("Failed to create new KEK cipher: {}", e)))?;

        let mut new_dek_nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut new_dek_nonce_bytes);
        let new_dek_nonce = Nonce::from_slice(&new_dek_nonce_bytes);

        let new_encrypted_dek = new_kek_cipher
            .encrypt(new_dek_nonce, dek_material.as_ref())
            .map_err(|e| CryptoError::KeyRotationFailed(format!("Failed to encrypt DEK with new KEK: {}", e)))?;

        // Return new envelope with rotated KEK
        Ok(EncryptedEnvelope {
            envelope_id: uuid::Uuid::new_v4().to_string(),
            dek_id: envelope.dek_id.clone(),
            encrypted_dek: new_encrypted_dek,
            dek_nonce: new_dek_nonce_bytes.to_vec(),
            ciphertext: envelope.ciphertext.clone(),
            data_nonce: envelope.data_nonce.clone(),
            aad: envelope.aad.clone(),
            created_at: chrono::Utc::now(),
            algorithm: envelope.algorithm.clone(),
            kek_id: new_kek_id.to_string(),
        })
    }

    /// Serialize an encrypted envelope to bytes.
    pub fn serialize_envelope(&self, envelope: &EncryptedEnvelope) -> CryptoResult<Vec<u8>> {
        bincode::serialize(envelope)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to serialize envelope: {}", e)))
    }

    /// Deserialize an encrypted envelope from bytes.
    pub fn deserialize_envelope(&self, data: &[u8]) -> CryptoResult<EncryptedEnvelope> {
        bincode::deserialize(data)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to deserialize envelope: {}", e)))
    }
}

impl Default for EnvelopeEncryption {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_encryption() {
        let envelope_enc = EnvelopeEncryption::new();
        let plaintext = b"Sensitive GIS data that needs protection";
        let mut kek = vec![0u8; 32];
        OsRng.fill_bytes(&mut kek);

        let envelope = envelope_enc
            .encrypt(plaintext, &kek, "test-kek-1", None)
            .expect("Encryption should succeed");

        let decrypted = envelope_enc
            .decrypt(&envelope, &kek)
            .expect("Decryption should succeed");

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_envelope_encryption_with_aad() {
        let envelope_enc = EnvelopeEncryption::new();
        let plaintext = b"Sensitive data";
        let aad = b"context-metadata";
        let mut kek = vec![0u8; 32];
        OsRng.fill_bytes(&mut kek);

        let envelope = envelope_enc
            .encrypt(plaintext, &kek, "test-kek-1", Some(aad))
            .expect("Encryption should succeed");

        let decrypted = envelope_enc
            .decrypt(&envelope, &kek)
            .expect("Decryption should succeed");

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_kek_rotation() {
        let envelope_enc = EnvelopeEncryption::new();
        let plaintext = b"Data to encrypt";
        let mut old_kek = vec![0u8; 32];
        let mut new_kek = vec![0u8; 32];
        OsRng.fill_bytes(&mut old_kek);
        OsRng.fill_bytes(&mut new_kek);

        let envelope = envelope_enc
            .encrypt(plaintext, &old_kek, "old-kek", None)
            .expect("Encryption should succeed");

        let rotated_envelope = envelope_enc
            .rotate_kek(&envelope, &old_kek, &new_kek, "new-kek")
            .expect("KEK rotation should succeed");

        let decrypted = envelope_enc
            .decrypt(&rotated_envelope, &new_kek)
            .expect("Decryption with new KEK should succeed");

        assert_eq!(plaintext, decrypted.as_slice());
    }
}
