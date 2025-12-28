//! Health check system with detailed component status.

use crate::error::{MetricsError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::System;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Component is healthy
    Healthy,
    /// Component is degraded but operational
    Degraded,
    /// Component is unhealthy
    Unhealthy,
    /// Component status is unknown
    Unknown,
}

impl HealthStatus {
    /// Check if status is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Check if status is degraded
    pub fn is_degraded(&self) -> bool {
        matches!(self, HealthStatus::Degraded)
    }

    /// Check if status is unhealthy
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, HealthStatus::Unhealthy)
    }

    /// Get the worst status between two
    pub fn worst(&self, other: &Self) -> Self {
        match (self, other) {
            (HealthStatus::Unhealthy, _) | (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
            (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
            (HealthStatus::Unknown, _) | (_, HealthStatus::Unknown) => HealthStatus::Unknown,
            _ => HealthStatus::Healthy,
        }
    }
}

/// Health check result for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,

    /// Current status
    pub status: HealthStatus,

    /// Status message
    pub message: Option<String>,

    /// Last check timestamp
    pub last_check: DateTime<Utc>,

    /// Check duration in milliseconds
    pub check_duration_ms: f64,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ComponentHealth {
    /// Create a new healthy component
    pub fn healthy<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
            last_check: Utc::now(),
            check_duration_ms: 0.0,
            metadata: HashMap::new(),
        }
    }

    /// Create a degraded component
    pub fn degraded<S: Into<String>>(name: S, message: S) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            last_check: Utc::now(),
            check_duration_ms: 0.0,
            metadata: HashMap::new(),
        }
    }

    /// Create an unhealthy component
    pub fn unhealthy<S: Into<String>>(name: S, message: S) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            last_check: Utc::now(),
            check_duration_ms: 0.0,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Overall system health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall status
    pub status: HealthStatus,

    /// Report timestamp
    pub timestamp: DateTime<Utc>,

    /// System uptime in seconds
    pub uptime_secs: u64,

    /// Component health statuses
    pub components: Vec<ComponentHealth>,

    /// System metrics
    pub system: SystemMetrics,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_usage: f32,

    /// Memory usage in bytes
    pub memory_used: u64,

    /// Total memory in bytes
    pub memory_total: u64,

    /// Memory usage percentage (0-100)
    pub memory_percent: f32,

    /// Number of CPU cores
    pub cpu_cores: usize,

    /// Load average (1 minute)
    pub load_avg_1m: f64,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,

    /// Check interval in seconds
    pub interval_secs: u64,

    /// Timeout for individual checks in seconds
    pub timeout_secs: u64,

    /// CPU usage threshold for degraded status
    pub cpu_degraded_threshold: f32,

    /// CPU usage threshold for unhealthy status
    pub cpu_unhealthy_threshold: f32,

    /// Memory usage threshold for degraded status (percentage)
    pub memory_degraded_threshold: f32,

    /// Memory usage threshold for unhealthy status (percentage)
    pub memory_unhealthy_threshold: f32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 30,
            timeout_secs: 5,
            cpu_degraded_threshold: 70.0,
            cpu_unhealthy_threshold: 90.0,
            memory_degraded_threshold: 75.0,
            memory_unhealthy_threshold: 90.0,
        }
    }
}

/// Health checker trait for custom health checks
#[async_trait::async_trait]
pub trait HealthChecker: Send + Sync {
    /// Perform the health check
    async fn check(&self) -> Result<ComponentHealth>;

    /// Get the checker name
    fn name(&self) -> &str;
}

/// Main health check system
pub struct HealthCheckSystem {
    config: HealthCheckConfig,
    checkers: Arc<RwLock<Vec<Box<dyn HealthChecker>>>>,
    last_report: Arc<RwLock<Option<HealthReport>>>,
    start_time: Instant,
}

