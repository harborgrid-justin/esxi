//! Geometry types with spatial indexing support.
//!
//! This module provides wrapper types around geo-types geometries with additional
//! functionality for coordinate transformations, spatial indexing, and CRS support.
//!
//! # Examples
//!
//! ```ignore
//! use meridian_core::geometry::{Point, LineString, Polygon};
//! use meridian_core::crs::Crs;
//!
//! // Create a point in WGS84
//! let mut point = Point::new(-122.4194, 37.7749, Crs::wgs84());
//!
//! // Transform to Web Mercator
//! point.transform_inplace(&Crs::web_mercator())?;
//! ```

use crate::bbox::BoundingBox;
use crate::crs::Crs;
use crate::error::Result;
use crate::traits::{Bounded, Transformable};
use geo::{Area, BoundingRect, Contains, EuclideanLength};
use geo_types::{
    Coord, LineString as GeoLineString,
    MultiLineString as GeoMultiLineString, MultiPoint as GeoMultiPoint,
    MultiPolygon as GeoMultiPolygon, Point as GeoPoint, Polygon as GeoPolygon,
};
use rstar::{PointDistance, RTreeObject, AABB};
use serde::{Deserialize, Serialize};

/// A point geometry with CRS support.
///
/// Represents a single location in space with x (longitude/easting) and
/// y (latitude/northing) coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// The underlying geo-types point
    pub geom: GeoPoint<f64>,
    /// Coordinate reference system
    pub crs: Crs,
}

impl Point {
    /// Creates a new point.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (longitude or easting)
    /// * `y` - Y coordinate (latitude or northing)
    /// * `crs` - Coordinate reference system
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use meridian_core::geometry::Point;
    /// use meridian_core::crs::Crs;
    ///
    /// let point = Point::new(-122.4194, 37.7749, Crs::wgs84());
    /// ```
    pub fn new(x: f64, y: f64, crs: Crs) -> Self {
        Self {
            geom: GeoPoint::new(x, y),
            crs,
        }
    }

    /// Returns the x coordinate.
    pub fn x(&self) -> f64 {
        self.geom.x()
    }

    /// Returns the y coordinate.
    pub fn y(&self) -> f64 {
        self.geom.y()
    }

    /// Returns the coordinate.
    pub fn coord(&self) -> Coord<f64> {
        self.geom.0
    }

    /// Creates a point from a geo-types Point and CRS.
    pub fn from_geo(geom: GeoPoint<f64>, crs: Crs) -> Self {
        Self { geom, crs }
    }
}

impl Bounded for Point {
    fn bounds(&self) -> BoundingBox {
        BoundingBox::new(self.x(), self.y(), self.x(), self.y())
    }
}

impl Transformable for Point {
    fn transform(&self, target_crs: &Crs) -> Result<Self> {
        let mut source_crs = self.crs.clone();
        let (x, y) = source_crs.transform_point(self.x(), self.y(), target_crs)?;
        Ok(Point::new(x, y, target_crs.clone()))
    }

    fn transform_inplace(&mut self, target_crs: &Crs) -> Result<()> {
        let (x, y) = self.crs.transform_point(self.x(), self.y(), target_crs)?;
        self.geom = GeoPoint::new(x, y);
        self.crs = target_crs.clone();
        Ok(())
    }
}

impl RTreeObject for Point {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.x(), self.y()])
    }
}

impl PointDistance for Point {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = self.x() - point[0];
        let dy = self.y() - point[1];
        dx * dx + dy * dy
    }
}

/// A collection of points with CRS support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiPoint {
    /// The underlying geo-types multipoint
    pub geom: GeoMultiPoint<f64>,
    /// Coordinate reference system
    pub crs: Crs,
}

impl MultiPoint {
    /// Creates a new multipoint from a vector of points.
    pub fn new(points: Vec<GeoPoint<f64>>, crs: Crs) -> Self {
        Self {
            geom: GeoMultiPoint::new(points),
            crs,
        }
    }

    /// Returns the number of points.
    pub fn len(&self) -> usize {
        self.geom.0.len()
    }

    /// Checks if the multipoint is empty.
    pub fn is_empty(&self) -> bool {
        self.geom.0.is_empty()
    }

