//! API data sources for REST, WFS, and WMS services.

use crate::error::{PipelineError, Result, SourceError};
use crate::sources::{DataSource, RecordBatchStream, SourceStatistics};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use async_trait::async_trait;
use futures::stream;
use reqwest::{Client, header::{HeaderMap, HeaderName, HeaderValue}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// API source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiType {
    /// Generic REST API.
    Rest,
    /// OGC Web Feature Service (WFS).
    Wfs,
    /// OGC Web Map Service (WMS).
    Wms,
    /// OGC API - Features.
    OgcApiFeatures,
    /// ArcGIS REST API.
    ArcGisRest,
}

/// API source configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API type.
    pub api_type: ApiType,
    /// Base URL.
    pub base_url: String,
    /// Query parameters.
    pub params: HashMap<String, String>,
    /// HTTP headers.
    #[serde(skip)]
    pub headers: HeaderMap,
    /// Authentication token.
    pub auth_token: Option<String>,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Pagination settings.
    pub pagination: Option<PaginationConfig>,
    /// Retry configuration.
    pub retry_config: RetryConfig,
}

/// Pagination configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    /// Page size parameter name.
    pub size_param: String,
    /// Page offset parameter name.
    pub offset_param: String,
    /// Page size.
    pub page_size: usize,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            size_param: "limit".to_string(),
            offset_param: "offset".to_string(),
            page_size: 1000,
        }
    }
}

/// Retry configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Initial backoff delay in milliseconds.
    pub initial_backoff_ms: u64,
    /// Maximum backoff delay in milliseconds.
    pub max_backoff_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 30000,
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            api_type: ApiType::Rest,
            base_url: String::new(),
            params: HashMap::new(),
            headers: HeaderMap::new(),
            auth_token: None,
            timeout_secs: 30,
            pagination: Some(PaginationConfig::default()),
            retry_config: RetryConfig::default(),
        }
    }
}

/// API data source.
pub struct ApiSource {
    config: ApiConfig,
    client: Client,
}

impl ApiSource {
    /// Create a new API source.
    pub fn new(config: ApiConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| PipelineError::Http(e))?;

        Ok(Self { config, client })
    }

    /// Create a REST API source.
    pub fn rest(base_url: impl Into<String>) -> Result<Self> {
        Self::new(ApiConfig {
            api_type: ApiType::Rest,
            base_url: base_url.into(),
            ..Default::default()
        })
    }

    /// Create a WFS source.
    pub fn wfs(base_url: impl Into<String>) -> Result<Self> {
        let mut config = ApiConfig {
            api_type: ApiType::Wfs,
            base_url: base_url.into(),
            ..Default::default()
        };

        // Add default WFS parameters
        config.params.insert("service".to_string(), "WFS".to_string());
        config.params.insert("version".to_string(), "2.0.0".to_string());
        config.params.insert("request".to_string(), "GetFeature".to_string());
        config.params.insert("outputFormat".to_string(), "application/json".to_string());

        Self::new(config)
    }

    /// Create an OGC API - Features source.
    pub fn ogc_api_features(base_url: impl Into<String>) -> Result<Self> {
        Self::new(ApiConfig {
            api_type: ApiType::OgcApiFeatures,
            base_url: base_url.into(),
            ..Default::default()
        })
    }

    /// Create an ArcGIS REST API source.
    pub fn arcgis_rest(base_url: impl Into<String>) -> Result<Self> {
        let mut config = ApiConfig {
            api_type: ApiType::ArcGisRest,
            base_url: base_url.into(),
            ..Default::default()
        };

        // Add default ArcGIS REST parameters
        config.params.insert("f".to_string(), "geojson".to_string());

        Self::new(config)
    }

    /// Add a query parameter.
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.params.insert(key.into(), value.into());
        self
    }

    /// Add multiple query parameters.
    pub fn with_params(mut self, params: HashMap<String, String>) -> Self {
        self.config.params.extend(params);
        self
    }

    /// Set authentication token.
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.config.auth_token = Some(token.into());
        self
    }

    /// Set timeout.
    pub fn with_timeout(mut self, timeout_secs: u64) -> Result<Self> {
        self.config.timeout_secs = timeout_secs;

        // Rebuild client with new timeout
        self.client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| PipelineError::Http(e))?;

        Ok(self)
    }

    /// Enable pagination.
    pub fn with_pagination(mut self, page_size: usize) -> Self {
        self.config.pagination = Some(PaginationConfig {
            page_size,
            ..Default::default()
        });
        self
    }

    /// Build the request URL.
    fn build_url(&self, offset: Option<usize>) -> Result<Url> {
        let mut url = Url::parse(&self.config.base_url)
            .map_err(|e| PipelineError::Source(SourceError::Parse(e.to_string())))?;

        // Add query parameters
        for (key, value) in &self.config.params {
            url.query_pairs_mut().append_pair(key, value);
        }

        // Add pagination parameters if configured
        if let Some(ref pagination) = self.config.pagination {
            if let Some(offset_value) = offset {
                url.query_pairs_mut()
                    .append_pair(&pagination.offset_param, &offset_value.to_string());
                url.query_pairs_mut()
                    .append_pair(&pagination.size_param, &pagination.page_size.to_string());
            }
        }

        Ok(url)
    }

    /// Fetch a single page of data.
    async fn fetch_page(&self, offset: Option<usize>) -> Result<serde_json::Value> {
        let url = self.build_url(offset)?;

        tracing::debug!(
            api_type = ?self.config.api_type,
            url = %url,
            "Fetching API data"
        );

        let mut request = self.client.get(url.clone());

        // Add authentication header if configured
        if let Some(ref token) = self.config.auth_token {
            request = request.bearer_auth(token);
        }

        // Add custom headers
        request = request.headers(self.config.headers.clone());

        // Execute request with retry logic
        let mut attempt = 0;
        let max_retries = self.config.retry_config.max_retries;

        loop {
            attempt += 1;

            match request.try_clone().unwrap().send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        let json = response.json::<serde_json::Value>().await
                            .map_err(|e| PipelineError::Http(e))?;
                        return Ok(json);
                    } else {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();

                        if attempt >= max_retries {
                            return Err(PipelineError::Source(SourceError::ApiError {
                                endpoint: url.to_string(),
                                message: format!("HTTP {}: {}", status, error_text),
                            }));
                        }

                        tracing::warn!(
                            attempt = attempt,
                            status = %status,
                            "API request failed, retrying"
                        );
                    }
                }
                Err(e) => {
                    if attempt >= max_retries {
                        return Err(PipelineError::Http(e));
                    }

                    tracing::warn!(
                        attempt = attempt,
                        error = %e,
                        "API request failed, retrying"
                    );
                }
            }

            // Exponential backoff
            let backoff = self.config.retry_config.initial_backoff_ms * 2_u64.pow(attempt - 1);
            let backoff = backoff.min(self.config.retry_config.max_backoff_ms);
            tokio::time::sleep(Duration::from_millis(backoff)).await;
        }
    }

    /// Create schema for API data.
    fn create_schema(&self) -> SchemaRef {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, true),
            Field::new("geometry", DataType::Utf8, true),
            Field::new("properties", DataType::Utf8, true),
        ]))
    }
}

