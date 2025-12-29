//! Synchronization module with CRDT and operational transformation

pub mod crdt;
pub mod operational;
pub mod conflict;
pub mod vector_clock;

pub use crdt::{CrdtMap, CrdtSet, CrdtValue};
pub use operational::{Operation, OperationalTransform};
pub use conflict::{ConflictResolver, ConflictResolution, ConflictStrategy};
pub use vector_clock::VectorClock;

use crate::error::Result;

/// Synchronization trait for mergeable types
pub trait Mergeable: Clone + Send + Sync {
    /// Merge with another instance
    fn merge(&mut self, other: &Self) -> Result<()>;

    /// Check if this instance happened before another
    fn happens_before(&self, other: &Self) -> bool;
}

/// Synchronization engine
pub struct SyncEngine {
    /// Vector clock for causality tracking
    clock: VectorClock,

    /// Conflict resolver
    resolver: ConflictResolver,
}

impl SyncEngine {
    /// Create new sync engine
    pub fn new(node_id: String) -> Self {
        Self {
            clock: VectorClock::new(node_id.clone()),
            resolver: ConflictResolver::new(ConflictStrategy::LastWriteWins),
        }
    }

    /// Create with custom conflict strategy
    pub fn with_strategy(node_id: String, strategy: ConflictStrategy) -> Self {
        Self {
            clock: VectorClock::new(node_id),
            resolver: ConflictResolver::new(strategy),
        }
    }

    /// Get vector clock
    pub fn clock(&self) -> &VectorClock {
        &self.clock
    }

    /// Get mutable vector clock
    pub fn clock_mut(&mut self) -> &mut VectorClock {
        &mut self.clock
    }

    /// Get conflict resolver
    pub fn resolver(&self) -> &ConflictResolver {
        &self.resolver
    }

    /// Increment local clock
    pub fn tick(&mut self) {
        self.clock.increment();
    }

    /// Merge remote clock
    pub fn merge_clock(&mut self, remote: &VectorClock) {
        self.clock.merge(remote);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_engine() {
        let mut engine = SyncEngine::new("node1".to_string());
        assert_eq!(engine.clock().get("node1"), Some(0));

        engine.tick();
        assert_eq!(engine.clock().get("node1"), Some(1));
    }

    #[test]
    fn test_sync_engine_merge() {
        let mut engine1 = SyncEngine::new("node1".to_string());
        let mut engine2 = SyncEngine::new("node2".to_string());

        engine1.tick();
        engine2.tick();
        engine2.tick();

        engine1.merge_clock(engine2.clock());

        assert_eq!(engine1.clock().get("node1"), Some(1));
        assert_eq!(engine1.clock().get("node2"), Some(2));
    }
}
