//! # User Presence and Cursor Tracking
//!
//! This module provides functionality for tracking user presence, cursors, and selections
//! in collaborative editing sessions.
//!
//! ## Features
//! - Real-time cursor position tracking
//! - Selection ranges
//! - User metadata (name, color, avatar)
//! - Activity status (active, idle, away)
//! - Cursor transformation through operations

use crate::crdt::ReplicaId;
use crate::ot::Operation;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// User presence information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPresence {
    /// User identifier
    pub user_id: String,

    /// Replica ID for this user's session
    pub replica_id: ReplicaId,

    /// Display name
    pub name: String,

    /// User avatar URL or identifier
    pub avatar: Option<String>,

    /// Color for cursor/selection display (hex format)
    pub color: String,

    /// Current cursor position
    pub cursor: Option<CursorPosition>,

    /// Current selection range
    pub selection: Option<Selection>,

    /// Activity status
    pub status: PresenceStatus,

    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,

    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl UserPresence {
    /// Create a new user presence
    pub fn new(user_id: String, replica_id: ReplicaId, name: String, color: String) -> Self {
        Self {
            user_id,
            replica_id,
            name,
            avatar: None,
            color,
            cursor: None,
            selection: None,
            status: PresenceStatus::Active,
            last_activity: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Update cursor position
    pub fn set_cursor(&mut self, cursor: CursorPosition) {
        self.cursor = Some(cursor);
        self.last_activity = Utc::now();
        self.status = PresenceStatus::Active;
    }

    /// Update selection range
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = Some(selection);
        self.last_activity = Utc::now();
        self.status = PresenceStatus::Active;
    }

    /// Clear cursor and selection
    pub fn clear_cursor(&mut self) {
        self.cursor = None;
        self.selection = None;
    }

    /// Update activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Utc::now();
        if self.status == PresenceStatus::Idle || self.status == PresenceStatus::Away {
            self.status = PresenceStatus::Active;
        }
    }

    /// Check if user has been idle for a duration
    pub fn is_idle(&self, idle_threshold: Duration) -> bool {
        Utc::now() - self.last_activity > idle_threshold
    }

    /// Update status based on activity
    pub fn update_status(&mut self, idle_threshold: Duration, away_threshold: Duration) {
        let inactive_duration = Utc::now() - self.last_activity;

        if inactive_duration > away_threshold {
            self.status = PresenceStatus::Away;
        } else if inactive_duration > idle_threshold {
            self.status = PresenceStatus::Idle;
        } else {
            self.status = PresenceStatus::Active;
        }
    }

    /// Transform cursor and selection through an operation
    pub fn transform_through_op(&mut self, op: &Operation) {
        if let Some(cursor) = &mut self.cursor {
            cursor.transform_through_op(op);
        }

        if let Some(selection) = &mut self.selection {
            selection.transform_through_op(op);
        }
    }
}

/// User activity status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresenceStatus {
    /// Actively editing
    Active,

    /// Idle (viewing but not editing)
    Idle,

    /// Away (inactive for extended period)
    Away,

    /// Offline
    Offline,
}

/// Cursor position in a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorPosition {
    /// Character offset in the document
    pub offset: usize,

    /// Line number (optional, for display)
    pub line: Option<usize>,

    /// Column number (optional, for display)
    pub column: Option<usize>,
}

impl CursorPosition {
    /// Create a new cursor position with just offset
    pub fn new(offset: usize) -> Self {
        Self {
            offset,
            line: None,
            column: None,
        }
    }

    /// Create a cursor with line and column information
    pub fn with_line_col(offset: usize, line: usize, column: usize) -> Self {
        Self {
            offset,
            line: Some(line),
            column: Some(column),
        }
    }

    /// Transform cursor position through an operation
    pub fn transform_through_op(&mut self, op: &Operation) {
        self.offset = crate::ot::transform::transform_cursor(self.offset, op);
        // Line and column would need to be recalculated from the new text
        self.line = None;
        self.column = None;
    }
}

