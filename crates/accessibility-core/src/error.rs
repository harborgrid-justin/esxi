//! Error types for accessibility platform

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

use crate::severity::SeverityLevel;
use crate::types::{ErrorCategory, ErrorCode, ErrorMetadata};

/// Result type alias for accessibility operations
pub type Result<T> = std::result::Result<T, AccessibilityError>;

/// Base error type for all accessibility platform errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct AccessibilityError {
    /// Error message
    pub message: String,

    /// Error code
    pub code: ErrorCode,

    /// Error category
    pub category: ErrorCategory,

    /// HTTP status code
    pub status_code: u16,

    /// Whether error is retryable
    pub retryable: bool,

    /// Error metadata
    pub metadata: ErrorMetadata,

    /// Original error cause (if any)
    #[serde(skip)]
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl AccessibilityError {
    /// Create a new AccessibilityError
    pub fn new(
        message: impl Into<String>,
        code: ErrorCode,
        category: ErrorCategory,
    ) -> Self {
        Self {
            message: message.into(),
            code,
            category,
            status_code: 500,
            retryable: false,
            metadata: ErrorMetadata::new(),
            cause: None,
        }
    }

    /// Set HTTP status code
    pub fn with_status_code(mut self, status_code: u16) -> Self {
        self.status_code = status_code;
        self
    }

    /// Set retryable flag
    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    /// Set error cause
    pub fn with_cause<E: std::error::Error + Send + Sync + 'static>(
        mut self,
        cause: E,
    ) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Add context to metadata
    pub fn with_context(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.metadata.add_context(key, value);
        self
    }

    /// Set user ID in metadata
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.metadata.user_id = Some(user_id.into());
        self
    }

    /// Set session ID in metadata
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.metadata.session_id = Some(session_id.into());
        self
    }

    /// Set request ID in metadata
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.metadata.request_id = Some(request_id.into());
        self
    }

    /// Check if error is of a specific category
    pub fn is_category(&self, category: ErrorCategory) -> bool {
        self.category == category
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        self.retryable
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for AccessibilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.category, self.code, self.message
        )
    }
}

/// Validation error
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message
    pub message: String,

    /// Validation error details
    pub details: Vec<ValidationErrorDetail>,
}

