//! Security audit logging
//!
//! Comprehensive audit logging for security events and compliance.
//!
//! ## Purpose
//! - Compliance (SOC 2, ISO 27001, GDPR, HIPAA)
//! - Security incident investigation
//! - Threat detection and monitoring
//! - Forensic analysis
//!
//! ## What to Audit
//! - Authentication attempts (success/failure)
//! - Authorization decisions
//! - Data access (read/write/delete)
//! - Configuration changes
//! - Key operations (rotation, generation)
//! - Policy changes
//! - Security events (anomalies, violations)
//!
//! ## OWASP Logging Best Practices
//! - Log security-relevant events
//! - Include timestamp, user, action, result
//! - Never log sensitive data (passwords, keys, PII)
//! - Tamper-evident logs
//! - Centralized log storage
//! - Log retention policy

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{SecurityError, SecurityResult};

/// Audit event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuditSeverity {
    /// Informational event
    Info,
    /// Warning - potential issue
    Warning,
    /// Error - operation failed
    Error,
    /// Critical - security incident
    Critical,
}

/// Audit event categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditCategory {
    /// Authentication event
    Authentication,
    /// Authorization event
    Authorization,
    /// Data access event
    DataAccess,
    /// Configuration change
    Configuration,
    /// Key management operation
    KeyManagement,
    /// Policy change
    Policy,
    /// Security event
    Security,
    /// User management
    UserManagement,
    /// System event
    System,
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event ID (unique)
    pub event_id: String,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Event category
    pub category: AuditCategory,

    /// Event severity
    pub severity: AuditSeverity,

    /// Event action/operation
    pub action: String,

    /// User ID (if applicable)
    pub user_id: Option<String>,

    /// Organization ID
    pub organization_id: Option<String>,

    /// Resource affected
    pub resource: Option<String>,

    /// IP address
    pub ip_address: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Result (success/failure)
    pub result: AuditResult,

    /// Error message (if failed)
    pub error_message: Option<String>,

    /// Session ID
    pub session_id: Option<String>,

    /// Additional metadata
    #[serde(flatten)]
    pub metadata: serde_json::Value,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(category: AuditCategory, action: &str, result: AuditResult) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            category,
            severity: AuditSeverity::Info,
            action: action.to_string(),
            user_id: None,
            organization_id: None,
            resource: None,
            ip_address: None,
            user_agent: None,
            result,
            error_message: None,
            session_id: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Set severity
    pub fn with_severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set organization ID
    pub fn with_organization_id(mut self, org_id: String) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    /// Set resource
    pub fn with_resource(mut self, resource: String) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Set IP address
    pub fn with_ip_address(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set error message
    pub fn with_error(mut self, error: String) -> Self {
        self.error_message = Some(error);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Audit result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditResult {
    /// Operation succeeded
    Success,
    /// Operation failed
    Failure,
    /// Operation denied by policy
    Denied,
}

/// Audit logger trait
///
/// Implement this trait to create custom audit log destinations
pub trait AuditLogger: Send + Sync {
    /// Log an audit event
    fn log(&self, event: &AuditEvent) -> SecurityResult<()>;

    /// Flush any pending events
    fn flush(&self) -> SecurityResult<()> {
        Ok(())
    }
}

/// In-memory audit logger (for testing/development)
#[derive(Debug)]
pub struct MemoryAuditLogger {
    events: std::sync::Arc<std::sync::Mutex<Vec<AuditEvent>>>,
}

impl MemoryAuditLogger {
    /// Create a new in-memory logger
    pub fn new() -> Self {
        Self {
            events: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Get all logged events
    pub fn get_events(&self) -> SecurityResult<Vec<AuditEvent>> {
        let events = self
            .events
            .lock()
            .map_err(|_| SecurityError::AuditError("Failed to acquire lock".to_string()))?;
        Ok(events.clone())
    }

    /// Clear all events
    pub fn clear(&self) -> SecurityResult<()> {
        let mut events = self
            .events
            .lock()
            .map_err(|_| SecurityError::AuditError("Failed to acquire lock".to_string()))?;
        events.clear();
        Ok(())
    }

    /// Get event count
    pub fn count(&self) -> SecurityResult<usize> {
        let events = self
            .events
            .lock()
            .map_err(|_| SecurityError::AuditError("Failed to acquire lock".to_string()))?;
        Ok(events.len())
    }
}

impl Default for MemoryAuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLogger for MemoryAuditLogger {
    fn log(&self, event: &AuditEvent) -> SecurityResult<()> {
        let mut events = self
            .events
            .lock()
            .map_err(|_| SecurityError::AuditError("Failed to acquire lock".to_string()))?;
        events.push(event.clone());
        Ok(())
    }
}

/// JSON file audit logger
pub struct JsonFileAuditLogger {
    file_path: String,
}

impl JsonFileAuditLogger {
    /// Create a new JSON file logger
    pub fn new(file_path: &str) -> Self {
        Self {
            file_path: file_path.to_string(),
        }
    }
}

impl AuditLogger for JsonFileAuditLogger {
    fn log(&self, event: &AuditEvent) -> SecurityResult<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|e| SecurityError::AuditError(format!("Failed to open audit log: {}", e)))?;

        let json = serde_json::to_string(event)
            .map_err(|e| SecurityError::AuditError(format!("Failed to serialize event: {}", e)))?;

        writeln!(file, "{}", json)
            .map_err(|e| SecurityError::AuditError(format!("Failed to write event: {}", e)))?;

        Ok(())
    }

    fn flush(&self) -> SecurityResult<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.file_path)
            .map_err(|e| SecurityError::AuditError(format!("Failed to open audit log: {}", e)))?;

        file.flush()
            .map_err(|e| SecurityError::AuditError(format!("Failed to flush: {}", e)))?;

        Ok(())
    }
}

