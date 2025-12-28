//! Backup retention policies and lifecycle management.

use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{BackupError, Result};
use crate::incremental::{BackupManifest, BackupType};
use crate::storage::StorageBackend;

/// Retention policy type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPolicyType {
    /// Time-based retention (delete after N days).
    TimeBased { days: u32 },
    /// Count-based retention (keep last N backups).
    CountBased { count: u32 },
    /// GFS (Grandfather-Father-Son) rotation.
    Gfs {
        daily: u32,
        weekly: u32,
        monthly: u32,
        yearly: u32,
    },
    /// Custom retention logic.
    Custom,
}

/// Retention policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: Uuid,
    pub name: String,
    pub policy_type: RetentionPolicyType,
    pub min_backups_to_keep: u32,
    pub max_backups_to_keep: Option<u32>,
    pub enabled: bool,
}

/// Retention action to be taken on a backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionAction {
    Keep,
    Delete,
    Archive,
    Transition { to_storage_class: String },
}

/// Retention decision for a backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionDecision {
    pub backup_id: Uuid,
    pub action: RetentionAction,
    pub reason: String,
}

/// Retention policy manager.
pub struct RetentionManager {
    policies: HashMap<Uuid, RetentionPolicy>,
    storage: Box<dyn StorageBackend>,
}

impl RetentionManager {
    /// Create a new retention manager.
    pub fn new(storage: Box<dyn StorageBackend>) -> Self {
        Self {
            policies: HashMap::new(),
            storage,
        }
    }

    /// Add a retention policy.
    pub fn add_policy(&mut self, policy: RetentionPolicy) {
        self.policies.insert(policy.id, policy);
    }

    /// Remove a retention policy.
    pub fn remove_policy(&mut self, policy_id: Uuid) -> Result<()> {
        self.policies
            .remove(&policy_id)
            .ok_or_else(|| BackupError::BackupNotFound(policy_id.to_string()))?;
        Ok(())
    }

    /// Evaluate retention policy for backups.
    pub fn evaluate_retention(
        &self,
        policy_id: Uuid,
        backups: &[BackupManifest],
    ) -> Result<Vec<RetentionDecision>> {
        let policy = self
            .policies
            .get(&policy_id)
            .ok_or_else(|| BackupError::BackupNotFound(policy_id.to_string()))?;

        if !policy.enabled {
            return Ok(Vec::new());
        }

        match &policy.policy_type {
            RetentionPolicyType::TimeBased { days } => {
                self.evaluate_time_based(backups, *days, policy)
            }
            RetentionPolicyType::CountBased { count } => {
                self.evaluate_count_based(backups, *count, policy)
            }
            RetentionPolicyType::Gfs {
                daily,
                weekly,
                monthly,
                yearly,
            } => self.evaluate_gfs(backups, *daily, *weekly, *monthly, *yearly, policy),
            RetentionPolicyType::Custom => self.evaluate_custom(backups, policy),
        }
    }

    /// Evaluate time-based retention.
    fn evaluate_time_based(
        &self,
        backups: &[BackupManifest],
        retention_days: u32,
        policy: &RetentionPolicy,
    ) -> Result<Vec<RetentionDecision>> {
        let now = chrono::Utc::now();
        let cutoff =
            now - chrono::Duration::days(retention_days as i64);

        let mut decisions = Vec::new();
        let mut kept_count = 0;

        // Sort backups by timestamp (newest first)
        let mut sorted_backups = backups.to_vec();
        sorted_backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        for backup in sorted_backups {
            if backup.timestamp < cutoff && kept_count >= policy.min_backups_to_keep {
                decisions.push(RetentionDecision {
                    backup_id: backup.id,
                    action: RetentionAction::Delete,
                    reason: format!(
                        "Backup older than {} days and minimum backups requirement met",
                        retention_days
                    ),
                });
            } else {
                decisions.push(RetentionDecision {
                    backup_id: backup.id,
                    action: RetentionAction::Keep,
                    reason: "Within retention period".to_string(),
                });
                kept_count += 1;
            }
        }

        Ok(decisions)
    }

