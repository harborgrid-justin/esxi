//! Multi-region replication for disaster recovery.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{BackupError, Result};
use crate::storage::{StorageBackend, StorageConfig};

/// Replication region configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationRegion {
    pub name: String,
    pub priority: u8,
    pub storage_config: StorageConfig,
    pub enabled: bool,
}

/// Replication strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationStrategy {
    /// Synchronous replication - wait for all regions
    Synchronous,
    /// Asynchronous replication - fire and forget
    Asynchronous,
    /// Quorum - wait for majority of regions
    Quorum { min_regions: usize },
}

/// Replication policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationPolicy {
    pub strategy: ReplicationStrategy,
    pub min_regions: usize,
    pub max_regions: usize,
    pub retry_attempts: u32,
    pub retry_delay_secs: u64,
}

impl Default for ReplicationPolicy {
    fn default() -> Self {
        Self {
            strategy: ReplicationStrategy::Asynchronous,
            min_regions: 1,
            max_regions: 5,
            retry_attempts: 3,
            retry_delay_secs: 5,
        }
    }
}

/// Replication status for an object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStatus {
    pub object_key: String,
    pub regions: HashMap<String, RegionStatus>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_complete: bool,
}

/// Status of replication in a specific region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegionStatus {
    Pending,
    InProgress,
    Completed {
        timestamp: chrono::DateTime<chrono::Utc>,
        size: u64,
    },
    Failed {
        error: String,
        retry_count: u32,
    },
}

/// Multi-region replication manager.
pub struct ReplicationManager {
    policy: ReplicationPolicy,
    regions: Arc<RwLock<HashMap<String, ReplicationRegion>>>,
    storage_backends: Arc<RwLock<HashMap<String, Arc<dyn StorageBackend>>>>,
    replication_status: Arc<RwLock<HashMap<String, ReplicationStatus>>>,
}

