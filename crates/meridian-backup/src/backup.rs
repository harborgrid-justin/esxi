//! Main backup orchestration and coordination.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::compression::{CompressionAlgorithm, CompressionLevel, CompressionManager};
use crate::encryption::{EncryptionConfig, EncryptionManager};
use crate::error::Result;
use crate::failover::FailoverManager;
use crate::incremental::{BackupManifest, BackupType, IncrementalBackupManager};
use crate::pitr::{PitrConfig, PitrManager, RecoveryPoint, SnapshotType};
use crate::replication::{ReplicationManager, ReplicationPolicy};
use crate::retention::{RetentionManager, RetentionPolicy};
use crate::rto::{RtoMonitor, SlaConfig};
use crate::runbook::RunbookManager;
use crate::scheduler::BackupScheduler;
use crate::storage::{StorageBackend, StorageConfig};
use crate::verification::{VerificationLevel, VerificationManager, VerificationReport};

/// Overall backup configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    pub storage: StorageConfig,
    pub compression_algorithm: CompressionAlgorithm,
    pub compression_level: CompressionLevel,
    pub encryption_enabled: bool,
    pub encryption_config: Option<EncryptionConfig>,
    pub replication_policy: ReplicationPolicy,
    pub pitr_config: PitrConfig,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            storage: StorageConfig::S3 {
                region: "us-east-1".to_string(),
                bucket: "meridian-backups".to_string(),
                endpoint: None,
                access_key: None,
                secret_key: None,
            },
            compression_algorithm: CompressionAlgorithm::Zstd,
            compression_level: CompressionLevel::Balanced,
            encryption_enabled: true,
            encryption_config: Some(EncryptionConfig::default()),
            replication_policy: ReplicationPolicy::default(),
            pitr_config: PitrConfig::default(),
        }
    }
}

/// Main backup orchestrator.
pub struct BackupOrchestrator {
    config: BackupConfig,
    storage: Arc<Box<dyn StorageBackend>>,
    compression_manager: CompressionManager,
    encryption_manager: Option<EncryptionManager>,
    incremental_manager: Arc<RwLock<IncrementalBackupManager>>,
    pitr_manager: Arc<RwLock<PitrManager>>,
    replication_manager: Arc<ReplicationManager>,
    retention_manager: Arc<RwLock<RetentionManager>>,
    verification_manager: Arc<VerificationManager>,
    scheduler: Arc<BackupScheduler>,
    rto_monitor: Arc<RwLock<RtoMonitor>>,
    runbook_manager: Arc<RwLock<RunbookManager>>,
    failover_manager: Arc<RwLock<FailoverManager>>,
}

impl BackupOrchestrator {
    /// Create a new backup orchestrator.
    pub async fn new(config: BackupConfig) -> Result<Self> {
        let storage = Arc::new(crate::storage::create_storage_backend(config.storage.clone()).await?);

        let compression_manager =
            CompressionManager::new(config.compression_algorithm, config.compression_level);

        let encryption_manager = if config.encryption_enabled {
            if let Some(enc_config) = &config.encryption_config {
                let key = EncryptionManager::generate_key(enc_config.key_size);
                Some(EncryptionManager::new(enc_config.clone(), key)?)
            } else {
                None
            }
        } else {
            None
        };

        let incremental_manager = Arc::new(RwLock::new(IncrementalBackupManager::new(
            crate::storage::create_storage_backend(config.storage.clone()).await?,
            compression_manager.clone(),
        )));

        let pitr_manager = Arc::new(RwLock::new(PitrManager::new(
            config.pitr_config.clone(),
            crate::storage::create_storage_backend(config.storage.clone()).await?,
        )));

        let replication_manager = Arc::new(ReplicationManager::new(config.replication_policy.clone()));

        let retention_manager = Arc::new(RwLock::new(RetentionManager::new(
            crate::storage::create_storage_backend(config.storage.clone()).await?,
        )));

        let verification_manager = Arc::new(VerificationManager::new(
            crate::storage::create_storage_backend(config.storage.clone()).await?,
            compression_manager.clone(),
        ));

        let scheduler = Arc::new(BackupScheduler::new());
        let rto_monitor = Arc::new(RwLock::new(RtoMonitor::new()));
        let runbook_manager = Arc::new(RwLock::new(RunbookManager::new()));
        let failover_manager = Arc::new(RwLock::new(FailoverManager::new()));

        Ok(Self {
            config,
            storage,
            compression_manager,
            encryption_manager,
            incremental_manager,
            pitr_manager,
            replication_manager,
            retention_manager,
            verification_manager,
            scheduler,
            rto_monitor,
            runbook_manager,
            failover_manager,
        })
    }

    /// Create a full backup.
    pub async fn create_full_backup(
        &self,
        files: Vec<(PathBuf, Bytes)>,
    ) -> Result<BackupManifest> {
        let mut manager = self.incremental_manager.write().await;
        let manifest = manager.create_full_backup(files).await?;

        // Create recovery point
        let mut pitr = self.pitr_manager.write().await;
        pitr.create_recovery_point(
            &manifest,
            SnapshotType::Automatic,
            "Full backup recovery point".to_string(),
        )
        .await?;

        Ok(manifest)
    }

