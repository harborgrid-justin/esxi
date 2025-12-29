//! A* algorithm with geographic heuristic

use crate::api::{RoutingRequest, RoutingResponse, RouteGeometry};
use crate::error::{Result, RoutingError};
use crate::graph::{Graph, NodeId, EdgeId};
use hashbrown::HashMap;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;

/// A* router with haversine distance heuristic
pub struct AStarRouter {
    max_settled: usize,
}

impl AStarRouter {
    /// Create new A* router
    pub fn new() -> Self {
        Self {
            max_settled: 1_000_000,
        }
    }

    /// Compute route using A*
    pub fn compute(
        &self,
        graph: &Graph,
        source: NodeId,
        target: NodeId,
        speed_kmh: f64,
    ) -> Result<AStarResult> {
        let target_node = graph
            .node(target)
            .ok_or_else(|| RoutingError::NodeNotFound(target.0))?;
        let target_location = target_node.location;

        let mut pq: PriorityQueue<NodeId, Reverse<OrderedFloat<f64>>> = PriorityQueue::new();
        let mut g_scores: HashMap<NodeId, f64> = HashMap::new();
        let mut predecessors: HashMap<NodeId, (NodeId, EdgeId)> = HashMap::new();
        let mut settled = 0;

        g_scores.insert(source, 0.0);

        // Calculate initial heuristic
        if let Some(source_node) = graph.node(source) {
            let h = haversine_heuristic(source_node.location, target_location, speed_kmh);
            pq.push(source, Reverse(OrderedFloat(h)));
        }

        while let Some((node, Reverse(OrderedFloat(_f_score)))) = pq.pop() {
            settled += 1;

            if settled > self.max_settled {
                return Err(RoutingError::other("Search space exhausted"));
            }

            if node == target {
                let path = reconstruct_path(source, target, &predecessors);
                let total_cost = g_scores[&target];
                return Ok(AStarResult {
                    cost: total_cost,
                    path,
                    settled_nodes: settled,
                });
            }

            let g_score = g_scores[&node];

            for &edge_id in graph.outgoing_edges(node) {
                if let Some(edge) = graph.edge(edge_id) {
                    let next = edge.target;
                    let tentative_g = g_score + edge.cost.base_time;

                    let current_g = g_scores.get(&next).copied().unwrap_or(f64::INFINITY);

                    if tentative_g < current_g {
                        g_scores.insert(next, tentative_g);
                        predecessors.insert(next, (node, edge_id));

                        // Calculate f = g + h
                        if let Some(next_node) = graph.node(next) {
                            let h = haversine_heuristic(
                                next_node.location,
                                target_location,
                                speed_kmh,
                            );
                            let f = tentative_g + h;
                            pq.push(next, Reverse(OrderedFloat(f)));
                        }
                    }
                }
            }
        }

        Err(RoutingError::NoRouteFound {
            origin: graph.node(source).map(|n| n.location).unwrap_or_default(),
            destination: target_location,
        })
    }
}

impl Default for AStarRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl super::RouteAlgorithm for AStarRouter {
    fn route(&self, request: &RoutingRequest, graph: &Graph) -> Result<RoutingResponse> {
        let source = graph
            .nearest_node(request.origin)
            .ok_or_else(|| RoutingError::InvalidCoordinates("Origin not found".into()))?;

        let target = graph
            .nearest_node(request.destination)
            .ok_or_else(|| RoutingError::InvalidCoordinates("Destination not found".into()))?;

        // Use profile speed for heuristic
        let speed_kmh = request.profile.average_speed_kmh();

        let result = self.compute(graph, source, target, speed_kmh)?;

        Ok(RoutingResponse {
            distance: result.cost,
            duration: result.cost,
            geometry: RouteGeometry::from_edges(&result.path, graph),
            segments: vec![],
            waypoints: vec![request.origin, request.destination],
        })
    }

    fn name(&self) -> &'static str {
        "A*"
    }
}

/// Result from A* computation
#[derive(Debug)]
pub struct AStarResult {
    pub cost: f64,
    pub path: Vec<EdgeId>,
    pub settled_nodes: usize,
}

/// Calculate admissible heuristic using haversine distance
fn haversine_heuristic(
    from: geo_types::Point,
    to: geo_types::Point,
    speed_kmh: f64,
) -> f64 {
    const EARTH_RADIUS: f64 = 6371000.0; // meters

    let lat1 = from.y().to_radians();
    let lat2 = to.y().to_radians();
    let delta_lat = (to.y() - from.y()).to_radians();
    let delta_lon = (to.x() - from.x()).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    let distance_m = EARTH_RADIUS * c;

    // Convert to time (seconds) using speed
    let speed_mps = speed_kmh * 1000.0 / 3600.0;
    distance_m / speed_mps
}

/// Reconstruct path from predecessors
fn reconstruct_path(
    source: NodeId,
    target: NodeId,
    predecessors: &HashMap<NodeId, (NodeId, EdgeId)>,
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
    fn test_astar_simple() {
        let graph = GraphBuilder::create_grid(5, 5, 0.01).unwrap();
        let router = AStarRouter::new();

        let source = NodeId(0);
        let target = NodeId(24);

        let result = router.compute(&graph, source, target, 50.0).unwrap();
        assert!(result.cost > 0.0);
        assert!(!result.path.is_empty());
    }
}
