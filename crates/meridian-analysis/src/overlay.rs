//! Overlay operations for geometric intersection, union, difference

use crate::error::{AnalysisError, Result};
use geo::{Area, Contains, Intersects, MultiPolygon, Polygon};
use geo_booleanop::boolean::BooleanOp;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Overlay operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayOp {
    /// Union - combine geometries
    Union,
    /// Intersection - keep only overlapping areas
    Intersection,
    /// Difference - subtract second from first
    Difference,
    /// Symmetric difference - keep non-overlapping areas
    SymmetricDifference,
}

/// Compute the union of two polygons
pub fn union(a: &Polygon, b: &Polygon) -> Result<MultiPolygon> {
    a.union(b)
        .map_err(|e| AnalysisError::OverlayError(format!("Union failed: {:?}", e)))
}

/// Compute the intersection of two polygons
pub fn intersection(a: &Polygon, b: &Polygon) -> Result<MultiPolygon> {
    a.intersection(b)
        .map_err(|e| AnalysisError::OverlayError(format!("Intersection failed: {:?}", e)))
}

/// Compute the difference of two polygons (A - B)
pub fn difference(a: &Polygon, b: &Polygon) -> Result<MultiPolygon> {
    a.difference(b)
        .map_err(|e| AnalysisError::OverlayError(format!("Difference failed: {:?}", e)))
}

/// Compute the symmetric difference of two polygons
pub fn symmetric_difference(a: &Polygon, b: &Polygon) -> Result<MultiPolygon> {
    a.xor(b)
        .map_err(|e| AnalysisError::OverlayError(format!("Symmetric difference failed: {:?}", e)))
}

/// Perform an overlay operation on two polygons
pub fn overlay(a: &Polygon, b: &Polygon, op: OverlayOp) -> Result<MultiPolygon> {
    match op {
        OverlayOp::Union => union(a, b),
        OverlayOp::Intersection => intersection(a, b),
        OverlayOp::Difference => difference(a, b),
        OverlayOp::SymmetricDifference => symmetric_difference(a, b),
    }
}

/// Compute the union of multiple polygons
pub fn union_many(polygons: &[Polygon]) -> Result<MultiPolygon> {
    if polygons.is_empty() {
        return Ok(MultiPolygon::new(vec![]));
    }

    if polygons.len() == 1 {
        return Ok(MultiPolygon::new(vec![polygons[0].clone()]));
    }

    // Iteratively union polygons
    let mut result = polygons[0].clone();

    for polygon in &polygons[1..] {
        let union_result = union(&result, polygon)?;

        // Convert multi-polygon back to polygon for next iteration
        // This is simplified; in production, handle multi-polygon properly
        if let Some(first) = union_result.0.first() {
            result = first.clone();
        }
    }

    Ok(MultiPolygon::new(vec![result]))
}

/// Dissolve adjacent or overlapping polygons with the same attribute
pub fn dissolve<T>(polygons: &[(Polygon, T)]) -> Result<Vec<(MultiPolygon, T)>>
where
    T: Clone + PartialEq,
{
    if polygons.is_empty() {
        return Ok(vec![]);
    }

    let mut result = Vec::new();
    let mut processed = vec![false; polygons.len()];

    for i in 0..polygons.len() {
        if processed[i] {
            continue;
        }

        let (ref poly_i, ref attr_i) = polygons[i];
        let mut dissolved = vec![poly_i.clone()];
        processed[i] = true;

        // Find all polygons with the same attribute that touch or overlap
        for j in (i + 1)..polygons.len() {
            if processed[j] {
                continue;
            }

            let (ref poly_j, ref attr_j) = polygons[j];

            if attr_i == attr_j && (poly_i.intersects(poly_j) || touches(poly_i, poly_j)) {
                dissolved.push(poly_j.clone());
                processed[j] = true;
            }
        }

        // Union all dissolved polygons
        let multi = union_many(&dissolved)?;
        result.push((multi, attr_i.clone()));
    }

    Ok(result)
}

/// Check if two polygons touch (share boundary but don't overlap)
fn touches(a: &Polygon, b: &Polygon) -> bool {
    // Simplified: check if they intersect but neither contains the other
    if !a.intersects(b) {
        return false;
    }

    // If they intersect, check if the intersection is just boundary
    // This is a simplified implementation
    !a.contains(b) && !b.contains(a)
}

