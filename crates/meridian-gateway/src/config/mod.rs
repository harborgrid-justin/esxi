//! Gateway Configuration Module
//!
//! Enterprise-grade configuration management for the API Gateway.
//! Supports dynamic reloading, multi-environment configs, and validation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Invalid configuration value
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    /// Required configuration field is missing
    #[error("Missing required field: {0}")]
    MissingField(String),
    /// Configuration parsing error
    #[error("Parse error: {0}")]
    Parse(String),
}

/// Result type for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Main Gateway Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct GatewayConfig {
    /// Server bind address
    pub server: ServerConfig,

    /// Route configurations
    pub routes: Vec<RouteConfig>,

    /// Authentication settings
    pub auth: AuthConfig,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    /// Circuit breaker settings
    pub circuit_breaker: CircuitBreakerConfig,

    /// Caching configuration
    pub cache: CacheConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// CORS configuration
    pub cors: CorsConfig,
}

/// Server Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Bind address (e.g., "0.0.0.0:8080")
    pub bind: SocketAddr,

    /// Request timeout
    #[serde(with = "humantime_serde")]
    pub request_timeout: Duration,

    /// Keep-alive timeout
    #[serde(with = "humantime_serde")]
    pub keepalive_timeout: Duration,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// TLS configuration (optional)
    pub tls: Option<TlsConfig>,
}

/// TLS Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to certificate file
    pub cert_path: String,

    /// Path to private key file
    pub key_path: String,

    /// Require client certificates
    pub require_client_cert: bool,
}

/// Route Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Route ID
    pub id: String,

    /// Path pattern (supports wildcards)
    pub path: String,

    /// HTTP methods allowed
    pub methods: Vec<String>,

    /// Upstream targets
    pub upstreams: Vec<UpstreamConfig>,

    /// Load balancing strategy
    pub load_balancer: LoadBalancerStrategy,

    /// Route-specific middleware
    pub middleware: Vec<String>,

    /// Route timeout override
    #[serde(with = "humantime_serde")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Duration>,

    /// Enable circuit breaker
    pub circuit_breaker_enabled: bool,

    /// Enable caching
    pub cache_enabled: bool,
}

/// Upstream Server Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    /// Upstream ID
    pub id: String,

    /// Target URL
    pub url: String,

    /// Weight for weighted load balancing
    pub weight: u32,

    /// Health check configuration
    pub health_check: Option<HealthCheckConfig>,

    /// Connection timeout
    #[serde(with = "humantime_serde")]
    pub connect_timeout: Duration,

    /// Maximum number of retries
    pub max_retries: u32,
}

/// Health Check Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check endpoint path
    pub path: String,

    /// Check interval
    #[serde(with = "humantime_serde")]
    pub interval: Duration,

    /// Timeout for health check
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,

    /// Number of consecutive successes required
    pub healthy_threshold: u32,

    /// Number of consecutive failures required
    pub unhealthy_threshold: u32,
}

/// Load Balancer Strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancerStrategy {
    /// Round-robin distribution across upstreams
    RoundRobin,
    /// Route to upstream with fewest active connections
    LeastConnections,
    /// Weighted distribution based on upstream capacity
    Weighted,
    /// Hash client IP to consistently route to same upstream
    IpHash,
    /// Random selection from available upstreams
    Random,
}

/// Authentication Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct AuthConfig {
    /// JWT configuration
    pub jwt: Option<JwtConfig>,

    /// API Key configuration
    pub api_key: Option<ApiKeyConfig>,

    /// OAuth configuration
    pub oauth: Option<OAuthConfig>,
}

/// JWT Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT secret key
    pub secret: String,

    /// Token expiration time
    #[serde(with = "humantime_serde")]
    pub expiration: Duration,

    /// Issuer claim
    pub issuer: String,

    /// Audience claim
    pub audience: String,

    /// Algorithm (HS256, RS256, etc.)
    pub algorithm: String,
}

/// API Key Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// Header name for API key
    pub header_name: String,

    /// Valid API keys (in production, use external store)
    pub keys: HashMap<String, ApiKeyMetadata>,
}

/// API Key Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyMetadata {
    /// Key owner/description
    pub owner: String,

    /// Allowed routes
    pub allowed_routes: Vec<String>,

    /// Rate limit tier
    pub rate_limit_tier: String,
}

/// OAuth Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// OAuth provider (e.g., "auth0", "okta")
    pub provider: String,

    /// Client ID
    pub client_id: String,

    /// Client secret
    pub client_secret: String,

    /// Authorization endpoint
    pub auth_url: String,

    /// Token endpoint
    pub token_url: String,

    /// Redirect URI
    pub redirect_uri: String,

    /// Scopes
    pub scopes: Vec<String>,
}

/// Rate Limiting Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    pub enabled: bool,

    /// Default rate limit (requests per duration)
    pub default_limit: u32,

    /// Duration for rate limit window
    #[serde(with = "humantime_serde")]
    pub window: Duration,

    /// Per-route overrides
    pub route_overrides: HashMap<String, u32>,

    /// Per-tier configurations
    pub tiers: HashMap<String, RateLimitTier>,
}

