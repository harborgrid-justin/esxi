//! Core traits for spatial operations.
//!
//! This module defines fundamental traits that enable spatial operations,
//! coordinate transformations, and bounding box calculations across different
//! geometry types in the Meridian GIS platform.

use crate::bbox::BoundingBox;
use crate::crs::Crs;
use crate::error::Result;
use geo_types::CoordFloat;

/// Trait for types that have spatial extent and can calculate bounding boxes.
///
/// This trait is fundamental for spatial operations, enabling efficient spatial
/// queries and indexing through bounding box calculations.
///
/// # Examples
///
/// ```ignore
/// use meridian_core::traits::Bounded;
/// use meridian_core::geometry::Point;
///
/// let point = Point::new(10.0, 20.0);
/// let bbox = point.bounds();
/// ```
pub trait Bounded {
    /// Returns the bounding box that encompasses this spatial object.
    ///
    /// The bounding box is the smallest axis-aligned rectangle that contains
    /// all points in the geometry.
    fn bounds(&self) -> BoundingBox;

    /// Checks if this object's bounds intersect with another bounded object.
    ///
    /// # Arguments
    ///
    /// * `other` - Another bounded object to check intersection with
    ///
    /// # Returns
    ///
    /// `true` if the bounding boxes intersect, `false` otherwise
    fn intersects_bounds<T: Bounded>(&self, other: &T) -> bool {
        self.bounds().intersects(&other.bounds())
    }

    /// Checks if this object's bounds are completely contained within another.
    ///
    /// # Arguments
    ///
    /// * `other` - Another bounded object to check containment within
    ///
    /// # Returns
    ///
    /// `true` if this object's bounds are within the other's bounds
    fn within_bounds<T: Bounded>(&self, other: &T) -> bool {
        other.bounds().contains_bbox(&self.bounds())
    }
}

/// Trait for geometries that can be transformed between coordinate reference systems.
///
/// This trait enables coordinate transformations, which are essential for working
/// with spatial data from different sources and projections.
///
/// # Examples
///
/// ```ignore
/// use meridian_core::traits::Transformable;
/// use meridian_core::crs::Crs;
/// use meridian_core::geometry::Point;
///
/// let point = Point::new(-122.4194, 37.7749); // San Francisco in WGS84
/// let web_mercator = Crs::web_mercator();
/// let transformed = point.transform(&web_mercator)?;
/// ```
pub trait Transformable {
    /// Transforms this geometry to a different coordinate reference system.
    ///
    /// # Arguments
    ///
    /// * `target_crs` - The target coordinate reference system
    ///
    /// # Returns
    ///
    /// A new geometry in the target CRS, or an error if transformation fails
    ///
    /// # Errors
    ///
    /// Returns `MeridianError::TransformError` if the transformation cannot be performed
    fn transform(&self, target_crs: &Crs) -> Result<Self>
    where
        Self: Sized;

    /// Transforms this geometry in place to a different coordinate reference system.
    ///
    /// # Arguments
    ///
    /// * `target_crs` - The target coordinate reference system
    ///
    /// # Errors
    ///
    /// Returns `MeridianError::TransformError` if the transformation cannot be performed
    fn transform_inplace(&mut self, target_crs: &Crs) -> Result<()>;
}

/// Trait for geometries that support spatial relationships and operations.
///
/// This trait provides fundamental spatial predicates and measurements that are
/// common across different geometry types.
pub trait Spatial<T: CoordFloat = f64> {
    /// Calculates the area of this geometry.
    ///
    /// For point and line geometries, returns 0.0.
    /// For polygonal geometries, returns the area in square units of the CRS.
    fn area(&self) -> T;

    /// Calculates the perimeter or length of this geometry.
    ///
    /// - For polygons: returns the perimeter
    /// - For lines: returns the length
    /// - For points: returns 0.0
    fn length(&self) -> T;

    /// Checks if this geometry contains another geometry.
    ///
    /// # Arguments
    ///
    /// * `other` - The geometry to check for containment
    ///
    /// # Returns
    ///
    /// `true` if this geometry completely contains the other geometry
    fn contains(&self, other: &Self) -> bool;

    /// Checks if this geometry intersects with another geometry.
    ///
    /// # Arguments
    ///
    /// * `other` - The geometry to check for intersection
    ///
    /// # Returns
    ///
    /// `true` if the geometries share at least one point
    fn intersects(&self, other: &Self) -> bool;

    /// Calculates the distance to another geometry.
    ///
    /// # Arguments
    ///
    /// * `other` - The geometry to measure distance to
    ///
    /// # Returns
    ///
    /// The minimum distance between any two points in the geometries
    fn distance(&self, other: &Self) -> T;
}

/// Trait for geometries that can be simplified while preserving topology.
///
/// Simplification is important for rendering performance and data reduction.
pub trait Simplifiable {
    /// Simplifies the geometry using the Douglas-Peucker algorithm.
    ///
    /// # Arguments
    ///
    /// * `epsilon` - The tolerance parameter for simplification
    ///
    /// # Returns
    ///
    /// A simplified version of the geometry
    fn simplify(&self, epsilon: f64) -> Self
    where
        Self: Sized;

    /// Simplifies the geometry while preserving topology.
    ///
    /// # Arguments
    ///
    /// * `epsilon` - The tolerance parameter for simplification
    ///
    /// # Returns
    ///
    /// A simplified version of the geometry that maintains topological relationships
    fn simplify_preserve_topology(&self, epsilon: f64) -> Self
    where
        Self: Sized;
}

/// Trait for geometries that can be validated for correctness.
///
/// Validation ensures that geometries conform to OGC Simple Features standards.
pub trait Validatable {
    /// Checks if the geometry is valid according to OGC standards.
    ///
    /// # Returns
    ///
    /// `true` if the geometry is valid, `false` otherwise
    fn is_valid(&self) -> bool;

    /// Returns a detailed validation error message if the geometry is invalid.
    ///
    /// # Returns
    ///
    /// `None` if valid, or a description of the validation error
    fn validation_error(&self) -> Option<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traits_compile() {
        // This test ensures that the traits are properly defined and compile
        // Actual implementations will be tested in the geometry module
    }
}
