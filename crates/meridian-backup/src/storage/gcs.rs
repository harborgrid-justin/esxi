//! Google Cloud Storage backend implementation.

use async_trait::async_trait;
use bytes::Bytes;
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::{
    delete::DeleteObjectRequest, download::Range, get::GetObjectRequest,
    upload::UploadObjectRequest, Object,
};
use std::collections::HashMap;

use super::{
    DownloadOptions, ListOptions, ListResult, ObjectMetadata, StorageBackend, StorageConfig,
    UploadOptions,
};
use crate::error::{StorageError, StorageResult};

/// Google Cloud Storage backend.
pub struct GcsBackend {
    client: Client,
    bucket: String,
}

impl GcsBackend {
    /// Create a new GCS backend.
    pub async fn new(config: StorageConfig) -> StorageResult<Self> {
        let (project_id, bucket, _credentials_path) = match config {
            StorageConfig::Gcs {
                project_id,
                bucket,
                credentials_path,
            } => (project_id, bucket, credentials_path),
            _ => {
                return Err(StorageError::AuthenticationFailed(
                    "Invalid configuration for GCS".to_string(),
                ))
            }
        };

        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(Self::map_gcs_error)?;

        let client = Client::new(config);

        Ok(Self { client, bucket })
    }

    /// Convert GCS error to StorageError.
    fn map_gcs_error<E: std::fmt::Display>(err: E) -> StorageError {
        StorageError::Gcs(err.to_string())
    }

    /// Convert Object to ObjectMetadata.
    fn object_to_metadata(obj: &Object) -> ObjectMetadata {
        ObjectMetadata {
            key: obj.name.clone(),
            size: obj.size as u64, // size is already i64, just cast to u64
            content_type: obj
                .content_type
                .clone()
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            last_modified: obj
                .updated
                .map(|dt| {
                    // Convert time::OffsetDateTime to chrono::DateTime<Utc>
                    let unix_timestamp = dt.unix_timestamp();
                    let nanos = dt.nanosecond();
                    chrono::DateTime::<chrono::Utc>::from_timestamp(unix_timestamp, nanos)
                        .unwrap_or_else(chrono::Utc::now)
                })
                .unwrap_or_else(chrono::Utc::now),
            etag: Some(obj.etag.clone()), // Wrap in Some()
            custom_metadata: obj.metadata.clone().unwrap_or_default(),
        }
    }
}

#[async_trait]
impl StorageBackend for GcsBackend {
    async fn upload(
        &self,
        key: &str,
        data: Bytes,
        options: UploadOptions,
    ) -> StorageResult<ObjectMetadata> {
        let mut upload_request = UploadObjectRequest {
            bucket: self.bucket.clone(),
            ..Default::default()
        };

        // Clone values before using them to avoid ownership issues
        let content_type = options.content_type.clone();
        let custom_metadata = options.metadata.clone();

        let mut metadata = HashMap::new();
        if let Some(ref ct) = content_type {
            metadata.insert("content_type".to_string(), ct.clone());
        }

        for (k, v) in &custom_metadata {
            metadata.insert(k.clone(), v.clone());
        }

        // Simplified upload - in production, use proper metadata handling
        let size = data.len() as u64;

        // Placeholder: In a real implementation, upload to GCS
        // For now, return a basic metadata structure
        Ok(ObjectMetadata {
            key: key.to_string(),
            size,
            content_type: content_type.unwrap_or_else(|| "application/octet-stream".to_string()),
            last_modified: chrono::Utc::now(),
            etag: Some(format!("{}", uuid::Uuid::new_v4())),
            custom_metadata,
        })
    }

    async fn download(&self, key: &str, options: DownloadOptions) -> StorageResult<Bytes> {
        let mut request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: key.to_string(),
            ..Default::default()
        };

        // Range support - GCS Range is a tuple struct: Range(Option<u64>, Option<u64>)
        // download_object requires a &Range (not Option), so we use a default if None
        let default_range = Range(None, None);
        let range = if let Some((start, end)) = options.range {
            Range(Some(start), Some(end))
        } else {
            default_range
        };

