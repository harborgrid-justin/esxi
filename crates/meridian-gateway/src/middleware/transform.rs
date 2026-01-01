//! Request/Response Transformation Middleware
//!
//! Enterprise transformation capabilities for modifying requests and responses.

use axum::{
    extract::{Request, State},
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Transformation rule
#[derive(Debug, Clone)]
pub enum TransformRule {
    /// Add header
    AddHeader {
        /// Header name
        name: String,
        /// Header value
        value: String
    },

    /// Remove header
    RemoveHeader {
        /// Header name to remove
        name: String
    },

    /// Replace header value
    ReplaceHeader {
        /// Header name
        name: String,
        /// New header value
        value: String
    },

    /// Add path prefix
    AddPathPrefix {
        /// Path prefix to add
        prefix: String
    },

    /// Remove path prefix
    RemovePathPrefix {
        /// Path prefix to remove
        prefix: String
    },

    /// Rewrite path
    RewritePath {
        /// Source path pattern
        from: String,
        /// Target path
        to: String
    },
}

/// Transform Middleware
#[derive(Clone)]
pub struct TransformMiddleware {
    /// Request transformations
    request_transforms: Arc<Vec<TransformRule>>,

    /// Response transformations
    response_transforms: Arc<Vec<TransformRule>>,

    /// Custom headers to add to all requests
    custom_headers: Arc<HashMap<String, String>>,
}

impl TransformMiddleware {
    /// Create a new transform middleware
    pub fn new() -> Self {
        Self {
            request_transforms: Arc::new(vec![]),
            response_transforms: Arc::new(vec![]),
            custom_headers: Arc::new(HashMap::new()),
        }
    }

    /// Add request transformation
    pub fn with_request_transform(mut self, rule: TransformRule) -> Self {
        let mut transforms = (*self.request_transforms).clone();
        transforms.push(rule);
        self.request_transforms = Arc::new(transforms);
        self
    }

    /// Add response transformation
    pub fn with_response_transform(mut self, rule: TransformRule) -> Self {
        let mut transforms = (*self.response_transforms).clone();
        transforms.push(rule);
        self.response_transforms = Arc::new(transforms);
        self
    }

    /// Add custom header
    pub fn with_custom_header(mut self, name: String, value: String) -> Self {
        let mut headers = (*self.custom_headers).clone();
        headers.insert(name, value);
        self.custom_headers = Arc::new(headers);
        self
    }

    /// Apply transformations to request
    pub fn transform_request(&self, mut request: Request) -> Request {
        // Apply custom headers
        for (name, value) in self.custom_headers.iter() {
            if let Ok(header_value) = HeaderValue::from_str(value) {
                request.headers_mut().insert(
                    name.parse().unwrap_or_else(|_| {
                        http::header::HeaderName::from_static("x-custom")
                    }),
                    header_value,
                );
            }
        }

        // Apply transformation rules
        for rule in self.request_transforms.iter() {
            request = self.apply_request_rule(request, rule);
        }

        request
    }

    /// Apply transformation to response
    pub fn transform_response(&self, mut response: Response) -> Response {
        for rule in self.response_transforms.iter() {
            response = self.apply_response_rule(response, rule);
        }
        response
    }

    /// Apply single transformation rule to request
    fn apply_request_rule(&self, mut request: Request, rule: &TransformRule) -> Request {
        match rule {
            TransformRule::AddHeader { name, value } => {
                if let Ok(header_value) = HeaderValue::from_str(value) {
                    request.headers_mut().insert(
                        name.parse().unwrap_or_else(|_| {
                            http::header::HeaderName::from_static("x-custom")
                        }),
                        header_value,
                    );
                }
            }
            TransformRule::RemoveHeader { name } => {
                if let Ok(header_name) = name.parse::<http::header::HeaderName>() {
                    request.headers_mut().remove(&header_name);
                }
            }
            TransformRule::ReplaceHeader { name, value } => {
                if let Ok(header_name) = name.parse::<http::header::HeaderName>() {
                    request.headers_mut().remove(&header_name);
                    if let Ok(header_value) = HeaderValue::from_str(value) {
                        request.headers_mut().insert(header_name, header_value);
                    }
                }
            }
            TransformRule::AddPathPrefix { prefix } => {
                let current_path = request.uri().path();
                let new_path = format!("{}{}", prefix, current_path);
                if let Ok(uri) = new_path.parse() {
                    *request.uri_mut() = uri;
                }
            }
            TransformRule::RemovePathPrefix { prefix } => {
                let current_path = request.uri().path();
                if let Some(stripped) = current_path.strip_prefix(prefix) {
                    if let Ok(uri) = stripped.parse() {
                        *request.uri_mut() = uri;
                    }
                }
            }
            TransformRule::RewritePath { from, to } => {
                let current_path = request.uri().path();
                if current_path == from {
                    if let Ok(uri) = to.parse() {
                        *request.uri_mut() = uri;
                    }
                }
            }
        }
        request
    }

    /// Apply single transformation rule to response
    fn apply_response_rule(&self, mut response: Response, rule: &TransformRule) -> Response {
        match rule {
            TransformRule::AddHeader { name, value } => {
                if let Ok(header_value) = HeaderValue::from_str(value) {
                    response.headers_mut().insert(
                        name.parse().unwrap_or_else(|_| {
                            http::header::HeaderName::from_static("x-custom")
                        }),
                        header_value,
                    );
                }
            }
            TransformRule::RemoveHeader { name } => {
                if let Ok(header_name) = name.parse::<http::header::HeaderName>() {
                    response.headers_mut().remove(&header_name);
                }
            }
            TransformRule::ReplaceHeader { name, value } => {
                if let Ok(header_name) = name.parse::<http::header::HeaderName>() {
                    response.headers_mut().remove(&header_name);
                    if let Ok(header_value) = HeaderValue::from_str(value) {
                        response.headers_mut().insert(header_name, header_value);
                    }
                }
            }
            _ => {
                // Path transformations don't apply to responses
            }
        }
        response
    }
}

impl Default for TransformMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware handler
pub async fn transform_middleware(
    State(transformer): State<Arc<TransformMiddleware>>,
    request: Request,
    next: Next,
) -> Response {
    // Transform request
    let transformed_request = transformer.transform_request(request);

    // Process request
    let response = next.run(transformed_request).await;

    // Transform response
    transformer.transform_response(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_add_custom_header() {
        let transformer = TransformMiddleware::new()
            .with_custom_header("X-Gateway-Version".to_string(), "1.0".to_string());

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let transformed = transformer.transform_request(request);
        assert!(transformed.headers().contains_key("x-gateway-version"));
    }

    #[test]
    fn test_add_header_rule() {
        let transformer = TransformMiddleware::new()
            .with_request_transform(TransformRule::AddHeader {
                name: "X-Custom".to_string(),
                value: "test".to_string(),
            });

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let transformed = transformer.transform_request(request);
        assert!(transformed.headers().contains_key("x-custom"));
    }

    #[test]
    fn test_remove_header_rule() {
        let transformer = TransformMiddleware::new()
            .with_request_transform(TransformRule::RemoveHeader {
                name: "user-agent".to_string(),
            });

        let request = Request::builder()
            .uri("/test")
            .header("user-agent", "test-agent")
            .body(Body::empty())
            .unwrap();

        let transformed = transformer.transform_request(request);
        assert!(!transformed.headers().contains_key("user-agent"));
    }
}
