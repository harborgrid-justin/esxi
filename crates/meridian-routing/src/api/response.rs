//! Routing response types

use crate::graph::{EdgeId, Graph};
use geo_types::{LineString, Point};
use serde::{Deserialize, Serialize};

/// A routing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResponse {
    /// Total distance (meters)
    pub distance: f64,

    /// Total duration (seconds)
    pub duration: f64,

    /// Route geometry
    pub geometry: RouteGeometry,

    /// Detailed segments
    pub segments: Vec<RouteSegment>,

    /// Waypoints (including start and end)
    pub waypoints: Vec<Point>,
}

impl RoutingResponse {
    /// Create empty response
    pub fn empty() -> Self {
        Self {
            distance: 0.0,
            duration: 0.0,
            geometry: RouteGeometry::default(),
            segments: Vec::new(),
            waypoints: Vec::new(),
        }
    }

    /// Get average speed (km/h)
    pub fn average_speed_kmh(&self) -> f64 {
        if self.duration > 0.0 {
            (self.distance / 1000.0) / (self.duration / 3600.0)
        } else {
            0.0
        }
    }
}

/// Route geometry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteGeometry {
    /// Encoded polyline or coordinates
    pub coordinates: Vec<Point>,

    /// Polyline encoding (optional)
    pub encoded: Option<String>,
}

impl RouteGeometry {
    /// Create from edge path
    pub fn from_edges(edges: &[EdgeId], graph: &Graph) -> Self {
        let mut coordinates = Vec::new();

        for &edge_id in edges {
            if let Some(edge) = graph.edge(edge_id) {
                // Add source node
                if let Some(node) = graph.node(edge.source) {
                    coordinates.push(node.location);
                }

                // Add intermediate geometry if available
                if let Some(ref geom) = edge.geometry {
                    for coord in &geom.0 {
                        coordinates.push(Point::new(coord.x, coord.y));
                    }
                }

                // Add target node
                if let Some(node) = graph.node(edge.target) {
                    coordinates.push(node.location);
                }
            }
        }

        // Remove duplicates
        coordinates.dedup_by(|a, b| {
            (a.x() - b.x()).abs() < 1e-6 && (a.y() - b.y()).abs() < 1e-6
        });

        Self {
            coordinates,
            encoded: None,
        }
    }

    /// Get as LineString
    pub fn as_linestring(&self) -> LineString {
        LineString::from(
            self.coordinates
                .iter()
                .map(|p| (p.x(), p.y()))
                .collect::<Vec<_>>(),
        )
    }

    /// Encode as polyline (simplified)
    pub fn encode(&mut self) {
        // Would use polyline encoding algorithm
        self.encoded = Some(format!("encoded_{}_points", self.coordinates.len()));
    }
}

/// A segment of the route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSegment {
    /// Segment index
    pub index: usize,

    /// Distance of this segment (meters)
    pub distance: f64,

    /// Duration of this segment (seconds)
    pub duration: f64,

    /// Instruction for this segment
    pub instruction: Option<Instruction>,

    /// Road name
    pub name: Option<String>,

    /// Road class
    pub road_class: Option<String>,

    /// Geometry for this segment
    pub geometry: Vec<Point>,
}

/// Turn-by-turn instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    /// Instruction type
    pub instruction_type: InstructionType,

    /// Text description
    pub text: String,

    /// Distance to next instruction (meters)
    pub distance_to_next: f64,

    /// Time to next instruction (seconds)
    pub time_to_next: f64,

    /// Exit number (for roundabouts)
    pub exit: Option<u8>,
}

/// Type of navigation instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstructionType {
    Depart,
    Arrive,
    TurnLeft,
    TurnRight,
    TurnSlightLeft,
    TurnSlightRight,
    TurnSharpLeft,
    TurnSharpRight,
    UturnLeft,
    UturnRight,
    Continue,
    Merge,
    OnRamp,
    OffRamp,
    Fork,
    Roundabout,
    RotaryLeft,
    RotaryRight,
    FerryEnter,
    FerryExit,
}

impl InstructionType {
    pub fn to_text(&self) -> &'static str {
        match self {
            InstructionType::Depart => "Depart",
            InstructionType::Arrive => "Arrive at destination",
            InstructionType::TurnLeft => "Turn left",
            InstructionType::TurnRight => "Turn right",
            InstructionType::TurnSlightLeft => "Turn slight left",
            InstructionType::TurnSlightRight => "Turn slight right",
            InstructionType::TurnSharpLeft => "Turn sharp left",
            InstructionType::TurnSharpRight => "Turn sharp right",
            InstructionType::UturnLeft => "Make a U-turn",
            InstructionType::UturnRight => "Make a U-turn",
            InstructionType::Continue => "Continue",
            InstructionType::Merge => "Merge",
            InstructionType::OnRamp => "Take the ramp",
            InstructionType::OffRamp => "Take the exit",
            InstructionType::Fork => "At the fork",
            InstructionType::Roundabout => "Enter the roundabout",
            InstructionType::RotaryLeft => "At the rotary, turn left",
            InstructionType::RotaryRight => "At the rotary, turn right",
            InstructionType::FerryEnter => "Board the ferry",
            InstructionType::FerryExit => "Exit the ferry",
        }
    }
}

/// Alternative route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeRoute {
    /// Route index
    pub index: usize,

    /// Main response
    pub route: RoutingResponse,

    /// Difference from primary route
    pub difference: RouteDifference,
}

/// Difference between routes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDifference {
    /// Distance difference (meters)
    pub distance_diff: f64,

    /// Duration difference (seconds)
    pub duration_diff: f64,

    /// Percentage longer
    pub percent_longer: f64,
}

/// Batch routing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRoutingResponse {
    /// Distance matrix (sources × targets)
    pub distances: Vec<Vec<f64>>,

    /// Duration matrix (sources × targets)
    pub durations: Vec<Vec<f64>>,

    /// Full routes (if requested)
    pub routes: Option<Vec<Vec<RoutingResponse>>>,
}

impl BatchRoutingResponse {
    pub fn new(distances: Vec<Vec<f64>>, durations: Vec<Vec<f64>>) -> Self {
        Self {
            distances,
            durations,
            routes: None,
        }
    }

    pub fn with_routes(mut self, routes: Vec<Vec<RoutingResponse>>) -> Self {
        self.routes = Some(routes);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_geometry() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
        ];

        let geometry = RouteGeometry {
            coordinates: points,
            encoded: None,
        };

        assert_eq!(geometry.coordinates.len(), 3);

        let linestring = geometry.as_linestring();
        assert_eq!(linestring.0.len(), 3);
    }

    #[test]
    fn test_instruction_text() {
        assert_eq!(InstructionType::TurnLeft.to_text(), "Turn left");
        assert_eq!(InstructionType::Arrive.to_text(), "Arrive at destination");
    }
}
