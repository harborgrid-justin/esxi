//! Data lineage tracking using directed acyclic graphs (DAGs)

use crate::error::{GovernanceError, Result};
use chrono::{DateTime, Utc};
use petgraph::algo::{has_path_connecting, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Data lineage tracker using DAG representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageTracker {
    /// The lineage graph
    #[serde(skip)]
    graph: DiGraph<DataNode, LineageEdge>,
    /// Node lookup by entity ID
    #[serde(skip)]
    node_index: HashMap<String, NodeIndex>,
    /// Serialized nodes for persistence
    nodes: Vec<(NodeIndex, DataNode)>,
    /// Serialized edges for persistence
    edges: Vec<(NodeIndex, NodeIndex, LineageEdge)>,
}

/// Node in the lineage graph representing a data entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataNode {
    /// Unique identifier
    pub id: Uuid,
    /// Entity identifier (dataset name, table name, etc.)
    pub entity_id: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Owner
    pub owner: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Custom properties
    pub properties: HashMap<String, String>,
}

/// Edge in the lineage graph representing a transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    /// Transformation type
    pub transformation_type: TransformationType,
    /// Transformation description
    pub description: Option<String>,
    /// Process/job that performed the transformation
    pub process_id: Option<String>,
    /// Timestamp when transformation occurred
    pub timestamp: DateTime<Utc>,
    /// Fields involved in transformation
    pub fields: Vec<FieldMapping>,
    /// Custom properties
    pub properties: HashMap<String, String>,
}

/// Entity type in the lineage graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityType {
    /// Source system or external data source
    Source,
    /// Dataset
    Dataset,
    /// Table
    Table,
    /// View
    View,
    /// Materialized view
    MaterializedView,
    /// ETL/transformation process
    Process,
    /// Report or dashboard
    Report,
    /// ML model
    Model,
    /// API endpoint
    ApiEndpoint,
    /// File
    File,
    /// Custom entity type
    Custom(String),
}

/// Transformation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransformationType {
    /// Direct copy/load
    Copy,
    /// Filtering/selection
    Filter,
    /// Projection/column selection
    Project,
    /// Aggregation
    Aggregate,
    /// Join operation
    Join,
    /// Union operation
    Union,
    /// Transformation/mapping
    Transform,
    /// Enrichment
    Enrich,
    /// Data quality cleaning
    Cleanse,
    /// Machine learning training
    MlTrain,
    /// Machine learning prediction
    MlPredict,
    /// Custom transformation
    Custom(String),
}

/// Field-level lineage mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Source field path
    pub source_field: String,
    /// Target field path
    pub target_field: String,
    /// Transformation logic/expression
    pub transformation: Option<String>,
}

/// Lineage path from source to target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineagePath {
    /// Nodes in the path
    pub nodes: Vec<DataNode>,
    /// Edges in the path
    pub edges: Vec<LineageEdge>,
    /// Total number of transformations
    pub transformation_count: usize,
}

/// Lineage impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    /// Entity being analyzed
    pub entity_id: String,
    /// Upstream dependencies (sources)
    pub upstream: Vec<DataNode>,
    /// Downstream dependents (targets)
    pub downstream: Vec<DataNode>,
    /// Number of upstream hops
    pub upstream_depth: usize,
    /// Number of downstream hops
    pub downstream_depth: usize,
}

impl LineageTracker {
    /// Create a new lineage tracker
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_index: HashMap::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a data node to the lineage graph
    pub fn add_node(&mut self, node: DataNode) -> Result<()> {
        if self.node_index.contains_key(&node.entity_id) {
            return Err(GovernanceError::lineage(format!(
                "Node already exists: {}",
                node.entity_id
            )));
        }

        let idx = self.graph.add_node(node.clone());
        self.node_index.insert(node.entity_id.clone(), idx);
        self.nodes.push((idx, node));

        Ok(())
    }

    /// Add a lineage edge between two nodes
    pub fn add_lineage(
        &mut self,
        source_id: &str,
        target_id: &str,
        edge: LineageEdge,
    ) -> Result<()> {
        let source_idx = self
            .node_index
            .get(source_id)
            .ok_or_else(|| GovernanceError::lineage(format!("Source node not found: {}", source_id)))?;

        let target_idx = self
            .node_index
            .get(target_id)
            .ok_or_else(|| GovernanceError::lineage(format!("Target node not found: {}", target_id)))?;

        // Check for circular dependencies
        if has_path_connecting(&self.graph, *target_idx, *source_idx, None) {
            return Err(GovernanceError::CircularDependency);
        }

        let edge_idx = self.graph.add_edge(*source_idx, *target_idx, edge.clone());
        self.edges.push((*source_idx, *target_idx, edge));

        Ok(())
    }

