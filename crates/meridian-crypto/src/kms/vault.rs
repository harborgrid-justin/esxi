//! HashiCorp Vault Key Management Service implementation.

#[cfg(feature = "vault")]
use crate::error::{CryptoError, CryptoResult};
use crate::kms::{
    DataKeyPair, EncryptionContext, KeyManagementService, KeyMetadata, KeySpec, KeyState,
    KeyUsage, SigningAlgorithm,
};
use async_trait::async_trait;
use std::collections::HashMap;

#[cfg(feature = "vault")]
use vaultrs::{
    api::transit::{
        requests::{
            CreateKeyRequest, DecryptDataRequest, EncryptDataRequest, GenerateDataKeyRequest,
            SignDataRequest, VerifySignedDataRequest,
        },
        responses::{KeyData, KeyType},
    },
    client::{VaultClient, VaultClientSettingsBuilder},
};

/// HashiCorp Vault KMS client.
#[cfg(feature = "vault")]
pub struct VaultKms {
    client: VaultClient,
    mount_path: String,
}

#[cfg(feature = "vault")]
impl VaultKms {
    /// Create a new Vault KMS client.
    pub async fn new(
        vault_addr: &str,
        token: &str,
        mount_path: Option<&str>,
    ) -> CryptoResult<Self> {
        let settings = VaultClientSettingsBuilder::default()
            .address(vault_addr)
            .token(token)
            .build()
            .map_err(|e| CryptoError::VaultError(format!("Failed to build Vault client: {}", e)))?;

        let client = VaultClient::new(settings)
            .map_err(|e| CryptoError::VaultError(format!("Failed to create Vault client: {}", e)))?;

        Ok(Self {
            client,
            mount_path: mount_path.unwrap_or("transit").to_string(),
        })
    }

    /// Create a new Vault KMS client with custom settings.
    pub fn with_client(client: VaultClient, mount_path: String) -> Self {
        Self { client, mount_path }
    }

    /// Convert KeySpec to Vault key type.
    fn to_vault_key_type(spec: KeySpec) -> String {
        match spec {
            KeySpec::Aes256 | KeySpec::Aes128 => "aes256-gcm96".to_string(),
            KeySpec::Rsa2048 => "rsa-2048".to_string(),
            KeySpec::Rsa4096 => "rsa-4096".to_string(),
            KeySpec::EccNistP256 => "ecdsa-p256".to_string(),
            KeySpec::EccNistP384 => "ecdsa-p384".to_string(),
            KeySpec::Custom(_) => "aes256-gcm96".to_string(),
        }
    }

    /// Build context string from encryption context.
    fn build_context(context: Option<&EncryptionContext>) -> Option<String> {
        context.map(|ctx| {
            let json = serde_json::to_string(ctx).unwrap_or_default();
            base64::encode(json.as_bytes())
        })
    }
}

#[cfg(feature = "vault")]
#[async_trait]
impl KeyManagementService for VaultKms {
    async fn generate_data_key(
        &self,
        key_id: &str,
        key_spec: KeySpec,
        encryption_context: Option<&EncryptionContext>,
    ) -> CryptoResult<DataKeyPair> {
        let bits = (key_spec.size_bytes() * 8) as u64;

        let mut request = GenerateDataKeyRequest::builder()
            .bits(bits);

        if let Some(ctx_str) = Self::build_context(encryption_context) {
            request = request.context(ctx_str);
        }

        let response = vaultrs::api::transit::data::generate(
            &self.client,
            &self.mount_path,
            key_id,
            Some(&mut request.build().unwrap()),
        )
        .await
        .map_err(|e| CryptoError::VaultError(format!("Failed to generate data key: {}", e)))?;

        let plaintext = base64::decode(&response.plaintext).map_err(|e| {
            CryptoError::VaultError(format!("Failed to decode plaintext key: {}", e))
        })?;

        Ok(DataKeyPair {
            plaintext,
            ciphertext: response.ciphertext.into_bytes(),
            key_id: key_id.to_string(),
        })
    }

    async fn encrypt(
        &self,
        key_id: &str,
        plaintext: &[u8],
        encryption_context: Option<&EncryptionContext>,
    ) -> CryptoResult<Vec<u8>> {
        let plaintext_b64 = base64::encode(plaintext);

        let mut request = EncryptDataRequest::builder()
            .plaintext(plaintext_b64);

        if let Some(ctx_str) = Self::build_context(encryption_context) {
            request = request.context(ctx_str);
        }

        let response = vaultrs::api::transit::data::encrypt(
            &self.client,
            &self.mount_path,
            key_id,
            Some(&mut request.build().unwrap()),
        )
        .await
        .map_err(|e| CryptoError::VaultError(format!("Encryption failed: {}", e)))?;

        Ok(response.ciphertext.into_bytes())
    }

