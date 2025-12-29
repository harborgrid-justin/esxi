//! Graph data structures for routing

pub mod builder;
pub mod edge;
pub mod node;
pub mod partition;

pub use builder::GraphBuilder;
pub use edge::{Edge, EdgeCost, EdgeId, TurnRestriction};
pub use node::{Node, NodeId};
pub use partition::GraphPartition;

use crate::error::{Result, RoutingError};
use hashbrown::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Main graph structure for routing
///
/// Uses compressed adjacency lists for memory-efficient storage.
/// Supports turn restrictions, time-dependent costs, and partitioning.
#[derive(Debug, Clone, Default)]
pub struct Graph {
    /// Nodes in the graph
    nodes: Vec<Node>,

    /// Edges in the graph
    edges: Vec<Edge>,

    /// Adjacency list: node -> outgoing edges
    adjacency: Vec<Vec<EdgeId>>,

    /// Reverse adjacency list: node -> incoming edges
    reverse_adjacency: Vec<Vec<EdgeId>>,

    /// Spatial index for node lookup
    spatial_index: NodeSpatialIndex,

    /// Turn restrictions
    turn_restrictions: Vec<TurnRestriction>,

    /// Graph partition for hierarchical routing
    partition: Option<Arc<GraphPartition>>,

    /// Metadata
    metadata: GraphMetadata,
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Get number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get node by ID
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
    }

    /// Get edge by ID
    pub fn edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.get(id.0)
    }

    /// Get outgoing edges from a node
    pub fn outgoing_edges(&self, node: NodeId) -> &[EdgeId] {
        self.adjacency.get(node.0).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get incoming edges to a node
    pub fn incoming_edges(&self, node: NodeId) -> &[EdgeId] {
        self.reverse_adjacency.get(node.0).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Find nearest node to a point
    pub fn nearest_node(&self, point: geo_types::Point) -> Option<NodeId> {
        self.spatial_index.nearest(point, &self.nodes)
    }

    /// Find nodes within radius
    pub fn nodes_within_radius(
        &self,
        point: geo_types::Point,
        radius: f64,
    ) -> Vec<NodeId> {
        self.spatial_index.within_radius(point, radius, &self.nodes)
    }

    /// Check if turn is restricted
    pub fn is_turn_restricted(
        &self,
        from_edge: EdgeId,
        via_node: NodeId,
        to_edge: EdgeId,
    ) -> bool {
        self.turn_restrictions.iter().any(|r| {
            r.from_edge == from_edge && r.via_node == via_node && r.to_edge == to_edge
        })
    }

    /// Get turn penalty (in seconds)
    pub fn turn_penalty(&self, from_edge: EdgeId, to_edge: EdgeId) -> f64 {
        let from = self.edge(from_edge)?;
        let to = self.edge(to_edge)?;

        // Calculate angle-based turn penalty
        let angle_diff = (from.bearing - to.bearing).abs();
        let normalized = if angle_diff > 180.0 {
            360.0 - angle_diff
        } else {
            angle_diff
        };

        // More penalty for sharper turns
        Some(match normalized {
            0.0..=30.0 => 2.0,      // Slight turn
            30.0..=90.0 => 5.0,     // Medium turn
            90.0..=150.0 => 10.0,   // Sharp turn
            _ => 15.0,              // U-turn
        })
    }

    /// Get graph partition
    pub fn partition(&self) -> Option<&GraphPartition> {
        self.partition.as_deref()
    }

    /// Set graph partition
    pub fn set_partition(&mut self, partition: GraphPartition) {
        self.partition = Some(Arc::new(partition));
    }

    /// Get metadata
    pub fn metadata(&self) -> &GraphMetadata {
        &self.metadata
    }

    /// Save graph to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::fs::File;

        let file = File::create(path)?;
        let encoder = GzEncoder::new(file, Compression::default());
        bincode::serialize_into(encoder, self)?;
        Ok(())
    }

    /// Load graph from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        use flate2::read::GzDecoder;
        use std::fs::File;

        let file = File::open(path)?;
        let decoder = GzDecoder::new(file);
        let graph = bincode::deserialize_from(decoder)?;
        Ok(graph)
    }

    /// Validate graph consistency
    pub fn validate(&self) -> Result<()> {
        // Check node count matches adjacency
        if self.nodes.len() != self.adjacency.len() {
            return Err(RoutingError::GraphConstruction(
                "Node count mismatch with adjacency".into(),
            ));
        }

        // Check edge references
        for (node_id, adj) in self.adjacency.iter().enumerate() {
            for &edge_id in adj {
                if let Some(edge) = self.edge(edge_id) {
                    if edge.source.0 != node_id {
                        return Err(RoutingError::GraphConstruction(
                            format!("Edge {} has wrong source", edge_id.0),
                        ));
                    }
                } else {
                    return Err(RoutingError::EdgeNotFound(edge_id.0));
                }
            }
        }

        Ok(())
    }
}

