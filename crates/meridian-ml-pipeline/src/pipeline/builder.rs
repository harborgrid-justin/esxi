//! Fluent pipeline builder
//!
//! Provides a builder pattern for constructing ML pipelines

use super::{Pipeline, PipelineMetadata, PipelineNode};
use crate::models::loader::OnnxModelLoader;
use crate::transforms::*;
use crate::{Error, Result};
use std::path::Path;
use std::sync::Arc;

/// Fluent builder for constructing ML pipelines
pub struct PipelineBuilder {
    name: String,
    version: String,
    nodes: Vec<Arc<dyn PipelineNode>>,
    metadata: PipelineMetadata,
}

impl PipelineBuilder {
    /// Create a new pipeline builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "1.0.0".to_string(),
            nodes: Vec::new(),
            metadata: PipelineMetadata::default(),
        }
    }

    /// Set the pipeline version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self.metadata.updated_at = chrono::Utc::now();
        self
    }

    /// Set the pipeline author
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.metadata.author = author.into();
        self
    }

    /// Set the pipeline description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = description.into();
        self
    }

    /// Add a tag to the pipeline
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.metadata.tags.push(tag.into());
        self
    }

    /// Add a custom property
    pub fn property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.properties.insert(key.into(), value.into());
        self
    }

    /// Add a transformation node to the pipeline
    pub fn add_transform<T: PipelineNode + 'static>(mut self, transform: T) -> Self {
        self.nodes.push(Arc::new(transform));
        self
    }

    /// Add a model from a path (ONNX format)
    pub async fn add_model(mut self, model_path: impl AsRef<Path>) -> Result<Self> {
        use crate::models::loader::ModelLoader;
        let loader = OnnxModelLoader::new();
        let model = loader.load(model_path.as_ref()).await?;
        self.nodes.push(model);
        Ok(self)
    }

    /// Add a normalization step (standard scaler)
    pub fn normalize_standard(self) -> Self {
        self.add_transform(normalize::StandardScaler::new())
    }

    /// Add a normalization step (min-max scaler)
    pub fn normalize_minmax(self, min: f32, max: f32) -> Self {
        self.add_transform(normalize::MinMaxScaler::new(min, max))
    }

    /// Add a categorical encoding step
    pub fn encode_categorical(self, columns: Vec<String>) -> Self {
        self.add_transform(encode::CategoricalEncoder::new(columns))
    }

    /// Add a missing value imputation step
    pub fn impute_missing(self, strategy: impute::ImputeStrategy) -> Self {
        self.add_transform(impute::MissingValueImputer::new(strategy))
    }

    /// Add a feature engineering step
    pub fn engineer_features(self) -> Self {
        self.add_transform(feature::FeatureEngineer::new())
    }

    /// Build the pipeline
    pub async fn build(self) -> Result<Pipeline> {
        if self.nodes.is_empty() {
            return Err(Error::pipeline("Cannot build empty pipeline"));
        }

        let mut pipeline = Pipeline::new(self.name, self.version, self.nodes);
        pipeline.metadata = self.metadata;

        // Validate the pipeline
        pipeline.validate()?;

        Ok(pipeline)
    }

    /// Build the pipeline without validation
    pub async fn build_unchecked(self) -> Pipeline {
        let mut pipeline = Pipeline::new(self.name, self.version, self.nodes);
        pipeline.metadata = self.metadata;
        pipeline
    }
}

/// Quick pipeline builders for common scenarios
impl PipelineBuilder {
    /// Create a simple preprocessing pipeline
    pub fn preprocessing(name: impl Into<String>) -> Self {
        Self::new(name)
            .description("Standard preprocessing pipeline")
            .tag("preprocessing")
    }

    /// Create a fraud detection pipeline
    pub fn fraud_detection(name: impl Into<String>) -> Self {
        Self::new(name)
            .description("Fraud detection pipeline")
            .tag("fraud-detection")
            .tag("classification")
    }

    /// Create a recommendation pipeline
    pub fn recommendation(name: impl Into<String>) -> Self {
        Self::new(name)
            .description("Recommendation pipeline")
            .tag("recommendation")
            .tag("ranking")
    }

    /// Create a time series forecasting pipeline
    pub fn forecasting(name: impl Into<String>) -> Self {
        Self::new(name)
            .description("Time series forecasting pipeline")
            .tag("forecasting")
            .tag("time-series")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builder_basic() {
        let builder = PipelineBuilder::new("test")
            .version("2.0.0")
            .author("test-author")
            .description("test pipeline")
            .tag("test");

        assert_eq!(builder.name, "test");
        assert_eq!(builder.version, "2.0.0");
        assert_eq!(builder.metadata.author, "test-author");
    }

    #[tokio::test]
    async fn test_empty_pipeline_build_fails() {
        let builder = PipelineBuilder::new("empty");
        let result = builder.build().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_preprocessing_builder() {
        let builder = PipelineBuilder::preprocessing("preprocess");
        assert_eq!(builder.metadata.tags, vec!["preprocessing"]);
    }
}