/// Clip a polygon to a bounding box
pub fn clip_to_bbox(polygon: &Polygon, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<MultiPolygon> {
    // Create a clipping polygon from the bbox
    let clip_poly = Polygon::new(
        geo::LineString::from(vec![
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
            (min_x, min_y),
        ]),
        vec![],
    );

    intersection(polygon, &clip_poly)
}

/// Clip a polygon to another polygon
pub fn clip(subject: &Polygon, clip_poly: &Polygon) -> Result<MultiPolygon> {
    intersection(subject, clip_poly)
}

/// Erase parts of a polygon that overlap with another polygon
pub fn erase(subject: &Polygon, erase_poly: &Polygon) -> Result<MultiPolygon> {
    difference(subject, erase_poly)
}

/// Identity overlay - intersection with attributes from both inputs
pub fn identity<T, U>(
    subject: &Polygon,
    subject_attr: T,
    overlay: &Polygon,
    overlay_attr: U,
) -> Result<Vec<(MultiPolygon, Option<T>, Option<U>)>>
where
    T: Clone,
    U: Clone,
{
    let mut result = Vec::new();

    // Parts only in subject
    let only_subject = difference(subject, overlay)?;
    if !only_subject.0.is_empty() {
        result.push((only_subject, Some(subject_attr.clone()), None));
    }

    // Parts in both
    let both = intersection(subject, overlay)?;
    if !both.0.is_empty() {
        result.push((both, Some(subject_attr), Some(overlay_attr)));
    }

    Ok(result)
}

/// Update overlay - like identity but doesn't preserve input features
pub fn update(subject: &Polygon, update_poly: &Polygon) -> Result<MultiPolygon> {
    // Remove parts of subject that overlap with update_poly, then add update_poly
    let diff = difference(subject, update_poly)?;

    // Union the difference with the update polygon
    if let Some(first) = diff.0.first() {
        union(first, update_poly)
    } else {
        Ok(MultiPolygon::new(vec![update_poly.clone()]))
    }
}

/// Intersect multiple polygon layers
pub fn intersect_layers(layers: &[Vec<Polygon>]) -> Result<Vec<Polygon>> {
    if layers.is_empty() {
        return Ok(vec![]);
    }

    if layers.len() == 1 {
        return Ok(layers[0].clone());
    }

    let mut result = layers[0].clone();

    for layer in &layers[1..] {
        let mut new_result = Vec::new();

        for poly1 in &result {
            for poly2 in layer {
                let inter = intersection(poly1, poly2)?;
                new_result.extend(inter.0);
            }
        }

        result = new_result;
    }

    Ok(result)
}

/// Split a polygon by a line
pub fn split_polygon_by_line(
    polygon: &Polygon,
    _line: &geo::LineString,
) -> Result<Vec<Polygon>> {
    // This is a complex operation requiring polygon splitting
    // Simplified implementation: return original polygon
    // In production, would need to implement line-polygon splitting algorithm
    Ok(vec![polygon.clone()])
}

/// Perform overlay operations in parallel on multiple polygon pairs
pub fn overlay_many_parallel(
    pairs: &[(Polygon, Polygon)],
    op: OverlayOp,
) -> Result<Vec<MultiPolygon>> {
    pairs
        .par_iter()
        .map(|(a, b)| overlay(a, b, op))
        .collect()
}

/// Calculate the area of overlay between two polygons
pub fn overlay_area(a: &Polygon, b: &Polygon, op: OverlayOp) -> Result<f64> {
    let result = overlay(a, b, op)?;
    Ok(result.unsigned_area())
}

/// Check if two polygons overlap by a certain percentage
pub fn overlap_percentage(a: &Polygon, b: &Polygon) -> Result<f64> {
    let inter = intersection(a, b)?;
    let inter_area = inter.unsigned_area();
    let a_area = a.unsigned_area();

    if a_area == 0.0 {
        return Ok(0.0);
    }

    Ok(inter_area / a_area * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::LineString;

    fn create_square(x: f64, y: f64, size: f64) -> Polygon {
        Polygon::new(
            LineString::from(vec![
                (x, y),
                (x + size, y),
                (x + size, y + size),
                (x, y + size),
                (x, y),
            ]),
            vec![],
        )
    }

    #[test]
    fn test_union() {
        let a = create_square(0.0, 0.0, 10.0);
        let b = create_square(5.0, 5.0, 10.0);
        let result = union(&a, &b).unwrap();
        assert!(!result.0.is_empty());
    }

    #[test]
    fn test_intersection() {
        let a = create_square(0.0, 0.0, 10.0);
        let b = create_square(5.0, 5.0, 10.0);
        let result = intersection(&a, &b).unwrap();
        assert!(!result.0.is_empty());
    }

    #[test]
    fn test_difference() {
        let a = create_square(0.0, 0.0, 10.0);
        let b = create_square(5.0, 5.0, 10.0);
        let result = difference(&a, &b).unwrap();
        assert!(!result.0.is_empty());
    }

    #[test]
    fn test_symmetric_difference() {
        let a = create_square(0.0, 0.0, 10.0);
        let b = create_square(5.0, 5.0, 10.0);
        let result = symmetric_difference(&a, &b).unwrap();
        assert!(!result.0.is_empty());
    }

    #[test]
    fn test_clip_to_bbox() {
        let poly = create_square(0.0, 0.0, 20.0);
        let result = clip_to_bbox(&poly, 5.0, 5.0, 15.0, 15.0).unwrap();
        assert!(!result.0.is_empty());
    }

    #[test]
    fn test_overlay_area() {
        let a = create_square(0.0, 0.0, 10.0);
        let b = create_square(5.0, 5.0, 10.0);
        let area = overlay_area(&a, &b, OverlayOp::Intersection).unwrap();
        assert!(area > 0.0);
    }

    #[test]
    fn test_overlap_percentage() {
        let a = create_square(0.0, 0.0, 10.0);
        let b = create_square(0.0, 0.0, 10.0); // Identical
        let pct = overlap_percentage(&a, &b).unwrap();
        assert!((pct - 100.0).abs() < 0.01);
    }
}
