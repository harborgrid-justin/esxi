//! Error types for the multi-tenant system.

use thiserror::Error;
use std::fmt;

/// Result type for tenant operations.
pub type TenantResult<T> = Result<T, TenantError>;

/// Comprehensive error types for multi-tenant operations.
#[derive(Debug, Error)]
pub enum TenantError {
    /// Tenant not found
    #[error("Tenant not found: {0}")]
    TenantNotFound(String),

    /// Tenant already exists
    #[error("Tenant already exists: {0}")]
    TenantAlreadyExists(String),

    /// Tenant is suspended
    #[error("Tenant is suspended: {0}")]
    TenantSuspended(String),

    /// Tenant is deleted
    #[error("Tenant is deleted: {0}")]
    TenantDeleted(String),

    /// Invalid tenant identifier
    #[error("Invalid tenant identifier: {0}")]
    InvalidTenantId(String),

    /// Tenant isolation violation
    #[error("Tenant isolation violation: {0}")]
    IsolationViolation(String),

    /// Resource quota exceeded
    #[error("Resource quota exceeded for {resource}: {current}/{limit}")]
    QuotaExceeded {
        resource: String,
        current: u64,
        limit: u64,
    },

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Cross-tenant access denied
    #[error("Cross-tenant access denied: cannot access tenant {target} from tenant {source_tenant}")]
    CrossTenantAccessDenied {
        source_tenant: String,
        target: String,
    },

    /// Provisioning failed
    #[error("Tenant provisioning failed: {0}")]
    ProvisioningFailed(String),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Feature not available for tenant
    #[error("Feature '{0}' not available for this tenant")]
    FeatureNotAvailable(String),

    /// Billing error
    #[error("Billing error: {0}")]
    BillingError(String),

    /// Migration error
    #[error("Migration error: {0}")]
    MigrationError(String),

    /// Hierarchy error
    #[error("Hierarchy error: {0}")]
    HierarchyError(String),

    /// Invalid parent tenant
    #[error("Invalid parent tenant: {0}")]
    InvalidParent(String),

    /// Circular hierarchy detected
    #[error("Circular hierarchy detected: {0}")]
    CircularHierarchy(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// External service error
    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

#[cfg(feature = "postgres")]
impl From<sqlx::Error> for TenantError {
    fn from(err: sqlx::Error) -> Self {
        TenantError::DatabaseError(err.to_string())
    }
}

impl From<validator::ValidationErrors> for TenantError {
    fn from(err: validator::ValidationErrors) -> Self {
        TenantError::ValidationError(err.to_string())
    }
}

impl From<serde_json::Error> for TenantError {
    fn from(err: serde_json::Error) -> Self {
        TenantError::ConfigError(err.to_string())
    }
}

/// Error context for enriched error information.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub tenant_id: Option<String>,
    pub operation: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

impl ErrorContext {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            tenant_id: None,
            operation: operation.into(),
            timestamp: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}
