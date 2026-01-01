//! Streaming inference engine
//!
//! Handles real-time streaming inference with backpressure control

use super::ServingStats;
use crate::models::inference::InferenceEngine;
use crate::pipeline::PipelineNode;
use crate::{Error, Result};
use ndarray::ArrayD;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Streaming inference request
#[derive(Debug, Clone)]
pub struct StreamRequest {
    /// Request identifier
    pub id: Uuid,

    /// Input data
    pub input: Vec<f32>,

    /// Request timestamp
    pub timestamp: Instant,
}

impl StreamRequest {
    /// Create a new stream request
    pub fn new(input: Vec<f32>) -> Self {
        Self {
            id: Uuid::new_v4(),
            input,
            timestamp: Instant::now(),
        }
    }
}

/// Streaming inference response
#[derive(Debug, Clone)]
pub struct StreamResponse {
    /// Request identifier
    pub request_id: Uuid,

    /// Prediction
    pub prediction: Vec<f32>,

    /// Processing time in milliseconds
    pub processing_time_ms: f64,

    /// Success status
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,
}

impl StreamResponse {
    /// Create a successful response
    pub fn success(request_id: Uuid, prediction: Vec<f32>, processing_time_ms: f64) -> Self {
        Self {
            request_id,
            prediction,
            processing_time_ms,
            success: true,
            error: None,
        }
    }

    /// Create an error response
    pub fn error(request_id: Uuid, error: String, processing_time_ms: f64) -> Self {
        Self {
            request_id,
            prediction: Vec::new(),
            processing_time_ms,
            success: false,
            error: Some(error),
        }
    }
}

/// Streaming configuration
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for incoming requests
    pub buffer_size: usize,

    /// Maximum concurrent processing
    pub max_concurrent: usize,

    /// Enable backpressure
    pub enable_backpressure: bool,

    /// Timeout for individual requests in milliseconds
    pub request_timeout_ms: u64,

    /// Enable request batching in stream
    pub enable_micro_batching: bool,

    /// Micro batch size
    pub micro_batch_size: usize,

    /// Micro batch timeout in milliseconds
    pub micro_batch_timeout_ms: u64,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            max_concurrent: 100,
            enable_backpressure: true,
            request_timeout_ms: 100,
            enable_micro_batching: true,
            micro_batch_size: 8,
            micro_batch_timeout_ms: 5,
        }
    }
}

/// Streaming inference engine
pub struct StreamingInferenceEngine {
    /// Configuration
    config: StreamConfig,

    /// Inference engine
    inference_engine: Arc<InferenceEngine>,

    /// Statistics
    stats: Arc<RwLock<ServingStats>>,

    /// Request sender
    request_tx: mpsc::Sender<StreamRequest>,

    /// Response receiver
    response_rx: Arc<RwLock<Option<mpsc::Receiver<StreamResponse>>>>,
}

impl StreamingInferenceEngine {
    /// Create a new streaming inference engine
    pub fn new(model: Arc<dyn PipelineNode>, config: StreamConfig) -> Self {
        let inference_config = crate::models::inference::InferenceConfig {
            max_batch_size: config.micro_batch_size,
            batch_timeout_ms: config.micro_batch_timeout_ms,
            max_concurrency: config.max_concurrent,
            ..Default::default()
        };

        let inference_engine = Arc::new(InferenceEngine::new(model, inference_config));

        let (request_tx, request_rx) = mpsc::channel(config.buffer_size);
        let (response_tx, response_rx) = mpsc::channel(config.buffer_size);

        let engine = Self {
            config: config.clone(),
            inference_engine: inference_engine.clone(),
            stats: Arc::new(RwLock::new(ServingStats::default())),
            request_tx,
            response_rx: Arc::new(RwLock::new(Some(response_rx))),
        };

        // Start background processing task
        tokio::spawn(Self::process_stream(
            inference_engine,
            config,
            request_rx,
            response_tx,
            engine.stats.clone(),
        ));

        engine
    }

    /// Create with default configuration
    pub fn with_defaults(model: Arc<dyn PipelineNode>) -> Self {
        Self::new(model, StreamConfig::default())
    }

    /// Submit a request for streaming inference
    pub async fn predict(&self, input: Vec<f32>) -> Result<()> {
        let request = StreamRequest::new(input);

        self.request_tx
            .send(request)
            .await
            .map_err(|e| Error::internal(format!("Failed to send request: {}", e)))
    }

    /// Get the response stream
    pub fn response_stream(&self) -> Option<mpsc::Receiver<StreamResponse>> {
        self.response_rx.write().take()
    }

