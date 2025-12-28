//! SLA monitoring and alerting system.

use crate::error::{MetricsError, Result};
use crate::types::{Gauge, HistogramMetric};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// SLA threshold comparison operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThresholdComparison {
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// Equal to
    Equal,
    /// Not equal to
    NotEqual,
}

impl ThresholdComparison {
    /// Evaluate the comparison
    pub fn evaluate(&self, actual: f64, threshold: f64) -> bool {
        match self {
            Self::GreaterThan => actual > threshold,
            Self::GreaterThanOrEqual => actual >= threshold,
            Self::LessThan => actual < threshold,
            Self::LessThanOrEqual => actual <= threshold,
            Self::Equal => (actual - threshold).abs() < f64::EPSILON,
            Self::NotEqual => (actual - threshold).abs() >= f64::EPSILON,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::GreaterThan => "greater than",
            Self::GreaterThanOrEqual => "greater than or equal to",
            Self::LessThan => "less than",
            Self::LessThanOrEqual => "less than or equal to",
            Self::Equal => "equal to",
            Self::NotEqual => "not equal to",
        }
    }
}

/// SLA threshold definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaThreshold {
    /// Threshold name
    pub name: String,

    /// Metric name to monitor
    pub metric_name: String,

    /// Threshold value
    pub threshold: f64,

    /// Comparison operator
    pub comparison: ThresholdComparison,

    /// Description
    pub description: String,

    /// Severity level
    pub severity: AlertSeverity,

    /// Evaluation window in seconds
    pub window_secs: u64,

    /// Minimum consecutive violations before alerting
    pub min_violations: u32,

    /// Enabled flag
    pub enabled: bool,
}

impl SlaThreshold {
    /// Create a new SLA threshold
    pub fn new<S: Into<String>>(
        name: S,
        metric_name: S,
        threshold: f64,
        comparison: ThresholdComparison,
    ) -> Self {
        Self {
            name: name.into(),
            metric_name: metric_name.into(),
            threshold,
            comparison,
            description: String::new(),
            severity: AlertSeverity::Warning,
            window_secs: 60,
            min_violations: 1,
            enabled: true,
        }
    }

    /// Set description
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = description.into();
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: AlertSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set window
    pub fn with_window(mut self, window_secs: u64) -> Self {
        self.window_secs = window_secs;
        self
    }

    /// Set minimum violations
    pub fn with_min_violations(mut self, min_violations: u32) -> Self {
        self.min_violations = min_violations;
        self
    }

    /// Check if a value violates this threshold
    pub fn is_violated(&self, value: f64) -> bool {
        self.enabled && self.comparison.evaluate(value, self.threshold)
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

impl AlertSeverity {
    /// Get human-readable name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

/// SLA violation alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaAlert {
    /// Alert ID
    pub id: String,

    /// Threshold that was violated
    pub threshold: SlaThreshold,

    /// Actual value that triggered the alert
    pub actual_value: f64,

    /// Timestamp when violation occurred
    pub timestamp: DateTime<Utc>,

    /// Number of consecutive violations
    pub violation_count: u32,

    /// Alert status
    pub status: AlertStatus,

    /// Additional context
    pub context: HashMap<String, String>,
}

impl SlaAlert {
    /// Create a new SLA alert
    pub fn new(threshold: SlaThreshold, actual_value: f64, violation_count: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            threshold,
            actual_value,
            timestamp: Utc::now(),
            violation_count,
            status: AlertStatus::Active,
            context: HashMap::new(),
        }
    }

    /// Add context
    pub fn with_context<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Get severity
    pub fn severity(&self) -> AlertSeverity {
        self.threshold.severity
    }

    /// Get metric name
    pub fn metric_name(&self) -> &str {
        &self.threshold.metric_name
    }
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    /// Alert is active
    Active,
    /// Alert has been acknowledged
    Acknowledged,
    /// Alert has been resolved
    Resolved,
}

/// SLA monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaMonitorConfig {
    /// Enable SLA monitoring
    pub enabled: bool,

    /// Check interval in seconds
    pub check_interval_secs: u64,

    /// Maximum active alerts
    pub max_active_alerts: usize,

    /// Alert retention period in seconds
    pub alert_retention_secs: u64,
}

impl Default for SlaMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: 10,
            max_active_alerts: 1000,
            alert_retention_secs: 86400, // 24 hours
        }
    }
}

/// Callback for alert notifications
pub type AlertCallback = Arc<dyn Fn(&SlaAlert) + Send + Sync>;

