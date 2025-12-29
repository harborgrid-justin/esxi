//! Shared types for accessibility modules

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::severity::SeverityLevel;
use crate::wcag::{WCAGCriterion, WCAGLevel, WCAGPrinciple};

/// Error category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// Validation errors
    Validation,
    /// Scan errors
    Scan,
    /// Network errors
    Network,
    /// Authentication errors
    Auth,
    /// Permission errors
    Permission,
    /// Accessibility errors
    Accessibility,
    /// System errors
    System,
    /// Unknown errors
    Unknown,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Validation => write!(f, "VALIDATION"),
            ErrorCategory::Scan => write!(f, "SCAN"),
            ErrorCategory::Network => write!(f, "NETWORK"),
            ErrorCategory::Auth => write!(f, "AUTH"),
            ErrorCategory::Permission => write!(f, "PERMISSION"),
            ErrorCategory::Accessibility => write!(f, "ACCESSIBILITY"),
            ErrorCategory::System => write!(f, "SYSTEM"),
            ErrorCategory::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Error code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // Validation errors
    ValidationInvalidInput,
    ValidationMissingRequired,
    ValidationInvalidFormat,
    ValidationInvalidType,
    ValidationOutOfRange,
    ValidationInvalidUrl,
    ValidationInvalidEmail,
    ValidationInvalidWcagLevel,
    ValidationSchemaFailed,

    // Scan errors
    ScanFailed,
    ScanTimeout,
    ScanCancelled,
    PageLoadFailed,
    InvalidPage,
    BrowserError,
    NavigationError,
    ScriptExecutionError,
    ScreenshotFailed,
    TooManyViolations,
    ScanQueueFull,

    // Network errors
    NetworkError,
    RequestTimeout,
    ConnectionRefused,
    DnsError,
    SslError,
    RateLimited,
    ServiceUnavailable,
    BadGateway,
    GatewayTimeout,
    CorsError,
    AbortError,

    // Authentication errors
    Unauthorized,
    InvalidCredentials,
    TokenExpired,
    TokenInvalid,
    TokenMissing,
    SessionExpired,
    SessionInvalid,
    MfaRequired,
    MfaFailed,
    AccountLocked,
    AccountDisabled,

    // Permission errors
    Forbidden,
    InsufficientPermissions,
    ResourceNotFound,
    OperationNotAllowed,
    QuotaExceeded,
    PlanLimitReached,
    FeatureNotAvailable,

    // Accessibility errors
    WcagViolation,
    AriaError,
    KeyboardNavigation,
    ColorContrast,
    MissingAltText,
    InvalidHeadingOrder,
    MissingLabel,
    InvalidRole,
    MissingLandmark,
    FocusOrder,

    // System errors
    InternalError,
    NotImplemented,
    ConfigurationError,
    DatabaseError,
    FileSystemError,
    MemoryError,
    DependencyError,
    InitializationError,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Error metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetadata {
    /// Unique error identifier
    pub error_id: String,

    /// Timestamp when error occurred
    pub timestamp: DateTime<Utc>,

    /// User ID if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Session ID if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Request ID for API calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// URL or route where error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// User agent string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// Additional custom context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
}

impl ErrorMetadata {
    /// Create new error metadata
    pub fn new() -> Self {
        Self {
            error_id: format!("err_{}_{}", Utc::now().timestamp(), Uuid::new_v4()),
            timestamp: Utc::now(),
            user_id: None,
            session_id: None,
            request_id: None,
            url: None,
            user_agent: None,
            context: None,
        }
    }

    /// Add context value
    pub fn add_context(&mut self, key: impl Into<String>, value: impl Serialize) {
        if self.context.is_none() {
            self.context = Some(HashMap::new());
        }

        if let Some(context) = &mut self.context {
            if let Ok(json_value) = serde_json::to_value(value) {
                context.insert(key.into(), json_value);
            }
        }
    }
}

impl Default for ErrorMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Accessibility violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityViolation {
    /// Unique identifier for the violation
    pub id: String,

    /// WCAG criterion that was violated
    pub criterion: WCAGCriterion,

    /// Severity of the violation
    pub severity: SeverityLevel,

    /// Description of the violation
    pub message: String,

    /// CSS selector or XPath to the problematic element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,

    /// HTML snippet of the problematic element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,

    /// Suggested fix for the violation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,

    /// Additional context about the violation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, serde_json::Value>>,
}

/// Scan configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// URL or URLs to scan
    pub urls: Vec<String>,

    /// WCAG level to test against
    pub level: WCAGLevel,

    /// Rules to include in scan
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_rules: Option<Vec<String>>,

    /// Rules to exclude from scan
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_rules: Option<Vec<String>>,

    /// Maximum number of pages to scan
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pages: Option<u32>,

    /// Timeout for each page scan (ms)
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Whether to capture screenshots
    #[serde(default)]
    pub screenshots: bool,
}

fn default_timeout() -> u64 {
    30000
}

/// Scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Unique scan identifier
    pub scan_id: String,

    /// URL that was scanned
    pub url: String,

    /// Timestamp of the scan
    pub timestamp: DateTime<Utc>,

    /// WCAG level used for scan
    pub level: WCAGLevel,

    /// List of violations found
    pub violations: Vec<AccessibilityViolation>,

    /// Number of elements tested
    pub elements_tested: u64,

    /// Scan duration in milliseconds
    pub duration: u64,

    /// Whether scan completed successfully
    pub success: bool,

    /// Error message if scan failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Error if request failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,

    /// Response metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ApiMetadata>,
}

/// API error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error message
    pub message: String,

    /// Error code
    pub code: String,

    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// API metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMetadata {
    /// Request ID
    pub request_id: String,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// API version
    pub version: String,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Initial delay before first retry (ms)
    #[serde(default = "default_initial_delay")]
    pub initial_delay: u64,

    /// Maximum delay between retries (ms)
    #[serde(default = "default_max_delay")]
    pub max_delay: u64,

    /// Backoff multiplier
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,

    /// Whether to use exponential backoff
    #[serde(default = "default_exponential_backoff")]
    pub exponential_backoff: bool,
}

fn default_max_retries() -> u32 {
    3
}

fn default_initial_delay() -> u64 {
    1000
}

fn default_max_delay() -> u64 {
    30000
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

fn default_exponential_backoff() -> bool {
    true
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            initial_delay: default_initial_delay(),
            max_delay: default_max_delay(),
            backoff_multiplier: default_backoff_multiplier(),
            exponential_backoff: default_exponential_backoff(),
        }
    }
}