    /// Process the request stream
    async fn process_stream(
        inference_engine: Arc<InferenceEngine>,
        config: StreamConfig,
        mut request_rx: mpsc::Receiver<StreamRequest>,
        response_tx: mpsc::Sender<StreamResponse>,
        stats: Arc<RwLock<ServingStats>>,
    ) {
        tracing::info!("Starting streaming inference processor");

        let mut micro_batch = Vec::new();
        let mut last_batch_time = Instant::now();

        while let Some(request) = request_rx.recv().await {
            micro_batch.push(request);

            // Check if we should process the batch
            let should_process = micro_batch.len() >= config.micro_batch_size
                || last_batch_time.elapsed().as_millis() >= config.micro_batch_timeout_ms as u128;

            if should_process && !micro_batch.is_empty() {
                let batch = std::mem::take(&mut micro_batch);
                last_batch_time = Instant::now();

                // Process batch
                Self::process_micro_batch(
                    batch,
                    inference_engine.clone(),
                    response_tx.clone(),
                    stats.clone(),
                )
                .await;
            }
        }

        // Process remaining requests
        if !micro_batch.is_empty() {
            Self::process_micro_batch(
                micro_batch,
                inference_engine.clone(),
                response_tx.clone(),
                stats.clone(),
            )
            .await;
        }

        tracing::info!("Streaming inference processor stopped");
    }

    /// Process a micro-batch of requests
    async fn process_micro_batch(
        requests: Vec<StreamRequest>,
        inference_engine: Arc<InferenceEngine>,
        response_tx: mpsc::Sender<StreamResponse>,
        stats: Arc<RwLock<ServingStats>>,
    ) {
        tracing::debug!("Processing micro-batch of {} requests", requests.len());

        for request in requests {
            let start = Instant::now();
            let request_id = request.id;

            // Convert input to ArrayD
            let input_len = request.input.len();
            let input = match ArrayD::from_shape_vec(vec![1, input_len], request.input) {
                Ok(arr) => arr,
                Err(e) => {
                    let _ = response_tx
                        .send(StreamResponse::error(
                            request_id,
                            format!("Failed to create array: {}", e),
                            0.0,
                        ))
                        .await;
                    continue;
                }
            };

            // Perform inference
            let result = inference_engine.predict(input).await;
            let processing_time_ms = start.elapsed().as_millis() as f64;

            // Update statistics and create response
            let response = {
                let mut stats_lock = stats.write();
                stats_lock.total_requests += 1;

                let response = match result {
                    Ok(prediction) => {
                        stats_lock.successful_requests += 1;

                        // Update latency
                        let alpha = 0.1;
                        stats_lock.avg_latency_ms =
                            alpha * processing_time_ms + (1.0 - alpha) * stats_lock.avg_latency_ms;

                        StreamResponse::success(
                            request_id,
                            prediction.into_iter().collect(),
                            processing_time_ms,
                        )
                    }
                    Err(e) => {
                        stats_lock.failed_requests += 1;
                        StreamResponse::error(request_id, e.to_string(), processing_time_ms)
                    }
                };

                drop(stats_lock);
                response
            };

            // Send response
            if let Err(e) = response_tx.send(response).await {
                tracing::error!("Failed to send response: {}", e);
            }
        }
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
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    /// Get buffer utilization (0.0 to 1.0)
    pub fn buffer_utilization(&self) -> f64 {
        let capacity = self.config.buffer_size;
        let current = self.request_tx.capacity();
        (capacity - current) as f64 / capacity as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_request_creation() {
        let input = vec![1.0, 2.0, 3.0];
        let request = StreamRequest::new(input.clone());
        assert_eq!(request.input, input);
    }

    #[test]
    fn test_stream_response_success() {
        let request_id = Uuid::new_v4();
        let prediction = vec![0.9, 0.1];
        let response = StreamResponse::success(request_id, prediction.clone(), 5.5);

        assert!(response.success);
        assert_eq!(response.prediction, prediction);
        assert_eq!(response.processing_time_ms, 5.5);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_stream_response_error() {
        let request_id = Uuid::new_v4();
        let response = StreamResponse::error(request_id, "Error".to_string(), 2.0);

        assert!(!response.success);
        assert!(response.prediction.is_empty());
        assert_eq!(response.error, Some("Error".to_string()));
    }

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.buffer_size, 1000);
        assert_eq!(config.max_concurrent, 100);
        assert!(config.enable_backpressure);
        assert!(config.enable_micro_batching);
    }
}
