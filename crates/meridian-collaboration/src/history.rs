//! # Version History and Time Travel
//!
//! This module provides comprehensive version history tracking and time travel capabilities
//! for collaborative documents.

use crate::crdt::ReplicaId;
use crate::ot::{compose, Operation};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, VecDeque};

/// Version identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VersionId(pub u64);

impl VersionId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn prev(&self) -> Option<Self> {
        if self.0 > 0 {
            Some(Self(self.0 - 1))
        } else {
            None
        }
    }
}

impl std::fmt::Display for VersionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// A snapshot of document state at a specific version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: VersionId,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub author: Option<ReplicaId>,
    pub message: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(version: VersionId, content: String) -> Self {
        Self {
            version,
            content,
            timestamp: Utc::now(),
            author: None,
            message: None,
            metadata: HashMap::new(),
        }
    }

    /// Create with metadata
    pub fn with_metadata(
        version: VersionId,
        content: String,
        author: ReplicaId,
        message: String,
    ) -> Self {
        Self {
            version,
            content,
            timestamp: Utc::now(),
            author: Some(author),
            message: Some(message),
            metadata: HashMap::new(),
        }
    }
}

/// Version history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub version: VersionId,
    pub operation: Operation,
    pub author: ReplicaId,
    pub timestamp: DateTime<Utc>,
    pub message: Option<String>,
    pub parent: Option<VersionId>,
}

/// Version history manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    /// Current version
    current_version: VersionId,

    /// All version entries (version -> entry)
    entries: BTreeMap<VersionId, HistoryEntry>,

    /// Snapshots at key versions
    snapshots: BTreeMap<VersionId, Snapshot>,

    /// Snapshot interval (create snapshot every N versions)
    snapshot_interval: u64,

    /// Initial content
    initial_content: String,

    /// Maximum entries to keep
    max_entries: usize,
}

impl VersionHistory {
    /// Create a new version history
    pub fn new(initial_content: String) -> Self {
        Self {
            current_version: VersionId(0),
            entries: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            snapshot_interval: 10,
            initial_content,
            max_entries: 10000,
        }
    }

    /// Create with custom snapshot interval
    pub fn with_snapshot_interval(initial_content: String, interval: u64) -> Self {
        let mut history = Self::new(initial_content);
        history.snapshot_interval = interval;
        history
    }

    /// Add a new version
    pub fn add_version(
        &mut self,
        operation: Operation,
        author: ReplicaId,
        message: Option<String>,
    ) -> VersionId {
        let new_version = self.current_version.next();

        let entry = HistoryEntry {
            version: new_version,
            operation,
            author,
            timestamp: Utc::now(),
            message,
            parent: Some(self.current_version),
        };

        self.entries.insert(new_version, entry);
        self.current_version = new_version;

        // Create snapshot if needed
        if new_version.0 % self.snapshot_interval == 0 {
            if let Ok(content) = self.get_content_at_version(new_version) {
                let snapshot = Snapshot::new(new_version, content);
                self.snapshots.insert(new_version, snapshot);
            }
        }

        // Trim if needed
        self.trim_history();

        new_version
    }

    /// Get content at a specific version
    pub fn get_content_at_version(&self, version: VersionId) -> Result<String, HistoryError> {
        if version.0 > self.current_version.0 {
            return Err(HistoryError::VersionNotFound(version));
        }

        // Find the nearest snapshot before this version
        let snapshot_version = self
            .snapshots
            .range(..=version)
            .next_back()
            .map(|(v, _)| *v);

        let (mut content, start_version) = if let Some(snap_version) = snapshot_version {
            let snapshot = self.snapshots.get(&snap_version).unwrap();
            (snapshot.content.clone(), snap_version.next())
        } else {
            (self.initial_content.clone(), VersionId(1))
        };

        // Apply operations from snapshot to target version
        for v in start_version.0..=version.0 {
            let vid = VersionId(v);
            if let Some(entry) = self.entries.get(&vid) {
                content = entry
                    .operation
                    .apply(&content)
                    .map_err(|e| HistoryError::OperationError(e.to_string()))?;
            }
        }

        Ok(content)
    }

    /// Get the current content
    pub fn get_current_content(&self) -> Result<String, HistoryError> {
        self.get_content_at_version(self.current_version)
    }

    /// Create a manual snapshot
    pub fn create_snapshot(&mut self, message: Option<String>) -> Result<(), HistoryError> {
        let content = self.get_current_content()?;
        let snapshot = Snapshot {
            version: self.current_version,
            content,
            timestamp: Utc::now(),
            author: None,
            message,
            metadata: HashMap::new(),
        };

        self.snapshots.insert(self.current_version, snapshot);
        Ok(())
    }

    /// Get a snapshot at a specific version
    pub fn get_snapshot(&self, version: VersionId) -> Option<&Snapshot> {
        self.snapshots.get(&version)
    }

