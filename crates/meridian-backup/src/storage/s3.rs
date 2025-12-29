//! AWS S3 storage backend implementation.

use async_trait::async_trait;
use aws_sdk_s3::{
    config::Region,
    primitives::ByteStream,
    types::{CompletedMultipartUpload, CompletedPart},
    Client,
};
use bytes::Bytes;
use std::collections::HashMap;
use std::time::Duration;

use super::{
    DownloadOptions, ListOptions, ListResult, ObjectMetadata, StorageBackend, StorageConfig,
    UploadOptions,
};
use crate::error::{StorageError, StorageResult};

/// AWS S3 storage backend.
pub struct S3Backend {
    client: Client,
    bucket: String,
}

impl S3Backend {
    /// Create a new S3 backend.
    pub async fn new(config: StorageConfig) -> StorageResult<Self> {
        let (region, bucket, endpoint, _access_key, _secret_key) = match config {
            StorageConfig::S3 {
                region,
                bucket,
                endpoint,
                access_key,
                secret_key,
            } => (region, bucket, endpoint, access_key, secret_key),
            _ => {
                return Err(StorageError::AuthenticationFailed(
                    "Invalid configuration for S3".to_string(),
                ))
            }
        };

        let mut config_builder = aws_config::from_env().region(Region::new(region));

        if let Some(endpoint_url) = endpoint {
            config_builder = config_builder.endpoint_url(endpoint_url);
        }

        let aws_config = config_builder.load().await;
        let client = Client::new(&aws_config);

        Ok(Self { client, bucket })
    }

    /// Convert S3 error to StorageError.
    fn map_s3_error<E: std::fmt::Display>(err: E) -> StorageError {
        StorageError::S3(err.to_string())
    }
}

