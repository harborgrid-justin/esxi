//! Batch inference engine
//!
//! Handles batch processing of inference requests with optimizations

use super::ServingStats;
use crate::models::inference::InferenceEngine;
use crate::pipeline::PipelineNode;
use crate::{Error, Result};
use ndarray::ArrayD;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use uuid::Uuid;

/// Batch inference request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchRequest {
    /// Request identifier
    pub id: Uuid,

    /// Input data batch
    pub inputs: Vec<Vec<f32>>,

    /// Request metadata
    pub metadata: std::collections::HashMap<String, String>,

    /// Request timestamp
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl BatchRequest {
    /// Create a new batch request
    pub fn new(inputs: Vec<Vec<f32>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            inputs,
            metadata: std::collections::HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get batch size
    pub fn batch_size(&self) -> usize {
        self.inputs.len()
    }
}

/// Batch inference response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchResponse {
    /// Request identifier
    pub request_id: Uuid,

    /// Predictions
    pub predictions: Vec<Vec<f32>>,

    /// Processing time in milliseconds
    pub processing_time_ms: f64,

    /// Response timestamp
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Success status
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,
}

impl BatchResponse {
    /// Create a successful response
    pub fn success(request_id: Uuid, predictions: Vec<Vec<f32>>, processing_time_ms: f64) -> Self {
        Self {
            request_id,
            predictions,
            processing_time_ms,
            timestamp: chrono::Utc::now(),
            success: true,
            error: None,
        }
    }

    /// Create an error response
    pub fn error(request_id: Uuid, error: String) -> Self {
        Self {
            request_id,
            predictions: Vec::new(),
            processing_time_ms: 0.0,
            timestamp: chrono::Utc::now(),
            success: false,
            error: Some(error),
        }
    }
}

/// Batch inference configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum batch size
    pub max_batch_size: usize,

    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,

    /// Maximum concurrent batches
    pub max_concurrent_batches: usize,

    /// Enable dynamic batching
    pub enable_dynamic_batching: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 128,
            batch_timeout_ms: 50,
            max_concurrent_batches: 10,
            enable_dynamic_batching: true,
        }
    }
}

/// Batch inference engine
pub struct BatchInferenceEngine {
    /// Configuration
    config: BatchConfig,

    /// Inference engine
    inference_engine: Arc<InferenceEngine>,

    /// Statistics
    stats: Arc<RwLock<ServingStats>>,

    /// Concurrency control
    semaphore: Arc<Semaphore>,
}

impl BatchInferenceEngine {
    /// Create a new batch inference engine
    pub fn new(model: Arc<dyn PipelineNode>, config: BatchConfig) -> Self {
        let inference_config = crate::models::inference::InferenceConfig {
            max_batch_size: config.max_batch_size,
            batch_timeout_ms: config.batch_timeout_ms,
            ..Default::default()
        };

        let inference_engine = Arc::new(InferenceEngine::new(model, inference_config));
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_batches));

        Self {
            config,
            inference_engine,
            stats: Arc::new(RwLock::new(ServingStats::default())),
            semaphore,
        }
    }

    /// Create with default configuration
    pub fn with_defaults(model: Arc<dyn PipelineNode>) -> Self {
        Self::new(model, BatchConfig::default())
    }

    /// Process a batch request
    pub async fn process_batch(&self, request: BatchRequest) -> BatchResponse {
        let start = Instant::now();
        let request_id = request.id;

        tracing::info!(
            "Processing batch request {} with {} samples",
            request_id,
            request.batch_size()
        );

        // Acquire semaphore permit
        let _permit = match self.semaphore.acquire().await {
            Ok(permit) => permit,
            Err(e) => {
                return BatchResponse::error(
                    request_id,
                    format!("Failed to acquire semaphore: {}", e),
                );
            }
        };

        // Convert inputs to ArrayD
        let inputs: Vec<ArrayD<f32>> = request
            .inputs
            .into_iter()
            .map(|input| {
                let len = input.len();
                ArrayD::from_shape_vec(vec![1, len], input)
                    .unwrap_or_else(|_| ArrayD::zeros(vec![1, len]))
            })
            .collect();

        // Perform batch inference
        let result = self.inference_engine.predict_batch(inputs).await;

        let processing_time_ms = start.elapsed().as_millis() as f64;

        // Update statistics
        let mut stats = self.stats.write();
        stats.total_requests += 1;

        match result {
            Ok(predictions) => {
                stats.successful_requests += 1;

                // Update latency metrics
                let alpha = 0.1;
                stats.avg_latency_ms =
                    alpha * processing_time_ms + (1.0 - alpha) * stats.avg_latency_ms;

                tracing::info!(
                    "Batch request {} completed successfully in {:.2}ms",
                    request_id,
                    processing_time_ms
                );

                // Convert predictions back to Vec<Vec<f32>>
                let predictions: Vec<Vec<f32>> = predictions
                    .into_iter()
                    .map(|arr| arr.into_iter().collect())
                    .collect();

                BatchResponse::success(request_id, predictions, processing_time_ms)
            }
            Err(e) => {
                stats.failed_requests += 1;

                tracing::error!("Batch request {} failed: {}", request_id, e);

                BatchResponse::error(request_id, e.to_string())
            }
        }
    }

    /// Process multiple batch requests concurrently
    pub async fn process_batches(&self, requests: Vec<BatchRequest>) -> Vec<BatchResponse> {
        use futures::stream::{self, StreamExt};

        let responses: Vec<BatchResponse> = stream::iter(requests)
            .map(|request| async move { self.process_batch(request).await })
            .buffer_unordered(self.config.max_concurrent_batches)
            .collect()
            .await;

        responses
    }

    /// Get serving statistics
    pub fn stats(&self) -> ServingStats {
        self.stats.read().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.write() = ServingStats::default();
    }

    /// Get configuration
    pub fn config(&self) -> &BatchConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_request_creation() {
        let inputs = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let request = BatchRequest::new(inputs);
        assert_eq!(request.batch_size(), 2);
        assert!(request.metadata.is_empty());
    }

    #[test]
    fn test_batch_request_with_metadata() {
        let request = BatchRequest::new(vec![vec![1.0, 2.0]])
            .with_metadata("model".to_string(), "v1".to_string());
        assert_eq!(request.metadata.get("model"), Some(&"v1".to_string()));
    }

    #[test]
    fn test_batch_response_success() {
        let request_id = Uuid::new_v4();
        let predictions = vec![vec![0.9, 0.1]];
        let response = BatchResponse::success(request_id, predictions.clone(), 10.5);

        assert!(response.success);
        assert_eq!(response.predictions, predictions);
        assert_eq!(response.processing_time_ms, 10.5);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_batch_response_error() {
        let request_id = Uuid::new_v4();
        let response = BatchResponse::error(request_id, "Model error".to_string());

        assert!(!response.success);
        assert!(response.predictions.is_empty());
        assert_eq!(response.error, Some("Model error".to_string()));
    }

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.max_batch_size, 128);
        assert_eq!(config.max_concurrent_batches, 10);
        assert!(config.enable_dynamic_batching);
    }
}
