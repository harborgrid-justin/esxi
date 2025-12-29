use crate::types::*;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use metrics::{counter, gauge, histogram};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Metrics collector for monitoring system performance
pub struct MetricsCollector {
    start_times: Arc<DashMap<Uuid, Instant>>,
    scan_counts: Arc<DashMap<ScanStatus, u64>>,
    issue_counts: Arc<DashMap<Severity, u64>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            start_times: Arc::new(DashMap::new()),
            scan_counts: Arc::new(DashMap::new()),
            issue_counts: Arc::new(DashMap::new()),
        }
    }

    /// Initialize Prometheus exporter
    pub fn init_prometheus_exporter(port: u16) -> Result<(), MetricsError> {
        let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
        builder
            .with_http_listener(([0, 0, 0, 0], port))
            .install()
            .map_err(|e| MetricsError::InitializationFailed(e.to_string()))?;

        tracing::info!(port = port, "Prometheus metrics exporter initialized");
        Ok(())
    }

    /// Record scan start
    pub fn record_scan_start(&self, scan_id: Uuid, scan_type: ScanType) {
        self.start_times.insert(scan_id, Instant::now());
        counter!("scans_started_total", "type" => format!("{:?}", scan_type)).increment(1);
        gauge!("scans_active").increment(1.0);
    }

    /// Record scan completion
    pub fn record_scan_complete(&self, scan_id: Uuid, result: &ScanResult) {
        if let Some((_, start)) = self.start_times.remove(&scan_id) {
            let duration = start.elapsed();
            histogram!("scan_duration_seconds").record(duration.as_secs_f64());
        }

        gauge!("scans_active").decrement(1.0);

        match result.status {
            ScanStatus::Completed => {
                counter!("scans_completed_total").increment(1);
                *self.scan_counts.entry(ScanStatus::Completed).or_insert(0) += 1;
            }
            ScanStatus::Failed => {
                counter!("scans_failed_total").increment(1);
                *self.scan_counts.entry(ScanStatus::Failed).or_insert(0) += 1;
            }
            _ => {}
        }

        gauge!("issues_found_total").increment(result.issues_found as f64);
        gauge!("pages_scanned_total").increment(result.pages_scanned as f64);
    }

    /// Record issue detected
    pub fn record_issue(&self, issue: &AccessibilityIssue) {
        counter!(
            "issues_detected_total",
            "severity" => format!("{:?}", issue.severity)
        )
        .increment(1);

        *self.issue_counts.entry(issue.severity).or_insert(0) += 1;

        for wcag in &issue.wcag_criteria {
            counter!("wcag_violations_total", "criterion" => wcag.clone()).increment(1);
        }
    }

    /// Record alert triggered
    pub fn record_alert(&self, alert: &Alert) {
        counter!(
            "alerts_triggered_total",
            "severity" => format!("{:?}", alert.severity)
        )
        .increment(1);
    }

    /// Record alert acknowledged
    pub fn record_alert_acknowledged(&self, alert_id: Uuid) {
        counter!("alerts_acknowledged_total").increment(1);
    }

    /// Record WebSocket connection
    pub fn record_ws_connection(&self) {
        gauge!("websocket_connections_active").increment(1.0);
        counter!("websocket_connections_total").increment(1);
    }

    /// Record WebSocket disconnection
    pub fn record_ws_disconnection(&self) {
        gauge!("websocket_connections_active").decrement(1.0);
    }

    /// Record event published
    pub fn record_event_published(&self, event_type: &str) {
        counter!("events_published_total", "type" => event_type.to_string()).increment(1);
    }

    /// Update system health gauge
    pub fn update_health(&self, health: HealthStatus) {
        let value = match health {
            HealthStatus::Healthy => 1.0,
            HealthStatus::Degraded => 0.5,
            HealthStatus::Unhealthy => 0.0,
            HealthStatus::Unknown => -1.0,
        };
        gauge!("system_health").set(value);
    }

    /// Get scan statistics
    pub fn get_scan_stats(&self) -> ScanStatistics {
        ScanStatistics {
            completed: *self
                .scan_counts
                .get(&ScanStatus::Completed)
                .unwrap_or_default(),
            failed: *self
                .scan_counts
                .get(&ScanStatus::Failed)
                .unwrap_or_default(),
            active: self.start_times.len() as u64,
        }
    }

    /// Get issue statistics
    pub fn get_issue_stats(&self) -> IssueStatistics {
        IssueStatistics {
            critical: *self.issue_counts.get(&Severity::Critical).unwrap_or_default(),
            high: *self.issue_counts.get(&Severity::High).unwrap_or_default(),
            medium: *self.issue_counts.get(&Severity::Medium).unwrap_or_default(),
            low: *self.issue_counts.get(&Severity::Low).unwrap_or_default(),
            info: *self.issue_counts.get(&Severity::Info).unwrap_or_default(),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Scan statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStatistics {
    pub completed: u64,
    pub failed: u64,
    pub active: u64,
}

/// Issue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStatistics {
    pub critical: u64,
    pub high: u64,
    pub medium: u64,
    pub low: u64,
    pub info: u64,
}

impl IssueStatistics {
    pub fn total(&self) -> u64 {
        self.critical + self.high + self.medium + self.low + self.info
    }
}

/// Metrics errors
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Failed to initialize metrics exporter: {0}")]
    InitializationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        let scan_id = Uuid::new_v4();
        collector.record_scan_start(scan_id, ScanType::Full);

        let result = ScanResult {
            id: Uuid::new_v4(),
            scan_id,
            status: ScanStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            duration_ms: Some(1000),
            issues_found: 5,
            issues_by_severity: std::collections::HashMap::new(),
            pages_scanned: 10,
            error: None,
        };

        collector.record_scan_complete(scan_id, &result);

        let stats = collector.get_scan_stats();
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.active, 0);
    }
}
