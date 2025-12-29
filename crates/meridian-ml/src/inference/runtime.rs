//! ONNX runtime integration

use crate::error::{MlError, Result};
use crate::inference::{InferenceConfig, InferenceOutput};
use ndarray::{ArrayD, IxDyn};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;

/// Runtime backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeBackend {
    /// CPU backend
    CPU,

    /// GPU backend (CUDA)
    GPU,

    /// OpenCL backend
    OpenCL,

    /// TensorRT backend
    TensorRT,
}

/// Inference runtime for ONNX models
pub struct InferenceRuntime {
    /// Configuration
    config: InferenceConfig,

    /// Model loaded
    model_loaded: bool,

    /// Input shape
    input_shape: Option<Vec<usize>>,

    /// Output shape
    output_shape: Option<Vec<usize>>,

    #[cfg(feature = "onnx")]
    /// ONNX model
    model: Option<tract_onnx::prelude::TypedModel>,
}

impl InferenceRuntime {
    /// Create a new inference runtime
    pub fn new(config: InferenceConfig) -> Self {
        Self {
            config,
            model_loaded: false,
            input_shape: None,
            output_shape: None,
            #[cfg(feature = "onnx")]
            model: None,
        }
    }

    /// Create with default CPU configuration
    pub fn cpu() -> Self {
        Self::new(InferenceConfig::cpu())
    }

    /// Create with GPU configuration
    pub fn gpu(device_id: usize) -> Self {
        Self::new(InferenceConfig::gpu(device_id))
    }

    /// Load ONNX model from file
    #[cfg(feature = "onnx")]
    pub async fn load_model<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        use tract_onnx::prelude::*;

        let model = tract_onnx::onnx()
            .model_for_path(path.as_ref())
            .map_err(|e| MlError::Onnx(e.to_string()))?;

        let model = if self.config.optimize {
            model
                .into_optimized()
                .map_err(|e| MlError::Onnx(e.to_string()))?
        } else {
            model
        };

        let model = model
            .into_runnable()
            .map_err(|e| MlError::Onnx(e.to_string()))?;

        // Extract input/output shapes
        // This is simplified - proper implementation would inspect model metadata
        self.input_shape = Some(vec![1, 3, 224, 224]); // Example shape
        self.output_shape = Some(vec![1, 1000]); // Example shape

        self.model = Some(model);
        self.model_loaded = true;

        Ok(())
    }

    /// Load ONNX model (non-ONNX feature)
    #[cfg(not(feature = "onnx"))]
    pub async fn load_model<P: AsRef<Path>>(&mut self, _path: P) -> Result<()> {
        Err(MlError::Model(
            "ONNX feature not enabled".to_string(),
        ))
    }

    /// Run inference on input data
    #[cfg(feature = "onnx")]
    pub fn infer(&self, input: &ArrayD<f32>) -> Result<InferenceOutput> {
        if !self.model_loaded {
            return Err(MlError::Inference("Model not loaded".to_string()));
        }

        let model = self
            .model
            .as_ref()
            .ok_or_else(|| MlError::Inference("Model not available".to_string()))?;

        let start = Instant::now();

        // Convert input to tensor
        use tract_onnx::prelude::*;
        let input_tensor: Tensor = input.clone().into();

        // Run inference
        let result = model
            .run(tvec![input_tensor.into()])
            .map_err(|e| MlError::Inference(e.to_string()))?;

        // Convert output
        let output = result[0]
            .to_array_view::<f32>()
            .map_err(|e| MlError::Inference(e.to_string()))?
            .into_dimensionality::<IxDyn>()
            .map_err(|e| MlError::Inference(e.to_string()))?
            .to_owned();

        let shape = output.shape().to_vec();
        let inference_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(InferenceOutput {
            output,
            shape,
            inference_time_ms,
        })
    }

    /// Run inference (non-ONNX feature)
    #[cfg(not(feature = "onnx"))]
    pub fn infer(&self, _input: &ArrayD<f32>) -> Result<InferenceOutput> {
        Err(MlError::Inference(
            "ONNX feature not enabled".to_string(),
        ))
    }

    /// Get input shape
    pub fn input_shape(&self) -> Option<&[usize]> {
        self.input_shape.as_deref()
    }

    /// Get output shape
    pub fn output_shape(&self) -> Option<&[usize]> {
        self.output_shape.as_deref()
    }

    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        self.model_loaded
    }

    /// Get backend type
    pub fn backend(&self) -> RuntimeBackend {
        self.config.backend
    }

    /// Warmup the runtime with dummy inputs
    pub fn warmup(&self, num_iterations: usize) -> Result<()> {
        if !self.model_loaded {
            return Err(MlError::Inference("Model not loaded".to_string()));
        }

        let input_shape = self
            .input_shape
            .as_ref()
            .ok_or_else(|| MlError::Inference("Input shape not available".to_string()))?;

        #[cfg(feature = "onnx")]
        {
            let dummy_input = ArrayD::zeros(IxDyn(input_shape));

            for _ in 0..num_iterations {
                self.infer(&dummy_input)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = InferenceRuntime::cpu();
        assert_eq!(runtime.backend(), RuntimeBackend::CPU);
        assert!(!runtime.is_loaded());
    }

    #[test]
    fn test_gpu_runtime() {
        let runtime = InferenceRuntime::gpu(0);
        assert_eq!(runtime.backend(), RuntimeBackend::GPU);
        assert_eq!(runtime.config.gpu_device_id, 0);
    }
}
