//! Prometheus Metrics Exporter
//!
//! Enterprise Prometheus integration for metrics export.

use super::{MetricsCollector, MetricsSnapshot};
use prometheus::{
    CounterVec, Encoder, Gauge, HistogramOpts, HistogramVec, Opts, Registry,
    TextEncoder,
};
use std::sync::Arc;
use parking_lot::RwLock;

/// Prometheus metrics collector
pub struct PrometheusCollector {
    registry: Arc<Registry>,

    // Request metrics
    requests_total: CounterVec,
    request_duration: HistogramVec,
    requests_in_flight: Gauge,

    // Upstream metrics
    upstream_requests: CounterVec,
    upstream_duration: HistogramVec,

    // Connection metrics
    active_connections: Gauge,

    // Cache metrics
    cache_hits: CounterVec,
    cache_misses: CounterVec,

    // Circuit breaker metrics
    circuit_breaker_state: CounterVec,

    // Rate limiting metrics
    rate_limit_total: CounterVec,
    rate_limited_requests: CounterVec,

    // Authentication metrics
    auth_attempts: CounterVec,

    // Snapshot cache
    snapshot: Arc<RwLock<MetricsSnapshot>>,
}

impl PrometheusCollector {
    /// Create a new Prometheus collector
    pub fn new(buckets: Vec<f64>) -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());

        // Request metrics
        let requests_total = CounterVec::new(
            Opts::new("gateway_requests_total", "Total number of HTTP requests")
                .namespace("meridian"),
            &["route", "method", "status"],
        )?;
        registry.register(Box::new(requests_total.clone()))?;

        let request_duration = HistogramVec::new(
            HistogramOpts::new("gateway_request_duration_seconds", "Request duration in seconds")
                .namespace("meridian")
                .buckets(buckets.clone()),
            &["route", "method"],
        )?;
        registry.register(Box::new(request_duration.clone()))?;

        let requests_in_flight = Gauge::new(
            "meridian_gateway_requests_in_flight",
            "Number of requests currently being processed",
        )?;
        registry.register(Box::new(requests_in_flight.clone()))?;

        // Upstream metrics
        let upstream_requests = CounterVec::new(
            Opts::new("gateway_upstream_requests_total", "Total upstream requests")
                .namespace("meridian"),
            &["upstream", "status"],
        )?;
        registry.register(Box::new(upstream_requests.clone()))?;

        let upstream_duration = HistogramVec::new(
            HistogramOpts::new("gateway_upstream_duration_seconds", "Upstream request duration")
                .namespace("meridian")
                .buckets(buckets),
            &["upstream"],
        )?;
        registry.register(Box::new(upstream_duration.clone()))?;

        // Connection metrics
        let active_connections = Gauge::new(
            "meridian_gateway_active_connections",
            "Number of active connections",
        )?;
        registry.register(Box::new(active_connections.clone()))?;

        // Cache metrics
        let cache_hits = CounterVec::new(
            Opts::new("gateway_cache_hits_total", "Total cache hits")
                .namespace("meridian"),
            &["route"],
        )?;
        registry.register(Box::new(cache_hits.clone()))?;

        let cache_misses = CounterVec::new(
            Opts::new("gateway_cache_misses_total", "Total cache misses")
                .namespace("meridian"),
            &["route"],
        )?;
        registry.register(Box::new(cache_misses.clone()))?;

        // Circuit breaker metrics
        let circuit_breaker_state = CounterVec::new(
            Opts::new("gateway_circuit_breaker_state_changes", "Circuit breaker state changes")
                .namespace("meridian"),
            &["upstream", "state"],
        )?;
        registry.register(Box::new(circuit_breaker_state.clone()))?;

        // Rate limiting metrics
        let rate_limit_total = CounterVec::new(
            Opts::new("gateway_rate_limit_checks_total", "Total rate limit checks")
                .namespace("meridian"),
            &["route"],
        )?;
        registry.register(Box::new(rate_limit_total.clone()))?;

        let rate_limited_requests = CounterVec::new(
            Opts::new("gateway_rate_limited_requests_total", "Total rate limited requests")
                .namespace("meridian"),
            &["route"],
        )?;
        registry.register(Box::new(rate_limited_requests.clone()))?;

        // Authentication metrics
        let auth_attempts = CounterVec::new(
            Opts::new("gateway_auth_attempts_total", "Total authentication attempts")
                .namespace("meridian"),
            &["method", "result"],
        )?;
        registry.register(Box::new(auth_attempts.clone()))?;

        Ok(Self {
            registry,
            requests_total,
            request_duration,
            requests_in_flight,
            upstream_requests,
            upstream_duration,
            active_connections,
            cache_hits,
            cache_misses,
            circuit_breaker_state,
            rate_limit_total,
            rate_limited_requests,
            auth_attempts,
            snapshot: Arc::new(RwLock::new(MetricsSnapshot::default())),
        })
    }

    /// Export metrics in Prometheus format
    pub fn export(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }

    /// Get registry for custom metrics
    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }
}

