//! Pedestrian routing

use crate::api::{RoutingRequest, RoutingResponse};
use crate::error::Result;
use crate::graph::Graph;

/// Walking router with pedestrian-specific preferences
pub struct WalkingRouter {
    /// Preferred walking speed (km/h)
    walking_speed: f64,

    /// Prefer sidewalks and pedestrian paths
    prefer_sidewalks: bool,

    /// Avoid stairs if possible
    avoid_stairs: bool,

    /// Maximum gradient (percent)
    max_gradient: f64,
}

impl WalkingRouter {
    pub fn new() -> Self {
        Self {
            walking_speed: 5.0, // 5 km/h default
            prefer_sidewalks: true,
            avoid_stairs: false,
            max_gradient: 15.0, // 15% gradient
        }
    }

    /// Set walking speed
    pub fn with_speed(mut self, speed_kmh: f64) -> Self {
        self.walking_speed = speed_kmh;
        self
    }

    /// Enable/disable sidewalk preference
    pub fn prefer_sidewalks(mut self, prefer: bool) -> Self {
        self.prefer_sidewalks = prefer;
        self
    }

    /// Avoid stairs
    pub fn avoid_stairs(mut self, avoid: bool) -> Self {
        self.avoid_stairs = avoid;
        self
    }

    /// Set maximum acceptable gradient
    pub fn max_gradient(mut self, gradient: f64) -> Self {
        self.max_gradient = gradient;
        self
    }

    /// Route for pedestrians
    pub fn route(&self, request: &RoutingRequest, graph: &Graph) -> Result<RoutingResponse> {
        // Use A* with pedestrian-friendly weights
        let router = crate::algorithms::AStarRouter::new();

        // Modify request for walking
        let mut walking_request = request.clone();
        walking_request.profile = crate::profile::RoutingProfile::walking();

        router.route(&walking_request, graph)
    }

    /// Calculate edge cost for walking
    pub fn edge_cost(&self, edge: &crate::graph::Edge) -> f64 {
        use crate::graph::RoadClass;

        let base_cost = edge.cost.base_time;

        // Adjust for road type
        let type_multiplier = match edge.road_class {
            RoadClass::Footway | RoadClass::Path => 1.0,  // Preferred
            RoadClass::Residential => 1.1,
            RoadClass::Cycleway => 1.2,  // Shared use
            RoadClass::Service => 1.3,
            _ => 2.0,  // Discourage non-pedestrian roads
        };

        // Elevation penalty
        let elevation_multiplier = if let Some(gradient) = calculate_gradient(edge) {
            if gradient > self.max_gradient {
                return f64::INFINITY; // Too steep
            }
            1.0 + (gradient / 10.0) // Add 10% cost per 10% gradient
        } else {
            1.0
        };

        base_cost * type_multiplier * elevation_multiplier
    }

    /// Check if edge is accessible for walking
    pub fn is_accessible(&self, edge: &crate::graph::Edge) -> bool {
        use crate::graph::VehicleType;

        // Must be accessible to pedestrians
        if !edge.accessible_by(VehicleType::Pedestrian) {
            return false;
        }

        // Check gradient
        if let Some(gradient) = calculate_gradient(edge) {
            if gradient > self.max_gradient {
                return false;
            }
        }

        true
    }
}

impl Default for WalkingRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate gradient of edge (percent)
fn calculate_gradient(edge: &crate::graph::Edge) -> Option<f64> {
    // Would use elevation data if available
    // For now, return None
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walking_router() {
        let router = WalkingRouter::new()
            .with_speed(4.5)
            .prefer_sidewalks(true)
            .avoid_stairs(true);

        assert_eq!(router.walking_speed, 4.5);
        assert!(router.prefer_sidewalks);
        assert!(router.avoid_stairs);
    }
}
