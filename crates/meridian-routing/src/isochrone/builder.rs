//! Isochrone polygon builder

use super::{IsochronePolygon, IsochroneMetadata};
use crate::error::Result;
use crate::graph::{Graph, NodeId};
use crate::profile::RoutingProfile;
use geo_types::{Point, Coord, Polygon, MultiPolygon};
use hashbrown::HashMap;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;

/// Builder for isochrone polygons
pub struct IsochroneBuilder<'a> {
    graph: &'a Graph,
}

impl<'a> IsochroneBuilder<'a> {
    /// Create new isochrone builder
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }

    /// Build isochrone polygon
    pub fn build(
        &self,
        origin: Point,
        max_time: f64,
        profile: &RoutingProfile,
    ) -> Result<IsochronePolygon> {
        let start_time = std::time::Instant::now();

        // Find start node
        let start_node = self
            .graph
            .nearest_node(origin)
            .ok_or_else(|| crate::error::RoutingError::InvalidCoordinates("Origin not found".into()))?;

        // Compute reachable nodes
        let reachable = self.compute_reachable(start_node, max_time);

        // Build polygon from reachable nodes
        let polygon = self.build_polygon(&reachable);

        let computation_time = start_time.elapsed().as_millis() as u64;

        Ok(IsochronePolygon {
            time_threshold: max_time,
            polygon,
            center: origin,
            nodes_reached: reachable.len(),
            metadata: IsochroneMetadata {
                transport_mode: Some(profile.name().to_string()),
                created_at: Some(chrono::Utc::now()),
                computation_time_ms: Some(computation_time),
            },
        })
    }

    /// Build multiple isochrone contours
    pub fn build_contours(
        &self,
        origin: Point,
        time_thresholds: &[f64],
        profile: &RoutingProfile,
    ) -> Result<Vec<IsochronePolygon>> {
        let start_node = self
            .graph
            .nearest_node(origin)
            .ok_or_else(|| crate::error::RoutingError::InvalidCoordinates("Origin not found".into()))?;

        let max_time = time_thresholds.iter().copied().fold(0.0, f64::max);
        let all_reachable = self.compute_reachable_with_times(start_node, max_time);

        let mut contours = Vec::new();

        for &threshold in time_thresholds {
            let reachable: HashMap<NodeId, f64> = all_reachable
                .iter()
                .filter(|(_, &time)| time <= threshold)
                .map(|(&node, &time)| (node, time))
                .collect();

            let polygon = self.build_polygon(&reachable.keys().copied().collect());

            contours.push(IsochronePolygon {
                time_threshold: threshold,
                polygon,
                center: origin,
                nodes_reached: reachable.len(),
                metadata: IsochroneMetadata {
                    transport_mode: Some(profile.name().to_string()),
                    created_at: Some(chrono::Utc::now()),
                    computation_time_ms: None,
                },
            });
        }

        Ok(contours)
    }

    /// Compute reachable nodes within time limit
    fn compute_reachable(&self, start: NodeId, max_time: f64) -> Vec<NodeId> {
        let mut pq: PriorityQueue<NodeId, Reverse<OrderedFloat<f64>>> = PriorityQueue::new();
        let mut times: HashMap<NodeId, f64> = HashMap::new();
        let mut reachable = Vec::new();

        pq.push(start, Reverse(OrderedFloat(0.0)));
        times.insert(start, 0.0);

        while let Some((node, Reverse(OrderedFloat(time)))) = pq.pop() {
            if time > max_time {
                continue;
            }

            reachable.push(node);

            for &edge_id in self.graph.outgoing_edges(node) {
                if let Some(edge) = self.graph.edge(edge_id) {
                    let next = edge.target;
                    let next_time = time + edge.cost.base_time;

                    if next_time <= max_time {
                        let current_time = times.get(&next).copied().unwrap_or(f64::INFINITY);
                        if next_time < current_time {
                            times.insert(next, next_time);
                            pq.push(next, Reverse(OrderedFloat(next_time)));
                        }
                    }
                }
            }
        }

        reachable
    }

    /// Compute reachable nodes with their times
    fn compute_reachable_with_times(
        &self,
        start: NodeId,
        max_time: f64,
    ) -> HashMap<NodeId, f64> {
        let mut pq: PriorityQueue<NodeId, Reverse<OrderedFloat<f64>>> = PriorityQueue::new();
        let mut times: HashMap<NodeId, f64> = HashMap::new();

        pq.push(start, Reverse(OrderedFloat(0.0)));
        times.insert(start, 0.0);

        while let Some((node, Reverse(OrderedFloat(time)))) = pq.pop() {
            if time > max_time {
                continue;
            }

            for &edge_id in self.graph.outgoing_edges(node) {
                if let Some(edge) = self.graph.edge(edge_id) {
                    let next = edge.target;
                    let next_time = time + edge.cost.base_time;

                    if next_time <= max_time {
                        let current_time = times.get(&next).copied().unwrap_or(f64::INFINITY);
                        if next_time < current_time {
                            times.insert(next, next_time);
                            pq.push(next, Reverse(OrderedFloat(next_time)));
                        }
                    }
                }
            }
        }

        times
    }

    /// Build polygon from reachable nodes using concave hull
    fn build_polygon(&self, nodes: &[NodeId]) -> MultiPolygon {
        if nodes.is_empty() {
            return MultiPolygon(vec![]);
        }

        // Extract node coordinates
        let mut points: Vec<Coord> = nodes
            .iter()
            .filter_map(|&node_id| {
                self.graph.node(node_id).map(|node| Coord {
                    x: node.location.x(),
                    y: node.location.y(),
                })
            })
            .collect();

        if points.is_empty() {
            return MultiPolygon(vec![]);
        }

        // Simple convex hull for now
        // In production, would use concave hull or grid-based approach
        let hull = convex_hull(&mut points);

        if hull.len() < 3 {
            return MultiPolygon(vec![]);
        }

        MultiPolygon(vec![Polygon::new(
            geo_types::LineString::from(hull),
            vec![],
        )])
    }
}

