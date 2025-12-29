//! Key derivation functions.
//!
//! This module provides secure key derivation using HKDF, PBKDF2, and Argon2.

use crate::error::{CryptoError, CryptoResult};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use hkdf::Hkdf;
use pbkdf2::pbkdf2_hmac;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Key derivation algorithm.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyDerivationAlgorithm {
    /// HKDF-SHA256.
    HkdfSha256,

    /// HKDF-SHA512.
    HkdfSha512,

    /// PBKDF2-HMAC-SHA256.
    Pbkdf2HmacSha256,

    /// PBKDF2-HMAC-SHA512.
    Pbkdf2HmacSha512,

    /// Argon2id.
    Argon2id,
}

/// Key derivation parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationParams {
    /// Algorithm to use.
    pub algorithm: KeyDerivationAlgorithm,

    /// Salt for key derivation.
    pub salt: Vec<u8>,

    /// Iteration count (for PBKDF2).
    pub iterations: Option<u32>,

    /// Info/context data (for HKDF).
    pub info: Option<Vec<u8>>,

    /// Memory cost (for Argon2).
    pub memory_cost: Option<u32>,

    /// Time cost (for Argon2).
    pub time_cost: Option<u32>,
}

/// Derived key material.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct DerivedKey {
    /// The derived key material.
    key_material: Vec<u8>,

    /// Parameters used for derivation.
    #[zeroize(skip)]
    params: KeyDerivationParams,
}

impl DerivedKey {
    /// Get the key material.
    pub fn key_material(&self) -> &[u8] {
        &self.key_material
    }

    /// Get the derivation parameters.
    pub fn params(&self) -> &KeyDerivationParams {
        &self.params
    }

    /// Convert to bytes (consuming self).
    pub fn into_bytes(self) -> Vec<u8> {
        self.key_material.clone()
    }
}

/// Key derivation service.
pub struct KeyDerivation;

impl KeyDerivation {
    /// Derive a key using HKDF-SHA256.
    pub fn hkdf_sha256(
        input_key_material: &[u8],
        salt: &[u8],
        info: &[u8],
        output_length: usize,
    ) -> CryptoResult<DerivedKey> {
        let hkdf = Hkdf::<sha2::Sha256>::new(Some(salt), input_key_material);

        let mut output = vec![0u8; output_length];
        hkdf.expand(info, &mut output)
            .map_err(|e| CryptoError::KeyDerivationFailed(format!("HKDF expansion failed: {}", e)))?;

        Ok(DerivedKey {
            key_material: output,
            params: KeyDerivationParams {
                algorithm: KeyDerivationAlgorithm::HkdfSha256,
                salt: salt.to_vec(),
                iterations: None,
                info: Some(info.to_vec()),
                memory_cost: None,
                time_cost: None,
            },
        })
    }

    /// Derive a key using HKDF-SHA512.
    pub fn hkdf_sha512(
        input_key_material: &[u8],
        salt: &[u8],
        info: &[u8],
        output_length: usize,
    ) -> CryptoResult<DerivedKey> {
        let hkdf = Hkdf::<sha2::Sha512>::new(Some(salt), input_key_material);

        let mut output = vec![0u8; output_length];
        hkdf.expand(info, &mut output)
            .map_err(|e| CryptoError::KeyDerivationFailed(format!("HKDF expansion failed: {}", e)))?;

        Ok(DerivedKey {
            key_material: output,
            params: KeyDerivationParams {
                algorithm: KeyDerivationAlgorithm::HkdfSha512,
                salt: salt.to_vec(),
                iterations: None,
                info: Some(info.to_vec()),
                memory_cost: None,
                time_cost: None,
            },
        })
    }

    /// Derive a key using PBKDF2-HMAC-SHA256.
    pub fn pbkdf2_sha256(
        password: &[u8],
        salt: &[u8],
        iterations: u32,
        output_length: usize,
    ) -> CryptoResult<DerivedKey> {
        if iterations < 10000 {
            return Err(CryptoError::InvalidConfiguration(
                "PBKDF2 iterations must be at least 10000".to_string(),
            ));
        }

        let mut output = vec![0u8; output_length];
        pbkdf2_hmac::<sha2::Sha256>(password, salt, iterations, &mut output);

        Ok(DerivedKey {
            key_material: output,
            params: KeyDerivationParams {
                algorithm: KeyDerivationAlgorithm::Pbkdf2HmacSha256,
                salt: salt.to_vec(),
                iterations: Some(iterations),
                info: None,
                memory_cost: None,
                time_cost: None,
            },
        })
    }