    /// Create an incremental backup.
    pub async fn create_incremental_backup(
        &self,
        parent_id: Uuid,
        files: Vec<(PathBuf, Bytes)>,
    ) -> Result<BackupManifest> {
        let mut manager = self.incremental_manager.write().await;
        let manifest = manager.create_incremental_backup(parent_id, files).await?;

        // Create recovery point
        let mut pitr = self.pitr_manager.write().await;
        pitr.create_recovery_point(
            &manifest,
            SnapshotType::Automatic,
            "Incremental backup recovery point".to_string(),
        )
        .await?;

        Ok(manifest)
    }

    /// Create a differential backup.
    pub async fn create_differential_backup(
        &self,
        base_full_id: Uuid,
        files: Vec<(PathBuf, Bytes)>,
    ) -> Result<BackupManifest> {
        let mut manager = self.incremental_manager.write().await;
        let manifest = manager.create_differential_backup(base_full_id, files).await?;

        // Create recovery point
        let mut pitr = self.pitr_manager.write().await;
        pitr.create_recovery_point(
            &manifest,
            SnapshotType::Automatic,
            "Differential backup recovery point".to_string(),
        )
        .await?;

        Ok(manifest)
    }

    /// Restore to a specific point in time.
    pub async fn restore_to_point(
        &self,
        target_time: chrono::DateTime<chrono::Utc>,
        sla_id: Option<Uuid>,
    ) -> Result<Vec<(PathBuf, Bytes)>> {
        // Start RTO measurement if SLA provided
        let measurement_id = if let Some(sla) = sla_id {
            let mut monitor = self.rto_monitor.write().await;
            Some(monitor.start_recovery_measurement(Uuid::new_v4(), sla)?)
        } else {
            None
        };

        let pitr = self.pitr_manager.read().await;
        let files = pitr.restore_to_point(target_time).await?;

        // Complete RTO measurement
        if let Some(mid) = measurement_id {
            let mut monitor = self.rto_monitor.write().await;
            monitor.complete_recovery_measurement(mid, target_time)?;
        }

        Ok(files)
    }

    /// Verify a backup.
    pub async fn verify_backup(
        &self,
        backup_id: Uuid,
        level: VerificationLevel,
    ) -> Result<VerificationReport> {
        self.verification_manager.verify_backup(backup_id, level).await
    }

    /// Apply retention policy.
    pub async fn apply_retention_policy(
        &self,
        policy: RetentionPolicy,
        backups: Vec<BackupManifest>,
    ) -> Result<()> {
        let mut manager = self.retention_manager.write().await;
        manager.add_policy(policy.clone());

        let decisions = manager.evaluate_retention(policy.id, &backups)?;
        manager.execute_retention(decisions).await?;

        Ok(())
    }

    /// Replicate backup to multiple regions.
    pub async fn replicate_backup(&self, key: &str, data: Bytes) -> Result<()> {
        self.replication_manager.replicate_object(key, data).await?;
        Ok(())
    }

    /// Get scheduler.
    pub fn scheduler(&self) -> Arc<BackupScheduler> {
        self.scheduler.clone()
    }

    /// Get RTO monitor.
    pub fn rto_monitor(&self) -> Arc<RwLock<RtoMonitor>> {
        self.rto_monitor.clone()
    }

    /// Get runbook manager.
    pub fn runbook_manager(&self) -> Arc<RwLock<RunbookManager>> {
        self.runbook_manager.clone()
    }

    /// Get failover manager.
    pub fn failover_manager(&self) -> Arc<RwLock<FailoverManager>> {
        self.failover_manager.clone()
    }

    /// Get overall system statistics.
    pub async fn get_statistics(&self) -> BackupStatistics {
        let scheduler_stats = self.scheduler.statistics().await;
        let rto_stats = self.rto_monitor.read().await.statistics();
        let runbook_stats = self.runbook_manager.read().await.statistics();
        let failover_stats = self.failover_manager.read().await.statistics();
        let replication_stats = self.replication_manager.statistics().await;
        let retention_stats = self.retention_manager.read().await.statistics();
        let pitr_stats = self.pitr_manager.read().await.statistics();

        BackupStatistics {
            scheduler: scheduler_stats,
            rto: rto_stats,
            runbook: runbook_stats,
            failover: failover_stats,
            replication: replication_stats,
            retention: retention_stats,
            pitr: pitr_stats,
        }
    }
}

/// Overall backup system statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatistics {
    pub scheduler: crate::scheduler::SchedulerStatistics,
    pub rto: crate::rto::RtoStatistics,
    pub runbook: crate::runbook::RunbookStatistics,
    pub failover: crate::failover::FailoverStatistics,
    pub replication: crate::replication::ReplicationStatistics,
    pub retention: crate::retention::RetentionStatistics,
    pub pitr: crate::pitr::PitrStatistics,
}
