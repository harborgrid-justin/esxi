use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Severity level for accessibility issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn priority(&self) -> u8 {
        match self {
            Severity::Critical => 5,
            Severity::High => 4,
            Severity::Medium => 3,
            Severity::Low => 2,
            Severity::Info => 1,
        }
    }
}

/// Status of a monitoring scan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScanStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Type of accessibility scan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanType {
    Full,
    Incremental,
    Targeted,
    Scheduled,
    OnDemand,
}

/// Health status of the monitoring system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Scan configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub id: Uuid,
    pub name: String,
    pub scan_type: ScanType,
    pub targets: Vec<String>,
    pub rules: Vec<String>,
    pub schedule: Option<String>, // Cron expression
    pub timeout_seconds: u64,
    pub retry_count: u32,
    pub metadata: HashMap<String, String>,
}

/// Scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub status: ScanStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub issues_found: usize,
    pub issues_by_severity: HashMap<String, usize>,
    pub pages_scanned: usize,
    pub error: Option<String>,
}

/// Accessibility issue detected during scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityIssue {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub severity: Severity,
    pub rule_id: String,
    pub rule_name: String,
    pub description: String,
    pub element: String,
    pub selector: String,
    pub page_url: String,
    pub context: String,
    pub wcag_criteria: Vec<String>,
    pub remediation: String,
    pub detected_at: DateTime<Utc>,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub conditions: AlertConditions,
    pub channels: Vec<AlertChannel>,
    pub throttle_minutes: Option<u32>,
}

/// Alert conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConditions {
    pub min_severity: Severity,
    pub issue_threshold: Option<usize>,
    pub failure_threshold: Option<u32>,
    pub scan_types: Option<Vec<ScanType>>,
}

/// Alert delivery channel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AlertChannel {
    Email { recipients: Vec<String> },
    Webhook { url: String, headers: HashMap<String, String> },
    Slack { webhook_url: String, channel: String },
    PagerDuty { integration_key: String },
}

/// Generated alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub config_id: Uuid,
    pub severity: Severity,
    pub title: String,
    pub message: String,
    pub scan_id: Option<Uuid>,
    pub issues: Vec<AccessibilityIssue>,
    pub created_at: DateTime<Utc>,
    pub acknowledged: bool,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<String>,
}

/// Monitoring metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorMetrics {
    pub timestamp: DateTime<Utc>,
    pub active_scans: usize,
    pub completed_scans: usize,
    pub failed_scans: usize,
    pub total_issues: usize,
    pub issues_by_severity: HashMap<String, usize>,
    pub average_scan_duration_ms: f64,
    pub system_health: HealthStatus,
}

/// Schedule for recurring scans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanSchedule {
    pub id: Uuid,
    pub name: String,
    pub cron: String,
    pub config: ScanConfig,
    pub enabled: bool,
    pub next_run: Option<DateTime<Utc>>,
    pub last_run: Option<DateTime<Utc>>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Default Scan".to_string(),
            scan_type: ScanType::Full,
            targets: vec![],
            rules: vec![],
            schedule: None,
            timeout_seconds: 300,
            retry_count: 3,
            metadata: HashMap::new(),
        }
    }
}
