//! # CRDT Module - Conflict-free Replicated Data Types
//!
//! This module provides enterprise-grade implementations of various CRDTs for
//! distributed state synchronization without coordination.
//!
//! ## CRDT Types
//!
//! - **State-based CRDTs (CvRDTs)**: Convergent Replicated Data Types
//!   - Merge operation must be commutative, associative, and idempotent
//!   - States form a monotonic semilattice
//!
//! - **Operation-based CRDTs (CmRDTs)**: Commutative Replicated Data Types
//!   - Operations must be commutative
//!   - Requires causal delivery of operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::hash::Hash;
use uuid::Uuid;

pub mod lww_register;
pub mod g_counter;
pub mod pn_counter;
pub mod g_set;
pub mod or_set;
pub mod lww_map;
pub mod rga;

pub use lww_register::LWWRegister;
pub use g_counter::GCounter;
pub use pn_counter::PNCounter;
pub use g_set::GSet;
pub use or_set::ORSet;
pub use lww_map::LWWMap;
pub use rga::RGA;

/// Unique identifier for a replica/node in the distributed system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ReplicaId(pub Uuid);

impl ReplicaId {
    /// Create a new random replica ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a replica ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for ReplicaId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ReplicaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lamport timestamp for causal ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LamportTimestamp {
    pub counter: u64,
    pub replica_id: ReplicaId,
}

impl LamportTimestamp {
    /// Create a new timestamp
    pub fn new(counter: u64, replica_id: ReplicaId) -> Self {
        Self { counter, replica_id }
    }

    /// Increment the timestamp
    pub fn increment(&mut self) {
        self.counter += 1;
    }

    /// Update timestamp based on received timestamp
    pub fn update(&mut self, other: &LamportTimestamp) {
        self.counter = self.counter.max(other.counter) + 1;
    }
}

impl PartialOrd for LamportTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LamportTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.counter.cmp(&other.counter) {
            Ordering::Equal => self.replica_id.cmp(&other.replica_id),
            ordering => ordering,
        }
    }
}

/// Hybrid Logical Clock for better timestamp ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HybridTimestamp {
    pub wall_clock: DateTime<Utc>,
    pub logical: u64,
    pub replica_id: ReplicaId,
}

impl HybridTimestamp {
    /// Create a new hybrid timestamp
    pub fn new(replica_id: ReplicaId) -> Self {
        Self {
            wall_clock: Utc::now(),
            logical: 0,
            replica_id,
        }
    }

    /// Create a timestamp with specific wall clock
    pub fn with_time(wall_clock: DateTime<Utc>, replica_id: ReplicaId) -> Self {
        Self {
            wall_clock,
            logical: 0,
            replica_id,
        }
    }

    /// Advance the timestamp
    pub fn tick(&mut self) {
        let now = Utc::now();
        if now > self.wall_clock {
            self.wall_clock = now;
            self.logical = 0;
        } else {
            self.logical += 1;
        }
    }

    /// Update based on received timestamp
    pub fn update(&mut self, other: &HybridTimestamp) {
        let now = Utc::now();
        let max_wall = self.wall_clock.max(other.wall_clock).max(now);

        if max_wall == self.wall_clock && max_wall == other.wall_clock {
            self.logical = self.logical.max(other.logical) + 1;
        } else if max_wall == self.wall_clock {
            self.logical += 1;
        } else if max_wall == other.wall_clock {
            self.logical = other.logical + 1;
        } else {
            self.logical = 0;
        }

        self.wall_clock = max_wall;
    }
}

impl PartialOrd for HybridTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HybridTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.wall_clock.cmp(&other.wall_clock) {
            Ordering::Equal => match self.logical.cmp(&other.logical) {
                Ordering::Equal => self.replica_id.cmp(&other.replica_id),
                ordering => ordering,
            },
            ordering => ordering,
        }
    }
}

/// Version Vector for tracking causal history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionVector {
    pub versions: std::collections::HashMap<ReplicaId, u64>,
}

impl VersionVector {
    /// Create a new empty version vector
    pub fn new() -> Self {
        Self {
            versions: std::collections::HashMap::new(),
        }
    }

    /// Get the version for a replica
    pub fn get(&self, replica_id: &ReplicaId) -> u64 {
        self.versions.get(replica_id).copied().unwrap_or(0)
    }

