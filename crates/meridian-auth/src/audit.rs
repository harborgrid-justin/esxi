//! Audit logging for security and compliance

use crate::error::AuthResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Audit event severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    /// Informational event
    Info,
    /// Warning event
    Warning,
    /// Security-related event
    Security,
    /// Critical security event
    Critical,
}

/// Audit event type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    // Authentication events
    LoginSuccess,
    LoginFailure,
    Logout,
    TokenGenerated,
    TokenRevoked,
    TokenExpired,

    // User management events
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserLocked,
    UserUnlocked,
    UserDisabled,
    UserEnabled,
    PasswordChanged,
    PasswordResetRequested,
    PasswordResetCompleted,
    EmailVerified,

    // Authorization events
    AccessGranted,
    AccessDenied,
    PermissionChanged,
    RoleAssigned,
    RoleRevoked,

    // Session events
    SessionCreated,
    SessionExpired,
    SessionDestroyed,

    // Security events
    SuspiciousActivity,
    BruteForceAttempt,
    UnauthorizedAccess,
    PrivilegeEscalation,

    // Resource events
    ResourceAccessed,
    ResourceCreated,
    ResourceUpdated,
    ResourceDeleted,

    // Policy events
    PolicyEvaluated,
    PolicyViolation,

    // OAuth events
    OAuthAuthorization,
    OAuthTokenExchange,

    // Custom event
    Custom(String),
}

