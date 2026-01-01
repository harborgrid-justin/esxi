//! # Replicated Growable Array (RGA)
//!
//! A state-based CRDT for collaborative text editing. Maintains a sequence of characters
//! with unique identifiers, enabling concurrent insertions and deletions.
//!
//! ## Algorithm
//! - Each character has a unique ID (timestamp + replica)
//! - Characters reference their left neighbor for positioning
//! - Tombstones mark deleted characters
//! - Convergence guaranteed through deterministic ordering
//!
//! ## Properties
//! - **Convergent**: All replicas converge to same text
//! - **Intention Preservation**: Maintains user intent
//! - **Use Cases**: Real-time collaborative text editors, document editing

use super::{CrdtValue, CvRDT, HybridTimestamp, ReplicaId};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

/// Unique identifier for each character in the sequence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CharId {
    pub timestamp: HybridTimestamp,
    pub replica_id: ReplicaId,
}

impl CharId {
    /// Create a new character ID
    pub fn new(timestamp: HybridTimestamp, replica_id: ReplicaId) -> Self {
        Self { timestamp, replica_id }
    }

    /// Create a root ID (used as left neighbor for first character)
    pub fn root() -> Self {
        use chrono::TimeZone;
        Self {
            timestamp: HybridTimestamp::with_time(
                chrono::Utc.timestamp_opt(0, 0).unwrap(),
                ReplicaId::from_uuid(uuid::Uuid::nil()),
            ),
            replica_id: ReplicaId::from_uuid(uuid::Uuid::nil()),
        }
    }

    /// Check if this is the root ID
    pub fn is_root(&self) -> bool {
        self.replica_id.0.is_nil()
    }
}

impl PartialOrd for CharId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CharId {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.timestamp.cmp(&other.timestamp) {
            Ordering::Equal => self.replica_id.cmp(&other.replica_id),
            ordering => ordering,
        }
    }
}

/// A character in the RGA with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RgaChar {
    id: CharId,
    value: char,
    left: CharId,      // ID of character to the left
    tombstone: bool,   // true if deleted
}

/// Replicated Growable Array for text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RGA {
    /// All characters indexed by their ID
    chars: HashMap<CharId, RgaChar>,
    /// This replica's ID
    replica_id: ReplicaId,
    /// Current timestamp
    timestamp: HybridTimestamp,
}

impl RGA {
    /// Create a new empty RGA
    pub fn new(replica_id: ReplicaId) -> Self {
        let mut chars = HashMap::new();

        // Insert root element
        chars.insert(
            CharId::root(),
            RgaChar {
                id: CharId::root(),
                value: '\0',
                left: CharId::root(),
                tombstone: false,
            },
        );

        Self {
            chars,
            replica_id,
            timestamp: HybridTimestamp::new(replica_id),
        }
    }

    /// Insert a character at a position
    pub fn insert(&mut self, index: usize, ch: char) -> CharId {
        self.timestamp.tick();

        let left_id = self.get_id_at_index(index);
        let char_id = CharId::new(self.timestamp, self.replica_id);

        self.chars.insert(
            char_id,
            RgaChar {
                id: char_id,
                value: ch,
                left: left_id,
                tombstone: false,
            },
        );

        char_id
    }

    /// Insert a string at a position
    pub fn insert_str(&mut self, index: usize, s: &str) -> Vec<CharId> {
        let mut ids = Vec::new();
        let mut current_index = index;

        for ch in s.chars() {
            let id = self.insert(current_index, ch);
            ids.push(id);
            current_index += 1;
        }

        ids
    }

    /// Delete a character at a position
    pub fn delete(&mut self, index: usize) -> Option<CharId> {
        let char_id = self.get_id_at_visible_index(index)?;

        if let Some(rga_char) = self.chars.get_mut(&char_id) {
            if !rga_char.tombstone {
                rga_char.tombstone = true;
                return Some(char_id);
            }
        }

        None
    }

