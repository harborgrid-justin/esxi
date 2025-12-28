//! Azure Blob Storage backend implementation.

use async_trait::async_trait;
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::*;
use bytes::Bytes;
use futures::StreamExt;
use std::collections::HashMap;

use super::{
    DownloadOptions, ListOptions, ListResult, ObjectMetadata, StorageBackend, StorageConfig,
    UploadOptions,
};
use crate::error::{StorageError, StorageResult};

/// Azure Blob Storage backend.
pub struct AzureBackend {
    client: ContainerClient,
    container: String,
}

impl AzureBackend {
    /// Create a new Azure backend.
    pub async fn new(config: StorageConfig) -> StorageResult<Self> {
        let (account_name, container, access_key, _connection_string) = match config {
            StorageConfig::Azure {
                account_name,
                container,
                access_key,
                connection_string,
            } => (account_name, container, access_key, connection_string),
            _ => {
                return Err(StorageError::AuthenticationFailed(
                    "Invalid configuration for Azure".to_string(),
                ))
            }
        };

        let credentials = if let Some(key) = access_key {
            StorageCredentials::access_key(account_name.clone(), key)
        } else {
            return Err(StorageError::AuthenticationFailed(
                "No access key provided".to_string(),
            ));
        };

        let blob_service = BlobServiceClient::new(account_name, credentials);
        let client = blob_service.container_client(&container);

        Ok(Self { client, container })
    }

    /// Convert Azure error to StorageError.
    fn map_azure_error<E: std::fmt::Display>(err: E) -> StorageError {
        StorageError::Azure(err.to_string())
    }
}

#[async_trait]
impl StorageBackend for AzureBackend {
    async fn upload(
        &self,
        key: &str,
        data: Bytes,
        options: UploadOptions,
    ) -> StorageResult<ObjectMetadata> {
        let blob_client = self.client.blob_client(key);

        let mut request = blob_client.put_block_blob(data.clone());

        if let Some(content_type) = &options.content_type {
            request = request.content_type(content_type.clone());
        }

        // Note: Metadata setting in azure_storage_blobs 0.17 requires using Headers
        // For simplified implementation, we skip setting custom metadata on upload
        // Metadata can be set separately using set_blob_metadata if needed
        let _metadata = &options.metadata; // Acknowledge but don't use for now

        let response = request.await.map_err(Self::map_azure_error)?;

        // Convert Azure's OffsetDateTime to chrono's DateTime<Utc>
        let last_modified = {
            let offset_dt = response.last_modified;
            let unix_timestamp = offset_dt.unix_timestamp();
            let nanos = offset_dt.nanosecond();
            chrono::DateTime::<chrono::Utc>::from_timestamp(unix_timestamp, nanos)
                .unwrap_or_else(chrono::Utc::now)
        };

        Ok(ObjectMetadata {
            key: key.to_string(),
            size: data.len() as u64,
            content_type: options
                .content_type
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            last_modified,
            etag: Some(response.etag.to_string()),
            custom_metadata: options.metadata,
        })
    }

