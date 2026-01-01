//! Pipeline node abstraction
//!
//! Defines the trait for pipeline nodes (transforms and models)

use crate::Result;
use ndarray::ArrayD;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

/// Type of pipeline node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Data transformation node
    Transform,

    /// Model inference node
    Model,

    /// Custom node type
    Custom,
}

/// Trait for pipeline nodes
pub trait PipelineNode: Send + Sync + Debug {
    /// Get the node's unique identifier
    fn id(&self) -> Uuid;

    /// Get the node's name
    fn name(&self) -> &str;

    /// Get the node type
    fn node_type(&self) -> NodeType;

    /// Execute the node on input data
    fn execute(&self, input: ArrayD<f32>) -> Result<ArrayD<f32>>;

    /// Check if this node is compatible with the next node
    fn is_compatible_with(&self, _next: &dyn PipelineNode) -> bool {
        // Default implementation: all nodes are compatible
        true
    }

    /// Get node configuration as JSON
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id(),
            "name": self.name(),
            "type": self.node_type(),
        })
    }

    /// Validate node configuration
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Base implementation for simple nodes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BaseNode {
    /// Node identifier
    pub id: Uuid,

    /// Node name
    pub name: String,

    /// Node type
    pub node_type: NodeType,
}

impl BaseNode {
    /// Create a new base node
    pub fn new(name: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            node_type,
        }
    }
}

/// Node execution context
#[derive(Debug, Clone)]
pub struct NodeContext {
    /// Execution identifier
    pub execution_id: Uuid,

    /// Batch size
    pub batch_size: usize,

    /// Execution start time
    pub start_time: chrono::DateTime<chrono::Utc>,

    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl Default for NodeContext {
    fn default() -> Self {
        Self {
            execution_id: Uuid::new_v4(),
            batch_size: 1,
            start_time: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// Node execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Node identifier
    pub node_id: Uuid,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// Input shape
    pub input_shape: Vec<usize>,

    /// Output shape
    pub output_shape: Vec<usize>,

    /// Memory used in bytes
    pub memory_bytes: usize,
}

impl NodeMetrics {
    /// Create new node metrics
    pub fn new(node_id: Uuid) -> Self {
        Self {
            node_id,
            execution_time_ms: 0,
            input_shape: Vec::new(),
            output_shape: Vec::new(),
            memory_bytes: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_node_creation() {
        let node = BaseNode::new("test_node", NodeType::Transform);
        assert_eq!(node.name, "test_node");
        assert_eq!(node.node_type, NodeType::Transform);
    }

    #[test]
    fn test_node_context_default() {
        let ctx = NodeContext::default();
        assert_eq!(ctx.batch_size, 1);
        assert!(ctx.metadata.is_empty());
    }

    #[test]
    fn test_node_metrics() {
        let node_id = Uuid::new_v4();
        let metrics = NodeMetrics::new(node_id);
        assert_eq!(metrics.node_id, node_id);
        assert_eq!(metrics.execution_time_ms, 0);
    }
}
