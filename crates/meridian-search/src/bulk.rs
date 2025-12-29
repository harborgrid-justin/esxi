//! Bulk indexing with batching and error handling.

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use elasticsearch::BulkParts;
use futures::executor::block_on;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Bulk indexer for efficient document indexing.
pub struct BulkIndexer {
    client: SearchClient,
    index: String,
    batch_size: usize,
    flush_interval: Duration,
    buffer: Vec<BulkOperation>,
    last_flush: Instant,
}

impl BulkIndexer {
    /// Create a new bulk indexer.
    pub fn new(client: SearchClient, index: impl Into<String>) -> Self {
        Self {
            client,
            index: index.into(),
            batch_size: 1000,
            flush_interval: Duration::from_secs(5),
            buffer: Vec::new(),
            last_flush: Instant::now(),
        }
    }

    /// Set the batch size for bulk operations.
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set the flush interval.
    pub fn flush_interval(mut self, interval: Duration) -> Self {
        self.flush_interval = interval;
        self
    }

    /// Add a document to be indexed.
    pub fn add_index<T: Serialize>(
        &mut self,
        id: Option<String>,
        document: T,
    ) -> SearchResult<()> {
        let doc = serde_json::to_value(document)?;
        self.buffer.push(BulkOperation::Index { id, document: doc });
        self.auto_flush()
    }

    /// Add a document to be created (fails if already exists).
    pub fn add_create<T: Serialize>(
        &mut self,
        id: String,
        document: T,
    ) -> SearchResult<()> {
        let doc = serde_json::to_value(document)?;
        self.buffer.push(BulkOperation::Create { id, document: doc });
        self.auto_flush()
    }

    /// Add a document update.
    pub fn add_update<T: Serialize>(
        &mut self,
        id: String,
        document: T,
        upsert: bool,
    ) -> SearchResult<()> {
        let doc = serde_json::to_value(document)?;
        self.buffer.push(BulkOperation::Update { id, document: doc, upsert });
        self.auto_flush()
    }

    /// Add a document deletion.
    pub fn add_delete(&mut self, id: String) -> SearchResult<()> {
        self.buffer.push(BulkOperation::Delete { id });
        self.auto_flush()
    }

    /// Auto-flush if batch size or time threshold is reached.
    fn auto_flush(&mut self) -> SearchResult<()> {
        if self.buffer.len() >= self.batch_size
            || self.last_flush.elapsed() >= self.flush_interval
        {
            block_on(self.flush())?;
        }
        Ok(())
    }

    /// Manually flush the buffer.
    pub async fn flush(&mut self) -> SearchResult<BulkResponse> {
        if self.buffer.is_empty() {
            return Ok(BulkResponse::default());
        }

        info!(
            "Flushing bulk buffer with {} operations to index '{}'",
            self.buffer.len(),
            self.index
        );

        let start = Instant::now();
        let body = self.build_bulk_body();
        let operation_count = self.buffer.len();

        debug!("Bulk request size: {} bytes", body.len());

        let response = self
            .client
            .client()
            .bulk(BulkParts::Index(&self.index))
            .body(vec![body])
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::BulkError(format!(
                "Bulk operation failed: {}",
                error_text
            )));
        }

        let result: Value = response.json().await?;
        let bulk_response = self.parse_bulk_response(result)?;

        self.buffer.clear();
        self.last_flush = Instant::now();

        let duration = start.elapsed();
        info!(
            "Bulk flush completed: {} operations, {} errors, {} ms",
            operation_count,
            bulk_response.errors.len(),
            duration.as_millis()
        );

        if !bulk_response.errors.is_empty() {
            warn!("Bulk operation had {} errors", bulk_response.errors.len());
            for error in &bulk_response.errors {
                debug!("Bulk error: {:?}", error);
            }
        }

        Ok(bulk_response)
    }

    /// Build the bulk request body.
    fn build_bulk_body(&self) -> String {
        let mut lines = Vec::new();

        for operation in &self.buffer {
            match operation {
                BulkOperation::Index { id, document } => {
                    let action = if let Some(id) = id {
                        json!({ "index": { "_id": id } })
                    } else {
                        json!({ "index": {} })
                    };
                    lines.push(serde_json::to_string(&action).unwrap());
                    lines.push(serde_json::to_string(document).unwrap());
                }
                BulkOperation::Create { id, document } => {
                    let action = json!({ "create": { "_id": id } });
                    lines.push(serde_json::to_string(&action).unwrap());
                    lines.push(serde_json::to_string(document).unwrap());
                }
                BulkOperation::Update { id, document, upsert } => {
                    let action = json!({ "update": { "_id": id } });
                    lines.push(serde_json::to_string(&action).unwrap());

                    let update_doc = if *upsert {
                        json!({ "doc": document, "doc_as_upsert": true })
                    } else {
                        json!({ "doc": document })
                    };
                    lines.push(serde_json::to_string(&update_doc).unwrap());
                }
                BulkOperation::Delete { id } => {
                    let action = json!({ "delete": { "_id": id } });
                    lines.push(serde_json::to_string(&action).unwrap());
                }
            }
        }

        lines.join("\n") + "\n"
    }

    /// Parse the bulk response.
    fn parse_bulk_response(&self, response: Value) -> SearchResult<BulkResponse> {
        let took = response["took"].as_u64().unwrap_or(0);
        let has_errors = response["errors"].as_bool().unwrap_or(false);

        let items = response["items"]
            .as_array()
            .ok_or_else(|| SearchError::BulkError("Missing items in response".to_string()))?;

        let mut successful = 0;
        let mut errors = Vec::new();

        for (idx, item) in items.iter().enumerate() {
            // Each item has a single key (index, create, update, or delete)
            let operation = item
                .as_object()
                .and_then(|obj| obj.values().next())
                .ok_or_else(|| SearchError::BulkError("Invalid item format".to_string()))?;

            let status = operation["status"].as_u64().unwrap_or(0);

            if status >= 200 && status < 300 {
                successful += 1;
            } else {
                let error = BulkError {
                    index: idx,
                    status: status as u16,
                    error_type: operation["error"]["type"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string(),
                    reason: operation["error"]["reason"]
                        .as_str()
                        .unwrap_or("No reason provided")
                        .to_string(),
                };
                errors.push(error);
            }
        }

        Ok(BulkResponse {
            took_ms: took,
            has_errors,
            successful,
            failed: errors.len(),
            errors,
        })
    }

    /// Get the current buffer size.
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// Clear the buffer without flushing.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

/// Bulk operation types.
#[derive(Debug, Clone)]
enum BulkOperation {
    Index {
        id: Option<String>,
        document: Value,
    },
    Create {
        id: String,
        document: Value,
    },
    Update {
        id: String,
        document: Value,
        upsert: bool,
    },
    Delete {
        id: String,
    },
}

/// Bulk response with error details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkResponse {
    pub took_ms: u64,
    pub has_errors: bool,
    pub successful: usize,
    pub failed: usize,
    pub errors: Vec<BulkError>,
}