    /// Evaluate count-based retention.
    fn evaluate_count_based(
        &self,
        backups: &[BackupManifest],
        max_count: u32,
        _policy: &RetentionPolicy,
    ) -> Result<Vec<RetentionDecision>> {
        let mut decisions = Vec::new();

        // Sort backups by timestamp (newest first)
        let mut sorted_backups = backups.to_vec();
        sorted_backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        for (index, backup) in sorted_backups.iter().enumerate() {
            if index < max_count as usize {
                decisions.push(RetentionDecision {
                    backup_id: backup.id,
                    action: RetentionAction::Keep,
                    reason: format!("Within top {} backups", max_count),
                });
            } else {
                decisions.push(RetentionDecision {
                    backup_id: backup.id,
                    action: RetentionAction::Delete,
                    reason: format!("Exceeds maximum count of {}", max_count),
                });
            }
        }

        Ok(decisions)
    }

    /// Evaluate GFS (Grandfather-Father-Son) retention.
    fn evaluate_gfs(
        &self,
        backups: &[BackupManifest],
        daily: u32,
        weekly: u32,
        monthly: u32,
        yearly: u32,
        _policy: &RetentionPolicy,
    ) -> Result<Vec<RetentionDecision>> {
        let now = chrono::Utc::now();
        let mut decisions = Vec::new();

        // Sort backups by timestamp (newest first)
        let mut sorted_backups = backups.to_vec();
        sorted_backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let mut daily_kept = 0;
        let mut weekly_kept = 0;
        let mut monthly_kept = 0;
        let mut yearly_kept = 0;

        for backup in sorted_backups {
            let age = now - backup.timestamp;
            let mut kept = false;

            // Check if it's a yearly backup (first of year)
            if backup.timestamp.month() == 1
                && backup.timestamp.day() <= 7
                && yearly_kept < yearly
            {
                decisions.push(RetentionDecision {
                    backup_id: backup.id,
                    action: RetentionAction::Keep,
                    reason: "Yearly backup".to_string(),
                });
                yearly_kept += 1;
                kept = true;
            }
            // Check if it's a monthly backup (first week of month)
            else if backup.timestamp.day() <= 7 && monthly_kept < monthly && !kept {
                decisions.push(RetentionDecision {
                    backup_id: backup.id,
                    action: RetentionAction::Keep,
                    reason: "Monthly backup".to_string(),
                });
                monthly_kept += 1;
                kept = true;
            }
            // Check if it's a weekly backup (Sunday)
            else if backup.timestamp.weekday() == chrono::Weekday::Sun
                && weekly_kept < weekly
                && !kept
            {
                decisions.push(RetentionDecision {
                    backup_id: backup.id,
                    action: RetentionAction::Keep,
                    reason: "Weekly backup".to_string(),
                });
                weekly_kept += 1;
                kept = true;
            }
            // Check if it's a daily backup
            else if age.num_days() < daily as i64 && daily_kept < daily && !kept {
                decisions.push(RetentionDecision {
                    backup_id: backup.id,
                    action: RetentionAction::Keep,
                    reason: "Daily backup".to_string(),
                });
                daily_kept += 1;
                kept = true;
            }

            if !kept {
                decisions.push(RetentionDecision {
                    backup_id: backup.id,
                    action: RetentionAction::Delete,
                    reason: "Does not meet GFS retention criteria".to_string(),
                });
            }
        }

        Ok(decisions)
    }

    /// Evaluate custom retention (placeholder).
    fn evaluate_custom(
        &self,
        backups: &[BackupManifest],
        _policy: &RetentionPolicy,
    ) -> Result<Vec<RetentionDecision>> {
        // Custom logic would go here
        let decisions: Vec<_> = backups
            .iter()
            .map(|backup| RetentionDecision {
                backup_id: backup.id,
                action: RetentionAction::Keep,
                reason: "Custom policy - keeping all".to_string(),
            })
            .collect();

        Ok(decisions)
    }

