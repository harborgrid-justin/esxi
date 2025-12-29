//! AWS Key Management Service (KMS) implementation.

#[cfg(feature = "aws-kms")]
use crate::error::{CryptoError, CryptoResult};
use crate::kms::{
    DataKeyPair, EncryptionContext, KeyManagementService, KeyMetadata, KeySpec, KeyState,
    KeyUsage, SigningAlgorithm,
};
use async_trait::async_trait;
use std::collections::HashMap;

#[cfg(feature = "aws-kms")]
use aws_sdk_kms::{
    types::{DataKeySpec, KeySpec as AwsKeySpec, MessageType, SigningAlgorithmSpec},
    Client as KmsClient,
};

/// AWS KMS client wrapper.
#[cfg(feature = "aws-kms")]
pub struct AwsKms {
    client: KmsClient,
    region: String,
}

#[cfg(feature = "aws-kms")]
impl AwsKms {
    /// Create a new AWS KMS client.
    pub async fn new(region: Option<String>) -> CryptoResult<Self> {
        let config = aws_config::load_from_env().await;
        let client = KmsClient::new(&config);
        let region = region.unwrap_or_else(|| config.region().unwrap().to_string());

        Ok(Self { client, region })
    }

    /// Create a new AWS KMS client with custom configuration.
    pub fn with_client(client: KmsClient, region: String) -> Self {
        Self { client, region }
    }

    /// Convert KeySpec to AWS DataKeySpec.
    fn to_aws_data_key_spec(spec: KeySpec) -> DataKeySpec {
        match spec {
            KeySpec::Aes256 => DataKeySpec::Aes256,
            KeySpec::Aes128 => DataKeySpec::Aes128,
            _ => DataKeySpec::Aes256, // Default to AES-256
        }
    }

    /// Convert KeyUsage to AWS KeyUsage.
    fn to_aws_key_usage(usage: KeyUsage) -> aws_sdk_kms::types::KeyUsageType {
        match usage {
            KeyUsage::EncryptDecrypt => aws_sdk_kms::types::KeyUsageType::EncryptDecrypt,
            KeyUsage::SignVerify => aws_sdk_kms::types::KeyUsageType::SignVerify,
            KeyUsage::WrapUnwrap => aws_sdk_kms::types::KeyUsageType::EncryptDecrypt,
        }
    }

    /// Convert SigningAlgorithm to AWS SigningAlgorithmSpec.
    fn to_aws_signing_algorithm(algorithm: SigningAlgorithm) -> SigningAlgorithmSpec {
        match algorithm {
            SigningAlgorithm::RsassaPssSha256 => SigningAlgorithmSpec::RsassaPssSha256,
            SigningAlgorithm::RsassaPssSha384 => SigningAlgorithmSpec::RsassaPssSha384,
            SigningAlgorithm::RsassaPssSha512 => SigningAlgorithmSpec::RsassaPssSha512,
            SigningAlgorithm::RsassaPkcs1V15Sha256 => SigningAlgorithmSpec::RsassaPkcs1V15Sha256,
            SigningAlgorithm::RsassaPkcs1V15Sha384 => SigningAlgorithmSpec::RsassaPkcs1V15Sha384,
            SigningAlgorithm::RsassaPkcs1V15Sha512 => SigningAlgorithmSpec::RsassaPkcs1V15Sha512,
            SigningAlgorithm::EcdsaSha256 => SigningAlgorithmSpec::EcdsaSha256,
            SigningAlgorithm::EcdsaSha384 => SigningAlgorithmSpec::EcdsaSha384,
            SigningAlgorithm::EcdsaSha512 => SigningAlgorithmSpec::EcdsaSha512,
        }
    }

    /// Build encryption context for AWS KMS.
    fn build_encryption_context(
        context: Option<&EncryptionContext>,
    ) -> Option<HashMap<String, String>> {
        context.cloned()
    }
}

#[cfg(feature = "aws-kms")]
#[async_trait]
impl KeyManagementService for AwsKms {
    async fn generate_data_key(
        &self,
        key_id: &str,
        key_spec: KeySpec,
        encryption_context: Option<&EncryptionContext>,
    ) -> CryptoResult<DataKeyPair> {
        let mut request = self
            .client
            .generate_data_key()
            .key_id(key_id)
            .key_spec(Self::to_aws_data_key_spec(key_spec));

        if let Some(ctx) = Self::build_encryption_context(encryption_context) {
            request = request.set_encryption_context(Some(ctx));
        }

        let response = request.send().await.map_err(|e| {
            CryptoError::AwsKmsError(format!("Failed to generate data key: {}", e))
        })?;

        Ok(DataKeyPair {
            plaintext: response.plaintext().unwrap().as_ref().to_vec(),
            ciphertext: response.ciphertext_blob().unwrap().as_ref().to_vec(),
            key_id: response.key_id().unwrap_or(key_id).to_string(),
        })
    }

