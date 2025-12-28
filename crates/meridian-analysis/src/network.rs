//! Network analysis operations including shortest path and routing

use crate::error::{AnalysisError, Result};
use geo::{EuclideanDistance, LineString, Point};
use petgraph::algo::{astar, dijkstra};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::Direction;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A network edge with cost/weight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEdge {
    pub from: usize,
    pub to: usize,
    pub cost: f64,
    pub geometry: Option<LineString>,
    pub one_way: bool,
}

/// A network node with location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    pub id: usize,
    pub location: Point,
    pub attributes: HashMap<String, String>,
}

/// Network graph structure
#[derive(Debug, Clone)]
pub struct Network {
    graph: Graph<NetworkNode, f64>,
    node_map: HashMap<usize, NodeIndex>,
}

impl Network {
    /// Create a new empty network
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Add a node to the network
    pub fn add_node(&mut self, node: NetworkNode) -> NodeIndex {
        let idx = self.graph.add_node(node.clone());
        self.node_map.insert(node.id, idx);
        idx
    }

    /// Add an edge to the network
    pub fn add_edge(&mut self, edge: NetworkEdge) -> Result<()> {
        let from_idx = self
            .node_map
            .get(&edge.from)
            .ok_or_else(|| AnalysisError::network_error("From node not found"))?;
        let to_idx = self
            .node_map
            .get(&edge.to)
            .ok_or_else(|| AnalysisError::network_error("To node not found"))?;

        self.graph.add_edge(*from_idx, *to_idx, edge.cost);

        if !edge.one_way {
            self.graph.add_edge(*to_idx, *from_idx, edge.cost);
        }

        Ok(())
    }

    /// Get node index from ID
    pub fn get_node_index(&self, id: usize) -> Result<NodeIndex> {
        self.node_map
            .get(&id)
            .copied()
            .ok_or_else(|| AnalysisError::network_error("Node not found"))
    }

