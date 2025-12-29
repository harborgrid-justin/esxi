//! Cryptographic audit logging.
//!
//! This module provides comprehensive audit logging for all cryptographic operations.

use crate::error::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Audit event type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuditEventType {
    /// Key generation event.
    KeyGeneration,

    /// Key rotation event.
    KeyRotation,

    /// Key deletion event.
    KeyDeletion,

    /// Encryption operation.
    Encryption,

    /// Decryption operation.
    Decryption,

    /// Signing operation.
    Signing,

    /// Signature verification.
    SignatureVerification,

    /// Certificate generation.
    CertificateGeneration,

    /// Certificate validation.
    CertificateValidation,

    /// KMS operation.
    KmsOperation,

    /// HSM operation.
    HsmOperation,

    /// Key derivation.
    KeyDerivation,

    /// Access control event.
    AccessControl,

    /// Configuration change.
    ConfigurationChange,

    /// Security violation.
    SecurityViolation,

    /// System error.
    SystemError,
}

/// Audit event severity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuditSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Audit event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier.
    pub event_id: String,

    /// Event type.
    pub event_type: AuditEventType,

    /// Event severity.
    pub severity: AuditSeverity,

    /// Timestamp when the event occurred.
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// User or service that triggered the event.
    pub principal: String,

    /// Resource being accessed (key ID, certificate ID, etc.).
    pub resource: Option<String>,

    /// Operation performed.
    pub operation: String,

    /// Operation result (success/failure).
    pub result: AuditResult,

    /// Error message (if operation failed).
    pub error_message: Option<String>,

    /// Source IP address.
    pub source_ip: Option<String>,

    /// Additional metadata.
    pub metadata: HashMap<String, String>,

    /// Session identifier.
    pub session_id: Option<String>,
}

/// Audit operation result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditResult {
    Success,
    Failure,
    PartialSuccess,
}

impl AuditEvent {
    /// Create a new audit event.
    pub fn new(
        event_type: AuditEventType,
        severity: AuditSeverity,
        principal: String,
        operation: String,
        result: AuditResult,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type,
            severity,
            timestamp: chrono::Utc::now(),
            principal,
            resource: None,
            operation,
            result,
            error_message: None,
            source_ip: None,
            metadata: HashMap::new(),
            session_id: None,
        }
    }

    /// Set the resource.
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Set the error message.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error_message = Some(error.into());
        self
    }

    /// Set the source IP address.
    pub fn with_source_ip(mut self, ip: impl Into<String>) -> Self {
        self.source_ip = Some(ip.into());
        self
    }

    /// Set the session ID.
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Audit log storage backend trait.
#[async_trait::async_trait]
pub trait AuditStorage: Send + Sync {
    /// Store an audit event.
    async fn store(&self, event: &AuditEvent) -> CryptoResult<()>;

    /// Query audit events.
    async fn query(&self, filter: &AuditFilter) -> CryptoResult<Vec<AuditEvent>>;

    /// Get event by ID.
    async fn get_event(&self, event_id: &str) -> CryptoResult<Option<AuditEvent>>;

    /// Delete old events (retention policy).
    async fn delete_old_events(&self, before: chrono::DateTime<chrono::Utc>) -> CryptoResult<usize>;
}

/// Audit event filter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditFilter {
    /// Filter by event type.
    pub event_types: Option<Vec<AuditEventType>>,

    /// Filter by severity.
    pub min_severity: Option<AuditSeverity>,

    /// Filter by principal.
    pub principal: Option<String>,

    /// Filter by resource.
    pub resource: Option<String>,

    /// Filter by result.
    pub result: Option<AuditResult>,

    /// Filter by time range.
    pub from_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub to_timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Limit number of results.
    pub limit: Option<usize>,
}

