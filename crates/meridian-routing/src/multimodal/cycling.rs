//! Bicycle routing

use crate::api::{RoutingRequest, RoutingResponse};
use crate::error::Result;
use crate::graph::Graph;

/// Cycling router with bicycle-specific preferences
pub struct CyclingRouter {
    /// Cycling speed (km/h)
    cycling_speed: f64,

    /// Prefer bike lanes and paths
    prefer_bike_infrastructure: bool,

    /// Avoid busy roads
    avoid_busy_roads: bool,

    /// Maximum gradient tolerance (percent)
    max_gradient: f64,

    /// Bike type
    bike_type: BikeType,
}

/// Type of bicycle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BikeType {
    Road,
    Mountain,
    Hybrid,
    Electric,
}

impl BikeType {
    pub fn average_speed_kmh(&self) -> f64 {
        match self {
            BikeType::Road => 25.0,
            BikeType::Mountain => 18.0,
            BikeType::Hybrid => 20.0,
            BikeType::Electric => 28.0,
        }
    }

    pub fn max_gradient(&self) -> f64 {
        match self {
            BikeType::Road => 12.0,
            BikeType::Mountain => 20.0,
            BikeType::Hybrid => 15.0,
            BikeType::Electric => 18.0,
        }
    }
}

impl CyclingRouter {
    pub fn new(bike_type: BikeType) -> Self {
        Self {
            cycling_speed: bike_type.average_speed_kmh(),
            prefer_bike_infrastructure: true,
            avoid_busy_roads: true,
            max_gradient: bike_type.max_gradient(),
            bike_type,
        }
    }

    /// Set cycling speed
    pub fn with_speed(mut self, speed_kmh: f64) -> Self {
        self.cycling_speed = speed_kmh;
        self
    }

    /// Prefer bike infrastructure
    pub fn prefer_bike_infrastructure(mut self, prefer: bool) -> Self {
        self.prefer_bike_infrastructure = prefer;
        self
    }

    /// Avoid busy roads
    pub fn avoid_busy_roads(mut self, avoid: bool) -> Self {
        self.avoid_busy_roads = avoid;
        self
    }

    /// Route for cycling
    pub fn route(&self, request: &RoutingRequest, graph: &Graph) -> Result<RoutingResponse> {
        let router = crate::algorithms::AStarRouter::new();

        let mut cycling_request = request.clone();
        cycling_request.profile = crate::profile::RoutingProfile::cycling();

        router.route(&cycling_request, graph)
    }

    /// Calculate edge cost for cycling
    pub fn edge_cost(&self, edge: &crate::graph::Edge) -> f64 {
        use crate::graph::RoadClass;

        let base_cost = edge.cost.base_time;

        // Road type preference
        let type_multiplier = match edge.road_class {
            RoadClass::Cycleway => 0.8,  // Preferred
            RoadClass::Path => 0.9,
            RoadClass::Residential => 1.0,
            RoadClass::Tertiary => 1.1,
            RoadClass::Secondary => 1.3,
            RoadClass::Primary => if self.avoid_busy_roads { 2.0 } else { 1.5 },
            RoadClass::Trunk | RoadClass::Motorway => {
                return f64::INFINITY; // Not allowed
            }
            _ => 1.2,
        };

        // Surface penalty
        let surface_multiplier = match edge.surface {
            crate::graph::SurfaceType::Paved => 1.0,
            crate::graph::SurfaceType::Unpaved => match self.bike_type {
                BikeType::Mountain => 1.1,
                BikeType::Hybrid => 1.3,
                BikeType::Road => 2.0,
                BikeType::Electric => 1.5,
            },
            crate::graph::SurfaceType::Gravel => 1.4,
            crate::graph::SurfaceType::Dirt => 1.8,
            _ => 2.0,
        };

        // Gradient penalty
        let gradient_multiplier = if let Some(gradient) = calculate_gradient(edge) {
            if gradient > self.max_gradient {
                return f64::INFINITY; // Too steep
            }

            // Electric bikes handle gradients better
            let penalty = if matches!(self.bike_type, BikeType::Electric) {
                1.0 + (gradient / 20.0)
            } else {
                1.0 + (gradient / 8.0)
            };

            penalty
        } else {
            1.0
        };

        base_cost * type_multiplier * surface_multiplier * gradient_multiplier
    }

    /// Check if edge is accessible for cycling
    pub fn is_accessible(&self, edge: &crate::graph::Edge) -> bool {
        use crate::graph::{RoadClass, VehicleType};

        // Must be accessible to bicycles
        if !edge.accessible_by(VehicleType::Bicycle) {
            return false;
        }

        // No motorways or trunks
        if matches!(edge.road_class, RoadClass::Motorway | RoadClass::Trunk) {
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

    /// Get bike type
    pub fn bike_type(&self) -> BikeType {
        self.bike_type
    }
}

/// Calculate gradient of edge (percent)
fn calculate_gradient(_edge: &crate::graph::Edge) -> Option<f64> {
    // Would use elevation data if available
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycling_router() {
        let router = CyclingRouter::new(BikeType::Road)
            .with_speed(30.0)
            .prefer_bike_infrastructure(true);

        assert_eq!(router.cycling_speed, 30.0);
        assert_eq!(router.bike_type, BikeType::Road);
    }

    #[test]
    fn test_bike_types() {
        assert_eq!(BikeType::Electric.average_speed_kmh(), 28.0);
        assert_eq!(BikeType::Mountain.max_gradient(), 20.0);
    }
}