/// Compute convex hull using Graham scan
fn convex_hull(points: &mut [Coord]) -> Vec<Coord> {
    if points.len() < 3 {
        return points.to_vec();
    }

    // Find bottom-most point (or leftmost in case of tie)
    let min_idx = points
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a.y.partial_cmp(&b.y)
                .unwrap()
                .then(a.x.partial_cmp(&b.x).unwrap())
        })
        .map(|(i, _)| i)
        .unwrap();

    points.swap(0, min_idx);
    let pivot = points[0];

    // Sort by polar angle
    points[1..].sort_by(|a, b| {
        let angle_a = (a.y - pivot.y).atan2(a.x - pivot.x);
        let angle_b = (b.y - pivot.y).atan2(b.x - pivot.x);
        angle_a.partial_cmp(&angle_b).unwrap()
    });

    let mut hull = Vec::new();
    hull.push(points[0]);
    hull.push(points[1]);

    for i in 2..points.len() {
        while hull.len() > 1 && !is_left_turn(hull[hull.len() - 2], hull[hull.len() - 1], points[i]) {
            hull.pop();
        }
        hull.push(points[i]);
    }

    // Close the polygon
    if !hull.is_empty() {
        hull.push(hull[0]);
    }

    hull
}

/// Check if three points make a left turn
fn is_left_turn(a: Coord, b: Coord, c: Coord) -> bool {
    let cross = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    cross > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::GraphBuilder;

    #[test]
    fn test_isochrone_basic() {
        let graph = GraphBuilder::create_grid(10, 10, 0.01).unwrap();
        let builder = IsochroneBuilder::new(&graph);

        let origin = Point::new(0.0, 0.0);
        let profile = crate::profile::RoutingProfile::driving();

        let isochrone = builder.build(origin, 300.0, &profile).unwrap();

        assert!(isochrone.nodes_reached > 0);
        assert_eq!(isochrone.time_threshold, 300.0);
    }
}
