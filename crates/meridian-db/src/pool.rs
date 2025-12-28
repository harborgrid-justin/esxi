//! Connection pool management with health checks and lifecycle management

use crate::error::{DbError, DbResult};
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::ConnectOptions;
use std::time::Duration;

/// Database connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Database host
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database name
    pub database: String,
    /// Username
    pub username: String,
    /// Password
    pub password: String,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Minimum number of connections
    pub min_connections: u32,
    /// Connection timeout in seconds
    pub connect_timeout: u64,
    /// Idle timeout in seconds
    pub idle_timeout: u64,
    /// Maximum lifetime in seconds
    pub max_lifetime: u64,
    /// Enable SSL
    pub ssl: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "meridian".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
            ssl: false,
        }
    }
}

impl PoolConfig {
    /// Create a new pool configuration builder
    pub fn builder() -> PoolConfigBuilder {
        PoolConfigBuilder::default()
    }

    /// Build connection string
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

/// Builder for PoolConfig
#[derive(Default)]
pub struct PoolConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    max_connections: Option<u32>,
    min_connections: Option<u32>,
    connect_timeout: Option<u64>,
    idle_timeout: Option<u64>,
    max_lifetime: Option<u64>,
    ssl: Option<bool>,
}

impl PoolConfigBuilder {
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = Some(min);
        self
    }

    pub fn connect_timeout(mut self, timeout: u64) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    pub fn idle_timeout(mut self, timeout: u64) -> Self {
        self.idle_timeout = Some(timeout);
        self
    }

    pub fn max_lifetime(mut self, lifetime: u64) -> Self {
        self.max_lifetime = Some(lifetime);
        self
    }

    pub fn ssl(mut self, ssl: bool) -> Self {
        self.ssl = Some(ssl);
        self
    }

    pub fn build(self) -> PoolConfig {
        let default = PoolConfig::default();
        PoolConfig {
            host: self.host.unwrap_or(default.host),
            port: self.port.unwrap_or(default.port),
            database: self.database.unwrap_or(default.database),
            username: self.username.unwrap_or(default.username),
            password: self.password.unwrap_or(default.password),
            max_connections: self.max_connections.unwrap_or(default.max_connections),
            min_connections: self.min_connections.unwrap_or(default.min_connections),
            connect_timeout: self.connect_timeout.unwrap_or(default.connect_timeout),
            idle_timeout: self.idle_timeout.unwrap_or(default.idle_timeout),
            max_lifetime: self.max_lifetime.unwrap_or(default.max_lifetime),
            ssl: self.ssl.unwrap_or(default.ssl),
        }
    }
}

/// Database connection pool wrapper
pub struct Pool {
    inner: PgPool,
    config: PoolConfig,
}

impl Pool {
    /// Create a new connection pool from configuration
    pub async fn new(config: PoolConfig) -> DbResult<Self> {
        let mut connect_opts = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .database(&config.database)
            .username(&config.username)
            .password(&config.password);

        // Disable logging for connection attempts
        connect_opts = connect_opts.disable_statement_logging();

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout))
            .idle_timeout(Duration::from_secs(config.idle_timeout))
            .max_lifetime(Duration::from_secs(config.max_lifetime))
            .connect_with(connect_opts)
            .await
            .map_err(|e| DbError::PoolError(format!("Failed to create pool: {}", e)))?;

        Ok(Self {
            inner: pool,
            config,
        })
    }

    /// Create a new connection pool from connection string
    pub async fn from_url(url: &str) -> DbResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await
            .map_err(|e| DbError::PoolError(format!("Failed to create pool: {}", e)))?;

        // Parse config from URL (simplified)
        let config = PoolConfig::default();

        Ok(Self {
            inner: pool,
            config,
        })
    }

    /// Get the underlying pool
    pub fn inner(&self) -> &PgPool {
        &self.inner
    }

    /// Get pool configuration
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    /// Perform health check
    pub async fn health_check(&self) -> DbResult<HealthStatus> {
        let start = std::time::Instant::now();

        // Simple ping query
        sqlx::query("SELECT 1")
            .execute(&self.inner)
            .await
            .map_err(|e| DbError::PoolError(format!("Health check failed: {}", e)))?;

        let latency = start.elapsed();

        // Check PostGIS availability
        let postgis_available = sqlx::query("SELECT PostGIS_Version()")
            .fetch_optional(&self.inner)
            .await
            .is_ok();

        Ok(HealthStatus {
            healthy: true,
            latency_ms: latency.as_millis() as u64,
            connections: self.inner.size(),
            idle_connections: self.inner.num_idle(),
            postgis_available,
        })
    }

    /// Close the pool
    pub async fn close(self) {
        self.inner.close().await;
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            size: self.inner.size(),
            idle: self.inner.num_idle(),
            max: self.config.max_connections,
            min: self.config.min_connections,
        }
    }
}

/// Health status of the database connection
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Whether the database is healthy
    pub healthy: bool,
    /// Query latency in milliseconds
    pub latency_ms: u64,
    /// Number of active connections
    pub connections: u32,
    /// Number of idle connections
    pub idle_connections: usize,
    /// PostGIS extension is available
    pub postgis_available: bool,
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Current pool size
    pub size: u32,
    /// Number of idle connections
    pub idle: usize,
    /// Maximum connections
    pub max: u32,
    /// Minimum connections
    pub min: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_builder() {
        let config = PoolConfig::builder()
            .host("localhost")
            .port(5432)
            .database("test")
            .username("user")
            .password("pass")
            .max_connections(20)
            .build();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "test");
        assert_eq!(config.max_connections, 20);
    }

    #[test]
    fn test_connection_string() {
        let config = PoolConfig::builder()
            .host("localhost")
            .port(5432)
            .database("meridian")
            .username("user")
            .password("secret")
            .build();

        assert_eq!(
            config.connection_string(),
            "postgres://user:secret@localhost:5432/meridian"
        );
    }
}
