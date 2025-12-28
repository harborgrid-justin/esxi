//! Point-in-time recovery (PITR) implementation.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use uuid::Uuid;

use crate::error::{BackupError, Result};
use crate::incremental::{BackupManifest, BackupType, FileMetadata};
use crate::storage::StorageBackend;

/// Recovery point representing a snapshot at a specific time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPoint {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub backup_id: Uuid,
    pub description: String,
    pub snapshot_type: SnapshotType,
    pub file_count: usize,
    pub total_size: u64,
}

/// Type of snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapshotType {
    Automatic,
    Manual,
    BeforeOperation,
    Scheduled,
}

/// PITR configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitrConfig {
    pub retention_hours: u64,
    pub snapshot_interval_minutes: u64,
    pub max_snapshots: usize,
    pub enable_wal: bool,
}

impl Default for PitrConfig {
    fn default() -> Self {
        Self {
            retention_hours: 168, // 7 days
            snapshot_interval_minutes: 60,
            max_snapshots: 1000,
            enable_wal: true,
        }
    }
}

/// Write-Ahead Log (WAL) entry for incremental changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: WalOperation,
    pub file_path: PathBuf,
    pub data: Option<Vec<u8>>,
    pub checksum: Vec<u8>,
}

/// WAL operation type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalOperation {
    Create,
    Update,
    Delete,
    Rename { old_path: PathBuf, new_path: PathBuf },
}

/// Point-in-time recovery manager.
pub struct PitrManager {
    config: PitrConfig,
    storage: Box<dyn StorageBackend>,
    recovery_points: BTreeMap<chrono::DateTime<chrono::Utc>, RecoveryPoint>,
    wal_entries: Vec<WalEntry>,
}

impl PitrManager {
    /// Create a new PITR manager.
    pub fn new(config: PitrConfig, storage: Box<dyn StorageBackend>) -> Self {
        Self {
            config,
            storage,
            recovery_points: BTreeMap::new(),
            wal_entries: Vec::new(),
        }
    }

    /// Create a recovery point.
    pub async fn create_recovery_point(
        &mut self,
        backup_manifest: &BackupManifest,
        snapshot_type: SnapshotType,
        description: String,
    ) -> Result<RecoveryPoint> {
        let recovery_point = RecoveryPoint {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            backup_id: backup_manifest.id,
            description,
            snapshot_type,
            file_count: backup_manifest.file_count,
            total_size: backup_manifest.total_size,
        };

        // Upload recovery point metadata
        let key = format!("pitr/recovery-points/{}.json", recovery_point.id);
        let data = serde_json::to_vec(&recovery_point)?;

        self.storage
            .upload(&key, Bytes::from(data), Default::default())
            .await?;

        self.recovery_points
            .insert(recovery_point.timestamp, recovery_point.clone());

        // Clean up old recovery points
        self.cleanup_old_recovery_points().await?;

        Ok(recovery_point)
    }

    /// Find recovery point at or before a specific time.
    pub fn find_recovery_point(
        &self,
        target_time: chrono::DateTime<chrono::Utc>,
    ) -> Option<&RecoveryPoint> {
        self.recovery_points
            .range(..=target_time)
            .next_back()
            .map(|(_, rp)| rp)
    }