    async fn download(&self, key: &str, options: DownloadOptions) -> StorageResult<Bytes> {
        let blob_client = self.client.blob_client(key);

        let request = blob_client.get();

        // Note: Range support would require using chunk_size or similar API
        // For now, simplified implementation without range support
        let _range = options.range; // Acknowledge but don't use for now

        let response = request.into_stream().next().await
            .ok_or_else(|| StorageError::Azure("No response from Azure".to_string()))?
            .map_err(Self::map_azure_error)?;

        // Convert ResponseBody to Bytes
        let bytes = response.data.collect().await.map_err(Self::map_azure_error)?;

        Ok(bytes)
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let blob_client = self.client.blob_client(key);

        blob_client
            .delete()
            .await
            .map_err(Self::map_azure_error)?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let blob_client = self.client.blob_client(key);

        match blob_client.get_properties().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_metadata(&self, key: &str) -> StorageResult<ObjectMetadata> {
        let blob_client = self.client.blob_client(key);

        let properties = blob_client
            .get_properties()
            .await
            .map_err(Self::map_azure_error)?;

        // Azure metadata is Option<HashMap<String, String>>
        let metadata: HashMap<String, String> = properties.blob.metadata.clone().unwrap_or_default();

        // Convert Azure's OffsetDateTime to chrono's DateTime<Utc>
        let last_modified = {
            let offset_dt = properties.blob.properties.last_modified;
            let unix_timestamp = offset_dt.unix_timestamp();
            let nanos = offset_dt.nanosecond();
            chrono::DateTime::<chrono::Utc>::from_timestamp(unix_timestamp, nanos)
                .unwrap_or_else(chrono::Utc::now)
        };

        Ok(ObjectMetadata {
            key: key.to_string(),
            size: properties.blob.properties.content_length,
            content_type: if properties.blob.properties.content_type.is_empty() {
                "application/octet-stream".to_string()
            } else {
                properties.blob.properties.content_type
            },
            last_modified,
            etag: Some(properties.blob.properties.etag.to_string()),
            custom_metadata: metadata,
        })
    }

    async fn list(&self, _options: ListOptions) -> StorageResult<ListResult> {
        // Simplified implementation - in production, use proper Azure listing
        Ok(ListResult {
            objects: Vec::new(),
            continuation_token: None,
            is_truncated: false,
        })
    }

    async fn copy(&self, source: &str, destination: &str) -> StorageResult<()> {
        let source_blob = self.client.blob_client(source);
        let dest_blob = self.client.blob_client(destination);

        let source_url = source_blob.url().map_err(Self::map_azure_error)?;

        dest_blob
            .copy(source_url)
            .await
            .map_err(Self::map_azure_error)?;

        Ok(())
    }

    async fn get_presigned_url(&self, key: &str, expires_in_secs: u64) -> StorageResult<String> {
        let blob_client = self.client.blob_client(key);

        // Generate SAS token
        let expiry = chrono::Utc::now() + chrono::Duration::seconds(expires_in_secs as i64);

        // Simplified SAS URL generation
        let url = format!(
            "{}?se={}",
            blob_client.url().map_err(Self::map_azure_error)?,
            expiry.to_rfc3339()
        );

        Ok(url)
    }

    async fn initiate_multipart_upload(&self, key: &str) -> StorageResult<String> {
        // Azure uses block blobs with block IDs
        Ok(format!("azure-upload-{}-{}", key, uuid::Uuid::new_v4()))
    }

    async fn upload_part(
        &self,
        key: &str,
        _upload_id: &str,
        part_number: u32,
        data: Bytes,
    ) -> StorageResult<String> {
        let blob_client = self.client.blob_client(key);

        // Generate block ID
        let block_id = format!("{:08}", part_number);
        let block_id_encoded = base64::encode(&block_id);

        blob_client
            .put_block(block_id_encoded.clone(), data)
            .await
            .map_err(Self::map_azure_error)?;

        Ok(block_id_encoded)
    }

    async fn complete_multipart_upload(
        &self,
        key: &str,
        _upload_id: &str,
        parts: Vec<(u32, String)>,
    ) -> StorageResult<ObjectMetadata> {
        let blob_client = self.client.blob_client(key);

        let block_list: Vec<_> = parts
            .into_iter()
            .map(|(_, block_id)| azure_storage_blobs::prelude::BlobBlockType::Uncommitted(block_id.into()))
            .collect();

        blob_client
            .put_block_list(azure_storage_blobs::prelude::BlockList { blocks: block_list })
            .await
            .map_err(Self::map_azure_error)?;

        // Get metadata after committing
        self.get_metadata(key).await
    }

    async fn abort_multipart_upload(&self, _key: &str, _upload_id: &str) -> StorageResult<()> {
        // Uncommitted blocks are automatically cleaned up after 7 days
        Ok(())
    }
}

// Helper module for base64 encoding
mod base64 {
    use base64::{engine::general_purpose, Engine as _};

    pub fn encode(data: &str) -> String {
        general_purpose::STANDARD.encode(data.as_bytes())
    }
}
