//! Access audit trails and activity logging

use crate::error::{GovernanceError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Audit trail manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditManager {
    /// Audit log entries
    entries: Vec<AuditEntry>,
    /// Maximum entries to retain in memory
    max_entries: usize,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry identifier
    pub id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: EventType,
    /// Actor who performed the action
    pub actor: Actor,
    /// Resource being accessed
    pub resource: Resource,
    /// Action performed
    pub action: Action,
    /// Result of the action
    pub result: ActionResult,
    /// Source IP address
    pub source_ip: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Request identifier
    pub request_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Geolocation
    pub location: Option<Location>,
}

/// Actor performing the action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    /// Actor identifier
    pub id: String,
    /// Actor type
    pub actor_type: ActorType,
    /// Actor name
    pub name: String,
    /// Actor email
    pub email: Option<String>,
    /// Actor roles
    pub roles: Vec<String>,
}

/// Actor type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActorType {
    User,
    ServiceAccount,
    Application,
    System,
    Anonymous,
}

/// Resource being accessed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource identifier
    pub id: String,
    /// Resource type
    pub resource_type: ResourceType,
    /// Resource name
    pub name: String,
    /// Parent resource (if any)
    pub parent: Option<String>,
    /// Resource classification
    pub classification: Option<String>,
}

/// Resource type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceType {
    Dataset,
    Table,
    View,
    Field,
    File,
    Report,
    Dashboard,
    Query,
    Schema,
    Policy,
    User,
    Role,
    Custom(String),
}

/// Action performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    /// Read operations
    Read,
    Query,
    Download,
    Export,
    View,

    /// Write operations
    Create,
    Update,
    Delete,
    Upload,
    Import,

    /// Access control
    Grant,
    Revoke,
    Authenticate,
    Authorize,

    /// Administrative
    Configure,
    Deploy,
    Execute,

    /// Data governance
    Classify,
    Tag,
    Mask,
    Anonymize,

    /// Custom action
    Custom(String),
}

/// Action result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionResult {
    Success,
    Failure,
    PartialSuccess,
    Denied,
    Error,
}

/// Event type for categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventType {
    DataAccess,
    DataModification,
    Authentication,
    Authorization,
    Configuration,
    Governance,
    Compliance,
    Security,
    Custom(String),
}

/// Geolocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

/// Audit query filter
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    /// Filter by actor ID
    pub actor_id: Option<String>,
    /// Filter by resource ID
    pub resource_id: Option<String>,
    /// Filter by action
    pub action: Option<Action>,
    /// Filter by event type
    pub event_type: Option<EventType>,
    /// Filter by result
    pub result: Option<ActionResult>,
    /// Start time
    pub start_time: Option<DateTime<Utc>>,
    /// End time
    pub end_time: Option<DateTime<Utc>>,
    /// Limit results
    pub limit: Option<usize>,
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    pub total_entries: usize,
    pub entries_by_event_type: HashMap<String, usize>,
    pub entries_by_action: HashMap<String, usize>,
    pub entries_by_result: HashMap<String, usize>,
    pub unique_actors: usize,
    pub unique_resources: usize,
    pub success_rate: f64,
}

/// Access pattern analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPattern {
    pub resource_id: String,
    pub access_count: usize,
    pub unique_actors: usize,
    pub first_access: DateTime<Utc>,
    pub last_access: DateTime<Utc>,
    pub actions: HashMap<String, usize>,
}

impl AuditManager {
    /// Create a new audit manager
    pub fn new() -> Self {
        Self::with_capacity(10000)
    }