    /// Increment version for a replica
    pub fn increment(&mut self, replica_id: ReplicaId) {
        *self.versions.entry(replica_id).or_insert(0) += 1;
    }

    /// Update with another version vector (max of each component)
    pub fn merge(&mut self, other: &VersionVector) {
        for (replica_id, version) in &other.versions {
            let entry = self.versions.entry(*replica_id).or_insert(0);
            *entry = (*entry).max(*version);
        }
    }

    /// Check if this version vector causally precedes another
    pub fn precedes(&self, other: &VersionVector) -> bool {
        let mut strictly_less = false;

        // Check all replicas in self
        for (replica_id, version) in &self.versions {
            let other_version = other.get(replica_id);
            if *version > other_version {
                return false;
            }
            if *version < other_version {
                strictly_less = true;
            }
        }

        // Check replicas only in other
        for replica_id in other.versions.keys() {
            if !self.versions.contains_key(replica_id) {
                strictly_less = true;
            }
        }

        strictly_less
    }

    /// Check if two version vectors are concurrent (neither precedes the other)
    pub fn concurrent(&self, other: &VersionVector) -> bool {
        !self.precedes(other) && !other.precedes(self) && self != other
    }
}

impl Default for VersionVector {
    fn default() -> Self {
        Self::new()
    }
}

/// Core trait for State-based CRDTs (CvRDTs)
pub trait CvRDT: Clone {
    /// Merge two CRDT states
    /// Must be: commutative, associative, and idempotent
    fn merge(&mut self, other: &Self);

    /// Check if this state is a subset of another (partial order)
    fn partial_cmp(&self, other: &Self) -> Option<Ordering>;
}

/// Core trait for Operation-based CRDTs (CmRDTs)
pub trait CmRDT {
    /// Type of operations supported
    type Operation: Clone;

    /// Apply an operation to the CRDT
    fn apply(&mut self, operation: Self::Operation);

    /// Prepare operation for transmission (optional causality metadata)
    fn prepare_operation(&self, operation: Self::Operation) -> Self::Operation {
        operation
    }
}

/// Trait for CRDTs that support querying their value
pub trait CrdtValue {
    /// Type of the value contained in the CRDT
    type Value;

    /// Get the current value
    fn value(&self) -> Self::Value;
}

/// Trait for CRDTs that can be reset/cleared
pub trait Resettable {
    /// Reset the CRDT to initial state
    fn reset(&mut self);
}

/// Delta-state CRDT trait for efficient synchronization
pub trait DeltaCRDT: CvRDT {
    /// Type representing a delta (partial state update)
    type Delta: Clone;

    /// Generate a delta since a given version
    fn delta(&self, since: &VersionVector) -> Option<Self::Delta>;

    /// Merge a delta into the current state
    fn merge_delta(&mut self, delta: Self::Delta);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replica_id() {
        let id1 = ReplicaId::new();
        let id2 = ReplicaId::new();
        assert_ne!(id1, id2);

        let id3 = ReplicaId::from_uuid(id1.0);
        assert_eq!(id1, id3);
    }

    #[test]
    fn test_lamport_timestamp() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut ts1 = LamportTimestamp::new(0, replica1);
        let mut ts2 = LamportTimestamp::new(0, replica2);

        ts1.increment();
        assert_eq!(ts1.counter, 1);

        ts2.update(&ts1);
        assert_eq!(ts2.counter, 2);
    }

    #[test]
    fn test_hybrid_timestamp() {
        let replica = ReplicaId::new();
        let mut ts1 = HybridTimestamp::new(replica);
        let ts2 = ts1.clone();

        ts1.tick();
        assert!(ts1 > ts2);
    }

    #[test]
    fn test_version_vector() {
        let replica1 = ReplicaId::new();
        let replica2 = ReplicaId::new();

        let mut vv1 = VersionVector::new();
        vv1.increment(replica1);
        vv1.increment(replica1);

        let mut vv2 = VersionVector::new();
        vv2.increment(replica2);

        assert!(vv1.concurrent(&vv2));

        let mut vv3 = vv1.clone();
        vv3.increment(replica1);

        assert!(vv1.precedes(&vv3));
        assert!(!vv3.precedes(&vv1));
    }
}
