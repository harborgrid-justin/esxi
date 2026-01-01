//! Key Derivation Functions (KDF)
//!
//! Implements secure key derivation using industry standards:
//! - HKDF (HMAC-based Key Derivation Function) - RFC 5869
//! - PBKDF2 (Password-Based Key Derivation Function 2)
//! - Argon2 for password-based derivation
//!
//! ## Use Cases
//! - Derive multiple keys from a single master key
//! - Convert passwords to encryption keys
//! - Key stretching for additional security
//! - Domain separation for different purposes
//!
//! ## OWASP Compliance
//! - Sufficient iteration counts
//! - Cryptographically random salts
//! - Approved KDF algorithms (NIST SP 800-132)

use hmac::{Hmac, Mac};
use ring::pbkdf2;
use sha2::Sha256;
use zeroize::Zeroizing;

use crate::error::{SecurityError, SecurityResult};

/// Key derivation function manager
pub struct KeyDerivation;

impl KeyDerivation {
    /// Derive a key using HKDF-SHA256
    ///
    /// HKDF is ideal for deriving multiple keys from a single master key
    /// with different context information for domain separation.
    ///
    /// # Arguments
    /// * `master_key` - Input key material (IKM)
    /// * `salt` - Optional salt (use empty slice if none)
    /// * `info` - Context and application-specific information
    /// * `output_length` - Desired output key length in bytes
    ///
    /// # Example
    /// ```rust,no_run
    /// use meridian_security::kms::derivation::KeyDerivation;
    ///
    /// let master_key = vec![0u8; 32];
    /// let salt = b"unique-salt";
    /// let encryption_key = KeyDerivation::hkdf(&master_key, salt, b"encryption", 32).unwrap();
    /// let signing_key = KeyDerivation::hkdf(&master_key, salt, b"signing", 32).unwrap();
    /// ```
    pub fn hkdf(
        master_key: &[u8],
        salt: &[u8],
        info: &[u8],
        output_length: usize,
    ) -> SecurityResult<Vec<u8>> {
        use ring::hkdf::{Salt, HKDF_SHA256};

        let salt_obj = if salt.is_empty() {
            Salt::new(HKDF_SHA256, &[0u8; 32])
        } else {
            Salt::new(HKDF_SHA256, salt)
        };

        let prk = salt_obj.extract(master_key);

        let okm = prk
            .expand(&[info], MyLength(output_length))
            .map_err(|_| SecurityError::KeyDerivationError("HKDF expansion failed".to_string()))?;

        let mut output = Zeroizing::new(vec![0u8; output_length]);
        okm.fill(&mut output)
            .map_err(|_| SecurityError::KeyDerivationError("HKDF fill failed".to_string()))?;

        Ok(output.to_vec())
    }

    /// Derive encryption and authentication keys from a single master key
    ///
    /// Returns (encryption_key, authentication_key)
    pub fn derive_encryption_keys(
        master_key: &[u8],
        salt: &[u8],
    ) -> SecurityResult<(Vec<u8>, Vec<u8>)> {
        let enc_key = Self::hkdf(master_key, salt, b"encryption", 32)?;
        let auth_key = Self::hkdf(master_key, salt, b"authentication", 32)?;

        Ok((enc_key, auth_key))
    }

    /// Derive a key using PBKDF2-HMAC-SHA256
    ///
    /// PBKDF2 is suitable for password-based key derivation with
    /// configurable iteration count for key stretching.
    ///
    /// # Arguments
    /// * `password` - Password or passphrase
    /// * `salt` - Cryptographically random salt (minimum 16 bytes)
    /// * `iterations` - Number of iterations (minimum 100,000 for 2023)
    /// * `output_length` - Desired key length in bytes
    ///
    /// # Security Note
    /// OWASP recommends minimum 310,000 iterations for PBKDF2-HMAC-SHA256 (2023).
    /// Consider using Argon2id for new applications (more resistant to GPU attacks).
    pub fn pbkdf2(
        password: &[u8],
        salt: &[u8],
        iterations: u32,
        output_length: usize,
    ) -> SecurityResult<Vec<u8>> {
        if salt.len() < 16 {
            return Err(SecurityError::KeyDerivationError(
                "Salt must be at least 16 bytes".to_string(),
            ));
        }

        if iterations < 100_000 {
            return Err(SecurityError::KeyDerivationError(format!(
                "Insufficient iterations: {} (minimum 100,000)",
                iterations
            )));
        }

        let mut output = Zeroizing::new(vec![0u8; output_length]);

        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(iterations).unwrap(),
            salt,
            password,
            &mut output,
        );

