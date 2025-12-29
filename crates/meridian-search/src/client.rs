//! Elasticsearch client with connection pooling.

use crate::error::{SearchError, SearchResult};
use elasticsearch::{
    auth::Credentials,
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    Elasticsearch,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};
use url::Url;

/// Configuration for Elasticsearch client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Elasticsearch node URLs
    pub nodes: Vec<String>,

    /// Username for authentication
    pub username: Option<String>,

    /// Password for authentication
    pub password: Option<String>,

    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,

    /// Enable compression
    #[serde(default = "default_compression")]
    pub compression: bool,

    /// Certificate validation
    #[serde(default = "default_cert_validation")]
    pub cert_validation: bool,
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> usize {
    3
}

fn default_compression() -> bool {
    true
}

fn default_cert_validation() -> bool {
    true
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            nodes: vec!["http://localhost:9200".to_string()],
            username: None,
            password: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            compression: default_compression(),
            cert_validation: default_cert_validation(),
        }
    }
}

/// Elasticsearch client wrapper with connection pooling.
#[derive(Clone)]
pub struct SearchClient {
    client: Arc<Elasticsearch>,
    config: SearchConfig,
}

impl SearchClient {
    /// Create a new search client.
    pub fn new(config: SearchConfig) -> SearchResult<Self> {
        info!("Initializing Elasticsearch client with {} nodes", config.nodes.len());

        let url = Url::parse(&config.nodes[0])
            .map_err(|e| SearchError::ConfigError(format!("Invalid URL: {}", e)))?;

        let conn_pool = SingleNodeConnectionPool::new(url);
        let mut transport_builder = TransportBuilder::new(conn_pool);

        // Set authentication if provided
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            debug!("Configuring authentication for user: {}", username);
            transport_builder = transport_builder
                .auth(Credentials::Basic(username.clone(), password.clone()));
        }

        // Set timeout
        transport_builder = transport_builder
            .timeout(Duration::from_secs(config.timeout));

        // Disable certificate validation if requested (for development)
        if !config.cert_validation {
            warn!("Certificate validation is disabled - this should only be used in development");
            transport_builder = transport_builder.cert_validation(
                elasticsearch::cert::CertificateValidation::None
            );
        }

        let transport = transport_builder
            .build()
            .map_err(|e| SearchError::ConnectionError(e.to_string()))?;

        let client = Arc::new(Elasticsearch::new(transport));

        Ok(Self { client, config })
    }

    /// Get the underlying Elasticsearch client.
    pub fn client(&self) -> &Elasticsearch {
        &self.client
    }

    /// Get the client configuration.
    pub fn config(&self) -> &SearchConfig {
        &self.config
    }

    /// Check if the Elasticsearch cluster is available.
    pub async fn health_check(&self) -> SearchResult<ClusterHealth> {
        debug!("Performing health check");

        let response = self.client
            .cluster()
            .health(elasticsearch::cluster::ClusterHealthParts::None)
            .send()
            .await?;

        let body = response
            .json::<ClusterHealth>()
            .await
            .map_err(|e| SearchError::ConnectionError(e.to_string()))?;

        info!("Cluster health: {:?}, nodes: {}", body.status, body.number_of_nodes);

        Ok(body)
    }

    /// Get cluster information.
    pub async fn info(&self) -> SearchResult<serde_json::Value> {
        debug!("Getting cluster info");

        let response = self.client
            .info()
            .send()
            .await?;

        let body = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| SearchError::ConnectionError(e.to_string()))?;

        Ok(body)
    }

    /// Ping the cluster to check connectivity.
    pub async fn ping(&self) -> SearchResult<bool> {
        debug!("Pinging cluster");

        let response = self.client
            .ping()
            .send()
            .await?;

        Ok(response.status_code().is_success())
    }
}

/// Cluster health status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterHealth {
    pub cluster_name: String,
    pub status: HealthStatus,
    pub timed_out: bool,
    pub number_of_nodes: u32,
    pub number_of_data_nodes: u32,
    pub active_primary_shards: u32,
    pub active_shards: u32,
    pub relocating_shards: u32,
    pub initializing_shards: u32,
    pub unassigned_shards: u32,
}

/// Health status of the cluster.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Green,
    Yellow,
    Red,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SearchConfig::default();
        assert_eq!(config.nodes.len(), 1);
        assert_eq!(config.timeout, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.compression);
        assert!(config.cert_validation);
    }

    #[test]
    fn test_config_serialization() {
        let config = SearchConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SearchConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.nodes, deserialized.nodes);
    }
}