    /// Returns an iterator over the points.
    pub fn iter(&self) -> impl Iterator<Item = &GeoPoint<f64>> {
        self.geom.0.iter()
    }
}

impl Bounded for MultiPoint {
    fn bounds(&self) -> BoundingBox {
        if let Some(rect) = self.geom.bounding_rect() {
            BoundingBox::from_rect(rect)
        } else {
            BoundingBox::new(0.0, 0.0, 0.0, 0.0)
        }
    }
}

impl Transformable for MultiPoint {
    fn transform(&self, target_crs: &Crs) -> Result<Self> {
        let mut source_crs = self.crs.clone();
        let transformed_points: Result<Vec<_>> = self
            .geom
            .0
            .iter()
            .map(|pt| {
                let (x, y) = source_crs.transform_point(pt.x(), pt.y(), target_crs)?;
                Ok(GeoPoint::new(x, y))
            })
            .collect();

        Ok(MultiPoint::new(transformed_points?, target_crs.clone()))
    }

    fn transform_inplace(&mut self, target_crs: &Crs) -> Result<()> {
        let transformed = self.transform(target_crs)?;
        *self = transformed;
        Ok(())
    }
}

/// A line string (polyline) with CRS support.
///
/// Represents a sequence of connected line segments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineString {
    /// The underlying geo-types linestring
    pub geom: GeoLineString<f64>,
    /// Coordinate reference system
    pub crs: Crs,
}

impl LineString {
    /// Creates a new linestring from a vector of coordinates.
    pub fn new(coords: Vec<Coord<f64>>, crs: Crs) -> Self {
        Self {
            geom: GeoLineString::new(coords),
            crs,
        }
    }

    /// Returns the number of coordinates.
    pub fn len(&self) -> usize {
        self.geom.0.len()
    }

    /// Checks if the linestring is empty.
    pub fn is_empty(&self) -> bool {
        self.geom.0.is_empty()
    }

    /// Returns the length of the linestring.
    pub fn length(&self) -> f64 {
        self.geom.euclidean_length()
    }

    /// Returns an iterator over the coordinates.
    pub fn coords(&self) -> impl Iterator<Item = &Coord<f64>> {
        self.geom.coords()
    }
}

impl Bounded for LineString {
    fn bounds(&self) -> BoundingBox {
        if let Some(rect) = self.geom.bounding_rect() {
            BoundingBox::from_rect(rect)
        } else {
            BoundingBox::new(0.0, 0.0, 0.0, 0.0)
        }
    }
}

impl Transformable for LineString {
    fn transform(&self, target_crs: &Crs) -> Result<Self> {
        let mut source_crs = self.crs.clone();
        let transformed_coords: Result<Vec<_>> = self
            .geom
            .coords()
            .map(|coord| {
                let (x, y) = source_crs.transform_point(coord.x, coord.y, target_crs)?;
                Ok(Coord { x, y })
            })
            .collect();

        Ok(LineString::new(transformed_coords?, target_crs.clone()))
    }

    fn transform_inplace(&mut self, target_crs: &Crs) -> Result<()> {
        let transformed = self.transform(target_crs)?;
        *self = transformed;
        Ok(())
    }
}

impl RTreeObject for LineString {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds().envelope()
    }
}

/// A collection of linestrings with CRS support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiLineString {
    /// The underlying geo-types multilinestring
    pub geom: GeoMultiLineString<f64>,
    /// Coordinate reference system
    pub crs: Crs,
}

impl MultiLineString {
    /// Creates a new multilinestring.
    pub fn new(lines: Vec<GeoLineString<f64>>, crs: Crs) -> Self {
        Self {
            geom: GeoMultiLineString::new(lines),
            crs,
        }
    }

    /// Returns the number of linestrings.
    pub fn len(&self) -> usize {
        self.geom.0.len()
    }

    /// Checks if the multilinestring is empty.
    pub fn is_empty(&self) -> bool {
        self.geom.0.is_empty()
    }

    /// Returns the total length of all linestrings.
    pub fn length(&self) -> f64 {
        self.geom.euclidean_length()
    }

