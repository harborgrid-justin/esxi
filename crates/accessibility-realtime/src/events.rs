use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// Event types emitted by the monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MonitorEvent {
    /// Scan lifecycle events
    ScanStarted {
        scan_id: Uuid,
        config: ScanConfig,
        timestamp: DateTime<Utc>,
    },
    ScanProgress {
        scan_id: Uuid,
        pages_scanned: usize,
        total_pages: usize,
        issues_found: usize,
        percentage: f64,
        timestamp: DateTime<Utc>,
    },
    ScanCompleted {
        scan_id: Uuid,
        result: ScanResult,
        timestamp: DateTime<Utc>,
    },
    ScanFailed {
        scan_id: Uuid,
        error: String,
        timestamp: DateTime<Utc>,
    },

    /// Issue events
    IssueDetected {
        issue: AccessibilityIssue,
        timestamp: DateTime<Utc>,
    },
    IssueResolved {
        issue_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    /// Alert events
    AlertTriggered {
        alert: Alert,
        timestamp: DateTime<Utc>,
    },
    AlertAcknowledged {
        alert_id: Uuid,
        acknowledged_by: String,
        timestamp: DateTime<Utc>,
    },

    /// System events
    SystemHealthChanged {
        old_status: HealthStatus,
        new_status: HealthStatus,
        reason: String,
        timestamp: DateTime<Utc>,
    },
    MetricsUpdated {
        metrics: MonitorMetrics,
        timestamp: DateTime<Utc>,
    },

    /// Client control events
    ClientConnected {
        client_id: String,
        timestamp: DateTime<Utc>,
    },
    ClientDisconnected {
        client_id: String,
        timestamp: DateTime<Utc>,
    },
}

impl MonitorEvent {
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            MonitorEvent::ScanStarted { timestamp, .. }
            | MonitorEvent::ScanProgress { timestamp, .. }
            | MonitorEvent::ScanCompleted { timestamp, .. }
            | MonitorEvent::ScanFailed { timestamp, .. }
            | MonitorEvent::IssueDetected { timestamp, .. }
            | MonitorEvent::IssueResolved { timestamp, .. }
            | MonitorEvent::AlertTriggered { timestamp, .. }
            | MonitorEvent::AlertAcknowledged { timestamp, .. }
            | MonitorEvent::SystemHealthChanged { timestamp, .. }
            | MonitorEvent::MetricsUpdated { timestamp, .. }
            | MonitorEvent::ClientConnected { timestamp, .. }
            | MonitorEvent::ClientDisconnected { timestamp, .. } => *timestamp,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            MonitorEvent::ScanStarted { .. } => "scan_started",
            MonitorEvent::ScanProgress { .. } => "scan_progress",
            MonitorEvent::ScanCompleted { .. } => "scan_completed",
            MonitorEvent::ScanFailed { .. } => "scan_failed",
            MonitorEvent::IssueDetected { .. } => "issue_detected",
            MonitorEvent::IssueResolved { .. } => "issue_resolved",
            MonitorEvent::AlertTriggered { .. } => "alert_triggered",
            MonitorEvent::AlertAcknowledged { .. } => "alert_acknowledged",
            MonitorEvent::SystemHealthChanged { .. } => "system_health_changed",
            MonitorEvent::MetricsUpdated { .. } => "metrics_updated",
            MonitorEvent::ClientConnected { .. } => "client_connected",
            MonitorEvent::ClientDisconnected { .. } => "client_disconnected",
        }
    }
}

/// Event bus for distributing monitoring events
#[derive(Clone)]
pub struct EventBus {
    sender: Arc<broadcast::Sender<MonitorEvent>>,
}

impl EventBus {
    /// Create a new event bus with specified capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: MonitorEvent) -> Result<usize, EventError> {
        self.sender
            .send(event)
            .map_err(|e| EventError::PublishFailed(e.to_string()))
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<MonitorEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Event handler trait for processing events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &MonitorEvent) -> Result<(), EventError>;
    fn name(&self) -> &str;
}

/// Event processor that runs handlers
pub struct EventProcessor {
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl EventProcessor {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn add_handler(&mut self, handler: Arc<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    pub async fn process(&self, event: &MonitorEvent) -> Vec<Result<(), EventError>> {
        let mut results = Vec::new();
        for handler in &self.handlers {
            let result = handler.handle(event).await;
            if let Err(ref e) = result {
                tracing::error!(
                    handler = handler.name(),
                    error = ?e,
                    "Event handler failed"
                );
            }
            results.push(result);
        }
        results
    }
}

impl Default for EventProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors related to event handling
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Failed to publish event: {0}")]
    PublishFailed(String),

    #[error("Failed to process event: {0}")]
    ProcessingFailed(String),

    #[error("Handler error: {0}")]
    HandlerError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus() {
        let bus = EventBus::new(10);
        let mut rx = bus.subscribe();

        let event = MonitorEvent::ClientConnected {
            client_id: "test-client".to_string(),
            timestamp: Utc::now(),
        };

        bus.publish(event.clone()).unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.event_type(), "client_connected");
    }
}