impl MetricsCollector for PrometheusCollector {
    fn record_request(&self, route: &str, method: &str, status: u16, duration: f64) {
        self.requests_total
            .with_label_values(&[route, method, &status.to_string()])
            .inc();

        self.request_duration
            .with_label_values(&[route, method])
            .observe(duration);

        // Update snapshot
        let mut snapshot = self.snapshot.write();
        snapshot.total_requests += 1;
    }

    fn record_upstream(&self, upstream: &str, status: u16, duration: f64) {
        self.upstream_requests
            .with_label_values(&[upstream, &status.to_string()])
            .inc();

        self.upstream_duration
            .with_label_values(&[upstream])
            .observe(duration);
    }

    fn increment_connections(&self) {
        self.active_connections.inc();
        self.requests_in_flight.inc();

        let mut snapshot = self.snapshot.write();
        snapshot.active_connections += 1;
    }

    fn decrement_connections(&self) {
        self.active_connections.dec();
        self.requests_in_flight.dec();

        let mut snapshot = self.snapshot.write();
        snapshot.active_connections = snapshot.active_connections.saturating_sub(1);
    }

    fn record_cache_hit(&self, route: &str) {
        self.cache_hits.with_label_values(&[route]).inc();

        let mut snapshot = self.snapshot.write();
        snapshot.cache_hits += 1;
    }

    fn record_cache_miss(&self, route: &str) {
        self.cache_misses.with_label_values(&[route]).inc();

        let mut snapshot = self.snapshot.write();
        snapshot.cache_misses += 1;
    }

    fn record_circuit_breaker(&self, upstream: &str, state: &str) {
        self.circuit_breaker_state
            .with_label_values(&[upstream, state])
            .inc();
    }

    fn record_rate_limit(&self, route: &str, limited: bool) {
        self.rate_limit_total.with_label_values(&[route]).inc();

        if limited {
            self.rate_limited_requests.with_label_values(&[route]).inc();

            let mut snapshot = self.snapshot.write();
            snapshot.rate_limited += 1;
        }
    }

    fn record_auth(&self, method: &str, success: bool) {
        let result = if success { "success" } else { "failure" };
        self.auth_attempts.with_label_values(&[method, result]).inc();

        let mut snapshot = self.snapshot.write();
        if success {
            snapshot.auth_success += 1;
        } else {
            snapshot.auth_failure += 1;
        }
    }

    fn snapshot(&self) -> MetricsSnapshot {
        self.snapshot.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_collector() {
        let collector = PrometheusCollector::new(vec![0.001, 0.01, 0.1, 1.0]).unwrap();

        collector.record_request("/api/users", "GET", 200, 0.05);
        collector.increment_connections();
        collector.record_cache_hit("/api/users");

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.total_requests, 1);
        assert_eq!(snapshot.active_connections, 1);
        assert_eq!(snapshot.cache_hits, 1);

        let export = collector.export().unwrap();
        assert!(export.contains("meridian_gateway_requests_total"));
    }
}