impl ValidationError {
    /// Create a new ValidationError
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            details: Vec::new(),
        }
    }

    /// Create validation error for a single field
    pub fn for_field(field: impl Into<String>, message: impl Into<String>) -> Self {
        let field = field.into();
        let message_str = message.into();
        Self {
            message: format!("Validation failed for field \"{}\": {}", field, message_str),
            details: vec![ValidationErrorDetail {
                field,
                message: message_str,
                rule: "unknown".to_string(),
                value: None,
                expected: None,
            }],
        }
    }

    /// Add a validation error detail
    pub fn add_detail(mut self, detail: ValidationErrorDetail) -> Self {
        self.details.push(detail);
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Validation error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorDetail {
    /// Field that failed validation
    pub field: String,

    /// Validation error message
    pub message: String,

    /// Validation rule that failed
    pub rule: String,

    /// Current value that failed validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,

    /// Expected value or format
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
}

/// Scan error
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub struct ScanError {
    /// Error message
    pub message: String,

    /// Error code
    pub code: ErrorCode,

    /// URL being scanned
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Scan ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_id: Option<String>,

    /// Whether error is retryable
    pub retryable: bool,
}

impl ScanError {
    /// Create a new ScanError
    pub fn new(message: impl Into<String>, code: ErrorCode) -> Self {
        Self {
            message: message.into(),
            code,
            url: None,
            scan_id: None,
            retryable: false,
        }
    }

    /// Create scan timeout error
    pub fn timeout(url: impl Into<String>, timeout_ms: u64) -> Self {
        Self {
            message: format!("Scan timed out after {}ms", timeout_ms),
            code: ErrorCode::ScanTimeout,
            url: Some(url.into()),
            scan_id: None,
            retryable: true,
        }
    }

    /// Create page load failed error
    pub fn page_load_failed(url: impl Into<String>) -> Self {
        Self {
            message: format!("Failed to load page: {}", url.into()),
            code: ErrorCode::PageLoadFailed,
            url: Some(url.into()),
            scan_id: None,
            retryable: true,
        }
    }
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Network error
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub struct NetworkError {
    /// Error message
    pub message: String,

    /// Error code
    pub code: ErrorCode,

    /// Request URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// HTTP method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// HTTP status code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,

    /// Whether error is retryable
    pub retryable: bool,
}

impl NetworkError {
    /// Create a new NetworkError
    pub fn new(message: impl Into<String>, code: ErrorCode) -> Self {
        Self {
            message: message.into(),
            code,
            url: None,
            method: None,
            status_code: None,
            retryable: true,
        }
    }

    /// Create request timeout error
    pub fn timeout(url: impl Into<String>, timeout_ms: u64) -> Self {
        Self {
            message: format!("Request timed out after {}ms", timeout_ms),
            code: ErrorCode::RequestTimeout,
            url: Some(url.into()),
            method: None,
            status_code: None,
            retryable: true,
        }
    }

    /// Create rate limit error
    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        let message = if let Some(seconds) = retry_after {
            format!("Rate limited. Retry after {} seconds", seconds)
        } else {
            "Rate limited. Please try again later".to_string()
        };

        Self {
            message,
            code: ErrorCode::RateLimited,
            url: None,
            method: None,
            status_code: Some(429),
            retryable: true,
        }
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Authentication error
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub struct AuthError {
    /// Error message
    pub message: String,

    /// Error code
    pub code: ErrorCode,

    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Session ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl AuthError {
    /// Create a new AuthError
    pub fn new(message: impl Into<String>, code: ErrorCode) -> Self {
        Self {
            message: message.into(),
            code,
            user_id: None,
            session_id: None,
        }
    }

    /// Create unauthorized error
    pub fn unauthorized() -> Self {
        Self::new(
            "You are not authorized to access this resource",
            ErrorCode::Unauthorized,
        )
    }

    /// Create token expired error
    pub fn token_expired() -> Self {
        Self::new(
            "Your session has expired. Please log in again",
            ErrorCode::TokenExpired,
        )
    }

    /// Create invalid credentials error
    pub fn invalid_credentials() -> Self {
        Self::new("Invalid username or password", ErrorCode::InvalidCredentials)
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Permission error
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub struct PermissionError {
    /// Error message
    pub message: String,

    /// Error code
    pub code: ErrorCode,

    /// Required permission
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_permission: Option<String>,

    /// Resource being accessed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,

    /// Action being attempted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

impl PermissionError {
    /// Create a new PermissionError
    pub fn new(message: impl Into<String>, code: ErrorCode) -> Self {
        Self {
            message: message.into(),
            code,
            required_permission: None,
            resource: None,
            action: None,
        }
    }

    /// Create forbidden error
    pub fn forbidden(resource: Option<String>, action: Option<String>) -> Self {
        let message = match (&resource, &action) {
            (Some(res), Some(act)) => format!("You do not have permission to {} {}", act, res),
            (Some(res), None) => format!("You do not have permission to access {}", res),
            _ => "You do not have permission to access this resource".to_string(),
        };

        Self {
            message,
            code: ErrorCode::Forbidden,
            required_permission: None,
            resource,
            action,
        }
    }

    /// Create insufficient permissions error
    pub fn insufficient_permissions(
        required: impl Into<String>,
        resource: Option<String>,
    ) -> Self {
        let required_str = required.into();
        let message = if let Some(res) = &resource {
            format!(
                "Insufficient permissions to access {}. Required: {}",
                res, required_str
            )
        } else {
            format!("Insufficient permissions. Required: {}", required_str)
        };

        Self {
            message,
            code: ErrorCode::InsufficientPermissions,
            required_permission: Some(required_str),
            resource,
            action: None,
        }
    }

    /// Create quota exceeded error
    pub fn quota_exceeded(quota_name: impl Into<String>, limit: u64, current: u64) -> Self {
        Self::new(
            format!(
                "You have exceeded your {} quota ({}/{})",
                quota_name.into(),
                current,
                limit
            ),
            ErrorCode::QuotaExceeded,
        )
    }
}

impl fmt::Display for PermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_error_creation() {
        let error = AccessibilityError::new("Test error", ErrorCode::InternalError, ErrorCategory::System)
            .with_status_code(500)
            .with_retryable(true);

        assert_eq!(error.message, "Test error");
        assert_eq!(error.status_code, 500);
        assert!(error.retryable);
    }

    #[test]
    fn test_validation_error() {
        let error = ValidationError::for_field("email", "Invalid email format");
        assert_eq!(error.details.len(), 1);
        assert_eq!(error.details[0].field, "email");
    }

    #[test]
    fn test_auth_error() {
        let error = AuthError::unauthorized();
        assert_eq!(error.code, ErrorCode::Unauthorized);
    }
}
