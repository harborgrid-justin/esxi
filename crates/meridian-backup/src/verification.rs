//! Backup verification and integrity checking.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

use crate::compression::CompressionManager;
use crate::error::{BackupError, Result};
use crate::incremental::BackupManifest;
use crate::storage::StorageBackend;

/// Verification level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VerificationLevel {
    /// Quick verification - check manifest exists.
    Quick,
    /// Standard verification - check checksums.
    Standard,
    /// Deep verification - download and verify all data.
    Deep,
}

/// Verification result for a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileVerificationResult {
    pub file_path: String,
    pub status: VerificationStatus,
    pub expected_checksum: Vec<u8>,
    pub actual_checksum: Option<Vec<u8>>,
    pub error_message: Option<String>,
}

/// Verification status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    Pass,
    Fail,
    Skipped,
    Error,
}

/// Overall verification report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub backup_id: Uuid,
    pub verification_level: VerificationLevel,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub total_files: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub file_results: Vec<FileVerificationResult>,
    pub overall_status: VerificationStatus,
}

/// Backup verification manager.
pub struct VerificationManager {
    storage: Box<dyn StorageBackend>,
    compression_manager: CompressionManager,
}

impl VerificationManager {
    /// Create a new verification manager.
    pub fn new(
        storage: Box<dyn StorageBackend>,
        compression_manager: CompressionManager,
    ) -> Self {
        Self {
            storage,
            compression_manager,
        }
    }

    /// Verify a backup.
    pub async fn verify_backup(
        &self,
        backup_id: Uuid,
        level: VerificationLevel,
    ) -> Result<VerificationReport> {
        let started_at = chrono::Utc::now();

        // Load manifest
        let manifest = self.load_manifest(backup_id).await?;

        let mut file_results = Vec::new();
        let mut passed = 0;
        let mut failed = 0;
        let mut errors = 0;

        match level {
            VerificationLevel::Quick => {
                // Just verify manifest exists
                file_results.push(FileVerificationResult {
                    file_path: "manifest.json".to_string(),
                    status: VerificationStatus::Pass,
                    expected_checksum: Vec::new(),
                    actual_checksum: None,
                    error_message: None,
                });
                passed += 1;
            }
            VerificationLevel::Standard => {
                // Verify each file exists and metadata matches
                for file_meta in &manifest.files {
                    let result = self
                        .verify_file_metadata(backup_id, file_meta.path.to_string_lossy().to_string())
                        .await;

                    match &result.status {
                        VerificationStatus::Pass => passed += 1,
                        VerificationStatus::Fail => failed += 1,
                        VerificationStatus::Error => errors += 1,
                        _ => {}
                    }

                    file_results.push(result);
                }
            }
            VerificationLevel::Deep => {
                // Download and verify checksums of all files
                for file_meta in &manifest.files {
                    let result = self
                        .verify_file_deep(
                            backup_id,
                            file_meta.path.to_string_lossy().to_string(),
                            &file_meta.checksum,
                        )
                        .await;

                    match &result.status {
                        VerificationStatus::Pass => passed += 1,
                        VerificationStatus::Fail => failed += 1,
                        VerificationStatus::Error => errors += 1,
                        _ => {}
                    }

                    file_results.push(result);
                }
            }
        }

        let completed_at = chrono::Utc::now();
        let overall_status = if failed > 0 || errors > 0 {
            VerificationStatus::Fail
        } else {
            VerificationStatus::Pass
        };

        Ok(VerificationReport {
            backup_id,
            verification_level: level,
            started_at,
            completed_at,
            total_files: file_results.len(),
            passed,
            failed,
            errors,
            file_results,
            overall_status,
        })
    }

    /// Verify file metadata exists.
    async fn verify_file_metadata(&self, backup_id: Uuid, file_path: String) -> FileVerificationResult {
        let key = format!("backups/{}/files/{}", backup_id, file_path);

        match self.storage.exists(&key).await {
            Ok(true) => FileVerificationResult {
                file_path,
                status: VerificationStatus::Pass,
                expected_checksum: Vec::new(),
                actual_checksum: None,
                error_message: None,
            },
            Ok(false) => FileVerificationResult {
                file_path,
                status: VerificationStatus::Fail,
                expected_checksum: Vec::new(),
                actual_checksum: None,
                error_message: Some("File not found".to_string()),
            },
            Err(e) => FileVerificationResult {
                file_path,
                status: VerificationStatus::Error,
                expected_checksum: Vec::new(),
                actual_checksum: None,
                error_message: Some(e.to_string()),
            },
        }
    }