    /// Derive a key using PBKDF2-HMAC-SHA512.
    pub fn pbkdf2_sha512(
        password: &[u8],
        salt: &[u8],
        iterations: u32,
        output_length: usize,
    ) -> CryptoResult<DerivedKey> {
        if iterations < 10000 {
            return Err(CryptoError::InvalidConfiguration(
                "PBKDF2 iterations must be at least 10000".to_string(),
            ));
        }

        let mut output = vec![0u8; output_length];
        pbkdf2_hmac::<sha2::Sha512>(password, salt, iterations, &mut output);

        Ok(DerivedKey {
            key_material: output,
            params: KeyDerivationParams {
                algorithm: KeyDerivationAlgorithm::Pbkdf2HmacSha512,
                salt: salt.to_vec(),
                iterations: Some(iterations),
                info: None,
                memory_cost: None,
                time_cost: None,
            },
        })
    }

    /// Derive a key using Argon2id.
    pub fn argon2id(
        password: &[u8],
        salt: &[u8],
        memory_cost: u32,
        time_cost: u32,
        output_length: usize,
    ) -> CryptoResult<DerivedKey> {
        use argon2::{Algorithm, Params, Version};

        let params = Params::new(memory_cost, time_cost, 1, Some(output_length))
            .map_err(|e| CryptoError::KeyDerivationFailed(format!("Invalid Argon2 params: {}", e)))?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut output = vec![0u8; output_length];
        argon2
            .hash_password_into(password, salt, &mut output)
            .map_err(|e| CryptoError::KeyDerivationFailed(format!("Argon2 derivation failed: {}", e)))?;

        Ok(DerivedKey {
            key_material: output,
            params: KeyDerivationParams {
                algorithm: KeyDerivationAlgorithm::Argon2id,
                salt: salt.to_vec(),
                iterations: None,
                info: None,
                memory_cost: Some(memory_cost),
                time_cost: Some(time_cost),
            },
        })
    }

    /// Derive a key using the specified algorithm and parameters.
    pub fn derive(
        input: &[u8],
        params: &KeyDerivationParams,
        output_length: usize,
    ) -> CryptoResult<DerivedKey> {
        match params.algorithm {
            KeyDerivationAlgorithm::HkdfSha256 => {
                let info = params.info.as_deref().unwrap_or(&[]);
                Self::hkdf_sha256(input, &params.salt, info, output_length)
            }
            KeyDerivationAlgorithm::HkdfSha512 => {
                let info = params.info.as_deref().unwrap_or(&[]);
                Self::hkdf_sha512(input, &params.salt, info, output_length)
            }
            KeyDerivationAlgorithm::Pbkdf2HmacSha256 => {
                let iterations = params.iterations.unwrap_or(100000);
                Self::pbkdf2_sha256(input, &params.salt, iterations, output_length)
            }
            KeyDerivationAlgorithm::Pbkdf2HmacSha512 => {
                let iterations = params.iterations.unwrap_or(100000);
                Self::pbkdf2_sha512(input, &params.salt, iterations, output_length)
            }
            KeyDerivationAlgorithm::Argon2id => {
                let memory_cost = params.memory_cost.unwrap_or(65536); // 64 MB
                let time_cost = params.time_cost.unwrap_or(3);
                Self::argon2id(input, &params.salt, memory_cost, time_cost, output_length)
            }
        }
    }

    /// Generate a random salt.
    pub fn generate_salt(length: usize) -> Vec<u8> {
        use aes_gcm::aead::OsRng;
        use rand::RngCore;

        let mut salt = vec![0u8; length];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// Derive multiple keys from a single master key.
    pub fn derive_multiple(
        master_key: &[u8],
        salt: &[u8],
        count: usize,
        key_length: usize,
    ) -> CryptoResult<Vec<DerivedKey>> {
        let mut keys = Vec::with_capacity(count);

        for i in 0..count {
            let info = format!("key-{}", i);
            let key = Self::hkdf_sha256(master_key, salt, info.as_bytes(), key_length)?;
            keys.push(key);
        }

        Ok(keys)
    }
}

/// Password hashing service using Argon2.
pub struct PasswordHashingService;

impl PasswordHashingService {
    /// Hash a password for storage.
    pub fn hash_password(password: &[u8]) -> CryptoResult<String> {
        use rand::rngs::OsRng;
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password, &salt)
            .map_err(|e| CryptoError::KeyDerivationFailed(format!("Password hashing failed: {}", e)))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against a hash.
    pub fn verify_password(password: &[u8], hash: &str) -> CryptoResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| CryptoError::KeyDerivationFailed(format!("Invalid password hash: {}", e)))?;

