//! Security configuration and constants

use std::time::Duration;

/// AES-256-GCM key size in bytes
pub const AES_256_KEY_SIZE: usize = 32;

/// AES-GCM nonce size in bytes
pub const AES_GCM_NONCE_SIZE: usize = 12;

/// ChaCha20-Poly1305 key size in bytes
pub const CHACHA20_KEY_SIZE: usize = 32;

/// ChaCha20-Poly1305 nonce size in bytes
pub const CHACHA20_NONCE_SIZE: usize = 12;

/// HMAC-SHA256 key size in bytes (minimum recommended)
pub const HMAC_KEY_SIZE: usize = 32;

/// Argon2id memory cost in KiB (OWASP recommended: 47 MiB = 47104 KiB)
pub const ARGON2_MEMORY_COST: u32 = 47104;

/// Argon2id time cost (iterations) - OWASP recommended: 1
pub const ARGON2_TIME_COST: u32 = 1;

/// Argon2id parallelism - OWASP recommended: 1
pub const ARGON2_PARALLELISM: u32 = 1;

/// Argon2id salt length in bytes
pub const ARGON2_SALT_LENGTH: usize = 16;

/// Argon2id output hash length in bytes
pub const ARGON2_HASH_LENGTH: usize = 32;

/// JWT access token lifetime
pub const JWT_ACCESS_TOKEN_LIFETIME: Duration = Duration::from_secs(15 * 60); // 15 minutes

/// JWT refresh token lifetime
pub const JWT_REFRESH_TOKEN_LIFETIME: Duration = Duration::from_secs(7 * 24 * 60 * 60); // 7 days

/// Maximum failed authentication attempts before lockout
pub const MAX_AUTH_ATTEMPTS: u32 = 5;

/// Account lockout duration after max failed attempts
pub const ACCOUNT_LOCKOUT_DURATION: Duration = Duration::from_secs(15 * 60); // 15 minutes

/// Key rotation interval (90 days per NIST recommendation)
pub const KEY_ROTATION_INTERVAL: Duration = Duration::from_secs(90 * 24 * 60 * 60);

/// Minimum password length (OWASP ASVS Level 3)
pub const MIN_PASSWORD_LENGTH: usize = 12;

/// Session timeout for inactive sessions
pub const SESSION_TIMEOUT: Duration = Duration::from_secs(30 * 60); // 30 minutes

/// Rate limit: maximum requests per minute per IP
pub const RATE_LIMIT_PER_MINUTE: u32 = 60;

/// Security configuration for the platform
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable strict mode (additional security checks)
    pub strict_mode: bool,

    /// Enable audit logging
    pub audit_enabled: bool,

    /// JWT issuer
    pub jwt_issuer: String,

    /// JWT audience
    pub jwt_audience: String,

    /// Minimum TLS version (1.2 or 1.3)
    pub min_tls_version: String,

    /// Enable HSTS (HTTP Strict Transport Security)
    pub hsts_enabled: bool,

    /// HSTS max age in seconds
    pub hsts_max_age: u64,

    /// Content Security Policy
    pub csp_policy: String,

    /// Enable CORS
    pub cors_enabled: bool,

    /// Allowed CORS origins
    pub cors_origins: Vec<String>,

    /// Enable rate limiting
    pub rate_limiting_enabled: bool,

    /// Key rotation enabled
    pub key_rotation_enabled: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            strict_mode: true,
            audit_enabled: true,
            jwt_issuer: "meridian-platform".to_string(),
            jwt_audience: "meridian-api".to_string(),
            min_tls_version: "1.3".to_string(),
            hsts_enabled: true,
            hsts_max_age: 31536000, // 1 year
            csp_policy: "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';"
                .to_string(),
            cors_enabled: true,
            cors_origins: vec![],
            rate_limiting_enabled: true,
            key_rotation_enabled: true,
        }
    }
}

impl SecurityConfig {
    /// Create a new security configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a production-ready configuration
    pub fn production() -> Self {
        Self {
            strict_mode: true,
            audit_enabled: true,
            ..Self::default()
        }
    }

    /// Create a development configuration (less strict)
    pub fn development() -> Self {
        Self {
            strict_mode: false,
            hsts_enabled: false,
            rate_limiting_enabled: false,
            ..Self::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.jwt_issuer.is_empty() {
            return Err("JWT issuer cannot be empty".to_string());
        }

        if self.jwt_audience.is_empty() {
            return Err("JWT audience cannot be empty".to_string());
        }

        if self.min_tls_version != "1.2" && self.min_tls_version != "1.3" {
            return Err("TLS version must be 1.2 or 1.3".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SecurityConfig::default();
        assert!(config.strict_mode);
        assert!(config.audit_enabled);
        assert_eq!(config.min_tls_version, "1.3");
    }

    #[test]
    fn test_production_config() {
        let config = SecurityConfig::production();
        assert!(config.strict_mode);
        assert!(config.audit_enabled);
        assert!(config.hsts_enabled);
    }

    #[test]
    fn test_development_config() {
        let config = SecurityConfig::development();
        assert!(!config.strict_mode);
        assert!(!config.hsts_enabled);
    }

    #[test]
    fn test_config_validation() {
        let config = SecurityConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = SecurityConfig::default();
        invalid_config.jwt_issuer = String::new();
        assert!(invalid_config.validate().is_err());
    }
}