    /// Returns an iterator over the linestrings.
    pub fn iter(&self) -> impl Iterator<Item = &GeoLineString<f64>> {
        self.geom.0.iter()
    }
}

impl Bounded for MultiLineString {
    fn bounds(&self) -> BoundingBox {
        if let Some(rect) = self.geom.bounding_rect() {
            BoundingBox::from_rect(rect)
        } else {
            BoundingBox::new(0.0, 0.0, 0.0, 0.0)
        }
    }
}

impl Transformable for MultiLineString {
    fn transform(&self, target_crs: &Crs) -> Result<Self> {
        let mut source_crs = self.crs.clone();
        let transformed_lines: Result<Vec<_>> = self
            .geom
            .0
            .iter()
            .map(|line| {
                let transformed_coords: Result<Vec<_>> = line
                    .coords()
                    .map(|coord| {
                        let (x, y) = source_crs.transform_point(coord.x, coord.y, target_crs)?;
                        Ok(Coord { x, y })
                    })
                    .collect();
                Ok(GeoLineString::new(transformed_coords?))
            })
            .collect();

        Ok(MultiLineString::new(transformed_lines?, target_crs.clone()))
    }

    fn transform_inplace(&mut self, target_crs: &Crs) -> Result<()> {
        let transformed = self.transform(target_crs)?;
        *self = transformed;
        Ok(())
    }
}

impl RTreeObject for MultiLineString {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds().envelope()
    }
}

/// A polygon with CRS support.
///
/// Represents a closed area with an exterior ring and optional holes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    /// The underlying geo-types polygon
    pub geom: GeoPolygon<f64>,
    /// Coordinate reference system
    pub crs: Crs,
}

impl Polygon {
    /// Creates a new polygon from an exterior ring and optional interior rings (holes).
    pub fn new(exterior: GeoLineString<f64>, interiors: Vec<GeoLineString<f64>>, crs: Crs) -> Self {
        Self {
            geom: GeoPolygon::new(exterior, interiors),
            crs,
        }
    }

    /// Returns the area of the polygon.
    pub fn area(&self) -> f64 {
        self.geom.unsigned_area()
    }

    /// Returns the exterior ring.
    pub fn exterior(&self) -> &GeoLineString<f64> {
        self.geom.exterior()
    }

    /// Returns the interior rings (holes).
    pub fn interiors(&self) -> &[GeoLineString<f64>] {
        self.geom.interiors()
    }

    /// Checks if the polygon contains a point.
    pub fn contains_point(&self, point: &GeoPoint<f64>) -> bool {
        self.geom.contains(point)
    }
}

impl Bounded for Polygon {
    fn bounds(&self) -> BoundingBox {
        if let Some(rect) = self.geom.bounding_rect() {
            BoundingBox::from_rect(rect)
        } else {
            BoundingBox::new(0.0, 0.0, 0.0, 0.0)
        }
    }
}

impl Transformable for Polygon {
    fn transform(&self, target_crs: &Crs) -> Result<Self> {
        let mut source_crs = self.crs.clone();

        // Transform exterior
        let exterior_coords: Result<Vec<_>> = self
            .geom
            .exterior()
            .coords()
            .map(|coord| {
                let (x, y) = source_crs.transform_point(coord.x, coord.y, target_crs)?;
                Ok(Coord { x, y })
            })
            .collect();
        let exterior = GeoLineString::new(exterior_coords?);

        // Transform interiors
        let interiors: Result<Vec<_>> = self
            .geom
            .interiors()
            .iter()
            .map(|interior| {
                let interior_coords: Result<Vec<_>> = interior
                    .coords()
                    .map(|coord| {
                        let (x, y) = source_crs.transform_point(coord.x, coord.y, target_crs)?;
                        Ok(Coord { x, y })
                    })
                    .collect();
                Ok(GeoLineString::new(interior_coords?))
            })
            .collect();

        Ok(Polygon::new(exterior, interiors?, target_crs.clone()))
    }

    fn transform_inplace(&mut self, target_crs: &Crs) -> Result<()> {
        let transformed = self.transform(target_crs)?;
        *self = transformed;
        Ok(())
    }
}