/// Selection range in a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// Start of selection (anchor)
    pub start: usize,

    /// End of selection (head/cursor)
    pub end: usize,
}

impl Selection {
    /// Create a new selection
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Check if selection is collapsed (cursor, not a range)
    pub fn is_collapsed(&self) -> bool {
        self.start == self.end
    }

    /// Get the length of the selection
    pub fn len(&self) -> usize {
        if self.end >= self.start {
            self.end - self.start
        } else {
            self.start - self.end
        }
    }

    /// Get normalized range (start <= end)
    pub fn normalized(&self) -> (usize, usize) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Transform selection through an operation
    pub fn transform_through_op(&mut self, op: &Operation) {
        let ot_selection = crate::ot::transform::Selection::new(self.start, self.end);
        let transformed = crate::ot::transform::transform_selection(ot_selection, op);
        self.start = transformed.start;
        self.end = transformed.end;
    }
}

/// Presence manager for tracking multiple users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceManager {
    /// Map of user ID to presence information
    users: HashMap<String, UserPresence>,

    /// Idle threshold duration
    idle_threshold: Duration,

    /// Away threshold duration
    away_threshold: Duration,
}

impl PresenceManager {
    /// Create a new presence manager
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            idle_threshold: Duration::seconds(30),
            away_threshold: Duration::minutes(5),
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(idle_threshold: Duration, away_threshold: Duration) -> Self {
        Self {
            users: HashMap::new(),
            idle_threshold,
            away_threshold,
        }
    }

    /// Add or update a user's presence
    pub fn update_user(&mut self, presence: UserPresence) {
        self.users.insert(presence.user_id.clone(), presence);
    }

    /// Remove a user
    pub fn remove_user(&mut self, user_id: &str) {
        self.users.remove(user_id);
    }

    /// Get a user's presence
    pub fn get_user(&self, user_id: &str) -> Option<&UserPresence> {
        self.users.get(user_id)
    }

    /// Get a mutable reference to user presence
    pub fn get_user_mut(&mut self, user_id: &str) -> Option<&mut UserPresence> {
        self.users.get_mut(user_id)
    }

    /// Get all active users
    pub fn active_users(&self) -> Vec<&UserPresence> {
        self.users
            .values()
            .filter(|p| p.status != PresenceStatus::Offline)
            .collect()
    }

    /// Get all users
    pub fn all_users(&self) -> Vec<&UserPresence> {
        self.users.values().collect()
    }

    /// Update all user statuses based on activity
    pub fn update_all_statuses(&mut self) {
        for user in self.users.values_mut() {
            user.update_status(self.idle_threshold, self.away_threshold);
        }
    }

    /// Transform all cursors and selections through an operation
    pub fn transform_all_through_op(&mut self, op: &Operation) {
        for user in self.users.values_mut() {
            user.transform_through_op(op);
        }
    }

    /// Clean up offline users that haven't been active for a long time
    pub fn cleanup_offline(&mut self, threshold: Duration) {
        let cutoff = Utc::now() - threshold;
        self.users.retain(|_, user| {
            user.status != PresenceStatus::Offline || user.last_activity > cutoff
        });
    }

    /// Get number of active users
    pub fn active_count(&self) -> usize {
        self.active_users().len()
    }