    /// Execute retention decisions.
    pub async fn execute_retention(
        &self,
        decisions: Vec<RetentionDecision>,
    ) -> Result<RetentionReport> {
        let mut report = RetentionReport {
            total_processed: 0,
            kept: 0,
            deleted: 0,
            archived: 0,
            transitioned: 0,
            errors: Vec::new(),
        };

        for decision in decisions {
            report.total_processed += 1;

            match decision.action {
                RetentionAction::Keep => {
                    report.kept += 1;
                }
                RetentionAction::Delete => {
                    match self.delete_backup(decision.backup_id).await {
                        Ok(_) => report.deleted += 1,
                        Err(e) => report.errors.push(format!(
                            "Failed to delete backup {}: {}",
                            decision.backup_id, e
                        )),
                    }
                }
                RetentionAction::Archive => {
                    // Archive logic would go here
                    report.archived += 1;
                }
                RetentionAction::Transition { .. } => {
                    // Transition logic would go here
                    report.transitioned += 1;
                }
            }
        }

        Ok(report)
    }

    /// Delete a backup and all its files.
    async fn delete_backup(&self, backup_id: Uuid) -> Result<()> {
        // Load manifest
        let key = format!("backups/{}/manifest.json", backup_id);
        let manifest_data = self.storage.download(&key, Default::default()).await?;
        let manifest: BackupManifest = serde_json::from_slice(&manifest_data)?;

        // Delete all files
        for file_meta in &manifest.files {
            let file_key = format!("backups/{}/files/{}", backup_id, file_meta.path.display());
            if let Err(e) = self.storage.delete(&file_key).await {
                tracing::warn!("Failed to delete file {}: {}", file_key, e);
            }
        }

        // Delete manifest
        self.storage.delete(&key).await?;

        Ok(())
    }

    /// Get retention statistics.
    pub fn statistics(&self) -> RetentionStatistics {
        RetentionStatistics {
            total_policies: self.policies.len(),
            enabled_policies: self.policies.values().filter(|p| p.enabled).count(),
        }
    }
}

/// Report of retention execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionReport {
    pub total_processed: usize,
    pub kept: usize,
    pub deleted: usize,
    pub archived: usize,
    pub transitioned: usize,
    pub errors: Vec<String>,
}

/// Retention statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionStatistics {
    pub total_policies: usize,
    pub enabled_policies: usize,
}

/// Pre-configured retention policies.
pub struct RetentionPolicyTemplates;

impl RetentionPolicyTemplates {
    /// 7-day retention policy.
    pub fn seven_days() -> RetentionPolicy {
        RetentionPolicy {
            id: Uuid::new_v4(),
            name: "7 Days".to_string(),
            policy_type: RetentionPolicyType::TimeBased { days: 7 },
            min_backups_to_keep: 1,
            max_backups_to_keep: None,
            enabled: true,
        }
    }

    /// 30-day retention policy.
    pub fn thirty_days() -> RetentionPolicy {
        RetentionPolicy {
            id: Uuid::new_v4(),
            name: "30 Days".to_string(),
            policy_type: RetentionPolicyType::TimeBased { days: 30 },
            min_backups_to_keep: 1,
            max_backups_to_keep: None,
            enabled: true,
        }
    }

    /// Keep last 10 backups.
    pub fn last_ten() -> RetentionPolicy {
        RetentionPolicy {
            id: Uuid::new_v4(),
            name: "Last 10 Backups".to_string(),
            policy_type: RetentionPolicyType::CountBased { count: 10 },
            min_backups_to_keep: 1,
            max_backups_to_keep: Some(10),
            enabled: true,
        }
    }

    /// Standard GFS policy.
    pub fn gfs_standard() -> RetentionPolicy {
        RetentionPolicy {
            id: Uuid::new_v4(),
            name: "GFS Standard".to_string(),
            policy_type: RetentionPolicyType::Gfs {
                daily: 7,
                weekly: 4,
                monthly: 12,
                yearly: 5,
            },
            min_backups_to_keep: 1,
            max_backups_to_keep: None,
            enabled: true,
        }
    }
}
