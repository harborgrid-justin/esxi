//! # Grow-Only Set (G-Set)
//!
//! A state-based CRDT that implements a set that only supports additions.
//! Elements can be added but never removed.
//!
//! ## Properties
//! - **Monotonic**: Set can only grow
//! - **Convergent**: All replicas converge to the same set
//! - **Use Cases**: Immutable collections, audit logs, tag systems (add-only)

use super::{CrdtValue, CvRDT};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::Hash;

/// Grow-Only Set implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GSet<T: Eq + Hash> {
    elements: HashSet<T>,
}

impl<T: Eq + Hash> GSet<T> {
    /// Create a new empty G-Set
    pub fn new() -> Self {
        Self {
            elements: HashSet::new(),
        }
    }

    /// Create a G-Set from an iterator
    pub fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            elements: iter.into_iter().collect(),
        }
    }

    /// Add an element to the set
    /// Returns true if the element was newly inserted
    pub fn insert(&mut self, element: T) -> bool {
        self.elements.insert(element)
    }

    /// Check if the set contains an element
    pub fn contains(&self, element: &T) -> bool {
        self.elements.contains(element)
    }

    /// Get the number of elements in the set
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Get an iterator over the elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.iter()
    }

    /// Get the elements as a reference to the internal HashSet
    pub fn as_set(&self) -> &HashSet<T> {
        &self.elements
    }

    /// Convert into the internal HashSet
    pub fn into_set(self) -> HashSet<T> {
        self.elements
    }
}

impl<T: Eq + Hash + Clone> CvRDT for GSet<T> {
    fn merge(&mut self, other: &Self) {
        for element in &other.elements {
            self.elements.insert(element.clone());
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let is_subset = self.elements.is_subset(&other.elements);
        let is_superset = self.elements.is_superset(&other.elements);

        if is_subset && is_superset {
            Some(Ordering::Equal)
        } else if is_subset {
            Some(Ordering::Less)
        } else if is_superset {
            Some(Ordering::Greater)
        } else {
            None // Sets are concurrent
        }
    }
}

impl<T: Eq + Hash + Clone> CrdtValue for GSet<T> {
    type Value = HashSet<T>;

    fn value(&self) -> Self::Value {
        self.elements.clone()
    }
}

impl<T: Eq + Hash> Default for GSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash> FromIterator<T> for GSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_iter(iter)
    }
}

/// Operation for G-Set (operation-based variant)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GSetOp<T> {
    pub element: T,
}

impl<T: Eq + Hash> GSet<T> {
    /// Apply an operation to the set
    pub fn apply_op(&mut self, op: GSetOp<T>) {
        self.elements.insert(op.element);
    }

    /// Create an insert operation
    pub fn create_insert_op(&self, element: T) -> GSetOp<T> {
        GSetOp { element }
    }
}

/// Delta-state for efficient G-Set synchronization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GSetDelta<T: Eq + Hash> {
    pub elements: HashSet<T>,
}

impl<T: Eq + Hash + Clone> GSet<T> {
    /// Generate a delta containing only new elements since a given state
    pub fn delta_since(&self, other: &GSet<T>) -> GSetDelta<T> {
        let new_elements = self.elements.difference(&other.elements).cloned().collect();
        GSetDelta { elements: new_elements }
    }

    /// Merge a delta into this set
    pub fn merge_delta(&mut self, delta: GSetDelta<T>) {
        for element in delta.elements {
            self.elements.insert(element);
        }
    }
}

/// Two-Phase Set (2P-Set) - supports both add and remove
/// Once an element is removed, it cannot be re-added
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TwoPhaseSet<T: Eq + Hash> {
    added: GSet<T>,
    removed: GSet<T>,
}

impl<T: Eq + Hash + Clone> TwoPhaseSet<T> {
    /// Create a new empty 2P-Set
    pub fn new() -> Self {
        Self {
            added: GSet::new(),
            removed: GSet::new(),
        }
    }

    /// Add an element to the set
    /// Returns true if the element was added (not previously removed)
    pub fn insert(&mut self, element: T) -> bool {
        if self.removed.contains(&element) {
            false
        } else {
            self.added.insert(element)
        }
    }

    /// Remove an element from the set
    /// Returns true if the element was in the set
    pub fn remove(&mut self, element: T) -> bool {
        if self.added.contains(&element) {
            self.removed.insert(element);
            true
        } else {
            false
        }
    }

    /// Check if the set contains an element
    pub fn contains(&self, element: &T) -> bool {
        self.added.contains(element) && !self.removed.contains(element)
    }