impl RTreeObject for Polygon {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds().envelope()
    }
}

/// A collection of polygons with CRS support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiPolygon {
    /// The underlying geo-types multipolygon
    pub geom: GeoMultiPolygon<f64>,
    /// Coordinate reference system
    pub crs: Crs,
}

impl MultiPolygon {
    /// Creates a new multipolygon.
    pub fn new(polygons: Vec<GeoPolygon<f64>>, crs: Crs) -> Self {
        Self {
            geom: GeoMultiPolygon::new(polygons),
            crs,
        }
    }

    /// Returns the number of polygons.
    pub fn len(&self) -> usize {
        self.geom.0.len()
    }

    /// Checks if the multipolygon is empty.
    pub fn is_empty(&self) -> bool {
        self.geom.0.is_empty()
    }

    /// Returns the total area of all polygons.
    pub fn area(&self) -> f64 {
        self.geom.unsigned_area()
    }

    /// Returns an iterator over the polygons.
    pub fn iter(&self) -> impl Iterator<Item = &GeoPolygon<f64>> {
        self.geom.0.iter()
    }
}

impl Bounded for MultiPolygon {
    fn bounds(&self) -> BoundingBox {
        if let Some(rect) = self.geom.bounding_rect() {
            BoundingBox::from_rect(rect)
        } else {
            BoundingBox::new(0.0, 0.0, 0.0, 0.0)
        }
    }
}

impl Transformable for MultiPolygon {
    fn transform(&self, target_crs: &Crs) -> Result<Self> {
        let mut source_crs = self.crs.clone();
        let transformed_polygons: Result<Vec<_>> = self
            .geom
            .0
            .iter()
            .map(|poly| {
                // Transform exterior
                let exterior_coords: Result<Vec<_>> = poly
                    .exterior()
                    .coords()
                    .map(|coord| {
                        let (x, y) = source_crs.transform_point(coord.x, coord.y, target_crs)?;
                        Ok(Coord { x, y })
                    })
                    .collect();
                let exterior = GeoLineString::new(exterior_coords?);

                // Transform interiors
                let interiors: Result<Vec<_>> = poly
                    .interiors()
                    .iter()
                    .map(|interior| {
                        let interior_coords: Result<Vec<_>> = interior
                            .coords()
                            .map(|coord| {
                                let (x, y) = source_crs.transform_point(coord.x, coord.y, target_crs)?;
                                Ok(Coord { x, y })
                            })
                            .collect();
                        Ok(GeoLineString::new(interior_coords?))
                    })
                    .collect();

                Ok(GeoPolygon::new(exterior, interiors?))
            })
            .collect();

        Ok(MultiPolygon::new(transformed_polygons?, target_crs.clone()))
    }

    fn transform_inplace(&mut self, target_crs: &Crs) -> Result<()> {
        let transformed = self.transform(target_crs)?;
        *self = transformed;
        Ok(())
    }
}

impl RTreeObject for MultiPolygon {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.bounds().envelope()
    }
}

/// A collection of heterogeneous geometries with CRS support.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeometryCollection {
    /// The geometries in the collection
    pub geometries: Vec<Geometry>,
    /// Coordinate reference system
    pub crs: Crs,
}

impl GeometryCollection {
    /// Creates a new geometry collection.
    pub fn new(geometries: Vec<Geometry>, crs: Crs) -> Self {
        Self { geometries, crs }
    }

    /// Returns the number of geometries.
    pub fn len(&self) -> usize {
        self.geometries.len()
    }

    /// Checks if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.geometries.is_empty()
    }

    /// Returns an iterator over the geometries.
    pub fn iter(&self) -> impl Iterator<Item = &Geometry> {
        self.geometries.iter()
    }
}

impl Bounded for GeometryCollection {
    fn bounds(&self) -> BoundingBox {
        if self.geometries.is_empty() {
            return BoundingBox::new(0.0, 0.0, 0.0, 0.0);
        }

        let mut bbox = self.geometries[0].bounds();
        for geom in &self.geometries[1..] {
            bbox.expand_to_include_bbox(&geom.bounds());
        }
        bbox
    }
}