    /// Get a node by entity ID
    pub fn get_node(&self, entity_id: &str) -> Result<&DataNode> {
        let idx = self
            .node_index
            .get(entity_id)
            .ok_or_else(|| GovernanceError::lineage(format!("Node not found: {}", entity_id)))?;

        self.graph
            .node_weight(*idx)
            .ok_or_else(|| GovernanceError::lineage(format!("Node weight not found: {}", entity_id)))
    }

    /// Get upstream dependencies (sources) for an entity
    pub fn get_upstream(&self, entity_id: &str) -> Result<Vec<DataNode>> {
        let idx = self
            .node_index
            .get(entity_id)
            .ok_or_else(|| GovernanceError::lineage(format!("Node not found: {}", entity_id)))?;

        let mut upstream = Vec::new();
        let mut visited = vec![*idx];
        let mut queue = vec![*idx];

        while let Some(current) = queue.pop() {
            for edge in self.graph.edges_directed(current, petgraph::Direction::Incoming) {
                let source = edge.source();
                if !visited.contains(&source) {
                    visited.push(source);
                    queue.push(source);
                    if let Some(node) = self.graph.node_weight(source) {
                        upstream.push(node.clone());
                    }
                }
            }
        }

        Ok(upstream)
    }

    /// Get downstream dependents (targets) for an entity
    pub fn get_downstream(&self, entity_id: &str) -> Result<Vec<DataNode>> {
        let idx = self
            .node_index
            .get(entity_id)
            .ok_or_else(|| GovernanceError::lineage(format!("Node not found: {}", entity_id)))?;

        let mut downstream = Vec::new();
        let mut visited = vec![*idx];
        let mut queue = vec![*idx];

        while let Some(current) = queue.pop() {
            for edge in self.graph.edges_directed(current, petgraph::Direction::Outgoing) {
                let target = edge.target();
                if !visited.contains(&target) {
                    visited.push(target);
                    queue.push(target);
                    if let Some(node) = self.graph.node_weight(target) {
                        downstream.push(node.clone());
                    }
                }
            }
        }

        Ok(downstream)
    }

    /// Get direct upstream dependencies (immediate parents)
    pub fn get_direct_upstream(&self, entity_id: &str) -> Result<Vec<DataNode>> {
        let idx = self
            .node_index
            .get(entity_id)
            .ok_or_else(|| GovernanceError::lineage(format!("Node not found: {}", entity_id)))?;

        let mut upstream = Vec::new();
        for edge in self.graph.edges_directed(*idx, petgraph::Direction::Incoming) {
            let source = edge.source();
            if let Some(node) = self.graph.node_weight(source) {
                upstream.push(node.clone());
            }
        }

        Ok(upstream)
    }

    /// Get direct downstream dependents (immediate children)
    pub fn get_direct_downstream(&self, entity_id: &str) -> Result<Vec<DataNode>> {
        let idx = self
            .node_index
            .get(entity_id)
            .ok_or_else(|| GovernanceError::lineage(format!("Node not found: {}", entity_id)))?;

        let mut downstream = Vec::new();
        for edge in self.graph.edges_directed(*idx, petgraph::Direction::Outgoing) {
            let target = edge.target();
            if let Some(node) = self.graph.node_weight(target) {
                downstream.push(node.clone());
            }
        }

        Ok(downstream)
    }

    /// Perform impact analysis for an entity
    pub fn analyze_impact(&self, entity_id: &str) -> Result<ImpactAnalysis> {
        let upstream = self.get_upstream(entity_id)?;
        let downstream = self.get_downstream(entity_id)?;

        Ok(ImpactAnalysis {
            entity_id: entity_id.to_string(),
            upstream_depth: upstream.len(),
            downstream_depth: downstream.len(),
            upstream,
            downstream,
        })
    }

