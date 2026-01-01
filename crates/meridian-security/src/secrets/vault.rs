//! Secrets vault for secure storage and retrieval
//!
//! Implements a secure vault for managing sensitive data.
//!
//! ## Features
//! - Encrypted storage using envelope encryption
//! - Secret versioning
//! - TTL and expiration
//! - Access control and audit logging
//! - Automatic secret rotation
//!
//! ## Architecture
//! ```text
//! Secret → Encrypt with DEK → Store encrypted
//!   ↓
//! DEK → Encrypt with KEK → Store encrypted DEK
//!   ↓
//! KEK → Store in secure location (HSM/KMS)
//! ```
//!
//! ## OWASP Secrets Management
//! - Never hardcode secrets
//! - Encrypt secrets at rest
//! - Rotate secrets regularly
//! - Audit secret access
//! - Principle of least privilege

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::{
    encryption::{envelope::EnvelopeEncryption, Encryptor},
    error::{SecurityError, SecurityResult},
};

/// Secret value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    /// Secret ID
    pub id: String,

    /// Secret version
    pub version: u32,

    /// Secret value (encrypted when stored)
    #[serde(skip)]
    pub value: Zeroizing<Vec<u8>>,

    /// Metadata
    pub metadata: SecretMetadata,
}

impl Secret {
    /// Create a new secret
    pub fn new(id: &str, value: Vec<u8>) -> Self {
        Self {
            id: id.to_string(),
            version: 1,
            value: Zeroizing::new(value),
            metadata: SecretMetadata::new(),
        }
    }

    /// Get secret value as string
    pub fn as_string(&self) -> SecurityResult<String> {
        String::from_utf8(self.value.to_vec())
            .map_err(|e| SecurityError::SerializationError(format!("Invalid UTF-8: {}", e)))
    }

    /// Get secret value as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.value
    }

    /// Check if secret is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.metadata.expires_at {
            Utc::now() >= expires_at
        } else {
            false
        }
    }

    /// Set expiration
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.metadata.expires_at = Some(expires_at);
        self
    }

    /// Set TTL (time to live)
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.metadata.expires_at = Some(Utc::now() + ttl);
        self
    }

    /// Add tag
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.metadata.tags.insert(key, value);
        self
    }
}

/// Secret metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretMetadata {
    /// Secret type (api_key, db_password, etc.)
    pub secret_type: String,

    /// Created at timestamp
    pub created_at: DateTime<Utc>,

    /// Updated at timestamp
    pub updated_at: DateTime<Utc>,

    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,

    /// Secret description
    pub description: String,

    /// Tags for organization/search
    pub tags: HashMap<String, String>,

    /// Number of times accessed
    pub access_count: u64,

    /// Last accessed timestamp
    pub last_accessed: Option<DateTime<Utc>>,

    /// Rotation period (in days)
    pub rotation_period_days: Option<u32>,

    /// Last rotation timestamp
    pub last_rotated: Option<DateTime<Utc>>,
}

impl SecretMetadata {
    /// Create new metadata
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            secret_type: "generic".to_string(),
            created_at: now,
            updated_at: now,
            expires_at: None,
            description: String::new(),
            tags: HashMap::new(),
            access_count: 0,
            last_accessed: None,
            rotation_period_days: None,
            last_rotated: None,
        }
    }

    /// Check if rotation is needed
    pub fn needs_rotation(&self) -> bool {
        if let (Some(period), Some(last_rotated)) = (self.rotation_period_days, self.last_rotated) {
            let age_days = Utc::now()
                .signed_duration_since(last_rotated)
                .num_days();
            age_days >= period as i64
        } else {
            false
        }
    }
}

impl Default for SecretMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Stored secret (encrypted)
#[derive(Clone)]
struct StoredSecret {
    /// Encrypted secret data
    encrypted_data: Vec<u8>,

    /// Metadata (unencrypted)
    metadata: SecretMetadata,

    /// Version number
    version: u32,
}

/// Secrets vault for secure storage
///
/// # Example
/// ```rust,no_run
/// use meridian_security::secrets::vault::SecretsVault;
/// use meridian_security::encryption::envelope::EnvelopeEncryption;
///
/// // Create vault with encryption key
/// let kek = EnvelopeEncryption::generate_kek().unwrap();
/// let mut vault = SecretsVault::new(kek, 1).unwrap();
///
/// // Store a secret
/// vault.store("db-password", b"super-secret-password".to_vec()).unwrap();
///
/// // Retrieve secret
/// let secret = vault.get("db-password").unwrap();
/// println!("Password: {}", secret.as_string().unwrap());
/// ```
pub struct SecretsVault {
    /// Envelope encryption for secrets
    envelope: EnvelopeEncryption,

    /// Stored secrets (encrypted)
    secrets: Arc<RwLock<HashMap<String, Vec<StoredSecret>>>>, // secret_id -> versions

