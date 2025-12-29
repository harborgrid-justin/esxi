//! # Meridian Routing Engine
//!
//! Advanced routing and network analysis for the Meridian GIS Platform.
//!
//! ## Features
//!
//! - **High-Performance Routing**: Sub-second routing for continental-scale networks
//! - **Multiple Algorithms**: Dijkstra, A*, Contraction Hierarchies, Hub Labels, ALT
//! - **Isochrone Generation**: Create reachability polygons for travel time analysis
//! - **Route Optimization**: TSP, VRP, pickup-delivery optimization
//! - **Traffic Integration**: Historical patterns, real-time updates, prediction
//! - **Multimodal Routing**: Transit, walking, cycling with GTFS support
//! - **Time-Dependent Routing**: Account for time-varying edge costs
//! - **Turn Restrictions**: Support complex intersection constraints
//!
//! ## Example
//!
//! ```rust,no_run
//! use meridian_routing::{RoutingEngine, RoutingRequest, RoutingProfile};
//! use geo_types::Point;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create routing engine
//! let mut engine = RoutingEngine::new();
//!
//! // Build graph from OSM data
//! engine.load_from_osm("map.osm.pbf")?;
//!
//! // Create routing request
//! let request = RoutingRequest::new(
//!     Point::new(-122.4194, 37.7749),  // San Francisco
//!     Point::new(-118.2437, 34.0522),  // Los Angeles
//!     RoutingProfile::driving(),
//! );
//!
//! // Find route
//! let response = engine.route(&request)?;
//! println!("Distance: {:.2} km", response.distance / 1000.0);
//! println!("Duration: {:.0} min", response.duration / 60.0);
//! # Ok(())
//! # }
//! ```

pub mod algorithms;
pub mod api;
pub mod error;
pub mod graph;
pub mod isochrone;
pub mod multimodal;
pub mod optimization;
pub mod profile;
pub mod traffic;

// Re-exports
pub use error::{RoutingError, Result};
pub use graph::{Graph, GraphBuilder, Node, Edge, EdgeCost};
pub use algorithms::{
    RouteAlgorithm, DijkstraRouter, AStarRouter, ContractionHierarchies,
};
pub use api::{RoutingRequest, RoutingResponse, RouteSegment, RouteGeometry};
pub use profile::{RoutingProfile, VehicleProfile};
pub use isochrone::{IsochroneBuilder, IsochronePolygon};

use std::path::Path;
use std::sync::Arc;

/// Main routing engine that coordinates all routing operations
pub struct RoutingEngine {
    /// Internal graph representation
    graph: Arc<Graph>,

    /// Preprocessed data for fast routing
    ch: Option<ContractionHierarchies>,

    /// Traffic manager
    traffic: Option<traffic::TrafficManager>,

    /// Transit data
    #[cfg(feature = "transit")]
    transit: Option<multimodal::TransitNetwork>,
}

impl RoutingEngine {
    /// Create a new routing engine
    pub fn new() -> Self {
        Self {
            graph: Arc::new(Graph::default()),
            ch: None,
            traffic: None,
            #[cfg(feature = "transit")]
            transit: None,
        }
    }

    /// Load graph from OSM PBF file
    pub fn load_from_osm<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let mut builder = GraphBuilder::new();
        builder.load_osm(path)?;
        self.graph = Arc::new(builder.build()?);
        Ok(())
    }

    /// Load graph from serialized format
    pub fn load_graph<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.graph = Arc::new(Graph::load(path)?);
        Ok(())
    }

    /// Save graph to disk
    pub fn save_graph<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.graph.save(path)
    }

    /// Preprocess graph with Contraction Hierarchies
    pub fn preprocess_ch(&mut self) -> Result<()> {
        let ch = ContractionHierarchies::preprocess(&self.graph)?;
        self.ch = Some(ch);
        Ok(())
    }

    /// Enable traffic integration
    pub fn enable_traffic(&mut self, config: traffic::TrafficConfig) -> Result<()> {
        self.traffic = Some(traffic::TrafficManager::new(config));
        Ok(())
    }

    /// Load GTFS transit data
    #[cfg(feature = "transit")]
    pub fn load_transit<P: AsRef<Path>>(&mut self, gtfs_path: P) -> Result<()> {
        let transit = multimodal::TransitNetwork::from_gtfs(gtfs_path)?;
        self.transit = Some(transit);
        Ok(())
    }

    /// Execute a routing request
    pub fn route(&self, request: &RoutingRequest) -> Result<RoutingResponse> {
        // Select best algorithm based on request and available preprocessing
        if let Some(ref ch) = self.ch {
            ch.route(request, &self.graph)
        } else {
            // Fall back to A* for one-off queries
            let router = AStarRouter::new();
            router.route(request, &self.graph)
        }
    }

    /// Generate isochrone polygon
    pub fn isochrone(
        &self,
        origin: geo_types::Point,
        max_time: f64,
        profile: &RoutingProfile,
    ) -> Result<IsochronePolygon> {
        let builder = IsochroneBuilder::new(&self.graph);
        builder.build(origin, max_time, profile)
    }

    /// Calculate many-to-many distance matrix
    pub fn distance_matrix(
        &self,
        sources: &[geo_types::Point],
        targets: &[geo_types::Point],
        profile: &RoutingProfile,
    ) -> Result<Vec<Vec<f64>>> {
        algorithms::many_to_many::calculate_matrix(&self.graph, sources, targets, profile)
    }

    /// Optimize traveling salesman problem
    pub fn optimize_tsp(
        &self,
        waypoints: &[geo_types::Point],
        profile: &RoutingProfile,
    ) -> Result<optimization::TspSolution> {
        optimization::tsp::solve(&self.graph, waypoints, profile)
    }

    /// Solve vehicle routing problem
    pub fn optimize_vrp(
        &self,
        problem: &optimization::VrpProblem,
    ) -> Result<optimization::VrpSolution> {
        optimization::vrp::solve(&self.graph, problem)
    }

    /// Get graph statistics
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            nodes: self.graph.node_count(),
            edges: self.graph.edge_count(),
            ch_preprocessed: self.ch.is_some(),
            traffic_enabled: self.traffic.is_some(),
            #[cfg(feature = "transit")]
            transit_loaded: self.transit.is_some(),
        }
    }
}

impl Default for RoutingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the routing graph
#[derive(Debug, Clone)]
pub struct GraphStats {
    pub nodes: usize,
    pub edges: usize,
    pub ch_preprocessed: bool,
    pub traffic_enabled: bool,
    #[cfg(feature = "transit")]
    pub transit_loaded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = RoutingEngine::new();
        let stats = engine.stats();
        assert_eq!(stats.nodes, 0);
        assert_eq!(stats.edges, 0);
    }
}