/// SLA monitoring system
pub struct SlaMonitor {
    config: SlaMonitorConfig,
    thresholds: Arc<RwLock<HashMap<String, SlaThreshold>>>,
    alerts: Arc<RwLock<Vec<SlaAlert>>>,
    violation_counts: Arc<RwLock<HashMap<String, u32>>>,
    alert_tx: mpsc::UnboundedSender<SlaAlert>,
    alert_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<SlaAlert>>>>,
    callbacks: Arc<RwLock<Vec<AlertCallback>>>,
}

impl SlaMonitor {
    /// Create a new SLA monitor
    pub fn new(config: SlaMonitorConfig) -> Self {
        let (alert_tx, alert_rx) = mpsc::unbounded_channel();

        info!("Initializing SLA monitor");

        Self {
            config,
            thresholds: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            violation_counts: Arc::new(RwLock::new(HashMap::new())),
            alert_tx,
            alert_rx: Arc::new(RwLock::new(Some(alert_rx))),
            callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(SlaMonitorConfig::default())
    }

    /// Add an SLA threshold
    pub fn add_threshold(&self, threshold: SlaThreshold) {
        let mut thresholds = self.thresholds.write();
        info!("Adding SLA threshold: {}", threshold.name);
        thresholds.insert(threshold.name.clone(), threshold);
    }

    /// Remove an SLA threshold
    pub fn remove_threshold(&self, name: &str) -> Option<SlaThreshold> {
        let mut thresholds = self.thresholds.write();
        info!("Removing SLA threshold: {}", name);
        thresholds.remove(name)
    }

    /// Get all thresholds
    pub fn thresholds(&self) -> Vec<SlaThreshold> {
        self.thresholds.read().values().cloned().collect()
    }

    /// Get a specific threshold
    pub fn get_threshold(&self, name: &str) -> Option<SlaThreshold> {
        self.thresholds.read().get(name).cloned()
    }

    /// Check a metric value against all relevant thresholds
    pub fn check_metric(&self, metric_name: &str, value: f64) -> Result<Vec<SlaAlert>> {
        let thresholds = self.thresholds.read();
        let mut new_alerts = Vec::new();

        for threshold in thresholds.values() {
            if threshold.metric_name == metric_name && threshold.is_violated(value) {
                let mut counts = self.violation_counts.write();
                let count = counts.entry(threshold.name.clone()).or_insert(0);
                *count += 1;

                if *count >= threshold.min_violations {
                    let alert = SlaAlert::new(threshold.clone(), value, *count);

                    debug!(
                        "SLA violation: {} {} {} (actual: {})",
                        threshold.metric_name,
                        threshold.comparison.description(),
                        threshold.threshold,
                        value
                    );

                    // Send to alert channel
                    if let Err(e) = self.alert_tx.send(alert.clone()) {
                        error!("Failed to send alert: {}", e);
                    }

                    // Store alert
                    let mut alerts = self.alerts.write();
                    alerts.push(alert.clone());

                    // Trim old alerts
                    if alerts.len() > self.config.max_active_alerts {
                        alerts.remove(0);
                    }

                    // Execute callbacks
                    let callbacks = self.callbacks.read();
                    for callback in callbacks.iter() {
                        callback(&alert);
                    }

                    new_alerts.push(alert);
                }
            } else {
                // Reset violation count if not violated
                let mut counts = self.violation_counts.write();
                counts.remove(&threshold.name);
            }
        }

        Ok(new_alerts)
    }

    /// Get all alerts
    pub fn alerts(&self) -> Vec<SlaAlert> {
        self.alerts.read().clone()
    }

    /// Get active alerts
    pub fn active_alerts(&self) -> Vec<SlaAlert> {
        self.alerts
            .read()
            .iter()
            .filter(|a| a.status == AlertStatus::Active)
            .cloned()
            .collect()
    }

    /// Get alerts by severity
    pub fn alerts_by_severity(&self, severity: AlertSeverity) -> Vec<SlaAlert> {
        self.alerts
            .read()
            .iter()
            .filter(|a| a.threshold.severity == severity)
            .cloned()
            .collect()
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&self, alert_id: &str) -> Result<()> {
        let mut alerts = self.alerts.write();

        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.status = AlertStatus::Acknowledged;
            info!("Alert acknowledged: {}", alert_id);
            Ok(())
        } else {
            Err(MetricsError::not_found(format!("Alert not found: {}", alert_id)))
        }
    }