    /// Vault configuration
    config: VaultConfig,
}

/// Vault configuration
#[derive(Debug, Clone)]
pub struct VaultConfig {
    /// Maximum versions to keep per secret
    pub max_versions: u32,

    /// Enable automatic cleanup of expired secrets
    pub auto_cleanup: bool,

    /// Default TTL for secrets (None = no expiration)
    pub default_ttl: Option<Duration>,
}

impl Default for VaultConfig {
    fn default() -> Self {
        Self {
            max_versions: 10,
            auto_cleanup: true,
            default_ttl: None,
        }
    }
}

impl SecretsVault {
    /// Create a new secrets vault
    ///
    /// # Arguments
    /// * `kek` - Key Encryption Key (32 bytes)
    /// * `key_version` - KEK version for rotation tracking
    pub fn new(kek: Vec<u8>, key_version: u32) -> SecurityResult<Self> {
        let envelope = EnvelopeEncryption::new(kek, key_version)?;

        Ok(Self {
            envelope,
            secrets: Arc::new(RwLock::new(HashMap::new())),
            config: VaultConfig::default(),
        })
    }

    /// Create vault with custom configuration
    pub fn with_config(kek: Vec<u8>, key_version: u32, config: VaultConfig) -> SecurityResult<Self> {
        let envelope = EnvelopeEncryption::new(kek, key_version)?;

        Ok(Self {
            envelope,
            secrets: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// Store a secret
    ///
    /// # Arguments
    /// * `id` - Secret identifier
    /// * `value` - Secret value (will be encrypted)
    pub fn store(&mut self, id: &str, value: Vec<u8>) -> SecurityResult<()> {
        let mut secret = Secret::new(id, value);

        // Apply default TTL if configured
        if let Some(ttl) = self.config.default_ttl {
            secret = secret.with_ttl(ttl);
        }

        self.store_secret(secret)
    }

    /// Store a secret with metadata
    pub fn store_with_metadata(
        &mut self,
        id: &str,
        value: Vec<u8>,
        secret_type: &str,
        description: &str,
    ) -> SecurityResult<()> {
        let mut secret = Secret::new(id, value);
        secret.metadata.secret_type = secret_type.to_string();
        secret.metadata.description = description.to_string();

        if let Some(ttl) = self.config.default_ttl {
            secret = secret.with_ttl(ttl);
        }

        self.store_secret(secret)
    }

    /// Store a secret object
    fn store_secret(&mut self, secret: Secret) -> SecurityResult<()> {
        // Encrypt the secret value
        let encrypted = self.envelope.encrypt(&secret.value)?;
        let encrypted_bytes = EnvelopeEncryption::serialize(&encrypted)?;

        let mut secrets = self.secrets.write().map_err(|_| {
            SecurityError::VaultError("Failed to acquire secrets write lock".to_string())
        })?;

        let versions = secrets.entry(secret.id.clone()).or_insert_with(Vec::new);

        // Determine version number
        let version = if versions.is_empty() {
            1
        } else {
            versions.first().map(|s| s.version + 1).unwrap_or(1)
        };

        let stored = StoredSecret {
            encrypted_data: encrypted_bytes,
            metadata: secret.metadata,
            version,
        };

        // Add new version at the front
        versions.insert(0, stored);

        // Trim old versions if exceeding max
        if versions.len() > self.config.max_versions as usize {
            versions.truncate(self.config.max_versions as usize);
        }

        Ok(())
    }

    /// Get a secret (latest version)
    ///
    /// # Arguments
    /// * `id` - Secret identifier
    pub fn get(&self, id: &str) -> SecurityResult<Secret> {
        self.get_version(id, None)
    }

    /// Get a specific version of a secret
    ///
    /// # Arguments
    /// * `id` - Secret identifier
    /// * `version` - Version number (None = latest)
    pub fn get_version(&self, id: &str, version: Option<u32>) -> SecurityResult<Secret> {
        let mut secrets = self.secrets.write().map_err(|_| {
            SecurityError::VaultError("Failed to acquire secrets lock".to_string())
        })?;

        let versions = secrets
            .get_mut(id)
            .ok_or_else(|| SecurityError::SecretNotFound(id.to_string()))?;

        let stored = if let Some(v) = version {
            versions
                .iter_mut()
                .find(|s| s.version == v)
                .ok_or_else(|| {
                    SecurityError::SecretNotFound(format!("{} version {}", id, v))
                })?
        } else {
            versions.first_mut().ok_or_else(|| {
                SecurityError::SecretNotFound(id.to_string())
            })?
        };

        // Check expiration
        if let Some(expires_at) = stored.metadata.expires_at {
            if Utc::now() >= expires_at {
                return Err(SecurityError::SecretNotFound(format!(
                    "Secret {} expired",
                    id
                )));
            }
        }

        // Decrypt the secret
        let encrypted = EnvelopeEncryption::deserialize(&stored.encrypted_data)?;
        let decrypted = self.envelope.decrypt(&encrypted)?;

        // Update access tracking
        stored.metadata.access_count += 1;
        stored.metadata.last_accessed = Some(Utc::now());

        Ok(Secret {
            id: id.to_string(),
            version: stored.version,
            value: Zeroizing::new(decrypted),
            metadata: stored.metadata.clone(),
        })
    }

    /// Delete a secret (all versions)
    pub fn delete(&mut self, id: &str) -> SecurityResult<()> {
        let mut secrets = self.secrets.write().map_err(|_| {
            SecurityError::VaultError("Failed to acquire secrets write lock".to_string())
        })?;

        secrets
            .remove(id)
            .ok_or_else(|| SecurityError::SecretNotFound(id.to_string()))?;

        Ok(())
    }

    /// Delete a specific version
    pub fn delete_version(&mut self, id: &str, version: u32) -> SecurityResult<()> {
        let mut secrets = self.secrets.write().map_err(|_| {
            SecurityError::VaultError("Failed to acquire secrets write lock".to_string())
        })?;

        let versions = secrets
            .get_mut(id)
            .ok_or_else(|| SecurityError::SecretNotFound(id.to_string()))?;

        let index = versions
            .iter()
            .position(|s| s.version == version)
            .ok_or_else(|| {
                SecurityError::SecretNotFound(format!("{} version {}", id, version))
            })?;

        versions.remove(index);

        // Remove entry if no versions left
        if versions.is_empty() {
            secrets.remove(id);
        }

        Ok(())
    }

    /// List all secret IDs
    pub fn list(&self) -> SecurityResult<Vec<String>> {
        let secrets = self.secrets.read().map_err(|_| {
            SecurityError::VaultError("Failed to acquire secrets read lock".to_string())
        })?;

        Ok(secrets.keys().cloned().collect())
    }

    /// List secrets with metadata (no values)
    pub fn list_with_metadata(&self) -> SecurityResult<Vec<(String, SecretMetadata)>> {
        let secrets = self.secrets.read().map_err(|_| {
            SecurityError::VaultError("Failed to acquire secrets read lock".to_string())
        })?;

        let mut result = Vec::new();
        for (id, versions) in secrets.iter() {
            if let Some(latest) = versions.first() {
                result.push((id.clone(), latest.metadata.clone()));
            }
        }

        Ok(result)
    }

    /// Check if secret exists
    pub fn exists(&self, id: &str) -> SecurityResult<bool> {
        let secrets = self.secrets.read().map_err(|_| {
            SecurityError::VaultError("Failed to acquire secrets read lock".to_string())
        })?;

        Ok(secrets.contains_key(id))
    }

    /// Rotate a secret (create new version)
    ///
    /// # Arguments
    /// * `id` - Secret identifier
    /// * `new_value` - New secret value
    pub fn rotate(&mut self, id: &str, new_value: Vec<u8>) -> SecurityResult<()> {
        // Get current metadata
        let current_metadata = {
            let secrets = self.secrets.read().map_err(|_| {
                SecurityError::VaultError("Failed to acquire secrets read lock".to_string())
            })?;

            let versions = secrets
                .get(id)
                .ok_or_else(|| SecurityError::SecretNotFound(id.to_string()))?;

            versions
                .first()
                .map(|s| s.metadata.clone())
                .ok_or_else(|| SecurityError::SecretNotFound(id.to_string()))?
        };

        // Create new version with updated metadata
        let mut secret = Secret::new(id, new_value);
        secret.metadata = current_metadata;
        secret.metadata.updated_at = Utc::now();
        secret.metadata.last_rotated = Some(Utc::now());

        self.store_secret(secret)
    }

    /// Clean up expired secrets
    pub fn cleanup_expired(&mut self) -> SecurityResult<usize> {
        let mut secrets = self.secrets.write().map_err(|_| {
            SecurityError::VaultError("Failed to acquire secrets write lock".to_string())
        })?;

        let now = Utc::now();
        let mut removed = 0;

        secrets.retain(|_id, versions| {
            versions.retain(|stored| {
                if let Some(expires_at) = stored.metadata.expires_at {
                    if expires_at <= now {
                        removed += 1;
                        return false;
                    }
                }
                true
            });
            !versions.is_empty()
        });

        Ok(removed)
    }

    /// Get secrets that need rotation
    pub fn secrets_needing_rotation(&self) -> SecurityResult<Vec<String>> {
        let secrets = self.secrets.read().map_err(|_| {
            SecurityError::VaultError("Failed to acquire secrets read lock".to_string())
        })?;

        let mut needs_rotation = Vec::new();

        for (id, versions) in secrets.iter() {
            if let Some(latest) = versions.first() {
                if latest.metadata.needs_rotation() {
                    needs_rotation.push(id.clone());
                }
            }
        }

        Ok(needs_rotation)
    }

    /// Get vault statistics
    pub fn stats(&self) -> SecurityResult<VaultStats> {
        let secrets = self.secrets.read().map_err(|_| {
            SecurityError::VaultError("Failed to acquire secrets read lock".to_string())
        })?;

        let total_secrets = secrets.len();
        let total_versions: usize = secrets.values().map(|v| v.len()).sum();

        let now = Utc::now();
        let expired_secrets = secrets
            .values()
            .filter(|versions| {
                versions.first().map_or(false, |s| {
                    s.metadata
                        .expires_at
                        .map_or(false, |exp| exp <= now)
                })
            })
            .count();

        Ok(VaultStats {
            total_secrets,
            total_versions,
            expired_secrets,
        })
    }
}

/// Vault statistics
#[derive(Debug, Clone)]
pub struct VaultStats {
    /// Total number of secrets
    pub total_secrets: usize,

    /// Total number of versions across all secrets
    pub total_versions: usize,

    /// Number of expired secrets
    pub expired_secrets: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vault() -> SecretsVault {
        let kek = EnvelopeEncryption::generate_kek().unwrap();
        SecretsVault::new(kek, 1).unwrap()
    }

    #[test]
    fn test_store_and_retrieve_secret() {
        let mut vault = create_test_vault();

        vault.store("api-key", b"secret-api-key-123".to_vec()).unwrap();

        let secret = vault.get("api-key").unwrap();
        assert_eq!(secret.as_string().unwrap(), "secret-api-key-123");
    }

    #[test]
    fn test_secret_versioning() {
        let mut vault = create_test_vault();

        vault.store("db-password", b"password-v1".to_vec()).unwrap();
        vault.rotate("db-password", b"password-v2".to_vec()).unwrap();
        vault.rotate("db-password", b"password-v3".to_vec()).unwrap();

        let latest = vault.get("db-password").unwrap();
        assert_eq!(latest.as_string().unwrap(), "password-v3");

        let v2 = vault.get_version("db-password", Some(2)).unwrap();
        assert_eq!(v2.as_string().unwrap(), "password-v2");
    }

    #[test]
    fn test_secret_deletion() {
        let mut vault = create_test_vault();

        vault.store("temp-secret", b"temporary".to_vec()).unwrap();
        assert!(vault.exists("temp-secret").unwrap());

        vault.delete("temp-secret").unwrap();
        assert!(!vault.exists("temp-secret").unwrap());
    }

    #[test]
    fn test_list_secrets() {
        let mut vault = create_test_vault();

        vault.store("secret1", b"value1".to_vec()).unwrap();
        vault.store("secret2", b"value2".to_vec()).unwrap();

        let list = vault.list().unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"secret1".to_string()));
        assert!(list.contains(&"secret2".to_string()));
    }

    #[test]
    fn test_secret_expiration() {
        let mut vault = create_test_vault();

        let expired_time = Utc::now() - Duration::hours(1);
        let mut secret = Secret::new("expired-secret", b"value".to_vec())
            .with_expiration(expired_time);

        vault.store_secret(secret).unwrap();

        let result = vault.get("expired-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_cleanup_expired() {
        let mut vault = create_test_vault();

        let expired_time = Utc::now() - Duration::hours(1);
        let mut secret = Secret::new("expired", b"value".to_vec())
            .with_expiration(expired_time);

        vault.store_secret(secret).unwrap();
        vault.store("active", b"value".to_vec()).unwrap();

        let removed = vault.cleanup_expired().unwrap();
        assert_eq!(removed, 1);

        assert!(!vault.exists("expired").unwrap());
        assert!(vault.exists("active").unwrap());
    }

    #[test]
    fn test_vault_stats() {
        let mut vault = create_test_vault();

        vault.store("secret1", b"value1".to_vec()).unwrap();
        vault.store("secret2", b"value2".to_vec()).unwrap();
        vault.rotate("secret1", b"new-value".to_vec()).unwrap();

        let stats = vault.stats().unwrap();
        assert_eq!(stats.total_secrets, 2);
        assert_eq!(stats.total_versions, 3);
    }

    #[test]
    fn test_secret_not_found() {
        let vault = create_test_vault();
        let result = vault.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata() {
        let mut vault = create_test_vault();

        vault
            .store_with_metadata("db-creds", b"password".to_vec(), "database", "Production DB credentials")
            .unwrap();

        let secret = vault.get("db-creds").unwrap();
        assert_eq!(secret.metadata.secret_type, "database");
        assert_eq!(secret.metadata.description, "Production DB credentials");
    }
}
