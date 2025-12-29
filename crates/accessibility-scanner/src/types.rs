use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Severity level of an accessibility issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Serious,
    Moderate,
    Minor,
    Info,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "critical",
            Severity::Serious => "serious",
            Severity::Moderate => "moderate",
            Severity::Minor => "minor",
            Severity::Info => "info",
        }
    }

    pub fn score(&self) -> u32 {
        match self {
            Severity::Critical => 100,
            Severity::Serious => 75,
            Severity::Moderate => 50,
            Severity::Minor => 25,
            Severity::Info => 10,
        }
    }
}

/// WCAG 2.1 conformance level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WCAGLevel {
    A,
    AA,
    AAA,
}

impl WCAGLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            WCAGLevel::A => "A",
            WCAGLevel::AA => "AA",
            WCAGLevel::AAA => "AAA",
        }
    }
}

/// WCAG 2.1 principle categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Principle {
    Perceivable,
    Operable,
    Understandable,
    Robust,
}

impl Principle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Principle::Perceivable => "Perceivable",
            Principle::Operable => "Operable",
            Principle::Understandable => "Understandable",
            Principle::Robust => "Robust",
        }
    }
}

/// A WCAG rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub level: WCAGLevel,
    pub principle: Principle,
    pub guideline: String,
    pub success_criterion: String,
    pub tags: Vec<String>,
}

/// Position of an issue in the HTML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub xpath: String,
    pub selector: String,
}

/// Context information about where an issue occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueContext {
    pub html: String,
    pub outer_html: String,
    pub position: Position,
    pub attributes: HashMap<String, String>,
    pub computed_styles: Option<HashMap<String, String>>,
}

/// An accessibility issue found during scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: Uuid,
    pub rule_id: String,
    pub rule_name: String,
    pub severity: Severity,
    pub level: WCAGLevel,
    pub principle: Principle,
    pub message: String,
    pub help: String,
    pub help_url: String,
    pub context: IssueContext,
    pub fix_suggestions: Vec<String>,
    pub impact_description: String,
    pub wcag_reference: String,
}

impl Issue {
    pub fn new(
        rule: &Rule,
        severity: Severity,
        message: String,
        help: String,
        context: IssueContext,
        fix_suggestions: Vec<String>,
        impact_description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            severity,
            level: rule.level,
            principle: rule.principle,
            message,
            help,
            help_url: format!("https://www.w3.org/WAI/WCAG21/Understanding/{}", rule.success_criterion),
            context,
            fix_suggestions,
            impact_description,
            wcag_reference: format!("WCAG 2.1 {} - {}", rule.level.as_str(), rule.success_criterion),
        }
    }
}

/// Statistics about a scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStatistics {
    pub total_issues: usize,
    pub critical: usize,
    pub serious: usize,
    pub moderate: usize,
    pub minor: usize,
    pub info: usize,
    pub pages_scanned: usize,
    pub elements_analyzed: usize,
    pub duration_ms: u64,
    pub compliance_score: f64,
}

impl ScanStatistics {
    pub fn new() -> Self {
        Self {
            total_issues: 0,
            critical: 0,
            serious: 0,
            moderate: 0,
            minor: 0,
            info: 0,
            pages_scanned: 0,
            elements_analyzed: 0,
            duration_ms: 0,
            compliance_score: 100.0,
        }
    }

    pub fn add_issue(&mut self, severity: Severity) {
        self.total_issues += 1;
        match severity {
            Severity::Critical => self.critical += 1,
            Severity::Serious => self.serious += 1,
            Severity::Moderate => self.moderate += 1,
            Severity::Minor => self.minor += 1,
            Severity::Info => self.info += 1,
        }
    }

    pub fn calculate_compliance_score(&mut self) {
        let penalty = (self.critical * 100 + self.serious * 75 + self.moderate * 50 + self.minor * 25 + self.info * 10) as f64;
        let max_score = 100.0;
        self.compliance_score = (max_score - (penalty / (self.elements_analyzed.max(1) as f64))).max(0.0).min(100.0);
    }
}

impl Default for ScanStatistics {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for a scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub target_url: String,
    pub levels: Vec<WCAGLevel>,
    pub max_pages: Option<usize>,
    pub max_depth: Option<usize>,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub timeout_seconds: u64,
    pub follow_external_links: bool,
    pub check_images: bool,
    pub check_videos: bool,
    pub check_pdfs: bool,
    pub parallel_threads: usize,
    pub incremental: bool,
    pub cache_enabled: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            target_url: String::new(),
            levels: vec![WCAGLevel::A, WCAGLevel::AA],
            max_pages: Some(100),
            max_depth: Some(3),
            include_patterns: vec![],
            exclude_patterns: vec![],
            timeout_seconds: 30,
            follow_external_links: false,
            check_images: true,
            check_videos: true,
            check_pdfs: false,
            parallel_threads: num_cpus::get(),
            incremental: true,
            cache_enabled: true,
        }
    }
}

/// Result of a single page scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageResult {
    pub url: String,
    pub title: String,
    pub issues: Vec<Issue>,
    pub elements_count: usize,
    pub scan_time_ms: u64,
    pub http_status: u16,
    pub content_type: String,
}

/// Complete scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub id: Uuid,
    pub config: ScanConfig,
    pub pages: Vec<PageResult>,
    pub statistics: ScanStatistics,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub version: String,
}

impl ScanResult {
    pub fn new(config: ScanConfig) -> Self {
        Self {
            id: Uuid::new_v4(),
            config,
            pages: Vec::new(),
            statistics: ScanStatistics::new(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn add_page(&mut self, page: PageResult) {
        for issue in &page.issues {
            self.statistics.add_issue(issue.severity);
        }
        self.statistics.elements_analyzed += page.elements_count;
        self.statistics.pages_scanned += 1;
        self.pages.push(page);
    }

    pub fn finalize(&mut self) {
        self.completed_at = Utc::now();
        let duration = self.completed_at - self.started_at;
        self.statistics.duration_ms = duration.num_milliseconds() as u64;
        self.statistics.calculate_compliance_score();
    }
}

/// Error types for the scanner
#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("HTML parsing failed: {0}")]
    ParseError(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Timeout exceeded: {0}")]
    Timeout(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Rule execution failed: {0}")]
    RuleError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, ScannerError>;
