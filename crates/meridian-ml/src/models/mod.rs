//! Model management and orchestration

pub mod registry;
pub mod serialization;

pub use registry::{ModelRegistry, ModelMetadata, ModelVersion};
pub use serialization::{ModelFormat, SerializableModel};

use crate::error::{MlError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Base trait for all machine learning models
pub trait Model: Send + Sync {
    /// Get the model type/name
    fn model_type(&self) -> &str;

    /// Get the model version
    fn version(&self) -> &str;

    /// Get model metadata
    fn metadata(&self) -> ModelMetadata;

    /// Validate the model
    fn validate(&self) -> Result<()>;

    /// Get the number of features expected
    fn num_features(&self) -> usize;

    /// Get the output dimension
    fn output_dim(&self) -> usize;

    /// Clone the model as a boxed trait object
    fn clone_boxed(&self) -> Box<dyn Model>;
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model name/identifier
    pub name: String,

    /// Model version
    pub version: String,

    /// Model type (e.g., "random_forest", "neural_network")
    pub model_type: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Model description
    pub description: Option<String>,

    /// Model author
    pub author: Option<String>,

    /// Training metrics
    pub metrics: HashMap<String, f64>,

    /// Hyperparameters
    pub hyperparameters: HashMap<String, serde_json::Value>,

    /// Feature names
    pub feature_names: Vec<String>,

    /// Number of input features
    pub num_features: usize,

    /// Output dimension
    pub output_dim: usize,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

impl ModelMetadata {
    /// Create new model metadata
    pub fn new(name: String, model_type: String, num_features: usize, output_dim: usize) -> Self {
        let now = Utc::now();
        Self {
            name,
            version: "0.1.0".to_string(),
            model_type,
            created_at: now,
            updated_at: now,
            description: None,
            author: None,
            metrics: HashMap::new(),
            hyperparameters: HashMap::new(),
            feature_names: Vec::new(),
            num_features,
            output_dim,
            tags: Vec::new(),
            custom: HashMap::new(),
        }
    }

    /// Add a metric
    pub fn add_metric(&mut self, name: String, value: f64) {
        self.metrics.insert(name, value);
        self.updated_at = Utc::now();
    }

    /// Add a hyperparameter
    pub fn add_hyperparameter(&mut self, name: String, value: serde_json::Value) {
        self.hyperparameters.insert(name, value);
        self.updated_at = Utc::now();
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Set feature names
    pub fn set_feature_names(&mut self, names: Vec<String>) {
        self.feature_names = names;
        self.updated_at = Utc::now();
    }

    /// Set description
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
        self.updated_at = Utc::now();
    }
}

/// Model builder for creating models with configuration
pub struct ModelBuilder {
    metadata: ModelMetadata,
    config: HashMap<String, serde_json::Value>,
}

impl ModelBuilder {
    /// Create a new model builder
    pub fn new(name: String, model_type: String) -> Self {
        Self {
            metadata: ModelMetadata::new(name, model_type, 0, 0),
            config: HashMap::new(),
        }
    }

    /// Set the number of features
    pub fn num_features(mut self, n: usize) -> Self {
        self.metadata.num_features = n;
        self
    }

    /// Set the output dimension
    pub fn output_dim(mut self, n: usize) -> Self {
        self.metadata.output_dim = n;
        self
    }

    /// Set description
    pub fn description(mut self, desc: String) -> Self {
        self.metadata.description = Some(desc);
        self
    }

    /// Add a configuration parameter
    pub fn config<V: Serialize>(mut self, key: String, value: V) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.config.insert(key, json_value);
        }
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: String) -> Self {
        self.metadata.add_tag(tag);
        self
    }

    /// Get the metadata
    pub fn metadata(&self) -> &ModelMetadata {
        &self.metadata
    }

    /// Get the configuration
    pub fn config(&self) -> &HashMap<String, serde_json::Value> {
        &self.config
    }
}

/// Model persistence trait
#[async_trait::async_trait]
pub trait ModelPersistence: Model {
    /// Save the model to a path
    async fn save<P: AsRef<Path> + Send>(&self, path: P) -> Result<()>;

    /// Load a model from a path
    async fn load<P: AsRef<Path> + Send>(path: P) -> Result<Self>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_metadata_creation() {
        let metadata = ModelMetadata::new(
            "test_model".to_string(),
            "test_type".to_string(),
            10,
            2,
        );
        assert_eq!(metadata.name, "test_model");
        assert_eq!(metadata.model_type, "test_type");
        assert_eq!(metadata.num_features, 10);
        assert_eq!(metadata.output_dim, 2);
    }

    #[test]
    fn test_model_builder() {
        let builder = ModelBuilder::new("test".to_string(), "classifier".to_string())
            .num_features(5)
            .output_dim(3)
            .description("Test model".to_string())
            .tag("experimental".to_string());

        assert_eq!(builder.metadata().num_features, 5);
        assert_eq!(builder.metadata().output_dim, 3);
        assert_eq!(builder.metadata().tags.len(), 1);
    }
}
