//! State snapshots for efficient synchronization

use std::sync::Arc;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::state::StateId;
use crate::sync::VectorClock;

/// State snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Snapshot ID
    id: String,

    /// State ID
    state_id: StateId,

    /// Version
    version: u64,

    /// Snapshot data
    data: Vec<u8>,

    /// Vector clock
    clock: VectorClock,

    /// Created timestamp
    created_at: i64,

    /// Compressed
    compressed: bool,

    /// Checksum
    checksum: String,
}

impl Snapshot {
    /// Create new snapshot
    pub fn new(state_id: StateId, version: u64, data: Vec<u8>, clock: VectorClock) -> Self {
        let checksum = format!("{:x}", crc32fast::hash(&data));

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            state_id,
            version,
            data,
            clock,
            created_at: chrono::Utc::now().timestamp_millis(),
            compressed: false,
            checksum,
        }
    }

    /// Get snapshot ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get state ID
    pub fn state_id(&self) -> &str {
        &self.state_id
    }

    /// Get version
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Get data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get vector clock
    pub fn clock(&self) -> &VectorClock {
        &self.clock
    }

    /// Get created timestamp
    pub fn created_at(&self) -> i64 {
        self.created_at
    }

    /// Get size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Verify checksum
    pub fn verify(&self) -> bool {
        let calculated = format!("{:x}", crc32fast::hash(&self.data));
        calculated == self.checksum
    }

    /// Compress snapshot data
    pub fn compress(&mut self) -> Result<()> {
        if self.compressed {
            return Ok(());
        }

        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&self.data)
            .map_err(|e| Error::Internal(format!("Compression failed: {}", e)))?;

        let compressed_data = encoder
            .finish()
            .map_err(|e| Error::Internal(format!("Compression failed: {}", e)))?;

        self.data = compressed_data;
        self.compressed = true;

        Ok(())
    }

    /// Decompress snapshot data
    pub fn decompress(&mut self) -> Result<()> {
        if !self.compressed {
            return Ok(());
        }

        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(&self.data[..]);
        let mut decompressed_data = Vec::new();

        decoder
            .read_to_end(&mut decompressed_data)
            .map_err(|e| Error::Internal(format!("Decompression failed: {}", e)))?;

        self.data = decompressed_data;
        self.compressed = false;

        Ok(())
    }

    /// Clone snapshot data
    pub fn clone_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}

/// Snapshot manager
pub struct SnapshotManager {
    /// Snapshots by ID
    snapshots: Arc<DashMap<String, Arc<Snapshot>>>,

    /// Snapshots by state ID
    state_snapshots: Arc<DashMap<StateId, Vec<Arc<Snapshot>>>>,

    /// Maximum snapshots to keep
    max_snapshots: usize,
}

