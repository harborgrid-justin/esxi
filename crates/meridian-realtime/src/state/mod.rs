//! State management with snapshots and delta synchronization

pub mod snapshot;
pub mod delta;

pub use snapshot::{Snapshot, SnapshotManager};
pub use delta::{Delta, DeltaManager, DeltaCompressor};

use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::sync::VectorClock;

/// State identifier
pub type StateId = String;

/// State metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetadata {
    /// State ID
    pub id: StateId,

    /// State version
    pub version: u64,

    /// Vector clock
    pub clock: VectorClock,

    /// Creation timestamp
    pub created_at: i64,

    /// Last modified timestamp
    pub modified_at: i64,

    /// State size in bytes
    pub size: usize,

    /// Checksum (for validation)
    pub checksum: String,
}

impl StateMetadata {
    /// Create new state metadata
    pub fn new(id: StateId, version: u64, clock: VectorClock, size: usize) -> Self {
        let now = chrono::Utc::now().timestamp_millis();

        Self {
            id,
            version,
            clock,
            created_at: now,
            modified_at: now,
            size,
            checksum: String::new(),
        }
    }

    /// With checksum
    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = checksum;
        self
    }

    /// Update modification time
    pub fn touch(&mut self) {
        self.modified_at = chrono::Utc::now().timestamp_millis();
    }
}

/// State manager for coordinating snapshots and deltas
pub struct StateManager {
    /// Snapshot manager
    snapshots: Arc<SnapshotManager>,

    /// Delta manager
    deltas: Arc<DeltaManager>,

    /// Current state version
    version: Arc<std::sync::atomic::AtomicU64>,
}

impl StateManager {
    /// Create new state manager
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(SnapshotManager::new(100)),
            deltas: Arc::new(DeltaManager::new(1000)),
            version: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Create with custom limits
    pub fn with_limits(max_snapshots: usize, max_deltas: usize) -> Self {
        Self {
            snapshots: Arc::new(SnapshotManager::new(max_snapshots)),
            deltas: Arc::new(DeltaManager::new(max_deltas)),
            version: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Get snapshot manager
    pub fn snapshots(&self) -> &SnapshotManager {
        &self.snapshots
    }

    /// Get delta manager
    pub fn deltas(&self) -> &DeltaManager {
        &self.deltas
    }

    /// Get current version
    pub fn current_version(&self) -> u64 {
        self.version.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Increment version
    pub fn increment_version(&self) -> u64 {
        self.version.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
    }

    /// Create snapshot from current state
    pub async fn create_snapshot(&self, state_id: StateId, data: Vec<u8>) -> Result<Snapshot> {
        let version = self.increment_version();
        let clock = VectorClock::new(state_id.clone());

        let snapshot = Snapshot::new(state_id, version, data, clock);
        self.snapshots.add_snapshot(snapshot.clone())?;

        Ok(snapshot)
    }

    /// Apply delta to state
    pub async fn apply_delta(&self, state_id: &str, delta: Delta) -> Result<()> {
        self.deltas.add_delta(delta)?;
        self.increment_version();
        Ok(())
    }

    /// Get state at specific version
    pub fn get_state_at_version(&self, state_id: &str, version: u64) -> Option<Vec<u8>> {
        // Get snapshot at or before version
        let snapshot = self.snapshots.get_snapshot_at_version(state_id, version)?;

        // Apply deltas from snapshot version to target version
        let deltas = self
            .deltas
            .get_deltas_in_range(state_id, snapshot.version(), version);

        if deltas.is_empty() {
            return Some(snapshot.data().to_vec());
        }

        // Apply deltas to reconstruct state
        let mut state = snapshot.data().to_vec();
        for delta in deltas {
            if let Some(new_state) = delta.apply(&state) {
                state = new_state;
            }
        }

        Some(state)
    }

    /// Optimize storage by creating snapshots
    pub fn optimize(&self, state_id: &str, snapshot_interval: usize) {
        // Create snapshot every N deltas
        let delta_count = self.deltas.delta_count_for_state(state_id);

        if delta_count >= snapshot_interval {
            // This would trigger snapshot creation in a real implementation
            tracing::info!("State {} should create snapshot", state_id);
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_metadata() {
        let clock = VectorClock::new("node1".to_string());
        let mut meta = StateMetadata::new("state1".to_string(), 1, clock, 1024);

        assert_eq!(meta.version, 1);
        assert_eq!(meta.size, 1024);

        let old_modified = meta.modified_at;
        std::thread::sleep(std::time::Duration::from_millis(10));
        meta.touch();

        assert!(meta.modified_at > old_modified);
    }

    #[tokio::test]
    async fn test_state_manager() {
        let manager = StateManager::new();
        assert_eq!(manager.current_version(), 0);

        let snapshot = manager
            .create_snapshot("state1".to_string(), vec![1, 2, 3])
            .await
            .unwrap();

        assert_eq!(snapshot.version(), 1);
        assert_eq!(manager.current_version(), 1);
    }
}
