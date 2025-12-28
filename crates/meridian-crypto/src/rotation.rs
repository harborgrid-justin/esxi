//! Key rotation and automatic re-encryption.
//!
//! This module provides key rotation capabilities with automatic re-encryption of data
//! encrypted with old keys.

use crate::envelope::{EncryptedEnvelope, EnvelopeEncryption};
use crate::error::{CryptoError, CryptoResult};
use crate::kms::KeyManagementService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Key rotation policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationPolicy {
    /// Rotation interval in days.
    pub rotation_interval_days: u32,

    /// Whether to automatically re-encrypt data after rotation.
    pub auto_reencrypt: bool,

    /// Maximum number of old key versions to retain.
    pub max_key_versions: u32,

    /// Grace period in days before old keys are deleted.
    pub grace_period_days: u32,

    /// Whether rotation is enabled.
    pub enabled: bool,
}

impl Default for KeyRotationPolicy {
    fn default() -> Self {
        Self {
            rotation_interval_days: 90, // Rotate every 90 days
            auto_reencrypt: true,
            max_key_versions: 5,
            grace_period_days: 30,
            enabled: true,
        }
    }
}

/// Key version information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyVersion {
    /// Key version identifier.
    pub version: u32,

    /// Key ID in the KMS.
    pub key_id: String,

    /// When this key version was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When this key version was rotated (if applicable).
    pub rotated_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Whether this is the current active version.
    pub is_active: bool,

    /// Whether this version is scheduled for deletion.
    pub deletion_scheduled: Option<chrono::DateTime<chrono::Utc>>,

    /// Number of envelopes encrypted with this key version.
    pub envelope_count: u64,
}

/// Key rotation manager.
pub struct KeyRotationManager {
    /// Envelope encryption service.
    envelope_encryption: EnvelopeEncryption,

    /// Key version tracking.
    key_versions: Arc<RwLock<HashMap<String, Vec<KeyVersion>>>>,

    /// Rotation policies by key ID.
    policies: Arc<RwLock<HashMap<String, KeyRotationPolicy>>>,

    /// Re-encryption queue.
    reencryption_queue: Arc<RwLock<Vec<ReencryptionTask>>>,
}