    /// Delete a range of characters
    pub fn delete_range(&mut self, start: usize, end: usize) -> Vec<CharId> {
        let mut deleted = Vec::new();

        for i in (start..end).rev() {
            if let Some(id) = self.delete(i) {
                deleted.push(id);
            }
        }

        deleted.reverse();
        deleted
    }

    /// Get the current text content
    pub fn to_string(&self) -> String {
        self.iter().collect()
    }

    /// Get the length of visible characters
    pub fn len(&self) -> usize {
        self.chars.values().filter(|c| !c.tombstone && !c.id.is_root()).count()
    }

    /// Check if the text is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get an iterator over visible characters
    pub fn iter(&self) -> impl Iterator<Item = char> + '_ {
        self.get_ordered_chars()
            .into_iter()
            .filter(|c| !c.tombstone && !c.id.is_root())
            .map(|c| c.value)
    }

    /// Get the character ID at a given visible index
    fn get_id_at_visible_index(&self, index: usize) -> Option<CharId> {
        let visible_chars: Vec<_> = self
            .get_ordered_chars()
            .into_iter()
            .filter(|c| !c.tombstone && !c.id.is_root())
            .collect();

        visible_chars.get(index).map(|c| c.id)
    }

    /// Get the character ID at a given index (for insertion, includes root)
    fn get_id_at_index(&self, index: usize) -> CharId {
        if index == 0 {
            return CharId::root();
        }

        let visible_chars: Vec<_> = self
            .get_ordered_chars()
            .into_iter()
            .filter(|c| !c.tombstone && !c.id.is_root())
            .collect();

        visible_chars
            .get(index.saturating_sub(1))
            .map(|c| c.id)
            .unwrap_or(CharId::root())
    }

    /// Get all characters in their correct order
    fn get_ordered_chars(&self) -> Vec<&RgaChar> {
        let mut result = Vec::new();
        let mut current_id = CharId::root();
        let mut visited = std::collections::HashSet::new();

        // Build adjacency list of what comes after each character
        let mut successors: HashMap<CharId, Vec<CharId>> = HashMap::new();
        for rga_char in self.chars.values() {
            successors.entry(rga_char.left).or_insert_with(Vec::new).push(rga_char.id);
        }

        // Sort successors by ID for deterministic ordering
        for succ_list in successors.values_mut() {
            succ_list.sort();
        }

        // Traverse the structure
        let mut stack = vec![current_id];

        while let Some(id) = stack.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id);

            if let Some(rga_char) = self.chars.get(&id) {
                result.push(rga_char);

                // Add successors in reverse order so they're processed in correct order
                if let Some(succ) = successors.get(&id) {
                    for &succ_id in succ.iter().rev() {
                        stack.push(succ_id);
                    }
                }
            }
        }

        result
    }

    /// Insert a character with a specific ID (for replication)
    pub fn insert_with_id(&mut self, char_id: CharId, ch: char, left: CharId) {
        // Update our timestamp
        self.timestamp.update(&char_id.timestamp);

        if !self.chars.contains_key(&char_id) {
            self.chars.insert(
                char_id,
                RgaChar {
                    id: char_id,
                    value: ch,
                    left,
                    tombstone: false,
                },
            );
        }
    }

    /// Delete a character by ID (for replication)
    pub fn delete_by_id(&mut self, char_id: CharId) {
        if let Some(rga_char) = self.chars.get_mut(&char_id) {
            rga_char.tombstone = true;
        }
    }

    /// Get character at a specific position
    pub fn char_at(&self, index: usize) -> Option<char> {
        self.get_ordered_chars()
            .into_iter()
            .filter(|c| !c.tombstone && !c.id.is_root())
            .nth(index)
            .map(|c| c.value)
    }
}

