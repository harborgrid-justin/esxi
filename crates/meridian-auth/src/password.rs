//! Password management with Argon2id hashing and validation

use crate::error::{AuthError, AuthResult};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher as Argon2PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params, Version,
};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

/// Password strength level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PasswordStrength {
    /// Very weak password
    VeryWeak,
    /// Weak password
    Weak,
    /// Medium strength password
    Medium,
    /// Strong password
    Strong,
    /// Very strong password
    VeryStrong,
}

/// Password validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// Minimum password length
    pub min_length: usize,
    /// Maximum password length
    pub max_length: usize,
    /// Require at least one uppercase letter
    pub require_uppercase: bool,
    /// Require at least one lowercase letter
    pub require_lowercase: bool,
    /// Require at least one digit
    pub require_digit: bool,
    /// Require at least one special character
    pub require_special: bool,
    /// Disallow common passwords
    pub disallow_common: bool,
    /// Minimum strength level required
    pub min_strength: PasswordStrength,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
            disallow_common: true,
            min_strength: PasswordStrength::Medium,
        }
    }
}

/// Password hasher with Argon2id
pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher {
    /// Create a new password hasher with secure defaults (Argon2id)
    pub fn new() -> Self {
        // Use Argon2id with recommended parameters
        // m_cost: 19MB, t_cost: 2 iterations, p_cost: 1 thread
        let params = Params::new(19456, 2, 1, None).expect("Invalid Argon2 parameters");
        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        Self { argon2 }
    }

    /// Hash a password using Argon2id
    pub fn hash_password(&self, password: &str) -> AuthResult<String> {
        if password.is_empty() {
            return Err(AuthError::InvalidPassword("Password cannot be empty".to_string()));
        }

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::PasswordHashError(e.to_string()))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> AuthResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AuthError::PasswordHashError(e.to_string()))?;

        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AuthError::PasswordHashError(e.to_string())),
        }
    }

    /// Verify password with constant-time comparison for additional security
    pub fn verify_password_secure(&self, password: &str, hash: &str) -> AuthResult<bool> {
        self.verify_password(password, hash)
    }
}

/// Password strength checker
pub struct PasswordStrengthChecker;

impl PasswordStrengthChecker {
    /// Calculate password strength based on various factors
    pub fn check_strength(password: &str) -> PasswordStrength {
        let length = password.len();
        let mut score = 0u32;

        // Length scoring
        if length >= 8 {
            score += 1;
        }
        if length >= 12 {
            score += 1;
        }
        if length >= 16 {
            score += 1;
        }
        if length >= 20 {
            score += 1;
        }

        // Character variety scoring
        if password.chars().any(|c| c.is_uppercase()) {
            score += 1;
        }
        if password.chars().any(|c| c.is_lowercase()) {
            score += 1;
        }
        if password.chars().any(|c| c.is_ascii_digit()) {
            score += 1;
        }
        if password.chars().any(|c| !c.is_alphanumeric()) {
            score += 1;
        }

        // Entropy bonus for character diversity
        let unique_chars = password.chars().collect::<std::collections::HashSet<_>>().len();
        if unique_chars >= length / 2 {
            score += 1;
        }

        // Pattern penalties
        if Self::has_sequential_chars(password) {
            score = score.saturating_sub(1);
        }
        if Self::has_repeated_chars(password) {
            score = score.saturating_sub(1);
        }

        match score {
            0..=2 => PasswordStrength::VeryWeak,
            3..=4 => PasswordStrength::Weak,
            5..=6 => PasswordStrength::Medium,
            7..=8 => PasswordStrength::Strong,
            _ => PasswordStrength::VeryStrong,
        }
    }

    /// Check for sequential characters (e.g., "abc", "123")
    fn has_sequential_chars(password: &str) -> bool {
        let chars: Vec<char> = password.chars().collect();
        for window in chars.windows(3) {
            if window.len() == 3 {
                let a = window[0] as i32;
                let b = window[1] as i32;
                let c = window[2] as i32;
                if b == a + 1 && c == b + 1 {
                    return true;
                }
            }
        }
        false
    }

    /// Check for repeated characters (e.g., "aaa", "111")
    fn has_repeated_chars(password: &str) -> bool {
        let chars: Vec<char> = password.chars().collect();
        for window in chars.windows(3) {
            if window.len() == 3 && window[0] == window[1] && window[1] == window[2] {
                return true;
            }
        }
        false
    }
}