    /// Get the number of elements in the set
    pub fn len(&self) -> usize {
        self.elements().count()
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an iterator over the current elements (added - removed)
    pub fn elements(&self) -> impl Iterator<Item = &T> {
        self.added.iter().filter(|e| !self.removed.contains(e))
    }

    /// Get elements as a HashSet
    pub fn to_set(&self) -> HashSet<T> {
        self.elements().cloned().collect()
    }
}

impl<T: Eq + Hash + Clone> CvRDT for TwoPhaseSet<T> {
    fn merge(&mut self, other: &Self) {
        self.added.merge(&other.added);
        self.removed.merge(&other.removed);
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let added_cmp = self.added.partial_cmp(&other.added);
        let removed_cmp = self.removed.partial_cmp(&other.removed);

        match (added_cmp, removed_cmp) {
            (Some(Ordering::Equal), Some(Ordering::Equal)) => Some(Ordering::Equal),
            (Some(Ordering::Less), Some(Ordering::Less)) |
            (Some(Ordering::Less), Some(Ordering::Equal)) |
            (Some(Ordering::Equal), Some(Ordering::Less)) => Some(Ordering::Less),
            (Some(Ordering::Greater), Some(Ordering::Greater)) |
            (Some(Ordering::Greater), Some(Ordering::Equal)) |
            (Some(Ordering::Equal), Some(Ordering::Greater)) => Some(Ordering::Greater),
            _ => None,
        }
    }
}

impl<T: Eq + Hash + Clone> Default for TwoPhaseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gset_basic() {
        let mut set: GSet<i32> = GSet::new();

        assert!(set.is_empty());
        assert_eq!(set.len(), 0);

        assert!(set.insert(1));
        assert!(set.insert(2));
        assert!(!set.insert(1)); // Already exists

        assert_eq!(set.len(), 2);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(!set.contains(&3));
    }

    #[test]
    fn test_gset_merge() {
        let mut set1: GSet<i32> = GSet::from_iter([1, 2, 3]);
        let set2: GSet<i32> = GSet::from_iter([3, 4, 5]);

        set1.merge(&set2);

        assert_eq!(set1.len(), 5);
        for i in 1..=5 {
            assert!(set1.contains(&i));
        }
    }

    #[test]
    fn test_gset_idempotent_merge() {
        let mut set1: GSet<i32> = GSet::from_iter([1, 2, 3]);
        let set2: GSet<i32> = GSet::from_iter([2, 3, 4]);

        set1.merge(&set2);
        let len1 = set1.len();

        set1.merge(&set2); // Merge again
        assert_eq!(set1.len(), len1); // Should be same
    }

    #[test]
    fn test_gset_partial_order() {
        let set1: GSet<i32> = GSet::from_iter([1, 2]);
        let set2: GSet<i32> = GSet::from_iter([1, 2, 3]);
        let set3: GSet<i32> = GSet::from_iter([2, 3]);

        assert_eq!(set1.partial_cmp(&set2), Some(Ordering::Less));
        assert_eq!(set2.partial_cmp(&set1), Some(Ordering::Greater));
        assert_eq!(set1.partial_cmp(&set1), Some(Ordering::Equal));
        assert_eq!(set1.partial_cmp(&set3), None); // Concurrent
    }

    #[test]
    fn test_gset_operations() {
        let mut set: GSet<String> = GSet::new();

        let op = set.create_insert_op("hello".to_string());
        set.apply_op(op);

        assert!(set.contains(&"hello".to_string()));
    }

    #[test]
    fn test_gset_delta() {
        let mut set1: GSet<i32> = GSet::from_iter([1, 2, 3]);
        let mut set2: GSet<i32> = GSet::from_iter([1, 2]);

        set2.insert(4);
        set2.insert(5);

        let delta = set2.delta_since(&set1);
        set1.merge_delta(delta);

        assert_eq!(set1.len(), 5);
        assert!(set1.contains(&4));
        assert!(set1.contains(&5));
    }

    #[test]
    fn test_two_phase_set() {
        let mut set: TwoPhaseSet<i32> = TwoPhaseSet::new();

        assert!(set.insert(1));
        assert!(set.insert(2));
        assert!(set.insert(3));

        assert_eq!(set.len(), 3);
        assert!(set.contains(&2));

        assert!(set.remove(2));
        assert_eq!(set.len(), 2);
        assert!(!set.contains(&2));

        // Can't re-add removed element
        assert!(!set.insert(2));
        assert!(!set.contains(&2));
    }

    #[test]
    fn test_two_phase_set_merge() {
        let mut set1: TwoPhaseSet<i32> = TwoPhaseSet::new();
        let mut set2: TwoPhaseSet<i32> = TwoPhaseSet::new();

        set1.insert(1);
        set1.insert(2);
        set1.remove(2);

        set2.insert(2);
        set2.insert(3);

        set1.merge(&set2);

        assert!(set1.contains(&1));
        assert!(!set1.contains(&2)); // Removed in set1
        assert!(set1.contains(&3));

        set2.merge(&set1);
        assert!(!set2.contains(&2)); // Remove propagated
    }

    #[test]
    fn test_gset_convergence() {
        let mut set1: GSet<i32> = GSet::from_iter([1, 2]);
        let mut set2: GSet<i32> = GSet::from_iter([2, 3]);
        let mut set3: GSet<i32> = GSet::from_iter([3, 4]);

        // Merge in different orders
        let mut result1 = set1.clone();
        result1.merge(&set2);
        result1.merge(&set3);

        let mut result2 = set3.clone();
        result2.merge(&set1);
        result2.merge(&set2);

        assert_eq!(result1, result2);
        assert_eq!(result1.len(), 4);
    }
}
