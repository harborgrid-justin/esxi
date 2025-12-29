//! Plugin signing and verification for security.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::fs;

use crate::error::{PluginError, PluginResult};

/// Plugin signature manager.
pub struct SignatureManager {
    /// Trusted public keys.
    trusted_keys: Vec<VerifyingKey>,

    /// Require signatures for all plugins.
    require_signatures: bool,
}

impl SignatureManager {
    /// Create a new signature manager.
    pub fn new(require_signatures: bool) -> Self {
        Self {
            trusted_keys: Vec::new(),
            require_signatures,
        }
    }

    /// Add a trusted public key.
    pub fn add_trusted_key(&mut self, public_key: VerifyingKey) {
        self.trusted_keys.push(public_key);
    }

    /// Add a trusted key from bytes.
    pub fn add_trusted_key_bytes(&mut self, key_bytes: &[u8; 32]) -> PluginResult<()> {
        let public_key = VerifyingKey::from_bytes(key_bytes)
            .map_err(|e| PluginError::SignatureError(format!("Invalid public key: {}", e)))?;

        self.trusted_keys.push(public_key);
        Ok(())
    }

    /// Verify a plugin signature.
    pub async fn verify_plugin(&self, plugin_path: &Path) -> PluginResult<()> {
        // Read the plugin file
        let plugin_data = fs::read(plugin_path).await?;

        // Look for signature file
        let sig_path = plugin_path.with_extension("sig");

        if !sig_path.exists() {
            if self.require_signatures {
                return Err(PluginError::SignatureError(
                    "Plugin signature file not found".to_string(),
                ));
            } else {
                tracing::warn!("Plugin has no signature, but signatures are not required");
                return Ok(());
            }
        }

        // Read and parse signature file
        let sig_data = fs::read_to_string(&sig_path).await?;
        let plugin_signature: PluginSignature = serde_json::from_str(&sig_data)
            .map_err(|e| PluginError::SignatureError(format!("Invalid signature file: {}", e)))?;

        // Verify the signature
        self.verify_signature(&plugin_data, &plugin_signature)?;

        tracing::info!("Plugin signature verified successfully");
        Ok(())
    }

    /// Verify a signature against plugin data.
    pub fn verify_signature(
        &self,
        data: &[u8],
        plugin_signature: &PluginSignature,
    ) -> PluginResult<()> {
        // Calculate hash of the data
        let hash = Self::calculate_hash(data);

        // Verify hash matches
        if hash != plugin_signature.hash {
            return Err(PluginError::SignatureError(
                "Plugin hash mismatch".to_string(),
            ));
        }

        // Parse signature
        let signature = Signature::from_bytes(&plugin_signature.signature);

        // Parse public key
        let public_key = VerifyingKey::from_bytes(&plugin_signature.public_key)
            .map_err(|e| PluginError::SignatureError(format!("Invalid public key: {}", e)))?;

        // Check if key is trusted
        if !self.trusted_keys.contains(&public_key) {
            return Err(PluginError::SignatureError(
                "Plugin signed with untrusted key".to_string(),
            ));
        }

        // Verify signature
        public_key
            .verify(&hash, &signature)
            .map_err(|e| PluginError::SignatureError(format!("Signature verification failed: {}", e)))?;

        Ok(())
    }

    /// Calculate SHA256 hash of data.
    fn calculate_hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

/// Plugin signer for creating signatures.
pub struct PluginSigner {
    signing_key: SigningKey,
}

impl PluginSigner {
    /// Create a new plugin signer.
    pub fn new(signing_key: SigningKey) -> Self {
        Self { signing_key }
    }

    /// Create a signer from seed bytes.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(seed);
        Self { signing_key }
    }

    /// Generate a new random key pair.
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut rng = rand::rngs::OsRng;
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        Self { signing_key }
    }

    /// Sign a plugin file.
    pub async fn sign_plugin(&self, plugin_path: &Path) -> PluginResult<PluginSignature> {
        // Read the plugin file
        let plugin_data = fs::read(plugin_path).await?;

        // Calculate hash
        let hash = SignatureManager::calculate_hash(&plugin_data);

        // Sign the hash
        let signature = self.signing_key.sign(&hash);

        // Create signature structure
        let plugin_signature = PluginSignature {
            version: 1,
            algorithm: "ed25519".to_string(),
            hash,
            signature: signature.to_bytes(),
            public_key: self.signing_key.verifying_key().to_bytes(),
            metadata: SignatureMetadata {
                signed_at: chrono::Utc::now(),
                signer: "Meridian Plugin Signer".to_string(),
            },
        };

        // Write signature file
        let sig_path = plugin_path.with_extension("sig");
        let sig_json = serde_json::to_string_pretty(&plugin_signature)?;
        fs::write(&sig_path, sig_json).await?;

        tracing::info!("Plugin signed successfully: {:?}", sig_path);

        Ok(plugin_signature)
    }

    /// Get the public key.
    pub fn public_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Export public key as bytes.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    /// Export private key as bytes.
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