/// Re-encryption task.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReencryptionTask {
    /// Task ID.
    task_id: String,

    /// Envelope to re-encrypt.
    envelope_id: String,

    /// Old KEK ID.
    old_kek_id: String,

    /// New KEK ID.
    new_kek_id: String,

    /// Task creation time.
    created_at: chrono::DateTime<chrono::Utc>,

    /// Task status.
    status: ReencryptionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum ReencryptionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl KeyRotationManager {
    /// Create a new key rotation manager.
    pub fn new() -> Self {
        Self {
            envelope_encryption: EnvelopeEncryption::new(),
            key_versions: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
            reencryption_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Set rotation policy for a key.
    pub async fn set_policy(&self, key_id: &str, policy: KeyRotationPolicy) -> CryptoResult<()> {
        let mut policies = self.policies.write().await;
        policies.insert(key_id.to_string(), policy);
        Ok(())
    }

    /// Get rotation policy for a key.
    pub async fn get_policy(&self, key_id: &str) -> CryptoResult<KeyRotationPolicy> {
        let policies = self.policies.read().await;
        policies
            .get(key_id)
            .cloned()
            .ok_or_else(|| CryptoError::KeyNotFound(format!("No policy for key: {}", key_id)))
    }

    /// Initialize a new key with version tracking.
    pub async fn initialize_key(&self, key_id: &str) -> CryptoResult<()> {
        let mut versions = self.key_versions.write().await;

        let initial_version = KeyVersion {
            version: 1,
            key_id: key_id.to_string(),
            created_at: chrono::Utc::now(),
            rotated_at: None,
            is_active: true,
            deletion_scheduled: None,
            envelope_count: 0,
        };

        versions.insert(key_id.to_string(), vec![initial_version]);

        // Set default policy
        self.set_policy(key_id, KeyRotationPolicy::default()).await?;

        Ok(())
    }

    /// Rotate a key to a new version.
    pub async fn rotate_key(&self, old_key_id: &str, new_key_id: &str) -> CryptoResult<()> {
        let mut versions = self.key_versions.write().await;

        let key_versions = versions
            .get_mut(old_key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(format!("Key not found: {}", old_key_id)))?;

        // Mark all existing versions as inactive
        for version in key_versions.iter_mut() {
            version.is_active = false;
            version.rotated_at = Some(chrono::Utc::now());
        }

        // Create new version
        let new_version_num = key_versions.len() as u32 + 1;
        let new_version = KeyVersion {
            version: new_version_num,
            key_id: new_key_id.to_string(),
            created_at: chrono::Utc::now(),
            rotated_at: None,
            is_active: true,
            deletion_scheduled: None,
            envelope_count: 0,
        };

        key_versions.push(new_version);

        // Check policy for auto-reencryption
        let policy = self.get_policy(old_key_id).await?;
        if policy.auto_reencrypt {
            tracing::info!(
                "Scheduling automatic re-encryption for key: {}",
                old_key_id
            );
            // Re-encryption would be triggered here
        }

        // Schedule old versions for deletion if policy dictates
        let max_versions = policy.max_key_versions as usize;
        if key_versions.len() > max_versions {
            let versions_to_delete = key_versions.len() - max_versions;
            let grace_period = chrono::Duration::days(policy.grace_period_days as i64);
            let deletion_time = chrono::Utc::now() + grace_period;

            for version in key_versions.iter_mut().take(versions_to_delete) {
                if version.envelope_count == 0 {
                    version.deletion_scheduled = Some(deletion_time);
                }
            }
        }

        Ok(())
    }

    /// Re-encrypt an envelope with a new key.
    pub async fn reencrypt_envelope(
        &self,
        envelope: &EncryptedEnvelope,
        old_kek: &[u8],
        new_kek: &[u8],
        new_kek_id: &str,
    ) -> CryptoResult<EncryptedEnvelope> {
        self.envelope_encryption
            .rotate_kek(envelope, old_kek, new_kek, new_kek_id)
    }

    /// Schedule re-encryption for an envelope.
    pub async fn schedule_reencryption(
        &self,
        envelope_id: &str,
        old_kek_id: &str,
        new_kek_id: &str,
    ) -> CryptoResult<String> {
        let task = ReencryptionTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            envelope_id: envelope_id.to_string(),
            old_kek_id: old_kek_id.to_string(),
            new_kek_id: new_kek_id.to_string(),
            created_at: chrono::Utc::now(),
            status: ReencryptionStatus::Pending,
        };

        let task_id = task.task_id.clone();
        let mut queue = self.reencryption_queue.write().await;
        queue.push(task);

        Ok(task_id)
    }

    /// Process pending re-encryption tasks.
    pub async fn process_reencryption_queue<F>(
        &self,
        mut reencrypt_fn: F,
    ) -> CryptoResult<ProcessingStats>
    where
        F: FnMut(&ReencryptionTask) -> CryptoResult<()>,
    {
        let mut queue = self.reencryption_queue.write().await;
        let mut stats = ProcessingStats::default();

        for task in queue.iter_mut() {
            if task.status != ReencryptionStatus::Pending {
                continue;
            }

            task.status = ReencryptionStatus::InProgress;
            stats.total_tasks += 1;

            match reencrypt_fn(task) {
                Ok(_) => {
                    task.status = ReencryptionStatus::Completed;
                    stats.successful += 1;
                }
                Err(e) => {
                    task.status = ReencryptionStatus::Failed;
                    stats.failed += 1;
                    tracing::error!("Re-encryption task {} failed: {}", task.task_id, e);
                }
            }
        }

        // Remove completed tasks
        queue.retain(|task| task.status != ReencryptionStatus::Completed);

        Ok(stats)
    }

    /// Get all versions for a key.
    pub async fn get_key_versions(&self, key_id: &str) -> CryptoResult<Vec<KeyVersion>> {
        let versions = self.key_versions.read().await;
        versions
            .get(key_id)
            .cloned()
            .ok_or_else(|| CryptoError::KeyNotFound(format!("Key not found: {}", key_id)))
    }

    /// Get the active version for a key.
    pub async fn get_active_version(&self, key_id: &str) -> CryptoResult<KeyVersion> {
        let versions = self.get_key_versions(key_id).await?;
        versions
            .into_iter()
            .find(|v| v.is_active)
            .ok_or_else(|| CryptoError::KeyNotFound(format!("No active version for key: {}", key_id)))
    }

    /// Update envelope count for a key version.
    pub async fn increment_envelope_count(&self, key_id: &str, version: u32) -> CryptoResult<()> {
        let mut versions = self.key_versions.write().await;
        let key_versions = versions
            .get_mut(key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(format!("Key not found: {}", key_id)))?;

        if let Some(ver) = key_versions.iter_mut().find(|v| v.version == version) {
            ver.envelope_count += 1;
        }

        Ok(())
    }

    /// Decrement envelope count for a key version.
    pub async fn decrement_envelope_count(&self, key_id: &str, version: u32) -> CryptoResult<()> {
        let mut versions = self.key_versions.write().await;
        let key_versions = versions
            .get_mut(key_id)
            .ok_or_else(|| CryptoError::KeyNotFound(format!("Key not found: {}", key_id)))?;

        if let Some(ver) = key_versions.iter_mut().find(|v| v.version == version) {
            if ver.envelope_count > 0 {
                ver.envelope_count -= 1;
            }
        }

        Ok(())
    }

    /// Check if a key needs rotation based on policy.
    pub async fn needs_rotation(&self, key_id: &str) -> CryptoResult<bool> {
        let policy = self.get_policy(key_id).await?;
        if !policy.enabled {
            return Ok(false);
        }

        let active_version = self.get_active_version(key_id).await?;
        let age = chrono::Utc::now() - active_version.created_at;
        let rotation_interval = chrono::Duration::days(policy.rotation_interval_days as i64);

        Ok(age >= rotation_interval)
    }

    /// Get rotation statistics.
    pub async fn get_rotation_stats(&self, key_id: &str) -> CryptoResult<RotationStats> {
        let versions = self.get_key_versions(key_id).await?;
        let policy = self.get_policy(key_id).await?;

        let total_versions = versions.len();
        let active_version = versions.iter().find(|v| v.is_active);
        let scheduled_deletions = versions.iter().filter(|v| v.deletion_scheduled.is_some()).count();
        let total_envelopes: u64 = versions.iter().map(|v| v.envelope_count).sum();

        Ok(RotationStats {
            key_id: key_id.to_string(),
            total_versions,
            active_version_num: active_version.map(|v| v.version),
            scheduled_deletions,
            total_envelopes,
            rotation_interval_days: policy.rotation_interval_days,
            next_rotation_due: active_version.map(|v| {
                v.created_at + chrono::Duration::days(policy.rotation_interval_days as i64)
            }),
        })
    }
}

impl Default for KeyRotationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Re-encryption processing statistics.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProcessingStats {
    pub total_tasks: usize,
    pub successful: usize,
    pub failed: usize,
}

/// Key rotation statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationStats {
    pub key_id: String,
    pub total_versions: usize,
    pub active_version_num: Option<u32>,
    pub scheduled_deletions: usize,
    pub total_envelopes: u64,
    pub rotation_interval_days: u32,
    pub next_rotation_due: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_initialization() {
        let manager = KeyRotationManager::new();
        assert!(manager.initialize_key("test-key").await.is_ok());

        let versions = manager.get_key_versions("test-key").await.unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version, 1);
        assert!(versions[0].is_active);
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let manager = KeyRotationManager::new();
        manager.initialize_key("test-key").await.unwrap();

        manager.rotate_key("test-key", "new-key").await.unwrap();

        let versions = manager.get_key_versions("test-key").await.unwrap();
        assert_eq!(versions.len(), 2);

        let active = manager.get_active_version("test-key").await.unwrap();
        assert_eq!(active.version, 2);
        assert_eq!(active.key_id, "new-key");
    }

    #[tokio::test]
    async fn test_rotation_policy() {
        let manager = KeyRotationManager::new();
        manager.initialize_key("test-key").await.unwrap();

        let mut policy = KeyRotationPolicy::default();
        policy.rotation_interval_days = 30;
        policy.max_key_versions = 3;

        manager.set_policy("test-key", policy).await.unwrap();

        let retrieved_policy = manager.get_policy("test-key").await.unwrap();
        assert_eq!(retrieved_policy.rotation_interval_days, 30);
        assert_eq!(retrieved_policy.max_key_versions, 3);
    }
}
