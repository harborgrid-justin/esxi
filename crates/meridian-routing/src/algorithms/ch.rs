//! Contraction Hierarchies algorithm for ultra-fast routing

use crate::api::{RoutingRequest, RoutingResponse, RouteGeometry};
use crate::error::{Result, RoutingError};
use crate::graph::{Graph, NodeId, EdgeId, Edge, EdgeCost};
use hashbrown::HashMap;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;

/// Contraction Hierarchies preprocessed data
pub struct ContractionHierarchies {
    /// Node ordering (lower = contracted earlier)
    node_order: Vec<usize>,

    /// Shortcut edges added during contraction
    shortcuts: Vec<Shortcut>,

    /// Forward search graph
    forward_graph: Vec<Vec<CHEdge>>,

    /// Backward search graph
    backward_graph: Vec<Vec<CHEdge>>,
}

/// Shortcut edge in CH
#[derive(Debug, Clone)]
struct Shortcut {
    source: NodeId,
    target: NodeId,
    cost: f64,
    via: NodeId,
}

/// Edge in CH graph
#[derive(Debug, Clone)]
struct CHEdge {
    target: NodeId,
    cost: f64,
    is_shortcut: bool,
    via: Option<NodeId>,
}

impl ContractionHierarchies {
    /// Preprocess graph to build contraction hierarchies
    pub fn preprocess(graph: &Graph) -> Result<Self> {
        log::info!("Starting CH preprocessing for {} nodes", graph.node_count());

        // Simple ordering strategy: contract nodes by degree (low degree first)
        let node_order = compute_node_ordering(graph);

        let mut shortcuts = Vec::new();
        let mut forward_graph = vec![Vec::new(); graph.node_count()];
        let mut backward_graph = vec![Vec::new(); graph.node_count()];

        // Initialize with original edges
        for node_id in 0..graph.node_count() {
            for &edge_id in graph.outgoing_edges(NodeId(node_id)) {
                if let Some(edge) = graph.edge(edge_id) {
                    forward_graph[node_id].push(CHEdge {
                        target: edge.target,
                        cost: edge.cost.base_time,
                        is_shortcut: false,
                        via: None,
                    });

                    backward_graph[edge.target.0].push(CHEdge {
                        target: NodeId(node_id),
                        cost: edge.cost.base_time,
                        is_shortcut: false,
                        via: None,
                    });
                }
            }
        }

        // Contract nodes in order
        for (order, &node_id) in node_order.iter().enumerate() {
            if order % 10000 == 0 {
                log::debug!("Contracted {}/{} nodes", order, node_order.len());
            }

            contract_node(
                NodeId(node_id),
                &node_order,
                &mut forward_graph,
                &mut backward_graph,
                &mut shortcuts,
            );
        }

        log::info!("CH preprocessing complete: {} shortcuts added", shortcuts.len());

        Ok(Self {
            node_order,
            shortcuts,
            forward_graph,
            backward_graph,
        })
    }

    /// Query route using bidirectional CH search
    pub fn query(&self, source: NodeId, target: NodeId) -> Result<CHQueryResult> {
        // Bidirectional search
        let mut forward_pq: PriorityQueue<NodeId, Reverse<OrderedFloat<f64>>> = PriorityQueue::new();
        let mut backward_pq: PriorityQueue<NodeId, Reverse<OrderedFloat<f64>>> = PriorityQueue::new();

        let mut forward_dist: HashMap<NodeId, f64> = HashMap::new();
        let mut backward_dist: HashMap<NodeId, f64> = HashMap::new();

        let mut forward_pred: HashMap<NodeId, NodeId> = HashMap::new();
        let mut backward_pred: HashMap<NodeId, NodeId> = HashMap::new();

        forward_pq.push(source, Reverse(OrderedFloat(0.0)));
        forward_dist.insert(source, 0.0);

        backward_pq.push(target, Reverse(OrderedFloat(0.0)));
        backward_dist.insert(target, 0.0);

        let mut best_distance = f64::INFINITY;
        let mut meeting_node = None;

        // Alternate between forward and backward search
        loop {
            // Forward step
            if let Some((node, Reverse(OrderedFloat(dist)))) = forward_pq.pop() {
                if dist > best_distance {
                    break; // Can't improve
                }

                // Check if backward search reached this node
                if let Some(&backward_d) = backward_dist.get(&node) {
                    let total = dist + backward_d;
                    if total < best_distance {
                        best_distance = total;
                        meeting_node = Some(node);
                    }
                }

                // Relax upward edges
                for edge in &self.forward_graph[node.0] {
                    if self.node_order[edge.target.0] > self.node_order[node.0] {
                        let new_dist = dist + edge.cost;
                        let current = forward_dist.get(&edge.target).copied().unwrap_or(f64::INFINITY);
                        if new_dist < current {
                            forward_dist.insert(edge.target, new_dist);
                            forward_pred.insert(edge.target, node);
                            forward_pq.push(edge.target, Reverse(OrderedFloat(new_dist)));
                        }
                    }
                }
            }

            // Backward step
            if let Some((node, Reverse(OrderedFloat(dist)))) = backward_pq.pop() {
                if dist > best_distance {
                    break;
                }

                if let Some(&forward_d) = forward_dist.get(&node) {
                    let total = forward_d + dist;
                    if total < best_distance {
                        best_distance = total;
                        meeting_node = Some(node);
                    }
                }

                // Relax upward edges (backward)
                for edge in &self.backward_graph[node.0] {
                    if self.node_order[edge.target.0] > self.node_order[node.0] {
                        let new_dist = dist + edge.cost;
                        let current = backward_dist.get(&edge.target).copied().unwrap_or(f64::INFINITY);
                        if new_dist < current {
                            backward_dist.insert(edge.target, new_dist);
                            backward_pred.insert(edge.target, node);
                            backward_pq.push(edge.target, Reverse(OrderedFloat(new_dist)));
                        }
                    }
                }
            }

            if forward_pq.is_empty() && backward_pq.is_empty() {
                break;
            }
        }

        if let Some(meeting) = meeting_node {
            Ok(CHQueryResult {
                distance: best_distance,
                meeting_node: meeting,
            })
        } else {
            Err(RoutingError::other("No route found"))
        }
    }
}

