//! ALT (A*, Landmarks, Triangle inequality) algorithm

use crate::error::Result;
use crate::graph::{Graph, NodeId};
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
use hashbrown::HashMap;

/// ALT preprocessed data
pub struct ALTPreprocessor {
    /// Landmark nodes
    landmarks: Vec<NodeId>,

    /// Distance from each landmark to all nodes
    forward_distances: Vec<Vec<f64>>,

    /// Distance from all nodes to each landmark
    backward_distances: Vec<Vec<f64>>,
}

impl ALTPreprocessor {
    /// Preprocess graph with landmarks
    pub fn preprocess(graph: &Graph, num_landmarks: usize) -> Result<Self> {
        log::info!("Starting ALT preprocessing with {} landmarks", num_landmarks);

        // Select landmarks (using farthest point sampling)
        let landmarks = select_landmarks(graph, num_landmarks);

        let n = graph.node_count();
        let mut forward_distances = Vec::new();
        let mut backward_distances = Vec::new();

        // Compute distances from/to each landmark
        for (i, &landmark) in landmarks.iter().enumerate() {
            log::debug!("Processing landmark {}/{}", i + 1, num_landmarks);

            // Forward: landmark -> all nodes
            let forward = dijkstra_single_source(graph, landmark, true);
            let mut forward_dist = vec![f64::INFINITY; n];
            for (node, dist) in forward {
                forward_dist[node.0] = dist;
            }
            forward_distances.push(forward_dist);

            // Backward: all nodes -> landmark
            let backward = dijkstra_single_source(graph, landmark, false);
            let mut backward_dist = vec![f64::INFINITY; n];
            for (node, dist) in backward {
                backward_dist[node.0] = dist;
            }
            backward_distances.push(backward_dist);
        }

        log::info!("ALT preprocessing complete");

        Ok(Self {
            landmarks,
            forward_distances,
            backward_distances,
        })
    }

    /// Compute ALT heuristic for A*
    pub fn heuristic(&self, node: NodeId, target: NodeId) -> f64 {
        let mut h = 0.0;

        // Use triangle inequality with all landmarks
        for i in 0..self.landmarks.len() {
            // Lower bound: |d(node, landmark) - d(target, landmark)|
            let d1 = (self.forward_distances[i][node.0] - self.forward_distances[i][target.0]).abs();
            let d2 = (self.backward_distances[i][node.0] - self.backward_distances[i][target.0]).abs();

            h = h.max(d1).max(d2);
        }

        h
    }

    /// Route using ALT-enhanced A*
    pub fn route(&self, graph: &Graph, source: NodeId, target: NodeId) -> Result<ALTResult> {
        let mut pq: PriorityQueue<NodeId, Reverse<OrderedFloat<f64>>> = PriorityQueue::new();
        let mut g_scores: HashMap<NodeId, f64> = HashMap::new();
        let mut settled = 0;

        g_scores.insert(source, 0.0);
        let h = self.heuristic(source, target);
        pq.push(source, Reverse(OrderedFloat(h)));

        while let Some((node, Reverse(OrderedFloat(_f_score)))) = pq.pop() {
            settled += 1;

            if node == target {
                return Ok(ALTResult {
                    distance: g_scores[&target],
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
                        let h = self.heuristic(next, target);
                        let f = tentative_g + h;
                        pq.push(next, Reverse(OrderedFloat(f)));
                    }
                }
            }
        }

        Err(crate::error::RoutingError::other("No route found"))
    }

    /// Get landmarks
    pub fn landmarks(&self) -> &[NodeId] {
        &self.landmarks
    }
}

/// Result from ALT routing
pub struct ALTResult {
    pub distance: f64,
    pub settled_nodes: usize,
}

/// Select landmarks using farthest point sampling
fn select_landmarks(graph: &Graph, k: usize) -> Vec<NodeId> {
    if graph.node_count() == 0 {
        return Vec::new();
    }

    let mut landmarks = Vec::new();
    let mut max_distances = vec![0.0; graph.node_count()];

    // Start with a random node (node 0)
    landmarks.push(NodeId(0));

    // Iteratively select farthest node
    for _ in 1..k {
        // Update distances from last landmark
        let last_landmark = *landmarks.last().unwrap();
        let distances = dijkstra_single_source(graph, last_landmark, true);

        for (node, dist) in distances {
            max_distances[node.0] = max_distances[node.0].max(dist);
        }

        // Find node with maximum distance
        let (farthest, _) = max_distances
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        landmarks.push(NodeId(farthest));
    }

    landmarks
}

/// Single-source Dijkstra (forward or backward)
fn dijkstra_single_source(
    graph: &Graph,
    source: NodeId,
    forward: bool,
) -> HashMap<NodeId, f64> {
    let mut pq: PriorityQueue<NodeId, Reverse<OrderedFloat<f64>>> = PriorityQueue::new();
    let mut distances: HashMap<NodeId, f64> = HashMap::new();

    pq.push(source, Reverse(OrderedFloat(0.0)));
    distances.insert(source, 0.0);

    while let Some((node, Reverse(OrderedFloat(dist)))) = pq.pop() {
        if let Some(&best_dist) = distances.get(&node) {
            if dist > best_dist {
                continue;
            }
        }

        let edges = if forward {
            graph.outgoing_edges(node)
        } else {
            graph.incoming_edges(node)
        };

        for &edge_id in edges {
            if let Some(edge) = graph.edge(edge_id) {
                let next = if forward { edge.target } else { edge.source };
                let next_dist = dist + edge.cost.base_time;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    #[test]
    fn test_alt() {
        let graph = GraphBuilder::create_grid(10, 10, 0.01).unwrap();
        let alt = ALTPreprocessor::preprocess(&graph, 4).unwrap();

        assert_eq!(alt.landmarks().len(), 4);

        let result = alt.route(&graph, NodeId(0), NodeId(99)).unwrap();
        assert!(result.distance > 0.0);
    }
}
