//! Model inference engine
//!
//! Provides high-performance inference with batching and caching

use super::{ModelMetrics, ModelInfo};
use crate::pipeline::PipelineNode;
use crate::{Error, Result};
use ndarray::ArrayD;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

/// Configuration for inference engine
#[derive(Debug, Clone)]
pub struct InferenceConfig {
    /// Maximum batch size
    pub max_batch_size: usize,

    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,

    /// Maximum concurrent inferences
    pub max_concurrency: usize,

    /// Enable result caching
    pub enable_caching: bool,

    /// Cache size (number of entries)
    pub cache_size: usize,

    /// Enable warmup runs
    pub enable_warmup: bool,

    /// Number of warmup iterations
    pub warmup_iterations: usize,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 32,
            batch_timeout_ms: 10,
            max_concurrency: 100,
            enable_caching: true,
            cache_size: 1000,
            enable_warmup: true,
            warmup_iterations: 3,
        }
    }
}

/// Inference request
#[derive(Debug)]
struct InferenceRequest {
    /// Request identifier
    id: uuid::Uuid,

    /// Input data
    input: ArrayD<f32>,

    /// Request timestamp
    timestamp: Instant,

    /// Response sender
    response_tx: tokio::sync::oneshot::Sender<Result<ArrayD<f32>>>,
}

/// Inference engine for high-performance model serving
pub struct InferenceEngine {
    /// Engine configuration
    config: InferenceConfig,

    /// Model being served
    model: Arc<dyn PipelineNode>,

    /// Model metrics
    metrics: Arc<RwLock<ModelMetrics>>,

    /// Concurrency semaphore
    semaphore: Arc<Semaphore>,

    /// Request queue for batching
    request_queue: Arc<RwLock<VecDeque<InferenceRequest>>>,

    /// Inference cache
    cache: Option<Arc<dashmap::DashMap<u64, ArrayD<f32>>>>,
}

impl InferenceEngine {
    /// Create a new inference engine
    pub fn new(model: Arc<dyn PipelineNode>, config: InferenceConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
        let metrics = Arc::new(RwLock::new(ModelMetrics::new(model.id())));

        let cache = if config.enable_caching {
            Some(Arc::new(dashmap::DashMap::new()))
        } else {
            None
        };

        Self {
            config,
            model,
            metrics,
            semaphore,
            request_queue: Arc::new(RwLock::new(VecDeque::new())),
            cache,
        }
    }

    /// Create with default configuration
    pub fn with_defaults(model: Arc<dyn PipelineNode>) -> Self {
        Self::new(model, InferenceConfig::default())
    }

    /// Perform inference on input data
    pub async fn predict(&self, input: ArrayD<f32>) -> Result<ArrayD<f32>> {
        // Check cache
        if let Some(cache) = &self.cache {
            let cache_key = self.compute_cache_key(&input);
            if let Some(cached) = cache.get(&cache_key) {
                tracing::debug!("Cache hit for key {}", cache_key);
                return Ok(cached.clone());
            }
        }

        // Acquire semaphore permit
        let _permit = self.semaphore.acquire().await
            .map_err(|e| Error::internal(format!("Semaphore error: {}", e)))?;

        let start = Instant::now();

        // Execute inference
        let result = self.model.execute(input.clone());

        let duration = start.elapsed();
        let duration_ms = duration.as_millis() as f64;

        // Update metrics
        let success = result.is_ok();
        self.metrics.write().record_inference(duration_ms, success);

        // Cache result on success
        if let (Some(cache), Ok(ref output)) = (&self.cache, &result) {
            let cache_key = self.compute_cache_key(&input);

            // Limit cache size
            if cache.len() < self.config.cache_size {
                cache.insert(cache_key, output.clone());
            }
        }

        result
    }

    /// Perform batched inference
    pub async fn predict_batch(&self, inputs: Vec<ArrayD<f32>>) -> Result<Vec<ArrayD<f32>>> {
        use rayon::prelude::*;

        let start = Instant::now();

        // Process in parallel using rayon
        let results: Vec<Result<ArrayD<f32>>> = inputs
            .into_par_iter()
            .map(|input| self.model.execute(input))
            .collect();

        let duration_ms = start.elapsed().as_millis() as f64;

        // Update metrics for each prediction
        let successes = results.iter().filter(|r| r.is_ok()).count();
        for _ in 0..results.len() {
            self.metrics.write().record_inference(
                duration_ms / results.len() as f64,
                true
            );
        }

        tracing::info!(
            "Batch inference completed: {} items in {:.2}ms ({} successful)",
            results.len(),
            duration_ms,
            successes
        );

        results.into_iter().collect()
    }

    /// Warmup the model
    pub async fn warmup(&self) -> Result<()> {
        if !self.config.enable_warmup {
            return Ok(());
        }

        tracing::info!("Warming up model with {} iterations", self.config.warmup_iterations);

        // Create dummy input (assuming 2D data)
        let dummy_input = ArrayD::zeros(vec![1, 10]);

        for i in 0..self.config.warmup_iterations {
            let start = Instant::now();
            self.model.execute(dummy_input.clone())?;
            tracing::debug!(
                "Warmup iteration {}/{} completed in {:?}",
                i + 1,
                self.config.warmup_iterations,
                start.elapsed()
            );
        }

        tracing::info!("Model warmup completed");
        Ok(())
    }

    /// Get model metrics
    pub fn metrics(&self) -> ModelMetrics {
        self.metrics.read().clone()
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        if let Some(cache) = &self.cache {
            cache.clear();
            tracing::info!("Inference cache cleared");
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> Option<CacheStats> {
        self.cache.as_ref().map(|cache| CacheStats {
            size: cache.len(),
            capacity: self.config.cache_size,
        })
    }

    /// Compute cache key for input
    fn compute_cache_key(&self, input: &ArrayD<f32>) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash the shape
        input.shape().hash(&mut hasher);

        // Hash a sample of values (to avoid hashing entire large arrays)
        let sample_size = input.len().min(100);
        for &val in input.iter().take(sample_size) {
            val.to_bits().hash(&mut hasher);
        }

        hasher.finish()
    }
}

/// Cache statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheStats {
    /// Current cache size
    pub size: usize,

    /// Maximum cache capacity
    pub capacity: usize,
}

/// Inference result with metadata
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Prediction output
    pub output: ArrayD<f32>,

    /// Inference time in milliseconds
    pub inference_time_ms: f64,

    /// Model version used
    pub model_version: String,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl InferenceResult {
    /// Create a new inference result
    pub fn new(output: ArrayD<f32>, inference_time_ms: f64) -> Self {
        Self {
            output,
            inference_time_ms,
            model_version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::BaseNode;

    #[test]
    fn test_inference_config_default() {
        let config = InferenceConfig::default();
        assert_eq!(config.max_batch_size, 32);
        assert_eq!(config.max_concurrency, 100);
        assert!(config.enable_caching);
    }

    #[test]
    fn test_cache_stats() {
        let stats = CacheStats {
            size: 50,
            capacity: 100,
        };
        assert_eq!(stats.size, 50);
        assert_eq!(stats.capacity, 100);
    }

    #[test]
    fn test_inference_result() {
        let output = ArrayD::zeros(vec![1, 10]);
        let result = InferenceResult::new(output, 15.5);
        assert_eq!(result.inference_time_ms, 15.5);
        assert_eq!(result.model_version, "1.0.0");
    }
}
