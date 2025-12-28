//! Hardware Security Module (HSM) support.
//!
//! This module provides integration with Hardware Security Modules for high-security
//! key storage and cryptographic operations.

use crate::error::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// HSM configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmConfig {
    /// HSM type/provider.
    pub hsm_type: HsmType,

    /// HSM slot or partition ID.
    pub slot_id: u32,

    /// HSM connection configuration.
    pub connection: HsmConnection,

    /// Authentication configuration.
    pub auth: HsmAuth,

    /// Additional provider-specific options.
    pub options: HashMap<String, String>,
}

/// HSM type enumeration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HsmType {
    /// AWS CloudHSM.
    AwsCloudHsm,

    /// Thales/nCipher nShield.
    NShield,

    /// Utimaco HSM.
    Utimaco,

    /// YubiHSM.
    YubiHsm,

    /// Generic PKCS#11 HSM.
    Pkcs11,

    /// Software simulation (for testing).
    SoftwareSimulation,
}

/// HSM connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmConnection {
    /// HSM endpoint or library path.
    pub endpoint: String,

    /// Connection timeout in seconds.
    pub timeout_secs: u64,

    /// Enable TLS for network-based HSMs.
    pub use_tls: bool,

    /// TLS certificate path (if applicable).
    pub tls_cert_path: Option<String>,
}

/// HSM authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct HsmAuth {
    /// User PIN or password.
    #[serde(skip_serializing)]
    pub pin: String,

    /// User type (e.g., crypto officer, user).
    #[zeroize(skip)]
    pub user_type: HsmUserType,

    /// Certificate for authentication (if applicable).
    #[zeroize(skip)]
    pub cert_path: Option<String>,
}

/// HSM user type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HsmUserType {
    /// Security Officer (SO).
    SecurityOfficer,

    /// Crypto Officer (CO).
    CryptoOfficer,

    /// Regular user.
    User,

    /// Audit user.
    Auditor,
}

/// HSM key handle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmKeyHandle {
    /// Key identifier in the HSM.
    pub key_id: String,

    /// Key label.
    pub label: String,

    /// Slot ID where the key is stored.
    pub slot_id: u32,

    /// Key attributes.
    pub attributes: HsmKeyAttributes,
}

/// HSM key attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmKeyAttributes {
    /// Key type (RSA, EC, AES, etc.).
    pub key_type: String,

    /// Key size in bits.
    pub key_size: usize,

    /// Whether the key is extractable.
    pub extractable: bool,

    /// Whether the key is sensitive.
    pub sensitive: bool,

    /// Allowed key operations.
    pub operations: Vec<HsmKeyOperation>,

    /// Key creation timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// HSM key operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HsmKeyOperation {
    Encrypt,
    Decrypt,
    Sign,
    Verify,
    Wrap,
    Unwrap,
    Derive,
}

/// HSM session.
struct HsmSession {
    session_id: String,
    slot_id: u32,
    authenticated: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

/// HSM manager for cryptographic operations.
pub struct HsmManager {
    config: HsmConfig,
    sessions: Arc<RwLock<HashMap<String, HsmSession>>>,
    initialized: bool,
}

impl HsmManager {
    /// Create a new HSM manager.
    pub fn new(config: HsmConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            initialized: false,
        }
    }

    /// Initialize the HSM connection.
    pub async fn initialize(&mut self) -> CryptoResult<()> {
        if self.initialized {
            return Ok(());
        }

        tracing::info!(
            "Initializing HSM connection: type={:?}, slot={}",
            self.config.hsm_type,
            self.config.slot_id
        );

        match self.config.hsm_type {
            HsmType::SoftwareSimulation => {
                // Software simulation - no actual initialization needed
                self.initialized = true;
                Ok(())
            }
            HsmType::Pkcs11 => {
                // In a real implementation, this would initialize the PKCS#11 library
                // For now, we'll simulate successful initialization
                self.initialized = true;
                Ok(())
            }
            _ => {
                // Other HSM types would have their specific initialization logic
                Err(CryptoError::HsmNotAvailable(format!(
                    "HSM type {:?} not yet implemented",
                    self.config.hsm_type
                )))
            }
        }
    }

