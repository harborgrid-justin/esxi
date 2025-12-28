//! Key Management Service (KMS) abstraction layer.
//!
//! This module provides a unified interface for interacting with various KMS providers,
//! including AWS KMS and HashiCorp Vault.

#[cfg(feature = "aws-kms")]
pub mod aws;

#[cfg(feature = "vault")]
pub mod vault;

use crate::error::{CryptoError, CryptoResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Key metadata information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Unique key identifier.
    pub key_id: String,

    /// Key alias or name.
    pub alias: Option<String>,

    /// Key state (enabled, disabled, pending deletion, etc.).
    pub state: KeyState,

    /// Key algorithm.
    pub algorithm: String,

    /// Key usage (encryption, signing, etc.).
    pub usage: KeyUsage,

    /// Creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last rotation timestamp.
    pub last_rotated_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Deletion timestamp (if scheduled for deletion).
    pub deletion_date: Option<chrono::DateTime<chrono::Utc>>,

    /// Key description.
    pub description: Option<String>,

    /// Custom tags/labels.
    pub tags: HashMap<String, String>,
}

/// Key state enumeration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyState {
    /// Key is enabled and can be used.
    Enabled,

    /// Key is disabled and cannot be used.
    Disabled,

    /// Key is scheduled for deletion.
    PendingDeletion,

    /// Key is being imported.
    PendingImport,

    /// Key has been deleted.
    Deleted,

    /// Key state is unavailable or unknown.
    Unavailable,
}

/// Key usage type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyUsage {
    /// Key is used for encryption and decryption.
    EncryptDecrypt,

    /// Key is used for signing and verification.
    SignVerify,

    /// Key is used for key wrapping.
    WrapUnwrap,
}

/// Encryption context for additional authenticated data.
pub type EncryptionContext = HashMap<String, String>;

/// Key Management Service trait.
///
/// This trait defines the interface that all KMS providers must implement.
#[async_trait]
pub trait KeyManagementService: Send + Sync {
    /// Generate a new data encryption key.
    ///
    /// Returns both the plaintext and encrypted versions of the key.
    async fn generate_data_key(
        &self,
        key_id: &str,
        key_spec: KeySpec,
        encryption_context: Option<&EncryptionContext>,
    ) -> CryptoResult<DataKeyPair>;

    /// Encrypt plaintext data using a KMS key.
    async fn encrypt(
        &self,
        key_id: &str,
        plaintext: &[u8],
        encryption_context: Option<&EncryptionContext>,
    ) -> CryptoResult<Vec<u8>>;

    /// Decrypt ciphertext using a KMS key.
    async fn decrypt(
        &self,
        key_id: &str,
        ciphertext: &[u8],
        encryption_context: Option<&EncryptionContext>,
    ) -> CryptoResult<Vec<u8>>;

    /// Create a new master key.
    async fn create_key(
        &self,
        description: Option<&str>,
        key_usage: KeyUsage,
        tags: Option<&HashMap<String, String>>,
    ) -> CryptoResult<KeyMetadata>;

    /// Describe a key and get its metadata.
    async fn describe_key(&self, key_id: &str) -> CryptoResult<KeyMetadata>;

    /// List all keys (with optional pagination).
    async fn list_keys(&self, limit: Option<usize>) -> CryptoResult<Vec<String>>;

    /// Enable a key.
    async fn enable_key(&self, key_id: &str) -> CryptoResult<()>;

    /// Disable a key.
    async fn disable_key(&self, key_id: &str) -> CryptoResult<()>;

    /// Schedule a key for deletion.
    async fn schedule_key_deletion(&self, key_id: &str, pending_days: u32) -> CryptoResult<()>;

    /// Cancel scheduled key deletion.
    async fn cancel_key_deletion(&self, key_id: &str) -> CryptoResult<()>;

    /// Rotate a key.
    async fn rotate_key(&self, key_id: &str) -> CryptoResult<()>;

    /// Create a key alias.
    async fn create_alias(&self, alias: &str, key_id: &str) -> CryptoResult<()>;