/// Spatial index for fast node lookup
#[derive(Debug, Clone, Default)]
struct NodeSpatialIndex {
    /// Simple grid-based spatial index
    /// Maps grid cell -> node IDs
    grid: HashMap<(i32, i32), Vec<NodeId>>,
    cell_size: f64,
}

impl NodeSpatialIndex {
    fn new(cell_size: f64) -> Self {
        Self {
            grid: HashMap::new(),
            cell_size,
        }
    }

    fn grid_cell(&self, point: geo_types::Point) -> (i32, i32) {
        (
            (point.x() / self.cell_size).floor() as i32,
            (point.y() / self.cell_size).floor() as i32,
        )
    }

    fn insert(&mut self, point: geo_types::Point, node_id: NodeId) {
        let cell = self.grid_cell(point);
        self.grid.entry(cell).or_insert_with(Vec::new).push(node_id);
    }

    fn nearest(&self, point: geo_types::Point, nodes: &[Node]) -> Option<NodeId> {
        let cell = self.grid_cell(point);
        let mut best = None;
        let mut best_dist = f64::INFINITY;

        // Search current cell and neighbors
        for dx in -1..=1 {
            for dy in -1..=1 {
                let search_cell = (cell.0 + dx, cell.1 + dy);
                if let Some(node_ids) = self.grid.get(&search_cell) {
                    for &node_id in node_ids {
                        if let Some(node) = nodes.get(node_id.0) {
                            let dist = haversine_distance(point, node.location);
                            if dist < best_dist {
                                best_dist = dist;
                                best = Some(node_id);
                            }
                        }
                    }
                }
            }
        }

        best
    }

    fn within_radius(
        &self,
        point: geo_types::Point,
        radius: f64,
        nodes: &[Node],
    ) -> Vec<NodeId> {
        let mut result = Vec::new();
        let cell = self.grid_cell(point);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let search_cell = (cell.0 + dx, cell.1 + dy);
                if let Some(node_ids) = self.grid.get(&search_cell) {
                    for &node_id in node_ids {
                        if let Some(node) = nodes.get(node_id.0) {
                            if haversine_distance(point, node.location) <= radius {
                                result.push(node_id);
                            }
                        }
                    }
                }
            }
        }

        result
    }
}

/// Graph metadata
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GraphMetadata {
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub source: Option<String>,
    pub bounds: Option<GeoBounds>,
}

/// Geographic bounds
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct GeoBounds {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
}

/// Calculate haversine distance between two points (in meters)
fn haversine_distance(a: geo_types::Point, b: geo_types::Point) -> f64 {
    const EARTH_RADIUS: f64 = 6371000.0; // meters

    let lat1 = a.y().to_radians();
    let lat2 = b.y().to_radians();
    let delta_lat = (b.y() - a.y()).to_radians();
    let delta_lon = (b.x() - a.x()).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    EARTH_RADIUS * c
}

// Implement serialization for Graph
impl serde::Serialize for Graph {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Graph", 5)?;
        state.serialize_field("nodes", &self.nodes)?;
        state.serialize_field("edges", &self.edges)?;
        state.serialize_field("adjacency", &self.adjacency)?;
        state.serialize_field("turn_restrictions", &self.turn_restrictions)?;
        state.serialize_field("metadata", &self.metadata)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Graph {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct GraphData {
            nodes: Vec<Node>,
            edges: Vec<Edge>,
            adjacency: Vec<Vec<EdgeId>>,
            turn_restrictions: Vec<TurnRestriction>,
            metadata: GraphMetadata,
        }

        let data = GraphData::deserialize(deserializer)?;
        let mut graph = Graph {
            nodes: data.nodes,
            edges: data.edges,
            adjacency: data.adjacency,
            reverse_adjacency: vec![Vec::new(); data.nodes.len()],
            spatial_index: NodeSpatialIndex::new(0.01), // ~1km cells
            turn_restrictions: data.turn_restrictions,
            partition: None,
            metadata: data.metadata,
        };

        // Rebuild reverse adjacency
        for (node_id, adj) in graph.adjacency.iter().enumerate() {
            for &edge_id in adj {
                if let Some(edge) = graph.edge(edge_id) {
                    graph.reverse_adjacency[edge.target.0].push(edge_id);
                }
            }
        }

        // Rebuild spatial index
        for (i, node) in graph.nodes.iter().enumerate() {
            graph.spatial_index.insert(node.location, NodeId(i));
        }

        Ok(graph)
    }
}
