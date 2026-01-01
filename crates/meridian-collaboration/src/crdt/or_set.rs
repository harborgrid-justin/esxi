//! # Observed-Remove Set (OR-Set)
//!
//! A state-based CRDT that implements a set with both add and remove operations.
//! Unlike 2P-Set, elements can be re-added after removal.
//!
//! ## Algorithm
//! - Each element is tagged with a unique identifier when added
//! - Remove operations specify which tags to remove
//! - An element is in the set if it has at least one tag that hasn't been removed
//!
//! ## Properties
//! - **Add-wins semantics**: Concurrent add and remove, add wins
//! - **Convergent**: All replicas converge to the same set
//! - **Use Cases**: Collaborative collections, shopping carts, task lists

use super::{CrdtValue, CvRDT, ReplicaId};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use uuid::Uuid;

/// Unique tag for each element addition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ElementTag(Uuid);

impl ElementTag {
    /// Create a new unique tag
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for ElementTag {
    fn default() -> Self {
        Self::new()
    }
}

/// Observed-Remove Set implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ORSet<T: Eq + Hash> {
    /// Maps elements to their set of unique tags
    elements: HashMap<T, HashSet<ElementTag>>,
    /// Replica ID for this instance
    replica_id: ReplicaId,
}

impl<T: Eq + Hash + Clone> ORSet<T> {
    /// Create a new empty OR-Set
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            elements: HashMap::new(),
            replica_id,
        }
    }

    /// Add an element to the set
    /// Returns the tag assigned to this addition
    pub fn insert(&mut self, element: T) -> ElementTag {
        let tag = ElementTag::new();
        self.elements.entry(element).or_insert_with(HashSet::new).insert(tag);
        tag
    }

    /// Add an element with a specific tag (for replication)
    pub fn insert_with_tag(&mut self, element: T, tag: ElementTag) {
        self.elements.entry(element).or_insert_with(HashSet::new).insert(tag);
    }

    /// Remove an element from the set
    /// This removes all current tags for the element
    pub fn remove(&mut self, element: &T) -> bool {
        self.elements.remove(element).is_some()
    }

    /// Remove specific tags of an element
    pub fn remove_tags(&mut self, element: &T, tags: &HashSet<ElementTag>) {
        if let Some(element_tags) = self.elements.get_mut(element) {
            element_tags.retain(|tag| !tags.contains(tag));
            if element_tags.is_empty() {
                self.elements.remove(element);
            }
        }
    }

    /// Check if the set contains an element
    pub fn contains(&self, element: &T) -> bool {
        self.elements.get(element).map_or(false, |tags| !tags.is_empty())
    }

    /// Get the number of distinct elements in the set
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Get an iterator over the elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.elements.keys()
    }

    /// Get the tags for a specific element
    pub fn get_tags(&self, element: &T) -> Option<&HashSet<ElementTag>> {
        self.elements.get(element)
    }

    /// Get all elements as a HashSet
    pub fn to_set(&self) -> HashSet<T> {
        self.elements.keys().cloned().collect()
    }

    /// Clear the set
    pub fn clear(&mut self) {
        self.elements.clear();
    }
}

impl<T: Eq + Hash + Clone> CvRDT for ORSet<T> {
    fn merge(&mut self, other: &Self) {
        for (element, other_tags) in &other.elements {
            let tags = self.elements.entry(element.clone()).or_insert_with(HashSet::new);
            for tag in other_tags {
                tags.insert(*tag);
            }
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut less_or_equal = true;
        let mut greater_or_equal = true;

        // Check all elements in both sets
        let all_elements: HashSet<_> = self
            .elements
            .keys()
            .chain(other.elements.keys())
            .collect();

        for element in all_elements {
            let self_tags = self.elements.get(element);
            let other_tags = other.elements.get(element);

            match (self_tags, other_tags) {
                (Some(st), Some(ot)) => {
                    if !st.is_subset(ot) {
                        less_or_equal = false;
                    }
                    if !st.is_superset(ot) {
                        greater_or_equal = false;
                    }
                }
                (Some(_), None) => less_or_equal = false,
                (None, Some(_)) => greater_or_equal = false,
                (None, None) => {}
            }
        }

        if less_or_equal && greater_or_equal {
            Some(Ordering::Equal)
        } else if less_or_equal {
            Some(Ordering::Less)
        } else if greater_or_equal {
            Some(Ordering::Greater)
        } else {
            None // Concurrent
        }
    }
}

impl<T: Eq + Hash + Clone> CrdtValue for ORSet<T> {
    type Value = HashSet<T>;

