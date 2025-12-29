//! Incremental and differential backup implementation.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use uuid::Uuid;

use crate::compression::{CompressionAlgorithm, CompressionManager, CompressedData};
use crate::error::Result;
use crate::storage::{StorageBackend, UploadOptions};

/// Backup type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BackupType {
    Full,
    Incremental,
    Differential,
}

/// File change type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}

/// File metadata for tracking changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub modified_time: chrono::DateTime<chrono::Utc>,
    pub checksum: Vec<u8>,
    pub permissions: u32,
}

/// File change record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub metadata: FileMetadata,
    pub change_type: ChangeType,
}

/// Backup manifest containing metadata about the backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub id: Uuid,
    pub backup_type: BackupType,
    pub parent_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub files: Vec<FileMetadata>,
    pub changes: Vec<FileChange>,
    pub total_size: u64,
    pub compressed_size: u64,
    pub file_count: usize,
    pub compression_algorithm: CompressionAlgorithm,
    pub encrypted: bool,
    pub checksum: Vec<u8>,
}

/// Incremental backup manager.
pub struct IncrementalBackupManager {
    storage: Box<dyn StorageBackend>,
    compression_manager: CompressionManager,
    manifests: HashMap<Uuid, BackupManifest>,
}

impl IncrementalBackupManager {
    /// Create a new incremental backup manager.
    pub fn new(
        storage: Box<dyn StorageBackend>,
        compression_manager: CompressionManager,
    ) -> Self {
        Self {
            storage,
            compression_manager,
            manifests: HashMap::new(),
        }
    }

    /// Create a full backup.
    pub async fn create_full_backup(
        &mut self,
        files: Vec<(PathBuf, Bytes)>,
    ) -> Result<BackupManifest> {
        let backup_id = Uuid::new_v4();
        let timestamp = chrono::Utc::now();

        let mut total_size = 0u64;
        let mut compressed_size = 0u64;
        let mut file_metadata = Vec::new();

        for (path, data) in files {
            let metadata = self.create_file_metadata(&path, &data);
            total_size += metadata.size;

            // Compress and upload
            let compressed = self.compression_manager.compress(&data)?;
            compressed_size += compressed.compressed_size as u64;

            let key = format!("backups/{}/files/{}", backup_id, path.display());
            self.upload_compressed_data(&key, &compressed).await?;

            file_metadata.push(metadata);
        }

        let file_count = file_metadata.len();
        let manifest = BackupManifest {
            id: backup_id,
            backup_type: BackupType::Full,
            parent_id: None,
            timestamp,
            files: file_metadata,
            changes: Vec::new(),
            total_size,
            compressed_size,
            file_count,
            compression_algorithm: CompressionAlgorithm::Zstd,
            encrypted: false,
            checksum: Vec::new(),
        };

        // Upload manifest
        self.upload_manifest(&manifest).await?;
        self.manifests.insert(backup_id, manifest.clone());

        Ok(manifest)
    }

    /// Create an incremental backup based on the last backup.
    pub async fn create_incremental_backup(
        &mut self,
        parent_id: Uuid,
        files: Vec<(PathBuf, Bytes)>,
    ) -> Result<BackupManifest> {
        let parent_manifest = self
            .manifests
            .get(&parent_id)
            .ok_or_else(|| crate::error::BackupError::BackupNotFound(parent_id.to_string()))?;

        let backup_id = Uuid::new_v4();
        let timestamp = chrono::Utc::now();

        // Detect changes
        let changes = self.detect_changes(parent_manifest, &files);

        let mut total_size = 0u64;
        let mut compressed_size = 0u64;
        let mut file_metadata = Vec::new();

        // Only backup changed files
        for change in &changes {
            if let ChangeType::Added | ChangeType::Modified = change.change_type {
                if let Some((_, data)) = files.iter().find(|(p, _)| p == &change.metadata.path) {
                    total_size += change.metadata.size;

                    let compressed = self.compression_manager.compress(data)?;
                    compressed_size += compressed.compressed_size as u64;

                    let key = format!(
                        "backups/{}/files/{}",
                        backup_id,
                        change.metadata.path.display()
                    );
                    self.upload_compressed_data(&key, &compressed).await?;

                    file_metadata.push(change.metadata.clone());
                }
            }
        }

        let file_count = file_metadata.len();
        let manifest = BackupManifest {
            id: backup_id,
            backup_type: BackupType::Incremental,
            parent_id: Some(parent_id),
            timestamp,
            files: file_metadata,
            changes,
            total_size,
            compressed_size,
            file_count,
            compression_algorithm: CompressionAlgorithm::Zstd,
            encrypted: false,
            checksum: Vec::new(),
        };

        self.upload_manifest(&manifest).await?;
        self.manifests.insert(backup_id, manifest.clone());

        Ok(manifest)
    }