/// In-memory audit storage (for testing and small deployments).
pub struct InMemoryAuditStorage {
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

impl InMemoryAuditStorage {
    /// Create a new in-memory audit storage.
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for InMemoryAuditStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AuditStorage for InMemoryAuditStorage {
    async fn store(&self, event: &AuditEvent) -> CryptoResult<()> {
        let mut events = self.events.write().await;
        events.push(event.clone());
        Ok(())
    }

    async fn query(&self, filter: &AuditFilter) -> CryptoResult<Vec<AuditEvent>> {
        let events = self.events.read().await;
        let mut filtered: Vec<_> = events
            .iter()
            .filter(|e| {
                // Filter by event types
                if let Some(types) = &filter.event_types {
                    if !types.contains(&e.event_type) {
                        return false;
                    }
                }

                // Filter by severity
                if let Some(min_sev) = &filter.min_severity {
                    if &e.severity < min_sev {
                        return false;
                    }
                }

                // Filter by principal
                if let Some(principal) = &filter.principal {
                    if &e.principal != principal {
                        return false;
                    }
                }

                // Filter by resource
                if let Some(resource) = &filter.resource {
                    if e.resource.as_ref() != Some(resource) {
                        return false;
                    }
                }

                // Filter by result
                if let Some(result) = &filter.result {
                    if &e.result != result {
                        return false;
                    }
                }

                // Filter by timestamp range
                if let Some(from) = &filter.from_timestamp {
                    if e.timestamp < *from {
                        return false;
                    }
                }

                if let Some(to) = &filter.to_timestamp {
                    if e.timestamp > *to {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Apply limit
        if let Some(limit) = filter.limit {
            filtered.truncate(limit);
        }

        Ok(filtered)
    }

    async fn get_event(&self, event_id: &str) -> CryptoResult<Option<AuditEvent>> {
        let events = self.events.read().await;
        Ok(events.iter().find(|e| e.event_id == event_id).cloned())
    }

    async fn delete_old_events(&self, before: chrono::DateTime<chrono::Utc>) -> CryptoResult<usize> {
        let mut events = self.events.write().await;
        let initial_len = events.len();
        events.retain(|e| e.timestamp >= before);
        Ok(initial_len - events.len())
    }
}

/// Audit logger.
pub struct AuditLogger {
    storage: Arc<dyn AuditStorage>,
    default_principal: String,
}

impl AuditLogger {
    /// Create a new audit logger.
    pub fn new(storage: Arc<dyn AuditStorage>, default_principal: String) -> Self {
        Self {
            storage,
            default_principal,
        }
    }

    /// Log an audit event.
    pub async fn log(&self, event: AuditEvent) -> CryptoResult<String> {
        let event_id = event.event_id.clone();

        // Also log to tracing for real-time monitoring
        match event.severity {
            AuditSeverity::Debug => tracing::debug!(
                event_type = ?event.event_type,
                principal = %event.principal,
                operation = %event.operation,
                result = ?event.result,
                "Audit event"
            ),
            AuditSeverity::Info => tracing::info!(
                event_type = ?event.event_type,
                principal = %event.principal,
                operation = %event.operation,
                result = ?event.result,
                "Audit event"
            ),
            AuditSeverity::Warning => tracing::warn!(
                event_type = ?event.event_type,
                principal = %event.principal,
                operation = %event.operation,
                result = ?event.result,
                "Audit event"
            ),
            AuditSeverity::Error => tracing::error!(
                event_type = ?event.event_type,
                principal = %event.principal,
                operation = %event.operation,
                result = ?event.result,
                error = ?event.error_message,
                "Audit event"
            ),
            AuditSeverity::Critical => tracing::error!(
                event_type = ?event.event_type,
                principal = %event.principal,
                operation = %event.operation,
                result = ?event.result,
                error = ?event.error_message,
                "CRITICAL audit event"
            ),
        }

        self.storage.store(&event).await?;
        Ok(event_id)
    }

    /// Log a successful operation.
    pub async fn log_success(
        &self,
        event_type: AuditEventType,
        operation: &str,
        resource: Option<&str>,
    ) -> CryptoResult<String> {
        let mut event = AuditEvent::new(
            event_type,
            AuditSeverity::Info,
            self.default_principal.clone(),
            operation.to_string(),
            AuditResult::Success,
        );

        if let Some(res) = resource {
            event = event.with_resource(res);
        }

        self.log(event).await
    }

    /// Log a failed operation.
    pub async fn log_failure(
        &self,
        event_type: AuditEventType,
        operation: &str,
        resource: Option<&str>,
        error: &str,
    ) -> CryptoResult<String> {
        let mut event = AuditEvent::new(
            event_type,
            AuditSeverity::Error,
            self.default_principal.clone(),
            operation.to_string(),
            AuditResult::Failure,
        );

        if let Some(res) = resource {
            event = event.with_resource(res);
        }

        event = event.with_error(error);

        self.log(event).await
    }

    /// Log a security violation.
    pub async fn log_security_violation(
        &self,
        operation: &str,
        reason: &str,
    ) -> CryptoResult<String> {
        let event = AuditEvent::new(
            AuditEventType::SecurityViolation,
            AuditSeverity::Critical,
            self.default_principal.clone(),
            operation.to_string(),
            AuditResult::Failure,
        )
        .with_error(reason);

        self.log(event).await
    }

    /// Query audit events.
    pub async fn query(&self, filter: &AuditFilter) -> CryptoResult<Vec<AuditEvent>> {
        self.storage.query(filter).await
    }

    /// Get a specific event.
    pub async fn get_event(&self, event_id: &str) -> CryptoResult<Option<AuditEvent>> {
        self.storage.get_event(event_id).await
    }

    /// Clean up old events based on retention policy.
    pub async fn cleanup_old_events(&self, retention_days: u32) -> CryptoResult<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(retention_days as i64);
        self.storage.delete_old_events(cutoff).await
    }

    /// Generate audit report.
    pub async fn generate_report(
        &self,
        filter: &AuditFilter,
    ) -> CryptoResult<AuditReport> {
        let events = self.query(filter).await?;

        let mut report = AuditReport {
            total_events: events.len(),
            successful_operations: 0,
            failed_operations: 0,
            events_by_type: HashMap::new(),
            events_by_severity: HashMap::new(),
            unique_principals: std::collections::HashSet::new(),
            generated_at: chrono::Utc::now(),
        };

        for event in &events {
            if event.result == AuditResult::Success {
                report.successful_operations += 1;
            } else {
                report.failed_operations += 1;
            }

            *report.events_by_type.entry(event.event_type.clone()).or_insert(0) += 1;
            *report.events_by_severity.entry(event.severity.clone()).or_insert(0) += 1;
            report.unique_principals.insert(event.principal.clone());
        }

        Ok(report)
    }
}

/// Audit report.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditReport {
    pub total_events: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub events_by_type: HashMap<AuditEventType, usize>,
    pub events_by_severity: HashMap<AuditSeverity, usize>,
    pub unique_principals: std::collections::HashSet<String>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logging() {
        let storage = Arc::new(InMemoryAuditStorage::new());
        let logger = AuditLogger::new(storage, "test-system".to_string());

        let event_id = logger
            .log_success(
                AuditEventType::Encryption,
                "encrypt_data",
                Some("key-123"),
            )
            .await
            .unwrap();

        assert!(!event_id.is_empty());
    }

    #[tokio::test]
    async fn test_audit_query() {
        let storage = Arc::new(InMemoryAuditStorage::new());
        let logger = AuditLogger::new(storage, "test-system".to_string());

        // Log some events
        logger
            .log_success(AuditEventType::Encryption, "encrypt1", None)
            .await
            .unwrap();
        logger
            .log_failure(AuditEventType::Decryption, "decrypt1", None, "Invalid key")
            .await
            .unwrap();

        // Query all events
        let all_events = logger.query(&AuditFilter::default()).await.unwrap();
        assert_eq!(all_events.len(), 2);

        // Query only failures
        let mut filter = AuditFilter::default();
        filter.result = Some(AuditResult::Failure);
        let failed_events = logger.query(&filter).await.unwrap();
        assert_eq!(failed_events.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_cleanup() {
        let storage = Arc::new(InMemoryAuditStorage::new());
        let logger = AuditLogger::new(storage, "test-system".to_string());

        logger
            .log_success(AuditEventType::Encryption, "encrypt1", None)
            .await
            .unwrap();

        // This should delete events older than 0 days (all events)
        let deleted = logger.cleanup_old_events(0).await.unwrap();
        assert_eq!(deleted, 0); // Events just created won't be deleted
    }

    #[tokio::test]
    async fn test_audit_report() {
        let storage = Arc::new(InMemoryAuditStorage::new());
        let logger = AuditLogger::new(storage, "test-system".to_string());

        logger
            .log_success(AuditEventType::Encryption, "op1", None)
            .await
            .unwrap();
        logger
            .log_failure(AuditEventType::Decryption, "op2", None, "error")
            .await
            .unwrap();

        let report = logger.generate_report(&AuditFilter::default()).await.unwrap();
        assert_eq!(report.total_events, 2);
        assert_eq!(report.successful_operations, 1);
        assert_eq!(report.failed_operations, 1);
    }
}