    fn value(&self) -> Self::Value {
        self.to_set()
    }
}

impl<T: Eq + Hash + Clone> Default for ORSet<T> {
    fn default() -> Self {
        Self::new(ReplicaId::new())
    }
}

/// Operation for OR-Set
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ORSetOp<T> {
    Add { element: T, tag: ElementTag },
    Remove { element: T, tags: HashSet<ElementTag> },
}

impl<T: Eq + Hash + Clone> ORSet<T> {
    /// Apply an operation to the set
    pub fn apply_op(&mut self, op: ORSetOp<T>) {
        match op {
            ORSetOp::Add { element, tag } => {
                self.insert_with_tag(element, tag);
            }
            ORSetOp::Remove { element, tags } => {
                self.remove_tags(&element, &tags);
            }
        }
    }

    /// Create an add operation
    pub fn create_add_op(&mut self, element: T) -> ORSetOp<T> {
        let tag = ElementTag::new();
        ORSetOp::Add {
            element: element.clone(),
            tag,
        }
    }

    /// Create a remove operation for an element
    pub fn create_remove_op(&self, element: &T) -> Option<ORSetOp<T>> {
        self.elements.get(element).map(|tags| ORSetOp::Remove {
            element: element.clone(),
            tags: tags.clone(),
        })
    }
}

/// Optimized OR-Set using version vectors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizedORSet<T: Eq + Hash> {
    /// Elements with their addition version
    added: HashMap<T, HashSet<(ReplicaId, u64)>>,
    /// Replica ID
    replica_id: ReplicaId,
    /// Version counter for this replica
    version: u64,
}

impl<T: Eq + Hash + Clone> OptimizedORSet<T> {
    /// Create a new optimized OR-Set
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            added: HashMap::new(),
            replica_id,
            version: 0,
        }
    }

    /// Add an element
    pub fn insert(&mut self, element: T) {
        self.version += 1;
        self.added
            .entry(element)
            .or_insert_with(HashSet::new)
            .insert((self.replica_id, self.version));
    }

    /// Remove an element (removes all current versions)
    pub fn remove(&mut self, element: &T) -> bool {
        self.added.remove(element).is_some()
    }

    /// Check if element exists
    pub fn contains(&self, element: &T) -> bool {
        self.added.get(element).map_or(false, |versions| !versions.is_empty())
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.added.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
    }

    /// Iterate over elements
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.added.keys()
    }
}

impl<T: Eq + Hash + Clone> CvRDT for OptimizedORSet<T> {
    fn merge(&mut self, other: &Self) {
        for (element, other_versions) in &other.added {
            let versions = self.added.entry(element.clone()).or_insert_with(HashSet::new);
            for version in other_versions {
                versions.insert(*version);
            }
        }
    }

    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        None // No total order for OR-Sets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orset_basic() {
        let replica = ReplicaId::new();
        let mut set: ORSet<i32> = ORSet::new(replica);

        assert!(set.is_empty());

        let tag1 = set.insert(1);
        assert!(set.contains(&1));
        assert_eq!(set.len(), 1);

        set.insert(2);
        assert_eq!(set.len(), 2);

