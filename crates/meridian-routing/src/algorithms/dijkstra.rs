//! Dijkstra's shortest path algorithm

use crate::api::{RoutingRequest, RoutingResponse, RouteSegment, RouteGeometry};
use crate::error::{Result, RoutingError};
use crate::graph::{Graph, NodeId, EdgeId};
use hashbrown::HashMap;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;

/// Dijkstra router implementation
pub struct DijkstraRouter {
    /// Maximum number of settled nodes before giving up
    max_settled: usize,
}

impl DijkstraRouter {
    /// Create new Dijkstra router
    pub fn new() -> Self {
        Self {
            max_settled: 1_000_000,
        }
    }

    /// Create with custom settled limit
    pub fn with_max_settled(max_settled: usize) -> Self {
        Self { max_settled }
    }

    /// Run Dijkstra from source to target
    pub fn compute(
        &self,
        graph: &Graph,
        source: NodeId,
        target: NodeId,
        time_of_day: Option<f64>,
    ) -> Result<DijkstraResult> {
        let mut pq: PriorityQueue<NodeId, Reverse<OrderedFloat<f64>>> = PriorityQueue::new();
        let mut distances: HashMap<NodeId, f64> = HashMap::new();
        let mut predecessors: HashMap<NodeId, (NodeId, EdgeId)> = HashMap::new();
        let mut settled = 0;

        pq.push(source, Reverse(OrderedFloat(0.0)));
        distances.insert(source, 0.0);

        while let Some((node, Reverse(OrderedFloat(dist)))) = pq.pop() {
            settled += 1;

            if settled > self.max_settled {
                return Err(RoutingError::other("Search space exhausted"));
            }

            // Found target
            if node == target {
                let path = reconstruct_path(source, target, &predecessors, graph);
                return Ok(DijkstraResult {
                    distance: dist,
                    path,
                    settled_nodes: settled,
                });
            }

            // Skip if we've already found a better path
            if let Some(&best_dist) = distances.get(&node) {
                if dist > best_dist {
                    continue;
                }
            }

            // Explore neighbors
            for &edge_id in graph.outgoing_edges(node) {
                if let Some(edge) = graph.edge(edge_id) {
                    let next = edge.target;
                    let edge_cost = edge.travel_time(time_of_day);
                    let next_dist = dist + edge_cost;

                    let current_dist = distances.get(&next).copied().unwrap_or(f64::INFINITY);
                    if next_dist < current_dist {
                        distances.insert(next, next_dist);
                        predecessors.insert(next, (node, edge_id));
                        pq.push(next, Reverse(OrderedFloat(next_dist)));
                    }
                }
            }
        }

        Err(RoutingError::NoRouteFound {
            origin: graph.node(source).map(|n| n.location).unwrap_or_default(),
            destination: graph.node(target).map(|n| n.location).unwrap_or_default(),
        })
    }

    /// Compute distance table from source to multiple targets
    pub fn compute_many_to_one(
        &self,
        graph: &Graph,
        source: NodeId,
        targets: &[NodeId],
    ) -> HashMap<NodeId, f64> {
        let mut pq: PriorityQueue<NodeId, Reverse<OrderedFloat<f64>>> = PriorityQueue::new();
        let mut distances: HashMap<NodeId, f64> = HashMap::new();
        let mut found_targets = 0;

        pq.push(source, Reverse(OrderedFloat(0.0)));
        distances.insert(source, 0.0);

        let target_set: hashbrown::HashSet<_> = targets.iter().copied().collect();

        while let Some((node, Reverse(OrderedFloat(dist)))) = pq.pop() {
            if target_set.contains(&node) {
                found_targets += 1;
                if found_targets == targets.len() {
                    break; // Found all targets
                }
            }

            if let Some(&best_dist) = distances.get(&node) {
                if dist > best_dist {
                    continue;
                }
            }

            for &edge_id in graph.outgoing_edges(node) {
                if let Some(edge) = graph.edge(edge_id) {
                    let next = edge.target;
                    let edge_cost = edge.cost.base_time;
                    let next_dist = dist + edge_cost;

                    let current_dist = distances.get(&next).copied().unwrap_or(f64::INFINITY);
                    if next_dist < current_dist {
                        distances.insert(next, next_dist);
                        pq.push(next, Reverse(OrderedFloat(next_dist)));
                    }
                }
            }
        }

        distances
    }
}

impl Default for DijkstraRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl super::RouteAlgorithm for DijkstraRouter {
    fn route(&self, request: &RoutingRequest, graph: &Graph) -> Result<RoutingResponse> {
        let source = graph
            .nearest_node(request.origin)
            .ok_or_else(|| RoutingError::InvalidCoordinates("Origin not found".into()))?;

        let target = graph
            .nearest_node(request.destination)
            .ok_or_else(|| RoutingError::InvalidCoordinates("Destination not found".into()))?;

        let result = self.compute(graph, source, target, request.departure_time)?;

        Ok(RoutingResponse {
            distance: result.distance,
            duration: result.distance, // Using time as both for now
            geometry: RouteGeometry::from_edges(&result.path, graph),
            segments: vec![], // Could build detailed segments
            waypoints: vec![request.origin, request.destination],
        })
    }

    fn name(&self) -> &'static str {
        "Dijkstra"
    }
}

/// Result from Dijkstra computation
#[derive(Debug)]
pub struct DijkstraResult {
    pub distance: f64,
    pub path: Vec<EdgeId>,
    pub settled_nodes: usize,
}

/// Reconstruct path from predecessor map
fn reconstruct_path(
    source: NodeId,
    target: NodeId,
    predecessors: &HashMap<NodeId, (NodeId, EdgeId)>,
    graph: &Graph,
) -> Vec<EdgeId> {
    let mut path = Vec::new();
    let mut current = target;

    while current != source {
        if let Some(&(prev_node, edge_id)) = predecessors.get(&current) {
            path.push(edge_id);
            current = prev_node;
        } else {
            break;
        }
    }

    path.reverse();
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    #[test]
    fn test_dijkstra_simple() {
        let graph = GraphBuilder::create_grid(5, 5, 0.01).unwrap();
        let router = DijkstraRouter::new();

        let source = NodeId(0);
        let target = NodeId(24);

        let result = router.compute(&graph, source, target, None).unwrap();
        assert!(result.distance > 0.0);
        assert!(!result.path.is_empty());
    }
}