    async fn encrypt(
        &self,
        key_id: &str,
        plaintext: &[u8],
        encryption_context: Option<&EncryptionContext>,
    ) -> CryptoResult<Vec<u8>> {
        let mut request = self
            .client
            .encrypt()
            .key_id(key_id)
            .plaintext(aws_sdk_kms::primitives::Blob::new(plaintext));

        if let Some(ctx) = Self::build_encryption_context(encryption_context) {
            request = request.set_encryption_context(Some(ctx));
        }

        let response = request
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Encryption failed: {}", e)))?;

        Ok(response.ciphertext_blob().unwrap().as_ref().to_vec())
    }

    async fn decrypt(
        &self,
        _key_id: &str,
        ciphertext: &[u8],
        encryption_context: Option<&EncryptionContext>,
    ) -> CryptoResult<Vec<u8>> {
        let mut request = self
            .client
            .decrypt()
            .ciphertext_blob(aws_sdk_kms::primitives::Blob::new(ciphertext));

        if let Some(ctx) = Self::build_encryption_context(encryption_context) {
            request = request.set_encryption_context(Some(ctx));
        }

        let response = request
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Decryption failed: {}", e)))?;

        Ok(response.plaintext().unwrap().as_ref().to_vec())
    }

    async fn create_key(
        &self,
        description: Option<&str>,
        key_usage: KeyUsage,
        tags: Option<&HashMap<String, String>>,
    ) -> CryptoResult<KeyMetadata> {
        let mut request = self
            .client
            .create_key()
            .key_usage(Self::to_aws_key_usage(key_usage));

        if let Some(desc) = description {
            request = request.description(desc);
        }

        if let Some(tag_map) = tags {
            let aws_tags: Vec<_> = tag_map
                .iter()
                .map(|(k, v)| {
                    aws_sdk_kms::types::Tag::builder()
                        .tag_key(k)
                        .tag_value(v)
                        .build()
                        .unwrap()
                })
                .collect();
            request = request.set_tags(Some(aws_tags));
        }

        let response = request
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Failed to create key: {}", e)))?;

        let metadata = response.key_metadata().unwrap();

        Ok(KeyMetadata {
            key_id: metadata.key_id().to_string(),
            alias: None,
            state: KeyState::Enabled,
            algorithm: format!("{:?}", metadata.key_spec().unwrap()),
            usage: key_usage,
            created_at: chrono::Utc::now(),
            last_rotated_at: None,
            deletion_date: None,
            description: description.map(String::from),
            tags: tags.cloned().unwrap_or_default(),
        })
    }

    async fn describe_key(&self, key_id: &str) -> CryptoResult<KeyMetadata> {
        let response = self
            .client
            .describe_key()
            .key_id(key_id)
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Failed to describe key: {}", e)))?;

        let metadata = response.key_metadata().unwrap();

        let state = match metadata.key_state() {
            Some(aws_sdk_kms::types::KeyState::Enabled) => KeyState::Enabled,
            Some(aws_sdk_kms::types::KeyState::Disabled) => KeyState::Disabled,
            Some(aws_sdk_kms::types::KeyState::PendingDeletion) => KeyState::PendingDeletion,
            Some(aws_sdk_kms::types::KeyState::PendingImport) => KeyState::PendingImport,
            _ => KeyState::Unavailable,
        };

        Ok(KeyMetadata {
            key_id: metadata.key_id().to_string(),
            alias: None,
            state,
            algorithm: format!("{:?}", metadata.key_spec().unwrap()),
            usage: KeyUsage::EncryptDecrypt,
            created_at: chrono::Utc::now(),
            last_rotated_at: None,
            deletion_date: None,
            description: metadata.description().map(String::from),
            tags: HashMap::new(),
        })
    }

    async fn list_keys(&self, limit: Option<usize>) -> CryptoResult<Vec<String>> {
        let mut request = self.client.list_keys();

        if let Some(lim) = limit {
            request = request.limit(lim as i32);
        }

        let response = request
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Failed to list keys: {}", e)))?;

        Ok(response
            .keys()
            .iter()
            .filter_map(|k| k.key_id().map(String::from))
            .collect())
    }

    async fn enable_key(&self, key_id: &str) -> CryptoResult<()> {
        self.client
            .enable_key()
            .key_id(key_id)
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Failed to enable key: {}", e)))?;

        Ok(())
    }

    async fn disable_key(&self, key_id: &str) -> CryptoResult<()> {
        self.client
            .disable_key()
            .key_id(key_id)
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Failed to disable key: {}", e)))?;

        Ok(())
    }

