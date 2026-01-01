//! ML monitoring and observability
//!
//! Provides drift detection and performance metrics for production ML

pub mod drift;
pub mod metrics;

pub use drift::{DriftDetector, DriftType, DriftReport};
pub use metrics::{MetricsCollector, ModelMetrics, PerformanceMetrics};

use serde::{Deserialize, Serialize};

/// Monitoring alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alert
    Info,

    /// Warning alert
    Warning,

    /// Critical alert
    Critical,
}

/// Monitoring alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert identifier
    pub id: uuid::Uuid,

    /// Alert severity
    pub severity: AlertSeverity,

    /// Alert title
    pub title: String,

    /// Alert description
    pub description: String,

    /// Alert timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Alert metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl Alert {
    /// Create a new alert
    pub fn new(severity: AlertSeverity, title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            severity,
            title: title.into(),
            description: description.into(),
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create an info alert
    pub fn info(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(AlertSeverity::Info, title, description)
    }

    /// Create a warning alert
    pub fn warning(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(AlertSeverity::Warning, title, description)
    }

    /// Create a critical alert
    pub fn critical(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(AlertSeverity::Critical, title, description)
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,

    /// System is degraded but operational
    Degraded,

    /// System is unhealthy
    Unhealthy,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Overall health status
    pub status: HealthStatus,

    /// Health check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Component statuses
    pub components: std::collections::HashMap<String, ComponentHealth>,

    /// Active alerts
    pub alerts: Vec<Alert>,
}

impl HealthCheck {
    /// Create a new health check
    pub fn new(status: HealthStatus) -> Self {
        Self {
            status,
            timestamp: chrono::Utc::now(),
            components: std::collections::HashMap::new(),
            alerts: Vec::new(),
        }
    }

    /// Add a component status
    pub fn add_component(&mut self, name: String, health: ComponentHealth) {
        self.components.insert(name, health);
    }

    /// Add an alert
    pub fn add_alert(&mut self, alert: Alert) {
        self.alerts.push(alert);
    }
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component status
    pub status: HealthStatus,

    /// Status message
    pub message: String,

    /// Last check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,

    /// Component metrics
    pub metrics: std::collections::HashMap<String, f64>,
}

impl ComponentHealth {
    /// Create healthy component
    pub fn healthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: message.into(),
            last_check: chrono::Utc::now(),
            metrics: std::collections::HashMap::new(),
        }
    }

    /// Create degraded component
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: message.into(),
            last_check: chrono::Utc::now(),
            metrics: std::collections::HashMap::new(),
        }
    }

    /// Create unhealthy component
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: message.into(),
            last_check: chrono::Utc::now(),
            metrics: std::collections::HashMap::new(),
        }
    }

    /// Add a metric
    pub fn with_metric(mut self, name: String, value: f64) -> Self {
        self.metrics.insert(name, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_creation() {
        let alert = Alert::info("Test", "Test description");
        assert_eq!(alert.severity, AlertSeverity::Info);
        assert_eq!(alert.title, "Test");
    }

    #[test]
    fn test_alert_severity_variants() {
        assert_ne!(AlertSeverity::Info, AlertSeverity::Warning);
        assert_ne!(AlertSeverity::Warning, AlertSeverity::Critical);
    }

    #[test]
    fn test_health_status_variants() {
        assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
        assert_ne!(HealthStatus::Degraded, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_component_health() {
        let health = ComponentHealth::healthy("All systems operational")
            .with_metric("cpu_usage".to_string(), 45.0);

        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.metrics.get("cpu_usage"), Some(&45.0));
    }

    #[test]
    fn test_health_check() {
        let mut check = HealthCheck::new(HealthStatus::Healthy);
        check.add_component(
            "inference".to_string(),
            ComponentHealth::healthy("Running"),
        );

        assert_eq!(check.components.len(), 1);
        assert_eq!(check.status, HealthStatus::Healthy);
    }
}
