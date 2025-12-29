//! Digital signatures and verification.
//!
//! This module provides digital signature generation and verification using various algorithms.

use crate::error::{CryptoError, CryptoResult};
use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, Verifier, VerifyingKey};
use ring::signature::{self, KeyPair, UnparsedPublicKey};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Signature algorithm enumeration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    /// Ed25519 signature scheme.
    Ed25519,

    /// ECDSA with P-256 curve and SHA-256.
    EcdsaP256Sha256,

    /// ECDSA with P-384 curve and SHA-384.
    EcdsaP384Sha384,

    /// RSA-PSS with 2048-bit key and SHA-256.
    RsaPss2048Sha256,

    /// RSA-PSS with 4096-bit key and SHA-256.
    RsaPss4096Sha256,
}

impl SignatureAlgorithm {
    /// Get the algorithm name as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureAlgorithm::Ed25519 => "Ed25519",
            SignatureAlgorithm::EcdsaP256Sha256 => "ECDSA-P256-SHA256",
            SignatureAlgorithm::EcdsaP384Sha384 => "ECDSA-P384-SHA384",
            SignatureAlgorithm::RsaPss2048Sha256 => "RSA-PSS-2048-SHA256",
            SignatureAlgorithm::RsaPss4096Sha256 => "RSA-PSS-4096-SHA256",
        }
    }
}

/// Digital signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalSignature {
    /// The signature bytes.
    pub signature: Vec<u8>,

    /// Algorithm used to create the signature.
    pub algorithm: SignatureAlgorithm,

    /// Public key identifier.
    pub key_id: String,

    /// Timestamp when signature was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Optional metadata.
    pub metadata: Option<serde_json::Value>,
}

/// Signing key pair.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SigningKeyPair {
    /// Private key material.
    private_key: Vec<u8>,

    /// Public key material.
    #[zeroize(skip)]
    public_key: Vec<u8>,

    /// Algorithm used by this key pair.
    #[zeroize(skip)]
    algorithm: SignatureAlgorithm,

    /// Key identifier.
    #[zeroize(skip)]
    key_id: String,
}

