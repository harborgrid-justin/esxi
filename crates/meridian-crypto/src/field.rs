//! Field-level encryption for sensitive data.
//!
//! This module provides granular encryption at the field level, allowing selective encryption
//! of sensitive fields in data structures.

use crate::error::{CryptoError, CryptoResult};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Field encryption context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldEncryptionContext {
    /// Field path (e.g., "user.email", "location.coordinates").
    pub field_path: String,

    /// Encryption algorithm used.
    pub algorithm: String,

    /// Key ID used for encryption.
    pub key_id: String,

    /// Additional authenticated data.
    pub aad: Option<Vec<u8>>,
}

/// Encrypted field value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedField {
    /// The encrypted value.
    pub ciphertext: Vec<u8>,

    /// Nonce used for encryption.
    pub nonce: Vec<u8>,

    /// Encryption context.
    pub context: FieldEncryptionContext,

    /// Timestamp when encrypted.
    pub encrypted_at: chrono::DateTime<chrono::Utc>,
}

/// Field encryption policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldEncryptionPolicy {
    /// Fields that should be encrypted.
    pub encrypted_fields: Vec<String>,

    /// Key ID to use for encryption.
    pub key_id: String,

    /// Whether to use deterministic encryption (for searchability).
    pub deterministic: bool,

    /// Additional authenticated data template.
    pub aad_template: Option<String>,
}

/// Field encryption service.
pub struct FieldEncryption {
    /// Policies by entity type.
    policies: HashMap<String, FieldEncryptionPolicy>,
}

impl FieldEncryption {
    /// Create a new field encryption service.
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    /// Register an encryption policy for an entity type.
    pub fn register_policy(&mut self, entity_type: &str, policy: FieldEncryptionPolicy) {
        self.policies.insert(entity_type.to_string(), policy);
    }

    /// Get policy for an entity type.
    pub fn get_policy(&self, entity_type: &str) -> Option<&FieldEncryptionPolicy> {
        self.policies.get(entity_type)
    }