    async fn decrypt(
        &self,
        key_id: &str,
        ciphertext: &[u8],
        encryption_context: Option<&EncryptionContext>,
    ) -> CryptoResult<Vec<u8>> {
        let ciphertext_str =
            String::from_utf8(ciphertext.to_vec()).map_err(|e| {
                CryptoError::VaultError(format!("Invalid ciphertext format: {}", e))
            })?;

        let mut request = DecryptDataRequest::builder()
            .ciphertext(ciphertext_str);

        if let Some(ctx_str) = Self::build_context(encryption_context) {
            request = request.context(ctx_str);
        }

        let response = vaultrs::api::transit::data::decrypt(
            &self.client,
            &self.mount_path,
            key_id,
            Some(&mut request.build().unwrap()),
        )
        .await
        .map_err(|e| CryptoError::VaultError(format!("Decryption failed: {}", e)))?;

        base64::decode(&response.plaintext)
            .map_err(|e| CryptoError::VaultError(format!("Failed to decode plaintext: {}", e)))
    }

    async fn create_key(
        &self,
        description: Option<&str>,
        key_usage: KeyUsage,
        _tags: Option<&HashMap<String, String>>,
    ) -> CryptoResult<KeyMetadata> {
        let key_type = match key_usage {
            KeyUsage::EncryptDecrypt => "aes256-gcm96",
            KeyUsage::SignVerify => "ecdsa-p256",
            KeyUsage::WrapUnwrap => "aes256-gcm96",
        };

        let mut request = CreateKeyRequest::builder()
            .key_type(key_type);

        if let Some(desc) = description {
            // Note: Vault doesn't have a description field, but we can store it in exportable
            // metadata if needed
            tracing::debug!("Key description: {}", desc);
        }

        vaultrs::api::transit::key::create(
            &self.client,
            &self.mount_path,
            "new-key",
            Some(&mut request.build().unwrap()),
        )
        .await
        .map_err(|e| CryptoError::VaultError(format!("Failed to create key: {}", e)))?;

        Ok(KeyMetadata {
            key_id: "new-key".to_string(),
            alias: None,
            state: KeyState::Enabled,
            algorithm: key_type.to_string(),
            usage: key_usage,
            created_at: chrono::Utc::now(),
            last_rotated_at: None,
            deletion_date: None,
            description: description.map(String::from),
            tags: HashMap::new(),
        })
    }

    async fn describe_key(&self, key_id: &str) -> CryptoResult<KeyMetadata> {
        let response = vaultrs::api::transit::key::read(&self.client, &self.mount_path, key_id)
            .await
            .map_err(|e| CryptoError::VaultError(format!("Failed to describe key: {}", e)))?;

        let state = if response.deletion_allowed {
            KeyState::Enabled
        } else {
            KeyState::Enabled
        };

        let usage = match response.key_type {
            KeyType::Aes128Gcm96 | KeyType::Aes256Gcm96 | KeyType::ChaCha20Poly1305 => {
                KeyUsage::EncryptDecrypt
            }
            KeyType::Ed25519
            | KeyType::EcdsaP256
            | KeyType::EcdsaP384
            | KeyType::EcdsaP521
            | KeyType::Rsa2048
            | KeyType::Rsa3072
            | KeyType::Rsa4096 => KeyUsage::SignVerify,
            _ => KeyUsage::EncryptDecrypt,
        };

        Ok(KeyMetadata {
            key_id: key_id.to_string(),
            alias: None,
            state,
            algorithm: format!("{:?}", response.key_type),
            usage,
            created_at: chrono::Utc::now(),
            last_rotated_at: None,
            deletion_date: None,
            description: None,
            tags: HashMap::new(),
        })
    }

    async fn list_keys(&self, _limit: Option<usize>) -> CryptoResult<Vec<String>> {
        let response = vaultrs::api::transit::key::list(&self.client, &self.mount_path)
            .await
            .map_err(|e| CryptoError::VaultError(format!("Failed to list keys: {}", e)))?;

        Ok(response)
    }

    async fn enable_key(&self, _key_id: &str) -> CryptoResult<()> {
        // Vault doesn't have an explicit enable/disable mechanism like AWS KMS
        // Keys are enabled by default when created
        Ok(())
    }

