//! Edge representation in the routing graph

use super::NodeId;
use geo_types::LineString;
use serde::{Deserialize, Serialize};

/// Edge identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub usize);

impl EdgeId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

/// An edge in the routing graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Unique edge identifier
    pub id: EdgeId,

    /// Source node
    pub source: NodeId,

    /// Target node
    pub target: NodeId,

    /// Edge cost(s)
    pub cost: EdgeCost,

    /// Original OSM way ID (if from OSM)
    pub osm_way_id: Option<i64>,

    /// Road classification
    pub road_class: RoadClass,

    /// Maximum speed (km/h)
    pub max_speed: Option<f32>,

    /// Road name
    pub name: Option<String>,

    /// Geometry (if needed for visualization)
    pub geometry: Option<LineString>,

    /// One-way restriction
    pub oneway: bool,

    /// Bearing at the start of the edge (degrees)
    pub bearing: f64,

    /// Access restrictions
    pub access: AccessRestrictions,

    /// Surface type
    pub surface: SurfaceType,

    /// Number of lanes
    pub lanes: Option<u8>,

    /// Shortcut edge (for Contraction Hierarchies)
    pub is_shortcut: bool,

    /// If shortcut, which edge does it bypass
    pub shortcut_via: Option<NodeId>,
}

impl Edge {
    /// Create a new edge
    pub fn new(id: EdgeId, source: NodeId, target: NodeId, cost: EdgeCost) -> Self {
        Self {
            id,
            source,
            target,
            cost,
            osm_way_id: None,
            road_class: RoadClass::Unclassified,
            max_speed: None,
            name: None,
            geometry: None,
            oneway: false,
            bearing: 0.0,
            access: AccessRestrictions::default(),
            surface: SurfaceType::Paved,
            lanes: None,
            is_shortcut: false,
            shortcut_via: None,
        }
    }

    /// Get travel time for this edge (in seconds)
    pub fn travel_time(&self, time_of_day: Option<f64>) -> f64 {
        if let Some(tod) = time_of_day {
            self.cost.time_dependent_cost(tod)
        } else {
            self.cost.base_time
        }
    }

    /// Get edge length (in meters)
    pub fn length(&self) -> f64 {
        self.cost.distance
    }

    /// Check if vehicle type can use this edge
    pub fn accessible_by(&self, vehicle: VehicleType) -> bool {
        match vehicle {
            VehicleType::Car => self.access.car,
            VehicleType::Bicycle => self.access.bicycle,
            VehicleType::Pedestrian => self.access.foot,
            VehicleType::Truck => self.access.truck,
            VehicleType::Bus => self.access.bus,
            VehicleType::Motorcycle => self.access.motorcycle,
        }
    }
}

/// Edge cost representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCost {
    /// Base travel time (seconds)
    pub base_time: f64,

    /// Distance (meters)
    pub distance: f64,

    /// Time-dependent cost profile (24 time slots for each hour)
    pub time_profile: Option<Vec<f64>>,

    /// Additional penalties
    pub toll_cost: Option<f64>,
    pub ferry: bool,
}

impl EdgeCost {
    /// Create simple cost from distance and speed
    pub fn from_distance_speed(distance: f64, speed_kmh: f64) -> Self {
        let time = (distance / 1000.0) / speed_kmh * 3600.0; // Convert to seconds
        Self {
            base_time: time,
            distance,
            time_profile: None,
            toll_cost: None,
            ferry: false,
        }
    }

    /// Get cost at specific time of day (0-24 hours)
    pub fn time_dependent_cost(&self, time_of_day: f64) -> f64 {
        if let Some(ref profile) = self.time_profile {
            let hour = time_of_day.floor() as usize % 24;
            profile.get(hour).copied().unwrap_or(self.base_time)
        } else {
            self.base_time
        }
    }

    /// Apply traffic multiplier
    pub fn with_traffic(&self, multiplier: f64) -> Self {
        let mut cost = self.clone();
        cost.base_time *= multiplier;
        if let Some(ref mut profile) = cost.time_profile {
            for t in profile.iter_mut() {
                *t *= multiplier;
            }
        }
        cost
    }
}

/// Road classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoadClass {
    Motorway,
    Trunk,
    Primary,
    Secondary,
    Tertiary,
    Residential,
    Service,
    Track,
    Path,
    Cycleway,
    Footway,
    Unclassified,
}

impl RoadClass {
    /// Get default speed for road class (km/h)
    pub fn default_speed(&self) -> f32 {
        match self {
            RoadClass::Motorway => 120.0,
            RoadClass::Trunk => 100.0,
            RoadClass::Primary => 80.0,
            RoadClass::Secondary => 60.0,
            RoadClass::Tertiary => 50.0,
            RoadClass::Residential => 30.0,
            RoadClass::Service => 20.0,
            RoadClass::Track => 15.0,
            RoadClass::Path => 5.0,
            RoadClass::Cycleway => 20.0,
            RoadClass::Footway => 5.0,
            RoadClass::Unclassified => 40.0,
        }
    }

    /// Road hierarchy level (higher = more important)
    pub fn hierarchy_level(&self) -> u8 {
        match self {
            RoadClass::Motorway => 10,
            RoadClass::Trunk => 9,
            RoadClass::Primary => 8,
            RoadClass::Secondary => 7,
            RoadClass::Tertiary => 6,
            RoadClass::Residential => 5,
            RoadClass::Service => 4,
            RoadClass::Unclassified => 3,
            RoadClass::Track => 2,
            _ => 1,
        }
    }
}

/// Access restrictions for different vehicle types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AccessRestrictions {
    pub car: bool,
    pub truck: bool,
    pub bus: bool,
    pub bicycle: bool,
    pub foot: bool,
    pub motorcycle: bool,
}

impl Default for AccessRestrictions {
    fn default() -> Self {
        Self {
            car: true,
            truck: true,
            bus: true,
            bicycle: true,
            foot: true,
            motorcycle: true,
        }
    }
}

/// Vehicle type for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Car,
    Truck,
    Bus,
    Bicycle,
    Pedestrian,
    Motorcycle,
}

/// Surface type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceType {
    Paved,
    Unpaved,
    Gravel,
    Dirt,
    Sand,
    Grass,
}

/// Turn restriction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnRestriction {
    /// Edge entering the intersection
    pub from_edge: EdgeId,
    /// Node at the intersection
    pub via_node: NodeId,
    /// Edge leaving the intersection
    pub to_edge: EdgeId,
    /// Type of restriction
    pub restriction_type: RestrictionType,
}

/// Type of turn restriction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestrictionType {
    /// Turn is not allowed
    NoTurn,
    /// Only this turn is allowed from this edge
    OnlyTurn,
    /// Conditional restriction (time-based, etc.)
    Conditional,
}
