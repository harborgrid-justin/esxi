//! Conflict resolution strategies

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::sync::VectorClock;

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStrategy {
    /// Last Write Wins (LWW) - use vector clock
    LastWriteWins,

    /// First Write Wins
    FirstWriteWins,

    /// Manual resolution required
    Manual,

    /// Merge strategy (custom logic)
    Merge,

    /// Keep both versions
    KeepBoth,
}

/// Conflict resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution<T> {
    /// Use first value
    UseFirst(T),

    /// Use second value
    UseSecond(T),

    /// Use merged value
    UseMerged(T),

    /// Keep both values
    KeepBoth(T, T),

    /// Requires manual resolution
    RequiresManual(T, T),
}

/// Versioned value for conflict detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Versioned<T> {
    /// Value
    pub value: T,

    /// Vector clock
    pub clock: VectorClock,

    /// Timestamp
    pub timestamp: i64,

    /// User ID who made the change
    pub user_id: String,
}

impl<T> Versioned<T> {
    /// Create new versioned value
    pub fn new(value: T, clock: VectorClock, user_id: String) -> Self {
        Self {
            value,
            clock,
            timestamp: chrono::Utc::now().timestamp_millis(),
            user_id,
        }
    }

    /// Create with explicit timestamp
    pub fn with_timestamp(value: T, clock: VectorClock, user_id: String, timestamp: i64) -> Self {
        Self {
            value,
            clock,
            timestamp,
            user_id,
        }
    }
}

/// Conflict resolver
pub struct ConflictResolver {
    /// Resolution strategy
    strategy: ConflictStrategy,
}

impl ConflictResolver {
    /// Create new conflict resolver
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self { strategy }
    }

    /// Resolve conflict between two versioned values
    pub fn resolve<T: Clone>(
        &self,
        first: &Versioned<T>,
        second: &Versioned<T>,
    ) -> Result<ConflictResolution<T>> {
        // Check if there's actually a conflict
        if first.clock == second.clock {
            return Ok(ConflictResolution::UseFirst(first.value.clone()));
        }

        // Check causality
        if first.clock.happens_before(&second.clock) {
            return Ok(ConflictResolution::UseSecond(second.value.clone()));
        } else if second.clock.happens_before(&first.clock) {
            return Ok(ConflictResolution::UseFirst(first.value.clone()));
        }

        // Clocks are concurrent - use strategy
        match self.strategy {
            ConflictStrategy::LastWriteWins => {
                if first.timestamp > second.timestamp {
                    Ok(ConflictResolution::UseFirst(first.value.clone()))
                } else if second.timestamp > first.timestamp {
                    Ok(ConflictResolution::UseSecond(second.value.clone()))
                } else {
                    // Same timestamp - use user_id as tiebreaker
                    if first.user_id > second.user_id {
                        Ok(ConflictResolution::UseFirst(first.value.clone()))
                    } else {
                        Ok(ConflictResolution::UseSecond(second.value.clone()))
                    }
                }
            }

            ConflictStrategy::FirstWriteWins => {
                if first.timestamp < second.timestamp {
                    Ok(ConflictResolution::UseFirst(first.value.clone()))
                } else if second.timestamp < first.timestamp {
                    Ok(ConflictResolution::UseSecond(second.value.clone()))
                } else {
                    // Same timestamp - use user_id as tiebreaker
                    if first.user_id < second.user_id {
                        Ok(ConflictResolution::UseFirst(first.value.clone()))
                    } else {
                        Ok(ConflictResolution::UseSecond(second.value.clone()))
                    }
                }
            }

            ConflictStrategy::KeepBoth => Ok(ConflictResolution::KeepBoth(
                first.value.clone(),
                second.value.clone(),
            )),

            ConflictStrategy::Manual => Ok(ConflictResolution::RequiresManual(
                first.value.clone(),
                second.value.clone(),
            )),

            ConflictStrategy::Merge => {
                // Default merge strategy - use LWW
                // Custom merge logic should be implemented per type
                if first.timestamp > second.timestamp {
                    Ok(ConflictResolution::UseMerged(first.value.clone()))
                } else {
                    Ok(ConflictResolution::UseMerged(second.value.clone()))
                }
            }
        }
    }

    /// Get current strategy
    pub fn strategy(&self) -> ConflictStrategy {
        self.strategy
    }

    /// Set strategy
    pub fn set_strategy(&mut self, strategy: ConflictStrategy) {
        self.strategy = strategy;
    }
}