impl ReplicationManager {
    /// Create a new replication manager.
    pub fn new(policy: ReplicationPolicy) -> Self {
        Self {
            policy,
            regions: Arc::new(RwLock::new(HashMap::new())),
            storage_backends: Arc::new(RwLock::new(HashMap::new())),
            replication_status: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a replication region.
    pub async fn add_region(&self, region: ReplicationRegion) -> Result<()> {
        let storage = crate::storage::create_storage_backend(region.storage_config.clone()).await?;

        let mut regions = self.regions.write().await;
        let mut backends = self.storage_backends.write().await;

        regions.insert(region.name.clone(), region.clone());
        // Convert Box<dyn StorageBackend> to Arc<dyn StorageBackend>
        backends.insert(region.name.clone(), Arc::from(storage));

        Ok(())
    }

    /// Remove a replication region.
    pub async fn remove_region(&self, region_name: &str) -> Result<()> {
        let mut regions = self.regions.write().await;
        let mut backends = self.storage_backends.write().await;

        regions.remove(region_name);
        backends.remove(region_name);

        Ok(())
    }

    /// Replicate an object to all enabled regions.
    pub async fn replicate_object(
        &self,
        key: &str,
        data: Bytes,
    ) -> Result<ReplicationStatus> {
        let regions = self.regions.read().await;
        let enabled_regions: Vec<_> = regions
            .values()
            .filter(|r| r.enabled)
            .cloned()
            .collect();

        if enabled_regions.len() < self.policy.min_regions {
            return Err(BackupError::Replication(format!(
                "Not enough enabled regions: {} < {}",
                enabled_regions.len(),
                self.policy.min_regions
            )));
        }

        drop(regions);

        let mut status = ReplicationStatus {
            object_key: key.to_string(),
            regions: HashMap::new(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            is_complete: false,
        };

        for region in &enabled_regions {
            status
                .regions
                .insert(region.name.clone(), RegionStatus::Pending);
        }

        match self.policy.strategy {
            ReplicationStrategy::Synchronous => {
                self.replicate_synchronous(key, data, &enabled_regions, &mut status)
                    .await?
            }
            ReplicationStrategy::Asynchronous => {
                self.replicate_asynchronous(key, data, &enabled_regions, &mut status)
                    .await?
            }
            ReplicationStrategy::Quorum { min_regions } => {
                self.replicate_quorum(key, data, &enabled_regions, &mut status, min_regions)
                    .await?
            }
        }

        status.completed_at = Some(chrono::Utc::now());
        status.is_complete = true;

        let mut statuses = self.replication_status.write().await;
        statuses.insert(key.to_string(), status.clone());

        Ok(status)
    }

    /// Synchronous replication - wait for all regions.
    async fn replicate_synchronous(
        &self,
        key: &str,
        data: Bytes,
        regions: &[ReplicationRegion],
        status: &mut ReplicationStatus,
    ) -> Result<()> {
        let backends = self.storage_backends.read().await;

        for region in regions {
            if let Some(backend) = backends.get(&region.name) {
                status
                    .regions
                    .insert(region.name.clone(), RegionStatus::InProgress);

                match self.upload_with_retry(backend.as_ref(), key, data.clone()).await {
                    Ok(size) => {
                        status.regions.insert(
                            region.name.clone(),
                            RegionStatus::Completed {
                                timestamp: chrono::Utc::now(),
                                size,
                            },
                        );
                    }
                    Err(e) => {
                        status.regions.insert(
                            region.name.clone(),
                            RegionStatus::Failed {
                                error: e.to_string(),
                                retry_count: self.policy.retry_attempts,
                            },
                        );
                        return Err(e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Asynchronous replication - fire and forget.
    async fn replicate_asynchronous(
        &self,
        key: &str,
        data: Bytes,
        regions: &[ReplicationRegion],
        _status: &mut ReplicationStatus,
    ) -> Result<()> {
        let backends = self.storage_backends.clone();
        let key = key.to_string();
        let policy = self.policy.clone();
        let _status_map = self.replication_status.clone();

        // Collect backends outside of spawn to avoid lifetime issues
        let mut backends_to_use = Vec::new();
        {
            let backends_guard = backends.read().await;
            for region in regions {
                if let Some(backend) = backends_guard.get(&region.name) {
                    backends_to_use.push((region.name.clone(), backend.clone()));
                }
            }
        } // backends_guard is dropped here

        // Spawn upload tasks
        for (_region_name, backend) in backends_to_use {
            let key = key.clone();
            let data = data.clone();
            let policy = policy.clone();

            tokio::spawn(async move {
                // Upload with retry logic
                for attempt in 0..=policy.retry_attempts {
                    match backend.upload(&key, data.clone(), Default::default()).await {
                        Ok(_) => break,
                        Err(_e) if attempt < policy.retry_attempts => {
                            tokio::time::sleep(tokio::time::Duration::from_secs(
                                policy.retry_delay_secs,
                            ))
                            .await;
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        Ok(())
    }

    /// Quorum replication - wait for minimum number of regions.
    async fn replicate_quorum(
        &self,
        key: &str,
        data: Bytes,
        regions: &[ReplicationRegion],
        status: &mut ReplicationStatus,
        min_regions: usize,
    ) -> Result<()> {
        let backends = self.storage_backends.read().await;
        let mut successful_replications = 0;

        for region in regions {
            if let Some(backend) = backends.get(&region.name) {
                status
                    .regions
                    .insert(region.name.clone(), RegionStatus::InProgress);

                match self.upload_with_retry(backend.as_ref(), key, data.clone()).await {
                    Ok(size) => {
                        status.regions.insert(
                            region.name.clone(),
                            RegionStatus::Completed {
                                timestamp: chrono::Utc::now(),
                                size,
                            },
                        );
                        successful_replications += 1;

                        if successful_replications >= min_regions {
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        status.regions.insert(
                            region.name.clone(),
                            RegionStatus::Failed {
                                error: e.to_string(),
                                retry_count: self.policy.retry_attempts,
                            },
                        );
                    }
                }
            }
        }

        if successful_replications < min_regions {
            Err(BackupError::Replication(format!(
                "Failed to replicate to minimum regions: {} < {}",
                successful_replications, min_regions
            )))
        } else {
            Ok(())
        }
    }

    /// Upload with retry logic.
    async fn upload_with_retry(
        &self,
        backend: &dyn StorageBackend,
        key: &str,
        data: Bytes,
    ) -> Result<u64> {
        let mut last_error = None;

        for attempt in 0..=self.policy.retry_attempts {
            match backend.upload(key, data.clone(), Default::default()).await {
                Ok(metadata) => return Ok(metadata.size),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.policy.retry_attempts {
                        tokio::time::sleep(tokio::time::Duration::from_secs(
                            self.policy.retry_delay_secs,
                        ))
                        .await;
                    }
                }
            }
        }

        Err(last_error.unwrap().into())
    }

    /// Get replication status for an object.
    pub async fn get_status(&self, key: &str) -> Option<ReplicationStatus> {
        let statuses = self.replication_status.read().await;
        statuses.get(key).cloned()
    }

    /// List all enabled regions.
    pub async fn list_regions(&self) -> Vec<ReplicationRegion> {
        let regions = self.regions.read().await;
        regions.values().filter(|r| r.enabled).cloned().collect()
    }

    /// Get replication statistics.
    pub async fn statistics(&self) -> ReplicationStatistics {
        let statuses = self.replication_status.read().await;
        let regions = self.regions.read().await;

        let total_objects = statuses.len();
        let completed_objects = statuses.values().filter(|s| s.is_complete).count();

        let mut region_stats = HashMap::new();
        for (region_name, _) in regions.iter() {
            let completed = statuses
                .values()
                .filter(|s| {
                    matches!(
                        s.regions.get(region_name),
                        Some(RegionStatus::Completed { .. })
                    )
                })
                .count();

            let failed = statuses
                .values()
                .filter(|s| {
                    matches!(s.regions.get(region_name), Some(RegionStatus::Failed { .. }))
                })
                .count();

            region_stats.insert(
                region_name.clone(),
                RegionStatistics { completed, failed },
            );
        }

        ReplicationStatistics {
            total_objects,
            completed_objects,
            total_regions: regions.len(),
            enabled_regions: regions.values().filter(|r| r.enabled).count(),
            region_stats,
        }
    }
}

/// Overall replication statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStatistics {
    pub total_objects: usize,
    pub completed_objects: usize,
    pub total_regions: usize,
    pub enabled_regions: usize,
    pub region_stats: HashMap<String, RegionStatistics>,
}

/// Per-region statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionStatistics {
    pub completed: usize,
    pub failed: usize,
}
