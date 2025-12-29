use crate::events::{EventBus, EventHandler, MonitorEvent};
use crate::types::*;
use chrono::Utc;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Alert manager for generating and routing alerts
pub struct AlertManager {
    event_bus: EventBus,
    configs: Arc<DashMap<Uuid, AlertConfig>>,
    alerts: Arc<DashMap<Uuid, Alert>>,
    throttle_state: Arc<DashMap<Uuid, chrono::DateTime<Utc>>>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            event_bus,
            configs: Arc::new(DashMap::new()),
            alerts: Arc::new(DashMap::new()),
            throttle_state: Arc::new(DashMap::new()),
        }
    }

    /// Add an alert configuration
    pub fn add_config(&self, config: AlertConfig) -> Uuid {
        let id = config.id;
        self.configs.insert(id, config);
        tracing::info!(config_id = %id, "Alert configuration added");
        id
    }

    /// Remove an alert configuration
    pub fn remove_config(&self, id: Uuid) -> Option<AlertConfig> {
        self.configs.remove(&id).map(|(_, c)| c)
    }

    /// Get an alert configuration
    pub fn get_config(&self, id: Uuid) -> Option<AlertConfig> {
        self.configs.get(&id).map(|c| c.value().clone())
    }

    /// Get all alert configurations
    pub fn get_configs(&self) -> Vec<AlertConfig> {
        self.configs
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Enable an alert configuration
    pub fn enable_config(&self, id: Uuid) -> Result<(), AlertError> {
        self.configs
            .get_mut(&id)
            .map(|mut c| c.enabled = true)
            .ok_or(AlertError::ConfigNotFound(id))
    }

    /// Disable an alert configuration
    pub fn disable_config(&self, id: Uuid) -> Result<(), AlertError> {
        self.configs
            .get_mut(&id)
            .map(|mut c| c.enabled = false)
            .ok_or(AlertError::ConfigNotFound(id))
    }

    /// Get an alert
    pub fn get_alert(&self, id: Uuid) -> Option<Alert> {
        self.alerts.get(&id).map(|a| a.value().clone())
    }

    /// Get all alerts
    pub fn get_alerts(&self) -> Vec<Alert> {
        let mut alerts: Vec<_> = self
            .alerts
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        alerts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        alerts
    }

    /// Get recent alerts (last N)
    pub fn get_recent_alerts(&self, limit: usize) -> Vec<Alert> {
        let mut alerts = self.get_alerts();
        alerts.truncate(limit);
        alerts
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(
        &self,
        id: Uuid,
        acknowledged_by: String,
    ) -> Result<(), AlertError> {
        let mut alert = self
            .alerts
            .get_mut(&id)
            .ok_or(AlertError::AlertNotFound(id))?;

        alert.acknowledged = true;
        alert.acknowledged_at = Some(Utc::now());
        alert.acknowledged_by = Some(acknowledged_by.clone());

        // Publish acknowledgement event
        let _ = self.event_bus.publish(MonitorEvent::AlertAcknowledged {
            alert_id: id,
            acknowledged_by,
            timestamp: Utc::now(),
        });

        tracing::info!(alert_id = %id, "Alert acknowledged");
        Ok(())
    }

    /// Process a scan result and generate alerts if conditions are met
    pub async fn process_scan_result(
        &self,
        result: &ScanResult,
        issues: &[AccessibilityIssue],
    ) -> Vec<Alert> {
        let mut generated_alerts = Vec::new();

        for entry in self.configs.iter() {
            let config = entry.value();

            if !config.enabled {
                continue;
            }

            // Check throttling
            if let Some(throttle_minutes) = config.throttle_minutes {
                if let Some(last_alert_time) = self.throttle_state.get(&config.id) {
                    let elapsed = Utc::now() - *last_alert_time;
                    if elapsed.num_minutes() < throttle_minutes as i64 {
                        tracing::debug!(
                            config_id = %config.id,
                            "Alert throttled"
                        );
                        continue;
                    }
                }
            }

            // Check conditions
            if self.check_conditions(config, result, issues) {
                if let Some(alert) = self.generate_alert(config, result, issues).await {
                    generated_alerts.push(alert);
                }
            }
        }

        generated_alerts
    }

    /// Check if alert conditions are met
    fn check_conditions(
        &self,
        config: &AlertConfig,
        result: &ScanResult,
        issues: &[AccessibilityIssue],
    ) -> bool {
        let conditions = &config.conditions;

        // Check scan failure
        if result.status == ScanStatus::Failed {
            if let Some(threshold) = conditions.failure_threshold {
                if threshold > 0 {
                    return true;
                }
            }
        }

        // Filter issues by severity
        let relevant_issues: Vec<_> = issues
            .iter()
            .filter(|issue| issue.severity.priority() >= conditions.min_severity.priority())
            .collect();

        // Check issue threshold
        if let Some(threshold) = conditions.issue_threshold {
            if relevant_issues.len() >= threshold {
                return true;
            }
        }

        false
    }

    /// Generate an alert
    async fn generate_alert(
        &self,
        config: &AlertConfig,
        result: &ScanResult,
        issues: &[AccessibilityIssue],
    ) -> Option<Alert> {
        let filtered_issues: Vec<_> = issues
            .iter()
            .filter(|issue| {
                issue.severity.priority() >= config.conditions.min_severity.priority()
            })
            .cloned()
            .collect();

        if filtered_issues.is_empty() && result.status != ScanStatus::Failed {
            return None;
        }

        let (title, message, severity) = if result.status == ScanStatus::Failed {
            (
                "Accessibility Scan Failed".to_string(),
                format!(
                    "Scan failed with error: {}",
                    result.error.as_ref().unwrap_or(&"Unknown error".to_string())
                ),
                Severity::High,
            )
        } else {
            let severity = filtered_issues
                .iter()
                .map(|i| i.severity)
                .max_by_key(|s| s.priority())
                .unwrap_or(Severity::Info);

            let title = format!(
                "{} Accessibility Issues Detected",
                filtered_issues.len()
            );

            let message = format!(
                "Found {} accessibility issues across {} pages.\n\nBreakdown:\n{}",
                filtered_issues.len(),
                result.pages_scanned,
                self.format_issue_breakdown(&filtered_issues)
            );

            (title, message, severity)
        };

        let alert = Alert {
            id: Uuid::new_v4(),
            config_id: config.id,
            severity,
            title,
            message,
            scan_id: Some(result.scan_id),
            issues: filtered_issues,
            created_at: Utc::now(),
            acknowledged: false,
            acknowledged_at: None,
            acknowledged_by: None,
        };

        // Store alert
        self.alerts.insert(alert.id, alert.clone());

        // Update throttle state
        self.throttle_state.insert(config.id, Utc::now());

        // Publish alert event
        let _ = self.event_bus.publish(MonitorEvent::AlertTriggered {
            alert: alert.clone(),
            timestamp: Utc::now(),
        });

        // Send alert through configured channels
        for channel in &config.channels {
            if let Err(e) = self.send_alert(&alert, channel).await {
                tracing::error!(
                    alert_id = %alert.id,
                    channel = ?channel,
                    error = ?e,
                    "Failed to send alert"
                );
            }
        }

        tracing::info!(
            alert_id = %alert.id,
            config_id = %config.id,
            severity = ?severity,
            "Alert generated"
        );

        Some(alert)
    }

    /// Format issue breakdown for alert message
    fn format_issue_breakdown(&self, issues: &[AccessibilityIssue]) -> String {
        let mut breakdown = HashMap::new();
        for issue in issues {
            *breakdown
                .entry(format!("{:?}", issue.severity))
                .or_insert(0) += 1;
        }

        breakdown
            .iter()
            .map(|(severity, count)| format!("- {}: {}", severity, count))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Send alert through a channel
    async fn send_alert(&self, alert: &Alert, channel: &AlertChannel) -> Result<(), AlertError> {
        match channel {
            AlertChannel::Email { recipients } => {
                tracing::info!(
                    alert_id = %alert.id,
                    recipients = ?recipients,
                    "Would send email alert (not implemented)"
                );
                // TODO: Implement email sending
                Ok(())
            }
            AlertChannel::Webhook { url, headers } => {
                let client = reqwest::Client::new();
                let mut request = client.post(url).json(alert);

                for (key, value) in headers {
                    request = request.header(key, value);
                }

                request
                    .send()
                    .await
                    .map_err(|e| AlertError::SendFailed(e.to_string()))?;

                tracing::info!(
                    alert_id = %alert.id,
                    url = %url,
                    "Webhook alert sent"
                );
                Ok(())
            }
            AlertChannel::Slack {
                webhook_url,
                channel,
            } => {
                let client = reqwest::Client::new();
                let payload = serde_json::json!({
                    "channel": channel,
                    "text": format!("*{}*\n{}", alert.title, alert.message),
                    "username": "Accessibility Monitor",
                });

                client
                    .post(webhook_url)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| AlertError::SendFailed(e.to_string()))?;

                tracing::info!(
                    alert_id = %alert.id,
                    channel = %channel,
                    "Slack alert sent"
                );
                Ok(())
            }
            AlertChannel::PagerDuty { integration_key } => {
                tracing::info!(
                    alert_id = %alert.id,
                    "Would send PagerDuty alert (not implemented)"
                );
                // TODO: Implement PagerDuty integration
                Ok(())
            }
        }
    }
}

/// Alert errors
#[derive(Debug, thiserror::Error)]
pub enum AlertError {
    #[error("Alert configuration not found: {0}")]
    ConfigNotFound(Uuid),

    #[error("Alert not found: {0}")]
    AlertNotFound(Uuid),

    #[error("Failed to send alert: {0}")]
    SendFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_alert_manager() {
        let event_bus = EventBus::new(100);
        let manager = AlertManager::new(event_bus);

        let config = AlertConfig {
            id: Uuid::new_v4(),
            name: "Test Alert".to_string(),
            enabled: true,
            conditions: AlertConditions {
                min_severity: Severity::Medium,
                issue_threshold: Some(5),
                failure_threshold: Some(1),
                scan_types: None,
            },
            channels: vec![],
            throttle_minutes: None,
        };

        let id = manager.add_config(config);
        assert!(manager.get_config(id).is_some());
    }
}