/// Multi-target audit logger (log to multiple destinations)
pub struct MultiAuditLogger {
    loggers: Vec<Box<dyn AuditLogger>>,
}

impl MultiAuditLogger {
    /// Create a new multi-target logger
    pub fn new() -> Self {
        Self {
            loggers: Vec::new(),
        }
    }

    /// Add a logger
    pub fn add_logger(&mut self, logger: Box<dyn AuditLogger>) {
        self.loggers.push(logger);
    }
}

impl Default for MultiAuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl AuditLogger for MultiAuditLogger {
    fn log(&self, event: &AuditEvent) -> SecurityResult<()> {
        for logger in &self.loggers {
            logger.log(event)?;
        }
        Ok(())
    }

    fn flush(&self) -> SecurityResult<()> {
        for logger in &self.loggers {
            logger.flush()?;
        }
        Ok(())
    }
}

/// Audit service for managing audit logging
pub struct AuditService {
    logger: Box<dyn AuditLogger>,
}

impl AuditService {
    /// Create a new audit service
    pub fn new(logger: Box<dyn AuditLogger>) -> Self {
        Self { logger }
    }

    /// Log an audit event
    pub fn log(&self, event: AuditEvent) -> SecurityResult<()> {
        self.logger.log(&event)
    }

    /// Log authentication attempt
    pub fn log_authentication(
        &self,
        user_id: &str,
        success: bool,
        ip_address: Option<String>,
    ) -> SecurityResult<()> {
        let result = if success {
            AuditResult::Success
        } else {
            AuditResult::Failure
        };

        let mut event = AuditEvent::new(AuditCategory::Authentication, "login", result)
            .with_user_id(user_id.to_string())
            .with_severity(if success {
                AuditSeverity::Info
            } else {
                AuditSeverity::Warning
            });

        if let Some(ip) = ip_address {
            event = event.with_ip_address(ip);
        }

        self.log(event)
    }