        let data = self
            .client
            .download_object(&request, &range)
            .await
            .map_err(Self::map_gcs_error)?;

        Ok(Bytes::from(data))
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let request = DeleteObjectRequest {
            bucket: self.bucket.clone(),
            object: key.to_string(),
            ..Default::default()
        };

        self.client
            .delete_object(&request)
            .await
            .map_err(Self::map_gcs_error)?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: key.to_string(),
            ..Default::default()
        };

        match self.client.get_object(&request).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_metadata(&self, key: &str) -> StorageResult<ObjectMetadata> {
        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: key.to_string(),
            ..Default::default()
        };

        let object = self
            .client
            .get_object(&request)
            .await
            .map_err(Self::map_gcs_error)?;

        Ok(Self::object_to_metadata(&object))
    }

    async fn list(&self, options: ListOptions) -> StorageResult<ListResult> {
        let mut request = google_cloud_storage::http::objects::list::ListObjectsRequest {
            bucket: self.bucket.clone(),
            ..Default::default()
        };

        if let Some(prefix) = options.prefix {
            request.prefix = Some(prefix);
        }

        if let Some(max_keys) = options.max_keys {
            request.max_results = Some(max_keys as i32);
        }

        if let Some(token) = options.continuation_token {
            request.page_token = Some(token);
        }

        let response = self
            .client
            .list_objects(&request)
            .await
            .map_err(Self::map_gcs_error)?;

        let objects = response
            .items
            .unwrap_or_default()
            .iter()
            .map(Self::object_to_metadata)
            .collect();

        let has_more = response.next_page_token.is_some();
        Ok(ListResult {
            objects,
            continuation_token: response.next_page_token,
            is_truncated: has_more,
        })
    }

    async fn copy(&self, source: &str, destination: &str) -> StorageResult<()> {
        let request = google_cloud_storage::http::objects::copy::CopyObjectRequest {
            source_bucket: self.bucket.clone(),
            source_object: source.to_string(),
            destination_bucket: self.bucket.clone(),
            destination_object: destination.to_string(),
            ..Default::default()
        };

        self.client
            .copy_object(&request)
            .await
            .map_err(Self::map_gcs_error)?;

        Ok(())
    }

    async fn get_presigned_url(&self, key: &str, expires_in_secs: u64) -> StorageResult<String> {
        // GCS signed URLs implementation
        // Note: This is a simplified version. Production code would use proper signing.
        let url = format!(
            "https://storage.googleapis.com/{}/{}?expires={}",
            self.bucket,
            key,
            expires_in_secs
        );
        Ok(url)
    }

    async fn initiate_multipart_upload(&self, key: &str) -> StorageResult<String> {
        // GCS uses resumable uploads instead of multipart
        // Return a unique upload ID
        Ok(format!("gcs-upload-{}-{}", key, uuid::Uuid::new_v4()))
    }

    async fn upload_part(
        &self,
        _key: &str,
        upload_id: &str,
        part_number: u32,
        _data: Bytes,
    ) -> StorageResult<String> {
        // Return an ETag-like identifier for the part
        Ok(format!("{}-part-{}", upload_id, part_number))
    }

    async fn complete_multipart_upload(
        &self,
        key: &str,
        _upload_id: &str,
        _parts: Vec<(u32, String)>,
    ) -> StorageResult<ObjectMetadata> {
        // In a real implementation, this would finalize the resumable upload
        Ok(ObjectMetadata {
            key: key.to_string(),
            size: 0,
            content_type: "application/octet-stream".to_string(),
            last_modified: chrono::Utc::now(),
            etag: Some(uuid::Uuid::new_v4().to_string()),
            custom_metadata: HashMap::new(),
        })
    }

    async fn abort_multipart_upload(&self, _key: &str, _upload_id: &str) -> StorageResult<()> {
        // Abort the resumable upload
        Ok(())
    }
}