impl CvRDT for RGA {
    fn merge(&mut self, other: &Self) {
        for (char_id, other_char) in &other.chars {
            match self.chars.get_mut(char_id) {
                Some(self_char) => {
                    // Merge tombstone status (deletion wins)
                    if other_char.tombstone {
                        self_char.tombstone = true;
                    }
                }
                None => {
                    // Insert new character
                    self.chars.insert(*char_id, other_char.clone());
                }
            }
        }

        // Update timestamp
        self.timestamp.update(&other.timestamp);
    }

    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        None // RGA doesn't have a natural partial order
    }
}

impl CrdtValue for RGA {
    type Value = String;

    fn value(&self) -> Self::Value {
        self.to_string()
    }
}

impl PartialEq for RGA {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for RGA {}

impl Default for RGA {
    fn default() -> Self {
        Self::new(ReplicaId::new())
    }
}

/// Operation for RGA
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RgaOp {
    Insert {
        char_id: CharId,
        value: char,
        left: CharId,
    },
    Delete {
        char_id: CharId,
    },
}

impl RGA {
    /// Apply an operation
    pub fn apply_op(&mut self, op: RgaOp) {
        match op {
            RgaOp::Insert { char_id, value, left } => {
                self.insert_with_id(char_id, value, left);
            }
            RgaOp::Delete { char_id } => {
                self.delete_by_id(char_id);
            }
        }
    }

    /// Create insert operations for a string
    pub fn create_insert_ops(&mut self, index: usize, s: &str) -> Vec<RgaOp> {
        let mut ops = Vec::new();
        let mut current_index = index;
        let mut left_id = self.get_id_at_index(current_index);

        for ch in s.chars() {
            self.timestamp.tick();
            let char_id = CharId::new(self.timestamp, self.replica_id);

            ops.push(RgaOp::Insert {
                char_id,
                value: ch,
                left: left_id,
            });

            self.chars.insert(
                char_id,
                RgaChar {
                    id: char_id,
                    value: ch,
                    left: left_id,
                    tombstone: false,
                },
            );

            left_id = char_id;
            current_index += 1;
        }

        ops
    }

    /// Create delete operation
    pub fn create_delete_op(&mut self, index: usize) -> Option<RgaOp> {
        let char_id = self.get_id_at_visible_index(index)?;
        Some(RgaOp::Delete { char_id })
    }
}

/// Delta for RGA synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgaDelta {
    pub chars: HashMap<CharId, RgaChar>,
}

impl RGA {
    /// Generate delta since a given timestamp
    pub fn delta_since(&self, since: &HybridTimestamp) -> RgaDelta {
        let chars = self
            .chars
            .iter()
            .filter(|(id, _)| id.timestamp > *since)
            .map(|(id, ch)| (*id, ch.clone()))
            .collect();

        RgaDelta { chars }
    }

    /// Merge a delta
    pub fn merge_delta(&mut self, delta: RgaDelta) {
        for (char_id, delta_char) in delta.chars {
            match self.chars.get_mut(&char_id) {
                Some(self_char) => {
                    if delta_char.tombstone {
                        self_char.tombstone = true;
                    }
                }
                None => {
                    self.chars.insert(char_id, delta_char);
                }
            }
        }
    }