/// Conflict-free merge for specific types
pub trait ConflictFreeMerge: Clone {
    /// Merge two values without conflict
    fn merge(&self, other: &Self) -> Self;
}

// Implement for basic types
impl ConflictFreeMerge for String {
    fn merge(&self, other: &Self) -> Self {
        // Simple concatenation - real implementation would be smarter
        format!("{}\n{}", self, other)
    }
}

impl ConflictFreeMerge for Vec<u8> {
    fn merge(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.extend_from_slice(other);
        result
    }
}

impl<T: ConflictFreeMerge> ConflictFreeMerge for Vec<T> {
    fn merge(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.extend_from_slice(other);
        result
    }
}

/// Three-way merge
pub struct ThreeWayMerge<T> {
    /// Base version
    pub base: T,

    /// First modified version
    pub ours: T,

    /// Second modified version
    pub theirs: T,
}

impl<T: Clone + PartialEq> ThreeWayMerge<T> {
    /// Create new three-way merge
    pub fn new(base: T, ours: T, theirs: T) -> Self {
        Self { base, ours, theirs }
    }

    /// Check if there's a conflict
    pub fn has_conflict(&self) -> bool {
        self.ours != self.theirs && self.ours != self.base && self.theirs != self.base
    }

    /// Resolve with strategy
    pub fn resolve(&self, strategy: ConflictStrategy) -> Result<T> {
        if self.ours == self.theirs {
            return Ok(self.ours.clone());
        }

        if self.ours == self.base {
            return Ok(self.theirs.clone());
        }

        if self.theirs == self.base {
            return Ok(self.ours.clone());
        }

        // Real conflict
        match strategy {
            ConflictStrategy::LastWriteWins => Ok(self.theirs.clone()), // Assume theirs is newer
            ConflictStrategy::FirstWriteWins => Ok(self.ours.clone()),  // Assume ours is older
            _ => Err(Error::Conflict(
                "Three-way merge conflict requires manual resolution".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_resolver_lww() {
        let resolver = ConflictResolver::new(ConflictStrategy::LastWriteWins);

        let clock1 = VectorClock::new("node1".to_string());
        let clock2 = VectorClock::new("node2".to_string());

        let v1 = Versioned::with_timestamp("value1".to_string(), clock1, "user1".to_string(), 100);
        let v2 = Versioned::with_timestamp("value2".to_string(), clock2, "user2".to_string(), 200);

        let resolution = resolver.resolve(&v1, &v2).unwrap();

        match resolution {
            ConflictResolution::UseSecond(v) => assert_eq!(v, "value2"),
            _ => panic!("Expected UseSecond"),
        }
    }

    #[test]
    fn test_conflict_resolver_causality() {
        let resolver = ConflictResolver::new(ConflictStrategy::LastWriteWins);

        let mut clock1 = VectorClock::new("node1".to_string());
        clock1.increment();

        let mut clock2 = clock1.clone();
        clock2.increment();

        let v1 = Versioned::new("value1".to_string(), clock1, "user1".to_string());
        let v2 = Versioned::new("value2".to_string(), clock2, "user2".to_string());

        let resolution = resolver.resolve(&v1, &v2).unwrap();

        match resolution {
            ConflictResolution::UseSecond(v) => assert_eq!(v, "value2"),
            _ => panic!("Expected UseSecond due to causality"),
        }
    }

    #[test]
    fn test_three_way_merge() {
        let merge = ThreeWayMerge::new("base", "ours", "theirs");
        assert!(merge.has_conflict());

        let resolved = merge.resolve(ConflictStrategy::FirstWriteWins).unwrap();
        assert_eq!(resolved, "ours");
    }

    #[test]
    fn test_three_way_merge_no_conflict() {
        let merge = ThreeWayMerge::new("base", "modified", "modified");
        assert!(!merge.has_conflict());

        let resolved = merge.resolve(ConflictStrategy::LastWriteWins).unwrap();
        assert_eq!(resolved, "modified");
    }
}
