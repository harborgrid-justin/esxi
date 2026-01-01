//! CORS Middleware
//!
//! Enterprise CORS handling with configurable policies.

use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// CORS Configuration
#[derive(Clone)]
pub struct CorsMiddleware {
    /// Allowed origins
    allowed_origins: Vec<String>,

    /// Allowed methods
    allowed_methods: Vec<Method>,

    /// Allowed headers
    allowed_headers: Vec<String>,

    /// Exposed headers
    exposed_headers: Vec<String>,

    /// Allow credentials
    allow_credentials: bool,

    /// Max age for preflight cache
    max_age: u64,
}

impl CorsMiddleware {
    /// Create a new CORS middleware with permissive defaults
    pub fn new() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ],
            allowed_headers: vec!["*".to_string()],
            exposed_headers: vec![],
            allow_credentials: false,
            max_age: 3600,
        }
    }

    /// Set allowed origins
    pub fn with_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    /// Set allowed methods
    pub fn with_methods(mut self, methods: Vec<Method>) -> Self {
        self.allowed_methods = methods;
        self
    }

    /// Set allowed headers
    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.allowed_headers = headers;
        self
    }

    /// Set exposed headers
    pub fn with_exposed_headers(mut self, headers: Vec<String>) -> Self {
        self.exposed_headers = headers;
        self
    }

    /// Allow credentials
    pub fn with_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// Set max age
    pub fn with_max_age(mut self, max_age: u64) -> Self {
        self.max_age = max_age;
        self
    }

    /// Check if origin is allowed
    fn is_origin_allowed(&self, origin: &str) -> bool {
        if self.allowed_origins.contains(&"*".to_string()) {
            return true;
        }

        self.allowed_origins.iter().any(|allowed| {
            if allowed.contains('*') {
                // Wildcard matching
                self.wildcard_match(allowed, origin)
            } else {
                allowed == origin
            }
        })
    }

    /// Simple wildcard matching
    fn wildcard_match(&self, pattern: &str, origin: &str) -> bool {
        if let Some(prefix) = pattern.strip_suffix('*') {
            origin.starts_with(prefix)
        } else if let Some(suffix) = pattern.strip_prefix('*') {
            origin.ends_with(suffix)
        } else {
            pattern == origin
        }
    }

    /// Add CORS headers to response
    pub fn add_cors_headers(&self, headers: &mut HeaderMap, origin: Option<&str>) {
        // Access-Control-Allow-Origin
        if let Some(origin) = origin {
            if self.is_origin_allowed(origin) {
                if self.allowed_origins.contains(&"*".to_string()) {
                    headers.insert(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        HeaderValue::from_static("*"),
                    );
                } else if let Ok(value) = HeaderValue::from_str(origin) {
                    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
                }
            }
        }

        // Access-Control-Allow-Methods
        let methods = self
            .allowed_methods
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        if let Ok(value) = HeaderValue::from_str(&methods) {
            headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, value);
        }

        // Access-Control-Allow-Headers
        if !self.allowed_headers.is_empty() {
            let allowed = self.allowed_headers.join(", ");
            if let Ok(value) = HeaderValue::from_str(&allowed) {
                headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, value);
            }
        }

        // Access-Control-Expose-Headers
        if !self.exposed_headers.is_empty() {
            let exposed = self.exposed_headers.join(", ");
            if let Ok(value) = HeaderValue::from_str(&exposed) {
                headers.insert(header::ACCESS_CONTROL_EXPOSE_HEADERS, value);
            }
        }

        // Access-Control-Allow-Credentials
        if self.allow_credentials {
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                HeaderValue::from_static("true"),
            );
        }

        // Access-Control-Max-Age
        if let Ok(value) = HeaderValue::from_str(&self.max_age.to_string()) {
            headers.insert(header::ACCESS_CONTROL_MAX_AGE, value);
        }
    }
}

impl Default for CorsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware handler
pub async fn cors_middleware(
    State(cors): State<Arc<CorsMiddleware>>,
    request: Request,
    next: Next,
) -> Response {
    // Extract origin (clone to avoid borrow checker issues)
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Handle preflight requests
    if request.method() == Method::OPTIONS {
        let mut response = StatusCode::NO_CONTENT.into_response();
        cors.add_cors_headers(response.headers_mut(), origin.as_deref());
        return response;
    }

    // Process request
    let mut response = next.run(request).await;

    // Add CORS headers to response
    cors.add_cors_headers(response.headers_mut(), origin.as_deref());

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_allowed_wildcard() {
        let cors = CorsMiddleware::new();
        assert!(cors.is_origin_allowed("https://example.com"));
        assert!(cors.is_origin_allowed("http://localhost:3000"));
    }

    #[test]
    fn test_origin_allowed_specific() {
        let cors = CorsMiddleware::new()
            .with_origins(vec!["https://example.com".to_string()]);

        assert!(cors.is_origin_allowed("https://example.com"));
        assert!(!cors.is_origin_allowed("https://other.com"));
    }

    #[test]
    fn test_origin_allowed_pattern() {
        let cors = CorsMiddleware::new()
            .with_origins(vec!["https://*.example.com".to_string()]);

        assert!(cors.is_origin_allowed("https://api.example.com"));
        assert!(cors.is_origin_allowed("https://www.example.com"));
        assert!(!cors.is_origin_allowed("https://example.com"));
    }

    #[test]
    fn test_cors_headers() {
        let cors = CorsMiddleware::new()
            .with_origins(vec!["https://example.com".to_string()])
            .with_credentials(true);

        let mut headers = HeaderMap::new();
        cors.add_cors_headers(&mut headers, Some("https://example.com"));

        assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
        assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_METHODS));
        assert!(headers.contains_key(header::ACCESS_CONTROL_ALLOW_CREDENTIALS));
    }
}
