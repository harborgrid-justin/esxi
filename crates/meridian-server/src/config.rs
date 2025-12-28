//! Server configuration module
//!
//! Handles loading and validation of server configuration from environment
//! variables, configuration files, and defaults.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,

    /// Server port
    pub port: u16,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// Maximum request body size in bytes
    pub max_body_size: usize,

    /// TLS configuration
    pub tls: Option<TlsConfig>,

    /// CORS configuration
    pub cors: CorsConfig,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    /// Database connection configuration
    pub database: DatabaseConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

/// TLS/SSL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to TLS certificate file
    pub cert_path: PathBuf,

    /// Path to TLS private key file
    pub key_path: PathBuf,

    /// Require client certificates
    pub require_client_cert: bool,

    /// Path to CA certificate for client verification
    pub ca_cert_path: Option<PathBuf>,
}

/// CORS (Cross-Origin Resource Sharing) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allow any origin (development only)
    pub allow_any_origin: bool,

    /// List of allowed origins
    pub allowed_origins: Vec<String>,

    /// Allowed HTTP methods
    pub allowed_methods: Vec<String>,

    /// Allowed headers
    pub allowed_headers: Vec<String>,

    /// Allow credentials
    pub allow_credentials: bool,

    /// Max age for preflight cache (seconds)
    pub max_age_secs: u64,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,

    /// Requests per minute per IP
    pub requests_per_minute: u32,

    /// Burst size
    pub burst_size: u32,

    /// Rate limit by authenticated user instead of IP
    pub by_user: bool,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,

    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Minimum number of connections in the pool
    pub min_connections: u32,

    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,

    /// Idle connection timeout in seconds
    pub idle_timeout_secs: u64,

    /// Enable SQL query logging
    pub log_queries: bool,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    pub enabled: bool,

    /// JWT secret key
    pub jwt_secret: String,

    /// JWT token expiration in seconds
    pub token_expiration_secs: u64,

    /// Refresh token expiration in seconds
    pub refresh_token_expiration_secs: u64,

    /// Enable API key authentication
    pub enable_api_keys: bool,

    /// Enable OAuth2
    pub enable_oauth2: bool,

    /// OAuth2 providers
    pub oauth2_providers: Vec<String>,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,

    /// Cache backend (memory, redis)
    pub backend: String,

    /// Redis connection URL (if using Redis)
    pub redis_url: Option<String>,

    /// Default cache TTL in seconds
    pub default_ttl_secs: u64,

    /// Maximum cache size in MB
    pub max_size_mb: usize,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Log format (json, pretty, compact)
    pub format: String,

    /// Enable request logging
    pub log_requests: bool,

    /// Enable SQL query logging
    pub log_queries: bool,

    /// Log output destination (stdout, file)
    pub output: String,

    /// Log file path (if output is file)
    pub file_path: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            request_timeout_secs: 30,
            max_body_size: 10 * 1024 * 1024, // 10MB
            tls: None,
            cors: CorsConfig::default(),
            rate_limit: RateLimitConfig::default(),
            database: DatabaseConfig::default(),
            auth: AuthConfig::default(),
            cache: CacheConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_any_origin: false,
            allowed_origins: vec!["http://localhost:3000".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "Accept".to_string(),
            ],
            allow_credentials: true,
            max_age_secs: 3600,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 60,
            burst_size: 10,
            by_user: false,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://postgres:postgres@localhost:5432/meridian".to_string(),
            max_connections: 20,
            min_connections: 5,
            connect_timeout_secs: 10,
            idle_timeout_secs: 300,
            log_queries: false,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jwt_secret: "change-me-in-production".to_string(),
            token_expiration_secs: 3600,           // 1 hour
            refresh_token_expiration_secs: 604800, // 7 days
            enable_api_keys: true,
            enable_oauth2: false,
            oauth2_providers: vec![],
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: "memory".to_string(),
            redis_url: None,
            default_ttl_secs: 300,
            max_size_mb: 100,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            log_requests: true,
            log_queries: false,
            output: "stdout".to_string(),
            file_path: None,
        }
    }
}

impl ServerConfig {
    /// Load configuration from environment and config files
    pub fn load() -> Result<Self, config::ConfigError> {
        let mut settings = config::Config::builder()
            // Start with default values
            .add_source(config::Config::try_from(&ServerConfig::default())?)
            // Add environment-specific config file
            .add_source(config::File::with_name("config/server").required(false))
            // Add environment variables with prefix MERIDIAN_
            .add_source(config::Environment::with_prefix("MERIDIAN").separator("__"))
            .build()?;

        settings.try_deserialize()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port must be greater than 0".to_string());
        }

        if self.max_body_size == 0 {
            return Err("Max body size must be greater than 0".to_string());
        }

        if self.auth.enabled && self.auth.jwt_secret == "change-me-in-production" {
            tracing::warn!("Using default JWT secret - CHANGE THIS IN PRODUCTION!");
        }

        if self.database.max_connections < self.database.min_connections {
            return Err("Max connections must be >= min connections".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
    }

    #[test]
    fn test_config_validation() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = ServerConfig::default();
        invalid_config.port = 0;
        assert!(invalid_config.validate().is_err());
    }
}