    async fn schedule_key_deletion(&self, key_id: &str, pending_days: u32) -> CryptoResult<()> {
        self.client
            .schedule_key_deletion()
            .key_id(key_id)
            .pending_window_in_days(pending_days as i32)
            .send()
            .await
            .map_err(|e| {
                CryptoError::AwsKmsError(format!("Failed to schedule key deletion: {}", e))
            })?;

        Ok(())
    }

    async fn cancel_key_deletion(&self, key_id: &str) -> CryptoResult<()> {
        self.client
            .cancel_key_deletion()
            .key_id(key_id)
            .send()
            .await
            .map_err(|e| {
                CryptoError::AwsKmsError(format!("Failed to cancel key deletion: {}", e))
            })?;

        Ok(())
    }

    async fn rotate_key(&self, key_id: &str) -> CryptoResult<()> {
        self.client
            .enable_key_rotation()
            .key_id(key_id)
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Failed to rotate key: {}", e)))?;

        Ok(())
    }

    async fn create_alias(&self, alias: &str, key_id: &str) -> CryptoResult<()> {
        let alias_name = if alias.starts_with("alias/") {
            alias.to_string()
        } else {
            format!("alias/{}", alias)
        };

        self.client
            .create_alias()
            .alias_name(alias_name)
            .target_key_id(key_id)
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Failed to create alias: {}", e)))?;

        Ok(())
    }

    async fn delete_alias(&self, alias: &str) -> CryptoResult<()> {
        let alias_name = if alias.starts_with("alias/") {
            alias.to_string()
        } else {
            format!("alias/{}", alias)
        };

        self.client
            .delete_alias()
            .alias_name(alias_name)
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Failed to delete alias: {}", e)))?;

        Ok(())
    }

    async fn tag_key(&self, key_id: &str, tags: &HashMap<String, String>) -> CryptoResult<()> {
        let aws_tags: Vec<_> = tags
            .iter()
            .map(|(k, v)| {
                aws_sdk_kms::types::Tag::builder()
                    .tag_key(k)
                    .tag_value(v)
                    .build()
                    .unwrap()
            })
            .collect();

        self.client
            .tag_resource()
            .key_id(key_id)
            .set_tags(Some(aws_tags))
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Failed to tag key: {}", e)))?;

        Ok(())
    }

    async fn untag_key(&self, key_id: &str, tag_keys: &[String]) -> CryptoResult<()> {
        self.client
            .untag_resource()
            .key_id(key_id)
            .set_tag_keys(Some(tag_keys.to_vec()))
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Failed to untag key: {}", e)))?;

        Ok(())
    }

    async fn sign(
        &self,
        key_id: &str,
        message: &[u8],
        signing_algorithm: SigningAlgorithm,
    ) -> CryptoResult<Vec<u8>> {
        let response = self
            .client
            .sign()
            .key_id(key_id)
            .message(aws_sdk_kms::primitives::Blob::new(message))
            .message_type(MessageType::Raw)
            .signing_algorithm(Self::to_aws_signing_algorithm(signing_algorithm))
            .send()
            .await
            .map_err(|e| CryptoError::SignatureFailed(format!("Signing failed: {}", e)))?;

        Ok(response.signature().unwrap().as_ref().to_vec())
    }

    async fn verify(
        &self,
        key_id: &str,
        message: &[u8],
        signature: &[u8],
        signing_algorithm: SigningAlgorithm,
    ) -> CryptoResult<bool> {
        let response = self
            .client
            .verify()
            .key_id(key_id)
            .message(aws_sdk_kms::primitives::Blob::new(message))
            .signature(aws_sdk_kms::primitives::Blob::new(signature))
            .message_type(MessageType::Raw)
            .signing_algorithm(Self::to_aws_signing_algorithm(signing_algorithm))
            .send()
            .await
            .map_err(|e| CryptoError::VerificationFailed(format!("Verification failed: {}", e)))?;

        Ok(response.signature_valid())
    }

    async fn get_public_key(&self, key_id: &str) -> CryptoResult<Vec<u8>> {
        let response = self
            .client
            .get_public_key()
            .key_id(key_id)
            .send()
            .await
            .map_err(|e| CryptoError::AwsKmsError(format!("Failed to get public key: {}", e)))?;

        Ok(response.public_key().unwrap().as_ref().to_vec())
    }
}

#[cfg(not(feature = "aws-kms"))]
pub struct AwsKms;

#[cfg(not(feature = "aws-kms"))]
impl AwsKms {
    pub async fn new(_region: Option<String>) -> CryptoResult<Self> {
        Err(CryptoError::UnsupportedOperation(
            "AWS KMS support not enabled. Enable 'aws-kms' feature".to_string(),
        ))
    }
}