    /// Delete a key alias.
    async fn delete_alias(&self, alias: &str) -> CryptoResult<()>;

    /// Update key tags.
    async fn tag_key(&self, key_id: &str, tags: &HashMap<String, String>) -> CryptoResult<()>;

    /// Remove key tags.
    async fn untag_key(&self, key_id: &str, tag_keys: &[String]) -> CryptoResult<()>;

    /// Sign data using a KMS key (for asymmetric keys).
    async fn sign(
        &self,
        key_id: &str,
        message: &[u8],
        signing_algorithm: SigningAlgorithm,
    ) -> CryptoResult<Vec<u8>>;

    /// Verify a signature using a KMS key (for asymmetric keys).
    async fn verify(
        &self,
        key_id: &str,
        message: &[u8],
        signature: &[u8],
        signing_algorithm: SigningAlgorithm,
    ) -> CryptoResult<bool>;

    /// Get the public key for an asymmetric key pair.
    async fn get_public_key(&self, key_id: &str) -> CryptoResult<Vec<u8>>;
}

/// Data key pair containing both plaintext and encrypted versions.
#[derive(Clone, Serialize, Deserialize)]
pub struct DataKeyPair {
    /// The plaintext data key.
    pub plaintext: Vec<u8>,

    /// The encrypted data key (encrypted with the master key).
    pub ciphertext: Vec<u8>,

    /// The ID of the master key used to encrypt this data key.
    pub key_id: String,
}

/// Key specification for generating data keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeySpec {
    /// AES-256 key (32 bytes).
    Aes256,

    /// AES-128 key (16 bytes).
    Aes128,

    /// RSA-2048 key pair.
    Rsa2048,

    /// RSA-4096 key pair.
    Rsa4096,

    /// ECC NIST P-256 key pair.
    EccNistP256,

    /// ECC NIST P-384 key pair.
    EccNistP384,

    /// Custom key size in bytes.
    Custom(usize),
}

impl KeySpec {
    /// Get the size of the key in bytes.
    pub fn size_bytes(&self) -> usize {
        match self {
            KeySpec::Aes256 => 32,
            KeySpec::Aes128 => 16,
            KeySpec::Rsa2048 => 256,
            KeySpec::Rsa4096 => 512,
            KeySpec::EccNistP256 => 32,
            KeySpec::EccNistP384 => 48,
            KeySpec::Custom(size) => *size,
        }
    }
}

/// Signing algorithm for digital signatures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SigningAlgorithm {
    /// RSASSA-PSS with SHA-256.
    RsassaPssSha256,

    /// RSASSA-PSS with SHA-384.
    RsassaPssSha384,

    /// RSASSA-PSS with SHA-512.
    RsassaPssSha512,

    /// RSASSA-PKCS1-v1_5 with SHA-256.
    RsassaPkcs1V15Sha256,

    /// RSASSA-PKCS1-v1_5 with SHA-384.
    RsassaPkcs1V15Sha384,

    /// RSASSA-PKCS1-v1_5 with SHA-512.
    RsassaPkcs1V15Sha512,

    /// ECDSA with SHA-256.
    EcdsaSha256,

    /// ECDSA with SHA-384.
    EcdsaSha384,

    /// ECDSA with SHA-512.
    EcdsaSha512,
}

/// KMS provider type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KmsProvider {
    /// AWS Key Management Service.
    AwsKms,

    /// HashiCorp Vault.
    Vault,

    /// Local key storage (for testing only).
    Local,
}

/// KMS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KmsConfig {
    /// KMS provider type.
    pub provider: KmsProvider,

    /// Provider-specific configuration.
    pub config: HashMap<String, String>,
}

impl KmsConfig {
    /// Create a new KMS configuration.
    pub fn new(provider: KmsProvider) -> Self {
        Self {
            provider,
            config: HashMap::new(),
        }
    }

    /// Set a configuration parameter.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.config.insert(key.into(), value.into());
        self
    }

    /// Get a configuration parameter.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.config.get(key).map(|s| s.as_str())
    }
}
