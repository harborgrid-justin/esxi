//! Application state management
//!
//! Manages shared state across request handlers including database
//! connections, caches, and configuration.

use crate::{config::ServerConfig, error::ServerResult, ServerError};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state
///
/// This struct is cloned for each request handler and contains
/// Arc-wrapped resources for efficient sharing.
#[derive(Clone)]
pub struct AppState {
    /// Server configuration
    pub config: Arc<ServerConfig>,

    /// Database connection pool (placeholder)
    pub db: Arc<DatabasePool>,

    /// Cache manager (placeholder)
    pub cache: Arc<CacheManager>,

    /// Metrics collector
    pub metrics: Arc<RwLock<MetricsCollector>>,
}

impl AppState {
    /// Create a new application state with the given configuration
    pub async fn new(config: ServerConfig) -> ServerResult<Self> {
        // Initialize database pool
        let db = DatabasePool::new(&config.database).await?;

        // Initialize cache
        let cache = CacheManager::new(&config.cache).await?;

        // Initialize metrics
        let metrics = MetricsCollector::new();

        Ok(Self {
            config: Arc::new(config),
            db: Arc::new(db),
            cache: Arc::new(cache),
            metrics: Arc::new(RwLock::new(metrics)),
        })
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Record a metric
    pub async fn record_metric(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.record(name, value);
    }

    /// Increment a counter metric
    pub async fn increment_counter(&self, name: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.increment(name);
    }
}

/// Database connection pool
///
/// Placeholder for the actual database pool implementation
/// which would integrate with meridian-db
pub struct DatabasePool {
    // This would contain the actual connection pool
    _phantom: std::marker::PhantomData<()>,
}

impl DatabasePool {
    /// Create a new database pool
    pub async fn new(config: &crate::config::DatabaseConfig) -> ServerResult<Self> {
        // TODO: Initialize actual database pool using meridian-db
        tracing::info!("Initializing database pool with URL: {}",
            config.url.split('@').last().unwrap_or("unknown"));

        // Placeholder validation
        if config.max_connections == 0 {
            return Err(ServerError::Configuration(
                "Max connections must be greater than 0".to_string()
            ));
        }

        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }

    /// Get a connection from the pool
    pub async fn get_connection(&self) -> ServerResult<DatabaseConnection> {
        // TODO: Get actual connection from pool
        Ok(DatabaseConnection {
            _phantom: std::marker::PhantomData,
        })
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            active_connections: 0,
            idle_connections: 0,
            total_connections: 0,
        }
    }
}

/// Database connection
///
/// Placeholder for actual database connection
pub struct DatabaseConnection {
    _phantom: std::marker::PhantomData<()>,
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_connections: usize,
}

/// Cache manager
///
/// Placeholder for cache implementation (in-memory or Redis)
pub struct CacheManager {
    backend: String,
}

impl CacheManager {
    /// Create a new cache manager
    pub async fn new(config: &crate::config::CacheConfig) -> ServerResult<Self> {
        tracing::info!("Initializing {} cache backend", config.backend);

        if config.enabled && config.backend == "redis" {
            if config.redis_url.is_none() {
                return Err(ServerError::Configuration(
                    "Redis URL required when using Redis backend".to_string()
                ));
            }
        }

        Ok(Self {
            backend: config.backend.clone(),
        })
    }

    /// Get a cached value
    pub async fn get<T>(&self, _key: &str) -> ServerResult<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        // TODO: Implement actual cache get
        Ok(None)
    }

    /// Set a cached value
    pub async fn set<T>(&self, _key: &str, _value: &T, _ttl_secs: Option<u64>) -> ServerResult<()>
    where
        T: serde::Serialize,
    {
        // TODO: Implement actual cache set
        Ok(())
    }

    /// Delete a cached value
    pub async fn delete(&self, _key: &str) -> ServerResult<()> {
        // TODO: Implement actual cache delete
        Ok(())
    }

    /// Clear all cached values
    pub async fn clear(&self) -> ServerResult<()> {
        // TODO: Implement actual cache clear
        Ok(())
    }
}

/// Metrics collector
///
/// Collects application metrics for monitoring and observability
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    counters: std::collections::HashMap<String, u64>,
    gauges: std::collections::HashMap<String, f64>,
    histograms: std::collections::HashMap<String, Vec<f64>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            counters: std::collections::HashMap::new(),
            gauges: std::collections::HashMap::new(),
            histograms: std::collections::HashMap::new(),
        }
    }

    /// Record a metric value
    pub fn record(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    /// Increment a counter
    pub fn increment(&mut self, name: &str) {
        let counter = self.counters.entry(name.to_string()).or_insert(0);
        *counter += 1;
    }

    /// Record a histogram value
    pub fn record_histogram(&mut self, name: &str, value: f64) {
        let histogram = self.histograms.entry(name.to_string()).or_insert_with(Vec::new);
        histogram.push(value);
    }

    /// Get all metrics
    pub fn get_all(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            counters: self.counters.clone(),
            gauges: self.gauges.clone(),
            histogram_stats: self.histograms.iter().map(|(k, v)| {
                (k.clone(), HistogramStats::from_values(v))
            }).collect(),
        }
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        self.counters.clear();
        self.gauges.clear();
        self.histograms.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricsSnapshot {
    pub counters: std::collections::HashMap<String, u64>,
    pub gauges: std::collections::HashMap<String, f64>,
    pub histogram_stats: std::collections::HashMap<String, HistogramStats>,
}

/// Statistics for a histogram
#[derive(Debug, Clone, serde::Serialize)]
pub struct HistogramStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
}

impl HistogramStats {
    fn from_values(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                count: 0,
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                median: 0.0,
            };
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let count = sorted.len();
        let min = sorted[0];
        let max = sorted[count - 1];
        let sum: f64 = sorted.iter().sum();
        let mean = sum / count as f64;
        let median = sorted[count / 2];

        Self {
            count,
            min,
            max,
            mean,
            median,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ServerConfig;

    #[tokio::test]
    async fn test_app_state_creation() {
        let config = ServerConfig::default();
        let state = AppState::new(config).await;
        assert!(state.is_ok());
    }

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();

        collector.increment("requests");
        collector.increment("requests");
        collector.record("latency", 42.0);

        let snapshot = collector.get_all();
        assert_eq!(snapshot.counters.get("requests"), Some(&2));
        assert_eq!(snapshot.gauges.get("latency"), Some(&42.0));
    }
}
