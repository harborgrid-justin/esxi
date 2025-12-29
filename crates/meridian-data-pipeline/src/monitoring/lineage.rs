//! Data lineage tracking for pipelines.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Data lineage tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLineage {
    /// Lineage ID.
    pub id: Uuid,
    /// Pipeline run ID.
    pub pipeline_run_id: Uuid,
    /// Lineage nodes (sources, transforms, sinks).
    pub nodes: Vec<LineageNode>,
    /// Edges connecting nodes.
    pub edges: Vec<LineageEdge>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

/// A node in the data lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageNode {
    /// Node ID.
    pub id: Uuid,
    /// Node type.
    pub node_type: LineageNodeType,
    /// Node name.
    pub name: String,
    /// Node description.
    pub description: Option<String>,
    /// Node metadata.
    pub metadata: HashMap<String, String>,
    /// Timestamp when node was executed.
    pub executed_at: Option<DateTime<Utc>>,
}

/// Type of lineage node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageNodeType {
    /// Data source.
    Source,
    /// Data transformation.
    Transform,
    /// Data sink.
    Sink,
}

/// An edge connecting two nodes in the lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEdge {
    /// Edge ID.
    pub id: Uuid,
    /// Source node ID.
    pub from_node: Uuid,
    /// Target node ID.
    pub to_node: Uuid,
    /// Number of records transferred.
    pub records: Option<usize>,
    /// Schema information.
    pub schema: Option<String>,
}

impl DataLineage {
    /// Create a new data lineage tracker.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            pipeline_run_id: Uuid::new_v4(),
            nodes: Vec::new(),
            edges: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Create with a specific pipeline run ID.
    pub fn with_run_id(run_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            pipeline_run_id: run_id,
            nodes: Vec::new(),
            edges: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Add a source node.
    pub fn add_source(&mut self, name: impl Into<String>, description: Option<String>) -> Uuid {
        let node = LineageNode {
            id: Uuid::new_v4(),
            node_type: LineageNodeType::Source,
            name: name.into(),
            description,
            metadata: HashMap::new(),
            executed_at: Some(Utc::now()),
        };
        let id = node.id;
        self.nodes.push(node);
        id
    }

    /// Add a transform node.
    pub fn add_transform(&mut self, name: impl Into<String>, description: Option<String>) -> Uuid {
        let node = LineageNode {
            id: Uuid::new_v4(),
            node_type: LineageNodeType::Transform,
            name: name.into(),
            description,
            metadata: HashMap::new(),
            executed_at: Some(Utc::now()),
        };
        let id = node.id;
        self.nodes.push(node);
        id
    }

    /// Add a sink node.
    pub fn add_sink(&mut self, name: impl Into<String>, description: Option<String>) -> Uuid {
        let node = LineageNode {
            id: Uuid::new_v4(),
            node_type: LineageNodeType::Sink,
            name: name.into(),
            description,
            metadata: HashMap::new(),
            executed_at: Some(Utc::now()),
        };
        let id = node.id;
        self.nodes.push(node);
        id
    }

    /// Add an edge between two nodes.
    pub fn add_edge(&mut self, from: Uuid, to: Uuid, records: Option<usize>) -> Uuid {
        let edge = LineageEdge {
            id: Uuid::new_v4(),
            from_node: from,
            to_node: to,
            records,
            schema: None,
        };
        let id = edge.id;
        self.edges.push(edge);
        id
    }

    /// Add metadata to a node.
    pub fn add_node_metadata(&mut self, node_id: Uuid, key: String, value: String) {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
            node.metadata.insert(key, value);
        }
    }

    /// Add schema information to an edge.
    pub fn add_edge_schema(&mut self, edge_id: Uuid, schema: String) {
        if let Some(edge) = self.edges.iter_mut().find(|e| e.id == edge_id) {
            edge.schema = Some(schema);
        }
    }

    /// Get all source nodes.
    pub fn get_sources(&self) -> Vec<&LineageNode> {
        self.nodes
            .iter()
            .filter(|n| n.node_type == LineageNodeType::Source)
            .collect()
    }

    /// Get all transform nodes.
    pub fn get_transforms(&self) -> Vec<&LineageNode> {
        self.nodes
            .iter()
            .filter(|n| n.node_type == LineageNodeType::Transform)
            .collect()
    }

