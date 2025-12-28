//! Error types for the Meridian Governance system

use thiserror::Error;

/// Result type for governance operations
pub type Result<T> = std::result::Result<T, GovernanceError>;

/// Main error type for the governance system
#[derive(Error, Debug)]
pub enum GovernanceError {
    /// Catalog-related errors
    #[error("Catalog error: {0}")]
    Catalog(String),

    /// Dataset not found
    #[error("Dataset not found: {0}")]
    DatasetNotFound(String),

    /// Lineage tracking errors
    #[error("Lineage error: {0}")]
    Lineage(String),

    /// Circular dependency detected in lineage
    #[error("Circular dependency detected in lineage graph")]
    CircularDependency,

    /// Data quality validation errors
    #[error("Data quality validation failed: {0}")]
    QualityValidation(String),

    /// Quality rule errors
    #[error("Quality rule error: {0}")]
    QualityRule(String),

    /// Classification errors
    #[error("Classification error: {0}")]
    Classification(String),

    /// Invalid sensitivity level
    #[error("Invalid sensitivity level: {0}")]
    InvalidSensitivityLevel(String),

    /// Compliance framework errors
    #[error("Compliance error: {0}")]
    Compliance(String),

    /// GDPR compliance violation
    #[error("GDPR compliance violation: {0}")]
    GdprViolation(String),

    /// CCPA compliance violation
    #[error("CCPA compliance violation: {0}")]
    CcpaViolation(String),

    /// SOC2 compliance violation
    #[error("SOC2 compliance violation: {0}")]
    Soc2Violation(String),

    /// Retention policy errors
    #[error("Retention policy error: {0}")]
    RetentionPolicy(String),

    /// Data expired according to retention policy
    #[error("Data expired: {0}")]
    DataExpired(String),

    /// Audit trail errors
    #[error("Audit error: {0}")]
    Audit(String),

    /// Access denied
    #[error("Access denied: {0}")]
    AccessDenied(String),

    /// Data masking errors
    #[error("Masking error: {0}")]
    Masking(String),

    /// Invalid masking configuration
    #[error("Invalid masking configuration: {0}")]
    InvalidMaskingConfig(String),

    /// Schema registry errors
    #[error("Schema registry error: {0}")]
    Schema(String),

    /// Schema not found
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),

    /// Schema version conflict
    #[error("Schema version conflict: {0}")]
    SchemaVersionConflict(String),

    /// Schema evolution error
    #[error("Schema evolution error: {0}")]
    SchemaEvolution(String),

    /// Incompatible schema change
    #[error("Incompatible schema change: {0}")]
    IncompatibleSchemaChange(String),

    /// Impact analysis errors
    #[error("Impact analysis error: {0}")]
    ImpactAnalysis(String),

    /// Stewardship workflow errors
    #[error("Stewardship error: {0}")]
    Stewardship(String),

    /// Invalid workflow state transition
    #[error("Invalid workflow state transition: from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    /// Glossary errors
    #[error("Glossary error: {0}")]
    Glossary(String),

    /// Business term not found
    #[error("Business term not found: {0}")]
    TermNotFound(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// YAML serialization errors
    #[error("YAML serialization error: {0}")]
    YamlSerialization(#[from] serde_yaml::Error),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Invalid UUID
    #[error("Invalid UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),

    /// Regex errors
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}

impl GovernanceError {
    /// Create a catalog error
    pub fn catalog<S: Into<String>>(msg: S) -> Self {
        Self::Catalog(msg.into())
    }

    /// Create a lineage error
    pub fn lineage<S: Into<String>>(msg: S) -> Self {
        Self::Lineage(msg.into())
    }

    /// Create a quality validation error
    pub fn quality_validation<S: Into<String>>(msg: S) -> Self {
        Self::QualityValidation(msg.into())
    }

    /// Create a compliance error
    pub fn compliance<S: Into<String>>(msg: S) -> Self {
        Self::Compliance(msg.into())
    }

    /// Create an access denied error
    pub fn access_denied<S: Into<String>>(msg: S) -> Self {
        Self::AccessDenied(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}
