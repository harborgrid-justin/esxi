//! Conflict-free Replicated Data Types (CRDT) implementations for geo data

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::sync::VectorClock;

/// CRDT value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtValue<T> {
    /// Actual value
    pub value: T,

    /// Vector clock for causality
    pub clock: VectorClock,

    /// Tombstone (for deletion)
    pub deleted: bool,
}

impl<T: Clone> CrdtValue<T> {
    /// Create new CRDT value
    pub fn new(value: T, node_id: String) -> Self {
        let mut clock = VectorClock::new(node_id);
        clock.increment();

        Self {
            value,
            clock,
            deleted: false,
        }
    }

    /// Mark as deleted
    pub fn delete(&mut self) {
        self.deleted = true;
        self.clock.increment();
    }

    /// Check if deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted
    }
}

/// Last-Write-Wins (LWW) Map CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Internal map
    entries: HashMap<K, CrdtValue<V>>,

    /// Node ID
    node_id: String,
}

impl<K, V> CrdtMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create new CRDT map
    pub fn new(node_id: String) -> Self {
        Self {
            entries: HashMap::new(),
            node_id,
        }
    }

    /// Insert value
    pub fn insert(&mut self, key: K, value: V) {
        let crdt_value = CrdtValue::new(value, self.node_id.clone());
        self.entries.insert(key, crdt_value);
    }

    /// Get value
    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|v| {
            if v.is_deleted() {
                None
            } else {
                Some(&v.value)
            }
        })
    }

    /// Remove value (tombstone)
    pub fn remove(&mut self, key: &K) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.delete();
        }
    }

    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries
            .iter()
            .filter(|(_, v)| !v.is_deleted())
            .map(|(k, _)| k)
    }

    /// Get all values
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries
            .iter()
            .filter(|(_, v)| !v.is_deleted())
            .map(|(_, v)| &v.value)
    }

    /// Get entry count (excluding tombstones)
    pub fn len(&self) -> usize {
        self.entries.iter().filter(|(_, v)| !v.is_deleted()).count()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Merge with another map
    pub fn merge(&mut self, other: &Self) {
        for (key, other_value) in &other.entries {
            match self.entries.get(key) {
                Some(our_value) => {
                    // Keep the one with the later clock
                    if other_value.clock > our_value.clock {
                        self.entries.insert(key.clone(), other_value.clone());
                    }
                }
                None => {
                    self.entries.insert(key.clone(), other_value.clone());
                }
            }
        }
    }

    /// Get state for synchronization
    pub fn state(&self) -> HashMap<K, CrdtValue<V>> {
        self.entries.clone()
    }

    /// Apply state from another replica
    pub fn apply_state(&mut self, state: HashMap<K, CrdtValue<V>>) {
        for (key, value) in state {
            match self.entries.get(&key) {
                Some(our_value) => {
                    if value.clock > our_value.clock {
                        self.entries.insert(key, value);
                    }
                }
                None => {
                    self.entries.insert(key, value);
                }
            }
        }
    }
}

/// Add-Only Set CRDT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtSet<T>
where
    T: Eq + Hash + Clone,
{
    /// Internal set with vector clocks
    elements: HashMap<T, VectorClock>,

    /// Node ID
    node_id: String,
}

impl<T> CrdtSet<T>
where
    T: Eq + Hash + Clone,
{
    /// Create new CRDT set
    pub fn new(node_id: String) -> Self {
        Self {
            elements: HashMap::new(),
            node_id,
        }
    }

    /// Add element
    pub fn insert(&mut self, element: T) {
        let mut clock = VectorClock::new(self.node_id.clone());
        clock.increment();
        self.elements.insert(element, clock);
    }

    /// Check if contains element
    pub fn contains(&self, element: &T) -> bool {
        self.elements.contains_key(element)
    }

    /// Get all elements
    pub fn elements(&self) -> impl Iterator<Item = &T> {
        self.elements.keys()
    }

    /// Get size
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Merge with another set
    pub fn merge(&mut self, other: &Self) {
        for (element, clock) in &other.elements {
            self.elements
                .entry(element.clone())
                .and_modify(|our_clock| {
                    our_clock.merge(clock);
                })
                .or_insert_with(|| clock.clone());
        }
    }
}