    /// Log authorization decision
    pub fn log_authorization(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
        allowed: bool,
    ) -> SecurityResult<()> {
        let result = if allowed {
            AuditResult::Success
        } else {
            AuditResult::Denied
        };

        let event = AuditEvent::new(AuditCategory::Authorization, action, result)
            .with_user_id(user_id.to_string())
            .with_resource(resource.to_string());

        self.log(event)
    }

    /// Log data access
    pub fn log_data_access(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
        success: bool,
    ) -> SecurityResult<()> {
        let result = if success {
            AuditResult::Success
        } else {
            AuditResult::Failure
        };

        let event = AuditEvent::new(AuditCategory::DataAccess, action, result)
            .with_user_id(user_id.to_string())
            .with_resource(resource.to_string());

        self.log(event)
    }

    /// Log security event
    pub fn log_security_event(
        &self,
        event_type: &str,
        severity: AuditSeverity,
        description: &str,
    ) -> SecurityResult<()> {
        let event = AuditEvent::new(AuditCategory::Security, event_type, AuditResult::Success)
            .with_severity(severity)
            .with_metadata(serde_json::json!({
                "description": description
            }));

        self.log(event)
    }

    /// Log key management operation
    pub fn log_key_operation(&self, operation: &str, key_id: &str, success: bool) -> SecurityResult<()> {
        let result = if success {
            AuditResult::Success
        } else {
            AuditResult::Failure
        };

        let event = AuditEvent::new(AuditCategory::KeyManagement, operation, result)
            .with_resource(key_id.to_string())
            .with_severity(AuditSeverity::Info);

        self.log(event)
    }

    /// Flush pending events
    pub fn flush(&self) -> SecurityResult<()> {
        self.logger.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(AuditCategory::Authentication, "login", AuditResult::Success)
            .with_user_id("user123".to_string())
            .with_ip_address("192.168.1.1".to_string())
            .with_severity(AuditSeverity::Info);

        assert_eq!(event.category, AuditCategory::Authentication);
        assert_eq!(event.action, "login");
        assert_eq!(event.result, AuditResult::Success);
        assert_eq!(event.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_memory_logger() {
        let logger = MemoryAuditLogger::new();

        let event = AuditEvent::new(AuditCategory::DataAccess, "read", AuditResult::Success);

        logger.log(&event).unwrap();

        let events = logger.get_events().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, "read");
    }

    #[test]
    fn test_audit_service() {
        let logger = MemoryAuditLogger::new();
        let service = AuditService::new(Box::new(logger));

        service
            .log_authentication("user123", true, Some("192.168.1.1".to_string()))
            .unwrap();

        service
            .log_authorization("user123", "/api/data", "read", true)
            .unwrap();
    }

    #[test]
    fn test_multi_logger() {
        let mut multi = MultiAuditLogger::new();
        let logger1 = MemoryAuditLogger::new();
        let logger2 = MemoryAuditLogger::new();

        multi.add_logger(Box::new(logger1));
        multi.add_logger(Box::new(logger2));

        let event = AuditEvent::new(AuditCategory::Security, "anomaly", AuditResult::Success);

        multi.log(&event).unwrap();
    }

    #[test]
    fn test_audit_severity_ordering() {
        assert!(AuditSeverity::Critical > AuditSeverity::Error);
        assert!(AuditSeverity::Error > AuditSeverity::Warning);
        assert!(AuditSeverity::Warning > AuditSeverity::Info);
    }

    #[test]
    fn test_security_event_logging() {
        let logger = MemoryAuditLogger::new();
        let service = AuditService::new(Box::new(logger));

        service
            .log_security_event("anomaly_detected", AuditSeverity::Critical, "Unusual access pattern")
            .unwrap();
    }

    #[test]
    fn test_key_operation_logging() {
        let logger = MemoryAuditLogger::new();
        let service = AuditService::new(Box::new(logger));

        service
            .log_key_operation("rotation", "key123", true)
            .unwrap();
    }
}