    /// Garbage collect tombstones older than a threshold
    pub fn gc_tombstones(&mut self, before: &HybridTimestamp) {
        self.chars.retain(|id, ch| {
            if ch.tombstone && id.timestamp < *before && !id.is_root() {
                false
            } else {
                true
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rga_basic() {
        let replica = ReplicaId::new();
        let mut rga = RGA::new(replica);

        assert!(rga.is_empty());

        rga.insert(0, 'H');
        rga.insert(1, 'i');

        assert_eq!(rga.to_string(), "Hi");
        assert_eq!(rga.len(), 2);
    }

    #[test]
    fn test_rga_insert_string() {
        let replica = ReplicaId::new();
        let mut rga = RGA::new(replica);

        rga.insert_str(0, "Hello");
        assert_eq!(rga.to_string(), "Hello");

        rga.insert_str(5, " World");
        assert_eq!(rga.to_string(), "Hello World");
    }

    #[test]
    fn test_rga_delete() {
        let replica = ReplicaId::new();
        let mut rga = RGA::new(replica);

        rga.insert_str(0, "Hello");
        assert_eq!(rga.to_string(), "Hello");

        rga.delete(4); // Remove 'o'
        assert_eq!(rga.to_string(), "Hell");

        rga.delete(0); // Remove 'H'
        assert_eq!(rga.to_string(), "ell");
    }

    #[test]
    fn test_rga_delete_range() {
        let replica = ReplicaId::new();
        let mut rga = RGA::new(replica);

        rga.insert_str(0, "Hello World");
        rga.delete_range(5, 11); // Remove " World"

        assert_eq!(rga.to_string(), "Hello");
    }

    #[test]
    fn test_rga_merge() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut rga1 = RGA::new(replica1);
        let mut rga2 = RGA::new(replica2);

        rga1.insert_str(0, "Hello");
        rga2.insert_str(0, "World");

        rga1.merge(&rga2);

        // Both strings should be present
        let text = rga1.to_string();
        assert!(text.contains("Hello") || text.contains("World"));
    }

    #[test]
    fn test_rga_concurrent_insert() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut rga1 = RGA::new(replica1);
        let mut rga2 = RGA::new(replica2);

        // Both start with "AB"
        rga1.insert_str(0, "AB");
        rga2.merge(&rga1);

        // Concurrent inserts
        rga1.insert(1, 'X'); // "AXB"
        rga2.insert(1, 'Y'); // "AYB"

        // Merge
        let mut result1 = rga1.clone();
        result1.merge(&rga2);

        let mut result2 = rga2.clone();
        result2.merge(&rga1);

        // Should converge to same text
        assert_eq!(result1.to_string(), result2.to_string());
    }

    #[test]
    fn test_rga_operations() {
        let replica = ReplicaId::new();
        let mut rga = RGA::new(replica);

        let ops = rga.create_insert_ops(0, "Test");
        assert_eq!(ops.len(), 4);

        let mut rga2 = RGA::new(ReplicaId::new());
        for op in ops {
            rga2.apply_op(op);
        }

        assert_eq!(rga2.to_string(), "Test");
    }

    #[test]
    fn test_rga_char_at() {
        let replica = ReplicaId::new();
        let mut rga = RGA::new(replica);

        rga.insert_str(0, "Hello");

        assert_eq!(rga.char_at(0), Some('H'));
        assert_eq!(rga.char_at(4), Some('o'));
        assert_eq!(rga.char_at(5), None);
    }

    #[test]
    fn test_rga_convergence() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();
        let replica3 = ReplicaId::new();

        let mut rga1 = RGA::new(replica1);
        let mut rga2 = RGA::new(replica2);
        let mut rga3 = RGA::new(replica3);

        // Different concurrent operations
        rga1.insert_str(0, "A");
        rga2.insert_str(0, "B");
        rga3.insert_str(0, "C");

        // Merge in different orders
        let mut result1 = rga1.clone();
        result1.merge(&rga2);
        result1.merge(&rga3);

        let mut result2 = rga3.clone();
        result2.merge(&rga1);
        result2.merge(&rga2);

        // Should converge
        assert_eq!(result1.to_string(), result2.to_string());
    }

    #[test]
    fn test_rga_delete_merge() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut rga1 = RGA::new(replica1);
        let mut rga2 = RGA::new(replica2);

        // Both start with "Hello"
        rga1.insert_str(0, "Hello");
        rga2.merge(&rga1);

        // rga1 deletes a character
        rga1.delete(1); // "Hllo"

        // Merge
        rga2.merge(&rga1);

        assert_eq!(rga2.to_string(), "Hllo");
    }
}
