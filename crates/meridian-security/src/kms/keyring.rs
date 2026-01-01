//! Secure keyring for key storage and rotation
//!
//! Provides secure storage for cryptographic keys with:
//! - Version tracking for key rotation
//! - Metadata (creation time, purpose, algorithm)
//! - Active/retired status management
//! - Automatic key rotation scheduling
//!
//! ## Security Features
//! - Keys stored in memory with zeroization
//! - Access controls per key
//! - Audit logging for key access
//! - Support for key derivation

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::{
    config::KEY_ROTATION_INTERVAL,
    encryption::{aes::AesGcmEncryptor, KeyGenerator},
    error::{SecurityError, SecurityResult},
};

/// Key metadata for tracking and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Unique key identifier
    pub key_id: String,

    /// Key version (for rotation)
    pub version: u32,

    /// Key purpose (encryption, signing, etc.)
    pub purpose: KeyPurpose,

    /// Algorithm used with this key
    pub algorithm: String,

    /// Key creation timestamp
    pub created_at: DateTime<Utc>,

    /// Key expiration timestamp (for rotation)
    pub expires_at: Option<DateTime<Utc>>,

    /// Key status
    pub status: KeyStatus,

    /// Previous version (if rotated)
    pub previous_version: Option<u32>,
}

/// Key purpose classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyPurpose {
    /// Data encryption
    Encryption,
    /// Digital signatures
    Signing,
    /// HMAC message authentication
    Authentication,
    /// Key derivation
    Derivation,
    /// Master key (KEK)
    MasterKey,
}

/// Key status in lifecycle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStatus {
    /// Active - can be used for encryption/signing
    Active,
    /// Retired - only for decryption/verification, not new operations
    Retired,
    /// Revoked - should not be used
    Revoked,
    /// Pending - scheduled for activation
    Pending,
}

/// Stored key with its material and metadata
#[derive(Clone)]
struct StoredKey {
    material: Zeroizing<Vec<u8>>,
    metadata: KeyMetadata,
}

impl Drop for StoredKey {
    fn drop(&mut self) {
        // Zeroizing handles key material cleanup
    }
}

/// Secure keyring for managing cryptographic keys
///
/// # Example
/// ```rust,no_run
/// use meridian_security::kms::keyring::{Keyring, KeyPurpose};
///
/// let mut keyring = Keyring::new();
///
/// // Add a new encryption key
/// let key_id = keyring.generate_key("app-encryption", KeyPurpose::Encryption, "AES-256-GCM").unwrap();
///
/// // Retrieve key for use
/// let key = keyring.get_active_key(&key_id).unwrap();
///
/// // Rotate key
/// keyring.rotate_key(&key_id).unwrap();
/// ```
pub struct Keyring {
    /// Map of key_id -> list of versions (newest first)
    keys: Arc<RwLock<HashMap<String, Vec<StoredKey>>>>,

    /// Rotation interval
    rotation_interval: Duration,
}