    async fn disable_key(&self, _key_id: &str) -> CryptoResult<()> {
        // Vault doesn't have an explicit enable/disable mechanism
        // You would need to use policy-based access control instead
        Err(CryptoError::UnsupportedOperation(
            "Vault does not support key disabling. Use policies instead.".to_string(),
        ))
    }

    async fn schedule_key_deletion(&self, key_id: &str, _pending_days: u32) -> CryptoResult<()> {
        vaultrs::api::transit::key::delete(&self.client, &self.mount_path, key_id)
            .await
            .map_err(|e| CryptoError::VaultError(format!("Failed to delete key: {}", e)))?;

        Ok(())
    }

    async fn cancel_key_deletion(&self, _key_id: &str) -> CryptoResult<()> {
        Err(CryptoError::UnsupportedOperation(
            "Vault does not support canceling key deletion".to_string(),
        ))
    }

    async fn rotate_key(&self, key_id: &str) -> CryptoResult<()> {
        vaultrs::api::transit::key::rotate(&self.client, &self.mount_path, key_id)
            .await
            .map_err(|e| CryptoError::VaultError(format!("Failed to rotate key: {}", e)))?;

        Ok(())
    }

    async fn create_alias(&self, _alias: &str, _key_id: &str) -> CryptoResult<()> {
        Err(CryptoError::UnsupportedOperation(
            "Vault does not support key aliases".to_string(),
        ))
    }

    async fn delete_alias(&self, _alias: &str) -> CryptoResult<()> {
        Err(CryptoError::UnsupportedOperation(
            "Vault does not support key aliases".to_string(),
        ))
    }

    async fn tag_key(&self, _key_id: &str, _tags: &HashMap<String, String>) -> CryptoResult<()> {
        Err(CryptoError::UnsupportedOperation(
            "Vault does not support key tagging in transit engine".to_string(),
        ))
    }

    async fn untag_key(&self, _key_id: &str, _tag_keys: &[String]) -> CryptoResult<()> {
        Err(CryptoError::UnsupportedOperation(
            "Vault does not support key tagging in transit engine".to_string(),
        ))
    }

    async fn sign(
        &self,
        key_id: &str,
        message: &[u8],
        _signing_algorithm: SigningAlgorithm,
    ) -> CryptoResult<Vec<u8>> {
        let input_b64 = base64::encode(message);

        let request = SignDataRequest::builder()
            .input(input_b64)
            .build()
            .unwrap();

        let response = vaultrs::api::transit::data::sign(
            &self.client,
            &self.mount_path,
            key_id,
            Some(&request),
        )
        .await
        .map_err(|e| CryptoError::SignatureFailed(format!("Signing failed: {}", e)))?;

        Ok(response.signature.into_bytes())
    }

    async fn verify(
        &self,
        key_id: &str,
        message: &[u8],
        signature: &[u8],
        _signing_algorithm: SigningAlgorithm,
    ) -> CryptoResult<bool> {
        let input_b64 = base64::encode(message);
        let signature_str = String::from_utf8(signature.to_vec())
            .map_err(|e| CryptoError::VerificationFailed(format!("Invalid signature format: {}", e)))?;

        let request = VerifySignedDataRequest::builder()
            .input(input_b64)
            .signature(signature_str)
            .build()
            .unwrap();

        let response = vaultrs::api::transit::data::verify(
            &self.client,
            &self.mount_path,
            key_id,
            Some(&request),
        )
        .await
        .map_err(|e| CryptoError::VerificationFailed(format!("Verification failed: {}", e)))?;

        Ok(response.valid)
    }

    async fn get_public_key(&self, key_id: &str) -> CryptoResult<Vec<u8>> {
        let key_data = vaultrs::api::transit::key::read(&self.client, &self.mount_path, key_id)
            .await
            .map_err(|e| CryptoError::VaultError(format!("Failed to get public key: {}", e)))?;

        // Get the latest key version
        let latest_version = key_data.latest_version;
        let key_info = key_data
            .keys
            .get(&latest_version.to_string())
            .ok_or_else(|| CryptoError::VaultError("No key version found".to_string()))?;

        key_info
            .public_key
            .as_ref()
            .map(|pk| pk.as_bytes().to_vec())
            .ok_or_else(|| CryptoError::VaultError("No public key available".to_string()))
    }
}

#[cfg(not(feature = "vault"))]
pub struct VaultKms;

#[cfg(not(feature = "vault"))]
impl VaultKms {
    pub async fn new(
        _vault_addr: &str,
        _token: &str,
        _mount_path: Option<&str>,
    ) -> CryptoResult<Self> {
        Err(CryptoError::UnsupportedOperation(
            "Vault support not enabled. Enable 'vault' feature".to_string(),
        ))
    }
}