    /// Create a new audit manager with specified capacity
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            max_entries,
        }
    }

    /// Log an audit event
    pub fn log(&mut self, mut entry: AuditEntry) -> Result<String> {
        if entry.id.is_empty() {
            entry.id = Uuid::new_v4().to_string();
        }

        let id = entry.id.clone();
        self.entries.push(entry);

        // Maintain max capacity (FIFO)
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }

        Ok(id)
    }

    /// Log a data access event
    pub fn log_access(
        &mut self,
        actor: Actor,
        resource: Resource,
        action: Action,
        result: ActionResult,
    ) -> Result<String> {
        let entry = AuditEntry {
            id: String::new(),
            timestamp: Utc::now(),
            event_type: EventType::DataAccess,
            actor,
            resource,
            action,
            result,
            source_ip: None,
            user_agent: None,
            session_id: None,
            request_id: None,
            metadata: HashMap::new(),
            location: None,
        };

        self.log(entry)
    }

    /// Query audit logs
    pub fn query(&self, filter: &AuditQuery) -> Vec<&AuditEntry> {
        let mut results: Vec<&AuditEntry> = self
            .entries
            .iter()
            .filter(|entry| {
                // Filter by actor
                if let Some(ref actor_id) = filter.actor_id {
                    if &entry.actor.id != actor_id {
                        return false;
                    }
                }

                // Filter by resource
                if let Some(ref resource_id) = filter.resource_id {
                    if &entry.resource.id != resource_id {
                        return false;
                    }
                }

                // Filter by action
                if let Some(ref action) = filter.action {
                    if &entry.action != action {
                        return false;
                    }
                }

                // Filter by event type
                if let Some(ref event_type) = filter.event_type {
                    if &entry.event_type != event_type {
                        return false;
                    }
                }

                // Filter by result
                if let Some(ref result) = filter.result {
                    if &entry.result != result {
                        return false;
                    }
                }

                // Filter by time range
                if let Some(start) = filter.start_time {
                    if entry.timestamp < start {
                        return false;
                    }
                }

                if let Some(end) = filter.end_time {
                    if entry.timestamp > end {
                        return false;
                    }
                }

                true
            })
            .collect();

        // Apply limit
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        results
    }

    /// Get audit entries for a specific actor
    pub fn get_actor_activity(&self, actor_id: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.actor.id == actor_id)
            .collect()
    }

    /// Get audit entries for a specific resource
    pub fn get_resource_activity(&self, resource_id: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.resource.id == resource_id)
            .collect()
    }

    /// Get failed access attempts
    pub fn get_failed_attempts(&self) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.result == ActionResult::Failure
                    || entry.result == ActionResult::Denied
                    || entry.result == ActionResult::Error
            })
            .collect()
    }

    /// Analyze access patterns for a resource
    pub fn analyze_access_pattern(&self, resource_id: &str) -> Option<AccessPattern> {
        let activities = self.get_resource_activity(resource_id);

        if activities.is_empty() {
            return None;
        }

        let mut unique_actors = std::collections::HashSet::new();
        let mut actions = HashMap::new();
        let mut first_access = activities[0].timestamp;
        let mut last_access = activities[0].timestamp;

        for entry in &activities {
            unique_actors.insert(entry.actor.id.clone());
            *actions
                .entry(format!("{:?}", entry.action))
                .or_insert(0) += 1;

            if entry.timestamp < first_access {
                first_access = entry.timestamp;
            }
            if entry.timestamp > last_access {
                last_access = entry.timestamp;
            }
        }

        Some(AccessPattern {
            resource_id: resource_id.to_string(),
            access_count: activities.len(),
            unique_actors: unique_actors.len(),
            first_access,
            last_access,
            actions,
        })
    }

    /// Get audit statistics
    pub fn get_statistics(&self) -> AuditStatistics {
        let mut entries_by_event_type = HashMap::new();
        let mut entries_by_action = HashMap::new();
        let mut entries_by_result = HashMap::new();
        let mut unique_actors = std::collections::HashSet::new();
        let mut unique_resources = std::collections::HashSet::new();
        let mut success_count = 0;

        for entry in &self.entries {
            *entries_by_event_type
                .entry(format!("{:?}", entry.event_type))
                .or_insert(0) += 1;

            *entries_by_action
                .entry(format!("{:?}", entry.action))
                .or_insert(0) += 1;

            *entries_by_result
                .entry(format!("{:?}", entry.result))
                .or_insert(0) += 1;

            unique_actors.insert(entry.actor.id.clone());
            unique_resources.insert(entry.resource.id.clone());

            if entry.result == ActionResult::Success {
                success_count += 1;
            }
        }

        let success_rate = if !self.entries.is_empty() {
            success_count as f64 / self.entries.len() as f64
        } else {
            0.0
        };

        AuditStatistics {
            total_entries: self.entries.len(),
            entries_by_event_type,
            entries_by_action,
            entries_by_result,
            unique_actors: unique_actors.len(),
            unique_resources: unique_resources.len(),
            success_rate,
        }
    }

    /// Get recent entries
    pub fn get_recent(&self, count: usize) -> Vec<&AuditEntry> {
        let start = if self.entries.len() > count {
            self.entries.len() - count
        } else {
            0
        };
        self.entries[start..].iter().rev().collect()
    }

    /// Clear all audit entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get entry count
    pub fn count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for AuditManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_manager_creation() {
        let manager = AuditManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_log_access() {
        let mut manager = AuditManager::new();

        let actor = Actor {
            id: "user123".to_string(),
            actor_type: ActorType::User,
            name: "Test User".to_string(),
            email: Some("test@example.com".to_string()),
            roles: vec!["analyst".to_string()],
        };

        let resource = Resource {
            id: "dataset1".to_string(),
            resource_type: ResourceType::Dataset,
            name: "Test Dataset".to_string(),
            parent: None,
            classification: Some("confidential".to_string()),
        };

        let id = manager
            .log_access(actor, resource, Action::Read, ActionResult::Success)
            .unwrap();

        assert!(!id.is_empty());
        assert_eq!(manager.count(), 1);
    }
}
