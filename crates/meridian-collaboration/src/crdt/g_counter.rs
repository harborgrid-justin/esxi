//! # Grow-Only Counter (G-Counter)
//!
//! A state-based CRDT that implements a counter that can only be incremented.
//! Each replica maintains its own counter value, and the total is the sum of all replicas.
//!
//! ## Properties
//! - **Monotonic**: Value can only increase
//! - **Convergent**: All replicas converge to the same total
//! - **Use Cases**: Like counters, view counts, vote tallies

use super::{CrdtValue, CvRDT, ReplicaId};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

/// Grow-Only Counter implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GCounter {
    /// Per-replica counters
    counts: HashMap<ReplicaId, u64>,
    /// This replica's ID
    replica_id: ReplicaId,
}

impl GCounter {
    /// Create a new G-Counter
    pub fn new(replica_id: ReplicaId) -> Self {
        let mut counts = HashMap::new();
        counts.insert(replica_id, 0);
        Self { counts, replica_id }
    }

    /// Increment the counter by 1
    pub fn increment(&mut self) {
        self.increment_by(1);
    }

    /// Increment the counter by a specific amount
    pub fn increment_by(&mut self, delta: u64) {
        *self.counts.entry(self.replica_id).or_insert(0) += delta;
    }

    /// Get the current total value (sum of all replica counters)
    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Get the count for a specific replica
    pub fn get_replica_count(&self, replica_id: &ReplicaId) -> u64 {
        self.counts.get(replica_id).copied().unwrap_or(0)
    }

    /// Get all replica counts
    pub fn replica_counts(&self) -> &HashMap<ReplicaId, u64> {
        &self.counts
    }

    /// Get the number of replicas that have incremented this counter
    pub fn replica_count(&self) -> usize {
        self.counts.len()
    }

    /// Check if this counter is empty (all zeros)
    pub fn is_empty(&self) -> bool {
        self.value() == 0
    }
}

impl CvRDT for GCounter {
    fn merge(&mut self, other: &Self) {
        for (replica_id, count) in &other.counts {
            let entry = self.counts.entry(*replica_id).or_insert(0);
            *entry = (*entry).max(*count);
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut less_or_equal = true;
        let mut greater_or_equal = true;

        // Check all replicas in both counters
        let all_replicas: std::collections::HashSet<_> = self
            .counts
            .keys()
            .chain(other.counts.keys())
            .collect();

        for replica_id in all_replicas {
            let self_count = self.get_replica_count(replica_id);
            let other_count = other.get_replica_count(replica_id);

            if self_count < other_count {
                greater_or_equal = false;
            }
            if self_count > other_count {
                less_or_equal = false;
            }
        }

        if less_or_equal && greater_or_equal {
            Some(Ordering::Equal)
        } else if less_or_equal {
            Some(Ordering::Less)
        } else if greater_or_equal {
            Some(Ordering::Greater)
        } else {
            None // Concurrent/incomparable
        }
    }
}

impl CrdtValue for GCounter {
    type Value = u64;

    fn value(&self) -> Self::Value {
        self.value()
    }
}

impl Default for GCounter {
    fn default() -> Self {
        Self::new(ReplicaId::new())
    }
}

/// Operation for G-Counter (operation-based variant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GCounterOp {
    pub replica_id: ReplicaId,
    pub delta: u64,
}

impl GCounter {
    /// Apply an operation to the counter
    pub fn apply_op(&mut self, op: GCounterOp) {
        *self.counts.entry(op.replica_id).or_insert(0) += op.delta;
    }

    /// Create an increment operation
    pub fn create_increment_op(&self, delta: u64) -> GCounterOp {
        GCounterOp {
            replica_id: self.replica_id,
            delta,
        }
    }
}

/// Delta-state for efficient G-Counter synchronization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GCounterDelta {
    pub counts: HashMap<ReplicaId, u64>,
}

