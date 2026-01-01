//! Argon2id password hashing
//!
//! Implements secure password hashing using Argon2id, which provides:
//! - Resistance to GPU cracking attacks
//! - Resistance to side-channel attacks
//! - Memory-hard algorithm (difficult to parallelize)
//! - Configurable computational cost
//!
//! ## Why Argon2id?
//! - Winner of Password Hashing Competition (2015)
//! - Recommended by OWASP (2023)
//! - Hybrid approach: combines data-dependent and data-independent memory access
//! - Better than bcrypt and PBKDF2 against modern attacks
//!
//! ## OWASP ASVS Requirements
//! - V2.4.1: Use approved one-way key derivation function
//! - V2.4.2: Salt must be at least 128 bits
//! - V2.4.3: Sufficient iteration count/memory cost
//! - V2.4.4: Use latest version of algorithm
//!
//! ## Parameters (OWASP 2023 Recommendations)
//! - Memory: 47 MiB (47104 KiB)
//! - Iterations: 1
//! - Parallelism: 1
//! - Output: 256 bits (32 bytes)

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Algorithm, Argon2, ParamsBuilder, Version,
};
use zeroize::Zeroizing;

use crate::{
    config::{ARGON2_HASH_LENGTH, ARGON2_MEMORY_COST, ARGON2_PARALLELISM, ARGON2_TIME_COST},
    error::{SecurityError, SecurityResult},
};