/// Two-Phase Set (2P-Set) CRDT with removals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoPhaseSet<T>
where
    T: Eq + Hash + Clone,
{
    /// Added elements
    added: CrdtSet<T>,

    /// Removed elements
    removed: CrdtSet<T>,
}

impl<T> TwoPhaseSet<T>
where
    T: Eq + Hash + Clone,
{
    /// Create new 2P-Set
    pub fn new(node_id: String) -> Self {
        Self {
            added: CrdtSet::new(node_id.clone()),
            removed: CrdtSet::new(node_id),
        }
    }

    /// Add element
    pub fn insert(&mut self, element: T) {
        self.added.insert(element);
    }

    /// Remove element
    pub fn remove(&mut self, element: T) {
        self.removed.insert(element);
    }

    /// Check if contains element
    pub fn contains(&self, element: &T) -> bool {
        self.added.contains(element) && !self.removed.contains(element)
    }

    /// Get all active elements
    pub fn elements(&self) -> Vec<&T> {
        self.added
            .elements()
            .filter(|e| !self.removed.contains(e))
            .collect()
    }

    /// Merge with another 2P-Set
    pub fn merge(&mut self, other: &Self) {
        self.added.merge(&other.added);
        self.removed.merge(&other.removed);
    }
}

/// Counter CRDT (G-Counter for grow-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCounter {
    /// Counts per node
    counts: HashMap<String, u64>,

    /// Node ID
    node_id: String,
}

impl GCounter {
    /// Create new G-Counter
    pub fn new(node_id: String) -> Self {
        Self {
            counts: HashMap::new(),
            node_id,
        }
    }

    /// Increment counter
    pub fn increment(&mut self) {
        *self.counts.entry(self.node_id.clone()).or_insert(0) += 1;
    }

    /// Increment by value
    pub fn add(&mut self, value: u64) {
        *self.counts.entry(self.node_id.clone()).or_insert(0) += value;
    }

    /// Get total count
    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Merge with another counter
    pub fn merge(&mut self, other: &Self) {
        for (node, count) in &other.counts {
            self.counts
                .entry(node.clone())
                .and_modify(|our_count| {
                    *our_count = (*our_count).max(*count);
                })
                .or_insert(*count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_map() {
        let mut map1 = CrdtMap::new("node1".to_string());
        map1.insert("key1", "value1");
        assert_eq!(map1.get(&"key1"), Some(&"value1"));

        let mut map2 = CrdtMap::new("node2".to_string());
        map2.insert("key2", "value2");

        map1.merge(&map2);
        assert_eq!(map1.len(), 2);
        assert_eq!(map1.get(&"key1"), Some(&"value1"));
        assert_eq!(map1.get(&"key2"), Some(&"value2"));
    }

    #[test]
    fn test_crdt_set() {
        let mut set1 = CrdtSet::new("node1".to_string());
        set1.insert("a");
        set1.insert("b");

        let mut set2 = CrdtSet::new("node2".to_string());
        set2.insert("b");
        set2.insert("c");

        set1.merge(&set2);
        assert_eq!(set1.len(), 3);
        assert!(set1.contains(&"a"));
        assert!(set1.contains(&"b"));
        assert!(set1.contains(&"c"));
    }

    #[test]
    fn test_gcounter() {
        let mut counter1 = GCounter::new("node1".to_string());
        counter1.increment();
        counter1.increment();

        let mut counter2 = GCounter::new("node2".to_string());
        counter2.add(3);

        counter1.merge(&counter2);
        assert_eq!(counter1.value(), 5);
    }

    #[test]
    fn test_two_phase_set() {
        let mut set = TwoPhaseSet::new("node1".to_string());
        set.insert("a");
        set.insert("b");

        assert!(set.contains(&"a"));
        assert!(set.contains(&"b"));

        set.remove("a");
        assert!(!set.contains(&"a"));
        assert!(set.contains(&"b"));
    }
}