impl GCounter {
    /// Generate a delta containing only changes since a given state
    pub fn delta_since(&self, other: &GCounter) -> GCounterDelta {
        let mut delta_counts = HashMap::new();

        for (replica_id, count) in &self.counts {
            let other_count = other.get_replica_count(replica_id);
            if *count > other_count {
                delta_counts.insert(*replica_id, *count);
            }
        }

        GCounterDelta { counts: delta_counts }
    }

    /// Merge a delta into this counter
    pub fn merge_delta(&mut self, delta: GCounterDelta) {
        for (replica_id, count) in delta.counts {
            let entry = self.counts.entry(replica_id).or_insert(0);
            *entry = (*entry).max(count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcounter_basic() {
        let replica = ReplicaId::new();
        let mut counter = GCounter::new(replica);

        assert_eq!(counter.value(), 0);
        assert!(counter.is_empty());

        counter.increment();
        assert_eq!(counter.value(), 1);
        assert!(!counter.is_empty());

        counter.increment_by(5);
        assert_eq!(counter.value(), 6);
    }

    #[test]
    fn test_gcounter_merge() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut counter1 = GCounter::new(replica1);
        let mut counter2 = GCounter::new(replica2);

        counter1.increment_by(3);
        counter2.increment_by(5);

        counter1.merge(&counter2);
        assert_eq!(counter1.value(), 8);

        counter2.merge(&counter1);
        assert_eq!(counter2.value(), 8);
    }

    #[test]
    fn test_gcounter_idempotent_merge() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut counter1 = GCounter::new(replica1);
        let mut counter2 = GCounter::new(replica2);

        counter1.increment_by(3);
        counter2.increment_by(5);

        counter1.merge(&counter2);
        let value1 = counter1.value();

        // Merging again should not change the value
        counter1.merge(&counter2);
        assert_eq!(counter1.value(), value1);
    }

    #[test]
    fn test_gcounter_partial_order() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut counter1 = GCounter::new(replica1);
        let mut counter2 = GCounter::new(replica1);

        counter1.increment_by(5);
        counter2.increment_by(3);

        assert_eq!(counter1.partial_cmp(&counter2), Some(Ordering::Greater));
        assert_eq!(counter2.partial_cmp(&counter1), Some(Ordering::Less));

        // Concurrent counters
        let mut counter3 = GCounter::new(replica2);
        counter3.increment_by(10);

        // counter1 and counter3 are concurrent
        assert_eq!(counter1.partial_cmp(&counter3), None);
    }

    #[test]
    fn test_gcounter_operations() {
        let replica = ReplicaId::new();
        let mut counter = GCounter::new(replica);

        let op = counter.create_increment_op(5);
        counter.apply_op(op);

        assert_eq!(counter.value(), 5);
    }

    #[test]
    fn test_gcounter_delta() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut counter1 = GCounter::new(replica1);
        let mut counter2 = GCounter::new(replica2);

        counter1.increment_by(3);
        counter2.increment_by(5);

        // Get delta from counter1's perspective
        let delta = counter2.delta_since(&counter1);
        counter1.merge_delta(delta);

        assert_eq!(counter1.value(), 8);
    }

    #[test]
    fn test_gcounter_multiple_replicas() {
        let replicas: Vec<_> = (0..5).map(|_| ReplicaId::new()).collect();
        let mut counters: Vec<_> = replicas.iter().map(|r| GCounter::new(*r)).collect();

        // Each replica increments by a different amount
        for (i, counter) in counters.iter_mut().enumerate() {
            counter.increment_by((i + 1) as u64);
        }

        // Merge all counters into the first one
        let mut merged = counters[0].clone();
        for counter in &counters[1..] {
            merged.merge(counter);
        }

        // Total should be 1 + 2 + 3 + 4 + 5 = 15
        assert_eq!(merged.value(), 15);
        assert_eq!(merged.replica_count(), 5);
    }
}