    /// Resolve an alert
    pub fn resolve_alert(&self, alert_id: &str) -> Result<()> {
        let mut alerts = self.alerts.write();

        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.status = AlertStatus::Resolved;
            info!("Alert resolved: {}", alert_id);
            Ok(())
        } else {
            Err(MetricsError::not_found(format!("Alert not found: {}", alert_id)))
        }
    }

    /// Register an alert callback
    pub fn on_alert<F>(&self, callback: F)
    where
        F: Fn(&SlaAlert) + Send + Sync + 'static,
    {
        let mut callbacks = self.callbacks.write();
        callbacks.push(Arc::new(callback));
    }

    /// Clear all alerts
    pub fn clear_alerts(&self) {
        self.alerts.write().clear();
        self.violation_counts.write().clear();
        info!("All alerts cleared");
    }

    /// Cleanup old alerts
    pub fn cleanup_old_alerts(&self) {
        let cutoff = Utc::now()
            - chrono::Duration::seconds(self.config.alert_retention_secs as i64);

        let mut alerts = self.alerts.write();
        let before_len = alerts.len();

        alerts.retain(|alert| {
            alert.status == AlertStatus::Active || alert.timestamp > cutoff
        });

        let removed = before_len - alerts.len();
        if removed > 0 {
            info!("Cleaned up {} old alerts", removed);
        }
    }

    /// Start background monitoring
    pub async fn start_alert_processor(self: Arc<Self>) {
        let mut rx = self.alert_rx.write().take();

        if let Some(mut rx) = rx {
            tokio::spawn(async move {
                info!("SLA alert processor started");

                while let Some(alert) = rx.recv().await {
                    match alert.severity() {
                        AlertSeverity::Critical => {
                            error!(
                                "CRITICAL SLA VIOLATION: {} - {} {} {}",
                                alert.threshold.name,
                                alert.threshold.metric_name,
                                alert.threshold.comparison.description(),
                                alert.threshold.threshold
                            );
                        }
                        AlertSeverity::Error => {
                            error!(
                                "SLA VIOLATION: {} - {} {} {}",
                                alert.threshold.name,
                                alert.threshold.metric_name,
                                alert.threshold.comparison.description(),
                                alert.threshold.threshold
                            );
                        }
                        AlertSeverity::Warning => {
                            warn!(
                                "SLA WARNING: {} - {} {} {}",
                                alert.threshold.name,
                                alert.threshold.metric_name,
                                alert.threshold.comparison.description(),
                                alert.threshold.threshold
                            );
                        }
                        AlertSeverity::Info => {
                            info!(
                                "SLA INFO: {} - {} {} {}",
                                alert.threshold.name,
                                alert.threshold.metric_name,
                                alert.threshold.comparison.description(),
                                alert.threshold.threshold
                            );
                        }
                    }
                }
            });
        }
    }
}

impl Clone for SlaMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            thresholds: Arc::clone(&self.thresholds),
            alerts: Arc::clone(&self.alerts),
            violation_counts: Arc::clone(&self.violation_counts),
            alert_tx: self.alert_tx.clone(),
            alert_rx: Arc::clone(&self.alert_rx),
            callbacks: Arc::clone(&self.callbacks),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_comparison() {
        let comp = ThresholdComparison::GreaterThan;
        assert!(comp.evaluate(10.0, 5.0));
        assert!(!comp.evaluate(5.0, 10.0));

        let comp = ThresholdComparison::LessThan;
        assert!(comp.evaluate(5.0, 10.0));
        assert!(!comp.evaluate(10.0, 5.0));
    }

    #[test]
    fn test_sla_threshold() {
        let threshold = SlaThreshold::new(
            "latency_threshold",
            "query_latency",
            100.0,
            ThresholdComparison::GreaterThan,
        );

        assert!(threshold.is_violated(150.0));
        assert!(!threshold.is_violated(50.0));
    }

    #[test]
    fn test_sla_monitor() {
        let monitor = SlaMonitor::default();

        let threshold = SlaThreshold::new(
            "test_threshold",
            "test_metric",
            100.0,
            ThresholdComparison::GreaterThan,
        )
        .with_severity(AlertSeverity::Warning);

        monitor.add_threshold(threshold);

        let alerts = monitor.check_metric("test_metric", 150.0).unwrap();
        assert_eq!(alerts.len(), 1);

        let all_alerts = monitor.alerts();
        assert_eq!(all_alerts.len(), 1);
    }

    #[test]
    fn test_alert_acknowledgement() {
        let monitor = SlaMonitor::default();

        let threshold = SlaThreshold::new(
            "test",
            "metric",
            10.0,
            ThresholdComparison::GreaterThan,
        );

        monitor.add_threshold(threshold);
        let alerts = monitor.check_metric("metric", 20.0).unwrap();

        let alert_id = &alerts[0].id;
        monitor.acknowledge_alert(alert_id).unwrap();

        let alert = monitor.alerts().into_iter().find(|a| &a.id == alert_id).unwrap();
        assert_eq!(alert.status, AlertStatus::Acknowledged);
    }
}
