//! # Collaboration Session Management
//!
//! This module provides session management for collaborative editing sessions.
//! It handles document state, operation history, and user coordination.

use crate::crdt::{ReplicaId, VersionVector};
use crate::ot::Operation;
use crate::presence::{PresenceManager, UserPresence};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Unique identifier for a collaboration session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

impl SessionId {
    /// Create a new random session ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Collaboration session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    /// Session identifier
    pub id: SessionId,

    /// Document identifier
    pub document_id: String,

    /// Current document content
    pub content: String,

    /// Document version
    pub version: u64,

    /// Operation history
    pub history: VecDeque<OperationRecord>,

    /// Maximum history size
    max_history: usize,

    /// Presence manager
    pub presence: PresenceManager,

    /// Session metadata
    pub metadata: SessionMetadata,

    /// Version vector for causal ordering
    pub version_vector: VersionVector,
}

impl CollaborationSession {
    /// Create a new collaboration session
    pub fn new(document_id: String, initial_content: String) -> Self {
        Self {
            id: SessionId::new(),
            document_id,
            content: initial_content,
            version: 0,
            history: VecDeque::new(),
            max_history: 1000,
            presence: PresenceManager::new(),
            metadata: SessionMetadata::new(),
            version_vector: VersionVector::new(),
        }
    }

    /// Create with specific session ID
    pub fn with_id(id: SessionId, document_id: String, initial_content: String) -> Self {
        let mut session = Self::new(document_id, initial_content);
        session.id = id;
        session
    }

    /// Set maximum history size
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        self.trim_history();
    }

    /// Apply an operation to the session
    pub fn apply_operation(
        &mut self,
        operation: Operation,
        author: ReplicaId,
    ) -> Result<(), SessionError> {
        // Validate operation
        if operation.base_len() != self.content.chars().count() {
            return Err(SessionError::InvalidOperation {
                expected_len: self.content.chars().count(),
                actual_len: operation.base_len(),
            });
        }

        // Apply operation to content
        self.content = operation
            .apply(&self.content)
            .map_err(|e| SessionError::OperationError(e.to_string()))?;

        // Update version
        self.version += 1;
        self.version_vector.increment(author);

        // Record operation
        let record = OperationRecord {
            version: self.version,
            operation: operation.clone(),
            author,
            timestamp: Utc::now(),
        };

        self.history.push_back(record);
        self.trim_history();

        // Transform all user cursors/selections
        self.presence.transform_all_through_op(&operation);

        // Update activity
        self.metadata.last_modified = Utc::now();

        Ok(())
    }

    /// Add a user to the session
    pub fn add_user(&mut self, presence: UserPresence) {
        self.presence.update_user(presence);
        self.metadata.participant_count = self.presence.active_count();
    }

    /// Remove a user from the session
    pub fn remove_user(&mut self, user_id: &str) {
        self.presence.remove_user(user_id);
        self.metadata.participant_count = self.presence.active_count();
    }

    /// Get operations since a version
    pub fn get_operations_since(&self, version: u64) -> Vec<&OperationRecord> {
        self.history
            .iter()
            .filter(|r| r.version > version)
            .collect()
    }

    /// Get the current document state
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// Get session statistics
    pub fn stats(&self) -> SessionStats {
        SessionStats {
            version: self.version,
            content_length: self.content.len(),
            operation_count: self.history.len(),
            participant_count: self.presence.active_count(),
            created_at: self.metadata.created_at,
            last_modified: self.metadata.last_modified,
        }
    }

    /// Trim history to max size
    fn trim_history(&mut self) {
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// Create a snapshot of the session
    pub fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            session_id: self.id,
            document_id: self.document_id.clone(),
            content: self.content.clone(),
            version: self.version,
            version_vector: self.version_vector.clone(),
            timestamp: Utc::now(),
        }
    }

    /// Restore from a snapshot
    pub fn restore_from_snapshot(&mut self, snapshot: SessionSnapshot) {
        self.content = snapshot.content;
        self.version = snapshot.version;
        self.version_vector = snapshot.version_vector;
        self.metadata.last_modified = snapshot.timestamp;
    }

    /// Check if session is idle
    pub fn is_idle(&self, threshold: chrono::Duration) -> bool {
        Utc::now() - self.metadata.last_modified > threshold
    }

    /// Get operation at specific version
    pub fn get_operation_at_version(&self, version: u64) -> Option<&OperationRecord> {
        self.history.iter().find(|r| r.version == version)
    }
}

/// Record of an applied operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    /// Version number when this operation was applied
    pub version: u64,

    /// The operation itself
    pub operation: Operation,

    /// Who authored this operation
    pub author: ReplicaId,

    /// When it was applied
    pub timestamp: DateTime<Utc>,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// When the session was created
    pub created_at: DateTime<Utc>,

    /// Last modification time
    pub last_modified: DateTime<Utc>,

    /// Number of participants
    pub participant_count: usize,

    /// Custom metadata
    pub custom: HashMap<String, String>,
}

impl SessionMetadata {
    fn new() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            last_modified: now,
            participant_count: 0,
            custom: HashMap::new(),
        }
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub version: u64,
    pub content_length: usize,
    pub operation_count: usize,
    pub participant_count: usize,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

/// Snapshot of a session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: SessionId,
    pub document_id: String,
    pub content: String,
    pub version: u64,
    pub version_vector: VersionVector,
    pub timestamp: DateTime<Utc>,
}

/// Session errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Invalid operation: expected base length {expected_len}, got {actual_len}")]
    InvalidOperation {
        expected_len: usize,
        actual_len: usize,
    },

    #[error("Operation error: {0}")]
    OperationError(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Version conflict: {0}")]
    VersionConflict(String),
}

