//! Multimodal isochrone generation

use super::{IsochronePolygon, IsochroneMetadata};
use crate::error::Result;
use crate::graph::{Graph, NodeId};
use geo_types::{Point, MultiPolygon};
use hashbrown::HashMap;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// Multimodal isochrone builder supporting multiple transport modes
pub struct MultimodalIsochroneBuilder<'a> {
    graph: &'a Graph,
}

impl<'a> MultimodalIsochroneBuilder<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }

    /// Build multimodal isochrone with mode transitions
    pub fn build(
        &self,
        origin: Point,
        max_time: f64,
        modes: &[TransportMode],
    ) -> Result<IsochronePolygon> {
        let start_node = self
            .graph
            .nearest_node(origin)
            .ok_or_else(|| crate::error::RoutingError::InvalidCoordinates("Origin not found".into()))?;

        // Multimodal Dijkstra
        let reachable = self.compute_multimodal_reachable(start_node, max_time, modes);

        // Build polygon (simplified)
        let polygon = MultiPolygon(vec![]);

        Ok(IsochronePolygon {
            time_threshold: max_time,
            polygon,
            center: origin,
            nodes_reached: reachable.len(),
            metadata: IsochroneMetadata {
                transport_mode: Some("multimodal".to_string()),
                created_at: Some(chrono::Utc::now()),
                computation_time_ms: None,
            },
        })
    }

    /// Compute reachable nodes with multimodal routing
    fn compute_multimodal_reachable(
        &self,
        start: NodeId,
        max_time: f64,
        modes: &[TransportMode],
    ) -> Vec<NodeId> {
        let mut heap = BinaryHeap::new();
        let mut states: HashMap<(NodeId, TransportMode), f64> = HashMap::new();
        let mut reachable = Vec::new();

        // Initial state
        for &mode in modes {
            heap.push(State {
                node: start,
                cost: 0.0,
                mode,
            });
            states.insert((start, mode), 0.0);
        }

        while let Some(State { node, cost, mode }) = heap.pop() {
            if cost > max_time {
                continue;
            }

            reachable.push(node);

            // Expand neighbors with current mode
            for &edge_id in self.graph.outgoing_edges(node) {
                if let Some(edge) = self.graph.edge(edge_id) {
                    // Check if edge is accessible by current mode
                    if !is_accessible(edge, mode) {
                        continue;
                    }

                    let next = edge.target;
                    let edge_cost = edge.cost.base_time * mode.speed_multiplier();
                    let next_cost = cost + edge_cost;

                    if next_cost <= max_time {
                        let key = (next, mode);
                        let current = states.get(&key).copied().unwrap_or(f64::INFINITY);

                        if next_cost < current {
                            states.insert(key, next_cost);
                            heap.push(State {
                                node: next,
                                cost: next_cost,
                                mode,
                            });
                        }
                    }
                }
            }

            // Mode transitions
            for &next_mode in modes {
                if next_mode != mode {
                    let transition_cost = mode_transition_time(mode, next_mode);
                    let next_cost = cost + transition_cost;

                    if next_cost <= max_time {
                        let key = (node, next_mode);
                        let current = states.get(&key).copied().unwrap_or(f64::INFINITY);

                        if next_cost < current {
                            states.insert(key, next_cost);
                            heap.push(State {
                                node,
                                cost: next_cost,
                                mode: next_mode,
                            });
                        }
                    }
                }
            }
        }

        reachable
    }
}

/// Transport mode for multimodal routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportMode {
    Walking,
    Cycling,
    Driving,
    Transit,
}

impl TransportMode {
    fn speed_multiplier(&self) -> f64 {
        match self {
            TransportMode::Walking => 2.0,    // Slower
            TransportMode::Cycling => 1.5,
            TransportMode::Driving => 1.0,    // Base
            TransportMode::Transit => 1.2,
        }
    }
}

/// State for multimodal search
#[derive(Clone)]
struct State {
    node: NodeId,
    cost: f64,
    mode: TransportMode,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.cost.eq(&other.cost)
    }
}

impl Eq for State {}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

/// Check if edge is accessible by transport mode
fn is_accessible(edge: &crate::graph::Edge, mode: TransportMode) -> bool {
    use crate::graph::VehicleType;

    match mode {
        TransportMode::Walking => edge.accessible_by(VehicleType::Pedestrian),
        TransportMode::Cycling => edge.accessible_by(VehicleType::Bicycle),
        TransportMode::Driving => edge.accessible_by(VehicleType::Car),
        TransportMode::Transit => true, // Simplified
    }
}

/// Get time cost for mode transition (in seconds)
fn mode_transition_time(from: TransportMode, to: TransportMode) -> f64 {
    match (from, to) {
        (TransportMode::Walking, TransportMode::Transit) => 120.0, // 2 min wait
        (TransportMode::Transit, TransportMode::Walking) => 30.0,
        (TransportMode::Driving, TransportMode::Walking) => 60.0,  // Park car
        (TransportMode::Walking, TransportMode::Driving) => 60.0,
        (TransportMode::Cycling, TransportMode::Transit) => 180.0, // Lock bike
        _ => 0.0,
    }
}
