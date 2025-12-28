//! # Meridian Analysis
//!
//! Spatial analysis and geoprocessing library for the Meridian GIS Platform.
//!
//! This crate provides a comprehensive suite of spatial analysis tools including:
//!
//! - **Buffer Analysis**: Point, line, and polygon buffering with variable widths
//! - **Overlay Operations**: Union, intersection, difference, and symmetric difference
//! - **Proximity Analysis**: Nearest neighbor, distance matrices, Voronoi diagrams
//! - **Network Analysis**: Shortest path, service areas, route optimization
//! - **Surface Analysis**: Slope, aspect, hillshade, contour generation
//! - **Spatial Statistics**: Hot spot detection, cluster analysis, autocorrelation
//! - **Geometry Transformation**: Simplification, smoothing, densification
//! - **Validation & Repair**: Geometry validation and automated repair
//!
//! ## Examples
//!
//! ### Buffer Analysis
//!
//! ```rust
//! use meridian_analysis::buffer::{buffer_point, BufferParams};
//! use geo::Point;
//!
//! let point = Point::new(0.0, 0.0);
//! let params = BufferParams::new(10.0).quadrant_segments(16);
//! let buffer = buffer_point(&point, &params).unwrap();
//! ```
//!
//! ### Overlay Operations
//!
//! ```rust
//! use meridian_analysis::overlay::{union, intersection};
//! use geo::{Polygon, LineString};
//!
//! let poly1 = Polygon::new(
//!     LineString::from(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0), (0.0, 0.0)]),
//!     vec![],
//! );
//! let poly2 = Polygon::new(
//!     LineString::from(vec![(5.0, 5.0), (15.0, 5.0), (15.0, 15.0), (5.0, 15.0), (5.0, 5.0)]),
//!     vec![],
//! );
//!
//! let union_result = union(&poly1, &poly2).unwrap();
//! let intersection_result = intersection(&poly1, &poly2).unwrap();
//! ```
//!
//! ### Spatial Statistics
//!
//! ```rust
//! use meridian_analysis::statistics::{point_pattern_analysis, calculate_mean_center};
//! use geo::{Point, Polygon, LineString};
//!
//! let points = vec![
//!     Point::new(0.0, 0.0),
//!     Point::new(5.0, 5.0),
//!     Point::new(10.0, 10.0),
//! ];
//!
//! let mean_center = calculate_mean_center(&points);
//! ```

pub mod buffer;
pub mod error;
pub mod network;
pub mod overlay;
pub mod proximity;
pub mod statistics;
pub mod surface;
pub mod transform;
pub mod validation;

// Re-export common types
pub use error::{AnalysisError, Result};

// Re-export key functions for convenience
pub use buffer::{buffer_point, buffer_line, buffer_polygon, BufferParams, CapStyle, JoinStyle};
pub use overlay::{union, intersection, difference, symmetric_difference, OverlayOp};
pub use proximity::{nearest_neighbor, k_nearest_neighbors, voronoi_diagram, NearestNeighbor};
pub use network::{Network, NetworkNode, NetworkEdge, ShortestPath, shortest_path_dijkstra, shortest_path_astar};
pub use surface::{Dem, slope, aspect, hillshade, contour};
pub use statistics::{
    point_pattern_analysis, calculate_mean_center, hot_spot_analysis, morans_i,
    k_means_clustering, dbscan_clustering, PointPatternStats, HotSpot, MoransI, Cluster,
};
pub use transform::{
    simplify_line_douglas_peucker, simplify_polygon_douglas_peucker, smooth_line_moving_average,
    smooth_line_chaikin, densify_line, densify_polygon, SimplificationAlgorithm, SmoothingAlgorithm,
};
pub use validation::{
    validate_line, validate_polygon, repair_polygon, clean_polygon, ValidationResult,
    ValidationIssue, IssueType, Severity,
};

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{Point, Polygon, LineString};

    #[test]
    fn test_basic_buffer() {
        let point = Point::new(0.0, 0.0);
        let params = BufferParams::new(10.0);
        let result = buffer_point(&point, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_basic_overlay() {
        let poly1 = Polygon::new(
            LineString::from(vec![
                (0.0, 0.0),
                (10.0, 0.0),
                (10.0, 10.0),
                (0.0, 10.0),
                (0.0, 0.0),
            ]),
            vec![],
        );
        let poly2 = Polygon::new(
            LineString::from(vec![
                (5.0, 5.0),
                (15.0, 5.0),
                (15.0, 15.0),
                (5.0, 15.0),
                (5.0, 5.0),
            ]),
            vec![],
        );

        let result = union(&poly1, &poly2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mean_center() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];

        let center = calculate_mean_center(&points);
        assert_eq!(center.x(), 5.0);
        assert_eq!(center.y(), 5.0);
    }

    #[test]
    fn test_validation() {
        let polygon = Polygon::new(
            LineString::from(vec![
                (0.0, 0.0),
                (10.0, 0.0),
                (10.0, 10.0),
                (0.0, 10.0),
                (0.0, 0.0),
            ]),
            vec![],
        );

        let result = validate_polygon(&polygon);
        assert!(result.is_valid || !result.has_errors());
    }

    #[test]
    fn test_simplification() {
        let line = LineString::from(vec![
            (0.0, 0.0),
            (1.0, 0.1),
            (2.0, -0.1),
            (3.0, 0.0),
            (10.0, 0.0),
        ]);

        let result = simplify_line_douglas_peucker(&line, 0.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_network_creation() {
        let mut network = Network::new();

        network.add_node(NetworkNode {
            id: 0,
            location: Point::new(0.0, 0.0),
            attributes: std::collections::HashMap::new(),
        });

        network.add_node(NetworkNode {
            id: 1,
            location: Point::new(10.0, 10.0),
            attributes: std::collections::HashMap::new(),
        });

        assert_eq!(network.node_count(), 2);
    }

    #[test]
    fn test_dem_creation() {
        let dem = Dem::new(10, 10, 1.0, 0.0, 10.0);
        assert_eq!(dem.width, 10);
        assert_eq!(dem.height, 10);
    }
}
