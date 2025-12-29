//! Graph construction from geographic data

use super::{Edge, EdgeCost, EdgeId, Graph, Node, NodeId, RoadClass, AccessRestrictions, SurfaceType, GeoBounds, GraphMetadata};
use crate::error::{Result, RoutingError};
use geo_types::{Point, LineString, Coord};
use hashbrown::HashMap;
use std::path::Path;

/// Builder for constructing routing graphs
pub struct GraphBuilder {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    node_map: HashMap<i64, NodeId>, // OSM ID -> NodeId
    bounds: Option<GeoBounds>,
}

impl GraphBuilder {
    /// Create a new graph builder
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_map: HashMap::new(),
            bounds: None,
        }
    }

    /// Load graph from OSM PBF file
    pub fn load_osm<P: AsRef<Path>>(&mut self, _path: P) -> Result<()> {
        // In a real implementation, this would use osmpbfreader or similar
        // For now, we'll provide a stub that could be implemented

        // Example implementation outline:
        // 1. Parse OSM PBF file
        // 2. Extract nodes with coordinates
        // 3. Extract ways (roads) with tags
        // 4. Build graph nodes from OSM nodes referenced by ways
        // 5. Build graph edges from way segments
        // 6. Parse road attributes (class, speed, access, etc.)

        log::warn!("OSM loading not fully implemented - this is a placeholder");
        Ok(())
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, location: Point, osm_id: Option<i64>) -> NodeId {
        let node_id = NodeId::new(self.nodes.len());
        let mut node = Node::new(node_id, location);

        if let Some(id) = osm_id {
            node.osm_id = Some(id);
            self.node_map.insert(id, node_id);
        }

        self.nodes.push(node);
        self.update_bounds(location);
        node_id
    }

    /// Add an edge to the graph
    pub fn add_edge(
        &mut self,
        source: NodeId,
        target: NodeId,
        distance: f64,
        speed_kmh: f64,
    ) -> EdgeId {
        let edge_id = EdgeId::new(self.edges.len());
        let cost = EdgeCost::from_distance_speed(distance, speed_kmh);
        let edge = Edge::new(edge_id, source, target, cost);
        self.edges.push(edge);
        edge_id
    }

    /// Add edge with full configuration
    pub fn add_edge_full(
        &mut self,
        source: NodeId,
        target: NodeId,
        cost: EdgeCost,
        road_class: RoadClass,
        max_speed: Option<f32>,
        name: Option<String>,
        geometry: Option<LineString>,
        oneway: bool,
        access: AccessRestrictions,
    ) -> EdgeId {
        let edge_id = EdgeId::new(self.edges.len());

        // Calculate bearing from geometry or node positions
        let bearing = if let Some(ref geom) = geometry {
            calculate_bearing_from_linestring(geom)
        } else if let (Some(src), Some(tgt)) = (self.nodes.get(source.0), self.nodes.get(target.0)) {
            calculate_bearing(src.location, tgt.location)
        } else {
            0.0
        };

        let mut edge = Edge::new(edge_id, source, target, cost);
        edge.road_class = road_class;
        edge.max_speed = max_speed;
        edge.name = name;
        edge.geometry = geometry;
        edge.oneway = oneway;
        edge.bearing = bearing;
        edge.access = access;

        self.edges.push(edge);
        edge_id
    }

    /// Get node by OSM ID
    pub fn get_node_by_osm_id(&self, osm_id: i64) -> Option<NodeId> {
        self.node_map.get(&osm_id).copied()
    }

    /// Build the final graph
    pub fn build(self) -> Result<Graph> {
        if self.nodes.is_empty() {
            return Err(RoutingError::EmptyGraph);
        }

        // Build adjacency lists
        let mut adjacency = vec![Vec::new(); self.nodes.len()];
        let mut reverse_adjacency = vec![Vec::new(); self.nodes.len()];

        for edge in &self.edges {
            adjacency[edge.source.0].push(edge.id);
            reverse_adjacency[edge.target.0].push(edge.id);
        }

        // Build spatial index
        use super::NodeSpatialIndex;
        let mut spatial_index = NodeSpatialIndex::new(0.01); // ~1km cells
        for (i, node) in self.nodes.iter().enumerate() {
            spatial_index.insert(node.location, NodeId(i));
        }

        let metadata = GraphMetadata {
            created_at: Some(chrono::Utc::now()),
            source: Some("GraphBuilder".to_string()),
            bounds: self.bounds,
        };

        let graph = Graph {
            nodes: self.nodes,
            edges: self.edges,
            adjacency,
            reverse_adjacency,
            spatial_index,
            turn_restrictions: Vec::new(),
            partition: None,
            metadata,
        };

        // Validate the graph
        graph.validate()?;

        Ok(graph)
    }

    /// Update geographic bounds
    fn update_bounds(&mut self, point: Point) {
        let lon = point.x();
        let lat = point.y();

        if let Some(ref mut bounds) = self.bounds {
            bounds.min_lon = bounds.min_lon.min(lon);
            bounds.max_lon = bounds.max_lon.max(lon);
            bounds.min_lat = bounds.min_lat.min(lat);
            bounds.max_lat = bounds.max_lat.max(lat);
        } else {
            self.bounds = Some(GeoBounds {
                min_lon: lon,
                max_lon: lon,
                min_lat: lat,
                max_lat: lat,
            });
        }
    }

    /// Create a simple grid graph for testing
    pub fn create_grid(width: usize, height: usize, cell_size: f64) -> Result<Graph> {
        let mut builder = GraphBuilder::new();

        // Create nodes
        let mut node_grid = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let lon = x as f64 * cell_size;
                let lat = y as f64 * cell_size;
                let node_id = builder.add_node(Point::new(lon, lat), None);
                node_grid.push(node_id);
            }
        }

        // Create edges (4-connected grid)
        for y in 0..height {
            for x in 0..width {
                let current = node_grid[y * width + x];

                // Right edge
                if x < width - 1 {
                    let right = node_grid[y * width + (x + 1)];
                    builder.add_edge(current, right, cell_size * 111_000.0, 50.0); // ~50 km/h
                    builder.add_edge(right, current, cell_size * 111_000.0, 50.0);
                }

                // Down edge
                if y < height - 1 {
                    let down = node_grid[(y + 1) * width + x];
                    builder.add_edge(current, down, cell_size * 111_000.0, 50.0);
                    builder.add_edge(down, current, cell_size * 111_000.0, 50.0);
                }
            }
        }

        builder.build()
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate bearing between two points (in degrees, 0-360)
fn calculate_bearing(from: Point, to: Point) -> f64 {
    let lat1 = from.y().to_radians();
    let lat2 = to.y().to_radians();
    let delta_lon = (to.x() - from.x()).to_radians();

    let y = delta_lon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * delta_lon.cos();

    let bearing = y.atan2(x).to_degrees();
    (bearing + 360.0) % 360.0
}

/// Calculate bearing from first segment of linestring
fn calculate_bearing_from_linestring(linestring: &LineString) -> f64 {
    if linestring.0.len() < 2 {
        return 0.0;
    }

    let from = Point::from(linestring.0[0]);
    let to = Point::from(linestring.0[1]);
    calculate_bearing(from, to)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let mut builder = GraphBuilder::new();
        let n1 = builder.add_node(Point::new(0.0, 0.0), None);
        let n2 = builder.add_node(Point::new(1.0, 1.0), None);
        builder.add_edge(n1, n2, 1000.0, 50.0);

        let graph = builder.build().unwrap();
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_grid_creation() {
        let graph = GraphBuilder::create_grid(5, 5, 0.01).unwrap();
        assert_eq!(graph.node_count(), 25);
        assert_eq!(graph.edge_count(), 80); // 2 * (4*5 + 4*5) = 2 * 40
    }
}
