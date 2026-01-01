//! # Positive-Negative Counter (PN-Counter)
//!
//! A state-based CRDT that implements a counter that can be both incremented and decremented.
//! Composed of two G-Counters: one for increments and one for decrements.
//!
//! ## Properties
//! - **Convergent**: All replicas converge to the same value
//! - **Bidirectional**: Supports both increment and decrement
//! - **Use Cases**: Inventory counts, reputation scores, vote totals (with downvotes)

use super::{CrdtValue, CvRDT, GCounter, ReplicaId};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Positive-Negative Counter implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PNCounter {
    /// Counter for increments
    increments: GCounter,
    /// Counter for decrements
    decrements: GCounter,
    /// This replica's ID
    replica_id: ReplicaId,
}

impl PNCounter {
    /// Create a new PN-Counter
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            increments: GCounter::new(replica_id),
            decrements: GCounter::new(replica_id),
            replica_id,
        }
    }

    /// Increment the counter by 1
    pub fn increment(&mut self) {
        self.increment_by(1);
    }

    /// Increment the counter by a specific amount
    pub fn increment_by(&mut self, delta: u64) {
        self.increments.increment_by(delta);
    }

    /// Decrement the counter by 1
    pub fn decrement(&mut self) {
        self.decrement_by(1);
    }

    /// Decrement the counter by a specific amount
    pub fn decrement_by(&mut self, delta: u64) {
        self.decrements.increment_by(delta);
    }

    /// Get the current value (increments - decrements)
    pub fn value(&self) -> i64 {
        self.increments.value() as i64 - self.decrements.value() as i64
    }

    /// Get the total number of increments
    pub fn total_increments(&self) -> u64 {
        self.increments.value()
    }

    /// Get the total number of decrements
    pub fn total_decrements(&self) -> u64 {
        self.decrements.value()
    }

    /// Get the absolute value of all operations
    pub fn total_operations(&self) -> u64 {
        self.total_increments() + self.total_decrements()
    }

    /// Check if the counter is at zero
    pub fn is_zero(&self) -> bool {
        self.value() == 0
    }

    /// Get a reference to the increment counter
    pub fn increments(&self) -> &GCounter {
        &self.increments
    }

    /// Get a reference to the decrement counter
    pub fn decrements(&self) -> &GCounter {
        &self.decrements
    }
}

impl CvRDT for PNCounter {
    fn merge(&mut self, other: &Self) {
        self.increments.merge(&other.increments);
        self.decrements.merge(&other.decrements);
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let inc_cmp = self.increments.partial_cmp(&other.increments);
        let dec_cmp = self.decrements.partial_cmp(&other.decrements);

        match (inc_cmp, dec_cmp) {
            (Some(Ordering::Equal), Some(Ordering::Equal)) => Some(Ordering::Equal),
            (Some(Ordering::Less), Some(Ordering::Less)) |
            (Some(Ordering::Less), Some(Ordering::Equal)) |
            (Some(Ordering::Equal), Some(Ordering::Less)) => Some(Ordering::Less),
            (Some(Ordering::Greater), Some(Ordering::Greater)) |
            (Some(Ordering::Greater), Some(Ordering::Equal)) |
            (Some(Ordering::Equal), Some(Ordering::Greater)) => Some(Ordering::Greater),
            _ => None, // Concurrent/incomparable
        }
    }
}

impl CrdtValue for PNCounter {
    type Value = i64;

    fn value(&self) -> Self::Value {
        self.value()
    }
}

impl Default for PNCounter {
    fn default() -> Self {
        Self::new(ReplicaId::new())
    }
}

/// Operation for PN-Counter (operation-based variant)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PNCounterOp {
    Increment { replica_id: ReplicaId, delta: u64 },
    Decrement { replica_id: ReplicaId, delta: u64 },
}

impl PNCounter {
    /// Apply an operation to the counter
    pub fn apply_op(&mut self, op: PNCounterOp) {
        match op {
            PNCounterOp::Increment { replica_id, delta } => {
                self.increments.apply_op(super::g_counter::GCounterOp { replica_id, delta });
            }
            PNCounterOp::Decrement { replica_id, delta } => {
                self.decrements.apply_op(super::g_counter::GCounterOp { replica_id, delta });
            }
        }
    }

    /// Create an increment operation
    pub fn create_increment_op(&self, delta: u64) -> PNCounterOp {
        PNCounterOp::Increment {
            replica_id: self.replica_id,
            delta,
        }
    }

    /// Create a decrement operation
    pub fn create_decrement_op(&self, delta: u64) -> PNCounterOp {
        PNCounterOp::Decrement {
            replica_id: self.replica_id,
            delta,
        }
    }
}

/// Delta-state for efficient PN-Counter synchronization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PNCounterDelta {
    pub increments: super::g_counter::GCounterDelta,
    pub decrements: super::g_counter::GCounterDelta,
}

impl PNCounter {
    /// Generate a delta containing only changes since a given state
    pub fn delta_since(&self, other: &PNCounter) -> PNCounterDelta {
        PNCounterDelta {
            increments: self.increments.delta_since(&other.increments),
            decrements: self.decrements.delta_since(&other.decrements),
        }
    }

