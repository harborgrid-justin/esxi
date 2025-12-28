//! Bounding box implementations with R-tree integration.
//!
//! This module provides axis-aligned bounding box functionality with support
//! for spatial indexing through R-tree integration.

use geo_types::{Coord, CoordFloat, Rect};
use rstar::{RTreeObject, AABB};
use serde::{Deserialize, Serialize};
use std::fmt;

/// An axis-aligned bounding box in 2D space.
///
/// A bounding box is defined by its minimum and maximum x and y coordinates.
/// It represents the smallest rectangle that contains a geometry.
///
/// # Examples
///
/// ```ignore
/// use meridian_core::bbox::BoundingBox;
///
/// let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
/// assert_eq!(bbox.width(), 10.0);
/// assert_eq!(bbox.height(), 10.0);
/// assert_eq!(bbox.area(), 100.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum x coordinate (west/left)
    pub min_x: f64,
    /// Minimum y coordinate (south/bottom)
    pub min_y: f64,
    /// Maximum x coordinate (east/right)
    pub max_x: f64,
    /// Maximum y coordinate (north/top)
    pub max_y: f64,
}

impl BoundingBox {
    /// Creates a new bounding box.
    ///
    /// # Arguments
    ///
    /// * `min_x` - Minimum x coordinate
    /// * `min_y` - Minimum y coordinate
    /// * `max_x` - Maximum x coordinate
    /// * `max_y` - Maximum y coordinate
    ///
    /// # Returns
    ///
    /// A new `BoundingBox` instance
    ///
    /// # Panics
    ///
    /// Panics if min values are greater than max values or if any coordinate is NaN.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let bbox = BoundingBox::new(-180.0, -90.0, 180.0, 90.0);
    /// ```
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        assert!(min_x <= max_x, "min_x must be <= max_x");
        assert!(min_y <= max_y, "min_y must be <= max_y");
        assert!(!min_x.is_nan() && !min_y.is_nan() && !max_x.is_nan() && !max_y.is_nan(),
                "Coordinates cannot be NaN");

        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Creates a bounding box from two corner coordinates.
    ///
    /// The coordinates will be ordered correctly regardless of which corner is provided.
    ///
    /// # Arguments
    ///
    /// * `c1` - First corner coordinate
    /// * `c2` - Second corner coordinate (opposite corner)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use geo_types::Coord;
    /// use meridian_core::bbox::BoundingBox;
    ///
    /// let bbox = BoundingBox::from_corners(
    ///     Coord { x: 10.0, y: 20.0 },
    ///     Coord { x: 0.0, y: 0.0 }
    /// );
    /// assert_eq!(bbox.min_x, 0.0);
    /// assert_eq!(bbox.max_x, 10.0);
    /// ```
    pub fn from_corners(c1: Coord<f64>, c2: Coord<f64>) -> Self {
        Self::new(
            c1.x.min(c2.x),
            c1.y.min(c2.y),
            c1.x.max(c2.x),
            c1.y.max(c2.y),
        )
    }

    /// Creates a bounding box from a geo-types Rect.
    ///
    /// # Arguments
    ///
    /// * `rect` - A geo-types Rect
    pub fn from_rect<T: CoordFloat>(rect: Rect<T>) -> Self {
        Self::new(
            rect.min().x.to_f64().unwrap(),
            rect.min().y.to_f64().unwrap(),
            rect.max().x.to_f64().unwrap(),
            rect.max().y.to_f64().unwrap(),
        )
    }

    /// Converts this bounding box to a geo-types Rect.
    pub fn to_rect(&self) -> Rect<f64> {
        Rect::new(
            Coord { x: self.min_x, y: self.min_y },
            Coord { x: self.max_x, y: self.max_y },
        )
    }

