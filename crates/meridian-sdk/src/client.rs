use reqwest::{Client as HttpClient, header};
use url::Url;
use std::time::Duration;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::layers::LayerClient;
use crate::features::FeatureClient;
use crate::query::QueryBuilder;
use crate::analysis::AnalysisClient;

/// Configuration for the Meridian client
#[derive(Clone, Debug)]
pub struct ClientConfig {
    base_url: String,
    api_key: Option<String>,
    timeout: Duration,
    max_retries: u32,
}

impl ClientConfig {
    /// Create a new client configuration
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }

    /// Set API key for authentication
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Main client for interacting with Meridian API
pub struct Client {
    config: ClientConfig,
    http_client: HttpClient,
    base_url: Url,
}

impl Client {
    /// Create a new Meridian client
    pub fn new(config: ClientConfig) -> Result<Self> {
        let base_url = Url::parse(&config.base_url)
            .map_err(|e| Error::ConfigError(format!("Invalid base URL: {}", e)))?;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        if let Some(ref api_key) = config.api_key {
            let mut auth_value = header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| Error::ConfigError(format!("Invalid API key: {}", e)))?;
            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);
        }

        let http_client = HttpClient::builder()
            .default_headers(headers)
            .timeout(config.timeout)
            .build()?;

        Ok(Self {
            config,
            http_client,
            base_url,
        })
    }

    /// Get a layer client for managing layers
    pub fn layers(&self) -> LayerClient {
        LayerClient::new(self)
    }

    /// Get a feature client for managing features
    pub fn features(&self, layer_name: impl Into<String>) -> FeatureClient {
        FeatureClient::new(self, layer_name.into())
    }

    /// Create a query builder
    pub fn query(&self, layer_name: impl Into<String>) -> QueryBuilder {
        QueryBuilder::new(self, layer_name.into())
    }

    /// Get an analysis client for spatial analysis operations
    pub fn analysis(&self) -> AnalysisClient {
        AnalysisClient::new(self)
    }

    /// Get the base URL
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Get the HTTP client
    pub fn http_client(&self) -> &HttpClient {
        &self.http_client
    }

    /// Get the configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Make a GET request
    pub(crate) async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = self.base_url.join(path)?;
        let response = self.http_client.get(url).send().await?;

        self.handle_response(response).await
    }

    /// Make a POST request
    pub(crate) async fn post<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        let url = self.base_url.join(path)?;
        let response = self.http_client.post(url).json(body).send().await?;

        self.handle_response(response).await
    }

    /// Make a PUT request
    pub(crate) async fn put<B, T>(&self, path: &str, body: &B) -> Result<T>
    where
        B: Serialize,
        T: for<'de> Deserialize<'de>,
    {
        let url = self.base_url.join(path)?;
        let response = self.http_client.put(url).json(body).send().await?;

        self.handle_response(response).await
    }

    /// Make a DELETE request
    pub(crate) async fn delete(&self, path: &str) -> Result<()> {
        let url = self.base_url.join(path)?;
        let response = self.http_client.delete(url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api_error(status, message))
        }
    }

    /// Handle HTTP response
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();

        if status.is_success() {
            response.json().await.map_err(Into::into)
        } else {
            let status_code = status.as_u16();
            let message = response.text().await.unwrap_or_default();

            match status_code {
                404 => Err(Error::not_found(message)),
                401 => Err(Error::AuthenticationError(message)),
                403 => Err(Error::AuthorizationError(message)),
                400 => Err(Error::validation(message)),
                _ => Err(Error::api_error(status_code, message)),
            }
        }
    }

    /// Health check
    pub async fn health(&self) -> Result<HealthStatus> {
        self.get("/health").await
    }

    /// Get API version
    pub async fn version(&self) -> Result<VersionInfo> {
        self.get("/version").await
    }
}

/// Health status response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub database: bool,
    pub timestamp: String,
}

/// Version information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionInfo {
    pub version: String,
    pub api_version: String,
    pub build: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = ClientConfig::new("http://localhost:3000")
            .with_api_key("test-key")
            .with_timeout(Duration::from_secs(60))
            .with_max_retries(5);

        assert_eq!(config.base_url, "http://localhost:3000");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_retries, 5);
    }
}
