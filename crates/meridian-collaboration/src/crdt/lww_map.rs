//! # Last-Write-Wins Map (LWW-Map)
//!
//! A state-based CRDT that implements a key-value map with timestamp-based conflict resolution.
//! Uses LWW-Register for each value and OR-Set for key management.
//!
//! ## Properties
//! - **Convergent**: All replicas converge to the same map state
//! - **Per-key LWW**: Each key's value is resolved independently
//! - **Use Cases**: Distributed configuration, user preferences, document metadata

use super::{CrdtValue, CvRDT, HybridTimestamp, ReplicaId};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

/// Entry in the LWW-Map with value and timestamp
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MapEntry<V> {
    value: V,
    timestamp: HybridTimestamp,
    tombstone: bool, // true if this key has been deleted
}

/// Last-Write-Wins Map implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LWWMap<K: Eq + Hash, V> {
    entries: HashMap<K, MapEntry<V>>,
    replica_id: ReplicaId,
}

impl<K: Eq + Hash + Clone, V: Clone> LWWMap<K, V> {
    /// Create a new empty LWW-Map
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            entries: HashMap::new(),
            replica_id,
        }
    }

    /// Insert or update a key-value pair
    pub fn insert(&mut self, key: K, value: V) {
        let mut timestamp = HybridTimestamp::new(self.replica_id);

        // If key exists, advance from its timestamp
        if let Some(entry) = self.entries.get(&key) {
            timestamp = entry.timestamp;
            timestamp.tick();
        }

        self.entries.insert(
            key,
            MapEntry {
                value,
                timestamp,
                tombstone: false,
            },
        );
    }

    /// Remove a key from the map
    pub fn remove(&mut self, key: &K) -> bool {
        if let Some(entry) = self.entries.get_mut(key) {
            if !entry.tombstone {
                entry.timestamp.tick();
                entry.tombstone = true;
                return true;
            }
        }
        false
    }

    /// Get a value for a key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|entry| {
            if entry.tombstone {
                None
            } else {
                Some(&entry.value)
            }
        })
    }

    /// Check if a key exists in the map
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Get the number of active entries
    pub fn len(&self) -> usize {
        self.entries.values().filter(|e| !e.tombstone).count()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an iterator over key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter().filter_map(|(k, entry)| {
            if entry.tombstone {
                None
            } else {
                Some((k, &entry.value))
            }
        })
    }

    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().filter_map(|(k, entry)| {
            if entry.tombstone {
                None
            } else {
                Some(k)
            }
        })
    }

    /// Get all values
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.values().filter_map(|entry| {
            if entry.tombstone {
                None
            } else {
                Some(&entry.value)
            }
        })
    }

    /// Convert to a regular HashMap
    pub fn to_hashmap(&self) -> HashMap<K, V> {
        self.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Clear all entries (marks as tombstones)
    pub fn clear(&mut self) {
        for entry in self.entries.values_mut() {
            if !entry.tombstone {
                entry.timestamp.tick();
                entry.tombstone = true;
            }
        }
    }

    /// Get the timestamp for a key
    pub fn get_timestamp(&self, key: &K) -> Option<&HybridTimestamp> {
        self.entries.get(key).map(|entry| &entry.timestamp)
    }

    /// Garbage collect old tombstones
    /// Removes tombstones older than the given timestamp
    pub fn gc_tombstones(&mut self, before: &HybridTimestamp) {
        self.entries.retain(|_, entry| {
            if entry.tombstone {
                entry.timestamp >= *before
            } else {
                true
            }
        });
    }
}

impl<K: Eq + Hash + Clone, V: Clone> CvRDT for LWWMap<K, V> {
    fn merge(&mut self, other: &Self) {
        for (key, other_entry) in &other.entries {
            match self.entries.get_mut(key) {
                Some(self_entry) => {
                    // Keep entry with higher timestamp
                    if other_entry.timestamp > self_entry.timestamp {
                        *self_entry = other_entry.clone();
                    }
                }
                None => {
                    self.entries.insert(key.clone(), other_entry.clone());
                }
            }
        }
    }

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut less_or_equal = true;
        let mut greater_or_equal = true;

        let all_keys: std::collections::HashSet<_> = self
            .entries
            .keys()
            .chain(other.entries.keys())
            .collect();

