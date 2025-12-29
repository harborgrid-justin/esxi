//! Node representation in the routing graph

use geo_types::Point;
use serde::{Deserialize, Serialize};

/// Node identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

impl NodeId {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

/// A node in the routing graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique node identifier
    pub id: NodeId,

    /// Geographic location
    pub location: Point,

    /// Original OSM node ID (if from OSM)
    pub osm_id: Option<i64>,

    /// Elevation in meters
    pub elevation: Option<f32>,

    /// Node type (intersection, endpoint, etc.)
    pub node_type: NodeType,

    /// Traffic light present
    pub has_traffic_light: bool,

    /// Contraction hierarchies level (for CH preprocessing)
    pub ch_level: Option<u32>,

    /// Hub label data (for hub labeling algorithm)
    pub hub_forward: Option<Vec<HubLabel>>,
    pub hub_backward: Option<Vec<HubLabel>>,
}

impl Node {
    /// Create a new node
    pub fn new(id: NodeId, location: Point) -> Self {
        Self {
            id,
            location,
            osm_id: None,
            elevation: None,
            node_type: NodeType::Intersection,
            has_traffic_light: false,
            ch_level: None,
            hub_forward: None,
            hub_backward: None,
        }
    }

    /// Create node with OSM ID
    pub fn with_osm_id(mut self, osm_id: i64) -> Self {
        self.osm_id = Some(osm_id);
        self
    }

    /// Set elevation
    pub fn with_elevation(mut self, elevation: f32) -> Self {
        self.elevation = Some(elevation);
        self
    }

    /// Set node type
    pub fn with_type(mut self, node_type: NodeType) -> Self {
        self.node_type = node_type;
        self
    }

    /// Set traffic light
    pub fn with_traffic_light(mut self) -> Self {
        self.has_traffic_light = true;
        self
    }

    /// Calculate delay at this node (traffic light, stop sign, etc.)
    pub fn delay(&self) -> f64 {
        match self.node_type {
            NodeType::TrafficLight => {
                if self.has_traffic_light {
                    15.0 // Average traffic light delay
                } else {
                    0.0
                }
            }
            NodeType::StopSign => 5.0,
            NodeType::YieldSign => 2.0,
            _ => 0.0,
        }
    }
}

/// Type of node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Regular intersection
    Intersection,
    /// Traffic light controlled intersection
    TrafficLight,
    /// Stop sign
    StopSign,
    /// Yield sign
    YieldSign,
    /// Dead end
    DeadEnd,
    /// Highway entry/exit
    HighwayJunction,
    /// Artificial node (for graph structure)
    Artificial,
}

/// Hub label for hub labeling algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubLabel {
    /// Hub node ID
    pub hub: NodeId,
    /// Distance to/from hub
    pub distance: f64,
}

impl HubLabel {
    pub fn new(hub: NodeId, distance: f64) -> Self {
        Self { hub, distance }
    }
}