    /// Encrypt a field value.
    pub fn encrypt_field(
        &self,
        field_path: &str,
        value: &[u8],
        key: &[u8],
        key_id: &str,
        aad: Option<&[u8]>,
        deterministic: bool,
    ) -> CryptoResult<EncryptedField> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKey(format!(
                "Key must be 32 bytes, got {}",
                key.len()
            )));
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| CryptoError::FieldEncryptionError(format!("Failed to create cipher: {}", e)))?;

        // Generate nonce
        let nonce_bytes = if deterministic {
            // For deterministic encryption, derive nonce from field path and value
            self.derive_deterministic_nonce(field_path, value)?
        } else {
            // Random nonce for non-deterministic encryption
            let mut nonce = [0u8; 12];
            OsRng.fill_bytes(&mut nonce);
            nonce
        };

        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the value
        let ciphertext = if let Some(aad_data) = aad {
            cipher
                .encrypt(nonce, aes_gcm::aead::Payload {
                    msg: value,
                    aad: aad_data,
                })
                .map_err(|e| CryptoError::FieldEncryptionError(format!("Encryption failed: {}", e)))?
        } else {
            cipher
                .encrypt(nonce, value)
                .map_err(|e| CryptoError::FieldEncryptionError(format!("Encryption failed: {}", e)))?
        };

        Ok(EncryptedField {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
            context: FieldEncryptionContext {
                field_path: field_path.to_string(),
                algorithm: "AES-256-GCM".to_string(),
                key_id: key_id.to_string(),
                aad: aad.map(|a| a.to_vec()),
            },
            encrypted_at: chrono::Utc::now(),
        })
    }

    /// Decrypt a field value.
    pub fn decrypt_field(&self, encrypted_field: &EncryptedField, key: &[u8]) -> CryptoResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(CryptoError::InvalidKey(format!(
                "Key must be 32 bytes, got {}",
                key.len()
            )));
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| CryptoError::FieldEncryptionError(format!("Failed to create cipher: {}", e)))?;

        let nonce = Nonce::from_slice(&encrypted_field.nonce);

        let plaintext = if let Some(aad) = &encrypted_field.context.aad {
            cipher
                .decrypt(nonce, aes_gcm::aead::Payload {
                    msg: &encrypted_field.ciphertext,
                    aad,
                })
                .map_err(|e| CryptoError::FieldEncryptionError(format!("Decryption failed: {}", e)))?
        } else {
            cipher
                .decrypt(nonce, encrypted_field.ciphertext.as_ref())
                .map_err(|e| CryptoError::FieldEncryptionError(format!("Decryption failed: {}", e)))?
        };

        Ok(plaintext)
    }

    /// Encrypt multiple fields in a data structure.
    pub fn encrypt_fields(
        &self,
        entity_type: &str,
        fields: &HashMap<String, Vec<u8>>,
        key: &[u8],
    ) -> CryptoResult<HashMap<String, EncryptedField>> {
        let policy = self
            .get_policy(entity_type)
            .ok_or_else(|| CryptoError::InvalidConfiguration(format!("No policy for: {}", entity_type)))?;

        let mut encrypted_fields = HashMap::new();

        for (field_path, value) in fields {
            if policy.encrypted_fields.contains(field_path) {
                let aad = policy
                    .aad_template
                    .as_ref()
                    .map(|template| self.build_aad(template, field_path).into_bytes());

                let encrypted = self.encrypt_field(
                    field_path,
                    value,
                    key,
                    &policy.key_id,
                    aad.as_deref(),
                    policy.deterministic,
                )?;

                encrypted_fields.insert(field_path.clone(), encrypted);
            }
        }

        Ok(encrypted_fields)
    }

    /// Decrypt multiple fields.
    pub fn decrypt_fields(
        &self,
        encrypted_fields: &HashMap<String, EncryptedField>,
        key: &[u8],
    ) -> CryptoResult<HashMap<String, Vec<u8>>> {
        let mut decrypted_fields = HashMap::new();

        for (field_path, encrypted) in encrypted_fields {
            let decrypted = self.decrypt_field(encrypted, key)?;
            decrypted_fields.insert(field_path.clone(), decrypted);
        }

        Ok(decrypted_fields)
    }

    /// Derive a deterministic nonce from field path and value (for searchable encryption).
    fn derive_deterministic_nonce(&self, field_path: &str, value: &[u8]) -> CryptoResult<[u8; 12]> {
        use ring::digest;

        let mut context = ring::digest::Context::new(&digest::SHA256);
        context.update(field_path.as_bytes());
        context.update(value);
        let digest = context.finish();

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&digest.as_ref()[..12]);
        Ok(nonce)
    }

    /// Build AAD from template.
    fn build_aad(&self, template: &str, field_path: &str) -> String {
        template.replace("{field_path}", field_path)
    }

    /// Create a searchable encrypted field (deterministic encryption).
    pub fn create_searchable_field(
        &self,
        field_path: &str,
        value: &[u8],
        key: &[u8],
        key_id: &str,
    ) -> CryptoResult<EncryptedField> {
        self.encrypt_field(field_path, value, key, key_id, None, true)
    }

    /// Encrypt field as string (convenience method).
    pub fn encrypt_string(
        &self,
        field_path: &str,
        value: &str,
        key: &[u8],
        key_id: &str,
    ) -> CryptoResult<EncryptedField> {
        self.encrypt_field(field_path, value.as_bytes(), key, key_id, None, false)
    }

    /// Decrypt field to string (convenience method).
    pub fn decrypt_string(&self, encrypted_field: &EncryptedField, key: &[u8]) -> CryptoResult<String> {
        let bytes = self.decrypt_field(encrypted_field, key)?;
        String::from_utf8(bytes)
            .map_err(|e| CryptoError::FieldEncryptionError(format!("Invalid UTF-8: {}", e)))
    }

    /// Serialize encrypted field to JSON.
    pub fn serialize_field(&self, field: &EncryptedField) -> CryptoResult<String> {
        serde_json::to_string(field)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to serialize field: {}", e)))
    }

    /// Deserialize encrypted field from JSON.
    pub fn deserialize_field(&self, json: &str) -> CryptoResult<EncryptedField> {
        serde_json::from_str(json)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to deserialize field: {}", e)))
    }
}

impl Default for FieldEncryption {
    fn default() -> Self {
        Self::new()
    }
}

/// GIS-specific field encryption helper.
pub struct GisFieldEncryption {
    field_encryption: FieldEncryption,
}

impl GisFieldEncryption {
    /// Create a new GIS field encryption service.
    pub fn new() -> Self {
        let mut field_encryption = FieldEncryption::new();

        // Register default GIS encryption policies
        Self::register_default_policies(&mut field_encryption);

        Self { field_encryption }
    }