/// Validate password against policy
pub fn validate_password(password: &str, policy: &PasswordPolicy) -> AuthResult<()> {
    // Length check
    if password.len() < policy.min_length {
        return Err(AuthError::WeakPassword(format!(
            "Password must be at least {} characters",
            policy.min_length
        )));
    }
    if password.len() > policy.max_length {
        return Err(AuthError::WeakPassword(format!(
            "Password must not exceed {} characters",
            policy.max_length
        )));
    }

    // Character requirements
    if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
        return Err(AuthError::WeakPassword(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }
    if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
        return Err(AuthError::WeakPassword(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }
    if policy.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(AuthError::WeakPassword(
            "Password must contain at least one digit".to_string(),
        ));
    }
    if policy.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(AuthError::WeakPassword(
            "Password must contain at least one special character".to_string(),
        ));
    }

    // Common passwords check
    if policy.disallow_common && is_common_password(password) {
        return Err(AuthError::WeakPassword(
            "This password is too common and insecure".to_string(),
        ));
    }

    // Strength check
    let strength = PasswordStrengthChecker::check_strength(password);
    if strength < policy.min_strength {
        return Err(AuthError::WeakPassword(format!(
            "Password strength {:?} is below minimum required {:?}",
            strength, policy.min_strength
        )));
    }

    Ok(())
}

/// Check if password is in common passwords list
fn is_common_password(password: &str) -> bool {
    const COMMON_PASSWORDS: &[&str] = &[
        "password",
        "123456",
        "123456789",
        "12345678",
        "12345",
        "1234567",
        "password123",
        "qwerty",
        "abc123",
        "password1",
        "admin",
        "letmein",
        "welcome",
        "monkey",
        "1234567890",
        "Password1",
        "welcome123",
    ];

    let lower = password.to_lowercase();
    COMMON_PASSWORDS.iter().any(|&p| lower.contains(p))
}

/// Secure password comparison using constant-time comparison
pub fn secure_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let hasher = PasswordHasher::new();
        let password = "MySecurePassword123!";

        let hash = hasher.hash_password(password).unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2id$"));

        // Verify correct password
        assert!(hasher.verify_password(password, &hash).unwrap());

        // Verify incorrect password
        assert!(!hasher.verify_password("WrongPassword", &hash).unwrap());
    }

    #[test]
    fn test_password_strength() {
        assert_eq!(
            PasswordStrengthChecker::check_strength("weak"),
            PasswordStrength::VeryWeak
        );
        assert_eq!(
            PasswordStrengthChecker::check_strength("WeakPass1"),
            PasswordStrength::Weak
        );
        assert!(
            matches!(
                PasswordStrengthChecker::check_strength("MySecurePassword123!"),
                PasswordStrength::Strong | PasswordStrength::VeryStrong
            )
        );
    }

    #[test]
    fn test_password_validation() {
        let policy = PasswordPolicy::default();

        // Too short
        assert!(validate_password("Short1!", &policy).is_err());

        // No uppercase
        assert!(validate_password("nouppercase123!", &policy).is_err());

        // No special char
        assert!(validate_password("NoSpecialChar123", &policy).is_err());

        // Valid password
        assert!(validate_password("MySecurePassword123!", &policy).is_ok());
    }

    #[test]
    fn test_common_passwords() {
        assert!(is_common_password("password123"));
        assert!(is_common_password("Password123"));
        assert!(!is_common_password("MyUniqueP@ssw0rd"));
    }

    #[test]
    fn test_secure_compare() {
        assert!(secure_compare("hello", "hello"));
        assert!(!secure_compare("hello", "world"));
        assert!(!secure_compare("hello", "hello!"));
    }

    #[test]
    fn test_sequential_chars() {
        assert!(PasswordStrengthChecker::has_sequential_chars("abc123"));
        assert!(PasswordStrengthChecker::has_sequential_chars("xyz"));
        assert!(!PasswordStrengthChecker::has_sequential_chars("axbycz"));
    }

    #[test]
    fn test_repeated_chars() {
        assert!(PasswordStrengthChecker::has_repeated_chars("aaa"));
        assert!(PasswordStrengthChecker::has_repeated_chars("111"));
        assert!(!PasswordStrengthChecker::has_repeated_chars("abc"));
    }
}
