//! Model inference and deployment

pub mod runtime;
pub mod batch;

pub use runtime::{InferenceRuntime, RuntimeBackend};
pub use batch::{BatchInference, BatchConfig};

use crate::error::{MlError, Result};
use ndarray::{Array1, ArrayD};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Inference configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Backend to use (CPU, GPU, etc.)
    pub backend: RuntimeBackend,

    /// Batch size for inference
    pub batch_size: usize,

    /// Number of threads
    pub num_threads: Option<usize>,

    /// Enable optimizations
    pub optimize: bool,

    /// Enable GPU acceleration
    pub use_gpu: bool,

    /// GPU device ID
    pub gpu_device_id: usize,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            backend: RuntimeBackend::CPU,
            batch_size: 32,
            num_threads: None,
            optimize: true,
            use_gpu: false,
            gpu_device_id: 0,
        }
    }
}

impl InferenceConfig {
    /// Create CPU configuration
    pub fn cpu() -> Self {
        Self {
            backend: RuntimeBackend::CPU,
            use_gpu: false,
            ..Default::default()
        }
    }

    /// Create GPU configuration
    pub fn gpu(device_id: usize) -> Self {
        Self {
            backend: RuntimeBackend::GPU,
            use_gpu: true,
            gpu_device_id: device_id,
            ..Default::default()
        }
    }

    /// Set batch size
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set number of threads
    pub fn with_num_threads(mut self, threads: usize) -> Self {
        self.num_threads = Some(threads);
        self
    }

    /// Enable optimizations
    pub fn with_optimization(mut self, enable: bool) -> Self {
        self.optimize = enable;
        self
    }
}

/// Inference result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceOutput {
    /// Output tensor
    pub output: ArrayD<f32>,

    /// Output shape
    pub shape: Vec<usize>,

    /// Inference time (milliseconds)
    pub inference_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_config() {
        let config = InferenceConfig::cpu();
        assert_eq!(config.backend, RuntimeBackend::CPU);
        assert!(!config.use_gpu);

        let gpu_config = InferenceConfig::gpu(0);
        assert_eq!(gpu_config.backend, RuntimeBackend::GPU);
        assert!(gpu_config.use_gpu);
    }
}