impl SnapshotManager {
    /// Create new snapshot manager
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: Arc::new(DashMap::new()),
            state_snapshots: Arc::new(DashMap::new()),
            max_snapshots,
        }
    }

    /// Add snapshot
    pub fn add_snapshot(&self, snapshot: Snapshot) -> Result<()> {
        let snapshot_id = snapshot.id().to_string();
        let state_id = snapshot.state_id().to_string();
        let snapshot = Arc::new(snapshot);

        // Add to snapshots map
        self.snapshots.insert(snapshot_id.clone(), snapshot.clone());

        // Add to state snapshots map
        let mut state_snaps = self.state_snapshots.entry(state_id.clone()).or_insert_with(Vec::new);
        state_snaps.push(snapshot.clone());

        // Sort by version
        state_snaps.sort_by_key(|s| s.version());

        // Limit snapshots per state
        if state_snaps.len() > self.max_snapshots {
            let excess = state_snaps.len() - self.max_snapshots;
            let removed: Vec<_> = state_snaps.drain(..excess).collect();

            // Remove from snapshots map
            for snapshot in removed {
                self.snapshots.remove(snapshot.id());
            }
        }

        Ok(())
    }

    /// Get snapshot by ID
    pub fn get_snapshot(&self, id: &str) -> Option<Arc<Snapshot>> {
        self.snapshots.get(id).map(|s| s.value().clone())
    }

    /// Get latest snapshot for state
    pub fn get_latest_snapshot(&self, state_id: &str) -> Option<Arc<Snapshot>> {
        self.state_snapshots
            .get(state_id)
            .and_then(|snaps| snaps.last().cloned())
    }

    /// Get snapshot at or before version
    pub fn get_snapshot_at_version(&self, state_id: &str, version: u64) -> Option<Arc<Snapshot>> {
        self.state_snapshots.get(state_id).and_then(|snaps| {
            snaps
                .iter()
                .filter(|s| s.version() <= version)
                .last()
                .cloned()
        })
    }

    /// Get all snapshots for state
    pub fn get_snapshots_for_state(&self, state_id: &str) -> Vec<Arc<Snapshot>> {
        self.state_snapshots
            .get(state_id)
            .map(|snaps| snaps.clone())
            .unwrap_or_default()
    }

    /// Remove snapshot
    pub fn remove_snapshot(&self, id: &str) -> Option<Arc<Snapshot>> {
        if let Some((_, snapshot)) = self.snapshots.remove(id) {
            // Remove from state snapshots
            if let Some(mut snaps) = self.state_snapshots.get_mut(snapshot.state_id()) {
                snaps.retain(|s| s.id() != id);
            }

            return Some(snapshot);
        }

        None
    }

    /// Remove all snapshots for state
    pub fn remove_state_snapshots(&self, state_id: &str) {
        if let Some((_, snaps)) = self.state_snapshots.remove(state_id) {
            for snapshot in snaps {
                self.snapshots.remove(snapshot.id());
            }
        }
    }

    /// Get snapshot count
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Get snapshot count for state
    pub fn snapshot_count_for_state(&self, state_id: &str) -> usize {
        self.state_snapshots
            .get(state_id)
            .map(|snaps| snaps.len())
            .unwrap_or(0)
    }

    /// Get total size of all snapshots
    pub fn total_size(&self) -> usize {
        self.snapshots.iter().map(|s| s.value().size()).sum()
    }

    /// Clear all snapshots
    pub fn clear(&self) {
        self.snapshots.clear();
        self.state_snapshots.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let clock = VectorClock::new("node1".to_string());
        let snapshot = Snapshot::new("state1".to_string(), 1, vec![1, 2, 3], clock);

        assert_eq!(snapshot.state_id(), "state1");
        assert_eq!(snapshot.version(), 1);
        assert_eq!(snapshot.data(), &[1, 2, 3]);
        assert_eq!(snapshot.size(), 3);
        assert!(snapshot.verify());
    }

    #[test]
    fn test_snapshot_compression() {
        let clock = VectorClock::new("node1".to_string());
        let data = vec![42u8; 1000];
        let mut snapshot = Snapshot::new("state1".to_string(), 1, data.clone(), clock);

        let original_size = snapshot.size();

        snapshot.compress().unwrap();
        assert!(snapshot.compressed);
        assert!(snapshot.size() < original_size);

        snapshot.decompress().unwrap();
        assert!(!snapshot.compressed);
        assert_eq!(snapshot.size(), original_size);
        assert_eq!(snapshot.data(), &data[..]);
    }

    #[test]
    fn test_snapshot_manager() {
        let manager = SnapshotManager::new(10);
        assert_eq!(manager.snapshot_count(), 0);

        let clock = VectorClock::new("node1".to_string());
        let snapshot = Snapshot::new("state1".to_string(), 1, vec![1, 2, 3], clock);
        let snapshot_id = snapshot.id().to_string();

        manager.add_snapshot(snapshot).unwrap();
        assert_eq!(manager.snapshot_count(), 1);

        let retrieved = manager.get_snapshot(&snapshot_id).unwrap();
        assert_eq!(retrieved.version(), 1);

        manager.remove_snapshot(&snapshot_id);
        assert_eq!(manager.snapshot_count(), 0);
    }

    #[test]
    fn test_snapshot_versioning() {
        let manager = SnapshotManager::new(10);

        for version in 1..=5 {
            let clock = VectorClock::new("node1".to_string());
            let snapshot = Snapshot::new(
                "state1".to_string(),
                version,
                vec![version as u8],
                clock,
            );
            manager.add_snapshot(snapshot).unwrap();
        }

        assert_eq!(manager.snapshot_count_for_state("state1"), 5);

        let latest = manager.get_latest_snapshot("state1").unwrap();
        assert_eq!(latest.version(), 5);

        let at_version_3 = manager.get_snapshot_at_version("state1", 3).unwrap();
        assert_eq!(at_version_3.version(), 3);
    }
}
