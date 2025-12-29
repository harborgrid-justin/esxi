//! Delta synchronization for efficient state updates

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::state::StateId;
use crate::sync::VectorClock;

/// Delta operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeltaOperation {
    /// Insert bytes at position
    Insert { position: usize, data: Vec<u8> },

    /// Delete bytes at position
    Delete { position: usize, length: usize },

    /// Replace bytes at position
    Replace {
        position: usize,
        length: usize,
        data: Vec<u8>,
    },

    /// Patch using binary diff
    Patch { patch: Vec<u8> },
}

/// State delta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    /// Delta ID
    id: String,

    /// State ID
    state_id: StateId,

    /// Base version (version this delta applies to)
    base_version: u64,

    /// Target version (version after applying delta)
    target_version: u64,

    /// Operations
    operations: Vec<DeltaOperation>,

    /// Vector clock
    clock: VectorClock,

    /// Created timestamp
    created_at: i64,

    /// Author (user or node ID)
    author: String,
}

impl Delta {
    /// Create new delta
    pub fn new(
        state_id: StateId,
        base_version: u64,
        target_version: u64,
        operations: Vec<DeltaOperation>,
        clock: VectorClock,
        author: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            state_id,
            base_version,
            target_version,
            operations,
            clock,
            created_at: chrono::Utc::now().timestamp_millis(),
            author,
        }
    }

    /// Get delta ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get state ID
    pub fn state_id(&self) -> &str {
        &self.state_id
    }

    /// Get base version
    pub fn base_version(&self) -> u64 {
        self.base_version
    }

    /// Get target version
    pub fn target_version(&self) -> u64 {
        self.target_version
    }

    /// Get operations
    pub fn operations(&self) -> &[DeltaOperation] {
        &self.operations
    }

    /// Get vector clock
    pub fn clock(&self) -> &VectorClock {
        &self.clock
    }

    /// Get author
    pub fn author(&self) -> &str {
        &self.author
    }

    /// Apply delta to state
    pub fn apply(&self, state: &[u8]) -> Option<Vec<u8>> {
        let mut result = state.to_vec();

        for operation in &self.operations {
            match operation {
                DeltaOperation::Insert { position, data } => {
                    if *position > result.len() {
                        return None;
                    }
                    result.splice(*position..*position, data.iter().cloned());
                }

                DeltaOperation::Delete { position, length } => {
                    let end = (*position + *length).min(result.len());
                    if *position > result.len() {
                        return None;
                    }
                    result.drain(*position..end);
                }

                DeltaOperation::Replace {
                    position,
                    length,
                    data,
                } => {
                    let end = (*position + *length).min(result.len());
                    if *position > result.len() {
                        return None;
                    }
                    result.splice(*position..end, data.iter().cloned());
                }

                DeltaOperation::Patch { patch } => {
                    // For simplicity, treat patch as full replacement
                    // In production, use a proper binary diff library
                    result = patch.clone();
                }
            }
        }

        Some(result)
    }

    /// Get size of delta
    pub fn size(&self) -> usize {
        self.operations
            .iter()
            .map(|op| match op {
                DeltaOperation::Insert { data, .. } => data.len(),
                DeltaOperation::Replace { data, .. } => data.len(),
                DeltaOperation::Patch { patch } => patch.len(),
                _ => 0,
            })
            .sum()
    }
}

/// Delta manager
pub struct DeltaManager {
    /// Deltas by ID
    deltas: Arc<DashMap<String, Arc<Delta>>>,

    /// Deltas by state ID
    state_deltas: Arc<DashMap<StateId, Vec<Arc<Delta>>>>,

    /// Maximum deltas to keep
    max_deltas: usize,
}

impl DeltaManager {
    /// Create new delta manager
    pub fn new(max_deltas: usize) -> Self {
        Self {
            deltas: Arc::new(DashMap::new()),
            state_deltas: Arc::new(DashMap::new()),
            max_deltas,
        }
    }

    /// Add delta
    pub fn add_delta(&self, delta: Delta) -> Result<()> {
        let delta_id = delta.id().to_string();
        let state_id = delta.state_id().to_string();
        let delta = Arc::new(delta);

        // Add to deltas map
        self.deltas.insert(delta_id.clone(), delta.clone());

        // Add to state deltas map
        let mut state_dels = self.state_deltas.entry(state_id.clone()).or_insert_with(Vec::new);
        state_dels.push(delta.clone());

        // Sort by base version
        state_dels.sort_by_key(|d| d.base_version());

        // Limit deltas per state
        if state_dels.len() > self.max_deltas {
            let excess = state_dels.len() - self.max_deltas;
            let removed: Vec<_> = state_dels.drain(..excess).collect();

            // Remove from deltas map
            for delta in removed {
                self.deltas.remove(delta.id());
            }
        }

        Ok(())
    }

    /// Get delta by ID
    pub fn get_delta(&self, id: &str) -> Option<Arc<Delta>> {
        self.deltas.get(id).map(|d| d.value().clone())
    }

    /// Get deltas for state
    pub fn get_deltas_for_state(&self, state_id: &str) -> Vec<Arc<Delta>> {
        self.state_deltas
            .get(state_id)
            .map(|dels| dels.clone())
            .unwrap_or_default()
    }