    /// Register default encryption policies for GIS entities.
    fn register_default_policies(encryption: &mut FieldEncryption) {
        // User entity policy
        encryption.register_policy(
            "user",
            FieldEncryptionPolicy {
                encrypted_fields: vec![
                    "email".to_string(),
                    "phone".to_string(),
                    "ssn".to_string(),
                    "address".to_string(),
                ],
                key_id: "user-data-key".to_string(),
                deterministic: false,
                aad_template: Some("user:{field_path}".to_string()),
            },
        );

        // Location entity policy
        encryption.register_policy(
            "location",
            FieldEncryptionPolicy {
                encrypted_fields: vec![
                    "coordinates".to_string(),
                    "address".to_string(),
                    "description".to_string(),
                ],
                key_id: "location-data-key".to_string(),
                deterministic: false,
                aad_template: Some("location:{field_path}".to_string()),
            },
        );

        // Asset entity policy
        encryption.register_policy(
            "asset",
            FieldEncryptionPolicy {
                encrypted_fields: vec![
                    "value".to_string(),
                    "owner".to_string(),
                    "metadata".to_string(),
                ],
                key_id: "asset-data-key".to_string(),
                deterministic: false,
                aad_template: Some("asset:{field_path}".to_string()),
            },
        );
    }

    /// Encrypt coordinates (lat/lon).
    pub fn encrypt_coordinates(
        &self,
        lat: f64,
        lon: f64,
        key: &[u8],
    ) -> CryptoResult<EncryptedField> {
        let coords = format!("{},{}", lat, lon);
        self.field_encryption.encrypt_string("coordinates", &coords, key, "location-data-key")
    }

    /// Decrypt coordinates.
    pub fn decrypt_coordinates(&self, encrypted: &EncryptedField, key: &[u8]) -> CryptoResult<(f64, f64)> {
        let coords_str = self.field_encryption.decrypt_string(encrypted, key)?;
        let parts: Vec<&str> = coords_str.split(',').collect();

        if parts.len() != 2 {
            return Err(CryptoError::FieldEncryptionError(
                "Invalid coordinate format".to_string(),
            ));
        }

        let lat = parts[0]
            .parse::<f64>()
            .map_err(|e| CryptoError::FieldEncryptionError(format!("Invalid latitude: {}", e)))?;
        let lon = parts[1]
            .parse::<f64>()
            .map_err(|e| CryptoError::FieldEncryptionError(format!("Invalid longitude: {}", e)))?;

        Ok((lat, lon))
    }

    /// Get the underlying field encryption service.
    pub fn inner(&self) -> &FieldEncryption {
        &self.field_encryption
    }

    /// Get mutable reference to the underlying field encryption service.
    pub fn inner_mut(&mut self) -> &mut FieldEncryption {
        &mut self.field_encryption
    }
}

impl Default for GisFieldEncryption {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_test_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    #[test]
    fn test_field_encryption() {
        let encryption = FieldEncryption::new();
        let key = generate_test_key();
        let value = b"secret@example.com";

        let encrypted = encryption
            .encrypt_field("user.email", value, &key, "test-key", None, false)
            .expect("Encryption should succeed");

        let decrypted = encryption
            .decrypt_field(&encrypted, &key)
            .expect("Decryption should succeed");

        assert_eq!(value, decrypted.as_slice());
    }

    #[test]
    fn test_deterministic_encryption() {
        let encryption = FieldEncryption::new();
        let key = generate_test_key();
        let value = b"test@example.com";

        let encrypted1 = encryption
            .encrypt_field("email", value, &key, "test-key", None, true)
            .expect("Encryption should succeed");

        let encrypted2 = encryption
            .encrypt_field("email", value, &key, "test-key", None, true)
            .expect("Encryption should succeed");

        // Deterministic encryption should produce the same ciphertext
        assert_eq!(encrypted1.ciphertext, encrypted2.ciphertext);
        assert_eq!(encrypted1.nonce, encrypted2.nonce);
    }

    #[test]
    fn test_gis_coordinates_encryption() {
        let gis_encryption = GisFieldEncryption::new();
        let key = generate_test_key();

        let lat = 37.7749;
        let lon = -122.4194;

        let encrypted = gis_encryption
            .encrypt_coordinates(lat, lon, &key)
            .expect("Encryption should succeed");

        let (decrypted_lat, decrypted_lon) = gis_encryption
            .decrypt_coordinates(&encrypted, &key)
            .expect("Decryption should succeed");

        assert!((lat - decrypted_lat).abs() < 0.0001);
        assert!((lon - decrypted_lon).abs() < 0.0001);
    }

    #[test]
    fn test_string_encryption() {
        let encryption = FieldEncryption::new();
        let key = generate_test_key();
        let value = "sensitive data";

        let encrypted = encryption
            .encrypt_string("test.field", value, &key, "test-key")
            .expect("Encryption should succeed");

        let decrypted = encryption
            .decrypt_string(&encrypted, &key)
            .expect("Decryption should succeed");

        assert_eq!(value, decrypted);
    }
}
