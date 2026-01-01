//! Logging Middleware
//!
//! Enterprise request/response logging with structured logging support.

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Logging Configuration
#[derive(Clone)]
pub struct LoggingMiddleware {
    /// Log request body
    log_request_body: bool,

    /// Log response body
    log_response_body: bool,

    /// Log headers
    log_headers: bool,

    /// Sensitive headers to redact
    sensitive_headers: Vec<String>,
}

impl LoggingMiddleware {
    /// Create a new logging middleware
    pub fn new() -> Self {
        Self {
            log_request_body: false,
            log_response_body: false,
            log_headers: true,
            sensitive_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "x-api-key".to_string(),
            ],
        }
    }

    /// Enable request body logging
    pub fn with_request_body(mut self) -> Self {
        self.log_request_body = true;
        self
    }

    /// Enable response body logging
    pub fn with_response_body(mut self) -> Self {
        self.log_response_body = true;
        self
    }

    /// Disable header logging
    pub fn without_headers(mut self) -> Self {
        self.log_headers = false;
        self
    }

    /// Add sensitive header to redact
    pub fn with_sensitive_header(mut self, header: String) -> Self {
        self.sensitive_headers.push(header.to_lowercase());
        self
    }

    /// Redact sensitive header value
    fn redact_header(&self, name: &str, value: &str) -> String {
        if self.sensitive_headers.contains(&name.to_lowercase()) {
            "[REDACTED]".to_string()
        } else {
            value.to_string()
        }
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Middleware handler
pub async fn logging_middleware(
    State(logger): State<Arc<LoggingMiddleware>>,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let uri = request.uri().to_string();
    let version = format!("{:?}", request.version());

    // Log request headers
    if logger.log_headers {
        let mut headers_log = Vec::new();
        for (name, value) in request.headers() {
            let name_str = name.as_str();
            let value_str = value.to_str().unwrap_or("[invalid UTF-8]");
            let redacted = logger.redact_header(name_str, value_str);
            headers_log.push(format!("{}: {}", name_str, redacted));
        }
        debug!(
            target: "gateway::request",
            method = %method,
            uri = %uri,
            version = %version,
            headers = ?headers_log,
            "Incoming request"
        );
    } else {
        debug!(
            target: "gateway::request",
            method = %method,
            uri = %uri,
            version = %version,
            "Incoming request"
        );
    }

    // Process request
    let response = next.run(request).await;

    // Calculate duration
    let duration = start.elapsed();
    let status = response.status().as_u16();

    // Determine log level based on status code
    if status >= 500 {
        warn!(
            target: "gateway::response",
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed with server error"
        );
    } else if status >= 400 {
        info!(
            target: "gateway::response",
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed with client error"
        );
    } else {
        info!(
            target: "gateway::response",
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = %duration.as_millis(),
            "Request completed successfully"
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_sensitive_header() {
        let logger = LoggingMiddleware::new();

        let redacted = logger.redact_header("authorization", "Bearer secret-token");
        assert_eq!(redacted, "[REDACTED]");

        let not_redacted = logger.redact_header("content-type", "application/json");
        assert_eq!(not_redacted, "application/json");
    }

    #[test]
    fn test_custom_sensitive_header() {
        let logger = LoggingMiddleware::new()
            .with_sensitive_header("x-custom-token".to_string());

        let redacted = logger.redact_header("x-custom-token", "secret");
        assert_eq!(redacted, "[REDACTED]");
    }
}
