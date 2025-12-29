//! Model serialization and deserialization

use crate::error::{MlError, Result};
use crate::models::ModelMetadata;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Supported model serialization formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    /// Binary format using bincode
    Bincode,
    /// JSON format
    Json,
    /// ONNX format
    #[cfg(feature = "onnx")]
    Onnx,
    /// Custom format
    Custom,
}

impl ModelFormat {
    /// Get the file extension for the format
    pub fn extension(&self) -> &str {
        match self {
            ModelFormat::Bincode => "bin",
            ModelFormat::Json => "json",
            #[cfg(feature = "onnx")]
            ModelFormat::Onnx => "onnx",
            ModelFormat::Custom => "dat",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "bin" | "bincode" => Some(ModelFormat::Bincode),
            "json" => Some(ModelFormat::Json),
            #[cfg(feature = "onnx")]
            "onnx" => Some(ModelFormat::Onnx),
            _ => None,
        }
    }
}

/// Trait for models that can be serialized
pub trait SerializableModel: Serialize + for<'de> Deserialize<'de> {
    /// Get the model metadata
    fn metadata(&self) -> &ModelMetadata;

    /// Serialize to bytes
    fn to_bytes(&self, format: ModelFormat) -> Result<Vec<u8>> {
        match format {
            ModelFormat::Bincode => {
                bincode::serialize(self).map_err(|e| MlError::Serialization(e.to_string()))
            }
            ModelFormat::Json => {
                serde_json::to_vec(self).map_err(|e| MlError::Serialization(e.to_string()))
            }
            #[cfg(feature = "onnx")]
            ModelFormat::Onnx => Err(MlError::Serialization(
                "ONNX serialization requires special handling".to_string(),
            )),
            ModelFormat::Custom => Err(MlError::Serialization(
                "Custom serialization not implemented".to_string(),
            )),
        }
    }

    /// Deserialize from bytes
    fn from_bytes(bytes: &[u8], format: ModelFormat) -> Result<Self>
    where
        Self: Sized,
    {
        match format {
            ModelFormat::Bincode => {
                bincode::deserialize(bytes).map_err(|e| MlError::Deserialization(e.to_string()))
            }
            ModelFormat::Json => {
                serde_json::from_slice(bytes).map_err(|e| MlError::Deserialization(e.to_string()))
            }
            #[cfg(feature = "onnx")]
            ModelFormat::Onnx => Err(MlError::Deserialization(
                "ONNX deserialization requires special handling".to_string(),
            )),
            ModelFormat::Custom => Err(MlError::Deserialization(
                "Custom deserialization not implemented".to_string(),
            )),
        }
    }

    /// Save to file
    async fn save_to_file<P: AsRef<Path> + Send>(&self, path: P, format: ModelFormat) -> Result<()> {
        let bytes = self.to_bytes(format)?;
        let mut file = fs::File::create(path).await?;
        file.write_all(&bytes).await?;
        Ok(())
    }

    /// Load from file
    async fn load_from_file<P: AsRef<Path> + Send>(path: P, format: ModelFormat) -> Result<Self>
    where
        Self: Sized,
    {
        let mut file = fs::File::open(path).await?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).await?;
        Self::from_bytes(&bytes, format)
    }
}

/// ONNX model export/import utilities
#[cfg(feature = "onnx")]
pub mod onnx {
    use super::*;
    use tract_onnx::prelude::*;

    /// ONNX model wrapper
    pub struct OnnxModel {
        pub model: TypedModel,
        pub metadata: ModelMetadata,
    }

    impl OnnxModel {
        /// Load from ONNX file
        pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
            let model = tract_onnx::onnx()
                .model_for_path(path.as_ref())
                .map_err(|e| MlError::Onnx(e.to_string()))?
                .into_optimized()
                .map_err(|e| MlError::Onnx(e.to_string()))?
                .into_runnable()
                .map_err(|e| MlError::Onnx(e.to_string()))?;

            // Extract metadata from ONNX model properties
            let metadata = ModelMetadata::new(
                path.as_ref()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("onnx_model")
                    .to_string(),
                "onnx".to_string(),
                0, // Will be updated based on model inputs
                0, // Will be updated based on model outputs
            );

            Ok(Self { model, metadata })
        }

        /// Save to ONNX file
        pub async fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
            // Note: tract doesn't support saving ONNX models directly
            // This would require converting back to ONNX format
            Err(MlError::Onnx(
                "ONNX export not yet implemented".to_string(),
            ))
        }

        /// Run inference
        pub fn infer(&self, input: &ndarray::ArrayD<f32>) -> Result<ndarray::ArrayD<f32>> {
            let input_tensor: Tensor = input.clone().into();
            let result = self
                .model
                .run(tvec![input_tensor.into()])
                .map_err(|e| MlError::Inference(e.to_string()))?;

            let output = result[0]
                .to_array_view::<f32>()
                .map_err(|e| MlError::Inference(e.to_string()))?
                .into_dimensionality()
                .map_err(|e| MlError::Inference(e.to_string()))?;

            Ok(output)
        }
    }
}

/// Model checkpoint for incremental learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCheckpoint {
    /// Model metadata
    pub metadata: ModelMetadata,

    /// Training iteration/epoch
    pub iteration: usize,

    /// Training metrics at this checkpoint
    pub metrics: std::collections::HashMap<String, f64>,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Model state (serialized)
    pub state: Vec<u8>,
}

impl ModelCheckpoint {
    /// Create a new checkpoint
    pub fn new(
        metadata: ModelMetadata,
        iteration: usize,
        metrics: std::collections::HashMap<String, f64>,
        state: Vec<u8>,
    ) -> Self {
        Self {
            metadata,
            iteration,
            metrics,
            timestamp: chrono::Utc::now(),
            state,
        }
    }

    /// Save checkpoint to file
    pub async fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let bytes = bincode::serialize(self)?;
        let mut file = fs::File::create(path).await?;
        file.write_all(&bytes).await?;
        Ok(())
    }

    /// Load checkpoint from file
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = fs::File::open(path).await?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).await?;
        Ok(bincode::deserialize(&bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_format() {
        assert_eq!(ModelFormat::Bincode.extension(), "bin");
        assert_eq!(ModelFormat::Json.extension(), "json");

        assert_eq!(
            ModelFormat::from_extension("bin"),
            Some(ModelFormat::Bincode)
        );
        assert_eq!(
            ModelFormat::from_extension("json"),
            Some(ModelFormat::Json)
        );
    }

    #[derive(Serialize, Deserialize)]
    struct TestModel {
        metadata: ModelMetadata,
        weights: Vec<f64>,
    }

    impl SerializableModel for TestModel {
        fn metadata(&self) -> &ModelMetadata {
            &self.metadata
        }
    }

    #[test]
    fn test_serialization() {
        let model = TestModel {
            metadata: ModelMetadata::new("test".to_string(), "test".to_string(), 1, 1),
            weights: vec![1.0, 2.0, 3.0],
        };

        let bytes = model.to_bytes(ModelFormat::Bincode).unwrap();
        assert!(!bytes.is_empty());

        let deserialized = TestModel::from_bytes(&bytes, ModelFormat::Bincode).unwrap();
        assert_eq!(deserialized.weights.len(), 3);
    }
}