    /// List recovery points within a time range.
    pub fn list_recovery_points(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<&RecoveryPoint> {
        self.recovery_points
            .range(start..=end)
            .map(|(_, rp)| rp)
            .collect()
    }

    /// Restore to a specific point in time.
    pub async fn restore_to_point(
        &self,
        target_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<(PathBuf, Bytes)>> {
        let recovery_point = self
            .find_recovery_point(target_time)
            .ok_or_else(|| {
                BackupError::Recovery(format!(
                    "No recovery point found for timestamp {}",
                    target_time
                ))
            })?;

        // Load the backup manifest
        let manifest = self.load_backup_manifest(recovery_point.backup_id).await?;

        // Reconstruct the file system state
        let mut files = self.restore_from_backup(&manifest).await?;

        // Apply WAL entries to reach the exact point in time
        if self.config.enable_wal {
            self.apply_wal_entries(&mut files, recovery_point.timestamp, target_time)
                .await?;
        }

        Ok(files)
    }

    /// Record a WAL entry.
    pub async fn record_wal_entry(&mut self, entry: WalEntry) -> Result<()> {
        // Upload WAL entry
        let key = format!("pitr/wal/{}.json", entry.id);
        let data = serde_json::to_vec(&entry)?;

        self.storage
            .upload(&key, Bytes::from(data), Default::default())
            .await?;

        self.wal_entries.push(entry);

        Ok(())
    }

    /// Apply WAL entries to reconstruct state at a specific time.
    async fn apply_wal_entries(
        &self,
        files: &mut Vec<(PathBuf, Bytes)>,
        from_time: chrono::DateTime<chrono::Utc>,
        to_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        let relevant_entries: Vec<_> = self
            .wal_entries
            .iter()
            .filter(|e| e.timestamp > from_time && e.timestamp <= to_time)
            .collect();

        for entry in relevant_entries {
            match &entry.operation {
                WalOperation::Create | WalOperation::Update => {
                    if let Some(data) = &entry.data {
                        // Update or add file
                        if let Some(pos) = files.iter().position(|(p, _)| p == &entry.file_path) {
                            files[pos].1 = Bytes::from(data.clone());
                        } else {
                            files.push((entry.file_path.clone(), Bytes::from(data.clone())));
                        }
                    }
                }
                WalOperation::Delete => {
                    // Remove file
                    files.retain(|(p, _)| p != &entry.file_path);
                }
                WalOperation::Rename { old_path, new_path } => {
                    // Rename file
                    if let Some(pos) = files.iter().position(|(p, _)| p == old_path) {
                        files[pos].0 = new_path.clone();
                    }
                }
            }
        }

        Ok(())
    }

    /// Restore files from a backup manifest.
    async fn restore_from_backup(&self, manifest: &BackupManifest) -> Result<Vec<(PathBuf, Bytes)>> {
        let mut files = Vec::new();

        for file_meta in &manifest.files {
            let key = format!("backups/{}/files/{}", manifest.id, file_meta.path.display());

            let data = self
                .storage
                .download(&key, Default::default())
                .await?;

            files.push((file_meta.path.clone(), data));
        }

        Ok(files)
    }

    /// Load a backup manifest from storage.
    async fn load_backup_manifest(&self, backup_id: Uuid) -> Result<BackupManifest> {
        let key = format!("backups/{}/manifest.json", backup_id);
        let data = self
            .storage
            .download(&key, Default::default())
            .await?;

        let manifest: BackupManifest = serde_json::from_slice(&data)?;
        Ok(manifest)
    }

    /// Clean up old recovery points based on retention policy.
    async fn cleanup_old_recovery_points(&mut self) -> Result<()> {
        let cutoff_time =
            chrono::Utc::now() - chrono::Duration::hours(self.config.retention_hours as i64);

        // Collect old recovery points
        let old_points: Vec<_> = self
            .recovery_points
            .range(..cutoff_time)
            .map(|(t, rp)| (*t, rp.id))
            .collect();

        // Delete old recovery points
        for (timestamp, id) in old_points {
            let key = format!("pitr/recovery-points/{}.json", id);
            self.storage.delete(&key).await?;
            self.recovery_points.remove(&timestamp);
        }

        // Enforce max snapshots limit
        while self.recovery_points.len() > self.config.max_snapshots {
            if let Some((timestamp, rp)) = self.recovery_points.iter().next() {
                let timestamp = *timestamp;
                let id = rp.id;

                let key = format!("pitr/recovery-points/{}.json", id);
                self.storage.delete(&key).await?;
                self.recovery_points.remove(&timestamp);
            }
        }

        Ok(())
    }

    /// Calculate Recovery Point Objective (RPO).
    pub fn calculate_rpo(&self) -> chrono::Duration {
        if self.recovery_points.is_empty() {
            return chrono::Duration::max_value();
        }

        let now = chrono::Utc::now();
        if let Some((latest_time, _)) = self.recovery_points.iter().next_back() {
            now - *latest_time
        } else {
            chrono::Duration::max_value()
        }
    }

    /// Get PITR statistics.
    pub fn statistics(&self) -> PitrStatistics {
        PitrStatistics {
            total_recovery_points: self.recovery_points.len(),
            oldest_recovery_point: self.recovery_points.keys().next().copied(),
            newest_recovery_point: self.recovery_points.keys().next_back().copied(),
            current_rpo: self.calculate_rpo(),
            wal_entries_count: self.wal_entries.len(),
        }
    }
}

/// PITR statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitrStatistics {
    pub total_recovery_points: usize,
    pub oldest_recovery_point: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_recovery_point: Option<chrono::DateTime<chrono::Utc>>,
    pub current_rpo: chrono::Duration,
    pub wal_entries_count: usize,
}
