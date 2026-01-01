//! # Meridian Core
//!
//! Core geometry and spatial primitives for the Meridian GIS Platform.
//!
//! This crate provides the foundational data structures and operations for working
//! with geospatial data, including:
//!
//! - **Geometry types**: Point, LineString, Polygon, and their Multi* variants
//! - **Coordinate Reference Systems (CRS)**: Support for transformations between different projections
//! - **Spatial indexing**: R-tree based spatial index for fast queries
//! - **Features and Layers**: GeoJSON-style features with properties and layer management
//! - **Bounding boxes**: Efficient spatial extent calculations
//!
//! ## Quick Start
//!
//! ```ignore
//! use meridian_core::prelude::*;
//!
//! // Create a point in WGS84
//! let point = Point::new(-122.4194, 37.7749, Crs::wgs84());
//!
//! // Create a feature with properties
//! let mut feature = Feature::new(Geometry::Point(point));
//! feature.set_property("name", json!("San Francisco"));
//! feature.set_property("population", json!(883305));
//!
//! // Add to a layer
//! let mut layer = Layer::new("cities", Crs::wgs84());
//! layer.add_feature(feature);
//!
//! // Build spatial index for fast queries
//! layer.build_index();
//!
//! // Query nearest feature
//! let nearest = layer.query_nearest(&[-122.4, 37.8]);
//! ```
//!
//! ## Coordinate Reference Systems
//!
//! Meridian Core supports various coordinate reference systems and transformations:
//!
//! ```ignore
//! use meridian_core::crs::Crs;
//! use meridian_core::geometry::Point;
//! use meridian_core::traits::Transformable;
//!
//! // Create a point in WGS84 (latitude/longitude)
//! let mut point = Point::new(-122.4194, 37.7749, Crs::wgs84());
//!
//! // Transform to Web Mercator (commonly used in web maps)
//! point.transform_inplace(&Crs::web_mercator())?;
//!
//! // Transform to UTM Zone 10N (for Northern California)
//! point.transform_inplace(&Crs::utm(10, true))?;
//! ```
//!
//! ## Spatial Queries
//!
//! The spatial index enables efficient spatial queries:
//!
//! ```ignore
//! use meridian_core::bbox::BoundingBox;
//!
//! // Query by bounding box
//! let bbox = BoundingBox::new(-122.5, 37.7, -122.3, 37.9);
//! let results = layer.query_bbox(&bbox);
//!
//! // Find nearest neighbor
//! let nearest = layer.query_nearest(&[-122.4, 37.8]);
//!
//! // Find k nearest neighbors
//! let k_nearest = layer.query_k_nearest(&[-122.4, 37.8], 5);
//!
//! // Find all features within distance
//! let within = layer.query_within_distance(&[-122.4, 37.8], 0.1);
//! ```
//!
//! ## Feature Properties
//!
//! Features support arbitrary JSON properties:
//!
//! ```ignore
//! use serde_json::json;
//!
//! let mut feature = Feature::new(geometry);
//!
//! // Set properties
//! feature.set_property("name", json!("Golden Gate Bridge"));
//! feature.set_property("length_meters", json!(2737));
//! feature.set_property("opened", json!(1937));
//!
//! // Get properties
//! let name: String = feature.get_property_as("name")?;
//! let length: i64 = feature.get_property_as("length_meters")?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

// Re-export commonly used types from dependencies
pub use geo;
pub use geo_types;
#[cfg(feature = "proj-transform")]
pub use proj;
pub use rstar;
pub use serde_json;

// Public modules
pub mod bbox;
pub mod crs;
pub mod error;
pub mod feature;
pub mod geometry;
pub mod layer;
pub mod spatial_index;
pub mod traits;

// Prelude for convenient imports
pub mod prelude {
    //! Prelude module for convenient imports.
    //!
    //! Import everything from the prelude to get started quickly:
    //!
    //! ```ignore
    //! use meridian_core::prelude::*;
    //! ```

    pub use crate::bbox::BoundingBox;
    pub use crate::crs::Crs;
    pub use crate::error::{MeridianError, Result};
    pub use crate::feature::{Feature, FeatureBuilder};
    pub use crate::geometry::{
        Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
        Point, Polygon,
    };
    pub use crate::layer::Layer;
    pub use crate::spatial_index::SpatialIndex;
    pub use crate::traits::{Bounded, Spatial, Transformable};
    pub use geo_types::{Coord, LineString as GeoLineString, Point as GeoPoint, Polygon as GeoPolygon};
    pub use serde_json::json;
}

// Version information
/// The version of the Meridian Core library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