        Ok(output.to_vec())
    }

    /// Derive a key using Argon2id (recommended for passwords)
    ///
    /// Argon2id provides the best resistance against GPU and ASIC attacks.
    ///
    /// # Arguments
    /// * `password` - Password to derive key from
    /// * `salt` - Cryptographic salt (16 bytes minimum)
    /// * `output_length` - Desired key length in bytes
    ///
    /// # Security Parameters
    /// Uses OWASP recommended parameters:
    /// - Memory: 47 MiB (47104 KiB)
    /// - Iterations: 1
    /// - Parallelism: 1
    pub fn argon2id(password: &[u8], salt: &[u8], output_length: usize) -> SecurityResult<Vec<u8>> {
        use argon2::{
            password_hash::SaltString, Algorithm, Argon2, ParamsBuilder, PasswordHasher, Version,
        };

        if salt.len() < 16 {
            return Err(SecurityError::KeyDerivationError(
                "Salt must be at least 16 bytes".to_string(),
            ));
        }

        // OWASP recommended parameters for Argon2id
        let mut params_builder = ParamsBuilder::new();
        params_builder
            .m_cost(47104) // 47 MiB
            .t_cost(1) // 1 iteration
            .p_cost(1) // 1 thread
            .output_len(output_length);

        let params = params_builder
            .build()
            .map_err(|e| SecurityError::KeyDerivationError(format!("Argon2 params error: {}", e)))?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        // Convert salt to base64 string format required by argon2
        let salt_b64 = base64::encode(salt);
        let salt_string = SaltString::encode_b64(&salt[..16.min(salt.len())])
            .map_err(|e| SecurityError::KeyDerivationError(format!("Salt encoding error: {}", e)))?;

        let hash = argon2
            .hash_password(password, &salt_string)
            .map_err(|e| SecurityError::KeyDerivationError(format!("Argon2 hash error: {}", e)))?;

        // Extract the hash bytes
        let hash_bytes = hash
            .hash
            .ok_or_else(|| SecurityError::KeyDerivationError("No hash output".to_string()))?;

        Ok(hash_bytes.as_bytes().to_vec())
    }

    /// Derive multiple keys from a single master key using HKDF
    ///
    /// # Arguments
    /// * `master_key` - Master key material
    /// * `salt` - Salt value
    /// * `contexts` - List of context strings for each derived key
    /// * `key_length` - Length of each derived key
    ///
    /// # Returns
    /// Vector of derived keys (one per context)
    pub fn derive_multiple_keys(
        master_key: &[u8],
        salt: &[u8],
        contexts: &[&[u8]],
        key_length: usize,
    ) -> SecurityResult<Vec<Vec<u8>>> {
        contexts
            .iter()
            .map(|ctx| Self::hkdf(master_key, salt, ctx, key_length))
            .collect()
    }

    /// Derive a deterministic key from a master key and domain
    ///
    /// Useful for generating consistent keys for specific purposes
    pub fn derive_domain_key(
        master_key: &[u8],
        domain: &str,
        key_length: usize,
    ) -> SecurityResult<Vec<u8>> {
        Self::hkdf(master_key, &[], domain.as_bytes(), key_length)
    }
}

// Helper struct for HKDF output length
struct MyLength(usize);