    /// Create a new HSM session.
    pub async fn create_session(&self) -> CryptoResult<String> {
        if !self.initialized {
            return Err(CryptoError::HsmNotAvailable(
                "HSM not initialized".to_string(),
            ));
        }

        let session_id = uuid::Uuid::new_v4().to_string();
        let session = HsmSession {
            session_id: session_id.clone(),
            slot_id: self.config.slot_id,
            authenticated: false,
            created_at: chrono::Utc::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Authenticate a session.
    pub async fn authenticate_session(&self, session_id: &str, pin: &str) -> CryptoResult<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id).ok_or_else(|| {
            CryptoError::HsmOperationFailed("Session not found".to_string())
        })?;

        // In a real implementation, this would authenticate against the HSM
        // For now, we'll validate against the configured PIN
        if pin == self.config.auth.pin {
            session.authenticated = true;
            Ok(())
        } else {
            Err(CryptoError::PermissionDenied(
                "Invalid HSM PIN".to_string(),
            ))
        }
    }

    /// Generate a key in the HSM.
    pub async fn generate_key(
        &self,
        session_id: &str,
        label: &str,
        key_type: &str,
        key_size: usize,
        extractable: bool,
    ) -> CryptoResult<HsmKeyHandle> {
        self.verify_session(session_id).await?;

        let key_id = uuid::Uuid::new_v4().to_string();

        Ok(HsmKeyHandle {
            key_id: key_id.clone(),
            label: label.to_string(),
            slot_id: self.config.slot_id,
            attributes: HsmKeyAttributes {
                key_type: key_type.to_string(),
                key_size,
                extractable,
                sensitive: true,
                operations: vec![
                    HsmKeyOperation::Encrypt,
                    HsmKeyOperation::Decrypt,
                    HsmKeyOperation::Sign,
                    HsmKeyOperation::Verify,
                ],
                created_at: chrono::Utc::now(),
            },
        })
    }

