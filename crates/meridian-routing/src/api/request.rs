//! Routing request types

use crate::profile::RoutingProfile;
use geo_types::Point;
use serde::{Deserialize, Serialize};

/// A routing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRequest {
    /// Origin point
    pub origin: Point,

    /// Destination point
    pub destination: Point,

    /// Routing profile (car, bike, walk, etc.)
    pub profile: RoutingProfile,

    /// Optional waypoints to visit
    pub waypoints: Vec<Point>,

    /// Departure time (for time-dependent routing)
    pub departure_time: Option<f64>,

    /// Arrival time (for reverse routing)
    pub arrival_time: Option<f64>,

    /// Request alternatives
    pub alternatives: bool,

    /// Number of alternative routes
    pub num_alternatives: usize,

    /// Include turn-by-turn instructions
    pub instructions: bool,

    /// Include full geometry
    pub geometry: bool,

    /// Optimization options
    pub options: RoutingOptions,
}

impl RoutingRequest {
    /// Create a new routing request
    pub fn new(origin: Point, destination: Point, profile: RoutingProfile) -> Self {
        Self {
            origin,
            destination,
            profile,
            waypoints: Vec::new(),
            departure_time: None,
            arrival_time: None,
            alternatives: false,
            num_alternatives: 0,
            instructions: true,
            geometry: true,
            options: RoutingOptions::default(),
        }
    }

    /// Add waypoint
    pub fn with_waypoint(mut self, waypoint: Point) -> Self {
        self.waypoints.push(waypoint);
        self
    }

    /// Add multiple waypoints
    pub fn with_waypoints(mut self, waypoints: Vec<Point>) -> Self {
        self.waypoints = waypoints;
        self
    }

    /// Set departure time
    pub fn with_departure_time(mut self, time: f64) -> Self {
        self.departure_time = Some(time);
        self
    }

    /// Request alternative routes
    pub fn with_alternatives(mut self, num: usize) -> Self {
        self.alternatives = true;
        self.num_alternatives = num;
        self
    }

    /// Enable/disable instructions
    pub fn with_instructions(mut self, enable: bool) -> Self {
        self.instructions = enable;
        self
    }

    /// Enable/disable geometry
    pub fn with_geometry(mut self, enable: bool) -> Self {
        self.geometry = enable;
        self
    }

    /// Set options
    pub fn with_options(mut self, options: RoutingOptions) -> Self {
        self.options = options;
        self
    }
}

/// Routing optimization options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingOptions {
    /// Optimize for (distance, time, or balanced)
    pub optimize_for: OptimizationGoal,

    /// Avoid tolls
    pub avoid_tolls: bool,

    /// Avoid highways
    pub avoid_highways: bool,

    /// Avoid ferries
    pub avoid_ferries: bool,

    /// Avoid unpaved roads
    pub avoid_unpaved: bool,

    /// Maximum detour factor (1.0 = no detour)
    pub max_detour_factor: f64,

    /// Use traffic data
    pub use_traffic: bool,
}

impl Default for RoutingOptions {
    fn default() -> Self {
        Self {
            optimize_for: OptimizationGoal::Time,
            avoid_tolls: false,
            avoid_highways: false,
            avoid_ferries: false,
            avoid_unpaved: false,
            max_detour_factor: 1.5,
            use_traffic: true,
        }
    }
}

/// Optimization goal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationGoal {
    /// Minimize travel time
    Time,

    /// Minimize distance
    Distance,

    /// Balanced between time and distance
    Balanced,

    /// Minimize fuel consumption
    Fuel,

    /// Safest route
    Safety,
}

/// Batch routing request for multiple origins/destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRoutingRequest {
    /// Source locations
    pub sources: Vec<Point>,

    /// Target locations
    pub targets: Vec<Point>,

    /// Routing profile
    pub profile: RoutingProfile,

    /// Return full routes or just distances
    pub full_routes: bool,
}

impl BatchRoutingRequest {
    pub fn new(sources: Vec<Point>, targets: Vec<Point>, profile: RoutingProfile) -> Self {
        Self {
            sources,
            targets,
            profile,
            full_routes: false,
        }
    }

    pub fn with_full_routes(mut self, enable: bool) -> Self {
        self.full_routes = enable;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let request = RoutingRequest::new(
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            RoutingProfile::driving(),
        )
        .with_waypoint(Point::new(0.5, 0.5))
        .with_alternatives(2)
        .with_instructions(true);

        assert_eq!(request.waypoints.len(), 1);
        assert!(request.alternatives);
        assert_eq!(request.num_alternatives, 2);
    }
}