    /// Get number of nodes
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get number of edges
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a shortest path query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortestPath {
    pub nodes: Vec<usize>,
    pub total_cost: f64,
    pub geometry: Option<LineString>,
}

/// Find shortest path using Dijkstra's algorithm
pub fn shortest_path_dijkstra(
    network: &Network,
    start_id: usize,
    end_id: usize,
) -> Result<ShortestPath> {
    let start_idx = network.get_node_index(start_id)?;
    let end_idx = network.get_node_index(end_id)?;

    let result = dijkstra(&network.graph, start_idx, Some(end_idx), |e| *e.weight());

    let total_cost = result
        .get(&end_idx)
        .copied()
        .ok_or_else(|| AnalysisError::network_error("No path found"))?;

    // Reconstruct path
    let path = reconstruct_path(network, start_idx, end_idx, &result)?;

    Ok(ShortestPath {
        nodes: path,
        total_cost,
        geometry: None,
    })
}

/// Find shortest path using A* algorithm
pub fn shortest_path_astar(
    network: &Network,
    start_id: usize,
    end_id: usize,
) -> Result<ShortestPath> {
    let start_idx = network.get_node_index(start_id)?;
    let end_idx = network.get_node_index(end_id)?;

    let goal_node = &network.graph[end_idx];
    let goal_point = &goal_node.location;

    let result = astar(
        &network.graph,
        start_idx,
        |idx| idx == end_idx,
        |e| *e.weight(),
        |idx| {
            let node = &network.graph[idx];
            node.location.euclidean_distance(goal_point)
        },
    );

    let (total_cost, path_indices) =
        result.ok_or_else(|| AnalysisError::network_error("No path found"))?;

    let nodes: Vec<usize> = path_indices
        .iter()
        .map(|idx| network.graph[*idx].id)
        .collect();

    Ok(ShortestPath {
        nodes,
        total_cost,
        geometry: None,
    })
}

/// Reconstruct path from Dijkstra result
fn reconstruct_path(
    network: &Network,
    start: NodeIndex,
    end: NodeIndex,
    distances: &HashMap<NodeIndex, f64>,
) -> Result<Vec<usize>> {
    let mut path = vec![end];
    let mut current = end;

    while current != start {
        let mut found = false;

        for neighbor in network.graph.neighbors_directed(current, Direction::Incoming) {
            if let (Some(&current_dist), Some(edge)) = (
                distances.get(&current),
                network.graph.find_edge(neighbor, current),
            ) {
                let edge_weight = network.graph[edge];
                let neighbor_dist = distances.get(&neighbor).copied().unwrap_or(f64::INFINITY);

                if (neighbor_dist + edge_weight - current_dist).abs() < 1e-10 {
                    path.push(neighbor);
                    current = neighbor;
                    found = true;
                    break;
                }
            }
        }

        if !found {
            return Err(AnalysisError::network_error("Failed to reconstruct path"));
        }
    }

    path.reverse();
    Ok(path.iter().map(|idx| network.graph[*idx].id).collect())
}

/// Calculate service area (isochrone) from a starting point
pub fn service_area(
    network: &Network,
    start_id: usize,
    max_cost: f64,
) -> Result<Vec<(usize, f64)>> {
    let start_idx = network.get_node_index(start_id)?;

    let distances = dijkstra(&network.graph, start_idx, None, |e| *e.weight());

    let reachable: Vec<(usize, f64)> = distances
        .iter()
        .filter(|(_, &cost)| cost <= max_cost)
        .map(|(idx, &cost)| (network.graph[*idx].id, cost))
        .collect();

    Ok(reachable)
}

/// Calculate multiple service areas
pub fn service_areas(
    network: &Network,
    start_id: usize,
    cost_breaks: &[f64],
) -> Result<Vec<Vec<(usize, f64)>>> {
    let start_idx = network.get_node_index(start_id)?;

    let distances = dijkstra(&network.graph, start_idx, None, |e| *e.weight());

    let mut sorted_breaks = cost_breaks.to_vec();
    sorted_breaks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut results = Vec::new();

    for &max_cost in &sorted_breaks {
        let reachable: Vec<(usize, f64)> = distances
            .iter()
            .filter(|(_, &cost)| cost <= max_cost)
            .map(|(idx, &cost)| (network.graph[*idx].id, cost))
            .collect();
        results.push(reachable);
    }

    Ok(results)
}

/// Find all shortest paths from one node to all others
pub fn shortest_paths_from_node(
    network: &Network,
    start_id: usize,
) -> Result<HashMap<usize, f64>> {
    let start_idx = network.get_node_index(start_id)?;

    let distances = dijkstra(&network.graph, start_idx, None, |e| *e.weight());

    Ok(distances
        .iter()
        .map(|(idx, &cost)| (network.graph[*idx].id, cost))
        .collect())
}

/// Calculate origin-destination cost matrix
pub fn od_cost_matrix(network: &Network, origins: &[usize], destinations: &[usize]) -> Result<Vec<Vec<f64>>> {
    let matrix: Vec<Vec<f64>> = origins
        .par_iter()
        .map(|&origin_id| {
            let start_idx = network.get_node_index(origin_id).unwrap();
            let distances = dijkstra(&network.graph, start_idx, None, |e| *e.weight());

            destinations
                .iter()
                .map(|&dest_id| {
                    let dest_idx = network.get_node_index(dest_id).unwrap();
                    distances.get(&dest_idx).copied().unwrap_or(f64::INFINITY)
                })
                .collect()
        })
        .collect();

    Ok(matrix)
}

/// Find closest facility
pub fn closest_facility(
    network: &Network,
    incident_id: usize,
    facility_ids: &[usize],
) -> Result<(usize, f64)> {
    if facility_ids.is_empty() {
        return Err(AnalysisError::InsufficientData {
            required: 1,
            actual: 0,
        });
    }

    let start_idx = network.get_node_index(incident_id)?;
    let distances = dijkstra(&network.graph, start_idx, None, |e| *e.weight());

    let (facility_idx, &min_cost) = facility_ids
        .iter()
        .filter_map(|&fid| {
            let idx = network.get_node_index(fid).ok()?;
            distances.get(&idx).map(|cost| (idx, cost))
        })
        .min_by(|(_, c1), (_, c2)| c1.partial_cmp(c2).unwrap_or(std::cmp::Ordering::Equal))
        .ok_or_else(|| AnalysisError::network_error("No reachable facility found"))?;

    Ok((network.graph[facility_idx].id, min_cost))
}

/// Trace network upstream or downstream from a point
pub fn network_trace(
    network: &Network,
    start_id: usize,
    direction: Direction,
    max_depth: Option<usize>,
) -> Result<Vec<usize>> {
    let start_idx = network.get_node_index(start_id)?;

    let mut visited = HashSet::new();
    let mut result = Vec::new();
    let mut queue = vec![(start_idx, 0)];

    while let Some((current, depth)) = queue.pop() {
        if visited.contains(&current) {
            continue;
        }

        if let Some(max) = max_depth {
            if depth > max {
                continue;
            }
        }

        visited.insert(current);
        result.push(network.graph[current].id);

        for neighbor in network.graph.neighbors_directed(current, direction) {
            if !visited.contains(&neighbor) {
                queue.push((neighbor, depth + 1));
            }
        }
    }

    Ok(result)
}

/// Calculate network connectivity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub average_degree: f64,
    pub is_connected: bool,
}