        assert!(set.remove(&1));
        assert!(!set.contains(&1));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_orset_readd() {
        let replica = ReplicaId::new();
        let mut set: ORSet<String> = ORSet::new(replica);

        set.insert("hello".to_string());
        assert!(set.contains(&"hello".to_string()));

        set.remove(&"hello".to_string());
        assert!(!set.contains(&"hello".to_string()));

        // Can re-add after removal (unlike 2P-Set)
        set.insert("hello".to_string());
        assert!(set.contains(&"hello".to_string()));
    }

    #[test]
    fn test_orset_merge() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut set1: ORSet<i32> = ORSet::new(replica1);
        let mut set2: ORSet<i32> = ORSet::new(replica2);

        set1.insert(1);
        set1.insert(2);

        set2.insert(2);
        set2.insert(3);

        set1.merge(&set2);

        assert!(set1.contains(&1));
        assert!(set1.contains(&2));
        assert!(set1.contains(&3));
        assert_eq!(set1.len(), 3);
    }

    #[test]
    fn test_orset_add_wins() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut set1: ORSet<i32> = ORSet::new(replica1);
        let mut set2: ORSet<i32> = ORSet::new(replica2);

        // Both add element 1
        let tag1 = set1.insert(1);
        let tag2 = set2.insert(1);

        // set1 removes it
        set1.remove(&1);

        // Merge: set2's add should win (concurrent add/remove)
        set1.merge(&set2);

        // Element should exist because set2's tag is still present
        assert!(set1.contains(&1));
    }

    #[test]
    fn test_orset_operations() {
        let replica = ReplicaId::new();
        let mut set: ORSet<String> = ORSet::new(replica);

        let add_op = set.create_add_op("test".to_string());
        set.apply_op(add_op.clone());

        assert!(set.contains(&"test".to_string()));

        let remove_op = set.create_remove_op(&"test".to_string()).unwrap();
        set.apply_op(remove_op);

        assert!(!set.contains(&"test".to_string()));
    }

    #[test]
    fn test_orset_concurrent_operations() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut set1: ORSet<i32> = ORSet::new(replica1);
        let mut set2: ORSet<i32> = ORSet::new(replica2);

        // Both add different elements
        set1.insert(1);
        set2.insert(2);

        // Clone for concurrent operations
        let mut set1_fork = set1.clone();
        let mut set2_fork = set2.clone();

        // Concurrent operations
        set1.insert(3);
        set2.remove(&2);

        // Merge in both directions
        set1.merge(&set2);
        set2_fork.merge(&set1_fork);

        // Verify convergence
        assert_eq!(set1.to_set(), set2_fork.to_set());
    }

    #[test]
    fn test_orset_tags() {
        let replica = ReplicaId::new();
        let mut set: ORSet<i32> = ORSet::new(replica);

        let tag1 = set.insert(1);
        let tag2 = set.insert(1); // Add same element again

        let tags = set.get_tags(&1).unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&tag1));
        assert!(tags.contains(&tag2));
    }

    #[test]
    fn test_optimized_orset() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut set1 = OptimizedORSet::new(replica1);
        let mut set2 = OptimizedORSet::new(replica2);

        set1.insert(1);
        set2.insert(2);

        set1.merge(&set2);

        assert!(set1.contains(&1));
        assert!(set1.contains(&2));
        assert_eq!(set1.len(), 2);
    }

    #[test]
    fn test_orset_convergence() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();
        let replica3 = ReplicaId::new();

        let mut set1: ORSet<i32> = ORSet::new(replica1);
        let mut set2: ORSet<i32> = ORSet::new(replica2);
        let mut set3: ORSet<i32> = ORSet::new(replica3);

        // Different operations
        set1.insert(1);
        set2.insert(2);
        set3.insert(3);

        // Merge in different orders
        let mut result1 = set1.clone();
        result1.merge(&set2);
        result1.merge(&set3);

        let mut result2 = set3.clone();
        result2.merge(&set1);
        result2.merge(&set2);

        assert_eq!(result1.to_set(), result2.to_set());
    }
}