    /// Merge a delta into this counter
    pub fn merge_delta(&mut self, delta: PNCounterDelta) {
        self.increments.merge_delta(delta.increments);
        self.decrements.merge_delta(delta.decrements);
    }
}

/// Bounded counter that can't go below zero
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedPNCounter {
    counter: PNCounter,
}

impl BoundedPNCounter {
    /// Create a new bounded PN-Counter
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            counter: PNCounter::new(replica_id),
        }
    }

    /// Increment the counter
    pub fn increment(&mut self) {
        self.increment_by(1);
    }

    /// Increment by a specific amount
    pub fn increment_by(&mut self, delta: u64) {
        self.counter.increment_by(delta);
    }

    /// Decrement the counter (only if result would be >= 0)
    pub fn decrement(&mut self) -> bool {
        self.decrement_by(1)
    }

    /// Decrement by a specific amount (only if result would be >= 0)
    pub fn decrement_by(&mut self, delta: u64) -> bool {
        if self.counter.value() >= delta as i64 {
            self.counter.decrement_by(delta);
            true
        } else {
            false
        }
    }

    /// Get the current value (always >= 0)
    pub fn value(&self) -> u64 {
        self.counter.value().max(0) as u64
    }

    /// Get the inner counter
    pub fn inner(&self) -> &PNCounter {
        &self.counter
    }
}

impl CvRDT for BoundedPNCounter {
    fn merge(&mut self, other: &Self) {
        self.counter.merge(&other.counter);
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.counter.partial_cmp(&other.counter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pncounter_basic() {
        let replica = ReplicaId::new();
        let mut counter = PNCounter::new(replica);

        assert_eq!(counter.value(), 0);
        assert!(counter.is_zero());

        counter.increment_by(5);
        assert_eq!(counter.value(), 5);

        counter.decrement_by(3);
        assert_eq!(counter.value(), 2);

        counter.decrement_by(10);
        assert_eq!(counter.value(), -8);
    }

    #[test]
    fn test_pncounter_merge() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut counter1 = PNCounter::new(replica1);
        let mut counter2 = PNCounter::new(replica2);

        counter1.increment_by(10);
        counter1.decrement_by(3);

        counter2.increment_by(5);
        counter2.decrement_by(2);

        counter1.merge(&counter2);
        assert_eq!(counter1.value(), 10); // (10 + 5) - (3 + 2) = 10

        counter2.merge(&counter1);
        assert_eq!(counter2.value(), 10);
    }

    #[test]
    fn test_pncounter_operations() {
        let replica = ReplicaId::new();
        let mut counter = PNCounter::new(replica);

        let inc_op = counter.create_increment_op(5);
        counter.apply_op(inc_op);
        assert_eq!(counter.value(), 5);

        let dec_op = counter.create_decrement_op(2);
        counter.apply_op(dec_op);
        assert_eq!(counter.value(), 3);
    }

    #[test]
    fn test_pncounter_delta() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut counter1 = PNCounter::new(replica1);
        let mut counter2 = PNCounter::new(replica2);

        counter1.increment_by(10);
        counter2.increment_by(5);
        counter2.decrement_by(2);

        let delta = counter2.delta_since(&counter1);
        counter1.merge_delta(delta);

        assert_eq!(counter1.value(), 13); // 10 + 5 - 2
    }

    #[test]
    fn test_pncounter_total_operations() {
        let replica = ReplicaId::new();
        let mut counter = PNCounter::new(replica);

        counter.increment_by(10);
        counter.decrement_by(7);
        counter.increment_by(3);

        assert_eq!(counter.value(), 6);
        assert_eq!(counter.total_operations(), 20);
        assert_eq!(counter.total_increments(), 13);
        assert_eq!(counter.total_decrements(), 7);
    }

    #[test]
    fn test_bounded_pncounter() {
        let replica = ReplicaId::new();
        let mut counter = BoundedPNCounter::new(replica);

        counter.increment_by(5);
        assert_eq!(counter.value(), 5);

        assert!(counter.decrement_by(3));
        assert_eq!(counter.value(), 2);

        // Can't decrement below zero
        assert!(!counter.decrement_by(10));
        assert_eq!(counter.value(), 2);

        assert!(counter.decrement_by(2));
        assert_eq!(counter.value(), 0);
    }

    #[test]
    fn test_pncounter_convergence() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();
        let replica3 = ReplicaId::new();

        let mut counter1 = PNCounter::new(replica1);
        let mut counter2 = PNCounter::new(replica2);
        let mut counter3 = PNCounter::new(replica3);

        // Different operations on each replica
        counter1.increment_by(10);
        counter2.decrement_by(5);
        counter3.increment_by(3);

        // Merge in different orders
        let mut result1 = counter1.clone();
        result1.merge(&counter2);
        result1.merge(&counter3);

        let mut result2 = counter3.clone();
        result2.merge(&counter1);
        result2.merge(&counter2);

        // Should converge to same value regardless of merge order
        assert_eq!(result1.value(), result2.value());
        assert_eq!(result1.value(), 8); // 10 + 3 - 5
    }
}