pub fn connectivity_metrics(network: &Network) -> ConnectivityMetrics {
    let node_count = network.node_count();
    let edge_count = network.edge_count();

    let total_degree: usize = (0..network.graph.node_count())
        .map(|i| {
            network
                .graph
                .neighbors(NodeIndex::new(i))
                .count()
        })
        .sum();

    let average_degree = if node_count > 0 {
        total_degree as f64 / node_count as f64
    } else {
        0.0
    };

    // Simple connectivity check (simplified)
    let is_connected = if node_count > 0 {
        let start = NodeIndex::new(0);
        let distances = dijkstra(&network.graph, start, None, |_| 1.0);
        distances.len() == node_count
    } else {
        true
    };

    ConnectivityMetrics {
        node_count,
        edge_count,
        average_degree,
        is_connected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_network() -> Network {
        let mut network = Network::new();

        // Add nodes
        for i in 0..5 {
            network.add_node(NetworkNode {
                id: i,
                location: Point::new(i as f64, 0.0),
                attributes: HashMap::new(),
            });
        }

        // Add edges
        network
            .add_edge(NetworkEdge {
                from: 0,
                to: 1,
                cost: 1.0,
                geometry: None,
                one_way: false,
            })
            .unwrap();

        network
            .add_edge(NetworkEdge {
                from: 1,
                to: 2,
                cost: 1.0,
                geometry: None,
                one_way: false,
            })
            .unwrap();

        network
            .add_edge(NetworkEdge {
                from: 2,
                to: 3,
                cost: 1.0,
                geometry: None,
                one_way: false,
            })
            .unwrap();

        network
            .add_edge(NetworkEdge {
                from: 0,
                to: 4,
                cost: 2.0,
                geometry: None,
                one_way: false,
            })
            .unwrap();

        network
            .add_edge(NetworkEdge {
                from: 4,
                to: 3,
                cost: 2.0,
                geometry: None,
                one_way: false,
            })
            .unwrap();

        network
    }

    #[test]
    fn test_network_creation() {
        let network = create_test_network();
        assert_eq!(network.node_count(), 5);
        assert!(network.edge_count() > 0);
    }

    #[test]
    fn test_shortest_path_dijkstra() {
        let network = create_test_network();
        let path = shortest_path_dijkstra(&network, 0, 3).unwrap();
        assert!(path.total_cost > 0.0);
        assert!(!path.nodes.is_empty());
    }

    #[test]
    fn test_shortest_path_astar() {
        let network = create_test_network();
        let path = shortest_path_astar(&network, 0, 3).unwrap();
        assert!(path.total_cost > 0.0);
        assert!(!path.nodes.is_empty());
    }

    #[test]
    fn test_service_area() {
        let network = create_test_network();
        let area = service_area(&network, 0, 2.5).unwrap();
        assert!(!area.is_empty());
    }

    #[test]
    fn test_connectivity_metrics() {
        let network = create_test_network();
        let metrics = connectivity_metrics(&network);
        assert_eq!(metrics.node_count, 5);
        assert!(metrics.average_degree > 0.0);
    }
}