    /// Get deltas in version range
    pub fn get_deltas_in_range(
        &self,
        state_id: &str,
        from_version: u64,
        to_version: u64,
    ) -> Vec<Arc<Delta>> {
        self.state_deltas
            .get(state_id)
            .map(|dels| {
                dels.iter()
                    .filter(|d| {
                        d.base_version() >= from_version && d.target_version() <= to_version
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Remove delta
    pub fn remove_delta(&self, id: &str) -> Option<Arc<Delta>> {
        if let Some((_, delta)) = self.deltas.remove(id) {
            // Remove from state deltas
            if let Some(mut dels) = self.state_deltas.get_mut(delta.state_id()) {
                dels.retain(|d| d.id() != id);
            }

            return Some(delta);
        }

        None
    }

    /// Remove all deltas for state
    pub fn remove_state_deltas(&self, state_id: &str) {
        if let Some((_, dels)) = self.state_deltas.remove(state_id) {
            for delta in dels {
                self.deltas.remove(delta.id());
            }
        }
    }

    /// Get delta count
    pub fn delta_count(&self) -> usize {
        self.deltas.len()
    }

    /// Get delta count for state
    pub fn delta_count_for_state(&self, state_id: &str) -> usize {
        self.state_deltas
            .get(state_id)
            .map(|dels| dels.len())
            .unwrap_or(0)
    }

    /// Get total size of all deltas
    pub fn total_size(&self) -> usize {
        self.deltas.iter().map(|d| d.value().size()).sum()
    }

    /// Clear all deltas
    pub fn clear(&self) {
        self.deltas.clear();
        self.state_deltas.clear();
    }
}

/// Delta compressor for combining multiple deltas
pub struct DeltaCompressor;

impl DeltaCompressor {
    /// Compress multiple deltas into one
    pub fn compress(deltas: &[Delta]) -> Option<Delta> {
        if deltas.is_empty() {
            return None;
        }

        let first = &deltas[0];
        let last = &deltas[deltas.len() - 1];

        // Combine all operations
        let mut operations = Vec::new();
        for delta in deltas {
            operations.extend(delta.operations().iter().cloned());
        }

        // Merge vector clocks
        let mut clock = first.clock().clone();
        for delta in &deltas[1..] {
            clock.merge(delta.clock());
        }

        Some(Delta::new(
            first.state_id().to_string(),
            first.base_version(),
            last.target_version(),
            operations,
            clock,
            first.author().to_string(),
        ))
    }

    /// Optimize operations (remove redundant operations)
    pub fn optimize_operations(operations: Vec<DeltaOperation>) -> Vec<DeltaOperation> {
        // Simplified optimization - just return as-is
        // A real implementation would merge consecutive operations,
        // remove operations that cancel each other out, etc.
        operations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_insert() {
        let clock = VectorClock::new("node1".to_string());
        let operations = vec![DeltaOperation::Insert {
            position: 2,
            data: vec![99, 100],
        }];

        let delta = Delta::new(
            "state1".to_string(),
            1,
            2,
            operations,
            clock,
            "user1".to_string(),
        );

        let state = vec![1, 2, 3, 4, 5];
        let result = delta.apply(&state).unwrap();

        assert_eq!(result, vec![1, 2, 99, 100, 3, 4, 5]);
    }

    #[test]
    fn test_delta_delete() {
        let clock = VectorClock::new("node1".to_string());
        let operations = vec![DeltaOperation::Delete {
            position: 1,
            length: 2,
        }];

        let delta = Delta::new(
            "state1".to_string(),
            1,
            2,
            operations,
            clock,
            "user1".to_string(),
        );

        let state = vec![1, 2, 3, 4, 5];
        let result = delta.apply(&state).unwrap();

        assert_eq!(result, vec![1, 4, 5]);
    }

    #[test]
    fn test_delta_replace() {
        let clock = VectorClock::new("node1".to_string());
        let operations = vec![DeltaOperation::Replace {
            position: 1,
            length: 2,
            data: vec![99, 100, 101],
        }];

        let delta = Delta::new(
            "state1".to_string(),
            1,
            2,
            operations,
            clock,
            "user1".to_string(),
        );

        let state = vec![1, 2, 3, 4, 5];
        let result = delta.apply(&state).unwrap();

        assert_eq!(result, vec![1, 99, 100, 101, 4, 5]);
    }

    #[test]
    fn test_delta_manager() {
        let manager = DeltaManager::new(100);
        assert_eq!(manager.delta_count(), 0);

        let clock = VectorClock::new("node1".to_string());
        let delta = Delta::new(
            "state1".to_string(),
            1,
            2,
            vec![DeltaOperation::Insert {
                position: 0,
                data: vec![1],
            }],
            clock,
            "user1".to_string(),
        );

        let delta_id = delta.id().to_string();
        manager.add_delta(delta).unwrap();

        assert_eq!(manager.delta_count(), 1);
        assert_eq!(manager.delta_count_for_state("state1"), 1);

        let retrieved = manager.get_delta(&delta_id).unwrap();
        assert_eq!(retrieved.base_version(), 1);
        assert_eq!(retrieved.target_version(), 2);
    }

    #[test]
    fn test_delta_compression() {
        let clock = VectorClock::new("node1".to_string());

        let delta1 = Delta::new(
            "state1".to_string(),
            1,
            2,
            vec![DeltaOperation::Insert {
                position: 0,
                data: vec![1],
            }],
            clock.clone(),
            "user1".to_string(),
        );

        let delta2 = Delta::new(
            "state1".to_string(),
            2,
            3,
            vec![DeltaOperation::Insert {
                position: 1,
                data: vec![2],
            }],
            clock,
            "user1".to_string(),
        );

        let compressed = DeltaCompressor::compress(&[delta1, delta2]).unwrap();

        assert_eq!(compressed.base_version(), 1);
        assert_eq!(compressed.target_version(), 3);
        assert_eq!(compressed.operations().len(), 2);
    }
}