        for key in all_keys {
            let self_ts = self.entries.get(key).map(|e| &e.timestamp);
            let other_ts = other.entries.get(key).map(|e| &e.timestamp);

            match (self_ts, other_ts) {
                (Some(st), Some(ot)) => {
                    if st > ot {
                        less_or_equal = false;
                    }
                    if st < ot {
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
            None
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> CrdtValue for LWWMap<K, V> {
    type Value = HashMap<K, V>;

    fn value(&self) -> Self::Value {
        self.to_hashmap()
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Default for LWWMap<K, V> {
    fn default() -> Self {
        Self::new(ReplicaId::new())
    }
}

/// Operation for LWW-Map
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LWWMapOp<K, V> {
    Insert {
        key: K,
        value: V,
        timestamp: HybridTimestamp,
    },
    Remove {
        key: K,
        timestamp: HybridTimestamp,
    },
}

impl<K: Eq + Hash + Clone, V: Clone> LWWMap<K, V> {
    /// Apply an operation to the map
    pub fn apply_op(&mut self, op: LWWMapOp<K, V>) {
        match op {
            LWWMapOp::Insert { key, value, timestamp } => {
                match self.entries.get_mut(&key) {
                    Some(entry) if timestamp > entry.timestamp => {
                        entry.value = value;
                        entry.timestamp = timestamp;
                        entry.tombstone = false;
                    }
                    None => {
                        self.entries.insert(
                            key,
                            MapEntry {
                                value,
                                timestamp,
                                tombstone: false,
                            },
                        );
                    }
                    _ => {} // Older timestamp, ignore
                }
            }
            LWWMapOp::Remove { key, timestamp } => {
                match self.entries.get_mut(&key) {
                    Some(entry) if timestamp > entry.timestamp => {
                        entry.timestamp = timestamp;
                        entry.tombstone = true;
                    }
                    _ => {} // No entry or older timestamp
                }
            }
        }
    }

    /// Create an insert operation
    pub fn create_insert_op(&mut self, key: K, value: V) -> LWWMapOp<K, V> {
        let mut timestamp = HybridTimestamp::new(self.replica_id);
        if let Some(entry) = self.entries.get(&key) {
            timestamp = entry.timestamp;
            timestamp.tick();
        }

        LWWMapOp::Insert {
            key,
            value,
            timestamp,
        }
    }

    /// Create a remove operation
    pub fn create_remove_op(&mut self, key: K) -> LWWMapOp<K, V> {
        let mut timestamp = HybridTimestamp::new(self.replica_id);
        if let Some(entry) = self.entries.get(&key) {
            timestamp = entry.timestamp;
            timestamp.tick();
        }

        LWWMapOp::Remove { key, timestamp }
    }
}

/// Delta-state for LWW-Map
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LWWMapDelta<K: Eq + Hash, V> {
    pub entries: HashMap<K, MapEntry<V>>,
}

impl<K: Eq + Hash + Clone, V: Clone> LWWMap<K, V> {
    /// Generate a delta since a given state
    pub fn delta_since(&self, other: &LWWMap<K, V>) -> LWWMapDelta<K, V> {
        let mut delta_entries = HashMap::new();

        for (key, entry) in &self.entries {
            let should_include = match other.entries.get(key) {
                Some(other_entry) => entry.timestamp > other_entry.timestamp,
                None => true,
            };

            if should_include {
                delta_entries.insert(key.clone(), entry.clone());
            }
        }

        LWWMapDelta { entries: delta_entries }
    }

    /// Merge a delta into this map
    pub fn merge_delta(&mut self, delta: LWWMapDelta<K, V>) {
        for (key, delta_entry) in delta.entries {
            match self.entries.get_mut(&key) {
                Some(entry) if delta_entry.timestamp > entry.timestamp => {
                    *entry = delta_entry;
                }
                None => {
                    self.entries.insert(key, delta_entry);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lwwmap_basic() {
        let replica = ReplicaId::new();
        let mut map: LWWMap<String, i32> = LWWMap::new(replica);

        assert!(map.is_empty());

        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);

        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&"a".to_string()), Some(&1));
        assert_eq!(map.get(&"b".to_string()), Some(&2));

        map.insert("a".to_string(), 10);
        assert_eq!(map.get(&"a".to_string()), Some(&10));
    }

    #[test]
    fn test_lwwmap_remove() {
        let replica = ReplicaId::new();
        let mut map: LWWMap<String, i32> = LWWMap::new(replica);

        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);

        assert!(map.remove(&"a".to_string()));
        assert!(!map.contains_key(&"a".to_string()));
        assert_eq!(map.len(), 1);

        // Removing again returns false
        assert!(!map.remove(&"a".to_string()));
    }

    #[test]
    fn test_lwwmap_merge() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut map1: LWWMap<String, i32> = LWWMap::new(replica1);
        let mut map2: LWWMap<String, i32> = LWWMap::new(replica2);

        map1.insert("a".to_string(), 1);
        map1.insert("b".to_string(), 2);

        std::thread::sleep(std::time::Duration::from_millis(10));

        map2.insert("b".to_string(), 20);
        map2.insert("c".to_string(), 3);

        map1.merge(&map2);

        assert_eq!(map1.get(&"a".to_string()), Some(&1));
        assert_eq!(map1.get(&"b".to_string()), Some(&20)); // map2's value wins
        assert_eq!(map1.get(&"c".to_string()), Some(&3));
    }

    #[test]
    fn test_lwwmap_concurrent_updates() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut map1: LWWMap<String, i32> = LWWMap::new(replica1);
        let mut map2: LWWMap<String, i32> = LWWMap::new(replica2);

        // Both start with same state
        map1.insert("key".to_string(), 1);
        map2.merge(&map1);

        // Concurrent updates
        map1.insert("key".to_string(), 100);
        map2.insert("key".to_string(), 200);

        // Merge should be deterministic
        let mut result1 = map1.clone();
        result1.merge(&map2);

        let mut result2 = map2.clone();
        result2.merge(&map1);

        assert_eq!(result1.get(&"key".to_string()), result2.get(&"key".to_string()));
    }

    #[test]
    fn test_lwwmap_operations() {
        let replica = ReplicaId::new();
        let mut map: LWWMap<String, i32> = LWWMap::new(replica);

        let insert_op = map.create_insert_op("test".to_string(), 42);
        map.apply_op(insert_op);

        assert_eq!(map.get(&"test".to_string()), Some(&42));

        let remove_op = map.create_remove_op("test".to_string());
        map.apply_op(remove_op);

        assert!(!map.contains_key(&"test".to_string()));
    }

    #[test]
    fn test_lwwmap_delta() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut map1: LWWMap<String, i32> = LWWMap::new(replica1);
        let mut map2: LWWMap<String, i32> = LWWMap::new(replica2);

        map1.insert("a".to_string(), 1);
        map1.insert("b".to_string(), 2);

        map2.insert("c".to_string(), 3);
        map2.insert("d".to_string(), 4);

        let delta = map2.delta_since(&map1);
        map1.merge_delta(delta);

        assert_eq!(map1.len(), 4);
        assert!(map1.contains_key(&"c".to_string()));
        assert!(map1.contains_key(&"d".to_string()));
    }

    #[test]
    fn test_lwwmap_clear() {
        let replica = ReplicaId::new();
        let mut map: LWWMap<String, i32> = LWWMap::new(replica);

        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);
        map.insert("c".to_string(), 3);

        assert_eq!(map.len(), 3);

        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn test_lwwmap_iteration() {
        let replica = ReplicaId::new();
        let mut map: LWWMap<String, i32> = LWWMap::new(replica);

        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);
        map.insert("c".to_string(), 3);

        let keys: Vec<_> = map.keys().cloned().collect();
        assert_eq!(keys.len(), 3);

        let values: Vec<_> = map.values().copied().collect();
        assert!(values.contains(&1));
        assert!(values.contains(&2));
        assert!(values.contains(&3));
    }

    #[test]
    fn test_lwwmap_tombstone_gc() {
        let replica = ReplicaId::new();
        let mut map: LWWMap<String, i32> = LWWMap::new(replica);

        map.insert("a".to_string(), 1);
        let old_ts = map.get_timestamp(&"a".to_string()).unwrap().clone();

        std::thread::sleep(std::time::Duration::from_millis(10));

        map.remove(&"a".to_string());

        // Before GC, tombstone exists
        assert!(map.entries.contains_key(&"a".to_string()));

        // GC tombstones older than current time (should not remove recent tombstone)
        let now = HybridTimestamp::new(replica);
        map.gc_tombstones(&old_ts);

        // After GC with old timestamp, tombstone should be gone
        assert!(!map.entries.contains_key(&"a".to_string()));
    }
}
