//! Reverse Proxy Implementation
//!
//! Enterprise reverse proxy with connection pooling, timeouts, and retry logic.

use axum::body::Body;
use http::{Request, Response, StatusCode, Uri};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, error};

/// Proxy errors
#[derive(Debug, Error)]
pub enum ProxyError {
    /// Failed to connect to upstream server
    #[error("Failed to connect to upstream: {0}")]
    Connection(String),

    /// Request timed out
    #[error("Request timeout")]
    Timeout,

    /// Invalid upstream URL
    #[error("Invalid upstream URL: {0}")]
    InvalidUrl(String),

    /// Upstream server returned an error
    #[error("Upstream error: {0}")]
    Upstream(String),

    /// Maximum retry attempts exceeded
    #[error("Too many retries")]
    TooManyRetries,
}

impl From<ProxyError> for Response<Body> {
    fn from(error: ProxyError) -> Self {
        let (status, message) = match error {
            ProxyError::Connection(_) => (StatusCode::BAD_GATEWAY, "Bad Gateway"),
            ProxyError::Timeout => (StatusCode::GATEWAY_TIMEOUT, "Gateway Timeout"),
            ProxyError::InvalidUrl(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Invalid Configuration"),
            ProxyError::Upstream(_) => (StatusCode::BAD_GATEWAY, "Upstream Error"),
            ProxyError::TooManyRetries => (StatusCode::BAD_GATEWAY, "Too Many Retries"),
        };

        Response::builder()
            .status(status)
            .body(Body::from(message))
            .unwrap()
    }
}

/// Proxy Client Configuration
#[derive(Clone)]
pub struct ProxyConfig {
    /// Connection timeout
    pub connect_timeout: Duration,

    /// Request timeout
    pub request_timeout: Duration,

    /// Maximum retries
    pub max_retries: u32,

    /// Follow redirects
    pub follow_redirects: bool,

    /// Headers to preserve
    pub preserve_headers: Vec<String>,

    /// Headers to remove
    pub remove_headers: Vec<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            max_retries: 3,
            follow_redirects: false,
            preserve_headers: vec![
                "content-type".to_string(),
                "content-length".to_string(),
                "accept".to_string(),
                "accept-encoding".to_string(),
                "user-agent".to_string(),
            ],
            remove_headers: vec![
                "host".to_string(),
                "connection".to_string(),
            ],
        }
    }
}

/// Reverse Proxy Client
pub struct ProxyClient {
    client: Client<hyper_util::client::legacy::connect::HttpConnector, Body>,
    config: ProxyConfig,
}

impl ProxyClient {
    /// Create a new proxy client
    pub fn new(config: ProxyConfig) -> Self {
        let client = Client::builder(TokioExecutor::new()).build_http();

        Self { client, config }
    }

    /// Forward request to upstream
    pub async fn forward(
        &self,
        request: Request<Body>,
        upstream_url: &str,
    ) -> Result<Response<Body>, ProxyError> {
        // For now, only try once (retries would require buffering the request body)
        self.try_forward(request, upstream_url).await
    }

    /// Try to forward request once
    async fn try_forward(
        &self,
        request: Request<Body>,
        upstream_url: &str,
    ) -> Result<Response<Body>, ProxyError> {
        // Build upstream URI
        let upstream_uri = self.build_upstream_uri(request.uri(), upstream_url)?;

        debug!("Proxying request to: {}", upstream_uri);

        // Build upstream request
        let mut upstream_request = Request::builder()
            .method(request.method())
            .uri(upstream_uri.clone());

        // Copy headers
        for (name, value) in request.headers() {
            if self.should_preserve_header(name.as_str()) {
                upstream_request = upstream_request.header(name, value);
            }
        }

        // Set host header
        if let Some(host) = upstream_uri.host() {
            upstream_request = upstream_request.header("host", host);
        }

        // Build request with body
        let upstream_request = upstream_request
            .body(request.into_body())
            .map_err(|e| ProxyError::InvalidUrl(e.to_string()))?;

        // Send request with timeout
        let response = tokio::time::timeout(
            self.config.request_timeout,
            self.client.request(upstream_request),
        )
        .await
        .map_err(|_| ProxyError::Timeout)?
        .map_err(|e| ProxyError::Connection(e.to_string()))?;

        // Convert response
        self.convert_response(response).await
    }

    /// Build upstream URI
    fn build_upstream_uri(&self, original_uri: &Uri, upstream_url: &str) -> Result<Uri, ProxyError> {
        let upstream_base: Uri = upstream_url
            .parse()
            .map_err(|e| ProxyError::InvalidUrl(format!("{}", e)))?;

        let path_and_query = original_uri
            .path_and_query()
            .map(|pq| pq.as_str())
            .unwrap_or("/");

        let uri_str = format!(
            "{}://{}{}",
            upstream_base.scheme_str().unwrap_or("http"),
            upstream_base.authority().unwrap().as_str(),
            path_and_query
        );

        uri_str
            .parse()
            .map_err(|e| ProxyError::InvalidUrl(format!("{}", e)))
    }

    /// Check if header should be preserved
    fn should_preserve_header(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();

        // Check if in remove list
        if self.config.remove_headers.contains(&name_lower) {
            return false;
        }

        // Check if in preserve list or if preserve list is empty (preserve all)
        self.config.preserve_headers.is_empty()
            || self.config.preserve_headers.contains(&name_lower)
            || !name_lower.starts_with("x-")
    }

    /// Convert hyper response to axum response
    async fn convert_response(
        &self,
        response: Response<Incoming>,
    ) -> Result<Response<Body>, ProxyError> {
        let (parts, body) = response.into_parts();

        // Collect body
        let body_bytes = body
            .collect()
            .await
            .map_err(|e| ProxyError::Upstream(e.to_string()))?
            .to_bytes();

        let response = Response::from_parts(parts, Body::from(body_bytes));

        Ok(response)
    }

}

impl Clone for ProxyClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
        }
    }
}

impl Default for ProxyClient {
    fn default() -> Self {
        Self::new(ProxyConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_upstream_uri() {
        let client = ProxyClient::default();

        let original_uri: Uri = "/api/users?page=1".parse().unwrap();
        let upstream_url = "http://localhost:8001";

        let result = client.build_upstream_uri(&original_uri, upstream_url).unwrap();
        assert_eq!(result.to_string(), "http://localhost:8001/api/users?page=1");
    }

    #[test]
    fn test_should_preserve_header() {
        let client = ProxyClient::default();

        assert!(client.should_preserve_header("content-type"));
        assert!(client.should_preserve_header("accept"));
        assert!(!client.should_preserve_header("host"));
        assert!(!client.should_preserve_header("connection"));
    }

    #[test]
    fn test_custom_preserve_headers() {
        let config = ProxyConfig {
            preserve_headers: vec!["x-custom".to_string()],
            remove_headers: vec![],
            ..Default::default()
        };

        let client = ProxyClient::new(config);

        assert!(client.should_preserve_header("x-custom"));
        assert!(!client.should_preserve_header("content-type"));
    }
}