impl ring::hkdf::KeyType for MyLength {
    fn len(&self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkdf_basic() {
        let master_key = vec![0x42u8; 32];
        let salt = b"test-salt";
        let info = b"application-context";

        let derived = KeyDerivation::hkdf(&master_key, salt, info, 32).unwrap();
        assert_eq!(derived.len(), 32);
    }

    #[test]
    fn test_hkdf_deterministic() {
        let master_key = vec![0x42u8; 32];
        let salt = b"test-salt";
        let info = b"context";

        let derived1 = KeyDerivation::hkdf(&master_key, salt, info, 32).unwrap();
        let derived2 = KeyDerivation::hkdf(&master_key, salt, info, 32).unwrap();

        assert_eq!(derived1, derived2);
    }

    #[test]
    fn test_hkdf_different_contexts() {
        let master_key = vec![0x42u8; 32];
        let salt = b"test-salt";

        let key1 = KeyDerivation::hkdf(&master_key, salt, b"encryption", 32).unwrap();
        let key2 = KeyDerivation::hkdf(&master_key, salt, b"signing", 32).unwrap();

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_encryption_keys() {
        let master_key = vec![0x42u8; 32];
        let salt = b"salt123";

        let (enc_key, auth_key) = KeyDerivation::derive_encryption_keys(&master_key, salt).unwrap();

        assert_eq!(enc_key.len(), 32);
        assert_eq!(auth_key.len(), 32);
        assert_ne!(enc_key, auth_key);
    }

    #[test]
    fn test_pbkdf2_basic() {
        let password = b"strong-password-123";
        let salt = b"random-salt-16by";
        let iterations = 100_000;

        let key = KeyDerivation::pbkdf2(password, salt, iterations, 32).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_pbkdf2_deterministic() {
        let password = b"my-password";
        let salt = b"fixed-salt-value";
        let iterations = 100_000;

        let key1 = KeyDerivation::pbkdf2(password, salt, iterations, 32).unwrap();
        let key2 = KeyDerivation::pbkdf2(password, salt, iterations, 32).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_pbkdf2_insufficient_iterations() {
        let password = b"password";
        let salt = b"1234567890123456";
        let iterations = 1000; // Too low

        let result = KeyDerivation::pbkdf2(password, salt, iterations, 32);
        assert!(result.is_err());
    }

    #[test]
    fn test_pbkdf2_short_salt() {
        let password = b"password";
        let salt = b"short"; // Too short
        let iterations = 100_000;

        let result = KeyDerivation::pbkdf2(password, salt, iterations, 32);
        assert!(result.is_err());
    }

    #[test]
    fn test_argon2id_basic() {
        let password = b"secure-password";
        let salt = b"0123456789abcdef";

        let key = KeyDerivation::argon2id(password, salt, 32).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_argon2id_deterministic() {
        let password = b"test-password";
        let salt = b"1234567890123456";

        let key1 = KeyDerivation::argon2id(password, salt, 32).unwrap();
        let key2 = KeyDerivation::argon2id(password, salt, 32).unwrap();

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_multiple_keys() {
        let master_key = vec![0x42u8; 32];
        let salt = b"multi-key-salt";
        let contexts = &[b"key1".as_slice(), b"key2".as_slice(), b"key3".as_slice()];

        let keys = KeyDerivation::derive_multiple_keys(&master_key, salt, contexts, 32).unwrap();

        assert_eq!(keys.len(), 3);
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[1], keys[2]);
        assert_ne!(keys[0], keys[2]);
    }

    #[test]
    fn test_derive_domain_key() {
        let master_key = vec![0x42u8; 32];

        let db_key = KeyDerivation::derive_domain_key(&master_key, "database", 32).unwrap();
        let api_key = KeyDerivation::derive_domain_key(&master_key, "api", 32).unwrap();

        assert_eq!(db_key.len(), 32);
        assert_eq!(api_key.len(), 32);
        assert_ne!(db_key, api_key);
    }

    #[test]
    fn test_domain_key_deterministic() {
        let master_key = vec![0x42u8; 32];
        let domain = "test-domain";

        let key1 = KeyDerivation::derive_domain_key(&master_key, domain, 32).unwrap();
        let key2 = KeyDerivation::derive_domain_key(&master_key, domain, 32).unwrap();

        assert_eq!(key1, key2);
    }
}