/// Rate Limit Tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitTier {
    /// Requests per window
    pub limit: u32,

    /// Burst capacity
    pub burst: u32,
}

/// Circuit Breaker Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Enable circuit breaker
    pub enabled: bool,

    /// Failure threshold percentage (0-100)
    pub failure_threshold: f64,

    /// Minimum number of requests before circuit can trip
    pub min_requests: u32,

    /// Time window for measuring failures
    #[serde(with = "humantime_serde")]
    pub window: Duration,

    /// How long to wait before attempting recovery
    #[serde(with = "humantime_serde")]
    pub recovery_timeout: Duration,

    /// Half-open state request limit
    pub half_open_max_requests: u32,
}

/// Cache Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,

    /// Maximum cache size (bytes)
    pub max_size: usize,

    /// Default TTL
    #[serde(with = "humantime_serde")]
    pub default_ttl: Duration,

    /// Cache key strategy
    pub key_strategy: CacheKeyStrategy,

    /// Per-route cache policies
    pub route_policies: HashMap<String, CachePolicy>,
}

/// Cache Key Strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CacheKeyStrategy {
    /// Use full URL
    FullUrl,

    /// Use path only
    PathOnly,

    /// Include query parameters
    PathAndQuery,

    /// Custom strategy
    Custom,
}

/// Cache Policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePolicy {
    /// Enable for this route
    pub enabled: bool,

    /// TTL override
    #[serde(with = "humantime_serde")]
    pub ttl: Duration,

    /// Cache only specific status codes
    pub cache_status_codes: Vec<u16>,

    /// Vary headers
    pub vary_headers: Vec<String>,
}

/// Metrics Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Prometheus endpoint path
    pub endpoint: String,

    /// Metrics port (if different from main server)
    pub port: Option<u16>,

    /// Include detailed request metrics
    pub detailed: bool,

    /// Histogram buckets for latency
    pub latency_buckets: Vec<f64>,
}

/// CORS Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Enable CORS
    pub enabled: bool,

    /// Allowed origins (* for all)
    pub allowed_origins: Vec<String>,

    /// Allowed methods
    pub allowed_methods: Vec<String>,

    /// Allowed headers
    pub allowed_headers: Vec<String>,

    /// Exposed headers
    pub exposed_headers: Vec<String>,

    /// Allow credentials
    pub allow_credentials: bool,

    /// Max age for preflight cache
    #[serde(with = "humantime_serde")]
    pub max_age: Duration,
}


impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0:8080".parse().unwrap(),
            request_timeout: Duration::from_secs(30),
            keepalive_timeout: Duration::from_secs(60),
            max_connections: 10000,
            tls: None,
        }
    }
}


impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_limit: 1000,
            window: Duration::from_secs(60),
            route_overrides: HashMap::new(),
            tiers: HashMap::new(),
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            failure_threshold: 50.0,
            min_requests: 10,
            window: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(30),
            half_open_max_requests: 5,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size: 1024 * 1024 * 1024, // 1GB
            default_ttl: Duration::from_secs(300),
            key_strategy: CacheKeyStrategy::PathAndQuery,
            route_policies: HashMap::new(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/metrics".to_string(),
            port: None,
            detailed: true,
            latency_buckets: vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        }
    }
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            allowed_headers: vec!["*".to_string()],
            exposed_headers: vec![],
            allow_credentials: false,
            max_age: Duration::from_secs(3600),
        }
    }
}

impl GatewayConfig {
    /// Load configuration from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| ConfigError::Parse(e.to_string()))
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate routes
        if self.routes.is_empty() {
            return Err(ConfigError::Invalid("No routes configured".to_string()));
        }

        for route in &self.routes {
            if route.upstreams.is_empty() {
                return Err(ConfigError::Invalid(
                    format!("Route {} has no upstreams", route.id)
                ));
            }
        }

        // Validate circuit breaker
        if self.circuit_breaker.enabled
            && (self.circuit_breaker.failure_threshold < 0.0
                || self.circuit_breaker.failure_threshold > 100.0) {
                return Err(ConfigError::Invalid(
                    "Circuit breaker failure threshold must be between 0 and 100".to_string()
                ));
            }

        Ok(())
    }

    /// Get route by ID
    pub fn get_route(&self, id: &str) -> Option<&RouteConfig> {
        self.routes.iter().find(|r| r.id == id)
    }

    /// Get route by path
    pub fn find_route(&self, path: &str) -> Option<&RouteConfig> {
        self.routes.iter().find(|r| self.path_matches(&r.path, path))
    }

    /// Check if path matches route pattern
    fn path_matches(&self, pattern: &str, path: &str) -> bool {
        // Simple wildcard matching (can be enhanced with regex)
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return path.starts_with(parts[0]) && path.ends_with(parts[1]);
            }
        }
        pattern == path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GatewayConfig::default();
        assert!(config.metrics.enabled);
        assert!(config.rate_limit.enabled);
    }

    #[test]
    fn test_path_matching() {
        let config = GatewayConfig::default();
        assert!(config.path_matches("/api/*", "/api/users"));
        assert!(config.path_matches("/api/users", "/api/users"));
        assert!(!config.path_matches("/api/users", "/api/posts"));
    }
}