/// Audit event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID
    pub id: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: AuditEventType,
    /// Severity level
    pub severity: AuditSeverity,
    /// User ID (if applicable)
    pub user_id: Option<String>,
    /// Session ID (if applicable)
    pub session_id: Option<String>,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Resource type
    pub resource_type: Option<String>,
    /// Resource ID
    pub resource_id: Option<String>,
    /// Action performed
    pub action: Option<String>,
    /// Event result (success/failure)
    pub result: AuditResult,
    /// Detailed message
    pub message: String,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Audit event result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditResult {
    Success,
    Failure,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(event_type: AuditEventType, severity: AuditSeverity, message: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            severity,
            user_id: None,
            session_id: None,
            ip_address: None,
            user_agent: None,
            resource_type: None,
            resource_id: None,
            action: None,
            result: AuditResult::Success,
            message,
            metadata: HashMap::new(),
        }
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set IP address
    pub fn with_ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    /// Set resource information
    pub fn with_resource(
        mut self,
        resource_type: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Self {
        self.resource_type = Some(resource_type.into());
        self.resource_id = Some(resource_id.into());
        self
    }

    /// Set action
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Set result
    pub fn with_result(mut self, result: AuditResult) -> Self {
        self.result = result;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Audit query builder for searching audit logs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditQuery {
    /// Filter by user ID
    pub user_id: Option<String>,
    /// Filter by event type
    pub event_type: Option<AuditEventType>,
    /// Filter by severity
    pub severity: Option<AuditSeverity>,
    /// Filter by result
    pub result: Option<AuditResult>,
    /// Filter by resource type
    pub resource_type: Option<String>,
    /// Filter by resource ID
    pub resource_id: Option<String>,
    /// Start timestamp
    pub start_time: Option<DateTime<Utc>>,
    /// End timestamp
    pub end_time: Option<DateTime<Utc>>,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

impl AuditQuery {
    /// Create a new audit query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by user ID
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Filter by event type
    pub fn event_type(mut self, event_type: AuditEventType) -> Self {
        self.event_type = Some(event_type);
        self
    }

    /// Filter by severity
    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Filter by time range
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Set limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Check if an event matches this query
    pub fn matches(&self, event: &AuditEvent) -> bool {
        if let Some(ref user_id) = self.user_id {
            if event.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }

        if let Some(ref event_type) = self.event_type {
            if &event.event_type != event_type {
                return false;
            }
        }

        if let Some(severity) = self.severity {
            if event.severity != severity {
                return false;
            }
        }

        if let Some(result) = self.result {
            if event.result != result {
                return false;
            }
        }

        if let Some(ref resource_type) = self.resource_type {
            if event.resource_type.as_ref() != Some(resource_type) {
                return false;
            }
        }

        if let Some(ref resource_id) = self.resource_id {
            if event.resource_id.as_ref() != Some(resource_id) {
                return false;
            }
        }

        if let Some(start_time) = self.start_time {
            if event.timestamp < start_time {
                return false;
            }
        }

        if let Some(end_time) = self.end_time {
            if event.timestamp > end_time {
                return false;
            }
        }

        true
    }
}

/// Audit logger trait for different storage backends
#[async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log an audit event
    async fn log(&mut self, event: AuditEvent) -> AuthResult<()>;

    /// Query audit events
    async fn query(&self, query: AuditQuery) -> AuthResult<Vec<AuditEvent>>;

    /// Count events matching query
    async fn count(&self, query: AuditQuery) -> AuthResult<usize>;

    /// Delete old audit events
    async fn cleanup(&mut self, before: DateTime<Utc>) -> AuthResult<usize>;
}

/// In-memory audit logger (for testing and development)
#[derive(Debug, Clone, Default)]
pub struct InMemoryAuditLogger {
    events: Vec<AuditEvent>,
}

impl InMemoryAuditLogger {
    /// Create a new in-memory audit logger
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Get all events
    pub fn get_all_events(&self) -> &[AuditEvent] {
        &self.events
    }
}

#[async_trait]
impl AuditLogger for InMemoryAuditLogger {
    async fn log(&mut self, event: AuditEvent) -> AuthResult<()> {
        self.events.push(event);
        Ok(())
    }

    async fn query(&self, query: AuditQuery) -> AuthResult<Vec<AuditEvent>> {
        let mut results: Vec<AuditEvent> = self
            .events
            .iter()
            .filter(|e| query.matches(e))
            .cloned()
            .collect();

        // Apply offset
        if let Some(offset) = query.offset {
            if offset < results.len() {
                results = results.into_iter().skip(offset).collect();
            } else {
                results.clear();
            }
        }

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    async fn count(&self, query: AuditQuery) -> AuthResult<usize> {
        Ok(self.events.iter().filter(|e| query.matches(e)).count())
    }

    async fn cleanup(&mut self, before: DateTime<Utc>) -> AuthResult<usize> {
        let initial_count = self.events.len();
        self.events.retain(|e| e.timestamp >= before);
        Ok(initial_count - self.events.len())
    }
}

/// Audit manager for simplified audit logging
pub struct AuditManager<L: AuditLogger> {
    logger: L,
}

impl<L: AuditLogger> AuditManager<L> {
    /// Create a new audit manager
    pub fn new(logger: L) -> Self {
        Self { logger }
    }

    /// Log a login success event
    pub async fn log_login_success(
        &mut self,
        user_id: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> AuthResult<()> {
        let mut event = AuditEvent::new(
            AuditEventType::LoginSuccess,
            AuditSeverity::Info,
            format!("User {} logged in successfully", user_id),
        )
        .with_user_id(user_id);

        if let Some(ip) = ip {
            event = event.with_ip_address(ip);
        }
        if let Some(ua) = user_agent {
            event = event.with_user_agent(ua);
        }

        self.logger.log(event).await
    }

    /// Log a login failure event
    pub async fn log_login_failure(
        &mut self,
        user_id: Option<&str>,
        reason: &str,
        ip: Option<String>,
    ) -> AuthResult<()> {
        let mut event = AuditEvent::new(
            AuditEventType::LoginFailure,
            AuditSeverity::Security,
            format!("Login failed: {}", reason),
        )
        .with_result(AuditResult::Failure);

        if let Some(uid) = user_id {
            event = event.with_user_id(uid);
        }
        if let Some(ip) = ip {
            event = event.with_ip_address(ip);
        }

        self.logger.log(event).await
    }

    /// Log an access denied event
    pub async fn log_access_denied(
        &mut self,
        user_id: &str,
        resource_type: &str,
        resource_id: &str,
        action: &str,
    ) -> AuthResult<()> {
        let event = AuditEvent::new(
            AuditEventType::AccessDenied,
            AuditSeverity::Security,
            format!(
                "Access denied for user {} on {}:{} action: {}",
                user_id, resource_type, resource_id, action
            ),
        )
        .with_user_id(user_id)
        .with_resource(resource_type, resource_id)
        .with_action(action)
        .with_result(AuditResult::Failure);

        self.logger.log(event).await
    }

    /// Log a resource access event
    pub async fn log_resource_access(
        &mut self,
        user_id: &str,
        resource_type: &str,
        resource_id: &str,
        action: &str,
    ) -> AuthResult<()> {
        let event = AuditEvent::new(
            AuditEventType::ResourceAccessed,
            AuditSeverity::Info,
            format!(
                "User {} accessed {}:{} with action: {}",
                user_id, resource_type, resource_id, action
            ),
        )
        .with_user_id(user_id)
        .with_resource(resource_type, resource_id)
        .with_action(action);

        self.logger.log(event).await
    }

    /// Log a custom event
    pub async fn log_custom_event(&mut self, event: AuditEvent) -> AuthResult<()> {
        self.logger.log(event).await
    }

    /// Query audit events
    pub async fn query(&self, query: AuditQuery) -> AuthResult<Vec<AuditEvent>> {
        self.logger.query(query).await
    }

    /// Cleanup old events
    pub async fn cleanup(&mut self, before: DateTime<Utc>) -> AuthResult<usize> {
        self.logger.cleanup(before).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditEventType::LoginSuccess,
            AuditSeverity::Info,
            "User logged in".to_string(),
        )
        .with_user_id("user123")
        .with_ip_address("127.0.0.1");

        assert_eq!(event.user_id, Some("user123".to_string()));
        assert_eq!(event.ip_address, Some("127.0.0.1".to_string()));
        assert_eq!(event.result, AuditResult::Success);
    }

    #[tokio::test]
    async fn test_in_memory_logger() {
        let mut logger = InMemoryAuditLogger::new();

        let event = AuditEvent::new(
            AuditEventType::LoginSuccess,
            AuditSeverity::Info,
            "Test event".to_string(),
        );

        logger.log(event).await.unwrap();
        assert_eq!(logger.get_all_events().len(), 1);
    }

    #[tokio::test]
    async fn test_audit_query() {
        let mut logger = InMemoryAuditLogger::new();

        // Add multiple events
        logger
            .log(
                AuditEvent::new(
                    AuditEventType::LoginSuccess,
                    AuditSeverity::Info,
                    "User 1 login".to_string(),
                )
                .with_user_id("user1"),
            )
            .await
            .unwrap();

        logger
            .log(
                AuditEvent::new(
                    AuditEventType::LoginFailure,
                    AuditSeverity::Security,
                    "User 2 login failed".to_string(),
                )
                .with_user_id("user2"),
            )
            .await
            .unwrap();

        // Query by user ID
        let query = AuditQuery::new().user_id("user1");
        let results = logger.query(query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].user_id, Some("user1".to_string()));

        // Query by event type
        let query = AuditQuery::new().event_type(AuditEventType::LoginFailure);
        let results = logger.query(query).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_manager() {
        let logger = InMemoryAuditLogger::new();
        let mut manager = AuditManager::new(logger);

        manager
            .log_login_success("user123", Some("127.0.0.1".to_string()), None)
            .await
            .unwrap();

        let query = AuditQuery::new().user_id("user123");
        let results = manager.query(query).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].event_type, AuditEventType::LoginSuccess);
    }

    #[tokio::test]
    async fn test_audit_cleanup() {
        let mut logger = InMemoryAuditLogger::new();

        // Add old event
        let mut old_event = AuditEvent::new(
            AuditEventType::LoginSuccess,
            AuditSeverity::Info,
            "Old event".to_string(),
        );
        old_event.timestamp = Utc::now() - chrono::Duration::days(10);
        logger.log(old_event).await.unwrap();

        // Add recent event
        logger
            .log(AuditEvent::new(
                AuditEventType::LoginSuccess,
                AuditSeverity::Info,
                "Recent event".to_string(),
            ))
            .await
            .unwrap();

        // Cleanup events older than 5 days
        let cutoff = Utc::now() - chrono::Duration::days(5);
        let deleted = logger.cleanup(cutoff).await.unwrap();

        assert_eq!(deleted, 1);
        assert_eq!(logger.get_all_events().len(), 1);
    }

    #[tokio::test]
    async fn test_query_pagination() {
        let mut logger = InMemoryAuditLogger::new();

        // Add 10 events
        for i in 0..10 {
            logger
                .log(AuditEvent::new(
                    AuditEventType::LoginSuccess,
                    AuditSeverity::Info,
                    format!("Event {}", i),
                ))
                .await
                .unwrap();
        }

        // Query with limit
        let query = AuditQuery::new().limit(5);
        let results = logger.query(query).await.unwrap();
        assert_eq!(results.len(), 5);

        // Query with offset and limit
        let query = AuditQuery::new().offset(5).limit(3);
        let results = logger.query(query).await.unwrap();
        assert_eq!(results.len(), 3);
    }
}