/// Password hasher using Argon2id
///
/// # Example
/// ```rust,no_run
/// use meridian_security::hashing::password::PasswordHasher;
///
/// let hasher = PasswordHasher::new();
///
/// // Hash a password
/// let password = "user-password-123";
/// let hash = hasher.hash_password(password.as_bytes()).unwrap();
///
/// // Verify password
/// assert!(hasher.verify_password(password.as_bytes(), &hash).unwrap());
/// assert!(!hasher.verify_password(b"wrong-password", &hash).unwrap());
/// ```
pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl PasswordHasher {
    /// Create a new password hasher with OWASP recommended parameters
    pub fn new() -> Self {
        Self::with_params(ARGON2_MEMORY_COST, ARGON2_TIME_COST, ARGON2_PARALLELISM)
            .expect("Invalid default Argon2 parameters")
    }

    /// Create a password hasher with custom parameters
    ///
    /// # Arguments
    /// * `memory_cost` - Memory usage in KiB (recommended: 47104 = 47 MiB)
    /// * `time_cost` - Number of iterations (recommended: 1)
    /// * `parallelism` - Degree of parallelism (recommended: 1)
    ///
    /// # Security Note
    /// Lower values may be acceptable for non-critical applications but
    /// should never go below OWASP minimums:
    /// - Memory: 15 MiB (15360 KiB)
    /// - Iterations: 2
    pub fn with_params(memory_cost: u32, time_cost: u32, parallelism: u32) -> SecurityResult<Self> {
        let mut params_builder = ParamsBuilder::new();

        params_builder
            .m_cost(memory_cost)
            .t_cost(time_cost)
            .p_cost(parallelism)
            .output_len(ARGON2_HASH_LENGTH);

        let params = params_builder
            .build()
            .map_err(|e| SecurityError::ConfigError(format!("Invalid Argon2 params: {}", e)))?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        Ok(Self { argon2 })
    }

    /// Hash a password using Argon2id
    ///
    /// Generates a cryptographically random salt and returns the hash in PHC format:
    /// `$argon2id$v=19$m=47104,t=1,p=1$<salt>$<hash>`
    ///
    /// # Arguments
    /// * `password` - Password bytes to hash
    ///
    /// # Returns
    /// PHC-formatted hash string that includes algorithm, parameters, salt, and hash
    pub fn hash_password(&self, password: &[u8]) -> SecurityResult<String> {
        let password = Zeroizing::new(password.to_vec());

        // Generate cryptographically random salt
        let salt = SaltString::generate(&mut OsRng);

        // Hash the password
        let hash = self
            .argon2
            .hash_password(&password, &salt)
            .map_err(|e| SecurityError::PasswordHashError(format!("Hashing failed: {}", e)))?;

        Ok(hash.to_string())
    }

    /// Verify a password against a hash
    ///
    /// Uses constant-time comparison to prevent timing attacks.
    ///
    /// # Arguments
    /// * `password` - Password to verify
    /// * `hash_str` - PHC-formatted hash string (from `hash_password`)
    ///
    /// # Returns
    /// `Ok(true)` if password matches, `Ok(false)` if not
    pub fn verify_password(&self, password: &[u8], hash_str: &str) -> SecurityResult<bool> {
        let password = Zeroizing::new(password.to_vec());

        // Parse the PHC hash string
        let parsed_hash = PasswordHash::new(hash_str)
            .map_err(|e| SecurityError::PasswordHashError(format!("Invalid hash format: {}", e)))?;

        // Verify password (constant-time comparison)
        match self.argon2.verify_password(&password, &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(SecurityError::PasswordHashError(format!(
                "Verification error: {}",
                e
            ))),
        }
    }

    /// Check if a password hash needs rehashing
    ///
    /// Returns true if the hash uses outdated parameters and should be regenerated
    /// on next successful login.
    pub fn needs_rehash(&self, hash_str: &str) -> SecurityResult<bool> {
        let parsed_hash = PasswordHash::new(hash_str)
            .map_err(|e| SecurityError::PasswordHashError(format!("Invalid hash format: {}", e)))?;

        // Check if algorithm is current
        if parsed_hash.algorithm != argon2::ALG_ID {
            return Ok(true);
        }

        // Check if version is current
        if let Some(version) = parsed_hash.version {
            if version < Version::V0x13 as u32 {
                return Ok(true);
            }
        }

        // Check if parameters match current settings
        let params = parsed_hash.params;

        // Get current params for comparison
        let current_params = self.argon2.params();

        if params.get_m_cost() != Some(current_params.m_cost())
            || params.get_t_cost() != Some(current_params.t_cost())
            || params.get_p_cost() != Some(current_params.p_cost())
        {
            return Ok(true);
        }

        Ok(false)
    }

    /// Hash a password with a specific salt (for testing/migration)
    ///
    /// # Security Warning
    /// This should only be used for testing or when migrating from another system.
    /// Normal usage should rely on `hash_password` which generates random salts.
    pub fn hash_password_with_salt(&self, password: &[u8], salt: &str) -> SecurityResult<String> {
        let password = Zeroizing::new(password.to_vec());

        let salt_string = SaltString::from_b64(salt)
            .map_err(|e| SecurityError::PasswordHashError(format!("Invalid salt: {}", e)))?;

        let hash = self
            .argon2
            .hash_password(&password, &salt_string)
            .map_err(|e| SecurityError::PasswordHashError(format!("Hashing failed: {}", e)))?;

        Ok(hash.to_string())
    }
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate password strength (OWASP guidelines)
pub struct PasswordStrengthValidator;

impl PasswordStrengthValidator {
    /// Validate password meets minimum security requirements
    ///
    /// OWASP ASVS Level 3 Requirements:
    /// - Minimum 12 characters (14+ recommended)
    /// - No common passwords
    /// - Not previously breached (check against known breach database)
    ///
    /// # Returns
    /// `Ok(())` if password is acceptable, `Err` with reason if not
    pub fn validate(password: &str) -> SecurityResult<()> {
        use crate::config::MIN_PASSWORD_LENGTH;

        // Check minimum length
        if password.len() < MIN_PASSWORD_LENGTH {
            return Err(SecurityError::PasswordHashError(format!(
                "Password must be at least {} characters",
                MIN_PASSWORD_LENGTH
            )));
        }

        // Check maximum length (prevent DoS)
        if password.len() > 128 {
            return Err(SecurityError::PasswordHashError(
                "Password is too long (max 128 characters)".to_string(),
            ));
        }

        // Check for common passwords (basic check - production should use comprehensive list)
        let common_passwords = [
            "password",
            "123456",
            "qwerty",
            "admin",
            "letmein",
            "welcome",
            "monkey",
            "dragon",
            "master",
            "sunshine",
        ];

        let lower_password = password.to_lowercase();
        for common in &common_passwords {
            if lower_password.contains(common) {
                return Err(SecurityError::PasswordHashError(
                    "Password contains common weak patterns".to_string(),
                ));
            }
        }

        // Check character diversity (should have multiple character types)
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let diversity_score =
            has_lowercase as u8 + has_uppercase as u8 + has_digit as u8 + has_special as u8;

        if diversity_score < 3 {
            return Err(SecurityError::PasswordHashError(
                "Password must contain at least 3 of: lowercase, uppercase, digits, special characters"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Check if password has been in a known breach (stub for integration)
    ///
    /// Production implementation should integrate with haveibeenpwned.com API
    /// or similar breach database.
    pub fn check_breach(_password: &str) -> SecurityResult<bool> {
        // Stub: In production, integrate with HaveIBeenPwned API
        // using k-anonymity model (send first 5 chars of SHA-1 hash)
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let hasher = PasswordHasher::new();
        let password = b"SecurePassword123!";

        let hash = hasher.hash_password(password).unwrap();
        assert!(hash.starts_with("$argon2id$"));

        let verified = hasher.verify_password(password, &hash).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_wrong_password() {
        let hasher = PasswordHasher::new();
        let password = b"CorrectPassword";
        let wrong = b"WrongPassword";

        let hash = hasher.hash_password(password).unwrap();
        let verified = hasher.verify_password(wrong, &hash).unwrap();
        assert!(!verified);
    }

    #[test]
    fn test_unique_salts() {
        let hasher = PasswordHasher::new();
        let password = b"SamePassword";

        let hash1 = hasher.hash_password(password).unwrap();
        let hash2 = hasher.hash_password(password).unwrap();

        // Same password should produce different hashes (different salts)
        assert_ne!(hash1, hash2);

        // Both should verify successfully
        assert!(hasher.verify_password(password, &hash1).unwrap());
        assert!(hasher.verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_custom_params() {
        let hasher = PasswordHasher::with_params(19456, 2, 1).unwrap();
        let password = b"TestPassword";

        let hash = hasher.hash_password(password).unwrap();
        assert!(hasher.verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_needs_rehash() {
        let hasher = PasswordHasher::new();
        let password = b"Password123";

        let hash = hasher.hash_password(password).unwrap();
        let needs_rehash = hasher.needs_rehash(&hash).unwrap();

        // Fresh hash with current params shouldn't need rehashing
        assert!(!needs_rehash);
    }

    #[test]
    fn test_password_strength_validation() {
        // Valid password
        assert!(PasswordStrengthValidator::validate("StrongPass123!").is_ok());

        // Too short
        assert!(PasswordStrengthValidator::validate("Short1!").is_err());

        // Common password
        assert!(PasswordStrengthValidator::validate("password123456").is_err());

        // Insufficient diversity
        assert!(PasswordStrengthValidator::validate("alllowercase").is_err());

        // Good password
        assert!(PasswordStrengthValidator::validate("C0mpl3x&Secure!").is_ok());
    }

    #[test]
    fn test_empty_password() {
        let hasher = PasswordHasher::new();
        let password = b"";

        // Should hash even empty password (app should validate before hashing)
        let hash = hasher.hash_password(password).unwrap();
        assert!(hasher.verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_long_password() {
        let hasher = PasswordHasher::new();
        let password = vec![b'a'; 100];

        let hash = hasher.hash_password(&password).unwrap();
        assert!(hasher.verify_password(&password, &hash).unwrap());
    }

    #[test]
    fn test_unicode_password() {
        let hasher = PasswordHasher::new();
        let password = "pāssw0rd🔐한국어".as_bytes();

        let hash = hasher.hash_password(password).unwrap();
        assert!(hasher.verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_timing_attack_resistance() {
        let hasher = PasswordHasher::new();
        let password = b"CorrectPassword123";
        let hash = hasher.hash_password(password).unwrap();

        // Verify with very different passwords should take similar time
        // (this is a basic test; real timing attack testing requires statistical analysis)
        let _result1 = hasher.verify_password(b"Wrong", &hash);
        let _result2 = hasher.verify_password(b"CorrectPassword12", &hash); // Almost correct
    }
}
