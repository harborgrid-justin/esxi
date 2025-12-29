//! Isochrone generation for reachability analysis

pub mod builder;
pub mod multimodal;

pub use builder::IsochroneBuilder;
pub use multimodal::MultimodalIsochroneBuilder;

use geo_types::{Polygon, MultiPolygon, Point};
use serde::{Deserialize, Serialize};

/// An isochrone polygon representing reachable area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsochronePolygon {
    /// Travel time threshold (seconds)
    pub time_threshold: f64,

    /// Reachable area polygon
    pub polygon: MultiPolygon,

    /// Center point
    pub center: Point,

    /// Number of nodes reached
    pub nodes_reached: usize,

    /// Metadata
    pub metadata: IsochroneMetadata,
}

impl IsochronePolygon {
    /// Get area in square meters
    pub fn area(&self) -> f64 {
        use geo::Area;
        self.polygon.unsigned_area()
    }

    /// Check if point is within isochrone
    pub fn contains(&self, point: Point) -> bool {
        use geo::Contains;
        self.polygon.contains(&point)
    }
}

impl Default for IsochronePolygon {
    fn default() -> Self {
        Self {
            time_threshold: 0.0,
            polygon: MultiPolygon(vec![]),
            center: Point::new(0.0, 0.0),
            nodes_reached: 0,
            metadata: IsochroneMetadata::default(),
        }
    }
}

/// Metadata for isochrone
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IsochroneMetadata {
    pub transport_mode: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub computation_time_ms: Option<u64>,
}
