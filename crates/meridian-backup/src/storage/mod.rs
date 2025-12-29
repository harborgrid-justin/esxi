//! Storage backend abstraction for multi-cloud support.

pub mod s3;
pub mod gcs;
pub mod azure;

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::StorageResult;

/// Metadata associated with a stored object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetadata {
    /// Object key/path
    pub key: String,
    /// Size in bytes
    pub size: u64,
    /// Content type
    pub content_type: String,
    /// Last modified timestamp
    pub last_modified: chrono::DateTime<chrono::Utc>,
    /// ETag or checksum
    pub etag: Option<String>,
    /// Custom metadata
    pub custom_metadata: HashMap<String, String>,
}

/// Storage backend configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageConfig {
    S3 {
        region: String,
        bucket: String,
        endpoint: Option<String>,
        access_key: Option<String>,
        secret_key: Option<String>,
    },
    Gcs {
        project_id: String,
        bucket: String,
        credentials_path: Option<String>,
    },
    Azure {
        account_name: String,
        container: String,
        access_key: Option<String>,
        connection_string: Option<String>,
    },
}

/// Options for uploading objects.
#[derive(Debug, Clone, Default)]
pub struct UploadOptions {
    pub content_type: Option<String>,
    pub metadata: HashMap<String, String>,
    pub encryption: bool,
    pub storage_class: Option<String>,
}

/// Options for downloading objects.
#[derive(Debug, Clone, Default)]
pub struct DownloadOptions {
    pub range: Option<(u64, u64)>,
    pub verify_checksum: bool,
}

/// Options for listing objects.
#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub prefix: Option<String>,
    pub max_keys: Option<usize>,
    pub continuation_token: Option<String>,
}

/// Result of listing objects.
#[derive(Debug)]
pub struct ListResult {
    pub objects: Vec<ObjectMetadata>,
    pub continuation_token: Option<String>,
    pub is_truncated: bool,
}

/// Trait for storage backend implementations.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Upload an object to storage.
    async fn upload(
        &self,
        key: &str,
        data: Bytes,
        options: UploadOptions,
    ) -> StorageResult<ObjectMetadata>;

    /// Download an object from storage.
    async fn download(&self, key: &str, options: DownloadOptions) -> StorageResult<Bytes>;

    /// Delete an object from storage.
    async fn delete(&self, key: &str) -> StorageResult<()>;

    /// Check if an object exists.
    async fn exists(&self, key: &str) -> StorageResult<bool>;

    /// Get metadata for an object.
    async fn get_metadata(&self, key: &str) -> StorageResult<ObjectMetadata>;

    /// List objects in storage.
    async fn list(&self, options: ListOptions) -> StorageResult<ListResult>;

    /// Copy an object within storage.
    async fn copy(&self, source: &str, destination: &str) -> StorageResult<()>;

    /// Get a pre-signed URL for temporary access.
    async fn get_presigned_url(&self, key: &str, expires_in_secs: u64) -> StorageResult<String>;

    /// Initiate multipart upload for large objects.
    async fn initiate_multipart_upload(&self, key: &str) -> StorageResult<String>;

    /// Upload a part in a multipart upload.
    async fn upload_part(
        &self,
        key: &str,
        upload_id: &str,
        part_number: u32,
        data: Bytes,
    ) -> StorageResult<String>;

    /// Complete a multipart upload.
    async fn complete_multipart_upload(
        &self,
        key: &str,
        upload_id: &str,
        parts: Vec<(u32, String)>,
    ) -> StorageResult<ObjectMetadata>;

    /// Abort a multipart upload.
    async fn abort_multipart_upload(&self, key: &str, upload_id: &str) -> StorageResult<()>;
}

/// Create a storage backend from configuration.
pub async fn create_storage_backend(
    config: StorageConfig,
) -> StorageResult<Box<dyn StorageBackend>> {
    match config {
        StorageConfig::S3 { .. } => Ok(Box::new(s3::S3Backend::new(config).await?)),
        StorageConfig::Gcs { .. } => Ok(Box::new(gcs::GcsBackend::new(config).await?)),
        StorageConfig::Azure { .. } => Ok(Box::new(azure::AzureBackend::new(config).await?)),
    }
}
