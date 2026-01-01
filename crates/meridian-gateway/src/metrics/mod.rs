//! Metrics Module
//!
//! Enterprise metrics collection and monitoring for the API Gateway.

pub mod prometheus;

use std::sync::Arc;
use std::time::Instant;

/// Metrics collector trait
pub trait MetricsCollector: Send + Sync {
    /// Record request
    fn record_request(&self, route: &str, method: &str, status: u16, duration: f64);

    /// Record upstream request
    fn record_upstream(&self, upstream: &str, status: u16, duration: f64);

    /// Increment active connections
    fn increment_connections(&self);

    /// Decrement active connections
    fn decrement_connections(&self);

    /// Record cache hit
    fn record_cache_hit(&self, route: &str);

    /// Record cache miss
    fn record_cache_miss(&self, route: &str);

    /// Record circuit breaker state change
    fn record_circuit_breaker(&self, upstream: &str, state: &str);

    /// Record rate limit
    fn record_rate_limit(&self, route: &str, limited: bool);

    /// Record authentication attempt
    fn record_auth(&self, method: &str, success: bool);

    /// Get current metrics snapshot
    fn snapshot(&self) -> MetricsSnapshot;
}

/// Metrics snapshot for reporting
#[derive(Debug, Clone, Default)]
pub struct MetricsSnapshot {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of currently active connections
    pub active_connections: u64,
    /// Number of cache hits
    pub cache_hits: u64,
    /// Number of cache misses
    pub cache_misses: u64,
    /// Number of rate-limited requests
    pub rate_limited: u64,
    /// Number of successful authentications
    pub auth_success: u64,
    /// Number of failed authentications
    pub auth_failure: u64,
}

/// Request timer for automatic duration tracking
pub struct RequestTimer {
    start: Instant,
    collector: Arc<dyn MetricsCollector>,
    route: String,
    method: String,
}

impl RequestTimer {
    /// Create a new request timer
    pub fn new(collector: Arc<dyn MetricsCollector>, route: String, method: String) -> Self {
        Self {
            start: Instant::now(),
            collector,
            route,
            method,
        }
    }

    /// Complete the request and record metrics
    pub fn complete(self, status: u16) {
        let duration = self.start.elapsed().as_secs_f64();
        self.collector.record_request(&self.route, &self.method, status, duration);
    }
}

impl Drop for RequestTimer {
    fn drop(&mut self) {
        // Record with 0 status if not explicitly completed
        let duration = self.start.elapsed().as_secs_f64();
        self.collector.record_request(&self.route, &self.method, 0, duration);
    }
}

/// No-op metrics collector for testing
pub struct NoOpCollector;

impl MetricsCollector for NoOpCollector {
    fn record_request(&self, _route: &str, _method: &str, _status: u16, _duration: f64) {}
    fn record_upstream(&self, _upstream: &str, _status: u16, _duration: f64) {}
    fn increment_connections(&self) {}
    fn decrement_connections(&self) {}
    fn record_cache_hit(&self, _route: &str) {}
    fn record_cache_miss(&self, _route: &str) {}
    fn record_circuit_breaker(&self, _upstream: &str, _state: &str) {}
    fn record_rate_limit(&self, _route: &str, _limited: bool) {}
    fn record_auth(&self, _method: &str, _success: bool) {}
    fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot::default()
    }
}