impl Default for BulkResponse {
    fn default() -> Self {
        Self {
            took_ms: 0,
            has_errors: false,
            successful: 0,
            failed: 0,
            errors: Vec::new(),
        }
    }
}

/// Individual bulk operation error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkError {
    pub index: usize,
    pub status: u16,
    pub error_type: String,
    pub reason: String,
}

/// Parallel bulk indexer for high-throughput indexing.
pub struct ParallelBulkIndexer {
    client: SearchClient,
    index: String,
    batch_size: usize,
    concurrency: usize,
}

impl ParallelBulkIndexer {
    /// Create a new parallel bulk indexer.
    pub fn new(client: SearchClient, index: impl Into<String>) -> Self {
        Self {
            client,
            index: index.into(),
            batch_size: 1000,
            concurrency: 4,
        }
    }

    /// Set the batch size.
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set the concurrency level.
    pub fn concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// Index documents in parallel batches.
    pub async fn index_all<T: Serialize + Clone>(
        &self,
        documents: Vec<(Option<String>, T)>,
    ) -> SearchResult<BulkIndexStats> {
        info!(
            "Starting parallel bulk indexing of {} documents",
            documents.len()
        );

        let start = Instant::now();
        let mut stats = BulkIndexStats::default();

        // Split into batches
        let batches: Vec<_> = documents
            .chunks(self.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        info!("Split into {} batches", batches.len());

        // Process batches with limited concurrency
        for batch_group in batches.chunks(self.concurrency) {
            let mut tasks = Vec::new();

            for batch in batch_group {
                let mut indexer = BulkIndexer::new(self.client.clone(), &self.index)
                    .batch_size(self.batch_size);

                let batch_clone = batch.clone();
                let task = async move {
                    for (id, doc) in batch_clone {
                        indexer.add_index(id.clone(), doc)?;
                    }
                    indexer.flush().await
                };

                tasks.push(task);
            }

            // Wait for all tasks in this group
            let results = join_all(tasks).await;

            for result in results {
                let response = result?;
                stats.total_operations += response.successful + response.failed;
                stats.successful += response.successful;
                stats.failed += response.failed;
                stats.took_ms += response.took_ms;
            }
        }

        let duration = start.elapsed();
        stats.total_time_ms = duration.as_millis() as u64;

        info!(
            "Parallel bulk indexing completed: {} successful, {} failed, {} ms",
            stats.successful, stats.failed, stats.total_time_ms
        );

        Ok(stats)
    }
}

/// Statistics for parallel bulk indexing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BulkIndexStats {
    pub total_operations: usize,
    pub successful: usize,
    pub failed: usize,
    pub took_ms: u64,
    pub total_time_ms: u64,
}

impl BulkIndexStats {
    /// Calculate operations per second.
    pub fn ops_per_second(&self) -> f64 {
        if self.total_time_ms == 0 {
            return 0.0;
        }
        (self.total_operations as f64) / (self.total_time_ms as f64 / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_response_default() {
        let response = BulkResponse::default();
        assert_eq!(response.successful, 0);
        assert_eq!(response.failed, 0);
        assert!(!response.has_errors);
    }

    #[test]
    fn test_bulk_index_stats() {
        let stats = BulkIndexStats {
            total_operations: 1000,
            successful: 950,
            failed: 50,
            took_ms: 500,
            total_time_ms: 1000,
        };

        assert_eq!(stats.ops_per_second(), 1000.0);
    }

    #[test]
    fn test_ops_per_second_zero_time() {
        let stats = BulkIndexStats {
            total_operations: 1000,
            total_time_ms: 0,
            ..Default::default()
        };

        assert_eq!(stats.ops_per_second(), 0.0);
    }
}
