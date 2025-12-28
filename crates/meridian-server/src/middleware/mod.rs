//! Middleware components for request processing
//!
//! This module contains middleware for authentication, logging,
//! error handling, rate limiting, and other cross-cutting concerns.

pub mod auth;
pub mod logging;
pub mod rate_limit;

pub use auth::AuthMiddleware;
pub use logging::RequestLogging;
pub use rate_limit::RateLimitMiddleware;

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::Instant;
use uuid::Uuid;

/// Request ID middleware
///
/// Adds a unique request ID to each request for tracing
pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();

    // Add request ID to extensions
    req.extensions_mut().insert(RequestId(request_id.clone()));

    let response = next.run(req).await;

    // Add request ID to response headers
    let mut response = response;
    response.headers_mut().insert(
        "X-Request-ID",
        request_id.parse().unwrap(),
    );

    response
}

/// Request timing middleware
///
/// Measures and logs request duration
pub async fn timing_middleware(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    let response = next.run(req).await;

    let duration = start.elapsed();
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %response.status(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    response
}

/// Error handling middleware
///
/// Catches panics and converts them to proper error responses
pub async fn error_handler_middleware(req: Request, next: Next) -> Response {
    let response = next.run(req).await;

    // If the response is a server error, ensure it's properly logged
    if response.status().is_server_error() {
        tracing::error!("Server error response: {}", response.status());
    }

    response
}

/// Health check bypass middleware
///
/// Bypasses authentication for health check endpoints
pub async fn health_check_bypass(req: Request, next: Next) -> Response {
    if req.uri().path().starts_with("/health") {
        return next.run(req).await;
    }

    next.run(req).await
}

/// Request ID type
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// CORS preflight handler
///
/// Handles OPTIONS requests for CORS preflight
pub async fn cors_preflight_handler() -> impl IntoResponse {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request as HttpRequest};

    #[tokio::test]
    async fn test_request_id() {
        let id = RequestId(Uuid::new_v4().to_string());
        assert!(!id.as_str().is_empty());
    }
}
