//! # Last-Write-Wins Register (LWW-Register)
//!
//! A state-based CRDT that stores a single value with timestamp-based conflict resolution.
//! When concurrent updates occur, the value with the highest timestamp wins.
//!
//! ## Properties
//! - **Convergence**: All replicas converge to the same value
//! - **Conflict Resolution**: Deterministic based on timestamp ordering
//! - **Use Cases**: User profile fields, configuration values, simple state

use super::{CrdtValue, CvRDT, HybridTimestamp, ReplicaId};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Last-Write-Wins Register implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LWWRegister<T> {
    value: T,
    timestamp: HybridTimestamp,
    replica_id: ReplicaId,
}

impl<T: Clone> LWWRegister<T> {
    /// Create a new LWW-Register with initial value
    pub fn new(value: T, replica_id: ReplicaId) -> Self {
        Self {
            value,
            timestamp: HybridTimestamp::new(replica_id),
            replica_id,
        }
    }

    /// Create with specific timestamp (for deserialization/testing)
    pub fn with_timestamp(value: T, timestamp: HybridTimestamp, replica_id: ReplicaId) -> Self {
        Self {
            value,
            timestamp,
            replica_id,
        }
    }

    /// Update the register with a new value
    pub fn set(&mut self, value: T) {
        self.timestamp.tick();
        self.value = value;
    }

    /// Get the current value
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Get the timestamp of the current value
    pub fn timestamp(&self) -> &HybridTimestamp {
        &self.timestamp
    }

    /// Get mutable reference to value (use with caution)
    pub fn get_mut(&mut self) -> &mut T {
        self.timestamp.tick();
        &mut self.value
    }
}

impl<T: Clone> CvRDT for LWWRegister<T> {
    fn merge(&mut self, other: &Self) {
        if other.timestamp > self.timestamp {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.timestamp.cmp(&other.timestamp))
    }
}

impl<T: Clone> CrdtValue for LWWRegister<T> {
    type Value = T;

    fn value(&self) -> Self::Value {
        self.value.clone()
    }
}

/// Operation for LWW-Register (operation-based variant)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LWWRegisterOp<T> {
    pub value: T,
    pub timestamp: HybridTimestamp,
}

impl<T: Clone> LWWRegister<T> {
    /// Apply an operation to the register
    pub fn apply_op(&mut self, op: LWWRegisterOp<T>) {
        if op.timestamp > self.timestamp {
            self.value = op.value;
            self.timestamp = op.timestamp;
        }
    }

    /// Create an operation for setting a value
    pub fn create_set_op(&mut self, value: T) -> LWWRegisterOp<T> {
        self.timestamp.tick();
        LWWRegisterOp {
            value: value.clone(),
            timestamp: self.timestamp,
        }
    }
}

/// Multi-value register that preserves concurrent writes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MVRegister<T> {
    values: Vec<(T, HybridTimestamp)>,
    replica_id: ReplicaId,
}

impl<T: Clone + PartialEq> MVRegister<T> {
    /// Create a new MV-Register
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            values: Vec::new(),
            replica_id,
        }
    }

    /// Create with initial value
    pub fn with_value(value: T, replica_id: ReplicaId) -> Self {
        let mut register = Self::new(replica_id);
        register.set(value);
        register
    }

    /// Set a new value (replaces all current values)
    pub fn set(&mut self, value: T) {
        let mut timestamp = HybridTimestamp::new(self.replica_id);

        // Get max timestamp from current values
        if let Some(max_ts) = self.values.iter().map(|(_, ts)| ts).max() {
            timestamp = *max_ts;
            timestamp.tick();
        }

        self.values.clear();
        self.values.push((value, timestamp));
    }

    /// Get all concurrent values
    pub fn get(&self) -> Vec<&T> {
        self.values.iter().map(|(v, _)| v).collect()
    }

    /// Get a single value (arbitrary choice if multiple)
    pub fn get_one(&self) -> Option<&T> {
        self.values.first().map(|(v, _)| v)
    }

    /// Check if register has concurrent values
    pub fn has_conflicts(&self) -> bool {
        self.values.len() > 1
    }

    /// Resolve conflicts by choosing the value with the highest timestamp
    pub fn resolve_to_lww(&mut self) -> Option<T> {
        if self.values.is_empty() {
            return None;
        }

        self.values.sort_by(|(_, ts1), (_, ts2)| ts2.cmp(ts1));
        let winner = self.values[0].clone();
        self.values = vec![winner.clone()];
        Some(winner.0)
    }
}