/// Plugin signature structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSignature {
    /// Signature format version.
    pub version: u32,

    /// Signature algorithm.
    pub algorithm: String,

    /// SHA256 hash of the plugin.
    #[serde(with = "hex_serde")]
    pub hash: [u8; 32],

    /// Ed25519 signature.
    #[serde(with = "hex_serde")]
    pub signature: [u8; 64],

    /// Public key used for signing.
    #[serde(with = "hex_serde")]
    pub public_key: [u8; 32],

    /// Signature metadata.
    pub metadata: SignatureMetadata,
}

/// Signature metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureMetadata {
    /// When the plugin was signed.
    pub signed_at: chrono::DateTime<chrono::Utc>,

    /// Who signed the plugin.
    pub signer: String,
}

/// Hex serialization helper for byte arrays.
mod hex_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S, const N: usize>(bytes: &[u8; N], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D, const N: usize>(deserializer: D) -> Result<[u8; N], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;

        if bytes.len() != N {
            return Err(serde::de::Error::custom(format!(
                "Expected {} bytes, got {}",
                N,
                bytes.len()
            )));
        }

        let mut array = [0u8; N];
        array.copy_from_slice(&bytes);
        Ok(array)
    }
}

/// Certificate chain for plugin trust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustChain {
    /// Root certificate.
    pub root: Certificate,

    /// Intermediate certificates.
    pub intermediates: Vec<Certificate>,

    /// Leaf (plugin) certificate.
    pub leaf: Certificate,
}

impl TrustChain {
    /// Verify the entire trust chain.
    pub fn verify(&self) -> PluginResult<()> {
        // Verify root is self-signed
        self.root.verify(&self.root.public_key)?;

        // Verify each intermediate is signed by parent
        let mut current_key = &self.root.public_key;

        for intermediate in &self.intermediates {
            intermediate.verify(current_key)?;
            current_key = &intermediate.public_key;
        }

        // Verify leaf is signed by last intermediate (or root if no intermediates)
        self.leaf.verify(current_key)?;

        Ok(())
    }
}

/// Certificate for trust chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// Certificate subject.
    pub subject: String,

    /// Certificate issuer.
    pub issuer: String,

    /// Public key.
    #[serde(with = "hex_serde")]
    pub public_key: [u8; 32],

    /// Signature by issuer.
    #[serde(with = "hex_serde")]
    pub signature: [u8; 64],

    /// Not valid before.
    pub not_before: chrono::DateTime<chrono::Utc>,

    /// Not valid after.
    pub not_after: chrono::DateTime<chrono::Utc>,
}

impl Certificate {
    /// Verify this certificate was signed by the given public key.
    pub fn verify(&self, issuer_public_key: &[u8; 32]) -> PluginResult<()> {
        // Create data to verify (subject + public key)
        let mut data = Vec::new();
        data.extend_from_slice(self.subject.as_bytes());
        data.extend_from_slice(&self.public_key);

        // Parse keys and signature
        let public_key = VerifyingKey::from_bytes(issuer_public_key)
            .map_err(|e| PluginError::SignatureError(format!("Invalid public key: {}", e)))?;

        let signature = Signature::from_bytes(&self.signature);

        // Verify signature
        public_key
            .verify(&data, &signature)
            .map_err(|e| {
                PluginError::SignatureError(format!("Certificate verification failed: {}", e))
            })?;

        // Verify validity period
        let now = chrono::Utc::now();
        if now < self.not_before || now > self.not_after {
            return Err(PluginError::SignatureError(
                "Certificate is not valid".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_sign_and_verify() {
        // Generate a key pair
        let signer = PluginSigner::generate();

        // Create a temporary plugin file
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), b"test plugin data")
            .await
            .unwrap();

        // Sign the plugin
        let signature = signer.sign_plugin(temp_file.path()).await.unwrap();

        // Create a signature manager with the trusted key
        let mut manager = SignatureManager::new(true);
        manager.add_trusted_key(signer.public_key());

        // Verify the signature
        let result = manager.verify_signature(b"test plugin data", &signature);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_untrusted_key() {
        let signer = PluginSigner::generate();
        let other_signer = PluginSigner::generate();

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), b"test plugin data")
            .await
            .unwrap();

        let signature = signer.sign_plugin(temp_file.path()).await.unwrap();

        // Manager trusts a different key
        let mut manager = SignatureManager::new(true);
        manager.add_trusted_key(other_signer.public_key());

        let result = manager.verify_signature(b"test plugin data", &signature);
        assert!(result.is_err());
    }
}