    /// Get all snapshots
    pub fn list_snapshots(&self) -> Vec<&Snapshot> {
        self.snapshots.values().collect()
    }

    /// Get version history between two versions
    pub fn get_history_range(
        &self,
        from: VersionId,
        to: VersionId,
    ) -> Vec<&HistoryEntry> {
        self.entries
            .range(from..=to)
            .map(|(_, entry)| entry)
            .collect()
    }

    /// Get the full history
    pub fn get_full_history(&self) -> Vec<&HistoryEntry> {
        self.entries.values().collect()
    }

    /// Undo to a specific version
    pub fn undo_to_version(&mut self, version: VersionId) -> Result<Operation, HistoryError> {
        if version.0 >= self.current_version.0 {
            return Err(HistoryError::InvalidVersion(version));
        }

        // Get operations to undo
        let mut ops_to_undo = Vec::new();
        for v in (version.0 + 1)..=self.current_version.0 {
            if let Some(entry) = self.entries.get(&VersionId(v)) {
                ops_to_undo.push(entry.operation.clone());
            }
        }

        if ops_to_undo.is_empty() {
            return Err(HistoryError::NoOperationsToUndo);
        }

        // Get content at target version to compute inverse
        let content_at_version = self.get_content_at_version(self.current_version)?;

        // Compose all operations to undo
        let mut combined = ops_to_undo[0].clone();
        for op in &ops_to_undo[1..] {
            combined = compose(&combined, op)
                .map_err(|e| HistoryError::ComposeError(e.to_string()))?;
        }

        // Invert the combined operation
        let undo_op = combined
            .invert(&content_at_version)
            .map_err(|e| HistoryError::OperationError(e.to_string()))?;

        Ok(undo_op)
    }

    /// Get the difference between two versions
    pub fn diff(
        &self,
        from: VersionId,
        to: VersionId,
    ) -> Result<Vec<DiffHunk>, HistoryError> {
        let from_content = self.get_content_at_version(from)?;
        let to_content = self.get_content_at_version(to)?;

        Ok(compute_diff(&from_content, &to_content))
    }

    /// Get current version
    pub fn current_version(&self) -> VersionId {
        self.current_version
    }

    /// Get version count
    pub fn version_count(&self) -> usize {
        self.entries.len()
    }

    /// Get a specific history entry
    pub fn get_entry(&self, version: VersionId) -> Option<&HistoryEntry> {
        self.entries.get(&version)
    }

    /// Trim old history
    fn trim_history(&mut self) {
        if self.entries.len() > self.max_entries {
            let to_remove = self.entries.len() - self.max_entries;
            let versions_to_remove: Vec<_> = self
                .entries
                .keys()
                .take(to_remove)
                .copied()
                .collect();

            for version in versions_to_remove {
                self.entries.remove(&version);
            }
        }
    }

    /// Set maximum entries
    pub fn set_max_entries(&mut self, max: usize) {
        self.max_entries = max;
        self.trim_history();
    }

    /// Get statistics
    pub fn stats(&self) -> HistoryStats {
        let authors: std::collections::HashSet<_> = self
            .entries
            .values()
            .map(|e| e.author)
            .collect();

        HistoryStats {
            version_count: self.entries.len(),
            snapshot_count: self.snapshots.len(),
            unique_authors: authors.len(),
            current_version: self.current_version,
            oldest_version: self.entries.keys().next().copied(),
        }
    }
}

/// History statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    pub version_count: usize,
    pub snapshot_count: usize,
    pub unique_authors: usize,
    pub current_version: VersionId,
    pub oldest_version: Option<VersionId>,
}

/// Diff hunk representing a change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffHunk {
    Equal { text: String },
    Insert { text: String },
    Delete { text: String },
}

/// Compute diff between two strings
fn compute_diff(from: &str, to: &str) -> Vec<DiffHunk> {
    // Simple line-based diff
    let from_lines: Vec<&str> = from.lines().collect();
    let to_lines: Vec<&str> = to.lines().collect();

    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;

    while i < from_lines.len() || j < to_lines.len() {
        if i < from_lines.len() && j < to_lines.len() && from_lines[i] == to_lines[j] {
            result.push(DiffHunk::Equal {
                text: from_lines[i].to_string(),
            });
            i += 1;
            j += 1;
        } else if j < to_lines.len() {
            result.push(DiffHunk::Insert {
                text: to_lines[j].to_string(),
            });
            j += 1;
        } else {
            result.push(DiffHunk::Delete {
                text: from_lines[i].to_string(),
            });
            i += 1;
        }
    }

    result
}

