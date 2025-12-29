//! Graph partitioning for hierarchical routing

use super::{Graph, NodeId, EdgeId};
use crate::error::Result;
use hashbrown::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Graph partition for hierarchical routing
///
/// Divides the graph into cells/regions to enable faster long-distance routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPartition {
    /// Number of partitions
    pub num_partitions: usize,

    /// Node -> partition mapping
    pub node_partition: Vec<PartitionId>,

    /// Border nodes for each partition
    pub border_nodes: Vec<Vec<NodeId>>,

    /// Overlay graph connecting partitions
    pub overlay: OverlayGraph,
}

/// Partition identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartitionId(pub usize);

impl GraphPartition {
    /// Create partition using simple grid-based approach
    pub fn create_grid(graph: &Graph, grid_size: usize) -> Result<Self> {
        let bounds = graph.metadata().bounds.ok_or_else(|| {
            crate::error::RoutingError::GraphConstruction("No bounds available".into())
        })?;

        let lon_step = (bounds.max_lon - bounds.min_lon) / grid_size as f64;
        let lat_step = (bounds.max_lat - bounds.min_lat) / grid_size as f64;

        let mut node_partition = vec![PartitionId(0); graph.node_count()];

        // Assign nodes to partitions
        for (i, node) in (0..graph.node_count()).filter_map(|i| graph.node(NodeId(i)).map(|n| (i, n))) {
            let lon_idx = ((node.location.x() - bounds.min_lon) / lon_step).floor() as usize;
            let lat_idx = ((node.location.y() - bounds.min_lat) / lat_step).floor() as usize;

            let lon_idx = lon_idx.min(grid_size - 1);
            let lat_idx = lat_idx.min(grid_size - 1);

            let partition_id = lat_idx * grid_size + lon_idx;
            node_partition[i] = PartitionId(partition_id);
        }

        let num_partitions = grid_size * grid_size;

        // Find border nodes
        let border_nodes = find_border_nodes(graph, &node_partition, num_partitions);

        // Build overlay graph
        let overlay = build_overlay_graph(graph, &border_nodes);

        Ok(Self {
            num_partitions,
            node_partition,
            border_nodes,
            overlay,
        })
    }

    /// Create partition using METIS-like balanced partitioning
    pub fn create_balanced(graph: &Graph, num_partitions: usize) -> Result<Self> {
        // Simplified balanced partitioning
        // In production, would use METIS or similar library

        let nodes_per_partition = (graph.node_count() + num_partitions - 1) / num_partitions;
        let mut node_partition = Vec::with_capacity(graph.node_count());

        for i in 0..graph.node_count() {
            let partition_id = i / nodes_per_partition;
            let partition_id = partition_id.min(num_partitions - 1);
            node_partition.push(PartitionId(partition_id));
        }

        let border_nodes = find_border_nodes(graph, &node_partition, num_partitions);
        let overlay = build_overlay_graph(graph, &border_nodes);

        Ok(Self {
            num_partitions,
            node_partition,
            border_nodes,
            overlay,
        })
    }

    /// Get partition for a node
    pub fn partition(&self, node: NodeId) -> Option<PartitionId> {
        self.node_partition.get(node.0).copied()
    }

    /// Check if node is a border node
    pub fn is_border_node(&self, node: NodeId) -> bool {
        if let Some(partition) = self.partition(node) {
            self.border_nodes[partition.0].contains(&node)
        } else {
            false
        }
    }

    /// Get border nodes for partition
    pub fn get_border_nodes(&self, partition: PartitionId) -> &[NodeId] {
        &self.border_nodes[partition.0]
    }
}

/// Overlay graph connecting partition border nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayGraph {
    /// Overlay edges: (from_node, to_node) -> distance
    pub edges: HashMap<(NodeId, NodeId), f64>,
}

impl OverlayGraph {
    fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    /// Get shortest distance between border nodes (if available)
    pub fn distance(&self, from: NodeId, to: NodeId) -> Option<f64> {
        self.edges.get(&(from, to)).copied()
    }
}

/// Find border nodes for each partition
fn find_border_nodes(
    graph: &Graph,
    node_partition: &[PartitionId],
    num_partitions: usize,
) -> Vec<Vec<NodeId>> {
    let mut border_nodes = vec![Vec::new(); num_partitions];
    let mut is_border = vec![false; graph.node_count()];

    // A node is a border node if it has an edge to a different partition
    for node_id in 0..graph.node_count() {
        let node = NodeId(node_id);
        let partition = node_partition[node_id];

        for &edge_id in graph.outgoing_edges(node) {
            if let Some(edge) = graph.edge(edge_id) {
                let target_partition = node_partition[edge.target.0];
                if target_partition != partition {
                    if !is_border[node_id] {
                        is_border[node_id] = true;
                        border_nodes[partition.0].push(node);
                    }
                    break;
                }
            }
        }
    }

    border_nodes
}

/// Build overlay graph from border nodes
fn build_overlay_graph(graph: &Graph, border_nodes: &[Vec<NodeId>]) -> OverlayGraph {
    let mut overlay = OverlayGraph::new();

    // For each partition, run Dijkstra between all pairs of border nodes
    for border_set in border_nodes {
        if border_set.len() < 2 {
            continue;
        }

        for &source in border_set {
            // Run single-source shortest path from this border node
            let distances = compute_border_distances(graph, source, border_set);

            for (&target, &dist) in &distances {
                if source != target {
                    overlay.edges.insert((source, target), dist);
                }
            }
        }
    }

    overlay
}

/// Compute shortest distances from source to all target border nodes
fn compute_border_distances(
    graph: &Graph,
    source: NodeId,
    targets: &[NodeId],
) -> HashMap<NodeId, f64> {
    use priority_queue::PriorityQueue;
    use ordered_float::OrderedFloat;
    use std::cmp::Reverse;

    let mut distances = HashMap::new();
    let mut pq = PriorityQueue::new();
    let mut visited = HashSet::new();

    pq.push(source, Reverse(OrderedFloat(0.0)));
    distances.insert(source, 0.0);

    let target_set: HashSet<_> = targets.iter().copied().collect();
    let mut found = 0;

    while let Some((node, Reverse(OrderedFloat(dist)))) = pq.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node);

        if target_set.contains(&node) {
            found += 1;
            if found == target_set.len() {
                break; // Found all targets
            }
        }

        for &edge_id in graph.outgoing_edges(node) {
            if let Some(edge) = graph.edge(edge_id) {
                let next = edge.target;
                let next_dist = dist + edge.cost.base_time;

                if !visited.contains(&next) {
                    let current_dist = distances.get(&next).copied().unwrap_or(f64::INFINITY);
                    if next_dist < current_dist {
                        distances.insert(next, next_dist);
                        pq.push(next, Reverse(OrderedFloat(next_dist)));
                    }
                }
            }
        }
    }

    distances
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    #[test]
    fn test_grid_partition() {
        let graph = GraphBuilder::create_grid(10, 10, 0.01).unwrap();
        let partition = GraphPartition::create_grid(&graph, 2).unwrap();

        assert_eq!(partition.num_partitions, 4);
        assert!(partition.border_nodes.iter().all(|b| !b.is_empty()));
    }
}