#[async_trait]
impl StorageBackend for S3Backend {
    async fn upload(
        &self,
        key: &str,
        data: Bytes,
        options: UploadOptions,
    ) -> StorageResult<ObjectMetadata> {
        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data.clone()));

        if let Some(content_type) = options.content_type {
            request = request.content_type(content_type);
        }

        for (k, v) in options.metadata {
            request = request.metadata(k, v);
        }

        if let Some(storage_class) = options.storage_class {
            request = request.storage_class(storage_class.parse().map_err(Self::map_s3_error)?);
        }

        let output = request.send().await.map_err(Self::map_s3_error)?;

        Ok(ObjectMetadata {
            key: key.to_string(),
            size: data.len() as u64,
            content_type: "application/octet-stream".to_string(),
            last_modified: chrono::Utc::now(),
            etag: output.e_tag().map(|s| s.to_string()),
            custom_metadata: HashMap::new(),
        })
    }

    async fn download(&self, key: &str, options: DownloadOptions) -> StorageResult<Bytes> {
        let mut request = self.client.get_object().bucket(&self.bucket).key(key);

        if let Some((start, end)) = options.range {
            request = request.range(format!("bytes={}-{}", start, end));
        }

        let output = request.send().await.map_err(Self::map_s3_error)?;

        let data = output
            .body
            .collect()
            .await
            .map_err(Self::map_s3_error)?
            .into_bytes();

        Ok(data)
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(Self::map_s3_error)?;

        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_metadata(&self, key: &str) -> StorageResult<ObjectMetadata> {
        let output = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(Self::map_s3_error)?;

        Ok(ObjectMetadata {
            key: key.to_string(),
            size: output.content_length().unwrap_or(0) as u64,
            content_type: output
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string(),
            last_modified: output
                .last_modified()
                .and_then(|dt| {
                    chrono::DateTime::parse_from_rfc3339(&dt.to_string())
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                })
                .unwrap_or_else(chrono::Utc::now),
            etag: output.e_tag().map(|s| s.to_string()),
            custom_metadata: output
                .metadata()
                .map(|m| {
                    m.iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default(),
        })
    }

    async fn list(&self, options: ListOptions) -> StorageResult<ListResult> {
        let mut request = self.client.list_objects_v2().bucket(&self.bucket);

        if let Some(prefix) = options.prefix {
            request = request.prefix(prefix);
        }

        if let Some(max_keys) = options.max_keys {
            request = request.max_keys(max_keys as i32);
        }

        if let Some(token) = options.continuation_token {
            request = request.continuation_token(token);
        }

        let output = request.send().await.map_err(Self::map_s3_error)?;

        let objects = output
            .contents()
            .iter()
            .filter_map(|obj| {
                Some(ObjectMetadata {
                    key: obj.key()?.to_string(),
                    size: obj.size().unwrap_or(0) as u64,
                    content_type: "application/octet-stream".to_string(),
                    last_modified: obj
                        .last_modified()
                        .and_then(|dt| {
                            chrono::DateTime::parse_from_rfc3339(&dt.to_string())
                                .ok()
                                .map(|dt| dt.with_timezone(&chrono::Utc))
                        })
                        .unwrap_or_else(chrono::Utc::now),
                    etag: obj.e_tag().map(|s| s.to_string()),
                    custom_metadata: HashMap::new(),
                })
            })
            .collect();

        Ok(ListResult {
            objects,
            continuation_token: output.next_continuation_token().map(|s| s.to_string()),
            is_truncated: output.is_truncated().unwrap_or(false),
        })
    }

    async fn copy(&self, source: &str, destination: &str) -> StorageResult<()> {
        let copy_source = format!("{}/{}", self.bucket, source);

        self.client
            .copy_object()
            .bucket(&self.bucket)
            .copy_source(copy_source)
            .key(destination)
            .send()
            .await
            .map_err(Self::map_s3_error)?;

        Ok(())
    }

    async fn get_presigned_url(&self, key: &str, expires_in_secs: u64) -> StorageResult<String> {
        let presigning_config = aws_sdk_s3::presigning::PresigningConfig::builder()
            .expires_in(Duration::from_secs(expires_in_secs))
            .build()
            .map_err(Self::map_s3_error)?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(Self::map_s3_error)?;

        Ok(presigned.uri().to_string())
    }

    async fn initiate_multipart_upload(&self, key: &str) -> StorageResult<String> {
        let output = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(Self::map_s3_error)?;

        output
            .upload_id()
            .map(|s| s.to_string())
            .ok_or_else(|| StorageError::UploadFailed("No upload ID returned".to_string()))
    }

    async fn upload_part(
        &self,
        key: &str,
        upload_id: &str,
        part_number: u32,
        data: Bytes,
    ) -> StorageResult<String> {
        let output = self
            .client
            .upload_part()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .part_number(part_number as i32)
            .body(ByteStream::from(data))
            .send()
            .await
            .map_err(Self::map_s3_error)?;

        output
            .e_tag()
            .map(|s| s.to_string())
            .ok_or_else(|| StorageError::UploadFailed("No ETag returned".to_string()))
    }

    async fn complete_multipart_upload(
        &self,
        key: &str,
        upload_id: &str,
        parts: Vec<(u32, String)>,
    ) -> StorageResult<ObjectMetadata> {
        let completed_parts: Vec<CompletedPart> = parts
            .into_iter()
            .map(|(part_num, etag)| {
                CompletedPart::builder()
                    .part_number(part_num as i32)
                    .e_tag(etag)
                    .build()
            })
            .collect();

        let completed_upload = CompletedMultipartUpload::builder()
            .set_parts(Some(completed_parts))
            .build();

        let output = self
            .client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .multipart_upload(completed_upload)
            .send()
            .await
            .map_err(Self::map_s3_error)?;

        Ok(ObjectMetadata {
            key: key.to_string(),
            size: 0, // Size not available in response
            content_type: "application/octet-stream".to_string(),
            last_modified: chrono::Utc::now(),
            etag: output.e_tag().map(|s| s.to_string()),
            custom_metadata: HashMap::new(),
        })
    }

    async fn abort_multipart_upload(&self, key: &str, upload_id: &str) -> StorageResult<()> {
        self.client
            .abort_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .send()
            .await
            .map_err(Self::map_s3_error)?;

        Ok(())
    }
}