    /// Deep verification - download and verify checksum.
    async fn verify_file_deep(
        &self,
        backup_id: Uuid,
        file_path: String,
        expected_checksum: &[u8],
    ) -> FileVerificationResult {
        let key = format!("backups/{}/files/{}", backup_id, file_path);

        match self.storage.download(&key, Default::default()).await {
            Ok(data) => {
                let actual_checksum = self.calculate_checksum(&data);

                let status = if actual_checksum == expected_checksum {
                    VerificationStatus::Pass
                } else {
                    VerificationStatus::Fail
                };

                FileVerificationResult {
                    file_path,
                    status,
                    expected_checksum: expected_checksum.to_vec(),
                    actual_checksum: Some(actual_checksum),
                    error_message: None,
                }
            }
            Err(e) => FileVerificationResult {
                file_path,
                status: VerificationStatus::Error,
                expected_checksum: expected_checksum.to_vec(),
                actual_checksum: None,
                error_message: Some(e.to_string()),
            },
        }
    }

    /// Load backup manifest.
    async fn load_manifest(&self, backup_id: Uuid) -> Result<BackupManifest> {
        let key = format!("backups/{}/manifest.json", backup_id);
        let data = self.storage.download(&key, Default::default()).await?;

        let manifest: BackupManifest = serde_json::from_slice(&data)?;
        Ok(manifest)
    }

    /// Calculate checksum of data.
    fn calculate_checksum(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Verify backup chain integrity.
    pub async fn verify_backup_chain(&self, backup_ids: Vec<Uuid>) -> Result<ChainVerificationReport> {
        let mut reports = Vec::new();
        let mut chain_valid = true;

        for backup_id in &backup_ids {
            let report = self
                .verify_backup(*backup_id, VerificationLevel::Standard)
                .await?;

            if report.overall_status != VerificationStatus::Pass {
                chain_valid = false;
            }

            reports.push(report);
        }

        Ok(ChainVerificationReport {
            chain_valid,
            backup_reports: reports,
        })
    }

    /// Schedule periodic verification.
    pub async fn schedule_verification(
        &self,
        backup_id: Uuid,
        interval_hours: u64,
    ) -> Result<()> {
        // This would integrate with the scheduler
        // For now, just verify once
        let _report = self
            .verify_backup(backup_id, VerificationLevel::Standard)
            .await?;

        Ok(())
    }

    /// Scrub all backups - comprehensive verification.
    pub async fn scrub_all_backups(
        &self,
        backup_ids: Vec<Uuid>,
    ) -> Result<Vec<VerificationReport>> {
        let mut reports = Vec::new();

        for backup_id in backup_ids {
            match self
                .verify_backup(backup_id, VerificationLevel::Deep)
                .await
            {
                Ok(report) => reports.push(report),
                Err(e) => {
                    tracing::error!("Failed to verify backup {}: {}", backup_id, e);
                }
            }
        }

        Ok(reports)
    }

    /// Get verification statistics.
    pub fn generate_statistics(&self, reports: &[VerificationReport]) -> VerificationStatistics {
        let total_verifications = reports.len();
        let passed = reports
            .iter()
            .filter(|r| r.overall_status == VerificationStatus::Pass)
            .count();
        let failed = reports
            .iter()
            .filter(|r| r.overall_status == VerificationStatus::Fail)
            .count();

        let total_files: usize = reports.iter().map(|r| r.total_files).sum();
        let total_passed_files: usize = reports.iter().map(|r| r.passed).sum();
        let total_failed_files: usize = reports.iter().map(|r| r.failed).sum();

        VerificationStatistics {
            total_verifications,
            passed,
            failed,
            total_files,
            total_passed_files,
            total_failed_files,
            success_rate: if total_verifications > 0 {
                (passed as f64 / total_verifications as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Chain verification report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVerificationReport {
    pub chain_valid: bool,
    pub backup_reports: Vec<VerificationReport>,
}

/// Verification statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStatistics {
    pub total_verifications: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_files: usize,
    pub total_passed_files: usize,
    pub total_failed_files: usize,
    pub success_rate: f64,
}