impl SigningKeyPair {
    /// Generate a new key pair.
    pub fn generate(algorithm: SignatureAlgorithm) -> CryptoResult<Self> {
        match algorithm {
            SignatureAlgorithm::Ed25519 => Self::generate_ed25519(),
            SignatureAlgorithm::EcdsaP256Sha256 => Self::generate_ecdsa_p256(),
            SignatureAlgorithm::EcdsaP384Sha384 => Self::generate_ecdsa_p384(),
            SignatureAlgorithm::RsaPss2048Sha256 => {
                Err(CryptoError::UnsupportedOperation(
                    "RSA key generation not yet implemented".to_string(),
                ))
            }
            SignatureAlgorithm::RsaPss4096Sha256 => {
                Err(CryptoError::UnsupportedOperation(
                    "RSA key generation not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Generate Ed25519 key pair.
    fn generate_ed25519() -> CryptoResult<Self> {
        use rand::RngCore;
        use rand::rngs::OsRng;

        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            private_key: signing_key.to_bytes().to_vec(),
            public_key: verifying_key.to_bytes().to_vec(),
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Generate ECDSA P-256 key pair.
    fn generate_ecdsa_p256() -> CryptoResult<Self> {
        use ring::rand::SystemRandom;

        let rng = SystemRandom::new();
        let pkcs8_bytes = signature::EcdsaKeyPair::generate_pkcs8(
            &signature::ECDSA_P256_SHA256_ASN1_SIGNING,
            &rng,
        )
        .map_err(|e| CryptoError::KeyGenerationFailed(format!("ECDSA P-256 generation failed: {:?}", e)))?;

        let key_pair = signature::EcdsaKeyPair::from_pkcs8(
            &signature::ECDSA_P256_SHA256_ASN1_SIGNING,
            pkcs8_bytes.as_ref(),
            &rng,
        )
        .map_err(|e| CryptoError::KeyGenerationFailed(format!("ECDSA P-256 key pair creation failed: {:?}", e)))?;

        Ok(Self {
            private_key: pkcs8_bytes.as_ref().to_vec(),
            public_key: key_pair.public_key().as_ref().to_vec(),
            algorithm: SignatureAlgorithm::EcdsaP256Sha256,
            key_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Generate ECDSA P-384 key pair.
    fn generate_ecdsa_p384() -> CryptoResult<Self> {
        use ring::rand::SystemRandom;

        let rng = SystemRandom::new();
        let pkcs8_bytes = signature::EcdsaKeyPair::generate_pkcs8(
            &signature::ECDSA_P384_SHA384_ASN1_SIGNING,
            &rng,
        )
        .map_err(|e| CryptoError::KeyGenerationFailed(format!("ECDSA P-384 generation failed: {:?}", e)))?;

        let key_pair = signature::EcdsaKeyPair::from_pkcs8(
            &signature::ECDSA_P384_SHA384_ASN1_SIGNING,
            pkcs8_bytes.as_ref(),
            &rng,
        )
        .map_err(|e| CryptoError::KeyGenerationFailed(format!("ECDSA P-384 key pair creation failed: {:?}", e)))?;

        Ok(Self {
            private_key: pkcs8_bytes.as_ref().to_vec(),
            public_key: key_pair.public_key().as_ref().to_vec(),
            algorithm: SignatureAlgorithm::EcdsaP384Sha384,
            key_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Create a key pair from existing key material.
    pub fn from_bytes(
        private_key: Vec<u8>,
        public_key: Vec<u8>,
        algorithm: SignatureAlgorithm,
        key_id: String,
    ) -> Self {
        Self {
            private_key,
            public_key,
            algorithm,
            key_id,
        }
    }

    /// Get the public key.
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    /// Get the key ID.
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    /// Get the algorithm.
    pub fn algorithm(&self) -> SignatureAlgorithm {
        self.algorithm
    }

    /// Sign a message.
    pub fn sign(&self, message: &[u8]) -> CryptoResult<DigitalSignature> {
        let signature = match self.algorithm {
            SignatureAlgorithm::Ed25519 => self.sign_ed25519(message)?,
            SignatureAlgorithm::EcdsaP256Sha256 => self.sign_ecdsa_p256(message)?,
            SignatureAlgorithm::EcdsaP384Sha384 => self.sign_ecdsa_p384(message)?,
            _ => {
                return Err(CryptoError::UnsupportedOperation(format!(
                    "Signing with {} not yet implemented",
                    self.algorithm.as_str()
                )))
            }
        };

        Ok(DigitalSignature {
            signature,
            algorithm: self.algorithm,
            key_id: self.key_id.clone(),
            created_at: chrono::Utc::now(),
            metadata: None,
        })
    }

    /// Sign with Ed25519.
    fn sign_ed25519(&self, message: &[u8]) -> CryptoResult<Vec<u8>> {
        if self.private_key.len() != 32 {
            return Err(CryptoError::InvalidKey(
                "Invalid Ed25519 private key length".to_string(),
            ));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&self.private_key);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let signature = signing_key.sign(message);

        Ok(signature.to_bytes().to_vec())
    }

    /// Sign with ECDSA P-256.
    fn sign_ecdsa_p256(&self, message: &[u8]) -> CryptoResult<Vec<u8>> {
        use ring::rand::SystemRandom;

        let rng = SystemRandom::new();
        let key_pair = signature::EcdsaKeyPair::from_pkcs8(
            &signature::ECDSA_P256_SHA256_ASN1_SIGNING,
            &self.private_key,
            &rng,
        )
        .map_err(|e| CryptoError::SignatureFailed(format!("Failed to load ECDSA key: {:?}", e)))?;

        let signature = key_pair
            .sign(&rng, message)
            .map_err(|e| CryptoError::SignatureFailed(format!("ECDSA signing failed: {:?}", e)))?;

        Ok(signature.as_ref().to_vec())
    }

    /// Sign with ECDSA P-384.
    fn sign_ecdsa_p384(&self, message: &[u8]) -> CryptoResult<Vec<u8>> {
        use ring::rand::SystemRandom;

        let rng = SystemRandom::new();
        let key_pair = signature::EcdsaKeyPair::from_pkcs8(
            &signature::ECDSA_P384_SHA384_ASN1_SIGNING,
            &self.private_key,
            &rng,
        )
        .map_err(|e| CryptoError::SignatureFailed(format!("Failed to load ECDSA key: {:?}", e)))?;

        let signature = key_pair
            .sign(&rng, message)
            .map_err(|e| CryptoError::SignatureFailed(format!("ECDSA signing failed: {:?}", e)))?;

        Ok(signature.as_ref().to_vec())
    }
}

/// Signature verifier.
pub struct SignatureVerifier;

impl SignatureVerifier {
    /// Verify a signature.
    pub fn verify(
        signature: &DigitalSignature,
        message: &[u8],
        public_key: &[u8],
    ) -> CryptoResult<bool> {
        match signature.algorithm {
            SignatureAlgorithm::Ed25519 => Self::verify_ed25519(&signature.signature, message, public_key),
            SignatureAlgorithm::EcdsaP256Sha256 => {
                Self::verify_ecdsa_p256(&signature.signature, message, public_key)
            }
            SignatureAlgorithm::EcdsaP384Sha384 => {
                Self::verify_ecdsa_p384(&signature.signature, message, public_key)
            }
            _ => Err(CryptoError::UnsupportedOperation(format!(
                "Verification with {} not yet implemented",
                signature.algorithm.as_str()
            ))),
        }
    }

    /// Verify Ed25519 signature.
    fn verify_ed25519(signature: &[u8], message: &[u8], public_key: &[u8]) -> CryptoResult<bool> {
        if public_key.len() != 32 {
            return Err(CryptoError::InvalidKey(
                "Invalid Ed25519 public key length".to_string(),
            ));
        }

        if signature.len() != 64 {
            return Err(CryptoError::VerificationFailed(
                "Invalid Ed25519 signature length".to_string(),
            ));
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(public_key);

        let verifying_key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| CryptoError::InvalidKey(format!("Invalid Ed25519 public key: {}", e)))?;

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(signature);

        let sig = Ed25519Signature::from_bytes(&sig_bytes);

        match verifying_key.verify(message, &sig) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Verify ECDSA P-256 signature.
    fn verify_ecdsa_p256(signature: &[u8], message: &[u8], public_key: &[u8]) -> CryptoResult<bool> {
        let public_key = UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_ASN1, public_key);

        match public_key.verify(message, signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Verify ECDSA P-384 signature.
    fn verify_ecdsa_p384(signature: &[u8], message: &[u8], public_key: &[u8]) -> CryptoResult<bool> {
        let public_key = UnparsedPublicKey::new(&signature::ECDSA_P384_SHA384_ASN1, public_key);

        match public_key.verify(message, signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Signature service for managing signing operations.
pub struct SignatureService {
    key_pairs: std::collections::HashMap<String, SigningKeyPair>,
}

impl SignatureService {
    /// Create a new signature service.
    pub fn new() -> Self {
        Self {
            key_pairs: std::collections::HashMap::new(),
        }
    }

    /// Generate and store a new key pair.
    pub fn generate_key_pair(&mut self, algorithm: SignatureAlgorithm) -> CryptoResult<String> {
        let key_pair = SigningKeyPair::generate(algorithm)?;
        let key_id = key_pair.key_id().to_string();
        self.key_pairs.insert(key_id.clone(), key_pair);
        Ok(key_id)
    }

    /// Add an existing key pair.
    pub fn add_key_pair(&mut self, key_pair: SigningKeyPair) -> String {
        let key_id = key_pair.key_id().to_string();
        self.key_pairs.insert(key_id.clone(), key_pair);
        key_id
    }

    /// Get a key pair by ID.
    pub fn get_key_pair(&self, key_id: &str) -> Option<&SigningKeyPair> {
        self.key_pairs.get(key_id)
    }

    /// Sign a message with a specific key.
    pub fn sign(&self, key_id: &str, message: &[u8]) -> CryptoResult<DigitalSignature> {
        let key_pair = self
            .key_pairs
            .get(key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(format!("Key not found: {}", key_id)))?;

        key_pair.sign(message)
    }

    /// Verify a signature.
    pub fn verify(&self, signature: &DigitalSignature, message: &[u8]) -> CryptoResult<bool> {
        let key_pair = self.key_pairs.get(&signature.key_id).ok_or_else(|| {
            CryptoError::KeyNotFound(format!("Key not found: {}", signature.key_id))
        })?;

        SignatureVerifier::verify(signature, message, key_pair.public_key())
    }

    /// Remove a key pair.
    pub fn remove_key_pair(&mut self, key_id: &str) -> Option<SigningKeyPair> {
        self.key_pairs.remove(key_id)
    }

    /// List all key IDs.
    pub fn list_keys(&self) -> Vec<String> {
        self.key_pairs.keys().cloned().collect()
    }
}

impl Default for SignatureService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_signing() {
        let key_pair = SigningKeyPair::generate(SignatureAlgorithm::Ed25519).unwrap();
        let message = b"Test message to sign";

        let signature = key_pair.sign(message).unwrap();
        assert_eq!(signature.algorithm, SignatureAlgorithm::Ed25519);

        let verified = SignatureVerifier::verify(&signature, message, key_pair.public_key()).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_ecdsa_p256_signing() {
        let key_pair = SigningKeyPair::generate(SignatureAlgorithm::EcdsaP256Sha256).unwrap();
        let message = b"Test message to sign";

        let signature = key_pair.sign(message).unwrap();
        assert_eq!(signature.algorithm, SignatureAlgorithm::EcdsaP256Sha256);

        let verified = SignatureVerifier::verify(&signature, message, key_pair.public_key()).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_signature_service() {
        let mut service = SignatureService::new();
        let key_id = service.generate_key_pair(SignatureAlgorithm::Ed25519).unwrap();

        let message = b"Test message";
        let signature = service.sign(&key_id, message).unwrap();

        let verified = service.verify(&signature, message).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_invalid_signature() {
        let key_pair = SigningKeyPair::generate(SignatureAlgorithm::Ed25519).unwrap();
        let message = b"Original message";
        let tampered_message = b"Tampered message";

        let signature = key_pair.sign(message).unwrap();

        let verified = SignatureVerifier::verify(&signature, tampered_message, key_pair.public_key()).unwrap();
        assert!(!verified);
    }
}