/// An enum representing any geometry type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Geometry {
    /// A single point geometry
    Point(Point),
    /// A collection of points
    MultiPoint(MultiPoint),
    /// A line string (polyline) geometry
    LineString(LineString),
    /// A collection of line strings
    MultiLineString(MultiLineString),
    /// A polygon geometry
    Polygon(Polygon),
    /// A collection of polygons
    MultiPolygon(MultiPolygon),
    /// A collection of mixed geometry types
    GeometryCollection(GeometryCollection),
}

impl Bounded for Geometry {
    fn bounds(&self) -> BoundingBox {
        match self {
            Geometry::Point(g) => g.bounds(),
            Geometry::MultiPoint(g) => g.bounds(),
            Geometry::LineString(g) => g.bounds(),
            Geometry::MultiLineString(g) => g.bounds(),
            Geometry::Polygon(g) => g.bounds(),
            Geometry::MultiPolygon(g) => g.bounds(),
            Geometry::GeometryCollection(g) => g.bounds(),
        }
    }
}

impl Transformable for Geometry {
    fn transform(&self, target_crs: &Crs) -> Result<Self> {
        Ok(match self {
            Geometry::Point(g) => Geometry::Point(g.transform(target_crs)?),
            Geometry::MultiPoint(g) => Geometry::MultiPoint(g.transform(target_crs)?),
            Geometry::LineString(g) => Geometry::LineString(g.transform(target_crs)?),
            Geometry::MultiLineString(g) => Geometry::MultiLineString(g.transform(target_crs)?),
            Geometry::Polygon(g) => Geometry::Polygon(g.transform(target_crs)?),
            Geometry::MultiPolygon(g) => Geometry::MultiPolygon(g.transform(target_crs)?),
            Geometry::GeometryCollection(gc) => {
                let transformed: Result<Vec<_>> = gc
                    .geometries
                    .iter()
                    .map(|g| g.transform(target_crs))
                    .collect();
                Geometry::GeometryCollection(GeometryCollection::new(
                    transformed?,
                    target_crs.clone(),
                ))
            }
        })
    }

    fn transform_inplace(&mut self, target_crs: &Crs) -> Result<()> {
        match self {
            Geometry::Point(g) => g.transform_inplace(target_crs),
            Geometry::MultiPoint(g) => g.transform_inplace(target_crs),
            Geometry::LineString(g) => g.transform_inplace(target_crs),
            Geometry::MultiLineString(g) => g.transform_inplace(target_crs),
            Geometry::Polygon(g) => g.transform_inplace(target_crs),
            Geometry::MultiPolygon(g) => g.transform_inplace(target_crs),
            Geometry::GeometryCollection(gc) => {
                for geom in &mut gc.geometries {
                    geom.transform_inplace(target_crs)?;
                }
                gc.crs = target_crs.clone();
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let point = Point::new(-122.4194, 37.7749, Crs::wgs84());
        assert_eq!(point.x(), -122.4194);
        assert_eq!(point.y(), 37.7749);
    }

    #[test]
    fn test_point_bounds() {
        let point = Point::new(10.0, 20.0, Crs::wgs84());
        let bbox = point.bounds();
        assert_eq!(bbox.min_x, 10.0);
        assert_eq!(bbox.max_x, 10.0);
        assert_eq!(bbox.min_y, 20.0);
        assert_eq!(bbox.max_y, 20.0);
    }

    #[test]
    fn test_linestring_length() {
        let coords = vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 3.0, y: 0.0 },
            Coord { x: 3.0, y: 4.0 },
        ];
        let line = LineString::new(coords, Crs::wgs84());
        assert_eq!(line.length(), 7.0); // 3 + 4
    }

    #[test]
    fn test_polygon_area() {
        let exterior = GeoLineString::new(vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 10.0, y: 0.0 },
            Coord { x: 10.0, y: 10.0 },
            Coord { x: 0.0, y: 10.0 },
            Coord { x: 0.0, y: 0.0 },
        ]);
        let polygon = Polygon::new(exterior, vec![], Crs::wgs84());
        assert_eq!(polygon.area(), 100.0);
    }
}
