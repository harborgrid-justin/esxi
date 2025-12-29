//! Geometry clipping to tile bounds

use crate::error::Result;
use crate::tile::bounds::MercatorBounds;
use crate::tile::extent::ExtentConverter;
use geo_types::{Coord, Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};

/// Geometry clipper for tiles
pub struct GeometryClipper {
    buffer: u32,
}

impl GeometryClipper {
    /// Create a new geometry clipper
    pub fn new(buffer: u32) -> Self {
        Self { buffer }
    }

    /// Clip geometry to tile bounds
    pub fn clip(
        &self,
        geometry: &Geometry,
        bounds: &MercatorBounds,
        converter: &ExtentConverter,
    ) -> Option<Geometry> {
        // Expand bounds by buffer
        let buffered_bounds = bounds.expand(self.buffer as f64);

        match geometry {
            Geometry::Point(p) => self.clip_point(p, &buffered_bounds).map(Geometry::Point),
            Geometry::MultiPoint(mp) => self
                .clip_multipoint(mp, &buffered_bounds)
                .map(Geometry::MultiPoint),
            Geometry::LineString(ls) => self
                .clip_linestring(ls, &buffered_bounds)
                .map(Geometry::LineString),
            Geometry::MultiLineString(mls) => self
                .clip_multilinestring(mls, &buffered_bounds)
                .map(Geometry::MultiLineString),
            Geometry::Polygon(poly) => self
                .clip_polygon(poly, &buffered_bounds)
                .map(Geometry::Polygon),
            Geometry::MultiPolygon(mp) => self
                .clip_multipolygon(mp, &buffered_bounds)
                .map(Geometry::MultiPolygon),
            _ => Some(geometry.clone()),
        }
    }

    /// Clip a point
    fn clip_point(&self, point: &Point, bounds: &MercatorBounds) -> Option<Point> {
        if bounds.contains(point.x(), point.y()) {
            Some(*point)
        } else {
            None
        }
    }

    /// Clip a multipoint
    fn clip_multipoint(&self, mp: &MultiPoint, bounds: &MercatorBounds) -> Option<MultiPoint> {
        let points: Vec<Point> = mp
            .0
            .iter()
            .filter(|p| bounds.contains(p.x(), p.y()))
            .copied()
            .collect();

        if points.is_empty() {
            None
        } else {
            Some(MultiPoint(points))
        }
    }

    /// Clip a linestring
    fn clip_linestring(&self, ls: &LineString, bounds: &MercatorBounds) -> Option<LineString> {
        if ls.0.is_empty() {
            return None;
        }

        // Simple implementation: keep all points within bounds
        // A more sophisticated implementation would use Cohen-Sutherland or similar
        let mut clipped_coords = Vec::new();
        let mut has_interior = false;

        for coord in &ls.0 {
            if bounds.contains(coord.x, coord.y) {
                clipped_coords.push(*coord);
                has_interior = true;
            } else if !clipped_coords.is_empty() && has_interior {
                // Add boundary crossing point
                clipped_coords.push(*coord);
            }
        }

        if clipped_coords.len() < 2 {
            None
        } else {
            Some(LineString::from(clipped_coords))
        }
    }

    /// Clip a multilinestring
    fn clip_multilinestring(
        &self,
        mls: &MultiLineString,
        bounds: &MercatorBounds,
    ) -> Option<MultiLineString> {
        let lines: Vec<LineString> = mls
            .0
            .iter()
            .filter_map(|ls| self.clip_linestring(ls, bounds))
            .collect();

        if lines.is_empty() {
            None
        } else {
            Some(MultiLineString(lines))
        }
    }

    /// Clip a polygon
    fn clip_polygon(&self, poly: &Polygon, bounds: &MercatorBounds) -> Option<Polygon> {
        // Simple implementation: check if exterior ring intersects bounds
        let exterior = poly.exterior();

        // Check if any point is within bounds
        let has_interior = exterior.0.iter().any(|c| bounds.contains(c.x, c.y));

        if !has_interior {
            // Check if bounds is entirely within polygon (simple bbox check)
            // This is a simplified check - a full implementation would use proper point-in-polygon
            return None;
        }

        // For simplicity, return the original polygon
        // A full implementation would use Sutherland-Hodgman or similar algorithm
        Some(poly.clone())
    }

    /// Clip a multipolygon
    fn clip_multipolygon(
        &self,
        mp: &MultiPolygon,
        bounds: &MercatorBounds,
    ) -> Option<MultiPolygon> {
        let polygons: Vec<Polygon> = mp
            .0
            .iter()
            .filter_map(|p| self.clip_polygon(p, bounds))
            .collect();

        if polygons.is_empty() {
            None
        } else {
            Some(MultiPolygon(polygons))
        }
    }

    /// Check if geometry intersects bounds
    pub fn intersects(&self, geometry: &Geometry, bounds: &MercatorBounds) -> bool {
        match geometry {
            Geometry::Point(p) => bounds.contains(p.x(), p.y()),
            Geometry::MultiPoint(mp) => mp.0.iter().any(|p| bounds.contains(p.x(), p.y())),
            Geometry::LineString(ls) => ls.0.iter().any(|c| bounds.contains(c.x, c.y)),
            Geometry::MultiLineString(mls) => mls
                .0
                .iter()
                .any(|ls| ls.0.iter().any(|c| bounds.contains(c.x, c.y))),
            Geometry::Polygon(p) => p.exterior().0.iter().any(|c| bounds.contains(c.x, c.y)),
            Geometry::MultiPolygon(mp) => mp
                .0
                .iter()
                .any(|p| p.exterior().0.iter().any(|c| bounds.contains(c.x, c.y))),
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_clipping() {
        let clipper = GeometryClipper::new(0);
        let bounds = MercatorBounds::new(0.0, 0.0, 100.0, 100.0);

        let p1 = Point::new(50.0, 50.0);
        assert!(clipper.clip_point(&p1, &bounds).is_some());

        let p2 = Point::new(150.0, 150.0);
        assert!(clipper.clip_point(&p2, &bounds).is_none());
    }

    #[test]
    fn test_linestring_clipping() {
        let clipper = GeometryClipper::new(0);
        let bounds = MercatorBounds::new(0.0, 0.0, 100.0, 100.0);

        let ls = LineString::from(vec![
            Coord { x: 10.0, y: 10.0 },
            Coord { x: 50.0, y: 50.0 },
            Coord { x: 90.0, y: 90.0 },
        ]);

        let clipped = clipper.clip_linestring(&ls, &bounds);
        assert!(clipped.is_some());
    }
}