/// History errors
#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    #[error("Version not found: {0}")]
    VersionNotFound(VersionId),

    #[error("Invalid version: {0}")]
    InvalidVersion(VersionId),

    #[error("Operation error: {0}")]
    OperationError(String),

    #[error("Compose error: {0}")]
    ComposeError(String),

    #[error("No operations to undo")]
    NoOperationsToUndo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_id() {
        let v1 = VersionId(1);
        let v2 = v1.next();

        assert_eq!(v2.0, 2);
        assert_eq!(v2.prev(), Some(v1));
    }

    #[test]
    fn test_version_history() {
        let mut history = VersionHistory::new("Hello".to_string());

        let replica_id = ReplicaId::new();

        let mut op1 = Operation::new();
        op1.retain(5).insert(" World");

        let v1 = history.add_version(op1, replica_id, Some("Add World".to_string()));

        assert_eq!(v1, VersionId(1));
        assert_eq!(history.current_version(), VersionId(1));

        let content = history.get_content_at_version(v1).unwrap();
        assert_eq!(content, "Hello World");
    }

    #[test]
    fn test_multiple_versions() {
        let mut history = VersionHistory::new("".to_string());
        let replica_id = ReplicaId::new();

        let mut op1 = Operation::new();
        op1.insert("A");
        history.add_version(op1, replica_id, None);

        let mut op2 = Operation::new();
        op2.retain(1).insert("B");
        history.add_version(op2, replica_id, None);

        let mut op3 = Operation::new();
        op3.retain(2).insert("C");
        history.add_version(op3, replica_id, None);

        assert_eq!(history.get_current_content().unwrap(), "ABC");
        assert_eq!(history.get_content_at_version(VersionId(1)).unwrap(), "A");
        assert_eq!(history.get_content_at_version(VersionId(2)).unwrap(), "AB");
    }

    #[test]
    fn test_snapshots() {
        let mut history = VersionHistory::with_snapshot_interval("".to_string(), 2);
        let replica_id = ReplicaId::new();

        for _ in 0..5 {
            let mut op = Operation::new();
            op.retain(history.current_version().0 as usize).insert("X");
            history.add_version(op, replica_id, None);
        }

        // Should have snapshots at versions 2 and 4
        assert!(history.get_snapshot(VersionId(2)).is_some());
        assert!(history.get_snapshot(VersionId(4)).is_some());
    }

    #[test]
    fn test_manual_snapshot() {
        let mut history = VersionHistory::new("Hello".to_string());
        let replica_id = ReplicaId::new();

        let mut op = Operation::new();
        op.retain(5).insert(" World");
        history.add_version(op, replica_id, None);

        history.create_snapshot(Some("Manual snapshot".to_string())).unwrap();

        let snapshot = history.get_snapshot(history.current_version()).unwrap();
        assert_eq!(snapshot.message, Some("Manual snapshot".to_string()));
    }

    #[test]
    fn test_history_range() {
        let mut history = VersionHistory::new("".to_string());
        let replica_id = ReplicaId::new();

        for i in 0..10 {
            let mut op = Operation::new();
            op.retain(i).insert("X");
            history.add_version(op, replica_id, None);
        }

        let range = history.get_history_range(VersionId(3), VersionId(7));
        assert_eq!(range.len(), 5);
    }

    #[test]
    fn test_diff() {
        let mut history = VersionHistory::new("Line1\nLine2\nLine3".to_string());
        let replica_id = ReplicaId::new();

        let mut op = Operation::new();
        op.retain(12).insert("\nLine4");
        history.add_version(op, replica_id, None);

        let diff = history.diff(VersionId(0), VersionId(1)).unwrap();

        // Should contain the added line
        assert!(diff.iter().any(|h| matches!(h, DiffHunk::Insert { .. })));
    }

    #[test]
    fn test_undo() {
        let mut history = VersionHistory::new("Hello".to_string());
        let replica_id = ReplicaId::new();

        let mut op = Operation::new();
        op.retain(5).insert(" World");
        history.add_version(op, replica_id, None);

        assert_eq!(history.get_current_content().unwrap(), "Hello World");

        let undo_op = history.undo_to_version(VersionId(0)).unwrap();
        let reverted = undo_op.apply("Hello World").unwrap();

        assert_eq!(reverted, "Hello");
    }

    #[test]
    fn test_history_stats() {
        let mut history = VersionHistory::new("".to_string());
        let replica_id = ReplicaId::new();

        for _ in 0..5 {
            let mut op = Operation::new();
            op.insert("X");
            history.add_version(op, replica_id, None);
        }

        let stats = history.stats();
        assert_eq!(stats.version_count, 5);
        assert_eq!(stats.current_version, VersionId(5));
    }

    #[test]
    fn test_trim_history() {
        let mut history = VersionHistory::new("".to_string());
        history.set_max_entries(5);

        let replica_id = ReplicaId::new();

        for i in 0..10 {
            let mut op = Operation::new();
            op.retain(i).insert("X");
            history.add_version(op, replica_id, None);
        }

        assert_eq!(history.version_count(), 5);
    }
}
