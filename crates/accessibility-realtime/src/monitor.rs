use crate::events::{EventBus, MonitorEvent};
use crate::types::*;
use chrono::Utc;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Real-time monitoring engine for accessibility scans
pub struct MonitorEngine {
    /// Event bus for broadcasting updates
    event_bus: EventBus,
    /// Active scans
    active_scans: Arc<DashMap<Uuid, ScanContext>>,
    /// Scan results cache
    results: Arc<DashMap<Uuid, ScanResult>>,
    /// System metrics
    metrics: Arc<RwLock<MonitorMetrics>>,
    /// System health status
    health: Arc<RwLock<HealthStatus>>,
}

/// Context for a running scan
#[derive(Debug, Clone)]
pub struct ScanContext {
    pub config: ScanConfig,
    pub started_at: chrono::DateTime<Utc>,
    pub status: ScanStatus,
    pub pages_scanned: usize,
    pub total_pages: usize,
    pub issues: Vec<AccessibilityIssue>,
}

impl MonitorEngine {
    /// Create a new monitoring engine
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            event_bus,
            active_scans: Arc::new(DashMap::new()),
            results: Arc::new(DashMap::new()),
            metrics: Arc::new(RwLock::new(MonitorMetrics {
                timestamp: Utc::now(),
                active_scans: 0,
                completed_scans: 0,
                failed_scans: 0,
                total_issues: 0,
                issues_by_severity: std::collections::HashMap::new(),
                average_scan_duration_ms: 0.0,
                system_health: HealthStatus::Healthy,
            })),
            health: Arc::new(RwLock::new(HealthStatus::Healthy)),
        }
    }

    /// Start a new scan
    pub async fn start_scan(&self, config: ScanConfig) -> Result<Uuid, MonitorError> {
        let scan_id = config.id;

        // Validate configuration
        self.validate_config(&config)?;

        let context = ScanContext {
            config: config.clone(),
            started_at: Utc::now(),
            status: ScanStatus::Running,
            pages_scanned: 0,
            total_pages: config.targets.len(),
            issues: Vec::new(),
        };

        self.active_scans.insert(scan_id, context);

        // Publish scan started event
        let _ = self.event_bus.publish(MonitorEvent::ScanStarted {
            scan_id,
            config,
            timestamp: Utc::now(),
        });

        // Update metrics
        self.update_metrics().await;

        tracing::info!(scan_id = %scan_id, "Scan started");

        Ok(scan_id)
    }

    /// Update scan progress
    pub async fn update_progress(
        &self,
        scan_id: Uuid,
        pages_scanned: usize,
        issues: Vec<AccessibilityIssue>,
    ) -> Result<(), MonitorError> {
        let mut context = self
            .active_scans
            .get_mut(&scan_id)
            .ok_or(MonitorError::ScanNotFound(scan_id))?;

        context.pages_scanned = pages_scanned;
        context.issues.extend(issues.clone());

        let total_pages = context.total_pages;
        let percentage = if total_pages > 0 {
            (pages_scanned as f64 / total_pages as f64) * 100.0
        } else {
            0.0
        };

        // Publish progress event
        let _ = self.event_bus.publish(MonitorEvent::ScanProgress {
            scan_id,
            pages_scanned,
            total_pages,
            issues_found: context.issues.len(),
            percentage,
            timestamp: Utc::now(),
        });

        // Publish issue detected events
        for issue in issues {
            let _ = self.event_bus.publish(MonitorEvent::IssueDetected {
                issue,
                timestamp: Utc::now(),
            });
        }

        Ok(())
    }

    /// Complete a scan
    pub async fn complete_scan(&self, scan_id: Uuid) -> Result<ScanResult, MonitorError> {
        let (_, context) = self
            .active_scans
            .remove(&scan_id)
            .ok_or(MonitorError::ScanNotFound(scan_id))?;

        let completed_at = Utc::now();
        let duration_ms = (completed_at - context.started_at).num_milliseconds() as u64;

        let mut issues_by_severity = std::collections::HashMap::new();
        for issue in &context.issues {
            *issues_by_severity
                .entry(format!("{:?}", issue.severity).to_lowercase())
                .or_insert(0) += 1;
        }

        let result = ScanResult {
            id: Uuid::new_v4(),
            scan_id,
            status: ScanStatus::Completed,
            started_at: context.started_at,
            completed_at: Some(completed_at),
            duration_ms: Some(duration_ms),
            issues_found: context.issues.len(),
            issues_by_severity,
            pages_scanned: context.pages_scanned,
            error: None,
        };

        self.results.insert(scan_id, result.clone());

        // Publish scan completed event
        let _ = self.event_bus.publish(MonitorEvent::ScanCompleted {
            scan_id,
            result: result.clone(),
            timestamp: Utc::now(),
        });

        // Update metrics
        self.update_metrics().await;

        tracing::info!(
            scan_id = %scan_id,
            duration_ms = duration_ms,
            issues = context.issues.len(),
            "Scan completed"
        );

        Ok(result)
    }

    /// Fail a scan
    pub async fn fail_scan(&self, scan_id: Uuid, error: String) -> Result<(), MonitorError> {
        let (_, context) = self
            .active_scans
            .remove(&scan_id)
            .ok_or(MonitorError::ScanNotFound(scan_id))?;

        let result = ScanResult {
            id: Uuid::new_v4(),
            scan_id,
            status: ScanStatus::Failed,
            started_at: context.started_at,
            completed_at: Some(Utc::now()),
            duration_ms: None,
            issues_found: 0,
            issues_by_severity: std::collections::HashMap::new(),
            pages_scanned: context.pages_scanned,
            error: Some(error.clone()),
        };

        self.results.insert(scan_id, result);

        // Publish scan failed event
        let _ = self.event_bus.publish(MonitorEvent::ScanFailed {
            scan_id,
            error,
            timestamp: Utc::now(),
        });

        // Update metrics
        self.update_metrics().await;

        tracing::error!(scan_id = %scan_id, "Scan failed");

        Ok(())
    }

    /// Get active scans
    pub fn get_active_scans(&self) -> Vec<(Uuid, ScanContext)> {
        self.active_scans
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect()
    }

    /// Get scan result
    pub fn get_result(&self, scan_id: Uuid) -> Option<ScanResult> {
        self.results.get(&scan_id).map(|r| r.value().clone())
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> MonitorMetrics {
        self.metrics.read().await.clone()
    }

    /// Get system health
    pub async fn get_health(&self) -> HealthStatus {
        *self.health.read().await
    }

    /// Update system health
    pub async fn update_health(&self, new_status: HealthStatus, reason: String) {
        let mut health = self.health.write().await;
        let old_status = *health;

        if old_status != new_status {
            *health = new_status;

            let _ = self.event_bus.publish(MonitorEvent::SystemHealthChanged {
                old_status,
                new_status,
                reason,
                timestamp: Utc::now(),
            });

            tracing::warn!(
                old_status = ?old_status,
                new_status = ?new_status,
                reason = %reason,
                "System health changed"
            );
        }
    }

    /// Update system metrics
    async fn update_metrics(&self) {
        let mut metrics = self.metrics.write().await;

        metrics.timestamp = Utc::now();
        metrics.active_scans = self.active_scans.len();

        let mut completed = 0;
        let mut failed = 0;
        let mut total_duration = 0u64;
        let mut total_issues = 0;
        let mut issues_by_severity = std::collections::HashMap::new();

        for entry in self.results.iter() {
            let result = entry.value();
            match result.status {
                ScanStatus::Completed => {
                    completed += 1;
                    if let Some(duration) = result.duration_ms {
                        total_duration += duration;
                    }
                }
                ScanStatus::Failed => failed += 1,
                _ => {}
            }

            total_issues += result.issues_found;
            for (severity, count) in &result.issues_by_severity {
                *issues_by_severity.entry(severity.clone()).or_insert(0) += count;
            }
        }

        metrics.completed_scans = completed;
        metrics.failed_scans = failed;
        metrics.total_issues = total_issues;
        metrics.issues_by_severity = issues_by_severity;
        metrics.average_scan_duration_ms = if completed > 0 {
            total_duration as f64 / completed as f64
        } else {
            0.0
        };
        metrics.system_health = *self.health.read().await;

        // Publish metrics updated event
        let _ = self.event_bus.publish(MonitorEvent::MetricsUpdated {
            metrics: metrics.clone(),
            timestamp: Utc::now(),
        });
    }

    fn validate_config(&self, config: &ScanConfig) -> Result<(), MonitorError> {
        if config.targets.is_empty() {
            return Err(MonitorError::InvalidConfig(
                "No scan targets specified".to_string(),
            ));
        }

        if config.timeout_seconds == 0 {
            return Err(MonitorError::InvalidConfig(
                "Timeout must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

/// Errors that can occur during monitoring
#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    #[error("Scan not found: {0}")]
    ScanNotFound(Uuid),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Scan already running: {0}")]
    ScanAlreadyRunning(Uuid),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_engine() {
        let event_bus = EventBus::new(100);
        let engine = MonitorEngine::new(event_bus);

        let config = ScanConfig {
            id: Uuid::new_v4(),
            name: "Test Scan".to_string(),
            scan_type: ScanType::Full,
            targets: vec!["https://example.com".to_string()],
            rules: vec!["wcag2aa".to_string()],
            schedule: None,
            timeout_seconds: 300,
            retry_count: 3,
            metadata: std::collections::HashMap::new(),
        };

        let scan_id = engine.start_scan(config).await.unwrap();
        assert_eq!(engine.active_scans.len(), 1);

        let result = engine.complete_scan(scan_id).await.unwrap();
        assert_eq!(result.status, ScanStatus::Completed);
        assert_eq!(engine.active_scans.len(), 0);
    }
}