        let argon2 = Argon2::default();

        match argon2.verify_password(password, &parsed_hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Derive a key for encryption from a password.
pub fn derive_encryption_key(password: &str, salt: &[u8]) -> CryptoResult<Vec<u8>> {
    let derived = KeyDerivation::pbkdf2_sha256(
        password.as_bytes(),
        salt,
        100000, // 100k iterations
        32,     // 256-bit key
    )?;

    Ok(derived.into_bytes())
}

/// Derive a key hierarchy from a master key.
pub struct KeyHierarchy {
    master_key: Vec<u8>,
    salt: Vec<u8>,
}

impl KeyHierarchy {
    /// Create a new key hierarchy from a master key.
    pub fn new(master_key: Vec<u8>, salt: Vec<u8>) -> Self {
        Self { master_key, salt }
    }

    /// Derive a sub-key for a specific purpose.
    pub fn derive_subkey(&self, purpose: &str, length: usize) -> CryptoResult<DerivedKey> {
        KeyDerivation::hkdf_sha256(&self.master_key, &self.salt, purpose.as_bytes(), length)
    }

    /// Derive multiple sub-keys.
    pub fn derive_subkeys(&self, purposes: &[&str], length: usize) -> CryptoResult<Vec<DerivedKey>> {
        let mut keys = Vec::with_capacity(purposes.len());

        for purpose in purposes {
            let key = self.derive_subkey(purpose, length)?;
            keys.push(key);
        }

        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkdf_sha256() {
        let ikm = b"input key material";
        let salt = b"salt";
        let info = b"info";

        let result = KeyDerivation::hkdf_sha256(ikm, salt, info, 32);
        assert!(result.is_ok());

        let key = result.unwrap();
        assert_eq!(key.key_material().len(), 32);
    }

    #[test]
    fn test_pbkdf2_sha256() {
        let password = b"password";
        let salt = b"salt12345678";

        let result = KeyDerivation::pbkdf2_sha256(password, salt, 10000, 32);
        assert!(result.is_ok());

        let key = result.unwrap();
        assert_eq!(key.key_material().len(), 32);
    }

    #[test]
    fn test_argon2id() {
        let password = b"password";
        let salt = KeyDerivation::generate_salt(16);

        let result = KeyDerivation::argon2id(password, &salt, 65536, 3, 32);
        assert!(result.is_ok());

        let key = result.unwrap();
        assert_eq!(key.key_material().len(), 32);
    }

    #[test]
    fn test_password_hashing() {
        let password = b"secure_password";

        let hash = PasswordHashingService::hash_password(password).unwrap();
        assert!(!hash.is_empty());

        let verified = PasswordHashingService::verify_password(password, &hash).unwrap();
        assert!(verified);

        let wrong_password = b"wrong_password";
        let wrong_verified = PasswordHashingService::verify_password(wrong_password, &hash).unwrap();
        assert!(!wrong_verified);
    }

    #[test]
    fn test_key_hierarchy() {
        let master_key = KeyDerivation::generate_salt(32);
        let salt = KeyDerivation::generate_salt(16);

        let hierarchy = KeyHierarchy::new(master_key, salt);

        let encryption_key = hierarchy.derive_subkey("encryption", 32).unwrap();
        let signing_key = hierarchy.derive_subkey("signing", 32).unwrap();

        assert_eq!(encryption_key.key_material().len(), 32);
        assert_eq!(signing_key.key_material().len(), 32);
        assert_ne!(encryption_key.key_material(), signing_key.key_material());
    }

    #[test]
    fn test_derive_multiple_keys() {
        let master_key = KeyDerivation::generate_salt(32);
        let salt = KeyDerivation::generate_salt(16);

        let keys = KeyDerivation::derive_multiple(&master_key, &salt, 5, 32).unwrap();
        assert_eq!(keys.len(), 5);

        // Ensure all keys are unique
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i].key_material(), keys[j].key_material());
            }
        }
    }
}
