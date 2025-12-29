//! Vector clock implementation for causality tracking

use std::collections::HashMap;
use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

/// Vector clock for tracking causality in distributed systems
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VectorClock {
    /// Clock values per node
    clocks: HashMap<String, u64>,

    /// This node's ID
    node_id: String,
}

impl VectorClock {
    /// Create new vector clock
    pub fn new(node_id: String) -> Self {
        let mut clocks = HashMap::new();
        clocks.insert(node_id.clone(), 0);

        Self { clocks, node_id }
    }

    /// Get clock value for a node
    pub fn get(&self, node_id: &str) -> Option<u64> {
        self.clocks.get(node_id).copied()
    }

    /// Increment this node's clock
    pub fn increment(&mut self) {
        *self.clocks.entry(self.node_id.clone()).or_insert(0) += 1;
    }

    /// Update clock for a specific node
    pub fn update(&mut self, node_id: String, value: u64) {
        self.clocks.insert(node_id, value);
    }

    /// Merge with another vector clock (take max of each component)
    pub fn merge(&mut self, other: &VectorClock) {
        for (node_id, &other_value) in &other.clocks {
            self.clocks
                .entry(node_id.clone())
                .and_modify(|our_value| {
                    *our_value = (*our_value).max(other_value);
                })
                .or_insert(other_value);
        }
    }

    /// Check if this clock happened before another
    pub fn happens_before(&self, other: &VectorClock) -> bool {
        let mut strictly_less = false;

        // Get all node IDs from both clocks
        let mut all_nodes: std::collections::HashSet<_> = self.clocks.keys().collect();
        all_nodes.extend(other.clocks.keys());

        for node_id in all_nodes {
            let our_value = self.clocks.get(node_id).copied().unwrap_or(0);
            let other_value = other.clocks.get(node_id).copied().unwrap_or(0);

            if our_value > other_value {
                return false; // Not happened before
            } else if our_value < other_value {
                strictly_less = true;
            }
        }

        strictly_less
    }

    /// Check if this clock is concurrent with another
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        !self.happens_before(other) && !other.happens_before(self) && self != other
    }

    /// Compare with another vector clock
    pub fn compare(&self, other: &VectorClock) -> ClockOrdering {
        if self == other {
            ClockOrdering::Equal
        } else if self.happens_before(other) {
            ClockOrdering::Before
        } else if other.happens_before(self) {
            ClockOrdering::After
        } else {
            ClockOrdering::Concurrent
        }
    }

    /// Get all node IDs
    pub fn node_ids(&self) -> impl Iterator<Item = &String> {
        self.clocks.keys()
    }

    /// Get this node's ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Get current value for this node
    pub fn value(&self) -> u64 {
        self.clocks.get(&self.node_id).copied().unwrap_or(0)
    }

    /// Create a copy with incremented value
    pub fn incremented(&self) -> Self {
        let mut new_clock = self.clone();
        new_clock.increment();
        new_clock
    }

    /// Reset clock
    pub fn reset(&mut self) {
        self.clocks.clear();
        self.clocks.insert(self.node_id.clone(), 0);
    }

    /// Get total events across all nodes
    pub fn total_events(&self) -> u64 {
        self.clocks.values().sum()
    }
}

/// Ordering between vector clocks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockOrdering {
    /// Clocks are equal
    Equal,
    /// This clock is before the other
    Before,
    /// This clock is after the other
    After,
    /// Clocks are concurrent
    Concurrent,
}

impl PartialOrd for VectorClock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.compare(other) {
            ClockOrdering::Equal => Some(Ordering::Equal),
            ClockOrdering::Before => Some(Ordering::Less),
            ClockOrdering::After => Some(Ordering::Greater),
            ClockOrdering::Concurrent => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_creation() {
        let clock = VectorClock::new("node1".to_string());
        assert_eq!(clock.get("node1"), Some(0));
        assert_eq!(clock.value(), 0);
    }

    #[test]
    fn test_vector_clock_increment() {
        let mut clock = VectorClock::new("node1".to_string());
        clock.increment();
        assert_eq!(clock.get("node1"), Some(1));
        clock.increment();
        assert_eq!(clock.get("node1"), Some(2));
    }

    #[test]
    fn test_vector_clock_merge() {
        let mut clock1 = VectorClock::new("node1".to_string());
        clock1.increment();
        clock1.increment();

        let mut clock2 = VectorClock::new("node2".to_string());
        clock2.increment();

        clock1.merge(&clock2);

        assert_eq!(clock1.get("node1"), Some(2));
        assert_eq!(clock1.get("node2"), Some(1));
    }

    #[test]
    fn test_happens_before() {
        let mut clock1 = VectorClock::new("node1".to_string());
        clock1.increment();

        let mut clock2 = clock1.clone();
        clock2.increment();

        assert!(clock1.happens_before(&clock2));
        assert!(!clock2.happens_before(&clock1));
    }

    #[test]
    fn test_concurrent() {
        let mut clock1 = VectorClock::new("node1".to_string());
        clock1.increment();

        let mut clock2 = VectorClock::new("node2".to_string());
        clock2.increment();

        assert!(clock1.is_concurrent(&clock2));
        assert!(clock2.is_concurrent(&clock1));
    }

    #[test]
    fn test_compare() {
        let mut clock1 = VectorClock::new("node1".to_string());
        clock1.increment();

        let clock2 = clock1.clone();
        assert_eq!(clock1.compare(&clock2), ClockOrdering::Equal);

        let mut clock3 = clock1.clone();
        clock3.increment();
        assert_eq!(clock1.compare(&clock3), ClockOrdering::Before);
        assert_eq!(clock3.compare(&clock1), ClockOrdering::After);

        let mut clock4 = VectorClock::new("node2".to_string());
        clock4.increment();
        assert_eq!(clock1.compare(&clock4), ClockOrdering::Concurrent);
    }

    #[test]
    fn test_total_events() {
        let mut clock = VectorClock::new("node1".to_string());
        clock.increment();
        clock.increment();

        let mut clock2 = VectorClock::new("node2".to_string());
        clock2.increment();

        clock.merge(&clock2);
        assert_eq!(clock.total_events(), 3);
    }
}