impl super::RouteAlgorithm for ContractionHierarchies {
    fn route(&self, request: &RoutingRequest, graph: &Graph) -> Result<RoutingResponse> {
        let source = graph
            .nearest_node(request.origin)
            .ok_or_else(|| RoutingError::InvalidCoordinates("Origin not found".into()))?;

        let target = graph
            .nearest_node(request.destination)
            .ok_or_else(|| RoutingError::InvalidCoordinates("Destination not found".into()))?;

        let result = self.query(source, target)?;

        Ok(RoutingResponse {
            distance: result.distance,
            duration: result.distance,
            geometry: RouteGeometry::default(),
            segments: vec![],
            waypoints: vec![request.origin, request.destination],
        })
    }

    fn name(&self) -> &'static str {
        "ContractionHierarchies"
    }
}

/// Result from CH query
pub struct CHQueryResult {
    pub distance: f64,
    pub meeting_node: NodeId,
}

/// Compute node ordering for contraction
fn compute_node_ordering(graph: &Graph) -> Vec<usize> {
    // Simple strategy: order by degree (low degree first)
    let mut nodes_with_degree: Vec<_> = (0..graph.node_count())
        .map(|i| {
            let degree = graph.outgoing_edges(NodeId(i)).len() + graph.incoming_edges(NodeId(i)).len();
            (i, degree)
        })
        .collect();

    nodes_with_degree.sort_by_key(|(_, degree)| *degree);

    let mut order = vec![0; graph.node_count()];
    for (rank, (node_id, _)) in nodes_with_degree.iter().enumerate() {
        order[*node_id] = rank;
    }

    nodes_with_degree.into_iter().map(|(id, _)| id).collect()
}

/// Contract a single node
fn contract_node(
    node: NodeId,
    order: &[usize],
    forward_graph: &mut [Vec<CHEdge>],
    backward_graph: &mut [Vec<CHEdge>],
    shortcuts: &mut Vec<Shortcut>,
) {
    // Find incoming and outgoing edges
    let incoming: Vec<_> = backward_graph[node.0].clone();
    let outgoing: Vec<_> = forward_graph[node.0].clone();

    // For each incoming-outgoing pair, check if shortcut is needed
    for in_edge in &incoming {
        for out_edge in &outgoing {
            let shortcut_cost = in_edge.cost + out_edge.cost;

            // Check if there's a witness path (without using this node)
            let has_witness = has_witness_path(
                in_edge.target,
                out_edge.target,
                shortcut_cost,
                node,
                order,
                forward_graph,
            );

            if !has_witness {
                // Add shortcut
                forward_graph[in_edge.target.0].push(CHEdge {
                    target: out_edge.target,
                    cost: shortcut_cost,
                    is_shortcut: true,
                    via: Some(node),
                });

                backward_graph[out_edge.target.0].push(CHEdge {
                    target: in_edge.target,
                    cost: shortcut_cost,
                    is_shortcut: true,
                    via: Some(node),
                });

                shortcuts.push(Shortcut {
                    source: in_edge.target,
                    target: out_edge.target,
                    cost: shortcut_cost,
                    via: node,
                });
            }
        }
    }
}

/// Check if witness path exists (simple local search)
fn has_witness_path(
    source: NodeId,
    target: NodeId,
    max_cost: f64,
    excluded: NodeId,
    order: &[usize],
    graph: &[Vec<CHEdge>],
) -> bool {
    if source == target {
        return true;
    }

    let mut pq: PriorityQueue<NodeId, Reverse<OrderedFloat<f64>>> = PriorityQueue::new();
    let mut dist: HashMap<NodeId, f64> = HashMap::new();

    pq.push(source, Reverse(OrderedFloat(0.0)));
    dist.insert(source, 0.0);

    let max_hops = 5; // Limit search depth
    let mut hops: HashMap<NodeId, usize> = HashMap::new();
    hops.insert(source, 0);

    while let Some((node, Reverse(OrderedFloat(d)))) = pq.pop() {
        if node == target {
            return d <= max_cost;
        }

        if d > max_cost {
            continue;
        }

        let current_hops = hops[&node];
        if current_hops >= max_hops {
            continue;
        }

        for edge in &graph[node.0] {
            if edge.target == excluded {
                continue;
            }

            let new_dist = d + edge.cost;
            if new_dist <= max_cost {
                let current = dist.get(&edge.target).copied().unwrap_or(f64::INFINITY);
                if new_dist < current {
                    dist.insert(edge.target, new_dist);
                    hops.insert(edge.target, current_hops + 1);
                    pq.push(edge.target, Reverse(OrderedFloat(new_dist)));
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    #[test]
    fn test_ch_preprocessing() {
        let graph = GraphBuilder::create_grid(5, 5, 0.01).unwrap();
        let ch = ContractionHierarchies::preprocess(&graph).unwrap();

        assert_eq!(ch.node_order.len(), 25);
    }

    #[test]
    fn test_ch_query() {
        let graph = GraphBuilder::create_grid(5, 5, 0.01).unwrap();
        let ch = ContractionHierarchies::preprocess(&graph).unwrap();

        let result = ch.query(NodeId(0), NodeId(24)).unwrap();
        assert!(result.distance > 0.0);
    }
}
