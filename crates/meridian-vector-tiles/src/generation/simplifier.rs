//! Zoom-level geometry simplification

use geo::algorithm::simplify::Simplify;
use geo_types::{Geometry, LineString, MultiLineString, MultiPolygon, Polygon};

/// Geometry simplifier with zoom-level awareness
pub struct GeometrySimplifier {
    base_tolerance: f64,
}

impl GeometrySimplifier {
    /// Create a new geometry simplifier
    pub fn new(base_tolerance: f64) -> Self {
        Self { base_tolerance }
    }

    /// Simplify geometry based on zoom level
    pub fn simplify(&self, geometry: &Geometry, zoom: u8) -> Geometry {
        let tolerance = self.calculate_tolerance(zoom);

        match geometry {
            Geometry::LineString(ls) => {
                Geometry::LineString(self.simplify_linestring(ls, tolerance))
            }
            Geometry::MultiLineString(mls) => {
                Geometry::MultiLineString(self.simplify_multilinestring(mls, tolerance))
            }
            Geometry::Polygon(poly) => {
                Geometry::Polygon(self.simplify_polygon(poly, tolerance))
            }
            Geometry::MultiPolygon(mp) => {
                Geometry::MultiPolygon(self.simplify_multipolygon(mp, tolerance))
            }
            _ => geometry.clone(),
        }
    }

    /// Calculate simplification tolerance for zoom level
    fn calculate_tolerance(&self, zoom: u8) -> f64 {
        // Higher zoom = less simplification (smaller tolerance)
        // Lower zoom = more simplification (larger tolerance)
        let scale = 2.0_f64.powi((20 - zoom as i32).max(0));
        self.base_tolerance * scale
    }

    /// Simplify a linestring
    fn simplify_linestring(&self, ls: &LineString, tolerance: f64) -> LineString {
        if tolerance <= 0.0 {
            return ls.clone();
        }
        ls.simplify(&tolerance)
    }

    /// Simplify a multilinestring
    fn simplify_multilinestring(&self, mls: &MultiLineString, tolerance: f64) -> MultiLineString {
        if tolerance <= 0.0 {
            return mls.clone();
        }

        MultiLineString(
            mls.0
                .iter()
                .map(|ls| self.simplify_linestring(ls, tolerance))
                .collect(),
        )
    }

    /// Simplify a polygon
    fn simplify_polygon(&self, poly: &Polygon, tolerance: f64) -> Polygon {
        if tolerance <= 0.0 {
            return poly.clone();
        }
        poly.simplify(&tolerance)
    }

    /// Simplify a multipolygon
    fn simplify_multipolygon(&self, mp: &MultiPolygon, tolerance: f64) -> MultiPolygon {
        if tolerance <= 0.0 {
            return mp.clone();
        }

        MultiPolygon(
            mp.0.iter()
                .map(|p| self.simplify_polygon(p, tolerance))
                .collect(),
        )
    }

    /// Get tolerance for a specific zoom level
    pub fn tolerance_for_zoom(&self, zoom: u8) -> f64 {
        self.calculate_tolerance(zoom)
    }
}

impl Default for GeometrySimplifier {
    fn default() -> Self {
        Self::new(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_types::Coord;

    #[test]
    fn test_tolerance_calculation() {
        let simplifier = GeometrySimplifier::new(1.0);

        let tol_z20 = simplifier.tolerance_for_zoom(20);
        let tol_z10 = simplifier.tolerance_for_zoom(10);
        let tol_z0 = simplifier.tolerance_for_zoom(0);

        // Lower zoom should have higher tolerance
        assert!(tol_z0 > tol_z10);
        assert!(tol_z10 > tol_z20);
    }

    #[test]
    fn test_linestring_simplification() {
        let simplifier = GeometrySimplifier::new(1.0);

        let ls = LineString::from(vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.1 },
            Coord { x: 2.0, y: 0.0 },
            Coord { x: 3.0, y: 0.0 },
        ]);

        let simplified_high = simplifier.simplify_linestring(&ls, 0.5);
        let simplified_low = simplifier.simplify_linestring(&ls, 0.01);

        // Higher tolerance should result in fewer points
        assert!(simplified_high.0.len() <= simplified_low.0.len());
    }
}
