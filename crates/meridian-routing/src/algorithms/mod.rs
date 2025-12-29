//! Routing algorithms

pub mod astar;
pub mod dijkstra;
pub mod ch;
pub mod hub_labels;
pub mod alt;
pub mod many_to_many;

pub use astar::AStarRouter;
pub use dijkstra::DijkstraRouter;
pub use ch::ContractionHierarchies;
pub use hub_labels::HubLabeling;
pub use alt::ALTPreprocessor;

use crate::api::{RoutingRequest, RoutingResponse};
use crate::error::Result;
use crate::graph::Graph;

/// Trait for routing algorithms
pub trait RouteAlgorithm {
    /// Compute route from request
    fn route(&self, request: &RoutingRequest, graph: &Graph) -> Result<RoutingResponse>;

    /// Name of the algorithm
    fn name(&self) -> &'static str;
}