    /// Returns the width of the bounding box.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let bbox = BoundingBox::new(0.0, 0.0, 10.0, 5.0);
    /// assert_eq!(bbox.width(), 10.0);
    /// ```
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Returns the height of the bounding box.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let bbox = BoundingBox::new(0.0, 0.0, 10.0, 5.0);
    /// assert_eq!(bbox.height(), 5.0);
    /// ```
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Returns the area of the bounding box.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let bbox = BoundingBox::new(0.0, 0.0, 10.0, 5.0);
    /// assert_eq!(bbox.area(), 50.0);
    /// ```
    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }

    /// Returns the center point of the bounding box.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
    /// let center = bbox.center();
    /// assert_eq!(center.x, 5.0);
    /// assert_eq!(center.y, 5.0);
    /// ```
    pub fn center(&self) -> Coord<f64> {
        Coord {
            x: (self.min_x + self.max_x) / 2.0,
            y: (self.min_y + self.max_y) / 2.0,
        }
    }

    /// Checks if this bounding box contains a point.
    ///
    /// # Arguments
    ///
    /// * `coord` - The coordinate to check
    ///
    /// # Returns
    ///
    /// `true` if the point is within or on the boundary of the box
    pub fn contains_point(&self, coord: &Coord<f64>) -> bool {
        coord.x >= self.min_x &&
        coord.x <= self.max_x &&
        coord.y >= self.min_y &&
        coord.y <= self.max_y
    }

    /// Checks if this bounding box completely contains another bounding box.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bounding box to check
    ///
    /// # Returns
    ///
    /// `true` if the other box is completely within this box
    pub fn contains_bbox(&self, other: &BoundingBox) -> bool {
        other.min_x >= self.min_x &&
        other.max_x <= self.max_x &&
        other.min_y >= self.min_y &&
        other.max_y <= self.max_y
    }

    /// Checks if this bounding box intersects with another bounding box.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bounding box to check
    ///
    /// # Returns
    ///
    /// `true` if the boxes overlap or touch
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min_x <= other.max_x &&
        self.max_x >= other.min_x &&
        self.min_y <= other.max_y &&
        self.max_y >= other.min_y
    }

    /// Expands this bounding box to include a point.
    ///
    /// # Arguments
    ///
    /// * `coord` - The coordinate to include
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
    /// bbox.expand_to_include_point(&Coord { x: 15.0, y: 5.0 });
    /// assert_eq!(bbox.max_x, 15.0);
    /// ```
    pub fn expand_to_include_point(&mut self, coord: &Coord<f64>) {
        self.min_x = self.min_x.min(coord.x);
        self.min_y = self.min_y.min(coord.y);
        self.max_x = self.max_x.max(coord.x);
        self.max_y = self.max_y.max(coord.y);
    }

    /// Expands this bounding box to include another bounding box.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bounding box to include
    pub fn expand_to_include_bbox(&mut self, other: &BoundingBox) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
    }

    /// Creates a new bounding box that is the union of this box and another.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bounding box
    ///
    /// # Returns
    ///
    /// A new bounding box that contains both input boxes
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox::new(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
        )
    }

    /// Creates a new bounding box that is the intersection of this box and another.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bounding box
    ///
    /// # Returns
    ///
    /// Some bounding box if the boxes intersect, None otherwise
    pub fn intersection(&self, other: &BoundingBox) -> Option<BoundingBox> {
        if !self.intersects(other) {
            return None;
        }

        Some(BoundingBox::new(
            self.min_x.max(other.min_x),
            self.min_y.max(other.min_y),
            self.max_x.min(other.max_x),
            self.max_y.min(other.max_y),
        ))
    }

    /// Expands the bounding box by a fixed amount in all directions.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount to expand in each direction
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
    /// let expanded = bbox.buffer(5.0);
    /// assert_eq!(expanded.min_x, -5.0);
    /// assert_eq!(expanded.max_x, 15.0);
    /// ```
    pub fn buffer(&self, amount: f64) -> BoundingBox {
        BoundingBox::new(
            self.min_x - amount,
            self.min_y - amount,
            self.max_x + amount,
            self.max_y + amount,
        )
    }

    /// Returns the four corner coordinates of the bounding box.
    ///
    /// # Returns
    ///
    /// Array of coordinates: [bottom-left, bottom-right, top-right, top-left]
    pub fn corners(&self) -> [Coord<f64>; 4] {
        [
            Coord { x: self.min_x, y: self.min_y }, // bottom-left
            Coord { x: self.max_x, y: self.min_y }, // bottom-right
            Coord { x: self.max_x, y: self.max_y }, // top-right
            Coord { x: self.min_x, y: self.max_y }, // top-left
        ]
    }
}

// R-tree integration
impl RTreeObject for BoundingBox {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners([self.min_x, self.min_y], [self.max_x, self.max_y])
    }
}

impl fmt::Display for BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BBox[({}, {}) -> ({}, {})]",
            self.min_x, self.min_y, self.max_x, self.max_y
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox_creation() {
        let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        assert_eq!(bbox.min_x, 0.0);
        assert_eq!(bbox.max_x, 10.0);
    }

    #[test]
    #[should_panic]
    fn test_bbox_invalid_order() {
        BoundingBox::new(10.0, 0.0, 0.0, 10.0);
    }

    #[test]
    fn test_bbox_dimensions() {
        let bbox = BoundingBox::new(0.0, 0.0, 10.0, 5.0);
        assert_eq!(bbox.width(), 10.0);
        assert_eq!(bbox.height(), 5.0);
        assert_eq!(bbox.area(), 50.0);
    }

    #[test]
    fn test_bbox_center() {
        let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        let center = bbox.center();
        assert_eq!(center.x, 5.0);
        assert_eq!(center.y, 5.0);
    }

    #[test]
    fn test_bbox_contains_point() {
        let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        assert!(bbox.contains_point(&Coord { x: 5.0, y: 5.0 }));
        assert!(bbox.contains_point(&Coord { x: 0.0, y: 0.0 }));
        assert!(!bbox.contains_point(&Coord { x: 11.0, y: 5.0 }));
    }

    #[test]
    fn test_bbox_intersects() {
        let bbox1 = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        let bbox2 = BoundingBox::new(5.0, 5.0, 15.0, 15.0);
        let bbox3 = BoundingBox::new(20.0, 20.0, 30.0, 30.0);

        assert!(bbox1.intersects(&bbox2));
        assert!(!bbox1.intersects(&bbox3));
    }

    #[test]
    fn test_bbox_union() {
        let bbox1 = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        let bbox2 = BoundingBox::new(5.0, 5.0, 15.0, 15.0);
        let union = bbox1.union(&bbox2);

        assert_eq!(union.min_x, 0.0);
        assert_eq!(union.min_y, 0.0);
        assert_eq!(union.max_x, 15.0);
        assert_eq!(union.max_y, 15.0);
    }

    #[test]
    fn test_bbox_intersection() {
        let bbox1 = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        let bbox2 = BoundingBox::new(5.0, 5.0, 15.0, 15.0);
        let intersection = bbox1.intersection(&bbox2).unwrap();

        assert_eq!(intersection.min_x, 5.0);
        assert_eq!(intersection.min_y, 5.0);
        assert_eq!(intersection.max_x, 10.0);
        assert_eq!(intersection.max_y, 10.0);
    }

    #[test]
    fn test_bbox_buffer() {
        let bbox = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        let buffered = bbox.buffer(5.0);

        assert_eq!(buffered.min_x, -5.0);
        assert_eq!(buffered.min_y, -5.0);
        assert_eq!(buffered.max_x, 15.0);
        assert_eq!(buffered.max_y, 15.0);
    }
}
