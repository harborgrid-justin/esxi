//! Model management and inference
//!
//! Provides model loading, versioning, and inference capabilities

pub mod registry;
pub mod loader;
pub mod inference;

pub use registry::{ModelRegistry, ModelMetadata, ModelVersion};
pub use loader::{OnnxModelLoader, ModelLoader};
pub use inference::{InferenceEngine, InferenceConfig};

use crate::{Error, Result};
use ndarray::ArrayD;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Model format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    /// ONNX model format
    Onnx,

    /// TensorFlow SavedModel
    TensorFlow,

    /// PyTorch TorchScript
    PyTorch,

    /// Custom format
    Custom,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier
    pub id: Uuid,

    /// Model name
    pub name: String,

    /// Model version
    pub version: String,

    /// Model format
    pub format: ModelFormat,

    /// Model file path
    pub path: PathBuf,

    /// Input shape
    pub input_shape: Vec<usize>,

    /// Output shape
    pub output_shape: Vec<usize>,

    /// Model size in bytes
    pub size_bytes: usize,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Model tags
    pub tags: Vec<String>,

    /// Custom metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl ModelInfo {
    /// Create new model info
    pub fn new(name: impl Into<String>, version: impl Into<String>, path: PathBuf) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            format: ModelFormat::Onnx,
            path,
            input_shape: Vec::new(),
            output_shape: Vec::new(),
            size_bytes: 0,
            created_at: chrono::Utc::now(),
            tags: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add a tag to the model
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Model identifier
    pub model_id: Uuid,

    /// Average inference time in milliseconds
    pub avg_inference_time_ms: f64,

    /// Throughput (predictions per second)
    pub throughput: f64,

    /// Memory usage in bytes
    pub memory_bytes: usize,

    /// Total predictions made
    pub total_predictions: u64,

    /// Error rate
    pub error_rate: f64,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ModelMetrics {
    /// Create new model metrics
    pub fn new(model_id: Uuid) -> Self {
        Self {
            model_id,
            avg_inference_time_ms: 0.0,
            throughput: 0.0,
            memory_bytes: 0,
            total_predictions: 0,
            error_rate: 0.0,
            updated_at: chrono::Utc::now(),
        }
    }

    /// Update metrics with a new inference
    pub fn record_inference(&mut self, duration_ms: f64, success: bool) {
        self.total_predictions += 1;

        // Update average inference time (exponential moving average)
        let alpha = 0.1;
        self.avg_inference_time_ms =
            alpha * duration_ms + (1.0 - alpha) * self.avg_inference_time_ms;

        // Update throughput
        if self.avg_inference_time_ms > 0.0 {
            self.throughput = 1000.0 / self.avg_inference_time_ms;
        }

        // Update error rate
        if !success {
            let errors = (self.error_rate * (self.total_predictions - 1) as f64) + 1.0;
            self.error_rate = errors / self.total_predictions as f64;
        }

        self.updated_at = chrono::Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_model_format_variants() {
        assert_eq!(ModelFormat::Onnx, ModelFormat::Onnx);
        assert_ne!(ModelFormat::Onnx, ModelFormat::TensorFlow);
    }

    #[test]
    fn test_model_info_creation() {
        let info = ModelInfo::new("test_model", "1.0.0", PathBuf::from("/models/test.onnx"));
        assert_eq!(info.name, "test_model");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.format, ModelFormat::Onnx);
    }

    #[test]
    fn test_model_info_tags() {
        let mut info = ModelInfo::new("test", "1.0", PathBuf::from("/test"));
        info.add_tag("classification");
        info.add_tag("production");
        assert_eq!(info.tags.len(), 2);
        assert!(info.tags.contains(&"classification".to_string()));
    }

    #[test]
    fn test_model_metrics() {
        let model_id = Uuid::new_v4();
        let mut metrics = ModelMetrics::new(model_id);

        metrics.record_inference(10.0, true);
        assert!(metrics.avg_inference_time_ms > 0.0);
        assert_eq!(metrics.total_predictions, 1);
        assert_eq!(metrics.error_rate, 0.0);

        metrics.record_inference(20.0, false);
        assert_eq!(metrics.total_predictions, 2);
        assert!(metrics.error_rate > 0.0);
    }
}