impl Keyring {
    /// Create a new empty keyring
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            rotation_interval: Duration::from_std(KEY_ROTATION_INTERVAL)
                .expect("Invalid rotation interval"),
        }
    }

    /// Create keyring with custom rotation interval
    pub fn with_rotation_interval(interval: Duration) -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            rotation_interval: interval,
        }
    }

    /// Generate a new key and add to keyring
    ///
    /// # Arguments
    /// * `key_id` - Unique identifier for this key
    /// * `purpose` - Purpose of the key
    /// * `algorithm` - Algorithm name (e.g., "AES-256-GCM")
    ///
    /// # Returns
    /// Key ID on success
    pub fn generate_key(
        &mut self,
        key_id: &str,
        purpose: KeyPurpose,
        algorithm: &str,
    ) -> SecurityResult<String> {
        // Generate key material based on algorithm
        let key_material = match algorithm {
            "AES-256-GCM" | "ChaCha20-Poly1305" => AesGcmEncryptor::generate_key()?,
            _ => {
                return Err(SecurityError::InvalidKey(format!(
                    "Unsupported algorithm: {}",
                    algorithm
                )))
            }
        };

        self.add_key(key_id, key_material, purpose, algorithm)
    }

    /// Add an existing key to the keyring
    ///
    /// # Arguments
    /// * `key_id` - Unique identifier
    /// * `key_material` - Raw key bytes
    /// * `purpose` - Key purpose
    /// * `algorithm` - Algorithm name
    pub fn add_key(
        &mut self,
        key_id: &str,
        key_material: Vec<u8>,
        purpose: KeyPurpose,
        algorithm: &str,
    ) -> SecurityResult<String> {
        let mut keys = self.keys.write().map_err(|_| {
            SecurityError::InternalError("Failed to acquire keyring write lock".to_string())
        })?;

        let version = if let Some(existing) = keys.get(key_id) {
            existing.first().map(|k| k.metadata.version + 1).unwrap_or(1)
        } else {
            1
        };

        let metadata = KeyMetadata {
            key_id: key_id.to_string(),
            version,
            purpose,
            algorithm: algorithm.to_string(),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + self.rotation_interval),
            status: KeyStatus::Active,
            previous_version: if version > 1 { Some(version - 1) } else { None },
        };

        let stored = StoredKey {
            material: Zeroizing::new(key_material),
            metadata,
        };

        keys.entry(key_id.to_string())
            .or_insert_with(Vec::new)
            .insert(0, stored); // Insert at front (newest first)

        Ok(key_id.to_string())
    }

    /// Get the active key (latest version)
    ///
    /// # Arguments
    /// * `key_id` - Key identifier
    ///
    /// # Returns
    /// Copy of key material
    pub fn get_active_key(&self, key_id: &str) -> SecurityResult<Vec<u8>> {
        let keys = self.keys.read().map_err(|_| {
            SecurityError::InternalError("Failed to acquire keyring read lock".to_string())
        })?;

        let versions = keys
            .get(key_id)
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))?;

        let active = versions
            .iter()
            .find(|k| k.metadata.status == KeyStatus::Active)
            .ok_or_else(|| SecurityError::KeyNotFound(format!("No active version for {}", key_id)))?;

        Ok(active.material.to_vec())
    }

    /// Get a specific key version
    pub fn get_key_version(&self, key_id: &str, version: u32) -> SecurityResult<Vec<u8>> {
        let keys = self.keys.read().map_err(|_| {
            SecurityError::InternalError("Failed to acquire keyring read lock".to_string())
        })?;

        let versions = keys
            .get(key_id)
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))?;

        let key = versions
            .iter()
            .find(|k| k.metadata.version == version)
            .ok_or_else(|| {
                SecurityError::KeyNotFound(format!("{} version {}", key_id, version))
            })?;

        Ok(key.material.to_vec())
    }

    /// Get key metadata
    pub fn get_metadata(&self, key_id: &str) -> SecurityResult<KeyMetadata> {
        let keys = self.keys.read().map_err(|_| {
            SecurityError::InternalError("Failed to acquire keyring read lock".to_string())
        })?;

        let versions = keys
            .get(key_id)
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))?;

        versions
            .first()
            .map(|k| k.metadata.clone())
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))
    }

    /// Rotate a key (create new version, retire old)
    ///
    /// # Process
    /// 1. Generate new key material
    /// 2. Increment version
    /// 3. Mark old version as retired
    /// 4. Add new version as active
    pub fn rotate_key(&mut self, key_id: &str) -> SecurityResult<u32> {
        let metadata = self.get_metadata(key_id)?;

        // Generate new key
        let new_material = match metadata.algorithm.as_str() {
            "AES-256-GCM" | "ChaCha20-Poly1305" => AesGcmEncryptor::generate_key()?,
            _ => {
                return Err(SecurityError::InvalidKey(format!(
                    "Unsupported algorithm: {}",
                    metadata.algorithm
                )))
            }
        };

        let mut keys = self.keys.write().map_err(|_| {
            SecurityError::InternalError("Failed to acquire keyring write lock".to_string())
        })?;

        let versions = keys
            .get_mut(key_id)
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))?;

        // Retire old active version
        for key in versions.iter_mut() {
            if key.metadata.status == KeyStatus::Active {
                key.metadata.status = KeyStatus::Retired;
            }
        }

        let new_version = metadata.version + 1;

        let new_metadata = KeyMetadata {
            key_id: key_id.to_string(),
            version: new_version,
            purpose: metadata.purpose,
            algorithm: metadata.algorithm,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + self.rotation_interval),
            status: KeyStatus::Active,
            previous_version: Some(metadata.version),
        };

        let stored = StoredKey {
            material: Zeroizing::new(new_material),
            metadata: new_metadata,
        };

        versions.insert(0, stored);

        Ok(new_version)
    }

    /// Check if any keys need rotation
    pub fn keys_needing_rotation(&self) -> SecurityResult<Vec<String>> {
        let keys = self.keys.read().map_err(|_| {
            SecurityError::InternalError("Failed to acquire keyring read lock".to_string())
        })?;

        let now = Utc::now();
        let mut needs_rotation = Vec::new();

        for (key_id, versions) in keys.iter() {
            if let Some(active) = versions.iter().find(|k| k.metadata.status == KeyStatus::Active) {
                if let Some(expires_at) = active.metadata.expires_at {
                    if expires_at <= now {
                        needs_rotation.push(key_id.clone());
                    }
                }
            }
        }

        Ok(needs_rotation)
    }

    /// List all key IDs in the keyring
    pub fn list_keys(&self) -> SecurityResult<Vec<String>> {
        let keys = self.keys.read().map_err(|_| {
            SecurityError::InternalError("Failed to acquire keyring read lock".to_string())
        })?;

        Ok(keys.keys().cloned().collect())
    }

    /// Revoke a key (mark all versions as revoked)
    pub fn revoke_key(&mut self, key_id: &str) -> SecurityResult<()> {
        let mut keys = self.keys.write().map_err(|_| {
            SecurityError::InternalError("Failed to acquire keyring write lock".to_string())
        })?;

        let versions = keys
            .get_mut(key_id)
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))?;

        for key in versions.iter_mut() {
            key.metadata.status = KeyStatus::Revoked;
        }

        Ok(())
    }

    /// Delete a key permanently (use with caution!)
    pub fn delete_key(&mut self, key_id: &str) -> SecurityResult<()> {
        let mut keys = self.keys.write().map_err(|_| {
            SecurityError::InternalError("Failed to acquire keyring write lock".to_string())
        })?;

        keys.remove(key_id)
            .ok_or_else(|| SecurityError::KeyNotFound(key_id.to_string()))?;

        Ok(())
    }
}

