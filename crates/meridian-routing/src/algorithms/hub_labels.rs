//! Hub labeling algorithm for constant-time distance queries

use crate::error::Result;
use crate::graph::{Graph, NodeId};
use hashbrown::{HashMap, HashSet};
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;

/// Hub labeling preprocessed data
pub struct HubLabeling {
    /// Forward labels: for each node, set of (hub, distance) pairs
    forward_labels: Vec<Vec<HubLabel>>,

    /// Backward labels: for each node, set of (hub, distance) pairs
    backward_labels: Vec<Vec<HubLabel>>,

    /// Node importance ordering
    importance: Vec<usize>,
}

/// Hub label entry
#[derive(Debug, Clone)]
pub struct HubLabel {
    pub hub: NodeId,
    pub distance: f64,
}

impl HubLabeling {
    /// Preprocess graph to build hub labels
    pub fn preprocess(graph: &Graph) -> Result<Self> {
        log::info!("Starting hub labeling preprocessing");

        let n = graph.node_count();
        let importance = compute_importance(graph);

        let mut forward_labels = vec![Vec::new(); n];
        let mut backward_labels = vec![Vec::new(); n];

        // Process nodes in order of importance (high to low)
        let mut nodes_by_importance: Vec<_> = (0..n).collect();
        nodes_by_importance.sort_by_key(|&i| std::cmp::Reverse(importance[i]));

        for (rank, &node_id) in nodes_by_importance.iter().enumerate() {
            if rank % 1000 == 0 {
                log::debug!("Processed {}/{} nodes", rank, n);
            }

            // Forward pruned Dijkstra
            let forward = pruned_dijkstra(
                graph,
                NodeId(node_id),
                true,
                &forward_labels,
                &backward_labels,
            );

            for (target, dist) in forward {
                forward_labels[node_id].push(HubLabel {
                    hub: target,
                    distance: dist,
                });
                backward_labels[target.0].push(HubLabel {
                    hub: NodeId(node_id),
                    distance: dist,
                });
            }
        }

        // Sort labels for binary search
        for labels in &mut forward_labels {
            labels.sort_by_key(|l| l.hub.0);
        }
        for labels in &mut backward_labels {
            labels.sort_by_key(|l| l.hub.0);
        }

        log::info!("Hub labeling preprocessing complete");

        Ok(Self {
            forward_labels,
            backward_labels,
            importance,
        })
    }

    /// Query distance between two nodes in O(k) time where k is label size
    pub fn query(&self, source: NodeId, target: NodeId) -> Option<f64> {
        let forward = &self.forward_labels[source.0];
        let backward = &self.backward_labels[target.0];

        let mut best_distance = f64::INFINITY;

        // Find common hubs (intersection)
        let mut i = 0;
        let mut j = 0;

        while i < forward.len() && j < backward.len() {
            match forward[i].hub.0.cmp(&backward[j].hub.0) {
                std::cmp::Ordering::Equal => {
                    let dist = forward[i].distance + backward[j].distance;
                    best_distance = best_distance.min(dist);
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
            }
        }

        if best_distance < f64::INFINITY {
            Some(best_distance)
        } else {
            None
        }
    }

    /// Get label statistics
    pub fn stats(&self) -> HubLabelingStats {
        let total_labels: usize = self.forward_labels.iter().map(|l| l.len()).sum();
        let max_label_size = self.forward_labels.iter().map(|l| l.len()).max().unwrap_or(0);
        let avg_label_size = total_labels as f64 / self.forward_labels.len() as f64;

        HubLabelingStats {
            total_labels,
            max_label_size,
            avg_label_size,
        }
    }
}

/// Statistics about hub labeling
#[derive(Debug)]
pub struct HubLabelingStats {
    pub total_labels: usize,
    pub max_label_size: usize,
    pub avg_label_size: f64,
}

/// Compute node importance (simple degree-based)
fn compute_importance(graph: &Graph) -> Vec<usize> {
    (0..graph.node_count())
        .map(|i| {
            graph.outgoing_edges(NodeId(i)).len() + graph.incoming_edges(NodeId(i)).len()
        })
        .collect()
}

/// Pruned Dijkstra for hub labeling
fn pruned_dijkstra(
    graph: &Graph,
    source: NodeId,
    forward: bool,
    forward_labels: &[Vec<HubLabel>],
    backward_labels: &[Vec<HubLabel>],
) -> HashMap<NodeId, f64> {
    let mut pq: PriorityQueue<NodeId, Reverse<OrderedFloat<f64>>> = PriorityQueue::new();
    let mut distances: HashMap<NodeId, f64> = HashMap::new();
    let mut visited = HashSet::new();

    pq.push(source, Reverse(OrderedFloat(0.0)));
    distances.insert(source, 0.0);

    while let Some((node, Reverse(OrderedFloat(dist)))) = pq.pop() {
        if visited.contains(&node) {
            continue;
        }
        visited.insert(node);

        // Pruning: check if current path can be improved by existing labels
        if can_prune(source, node, dist, forward_labels, backward_labels) {
            continue;
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

/// Check if path can be pruned
fn can_prune(
    source: NodeId,
    target: NodeId,
    current_dist: f64,
    forward_labels: &[Vec<HubLabel>],
    backward_labels: &[Vec<HubLabel>],
) -> bool {
    let forward = &forward_labels[source.0];
    let backward = &backward_labels[target.0];

    // Check if there's a shorter path through existing labels
    for f_label in forward {
        for b_label in backward {
            if f_label.hub == b_label.hub {
                let label_dist = f_label.distance + b_label.distance;
                if label_dist <= current_dist {
                    return true;
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
    fn test_hub_labeling() {
        let graph = GraphBuilder::create_grid(5, 5, 0.01).unwrap();
        let hl = HubLabeling::preprocess(&graph).unwrap();

        let dist = hl.query(NodeId(0), NodeId(24));
        assert!(dist.is_some());
        assert!(dist.unwrap() > 0.0);

        let stats = hl.stats();
        assert!(stats.total_labels > 0);
    }
}