    /// Get all sink nodes.
    pub fn get_sinks(&self) -> Vec<&LineageNode> {
        self.nodes
            .iter()
            .filter(|n| n.node_type == LineageNodeType::Sink)
            .collect()
    }

    /// Get node by ID.
    pub fn get_node(&self, node_id: Uuid) -> Option<&LineageNode> {
        self.nodes.iter().find(|n| n.id == node_id)
    }

    /// Get edges from a node.
    pub fn get_edges_from(&self, node_id: Uuid) -> Vec<&LineageEdge> {
        self.edges.iter().filter(|e| e.from_node == node_id).collect()
    }

    /// Get edges to a node.
    pub fn get_edges_to(&self, node_id: Uuid) -> Vec<&LineageEdge> {
        self.edges.iter().filter(|e| e.to_node == node_id).collect()
    }

    /// Export lineage as JSON.
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Export lineage as DOT format for visualization.
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph pipeline_lineage {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box];\n\n");

        // Add nodes
        for node in &self.nodes {
            let shape = match node.node_type {
                LineageNodeType::Source => "cylinder",
                LineageNodeType::Transform => "box",
                LineageNodeType::Sink => "folder",
            };
            let color = match node.node_type {
                LineageNodeType::Source => "lightblue",
                LineageNodeType::Transform => "lightgreen",
                LineageNodeType::Sink => "lightyellow",
            };

            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\", shape={}, style=filled, fillcolor={}];\n",
                node.id, node.name, shape, color
            ));
        }

        dot.push_str("\n");

        // Add edges
        for edge in &self.edges {
            let label = edge
                .records
                .map(|r| format!("{} records", r))
                .unwrap_or_else(|| "".to_string());

            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                edge.from_node, edge.to_node, label
            ));
        }

        dot.push_str("}\n");
        dot
    }
}

impl Default for DataLineage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineage_creation() {
        let lineage = DataLineage::new();
        assert_eq!(lineage.nodes.len(), 0);
        assert_eq!(lineage.edges.len(), 0);
    }

    #[test]
    fn test_add_nodes() {
        let mut lineage = DataLineage::new();

        let source_id = lineage.add_source("file_source", Some("GeoJSON input".to_string()));
        let transform_id = lineage.add_transform("projection", Some("EPSG:4326 to EPSG:3857".to_string()));
        let sink_id = lineage.add_sink("database_sink", Some("PostGIS output".to_string()));

        assert_eq!(lineage.nodes.len(), 3);
        assert_eq!(lineage.get_sources().len(), 1);
        assert_eq!(lineage.get_transforms().len(), 1);
        assert_eq!(lineage.get_sinks().len(), 1);
    }

    #[test]
    fn test_add_edges() {
        let mut lineage = DataLineage::new();

        let source_id = lineage.add_source("file_source", None);
        let transform_id = lineage.add_transform("filter", None);
        let sink_id = lineage.add_sink("file_sink", None);

        lineage.add_edge(source_id, transform_id, Some(1000));
        lineage.add_edge(transform_id, sink_id, Some(800));

        assert_eq!(lineage.edges.len(), 2);
        assert_eq!(lineage.get_edges_from(source_id).len(), 1);
        assert_eq!(lineage.get_edges_to(sink_id).len(), 1);
    }

    #[test]
    fn test_node_metadata() {
        let mut lineage = DataLineage::new();
        let node_id = lineage.add_source("test_source", None);

        lineage.add_node_metadata(node_id, "file_path".to_string(), "/data/input.geojson".to_string());

        let node = lineage.get_node(node_id).unwrap();
        assert_eq!(node.metadata.get("file_path"), Some(&"/data/input.geojson".to_string()));
    }

    #[test]
    fn test_json_export() {
        let mut lineage = DataLineage::new();
        lineage.add_source("source", None);
        lineage.add_sink("sink", None);

        let json = lineage.to_json();
        assert!(json.is_ok());
    }

    #[test]
    fn test_dot_export() {
        let mut lineage = DataLineage::new();
        let source = lineage.add_source("source", None);
        let sink = lineage.add_sink("sink", None);
        lineage.add_edge(source, sink, Some(100));

        let dot = lineage.to_dot();
        assert!(dot.contains("digraph pipeline_lineage"));
        assert!(dot.contains("source"));
        assert!(dot.contains("sink"));
        assert!(dot.contains("100 records"));
    }
}