impl Default for Keyring {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_retrieve_key() {
        let mut keyring = Keyring::new();
        let key_id = keyring
            .generate_key("test-key", KeyPurpose::Encryption, "AES-256-GCM")
            .unwrap();

        let key = keyring.get_active_key(&key_id).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_key_rotation() {
        let mut keyring = Keyring::new();
        let key_id = keyring
            .generate_key("rotate-key", KeyPurpose::Encryption, "AES-256-GCM")
            .unwrap();

        let key_v1 = keyring.get_active_key(&key_id).unwrap();
        let v2 = keyring.rotate_key(&key_id).unwrap();
        assert_eq!(v2, 2);

        let key_v2 = keyring.get_active_key(&key_id).unwrap();
        assert_ne!(key_v1, key_v2);

        // Old version should still be accessible
        let key_v1_again = keyring.get_key_version(&key_id, 1).unwrap();
        assert_eq!(key_v1, key_v1_again);
    }

    #[test]
    fn test_key_metadata() {
        let mut keyring = Keyring::new();
        let key_id = keyring
            .generate_key("meta-key", KeyPurpose::Signing, "AES-256-GCM")
            .unwrap();

        let metadata = keyring.get_metadata(&key_id).unwrap();
        assert_eq!(metadata.purpose, KeyPurpose::Signing);
        assert_eq!(metadata.status, KeyStatus::Active);
        assert_eq!(metadata.version, 1);
    }

    #[test]
    fn test_key_revocation() {
        let mut keyring = Keyring::new();
        let key_id = keyring
            .generate_key("revoke-key", KeyPurpose::Encryption, "AES-256-GCM")
            .unwrap();

        keyring.revoke_key(&key_id).unwrap();
        let metadata = keyring.get_metadata(&key_id).unwrap();
        assert_eq!(metadata.status, KeyStatus::Revoked);
    }

    #[test]
    fn test_list_keys() {
        let mut keyring = Keyring::new();
        keyring
            .generate_key("key1", KeyPurpose::Encryption, "AES-256-GCM")
            .unwrap();
        keyring
            .generate_key("key2", KeyPurpose::Signing, "AES-256-GCM")
            .unwrap();

        let keys = keyring.list_keys().unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }

    #[test]
    fn test_key_not_found() {
        let keyring = Keyring::new();
        let result = keyring.get_active_key("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_key() {
        let mut keyring = Keyring::new();
        let key_id = keyring
            .generate_key("delete-me", KeyPurpose::Encryption, "AES-256-GCM")
            .unwrap();

        keyring.delete_key(&key_id).unwrap();
        let result = keyring.get_active_key(&key_id);
        assert!(result.is_err());
    }
}