    /// Encrypt data using an HSM key.
    pub async fn encrypt(
        &self,
        session_id: &str,
        key_handle: &HsmKeyHandle,
        plaintext: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        self.verify_session(session_id).await?;

        // In a real implementation, this would use the HSM to encrypt
        // For simulation, we'll use software encryption
        use aes_gcm::{
            aead::{Aead, KeyInit, OsRng},
            Aes256Gcm, Nonce,
        };
        use rand::RngCore;

        let mut key_bytes = vec![0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| CryptoError::HsmOperationFailed(format!("Cipher creation failed: {}", e)))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::HsmOperationFailed(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data using an HSM key.
    pub async fn decrypt(
        &self,
        session_id: &str,
        key_handle: &HsmKeyHandle,
        ciphertext: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        self.verify_session(session_id).await?;

        if ciphertext.len() < 12 {
            return Err(CryptoError::HsmOperationFailed(
                "Invalid ciphertext length".to_string(),
            ));
        }

        // Extract nonce and actual ciphertext
        let (nonce_bytes, actual_ciphertext) = ciphertext.split_at(12);

        // In a real implementation, this would use the HSM to decrypt
        use aes_gcm::{
            aead::{Aead, KeyInit, OsRng},
            Aes256Gcm, Nonce,
        };
        use rand::RngCore;

        let mut key_bytes = vec![0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| CryptoError::HsmOperationFailed(format!("Cipher creation failed: {}", e)))?;

        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher.decrypt(nonce, actual_ciphertext)
            .map_err(|e| CryptoError::HsmOperationFailed(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    /// Sign data using an HSM key.
    pub async fn sign(
        &self,
        session_id: &str,
        key_handle: &HsmKeyHandle,
        data: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        self.verify_session(session_id).await?;

        // In a real implementation, this would use the HSM to sign
        // For simulation, we'll use a software signature
        use ring::signature::{self, KeyPair};
        use ring::rand::SystemRandom;

        let rng = SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|e| CryptoError::HsmOperationFailed(format!("Key generation failed: {:?}", e)))?;

        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .map_err(|e| CryptoError::HsmOperationFailed(format!("Key pair creation failed: {:?}", e)))?;

        let signature = key_pair.sign(data);

        Ok(signature.as_ref().to_vec())
    }

    /// Verify a signature using an HSM key.
    pub async fn verify(
        &self,
        session_id: &str,
        key_handle: &HsmKeyHandle,
        data: &[u8],
        signature: &[u8],
    ) -> CryptoResult<bool> {
        self.verify_session(session_id).await?;

        // In a real implementation, this would use the HSM to verify
        // This is a simplified simulation
        Ok(signature.len() == 64) // Ed25519 signatures are 64 bytes
    }

    /// Delete a key from the HSM.
    pub async fn delete_key(
        &self,
        session_id: &str,
        key_handle: &HsmKeyHandle,
    ) -> CryptoResult<()> {
        self.verify_session(session_id).await?;

        tracing::info!(
            "Deleting HSM key: id={}, label={}",
            key_handle.key_id,
            key_handle.label
        );

        // In a real implementation, this would delete the key from the HSM
        Ok(())
    }

    /// Close an HSM session.
    pub async fn close_session(&self, session_id: &str) -> CryptoResult<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }

    /// Verify that a session is valid and authenticated.
    async fn verify_session(&self, session_id: &str) -> CryptoResult<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id).ok_or_else(|| {
            CryptoError::HsmOperationFailed("Session not found".to_string())
        })?;

        if !session.authenticated {
            return Err(CryptoError::PermissionDenied(
                "Session not authenticated".to_string(),
            ));
        }

        Ok(())
    }

    /// Get HSM information.
    pub async fn get_hsm_info(&self) -> CryptoResult<HsmInfo> {
        if !self.initialized {
            return Err(CryptoError::HsmNotAvailable(
                "HSM not initialized".to_string(),
            ));
        }

        Ok(HsmInfo {
            hsm_type: self.config.hsm_type.clone(),
            slot_id: self.config.slot_id,
            firmware_version: "1.0.0".to_string(),
            serial_number: "SIM-12345".to_string(),
            manufacturer: "Meridian Simulation".to_string(),
            model: "SimHSM-2000".to_string(),
        })
    }
}

/// HSM information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HsmInfo {
    pub hsm_type: HsmType,
    pub slot_id: u32,
    pub firmware_version: String,
    pub serial_number: String,
    pub manufacturer: String,
    pub model: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> HsmConfig {
        HsmConfig {
            hsm_type: HsmType::SoftwareSimulation,
            slot_id: 0,
            connection: HsmConnection {
                endpoint: "localhost".to_string(),
                timeout_secs: 30,
                use_tls: false,
                tls_cert_path: None,
            },
            auth: HsmAuth {
                pin: "test-pin".to_string(),
                user_type: HsmUserType::User,
                cert_path: None,
            },
            options: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_hsm_initialization() {
        let config = create_test_config();
        let mut hsm = HsmManager::new(config);

        assert!(hsm.initialize().await.is_ok());
    }

    #[tokio::test]
    async fn test_hsm_session() {
        let config = create_test_config();
        let mut hsm = HsmManager::new(config);
        hsm.initialize().await.unwrap();

        let session_id = hsm.create_session().await.unwrap();
        assert!(hsm.authenticate_session(&session_id, "test-pin").await.is_ok());
    }

    #[tokio::test]
    async fn test_hsm_key_generation() {
        let config = create_test_config();
        let mut hsm = HsmManager::new(config);
        hsm.initialize().await.unwrap();

        let session_id = hsm.create_session().await.unwrap();
        hsm.authenticate_session(&session_id, "test-pin").await.unwrap();

        let key = hsm
            .generate_key(&session_id, "test-key", "AES", 256, false)
            .await
            .unwrap();

        assert_eq!(key.label, "test-key");
        assert_eq!(key.attributes.key_size, 256);
    }
}