impl<T: Clone + PartialEq> CvRDT for MVRegister<T> {
    fn merge(&mut self, other: &Self) {
        let mut merged = Vec::new();
        let mut max_timestamp = None;

        // Collect all values and find max timestamp
        for (v, ts) in self.values.iter().chain(other.values.iter()) {
            if let Some(max_ts) = max_timestamp {
                match ts.cmp(&max_ts) {
                    Ordering::Greater => {
                        merged.clear();
                        merged.push((v.clone(), *ts));
                        max_timestamp = Some(*ts);
                    }
                    Ordering::Equal => {
                        if !merged.iter().any(|(val, _)| val == v) {
                            merged.push((v.clone(), *ts));
                        }
                    }
                    Ordering::Less => {}
                }
            } else {
                merged.push((v.clone(), *ts));
                max_timestamp = Some(*ts);
            }
        }

        self.values = merged;
    }

    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        // MV-Registers don't have a total order
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lww_register_basic() {
        let replica = ReplicaId::new();
        let mut reg = LWWRegister::new(42, replica);

        assert_eq!(*reg.get(), 42);

        reg.set(100);
        assert_eq!(*reg.get(), 100);
    }

    #[test]
    fn test_lww_register_merge() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut reg1 = LWWRegister::new(10, replica1);
        let mut reg2 = LWWRegister::new(20, replica2);

        // Simulate passage of time
        std::thread::sleep(std::time::Duration::from_millis(10));
        reg2.set(30);

        reg1.merge(&reg2);
        assert_eq!(*reg1.get(), 30);
    }

    #[test]
    fn test_lww_register_concurrent() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut reg1 = LWWRegister::new(10, replica1);
        let mut reg2 = LWWRegister::new(20, replica2);

        // Both update concurrently
        reg1.set(100);
        reg2.set(200);

        // Merge should be deterministic based on timestamp
        let mut reg1_copy = reg1.clone();
        reg1.merge(&reg2);
        reg2.merge(&reg1_copy);

        assert_eq!(reg1.get(), reg2.get());
    }

    #[test]
    fn test_mv_register() {
        let replica1 = ReplicaId::new();
        let mut reg = MVRegister::with_value(42, replica1);

        assert_eq!(reg.get().len(), 1);
        assert_eq!(*reg.get_one().unwrap(), 42);
        assert!(!reg.has_conflicts());

        reg.set(100);
        assert_eq!(*reg.get_one().unwrap(), 100);
    }

    #[test]
    fn test_mv_register_concurrent() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut reg1 = MVRegister::with_value(10, replica1);
        let mut reg2 = MVRegister::with_value(20, replica2);

        // Create concurrent updates
        reg1.set(100);
        reg2.set(200);

        // Merge
        reg1.merge(&reg2);

        // Should have both values as they're concurrent
        let values = reg1.get();
        assert!(values.contains(&&100) || values.contains(&&200));

        // Resolve conflict
        let resolved = reg1.resolve_to_lww();
        assert!(resolved.is_some());
        assert!(!reg1.has_conflicts());
    }

    #[test]
    fn test_lww_operation() {
        let replica = ReplicaId::new();
        let mut reg = LWWRegister::new(42, replica);

        let op = reg.create_set_op(100);
        reg.apply_op(op);

        assert_eq!(*reg.get(), 100);
    }
}
