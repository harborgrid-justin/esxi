//! Data transformation module.
//!
//! Provides various transformations for geospatial data processing:
//! - Geometry operations (buffer, simplify, centroid, etc.)
//! - Coordinate reference system (CRS) transformations
//! - Spatial and attribute filtering
//! - Spatial aggregation
//! - Spatial joins
//! - Geometry validation
//! - Data enrichment

pub mod aggregate;
pub mod enrich;
pub mod filter;
pub mod geometry;
pub mod join;
pub mod projection;
pub mod validate;

use crate::error::Result;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;

/// Trait for data transformations.
#[async_trait]
pub trait Transform: Send + Sync {
    /// Apply the transformation to a record batch.
    async fn transform(&self, batch: RecordBatch) -> Result<RecordBatch>;

    /// Get transformation name.
    fn name(&self) -> &str;

    /// Get transformation description.
    fn description(&self) -> Option<&str> {
        None
    }

    /// Check if this transform modifies schema.
    fn modifies_schema(&self) -> bool {
        false
    }
}

/// Trait for schema-modifying transformations.
#[async_trait]
pub trait SchemaTransform: Transform {
    /// Get the output schema given an input schema.
    async fn output_schema(&self, input: arrow::datatypes::SchemaRef) -> Result<arrow::datatypes::SchemaRef>;
}

/// Chain multiple transforms together.
pub struct TransformChain {
    transforms: Vec<Box<dyn Transform>>,
}

impl TransformChain {
    /// Create a new transform chain.
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
        }
    }

    /// Add a transform to the chain.
    pub fn add<T: Transform + 'static>(mut self, transform: T) -> Self {
        self.transforms.push(Box::new(transform));
        self
    }

    /// Add a boxed transform to the chain.
    pub fn add_boxed(mut self, transform: Box<dyn Transform>) -> Self {
        self.transforms.push(transform);
        self
    }

    /// Get the number of transforms in the chain.
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }
}

impl Default for TransformChain {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transform for TransformChain {
    async fn transform(&self, mut batch: RecordBatch) -> Result<RecordBatch> {
        for transform in &self.transforms {
            batch = transform.transform(batch).await?;
        }
        Ok(batch)
    }

    fn name(&self) -> &str {
        "TransformChain"
    }

    fn description(&self) -> Option<&str> {
        Some("A chain of multiple transforms")
    }

    fn modifies_schema(&self) -> bool {
        self.transforms.iter().any(|t| t.modifies_schema())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyTransform {
        name: String,
    }

    #[async_trait]
    impl Transform for DummyTransform {
        async fn transform(&self, batch: RecordBatch) -> Result<RecordBatch> {
            Ok(batch)
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_transform_chain() {
        let chain = TransformChain::new()
            .add(DummyTransform {
                name: "transform1".to_string(),
            })
            .add(DummyTransform {
                name: "transform2".to_string(),
            });

        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());
    }
}