    /// Get user by replica ID
    pub fn get_by_replica_id(&self, replica_id: &ReplicaId) -> Option<&UserPresence> {
        self.users.values().find(|u| u.replica_id == *replica_id)
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Presence event for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresenceEvent {
    /// User joined the session
    UserJoined(UserPresence),

    /// User left the session
    UserLeft(String),

    /// User cursor moved
    CursorMoved {
        user_id: String,
        cursor: CursorPosition,
    },

    /// User selection changed
    SelectionChanged {
        user_id: String,
        selection: Selection,
    },

    /// User status changed
    StatusChanged {
        user_id: String,
        status: PresenceStatus,
    },

    /// User metadata updated
    MetadataUpdated {
        user_id: String,
        metadata: HashMap<String, String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_presence() {
        let replica_id = ReplicaId::new();
        let mut presence = UserPresence::new(
            "user1".to_string(),
            replica_id,
            "Alice".to_string(),
            "#FF0000".to_string(),
        );

        assert_eq!(presence.status, PresenceStatus::Active);

        presence.set_cursor(CursorPosition::new(10));
        assert_eq!(presence.cursor.unwrap().offset, 10);

        presence.set_selection(Selection::new(5, 15));
        assert_eq!(presence.selection.unwrap().len(), 10);
    }

    #[test]
    fn test_cursor_position() {
        let mut cursor = CursorPosition::new(5);
        assert_eq!(cursor.offset, 5);

        let mut op = Operation::new();
        op.retain(3).insert("XX");

        cursor.transform_through_op(&op);
        assert_eq!(cursor.offset, 7); // Cursor after "XX" insertion
    }

    #[test]
    fn test_selection() {
        let mut sel = Selection::new(5, 10);
        assert!(!sel.is_collapsed());
        assert_eq!(sel.len(), 5);

        let (start, end) = sel.normalized();
        assert_eq!(start, 5);
        assert_eq!(end, 10);

        let collapsed = Selection::new(5, 5);
        assert!(collapsed.is_collapsed());
    }

    #[test]
    fn test_presence_manager() {
        let mut manager = PresenceManager::new();

        let replica_id = ReplicaId::new();
        let presence = UserPresence::new(
            "user1".to_string(),
            replica_id,
            "Alice".to_string(),
            "#FF0000".to_string(),
        );

        manager.update_user(presence.clone());
        assert_eq!(manager.active_count(), 1);

        assert!(manager.get_user("user1").is_some());
        assert!(manager.get_by_replica_id(&replica_id).is_some());

        manager.remove_user("user1");
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_presence_status_update() {
        let replica_id = ReplicaId::new();
        let mut presence = UserPresence::new(
            "user1".to_string(),
            replica_id,
            "Alice".to_string(),
            "#FF0000".to_string(),
        );

        // Simulate idle
        let idle_threshold = Duration::milliseconds(10);
        let away_threshold = Duration::milliseconds(50);

        std::thread::sleep(std::time::Duration::from_millis(15));
        presence.update_status(idle_threshold, away_threshold);
        assert_eq!(presence.status, PresenceStatus::Idle);

        std::thread::sleep(std::time::Duration::from_millis(40));
        presence.update_status(idle_threshold, away_threshold);
        assert_eq!(presence.status, PresenceStatus::Away);
    }

    #[test]
    fn test_transform_through_operation() {
        let replica_id = ReplicaId::new();
        let mut presence = UserPresence::new(
            "user1".to_string(),
            replica_id,
            "Alice".to_string(),
            "#FF0000".to_string(),
        );

        presence.set_cursor(CursorPosition::new(10));

        let mut op = Operation::new();
        op.retain(5).insert("Hello");

        presence.transform_through_op(&op);

        assert_eq!(presence.cursor.unwrap().offset, 15);
    }

    #[test]
    fn test_presence_manager_transform() {
        let mut manager = PresenceManager::new();

        let replica_id = ReplicaId::new();
        let mut presence = UserPresence::new(
            "user1".to_string(),
            replica_id,
            "Alice".to_string(),
            "#FF0000".to_string(),
        );
        presence.set_cursor(CursorPosition::new(10));

        manager.update_user(presence);

        let mut op = Operation::new();
        op.retain(5).insert("XXX");

        manager.transform_all_through_op(&op);

        let user = manager.get_user("user1").unwrap();
        assert_eq!(user.cursor.unwrap().offset, 13);
    }
}