    /// Create a differential backup based on the last full backup.
    pub async fn create_differential_backup(
        &mut self,
        base_full_id: Uuid,
        files: Vec<(PathBuf, Bytes)>,
    ) -> Result<BackupManifest> {
        let base_manifest = self
            .manifests
            .get(&base_full_id)
            .ok_or_else(|| crate::error::BackupError::BackupNotFound(base_full_id.to_string()))?;

        if base_manifest.backup_type != BackupType::Full {
            return Err(crate::error::BackupError::InvalidState(
                "Differential backups must be based on a full backup".to_string(),
            ));
        }

        let backup_id = Uuid::new_v4();
        let timestamp = chrono::Utc::now();

        // Detect changes from the base full backup
        let changes = self.detect_changes(base_manifest, &files);

        let mut total_size = 0u64;
        let mut compressed_size = 0u64;
        let mut file_metadata = Vec::new();

        for change in &changes {
            if let ChangeType::Added | ChangeType::Modified = change.change_type {
                if let Some((_, data)) = files.iter().find(|(p, _)| p == &change.metadata.path) {
                    total_size += change.metadata.size;

                    let compressed = self.compression_manager.compress(data)?;
                    compressed_size += compressed.compressed_size as u64;

                    let key = format!(
                        "backups/{}/files/{}",
                        backup_id,
                        change.metadata.path.display()
                    );
                    self.upload_compressed_data(&key, &compressed).await?;

                    file_metadata.push(change.metadata.clone());
                }
            }
        }

        let file_count = file_metadata.len();
        let manifest = BackupManifest {
            id: backup_id,
            backup_type: BackupType::Differential,
            parent_id: Some(base_full_id),
            timestamp,
            files: file_metadata,
            changes,
            total_size,
            compressed_size,
            file_count,
            compression_algorithm: CompressionAlgorithm::Zstd,
            encrypted: false,
            checksum: Vec::new(),
        };

        self.upload_manifest(&manifest).await?;
        self.manifests.insert(backup_id, manifest.clone());

        Ok(manifest)
    }

    /// Detect changes between backups.
    fn detect_changes(
        &self,
        parent: &BackupManifest,
        current_files: &[(PathBuf, Bytes)],
    ) -> Vec<FileChange> {
        let mut changes = Vec::new();

        // Create lookup for parent files
        let parent_files: HashMap<&PathBuf, &FileMetadata> =
            parent.files.iter().map(|f| (&f.path, f)).collect();

        let current_paths: HashSet<&PathBuf> = current_files.iter().map(|(p, _)| p).collect();

        // Check for added and modified files
        for (path, data) in current_files {
            let metadata = self.create_file_metadata(path, data);

            if let Some(parent_meta) = parent_files.get(path) {
                // File exists in parent, check if modified
                if metadata.checksum != parent_meta.checksum {
                    changes.push(FileChange {
                        metadata,
                        change_type: ChangeType::Modified,
                    });
                }
            } else {
                // New file
                changes.push(FileChange {
                    metadata,
                    change_type: ChangeType::Added,
                });
            }
        }

        // Check for deleted files
        for parent_file in &parent.files {
            if !current_paths.contains(&parent_file.path) {
                changes.push(FileChange {
                    metadata: parent_file.clone(),
                    change_type: ChangeType::Deleted,
                });
            }
        }

        changes
    }

    /// Create file metadata.
    fn create_file_metadata(&self, path: &PathBuf, data: &[u8]) -> FileMetadata {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(data);
        let checksum = hasher.finalize().to_vec();

        FileMetadata {
            path: path.clone(),
            size: data.len() as u64,
            modified_time: chrono::Utc::now(),
            checksum,
            permissions: 0o644,
        }
    }

    /// Upload compressed data to storage.
    async fn upload_compressed_data(&self, key: &str, compressed: &CompressedData) -> Result<()> {
        let data = serde_json::to_vec(&compressed)?;
        let options = UploadOptions::default();

        self.storage
            .upload(key, Bytes::from(data), options)
            .await?;

        Ok(())
    }

    /// Upload manifest to storage.
    async fn upload_manifest(&self, manifest: &BackupManifest) -> Result<()> {
        let key = format!("backups/{}/manifest.json", manifest.id);
        let data = serde_json::to_vec(&manifest)?;
        let options = UploadOptions::default();

        self.storage
            .upload(&key, Bytes::from(data), options)
            .await?;

        Ok(())
    }

    /// Load manifest from storage.
    pub async fn load_manifest(&mut self, backup_id: Uuid) -> Result<BackupManifest> {
        let key = format!("backups/{}/manifest.json", backup_id);
        let data = self
            .storage
            .download(&key, Default::default())
            .await?;

        let manifest: BackupManifest = serde_json::from_slice(&data)?;
        self.manifests.insert(backup_id, manifest.clone());

        Ok(manifest)
    }

    /// Get backup chain (from oldest to newest).
    pub fn get_backup_chain(&self, backup_id: Uuid) -> Result<Vec<Uuid>> {
        let mut chain = Vec::new();
        let mut current_id = Some(backup_id);

        while let Some(id) = current_id {
            chain.push(id);

            let manifest = self
                .manifests
                .get(&id)
                .ok_or_else(|| crate::error::BackupError::BackupNotFound(id.to_string()))?;

            current_id = manifest.parent_id;
        }

        chain.reverse();
        Ok(chain)
    }
}
