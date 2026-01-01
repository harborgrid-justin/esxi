//! ML Pipeline orchestration and execution
//!
//! Provides a fluent API for building and executing ML pipelines with
//! transformations and models.

pub mod builder;
pub mod executor;
pub mod node;

pub use builder::PipelineBuilder;
pub use executor::PipelineExecutor;
pub use node::{PipelineNode, NodeType, BaseNode, NodeMetrics};

use crate::{Error, Result};
use ndarray::ArrayD;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// A complete ML pipeline with transformations and models
#[derive(Clone)]
pub struct Pipeline {
    /// Unique pipeline identifier
    pub id: Uuid,

    /// Pipeline name
    pub name: String,

    /// Version of the pipeline
    pub version: String,

    /// Ordered sequence of pipeline nodes
    pub nodes: Vec<Arc<dyn PipelineNode>>,

    /// Pipeline metadata
    pub metadata: PipelineMetadata,

    /// Pipeline executor
    executor: Arc<PipelineExecutor>,
}

/// Metadata for a pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetadata {
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Pipeline author
    pub author: String,

    /// Pipeline description
    pub description: String,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Custom properties
    #[serde(default)]
    pub properties: std::collections::HashMap<String, String>,
}

impl Default for PipelineMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            updated_at: now,
            author: "system".to_string(),
            description: String::new(),
            tags: Vec::new(),
            properties: std::collections::HashMap::new(),
        }
    }
}

impl Pipeline {
    /// Create a new pipeline
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        nodes: Vec<Arc<dyn PipelineNode>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            nodes,
            metadata: PipelineMetadata::default(),
            executor: Arc::new(PipelineExecutor::new()),
        }
    }

    /// Execute the pipeline on input data
    pub async fn execute(&self, input: ArrayD<f32>) -> Result<ArrayD<f32>> {
        self.executor.execute(&self.nodes, input).await
    }

    /// Execute prediction (alias for execute)
    pub async fn predict(&self, input: ArrayD<f32>) -> Result<ArrayD<f32>> {
        self.execute(input).await
    }

    /// Validate the pipeline structure
    pub fn validate(&self) -> Result<()> {
        if self.nodes.is_empty() {
            return Err(Error::pipeline("Pipeline must contain at least one node"));
        }

        // Validate node compatibility
        for window in self.nodes.windows(2) {
            let current = &window[0];
            let next = &window[1];

            if !current.is_compatible_with(next.as_ref()) {
                return Err(Error::pipeline(format!(
                    "Incompatible nodes: {} -> {}",
                    current.name(),
                    next.name()
                )));
            }
        }

        Ok(())
    }

    /// Get pipeline statistics
    pub fn stats(&self) -> PipelineStats {
        PipelineStats {
            node_count: self.nodes.len(),
            transform_count: self.nodes.iter()
                .filter(|n| matches!(n.node_type(), NodeType::Transform))
                .count(),
            model_count: self.nodes.iter()
                .filter(|n| matches!(n.node_type(), NodeType::Model))
                .count(),
        }
    }
}

/// Pipeline statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStats {
    /// Total number of nodes
    pub node_count: usize,

    /// Number of transform nodes
    pub transform_count: usize,

    /// Number of model nodes
    pub model_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_metadata() {
        let meta = PipelineMetadata::default();
        assert_eq!(meta.author, "system");
        assert!(meta.tags.is_empty());
    }

    #[test]
    fn test_empty_pipeline_validation() {
        let pipeline = Pipeline::new("test", "1.0", vec![]);
        assert!(pipeline.validate().is_err());
    }
}
