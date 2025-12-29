//! Accessibility Core - Error handling and types for Enterprise Web-Accessibility SaaS Platform
//!
//! This crate provides comprehensive error handling, validation, and type definitions
//! for building accessible web applications that comply with WCAG 2.1 standards.
//!
//! # Features
//!
//! - **Error Handling**: Structured error types for all platform operations
//! - **WCAG Standards**: Complete WCAG 2.1 success criteria definitions
//! - **Severity Levels**: Categorization of accessibility issues by severity
//! - **Type Safety**: Strongly-typed representations of accessibility concepts
//! - **Serialization**: Full serde support for all types
//!
//! # Examples
//!
//! ## Creating and handling errors
//!
//! ```rust
//! use accessibility_core::{
//!     error::{AccessibilityError, ValidationError},
//!     types::{ErrorCategory, ErrorCode},
//! };
//!
//! fn validate_url(url: &str) -> Result<(), AccessibilityError> {
//!     if url.is_empty() {
//!         return Err(AccessibilityError::new(
//!             "URL cannot be empty",
//!             ErrorCode::ValidationInvalidUrl,
//!             ErrorCategory::Validation,
//!         ));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Working with WCAG criteria
//!
//! ```rust
//! use accessibility_core::wcag::{WCAGLevel, get_criteria_for_level};
//!
//! let aa_criteria = get_criteria_for_level(WCAGLevel::AA);
//! println!("Found {} Level AA criteria", aa_criteria.len());
//! ```
//!
//! ## Using severity levels
//!
//! ```rust
//! use accessibility_core::severity::SeverityLevel;
//!
//! let critical = SeverityLevel::Critical;
//! let moderate = SeverityLevel::Moderate;
//!
//! assert!(critical > moderate);
//! assert!(critical.is_at_least(SeverityLevel::Serious));
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod error;
pub mod severity;
pub mod types;
pub mod wcag;

// Re-export commonly used items
pub use error::{
    AccessibilityError, AuthError, NetworkError, PermissionError, Result, ScanError,
    ValidationError, ValidationErrorDetail,
};
pub use severity::{SeverityLevel, SeverityColors};
pub use types::{
    AccessibilityViolation, ApiError, ApiMetadata, ApiResponse, ErrorCategory, ErrorCode,
    ErrorMetadata, RetryConfig, ScanConfig, ScanResult,
};
pub use wcag::{WCAGCriterion, WCAGLevel, WCAGPrinciple};

/// Prelude module for convenient imports
pub mod prelude {
    //! Commonly used items for quick access
    //!
    //! # Example
    //!
    //! ```rust
    //! use accessibility_core::prelude::*;
    //! ```

    pub use crate::error::{
        AccessibilityError, AuthError, NetworkError, PermissionError, Result, ScanError,
        ValidationError,
    };
    pub use crate::severity::SeverityLevel;
    pub use crate::types::{ErrorCategory, ErrorCode, ErrorMetadata};
    pub use crate::wcag::{WCAGCriterion, WCAGLevel, WCAGPrinciple};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = AccessibilityError::new(
            "Test error",
            ErrorCode::InternalError,
            ErrorCategory::System,
        );

        assert_eq!(error.message, "Test error");
        assert_eq!(error.code, ErrorCode::InternalError);
        assert_eq!(error.category, ErrorCategory::System);
    }

    #[test]
    fn test_severity_levels() {
        let critical = SeverityLevel::Critical;
        let info = SeverityLevel::Info;

        assert!(critical > info);
        assert_eq!(critical.priority(), 5);
        assert_eq!(info.priority(), 1);
    }

    #[test]
    fn test_wcag_levels() {
        use crate::wcag::{get_criteria_for_level, WCAGLevel};

        let level_a = get_criteria_for_level(WCAGLevel::A);
        let level_aa = get_criteria_for_level(WCAGLevel::AA);

        assert!(!level_a.is_empty());
        assert!(level_aa.len() >= level_a.len());
    }
}
