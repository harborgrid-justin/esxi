//! Request logging middleware
//!
//! Provides structured logging for HTTP requests and responses

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

/// Request logging middleware
pub struct RequestLogging;

impl RequestLogging {
    /// Create request logging middleware layer
    pub async fn layer(req: Request, next: Next) -> Response {
        let start = Instant::now();
        let method = req.method().clone();
        let uri = req.uri().clone();
        let version = req.version();

        // Extract request ID if present
        let request_id = req
            .extensions()
            .get::<crate::middleware::RequestId>()
            .map(|id| id.to_string());

        // Log the incoming request
        tracing::info!(
            method = %method,
            uri = %uri,
            version = ?version,
            request_id = ?request_id,
            "Incoming request"
        );

        // Process the request
        let response = next.run(req).await;

        // Calculate request duration
        let duration = start.elapsed();
        let status = response.status();

        // Log the response with appropriate level
        if status.is_server_error() {
            tracing::error!(
                method = %method,
                uri = %uri,
                status = %status,
                duration_ms = %duration.as_millis(),
                request_id = ?request_id,
                "Request completed with server error"
            );
        } else if status.is_client_error() {
            tracing::warn!(
                method = %method,
                uri = %uri,
                status = %status,
                duration_ms = %duration.as_millis(),
                request_id = ?request_id,
                "Request completed with client error"
            );
        } else {
            tracing::info!(
                method = %method,
                uri = %uri,
                status = %status,
                duration_ms = %duration.as_millis(),
                request_id = ?request_id,
                "Request completed successfully"
            );
        }

        response
    }

    /// Log request body (for debugging)
    pub async fn log_request_body(req: Request, next: Next) -> Response {
        // Only log in debug/trace mode
        if tracing::enabled!(tracing::Level::TRACE) {
            let method = req.method();
            let uri = req.uri();

            tracing::trace!(
                method = %method,
                uri = %uri,
                "Request body logging enabled"
            );
        }

        next.run(req).await
    }
}

/// Log slow requests
pub async fn slow_request_logger(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    let response = next.run(req).await;

    let duration = start.elapsed();

    // Log if request took longer than 1 second
    if duration.as_secs() >= 1 {
        tracing::warn!(
            method = %method,
            uri = %uri,
            duration_ms = %duration.as_millis(),
            "Slow request detected"
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_logging() {
        // RequestLogging is stateless, just verify it compiles
        let _ = RequestLogging;
    }
}
