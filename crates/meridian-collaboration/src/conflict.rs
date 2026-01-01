//! # Conflict Detection and Resolution
//!
//! This module provides advanced conflict detection and resolution strategies
//! for collaborative editing scenarios.

use crate::crdt::{ReplicaId, VersionVector};
use crate::ot::{Operation, OpComponent};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conflict detection result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictType {
    /// No conflict detected
    None,

    /// Concurrent modifications to same region
    ConcurrentEdit {
        region: TextRegion,
        operations: Vec<ConflictingOperation>,
    },

    /// Insert/Delete conflict
    InsertDeleteConflict {
        insert_op: ConflictingOperation,
        delete_op: ConflictingOperation,
    },

    /// Overlapping deletions
    OverlappingDelete {
        deletions: Vec<ConflictingOperation>,
    },

    /// Version vector indicates causal conflict
    CausalConflict {
        local_version: VersionVector,
        remote_version: VersionVector,
    },
}

/// Text region affected by operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRegion {
    pub start: usize,
    pub end: usize,
}

impl TextRegion {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Check if two regions overlap
    pub fn overlaps(&self, other: &TextRegion) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Get the intersection of two regions
    pub fn intersection(&self, other: &TextRegion) -> Option<TextRegion> {
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);

        if start < end {
            Some(TextRegion { start, end })
        } else {
            None
        }
    }

    /// Get the union of two regions
    pub fn union(&self, other: &TextRegion) -> TextRegion {
        TextRegion {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Get the length of the region
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if region is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Conflicting operation with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictingOperation {
    pub operation: Operation,
    pub author: ReplicaId,
    pub timestamp: DateTime<Utc>,
    pub affected_region: TextRegion,
}

/// Conflict detector
#[derive(Debug)]
pub struct ConflictDetector {
    /// Recent operations by author
    operation_history: HashMap<ReplicaId, Vec<OperationRecord>>,

    /// Maximum history to keep
    max_history: usize,
}

#[derive(Debug, Clone)]
struct OperationRecord {
    operation: Operation,
    timestamp: DateTime<Utc>,
    affected_region: TextRegion,
}

impl ConflictDetector {
    /// Create a new conflict detector
    pub fn new() -> Self {
        Self {
            operation_history: HashMap::new(),
            max_history: 100,
        }
    }

    /// Create with custom history size
    pub fn with_history_size(max_history: usize) -> Self {
        Self {
            operation_history: HashMap::new(),
            max_history,
        }
    }

    /// Record an operation
    pub fn record_operation(
        &mut self,
        operation: Operation,
        author: ReplicaId,
        timestamp: DateTime<Utc>,
    ) {
        let affected_region = Self::get_affected_region(&operation);

        let record = OperationRecord {
            operation,
            timestamp,
            affected_region,
        };

        let history = self.operation_history.entry(author).or_insert_with(Vec::new);
        history.push(record);

        // Trim history
        if history.len() > self.max_history {
            history.drain(0..history.len() - self.max_history);
        }
    }

    /// Detect conflicts between two operations
    pub fn detect_conflict(
        &self,
        op1: &Operation,
        author1: ReplicaId,
        timestamp1: DateTime<Utc>,
        op2: &Operation,
        author2: ReplicaId,
        timestamp2: DateTime<Utc>,
    ) -> ConflictType {
        if author1 == author2 {
            return ConflictType::None;
        }

        let region1 = Self::get_affected_region(op1);
        let region2 = Self::get_affected_region(op2);

        // Check for overlapping regions
        if region1.overlaps(&region2) {
            // Classify the type of conflict
            if Self::has_delete(op1) && Self::has_delete(op2) {
                return ConflictType::OverlappingDelete {
                    deletions: vec![
                        ConflictingOperation {
                            operation: op1.clone(),
                            author: author1,
                            timestamp: timestamp1,
                            affected_region: region1,
                        },
                        ConflictingOperation {
                            operation: op2.clone(),
                            author: author2,
                            timestamp: timestamp2,
                            affected_region: region2,
                        },
                    ],
                };
            } else if Self::has_insert(op1) && Self::has_delete(op2) {
                return ConflictType::InsertDeleteConflict {
                    insert_op: ConflictingOperation {
                        operation: op1.clone(),
                        author: author1,
                        timestamp: timestamp1,
                        affected_region: region1,
                    },
                    delete_op: ConflictingOperation {
                        operation: op2.clone(),
                        author: author2,
                        timestamp: timestamp2,
                        affected_region: region2,
                    },
                };
            } else if Self::has_insert(op2) && Self::has_delete(op1) {
                return ConflictType::InsertDeleteConflict {
                    insert_op: ConflictingOperation {
                        operation: op2.clone(),
                        author: author2,
                        timestamp: timestamp2,
                        affected_region: region2,
                    },
                    delete_op: ConflictingOperation {
                        operation: op1.clone(),
                        author: author1,
                        timestamp: timestamp1,
                        affected_region: region1,
                    },
                };
            } else {
                return ConflictType::ConcurrentEdit {
                    region: region1.union(&region2),
                    operations: vec![
                        ConflictingOperation {
                            operation: op1.clone(),
                            author: author1,
                            timestamp: timestamp1,
                            affected_region: region1,
                        },
                        ConflictingOperation {
                            operation: op2.clone(),
                            author: author2,
                            timestamp: timestamp2,
                            affected_region: region2,
                        },
                    ],
                };
            }
        }

        ConflictType::None
    }

    /// Detect conflicts using version vectors
    pub fn detect_causal_conflict(
        &self,
        local_vv: &VersionVector,
        remote_vv: &VersionVector,
    ) -> Option<ConflictType> {
        if local_vv.concurrent(remote_vv) {
            Some(ConflictType::CausalConflict {
                local_version: local_vv.clone(),
                remote_version: remote_vv.clone(),
            })
        } else {
            None
        }
    }

    /// Get the region affected by an operation
    fn get_affected_region(operation: &Operation) -> TextRegion {
        let mut start = 0;
        let mut end = 0;
        let mut position = 0;

        let mut first_modification = true;

        for component in operation.components() {
            match component {
                OpComponent::Retain(n) => {
                    position += n;
                }
                OpComponent::Insert(s) => {
                    let len = s.chars().count();
                    if first_modification {
                        start = position;
                        first_modification = false;
                    }
                    end = position + len;
                    position += len;
                }
                OpComponent::Delete(n) => {
                    if first_modification {
                        start = position;
                        first_modification = false;
                    }
                    end = position + n;
                }
            }
        }

        if first_modification {
            // No modifications, return empty region at end
            TextRegion::new(position, position)
        } else {
            TextRegion::new(start, end)
        }
    }

    /// Check if operation has delete component
    fn has_delete(operation: &Operation) -> bool {
        operation.components().iter().any(|c| matches!(c, OpComponent::Delete(_)))
    }

    /// Check if operation has insert component
    fn has_insert(operation: &Operation) -> bool {
        operation.components().iter().any(|c| matches!(c, OpComponent::Insert(_)))
    }

    /// Clear old history
    pub fn cleanup_old_history(&mut self, threshold: chrono::Duration) {
        let cutoff = Utc::now() - threshold;

        for history in self.operation_history.values_mut() {
            history.retain(|record| record.timestamp > cutoff);
        }

        self.operation_history.retain(|_, history| !history.is_empty());
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Last write wins (based on timestamp)
    LastWriteWins,

    /// First write wins
    FirstWriteWins,

    /// Prefer specific author
    PreferAuthor(ReplicaId),

    /// Manual resolution required
    Manual,

    /// Use Operational Transform
    OperationalTransform,

    /// Merge both changes
    Merge,
}

/// Conflict resolver
#[derive(Debug)]
pub struct ConflictResolver {
    default_strategy: ResolutionStrategy,
}

impl ConflictResolver {
    /// Create a new resolver with default strategy
    pub fn new(strategy: ResolutionStrategy) -> Self {
        Self {
            default_strategy: strategy,
        }
    }

    /// Resolve a conflict
    pub fn resolve(
        &self,
        conflict: &ConflictType,
        strategy: Option<ResolutionStrategy>,
    ) -> ResolutionResult {
        let strategy = strategy.unwrap_or(self.default_strategy);

        match conflict {
            ConflictType::None => ResolutionResult::NoConflict,

            ConflictType::ConcurrentEdit { operations, .. } => {
                self.resolve_concurrent_edit(operations, strategy)
            }

            ConflictType::InsertDeleteConflict { insert_op, delete_op } => {
                self.resolve_insert_delete(insert_op, delete_op, strategy)
            }

            ConflictType::OverlappingDelete { deletions } => {
                self.resolve_overlapping_delete(deletions, strategy)
            }

            ConflictType::CausalConflict { .. } => {
                ResolutionResult::RequiresManualResolution
            }
        }
    }

    fn resolve_concurrent_edit(
        &self,
        operations: &[ConflictingOperation],
        strategy: ResolutionStrategy,
    ) -> ResolutionResult {
        match strategy {
            ResolutionStrategy::LastWriteWins => {
                let winner = operations
                    .iter()
                    .max_by_key(|op| op.timestamp)
                    .unwrap();
                ResolutionResult::Resolved {
                    operation: winner.operation.clone(),
                    strategy,
                }
            }

            ResolutionStrategy::FirstWriteWins => {
                let winner = operations
                    .iter()
                    .min_by_key(|op| op.timestamp)
                    .unwrap();
                ResolutionResult::Resolved {
                    operation: winner.operation.clone(),
                    strategy,
                }
            }

            ResolutionStrategy::PreferAuthor(author) => {
                if let Some(op) = operations.iter().find(|op| op.author == author) {
                    ResolutionResult::Resolved {
                        operation: op.operation.clone(),
                        strategy,
                    }
                } else {
                    ResolutionResult::RequiresManualResolution
                }
            }

            ResolutionStrategy::OperationalTransform => {
                // OT automatically handles this
                ResolutionResult::UseOperationalTransform
            }

            ResolutionStrategy::Manual => {
                ResolutionResult::RequiresManualResolution
            }

            ResolutionStrategy::Merge => {
                // Attempt to merge operations
                ResolutionResult::MergeBoth
            }
        }
    }

    fn resolve_insert_delete(
        &self,
        insert_op: &ConflictingOperation,
        delete_op: &ConflictingOperation,
        strategy: ResolutionStrategy,
    ) -> ResolutionResult {
        match strategy {
            ResolutionStrategy::LastWriteWins => {
                if insert_op.timestamp > delete_op.timestamp {
                    ResolutionResult::Resolved {
                        operation: insert_op.operation.clone(),
                        strategy,
                    }
                } else {
                    ResolutionResult::Resolved {
                        operation: delete_op.operation.clone(),
                        strategy,
                    }
                }
            }

            _ => ResolutionResult::UseOperationalTransform,
        }
    }

    fn resolve_overlapping_delete(
        &self,
        deletions: &[ConflictingOperation],
        strategy: ResolutionStrategy,
    ) -> ResolutionResult {
        match strategy {
            ResolutionStrategy::LastWriteWins => {
                let winner = deletions
                    .iter()
                    .max_by_key(|op| op.timestamp)
                    .unwrap();
                ResolutionResult::Resolved {
                    operation: winner.operation.clone(),
                    strategy,
                }
            }

            _ => ResolutionResult::UseOperationalTransform,
        }
    }
}

/// Resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionResult {
    /// No conflict to resolve
    NoConflict,

    /// Conflict resolved with winning operation
    Resolved {
        operation: Operation,
        strategy: ResolutionStrategy,
    },

    /// Use OT to handle conflict
    UseOperationalTransform,

    /// Merge both operations
    MergeBoth,

    /// Manual resolution required
    RequiresManualResolution,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_region() {
        let region1 = TextRegion::new(5, 10);
        let region2 = TextRegion::new(8, 15);

        assert!(region1.overlaps(&region2));
        assert_eq!(region1.len(), 5);

        let intersection = region1.intersection(&region2).unwrap();
        assert_eq!(intersection.start, 8);
        assert_eq!(intersection.end, 10);

        let union = region1.union(&region2);
        assert_eq!(union.start, 5);
        assert_eq!(union.end, 15);
    }

    #[test]
    fn test_conflict_detector() {
        let mut detector = ConflictDetector::new();

        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut op1 = Operation::new();
        op1.retain(5).insert("X");

        let mut op2 = Operation::new();
        op2.retain(5).insert("Y");

        let conflict = detector.detect_conflict(
            &op1,
            replica1,
            Utc::now(),
            &op2,
            replica2,
            Utc::now(),
        );

        match conflict {
            ConflictType::ConcurrentEdit { .. } => {}
            _ => panic!("Expected concurrent edit conflict"),
        }
    }

    #[test]
    fn test_no_conflict() {
        let detector = ConflictDetector::new();

        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut op1 = Operation::new();
        op1.retain(5).insert("X");

        let mut op2 = Operation::new();
        op2.retain(10).insert("Y");

        let conflict = detector.detect_conflict(
            &op1,
            replica1,
            Utc::now(),
            &op2,
            replica2,
            Utc::now(),
        );

        assert_eq!(conflict, ConflictType::None);
    }

    #[test]
    fn test_conflict_resolver() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWriteWins);

        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut op1 = Operation::new();
        op1.insert("A");

        let mut op2 = Operation::new();
        op2.insert("B");

        let timestamp1 = Utc::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let timestamp2 = Utc::now();

        let conflict = ConflictType::ConcurrentEdit {
            region: TextRegion::new(0, 1),
            operations: vec![
                ConflictingOperation {
                    operation: op1,
                    author: replica1,
                    timestamp: timestamp1,
                    affected_region: TextRegion::new(0, 1),
                },
                ConflictingOperation {
                    operation: op2.clone(),
                    author: replica2,
                    timestamp: timestamp2,
                    affected_region: TextRegion::new(0, 1),
                },
            ],
        };

        let result = resolver.resolve(&conflict, None);

        match result {
            ResolutionResult::Resolved { operation, .. } => {
                assert_eq!(operation, op2); // Later timestamp wins
            }
            _ => panic!("Expected resolved conflict"),
        }
    }

    #[test]
    fn test_insert_delete_conflict() {
        let detector = ConflictDetector::new();

        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut op1 = Operation::new();
        op1.retain(5).insert("X");

        let mut op2 = Operation::new();
        op2.retain(5).delete(1);

        let conflict = detector.detect_conflict(
            &op1,
            replica1,
            Utc::now(),
            &op2,
            replica2,
            Utc::now(),
        );

        match conflict {
            ConflictType::InsertDeleteConflict { .. } => {}
            _ => panic!("Expected insert-delete conflict"),
        }
    }

    #[test]
    fn test_causal_conflict() {
        let detector = ConflictDetector::new();

        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut vv1 = VersionVector::new();
        vv1.increment(replica1);

        let mut vv2 = VersionVector::new();
        vv2.increment(replica2);

        let conflict = detector.detect_causal_conflict(&vv1, &vv2);
        assert!(conflict.is_some());
    }
}