#[async_trait]
impl DataSource for ApiSource {
    async fn schema(&self) -> Result<SchemaRef> {
        Ok(self.create_schema())
    }

    async fn read(&self) -> Result<RecordBatchStream> {
        tracing::info!(
            api_type = ?self.config.api_type,
            base_url = %self.config.base_url,
            "Reading from API source"
        );

        // In a real implementation, this would:
        // 1. Fetch data from the API (with pagination if needed)
        // 2. Parse the response (GeoJSON, JSON, etc.)
        // 3. Convert to Arrow RecordBatches
        // 4. Stream the results

        // For now, return empty stream
        let stream = stream::empty();
        Ok(Box::pin(stream))
    }

    async fn statistics(&self) -> SourceStatistics {
        SourceStatistics::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_source_rest() {
        let source = ApiSource::rest("https://api.example.com/features").unwrap();
        assert_eq!(source.config.api_type, ApiType::Rest);
    }

    #[test]
    fn test_api_source_wfs() {
        let source = ApiSource::wfs("https://geo.example.com/wfs").unwrap();
        assert_eq!(source.config.api_type, ApiType::Wfs);
        assert_eq!(source.config.params.get("service"), Some(&"WFS".to_string()));
    }

    #[test]
    fn test_api_source_with_params() {
        let source = ApiSource::rest("https://api.example.com/features")
            .unwrap()
            .with_param("layer", "cities")
            .with_param("format", "json");

        assert_eq!(source.config.params.get("layer"), Some(&"cities".to_string()));
        assert_eq!(source.config.params.get("format"), Some(&"json".to_string()));
    }

    #[test]
    fn test_api_source_with_auth() {
        let source = ApiSource::rest("https://api.example.com/features")
            .unwrap()
            .with_auth_token("secret-token");

        assert_eq!(source.config.auth_token, Some("secret-token".to_string()));
    }

    #[test]
    fn test_api_url_building() {
        let source = ApiSource::rest("https://api.example.com/features")
            .unwrap()
            .with_param("layer", "cities")
            .with_pagination(100);

        let url = source.build_url(Some(0)).unwrap();
        assert!(url.to_string().contains("layer=cities"));
        assert!(url.to_string().contains("limit=100"));
        assert!(url.to_string().contains("offset=0"));
    }
}