    /// Get all source nodes (nodes with no incoming edges)
    pub fn get_sources(&self) -> Vec<DataNode> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                self.graph
                    .edges_directed(idx, petgraph::Direction::Incoming)
                    .count()
                    == 0
            })
            .filter_map(|idx| self.graph.node_weight(idx))
            .cloned()
            .collect()
    }

    /// Get all sink nodes (nodes with no outgoing edges)
    pub fn get_sinks(&self) -> Vec<DataNode> {
        self.graph
            .node_indices()
            .filter(|&idx| {
                self.graph
                    .edges_directed(idx, petgraph::Direction::Outgoing)
                    .count()
                    == 0
            })
            .filter_map(|idx| self.graph.node_weight(idx))
            .cloned()
            .collect()
    }

    /// Get topological sort of the lineage graph
    pub fn get_topological_order(&self) -> Result<Vec<DataNode>> {
        let sorted = toposort(&self.graph, None)
            .map_err(|_| GovernanceError::CircularDependency)?;

        Ok(sorted
            .into_iter()
            .filter_map(|idx| self.graph.node_weight(idx))
            .cloned()
            .collect())
    }

    /// Trace lineage path from source to target
    pub fn trace_path(&self, source_id: &str, target_id: &str) -> Result<Vec<LineagePath>> {
        let source_idx = self
            .node_index
            .get(source_id)
            .ok_or_else(|| GovernanceError::lineage(format!("Source node not found: {}", source_id)))?;

        let target_idx = self
            .node_index
            .get(target_id)
            .ok_or_else(|| GovernanceError::lineage(format!("Target node not found: {}", target_id)))?;

        let mut paths = Vec::new();
        let mut current_path = Vec::new();
        self.dfs_paths(*source_idx, *target_idx, &mut current_path, &mut paths);

        Ok(paths)
    }

    /// Depth-first search to find all paths
    fn dfs_paths(
        &self,
        current: NodeIndex,
        target: NodeIndex,
        current_path: &mut Vec<NodeIndex>,
        all_paths: &mut Vec<LineagePath>,
    ) {
        current_path.push(current);

        if current == target {
            let nodes: Vec<DataNode> = current_path
                .iter()
                .filter_map(|&idx| self.graph.node_weight(idx))
                .cloned()
                .collect();

            let edges: Vec<LineageEdge> = current_path
                .windows(2)
                .filter_map(|window| {
                    self.graph
                        .find_edge(window[0], window[1])
                        .and_then(|edge_idx| self.graph.edge_weight(edge_idx))
                })
                .cloned()
                .collect();

            all_paths.push(LineagePath {
                transformation_count: edges.len(),
                nodes,
                edges,
            });
        } else {
            for edge in self.graph.edges_directed(current, petgraph::Direction::Outgoing) {
                let next = edge.target();
                if !current_path.contains(&next) {
                    self.dfs_paths(next, target, current_path, all_paths);
                }
            }
        }

        current_path.pop();
    }

    /// Get the number of nodes in the lineage graph
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get the number of edges in the lineage graph
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}

impl Default for LineageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineage_creation() {
        let tracker = LineageTracker::new();
        assert_eq!(tracker.node_count(), 0);
        assert_eq!(tracker.edge_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut tracker = LineageTracker::new();
        let node = DataNode {
            id: Uuid::new_v4(),
            entity_id: "test_dataset".to_string(),
            entity_type: EntityType::Dataset,
            name: "Test Dataset".to_string(),
            description: None,
            owner: None,
            created_at: Utc::now(),
            properties: HashMap::new(),
        };

        tracker.add_node(node).unwrap();
        assert_eq!(tracker.node_count(), 1);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut tracker = LineageTracker::new();

        let node1 = DataNode {
            id: Uuid::new_v4(),
            entity_id: "node1".to_string(),
            entity_type: EntityType::Dataset,
            name: "Node 1".to_string(),
            description: None,
            owner: None,
            created_at: Utc::now(),
            properties: HashMap::new(),
        };

        let node2 = DataNode {
            id: Uuid::new_v4(),
            entity_id: "node2".to_string(),
            entity_type: EntityType::Dataset,
            name: "Node 2".to_string(),
            description: None,
            owner: None,
            created_at: Utc::now(),
            properties: HashMap::new(),
        };

        tracker.add_node(node1).unwrap();
        tracker.add_node(node2).unwrap();

        let edge = LineageEdge {
            transformation_type: TransformationType::Copy,
            description: None,
            process_id: None,
            timestamp: Utc::now(),
            fields: Vec::new(),
            properties: HashMap::new(),
        };

        tracker.add_lineage("node1", "node2", edge.clone()).unwrap();
        let result = tracker.add_lineage("node2", "node1", edge);
        assert!(result.is_err());
    }
}
