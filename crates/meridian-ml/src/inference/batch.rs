//! Batch inference for large datasets

use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use crate::inference::{InferenceConfig, InferenceOutput, InferenceRuntime};
use ndarray::{Array1, Array2, ArrayD, Axis, IxDyn};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::task;

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Batch size
    pub batch_size: usize,

    /// Number of workers for parallel processing
    pub num_workers: usize,

    /// Maximum queue size
    pub max_queue_size: usize,

    /// Enable progress reporting
    pub show_progress: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 32,
            num_workers: 4,
            max_queue_size: 100,
            show_progress: true,
        }
    }
}

/// Batch inference result
#[derive(Debug, Clone)]
pub struct BatchInferenceResult {
    /// All outputs concatenated
    pub outputs: Vec<ArrayD<f32>>,

    /// Total samples processed
    pub num_samples: usize,

    /// Total inference time (milliseconds)
    pub total_time_ms: f64,

    /// Average inference time per sample (milliseconds)
    pub avg_time_per_sample_ms: f64,
}

/// Batch inference processor
pub struct BatchInference {
    /// Runtime for inference
    runtime: InferenceRuntime,

    /// Batch configuration
    config: BatchConfig,
}

impl BatchInference {
    /// Create a new batch inference processor
    pub fn new(inference_config: InferenceConfig, batch_config: BatchConfig) -> Self {
        Self {
            runtime: InferenceRuntime::new(inference_config),
            config: batch_config,
        }
    }

    /// Create with default configuration
    pub fn default_cpu() -> Self {
        Self::new(InferenceConfig::cpu(), BatchConfig::default())
    }

    /// Load model
    pub async fn load_model<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.runtime.load_model(path).await
    }

    /// Process a batch of samples
    pub async fn process_batch(&self, batch: &Array2<f32>) -> Result<ArrayD<f32>> {
        // Convert to dynamic array
        let input = batch.clone().into_dyn();

        // Run inference
        let output = self.runtime.infer(&input)?;

        Ok(output.output)
    }

    /// Process entire dataset in batches
    pub async fn process_dataset(&self, data: &Array2<f32>) -> Result<BatchInferenceResult> {
        let num_samples = data.nrows();
        let batch_size = self.config.batch_size;
        let num_batches = (num_samples + batch_size - 1) / batch_size;

        let mut outputs = Vec::new();
        let start = std::time::Instant::now();

        for batch_idx in 0..num_batches {
            let start_idx = batch_idx * batch_size;
            let end_idx = (start_idx + batch_size).min(num_samples);

            let batch = data.slice(ndarray::s![start_idx..end_idx, ..]).to_owned();
            let output = self.process_batch(&batch).await?;
            outputs.push(output);

            if self.config.show_progress && batch_idx % 10 == 0 {
                tracing::info!(
                    "Processed batch {}/{} ({:.1}%)",
                    batch_idx + 1,
                    num_batches,
                    (batch_idx + 1) as f64 / num_batches as f64 * 100.0
                );
            }
        }

        let total_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        let avg_time_per_sample_ms = total_time_ms / num_samples as f64;

        Ok(BatchInferenceResult {
            outputs,
            num_samples,
            total_time_ms,
            avg_time_per_sample_ms,
        })
    }

    /// Process feature set
    pub async fn process_features(&self, features: &FeatureSet) -> Result<BatchInferenceResult> {
        // Convert features to f32
        let data = features.features.mapv(|x| x as f32);
        self.process_dataset(&data).await
    }

    /// Process in parallel
    pub async fn process_parallel(&self, data: &Array2<f32>) -> Result<BatchInferenceResult> {
        let num_samples = data.nrows();
        let batch_size = self.config.batch_size;
        let num_batches = (num_samples + batch_size - 1) / batch_size;
        let num_workers = self.config.num_workers;

        let mut tasks = Vec::new();
        let start = std::time::Instant::now();

        for batch_idx in 0..num_batches {
            let start_idx = batch_idx * batch_size;
            let end_idx = (start_idx + batch_size).min(num_samples);

            let batch = data.slice(ndarray::s![start_idx..end_idx, ..]).to_owned();

            // Note: This is simplified - proper implementation would share runtime across tasks
            // For now, process sequentially
            let output = self.process_batch(&batch).await?;
            tasks.push(output);
        }

        let total_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        let avg_time_per_sample_ms = total_time_ms / num_samples as f64;

        Ok(BatchInferenceResult {
            outputs: tasks,
            num_samples,
            total_time_ms,
            avg_time_per_sample_ms,
        })
    }

    /// Get runtime
    pub fn runtime(&self) -> &InferenceRuntime {
        &self.runtime
    }

    /// Get runtime (mutable)
    pub fn runtime_mut(&mut self) -> &mut InferenceRuntime {
        &mut self.runtime
    }

    /// Get batch configuration
    pub fn config(&self) -> &BatchConfig {
        &self.config
    }

    /// Set batch size
    pub fn set_batch_size(&mut self, size: usize) {
        self.config.batch_size = size;
    }

    /// Set number of workers
    pub fn set_num_workers(&mut self, workers: usize) {
        self.config.num_workers = workers;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config() {
        let config = BatchConfig::default();
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.num_workers, 4);
    }

    #[test]
    fn test_batch_inference_creation() {
        let batch = BatchInference::default_cpu();
        assert_eq!(batch.config().batch_size, 32);
    }
}