impl HealthCheckSystem {
    /// Create a new health check system
    pub fn new(config: HealthCheckConfig) -> Self {
        info!("Initializing health check system");

        Self {
            config,
            checkers: Arc::new(RwLock::new(Vec::new())),
            last_report: Arc::new(RwLock::new(None)),
            start_time: Instant::now(),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(HealthCheckConfig::default())
    }

    /// Register a custom health checker
    pub async fn register_checker(&self, checker: Box<dyn HealthChecker>) {
        let mut checkers = self.checkers.write().await;
        info!("Registering health checker: {}", checker.name());
        checkers.push(checker);
    }

    /// Perform a complete health check
    pub async fn check(&self) -> Result<HealthReport> {
        debug!("Performing health check");

        let mut components = Vec::new();

        // Check system resources
        let system_health = self.check_system_resources().await?;
        components.push(system_health);

        // Run custom health checkers
        let checkers = self.checkers.read().await;
        for checker in checkers.iter() {
            let start = Instant::now();

            match tokio::time::timeout(
                Duration::from_secs(self.config.timeout_secs),
                checker.check(),
            )
            .await
            {
                Ok(Ok(mut health)) => {
                    health.check_duration_ms = start.elapsed().as_secs_f64() * 1000.0;
                    components.push(health);
                }
                Ok(Err(e)) => {
                    error!("Health check failed for {}: {}", checker.name(), e);
                    components.push(
                        ComponentHealth::unhealthy(
                            checker.name().to_string(),
                            format!("Check failed: {}", e),
                        )
                        .with_metadata("duration_ms", format!("{}", start.elapsed().as_millis())),
                    );
                }
                Err(_) => {
                    warn!("Health check timed out for {}", checker.name());
                    components.push(
                        ComponentHealth::unhealthy(checker.name().to_string(), "Check timed out".to_string())
                            .with_metadata("timeout_secs", self.config.timeout_secs.to_string()),
                    );
                }
            }
        }

        // Determine overall status
        let overall_status = components
            .iter()
            .fold(HealthStatus::Healthy, |acc, component| {
                acc.worst(&component.status)
            });

        let system_metrics = self.get_system_metrics().await;

        let report = HealthReport {
            status: overall_status,
            timestamp: Utc::now(),
            uptime_secs: self.start_time.elapsed().as_secs(),
            components,
            system: system_metrics,
        };

        // Cache the report
        *self.last_report.write().await = Some(report.clone());

        Ok(report)
    }

    /// Get the last health report
    pub async fn last_report(&self) -> Option<HealthReport> {
        self.last_report.read().await.clone()
    }

    /// Start background health checking
    pub async fn start_background_checks(self: Arc<Self>) {
        if !self.config.enabled {
            info!("Health checks disabled");
            return;
        }

        let interval = Duration::from_secs(self.config.interval_secs);

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);

            loop {
                ticker.tick().await;

                if let Err(e) = self.check().await {
                    error!("Background health check failed: {}", e);
                }
            }
        });

        info!("Background health checks started");
    }

    /// Check system resources
    async fn check_system_resources(&self) -> Result<ComponentHealth> {
        let metrics = self.get_system_metrics().await;

        let mut status = HealthStatus::Healthy;
        let mut messages = Vec::new();

        // Check CPU
        if metrics.cpu_usage >= self.config.cpu_unhealthy_threshold {
            status = status.worst(&HealthStatus::Unhealthy);
            messages.push(format!("CPU usage critical: {:.1}%", metrics.cpu_usage));
        } else if metrics.cpu_usage >= self.config.cpu_degraded_threshold {
            status = status.worst(&HealthStatus::Degraded);
            messages.push(format!("CPU usage high: {:.1}%", metrics.cpu_usage));
        }

        // Check memory
        if metrics.memory_percent >= self.config.memory_unhealthy_threshold {
            status = status.worst(&HealthStatus::Unhealthy);
            messages.push(format!("Memory usage critical: {:.1}%", metrics.memory_percent));
        } else if metrics.memory_percent >= self.config.memory_degraded_threshold {
            status = status.worst(&HealthStatus::Degraded);
            messages.push(format!("Memory usage high: {:.1}%", metrics.memory_percent));
        }

        let message = if messages.is_empty() {
            None
        } else {
            Some(messages.join("; "))
        };

        Ok(ComponentHealth {
            name: "system".to_string(),
            status,
            message,
            last_check: Utc::now(),
            check_duration_ms: 0.0,
            metadata: HashMap::from([
                ("cpu_usage".to_string(), format!("{:.1}", metrics.cpu_usage)),
                (
                    "memory_percent".to_string(),
                    format!("{:.1}", metrics.memory_percent),
                ),
            ]),
        })
    }

    /// Get current system metrics
    async fn get_system_metrics(&self) -> SystemMetrics {
        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_used = sys.used_memory();
        let memory_total = sys.total_memory();
        let memory_percent = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        let cpu_cores = sys.cpus().len();
        let load_avg_1m = System::load_average().one;

        SystemMetrics {
            cpu_usage,
            memory_used,
            memory_total,
            memory_percent,
            cpu_cores,
            load_avg_1m,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHealthChecker {
        name: String,
        status: HealthStatus,
    }

    #[async_trait::async_trait]
    impl HealthChecker for MockHealthChecker {
        async fn check(&self) -> Result<ComponentHealth> {
            Ok(ComponentHealth {
                name: self.name.clone(),
                status: self.status,
                message: None,
                last_check: Utc::now(),
                check_duration_ms: 0.0,
                metadata: HashMap::new(),
            })
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_health_check_system() {
        let system = HealthCheckSystem::default();

        let checker = Box::new(MockHealthChecker {
            name: "test_component".to_string(),
            status: HealthStatus::Healthy,
        });

        system.register_checker(checker).await;

        let report = system.check().await.unwrap();
        assert!(report.components.len() >= 1);
    }

    #[test]
    fn test_health_status() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Degraded.is_healthy());
        assert!(HealthStatus::Degraded.is_degraded());

        let worst = HealthStatus::Healthy.worst(&HealthStatus::Unhealthy);
        assert_eq!(worst, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_component_health() {
        let health = ComponentHealth::healthy("test")
            .with_metadata("version", "1.0.0");

        assert_eq!(health.name, "test");
        assert!(health.status.is_healthy());
        assert_eq!(health.metadata.get("version").unwrap(), "1.0.0");
    }
}