/// Manager for multiple collaboration sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManager {
    sessions: HashMap<SessionId, CollaborationSession>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Create a new session
    pub fn create_session(
        &mut self,
        document_id: String,
        initial_content: String,
    ) -> SessionId {
        let session = CollaborationSession::new(document_id, initial_content);
        let id = session.id;
        self.sessions.insert(id, session);
        id
    }

    /// Get a session
    pub fn get_session(&self, id: &SessionId) -> Option<&CollaborationSession> {
        self.sessions.get(id)
    }

    /// Get a mutable session
    pub fn get_session_mut(&mut self, id: &SessionId) -> Option<&mut CollaborationSession> {
        self.sessions.get_mut(id)
    }

    /// Remove a session
    pub fn remove_session(&mut self, id: &SessionId) -> Option<CollaborationSession> {
        self.sessions.remove(id)
    }

    /// Get all sessions for a document
    pub fn get_sessions_for_document(&self, document_id: &str) -> Vec<&CollaborationSession> {
        self.sessions
            .values()
            .filter(|s| s.document_id == document_id)
            .collect()
    }

    /// Get total session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Clean up idle sessions
    pub fn cleanup_idle_sessions(&mut self, threshold: chrono::Duration) {
        self.sessions.retain(|_, session| !session.is_idle(threshold));
    }

    /// Get all session IDs
    pub fn session_ids(&self) -> Vec<SessionId> {
        self.sessions.keys().copied().collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = CollaborationSession::new(
            "doc1".to_string(),
            "Hello World".to_string(),
        );

        assert_eq!(session.version, 0);
        assert_eq!(session.content, "Hello World");
        assert_eq!(session.document_id, "doc1");
    }

    #[test]
    fn test_apply_operation() {
        let mut session = CollaborationSession::new(
            "doc1".to_string(),
            "Hello".to_string(),
        );

        let mut op = Operation::new();
        op.retain(5).insert(" World");

        let replica_id = ReplicaId::new();
        session.apply_operation(op, replica_id).unwrap();

        assert_eq!(session.content, "Hello World");
        assert_eq!(session.version, 1);
        assert_eq!(session.history.len(), 1);
    }

    #[test]
    fn test_session_history() {
        let mut session = CollaborationSession::new(
            "doc1".to_string(),
            "".to_string(),
        );

        let replica_id = ReplicaId::new();

        let mut op1 = Operation::new();
        op1.insert("A");
        session.apply_operation(op1, replica_id).unwrap();

        let mut op2 = Operation::new();
        op2.retain(1).insert("B");
        session.apply_operation(op2, replica_id).unwrap();

        assert_eq!(session.version, 2);
        assert_eq!(session.content, "AB");

        let ops = session.get_operations_since(0);
        assert_eq!(ops.len(), 2);

        let ops = session.get_operations_since(1);
        assert_eq!(ops.len(), 1);
    }

    #[test]
    fn test_session_presence() {
        let mut session = CollaborationSession::new(
            "doc1".to_string(),
            "Hello".to_string(),
        );

        let replica_id = ReplicaId::new();
        let presence = UserPresence::new(
            "user1".to_string(),
            replica_id,
            "Alice".to_string(),
            "#FF0000".to_string(),
        );

        session.add_user(presence);
        assert_eq!(session.metadata.participant_count, 1);

        session.remove_user("user1");
        assert_eq!(session.metadata.participant_count, 0);
    }

    #[test]
    fn test_session_snapshot() {
        let mut session = CollaborationSession::new(
            "doc1".to_string(),
            "Hello".to_string(),
        );

        let mut op = Operation::new();
        op.retain(5).insert(" World");
        session.apply_operation(op, ReplicaId::new()).unwrap();

        let snapshot = session.snapshot();
        assert_eq!(snapshot.content, "Hello World");
        assert_eq!(snapshot.version, 1);

        let mut new_session = CollaborationSession::new(
            "doc2".to_string(),
            "".to_string(),
        );
        new_session.restore_from_snapshot(snapshot);

        assert_eq!(new_session.content, "Hello World");
        assert_eq!(new_session.version, 1);
    }

    #[test]
    fn test_session_manager() {
        let mut manager = SessionManager::new();

        let id1 = manager.create_session("doc1".to_string(), "Content1".to_string());
        let id2 = manager.create_session("doc2".to_string(), "Content2".to_string());

        assert_eq!(manager.session_count(), 2);

        let session1 = manager.get_session(&id1).unwrap();
        assert_eq!(session1.content, "Content1");

        manager.remove_session(&id1);
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_session_stats() {
        let mut session = CollaborationSession::new(
            "doc1".to_string(),
            "Hello".to_string(),
        );

        let mut op = Operation::new();
        op.retain(5).insert(" World");
        session.apply_operation(op, ReplicaId::new()).unwrap();

        let stats = session.stats();
        assert_eq!(stats.version, 1);
        assert_eq!(stats.operation_count, 1);
        assert_eq!(stats.content_length, 11);
    }

    #[test]
    fn test_max_history() {
        let mut session = CollaborationSession::new(
            "doc1".to_string(),
            "".to_string(),
        );
        session.set_max_history(5);

        let replica_id = ReplicaId::new();

        // Add 10 operations
        for i in 0..10 {
            let mut op = Operation::new();
            op.retain(i).insert("X");
            session.apply_operation(op, replica_id).unwrap();
        }

        // Should only keep last 5
        assert_eq!(session.history.len(), 5);
    }

    #[test]
    fn test_invalid_operation() {
        let mut session = CollaborationSession::new(
            "doc1".to_string(),
            "Hello".to_string(),
        );

        let mut op = Operation::new();
        op.retain(10).insert("X"); // Wrong base length

        let result = session.apply_operation(op, ReplicaId::new());
        assert!(result.is_err());
    }
}
