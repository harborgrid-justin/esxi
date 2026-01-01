//! Envelope encryption with key hierarchy
//!
//! Envelope encryption is a practice where data is encrypted with a Data Encryption Key (DEK),
//! and the DEK itself is encrypted with a Key Encryption Key (KEK). This provides:
//!
//! - **Key Rotation**: Rotate KEK without re-encrypting all data
//! - **Performance**: Encrypt large data with fast symmetric keys
//! - **Security**: KEKs can be stored in HSMs or KMS
//! - **Compliance**: Meets requirements for key management (SOC 2, PCI-DSS)
//!
//! ## Architecture
//! ```text
//! Master Key (KEK) → encrypts → Data Key (DEK) → encrypts → Your Data
//!     [In KMS/HSM]              [Per-record]                [Database]
//! ```
//!
//! ## OWASP Best Practices
//! - Unique DEK per data item (prevents correlation)
//! - KEK stored separately from data
//! - Regular key rotation (90 days recommended)
//! - Audit trail for all key operations

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::{
    encryption::{aes::AesGcmEncryptor, Encryptor, KeyGenerator},
    error::{SecurityError, SecurityResult},
};

/// Encrypted data with envelope encryption
///
/// Contains both the encrypted data and the encrypted DEK
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeEncrypted {
    /// Encrypted data (using DEK)
    pub ciphertext: Vec<u8>,

    /// Encrypted DEK (using KEK)
    pub encrypted_dek: Vec<u8>,

    /// Key version (for rotation tracking)
    pub key_version: u32,

    /// Additional metadata (not encrypted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Envelope encryption manager
///
/// Manages the encryption of data using envelope encryption pattern
pub struct EnvelopeEncryption {
    /// Key Encryption Key (KEK) - encrypts DEKs
    kek: Zeroizing<Vec<u8>>,

    /// Current key version
    key_version: u32,
}

impl EnvelopeEncryption {
    /// Create a new envelope encryption manager with a KEK
    ///
    /// # Arguments
    /// * `kek` - Key Encryption Key (32 bytes for AES-256)
    /// * `key_version` - Version number for key rotation tracking
    ///
    /// # Security Note
    /// The KEK should ideally be stored in a Hardware Security Module (HSM)
    /// or Key Management Service (KMS) like AWS KMS, Azure Key Vault, etc.
    pub fn new(kek: Vec<u8>, key_version: u32) -> SecurityResult<Self> {
        if kek.len() != 32 {
            return Err(SecurityError::InvalidKey(
                "KEK must be 32 bytes (256 bits)".to_string(),
            ));
        }

        Ok(Self {
            kek: Zeroizing::new(kek),
            key_version,
        })
    }

    /// Generate a new KEK
    pub fn generate_kek() -> SecurityResult<Vec<u8>> {
        AesGcmEncryptor::generate_key()
    }

    /// Encrypt data using envelope encryption
    ///
    /// # Process
    /// 1. Generate random DEK
    /// 2. Encrypt data with DEK
    /// 3. Encrypt DEK with KEK
    /// 4. Return both encrypted data and encrypted DEK
    ///
    /// # Arguments
    /// * `plaintext` - Data to encrypt
    pub fn encrypt(&self, plaintext: &[u8]) -> SecurityResult<EnvelopeEncrypted> {
        // Step 1: Generate random DEK for this data
        let dek = Zeroizing::new(AesGcmEncryptor::generate_key()?);

        // Step 2: Encrypt plaintext with DEK
        let dek_encryptor = AesGcmEncryptor::new(&dek)?;
        let ciphertext = dek_encryptor.encrypt(plaintext)?;

        // Step 3: Encrypt DEK with KEK
        let kek_encryptor = AesGcmEncryptor::new(&self.kek)?;
        let encrypted_dek = kek_encryptor.encrypt(&dek)?;

        Ok(EnvelopeEncrypted {
            ciphertext,
            encrypted_dek,
            key_version: self.key_version,
            metadata: None,
        })
    }

    /// Encrypt data with metadata
    ///
    /// Metadata is stored unencrypted but can be used for indexing/searching
    pub fn encrypt_with_metadata(
        &self,
        plaintext: &[u8],
        metadata: serde_json::Value,
    ) -> SecurityResult<EnvelopeEncrypted> {
        let mut encrypted = self.encrypt(plaintext)?;
        encrypted.metadata = Some(metadata);
        Ok(encrypted)
    }

    /// Decrypt data encrypted with envelope encryption
    ///
    /// # Process
    /// 1. Decrypt DEK using KEK
    /// 2. Decrypt data using DEK
    ///
    /// # Arguments
    /// * `encrypted` - Envelope encrypted data
    pub fn decrypt(&self, encrypted: &EnvelopeEncrypted) -> SecurityResult<Vec<u8>> {
        // Step 1: Decrypt DEK with KEK
        let kek_encryptor = AesGcmEncryptor::new(&self.kek)?;
        let dek = Zeroizing::new(kek_encryptor.decrypt(&encrypted.encrypted_dek)?);

        // Step 2: Decrypt data with DEK
        let dek_encryptor = AesGcmEncryptor::new(&dek)?;
        let plaintext = dek_encryptor.decrypt(&encrypted.ciphertext)?;

        Ok(plaintext)
    }

    /// Re-encrypt data with a new KEK (for key rotation)
    ///
    /// # Arguments
    /// * `encrypted` - Data encrypted with old KEK
    /// * `new_kek` - New Key Encryption Key
    /// * `new_version` - New key version number
    ///
    /// # Returns
    /// Data re-encrypted with new KEK (DEK remains the same)
    pub fn rotate_kek(
        &self,
        encrypted: &EnvelopeEncrypted,
        new_kek: &[u8],
        new_version: u32,
    ) -> SecurityResult<EnvelopeEncrypted> {
        // Decrypt DEK with old KEK
        let old_kek_encryptor = AesGcmEncryptor::new(&self.kek)?;
        let dek = Zeroizing::new(old_kek_encryptor.decrypt(&encrypted.encrypted_dek)?);

        // Re-encrypt DEK with new KEK
        let new_kek_encryptor = AesGcmEncryptor::new(new_kek)?;
        let new_encrypted_dek = new_kek_encryptor.encrypt(&dek)?;

        Ok(EnvelopeEncrypted {
            ciphertext: encrypted.ciphertext.clone(), // Data unchanged
            encrypted_dek: new_encrypted_dek,          // DEK re-encrypted
            key_version: new_version,
            metadata: encrypted.metadata.clone(),
        })
    }

    /// Get current key version
    pub fn key_version(&self) -> u32 {
        self.key_version
    }

    /// Serialize encrypted data to bytes (for storage)
    pub fn serialize(encrypted: &EnvelopeEncrypted) -> SecurityResult<Vec<u8>> {
        serde_json::to_vec(encrypted).map_err(|e| SecurityError::SerializationError(e.to_string()))
    }

    /// Deserialize encrypted data from bytes
    pub fn deserialize(data: &[u8]) -> SecurityResult<EnvelopeEncrypted> {
        serde_json::from_slice(data).map_err(|e| SecurityError::SerializationError(e.to_string()))
    }
}

impl Drop for EnvelopeEncryption {
    fn drop(&mut self) {
        // KEK is already wrapped in Zeroizing, will be zeroized on drop
    }
}

/// Multi-layer envelope encryption (KEK hierarchy)
///
/// For enterprise scenarios requiring multiple key layers:
/// Root KEK → Intermediate KEK → DEK → Data
pub struct LayeredEnvelopeEncryption {
    layers: Vec<Zeroizing<Vec<u8>>>,
}

impl LayeredEnvelopeEncryption {
    /// Create a new layered envelope encryption with key hierarchy
    ///
    /// # Arguments
    /// * `keys` - Ordered list of keys from root to leaf (all 32 bytes)
    pub fn new(keys: Vec<Vec<u8>>) -> SecurityResult<Self> {
        if keys.is_empty() {
            return Err(SecurityError::InvalidKey("At least one key required".to_string()));
        }

        for key in &keys {
            if key.len() != 32 {
                return Err(SecurityError::InvalidKey("All keys must be 32 bytes".to_string()));
            }
        }

        Ok(Self {
            layers: keys.into_iter().map(Zeroizing::new).collect(),
        })
    }

    /// Encrypt with multiple key layers
    pub fn encrypt(&self, plaintext: &[u8]) -> SecurityResult<Vec<u8>> {
        let mut current_data = plaintext.to_vec();

        // Encrypt with each layer
        for key in self.layers.iter().rev() {
            let encryptor = AesGcmEncryptor::new(key)?;
            current_data = encryptor.encrypt(&current_data)?;
        }

        Ok(current_data)
    }

    /// Decrypt with multiple key layers
    pub fn decrypt(&self, ciphertext: &[u8]) -> SecurityResult<Vec<u8>> {
        let mut current_data = ciphertext.to_vec();

        // Decrypt with each layer
        for key in &self.layers {
            let encryptor = AesGcmEncryptor::new(key)?;
            current_data = encryptor.decrypt(&current_data)?;
        }

        Ok(current_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_encryption_decrypt() {
        let kek = EnvelopeEncryption::generate_kek().unwrap();
        let envelope = EnvelopeEncryption::new(kek, 1).unwrap();

        let plaintext = b"Enterprise sensitive data";
        let encrypted = envelope.encrypt(plaintext).unwrap();
        let decrypted = envelope.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_envelope_with_metadata() {
        let kek = EnvelopeEncryption::generate_kek().unwrap();
        let envelope = EnvelopeEncryption::new(kek, 1).unwrap();

        let plaintext = b"Customer PII";
        let metadata = serde_json::json!({
            "customer_id": "12345",
            "data_type": "pii",
            "encrypted_at": "2026-01-01T00:00:00Z"
        });

        let encrypted = envelope.encrypt_with_metadata(plaintext, metadata.clone()).unwrap();
        assert_eq!(encrypted.metadata, Some(metadata));

        let decrypted = envelope.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_kek_rotation() {
        let old_kek = EnvelopeEncryption::generate_kek().unwrap();
        let new_kek = EnvelopeEncryption::generate_kek().unwrap();

        let old_envelope = EnvelopeEncryption::new(old_kek.clone(), 1).unwrap();

        let plaintext = b"Data to rotate";
        let encrypted_v1 = old_envelope.encrypt(plaintext).unwrap();

        // Rotate to new KEK
        let encrypted_v2 = old_envelope.rotate_kek(&encrypted_v1, &new_kek, 2).unwrap();
        assert_eq!(encrypted_v2.key_version, 2);

        // Decrypt with new KEK
        let new_envelope = EnvelopeEncryption::new(new_kek, 2).unwrap();
        let decrypted = new_envelope.decrypt(&encrypted_v2).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_serialization() {
        let kek = EnvelopeEncryption::generate_kek().unwrap();
        let envelope = EnvelopeEncryption::new(kek, 1).unwrap();

        let plaintext = b"Serialize me";
        let encrypted = envelope.encrypt(plaintext).unwrap();

        let serialized = EnvelopeEncryption::serialize(&encrypted).unwrap();
        let deserialized = EnvelopeEncryption::deserialize(&serialized).unwrap();

        let decrypted = envelope.decrypt(&deserialized).unwrap();
        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_unique_deks() {
        let kek = EnvelopeEncryption::generate_kek().unwrap();
        let envelope = EnvelopeEncryption::new(kek, 1).unwrap();

        let plaintext = b"Same data";
        let enc1 = envelope.encrypt(plaintext).unwrap();
        let enc2 = envelope.encrypt(plaintext).unwrap();

        // Different DEKs should be used
        assert_ne!(enc1.encrypted_dek, enc2.encrypted_dek);
        assert_ne!(enc1.ciphertext, enc2.ciphertext);
    }

    #[test]
    fn test_layered_encryption() {
        let key1 = AesGcmEncryptor::generate_key().unwrap();
        let key2 = AesGcmEncryptor::generate_key().unwrap();
        let key3 = AesGcmEncryptor::generate_key().unwrap();

        let layered = LayeredEnvelopeEncryption::new(vec![key1, key2, key3]).unwrap();

        let plaintext = b"Triple encrypted";
        let ciphertext = layered.encrypt(plaintext).unwrap();
        let decrypted = layered.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[test]
    fn test_invalid_kek_size() {
        let short_kek = vec![0u8; 16];
        let result = EnvelopeEncryption::new(short_kek, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_kek_fails_decrypt() {
        let kek1 = EnvelopeEncryption::generate_kek().unwrap();
        let kek2 = EnvelopeEncryption::generate_kek().unwrap();

        let envelope1 = EnvelopeEncryption::new(kek1, 1).unwrap();
        let envelope2 = EnvelopeEncryption::new(kek2, 1).unwrap();

        let plaintext = b"Secret";
        let encrypted = envelope1.encrypt(plaintext).unwrap();

        // Try to decrypt with wrong KEK
        let result = envelope2.decrypt(&encrypted);
        assert!(result.is_err());
    }
}
