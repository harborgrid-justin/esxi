//! Geometry validation and repair operations

use crate::error::Result;
use geo::{Area, Coord, EuclideanDistance, LineString, Point, Polygon};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Validation issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Critical error that prevents use
    Error,
    /// Warning that may cause issues
    Warning,
    /// Informational only
    Info,
}

/// Validation issue type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueType {
    /// Polygon ring not closed
    RingNotClosed,
    /// Self-intersection detected
    SelfIntersection,
    /// Duplicate vertices
    DuplicateVertices,
    /// Too few vertices
    InsufficientVertices,
    /// Ring has wrong orientation
    WrongOrientation,
    /// Holes outside shell
    HoleOutsideShell,
    /// Nested holes
    NestedHoles,
    /// Duplicate rings
    DuplicateRings,
    /// Spike detected
    Spike,
    /// Zero-length segment
    ZeroLengthSegment,
    /// Invalid area (zero or negative)
    InvalidArea,
    /// Coordinates out of bounds
    CoordinatesOutOfBounds,
    /// Invalid geometry structure
    InvalidStructure,
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub issue_type: IssueType,
    pub severity: Severity,
    pub message: String,
    pub location: Option<Point>,
}

impl ValidationIssue {
    pub fn new(issue_type: IssueType, severity: Severity, message: String) -> Self {
        Self {
            issue_type,
            severity,
            message,
            location: None,
        }
    }

    pub fn with_location(mut self, location: Point) -> Self {
        self.location = Some(location);
        self
    }
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            issues: vec![],
        }
    }

    pub fn invalid(issues: Vec<ValidationIssue>) -> Self {
        Self {
            is_valid: false,
            issues,
        }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        if issue.severity == Severity::Error {
            self.is_valid = false;
        }
        self.issues.push(issue);
    }

    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.issues.iter().any(|i| i.severity == Severity::Warning)
    }
}

/// Validate a line string
pub fn validate_line(line: &LineString) -> ValidationResult {
    let mut result = ValidationResult::valid();

    // Check minimum vertices
    if line.coords_count() < 2 {
        result.add_issue(ValidationIssue::new(
            IssueType::InsufficientVertices,
            Severity::Error,
            format!("Line must have at least 2 vertices, got {}", line.coords_count()),
        ));
        return result;
    }

    let coords: Vec<Coord> = line.coords().cloned().collect();

    // Check for duplicate consecutive vertices
    for i in 0..coords.len() - 1 {
        if coords_equal(&coords[i], &coords[i + 1]) {
            result.add_issue(
                ValidationIssue::new(
                    IssueType::DuplicateVertices,
                    Severity::Warning,
                    format!("Duplicate consecutive vertices at index {}", i),
                )
                .with_location(Point::from(coords[i])),
            );
        }
    }

    // Check for zero-length segments
    for i in 0..coords.len() - 1 {
        let dist = Point::from(coords[i]).euclidean_distance(&Point::from(coords[i + 1]));
        if dist < 1e-10 {
            result.add_issue(
                ValidationIssue::new(
                    IssueType::ZeroLengthSegment,
                    Severity::Warning,
                    format!("Zero-length segment at index {}", i),
                )
                .with_location(Point::from(coords[i])),
            );
        }
    }

    // Check for self-intersections (simplified check)
    if has_self_intersection(line) {
        result.add_issue(ValidationIssue::new(
            IssueType::SelfIntersection,
            Severity::Error,
            "Line has self-intersections".to_string(),
        ));
    }

    result
}

/// Validate a polygon
pub fn validate_polygon(polygon: &Polygon) -> ValidationResult {
    let mut result = ValidationResult::valid();

    // Validate exterior ring
    let exterior = polygon.exterior();

    // Check if ring is closed
    if !is_ring_closed(exterior) {
        result.add_issue(ValidationIssue::new(
            IssueType::RingNotClosed,
            Severity::Error,
            "Exterior ring is not closed".to_string(),
        ));
    }

    // Check minimum vertices (4 for a closed ring: 3 unique + closing)
    if exterior.coords_count() < 4 {
        result.add_issue(ValidationIssue::new(
            IssueType::InsufficientVertices,
            Severity::Error,
            format!("Exterior ring must have at least 4 vertices, got {}", exterior.coords_count()),
        ));
    }

    // Check ring orientation (exterior should be counter-clockwise)
    if !is_counter_clockwise(exterior) {
        result.add_issue(ValidationIssue::new(
            IssueType::WrongOrientation,
            Severity::Warning,
            "Exterior ring should be counter-clockwise".to_string(),
        ));
    }

    // Check for self-intersections in exterior
    if has_self_intersection(exterior) {
        result.add_issue(ValidationIssue::new(
            IssueType::SelfIntersection,
            Severity::Error,
            "Exterior ring has self-intersections".to_string(),
        ));
    }

    // Validate interior rings (holes)
    for (i, interior) in polygon.interiors().iter().enumerate() {
        if !is_ring_closed(interior) {
            result.add_issue(ValidationIssue::new(
                IssueType::RingNotClosed,
                Severity::Error,
                format!("Interior ring {} is not closed", i),
            ));
        }

        if interior.coords_count() < 4 {
            result.add_issue(ValidationIssue::new(
                IssueType::InsufficientVertices,
                Severity::Error,
                format!("Interior ring {} must have at least 4 vertices", i),
            ));
        }

        // Interior rings should be clockwise
        if is_counter_clockwise(interior) {
            result.add_issue(ValidationIssue::new(
                IssueType::WrongOrientation,
                Severity::Warning,
                format!("Interior ring {} should be clockwise", i),
            ));
        }

        if has_self_intersection(interior) {
            result.add_issue(ValidationIssue::new(
                IssueType::SelfIntersection,
                Severity::Error,
                format!("Interior ring {} has self-intersections", i),
            ));
        }
    }

    // Check polygon area
    let area = polygon.unsigned_area();
    if area == 0.0 {
        result.add_issue(ValidationIssue::new(
            IssueType::InvalidArea,
            Severity::Error,
            "Polygon has zero area".to_string(),
        ));
    }

    result
}

/// Check if a ring is closed (first and last coordinates are equal)
fn is_ring_closed(ring: &LineString) -> bool {
    if ring.coords_count() < 2 {
        return false;
    }

    let coords: Vec<Coord> = ring.coords().cloned().collect();
    coords_equal(&coords[0], coords.last().unwrap())
}

/// Check if two coordinates are equal (within tolerance)
fn coords_equal(c1: &Coord, c2: &Coord) -> bool {
    (c1.x - c2.x).abs() < 1e-10 && (c1.y - c2.y).abs() < 1e-10
}

/// Check if a ring is counter-clockwise (using signed area)
fn is_counter_clockwise(ring: &LineString) -> bool {
    let coords: Vec<Coord> = ring.coords().cloned().collect();
    let signed_area = calculate_signed_area(&coords);
    signed_area > 0.0
}

/// Calculate signed area of a ring
fn calculate_signed_area(coords: &[Coord]) -> f64 {
    let mut area = 0.0;

    for i in 0..coords.len() - 1 {
        area += (coords[i].x * coords[i + 1].y) - (coords[i + 1].x * coords[i].y);
    }

    area / 2.0
}

/// Check for self-intersections (simplified implementation)
fn has_self_intersection(line: &LineString) -> bool {
    let coords: Vec<Coord> = line.coords().cloned().collect();

    if coords.len() < 4 {
        return false;
    }

    // Check each segment pair (except adjacent segments)
    for i in 0..coords.len() - 1 {
        for j in (i + 2)..coords.len() - 1 {
            // Skip adjacent segments and last-first for closed rings
            if j == coords.len() - 1 && i == 0 {
                continue;
            }

            if segments_intersect(
                &coords[i],
                &coords[i + 1],
                &coords[j],
                &coords[j + 1],
            ) {
                return true;
            }
        }
    }

    false
}

/// Check if two line segments intersect
fn segments_intersect(a1: &Coord, a2: &Coord, b1: &Coord, b2: &Coord) -> bool {
    let d = (a2.x - a1.x) * (b2.y - b1.y) - (a2.y - a1.y) * (b2.x - b1.x);

    if d.abs() < 1e-10 {
        return false; // Parallel or collinear
    }

    let t = ((b1.x - a1.x) * (b2.y - b1.y) - (b1.y - a1.y) * (b2.x - b1.x)) / d;
    let u = ((b1.x - a1.x) * (a2.y - a1.y) - (b1.y - a1.y) * (a2.x - a1.x)) / d;

    t > 0.0 && t < 1.0 && u > 0.0 && u < 1.0
}

/// Repair a polygon by fixing common issues
pub fn repair_polygon(polygon: &Polygon) -> Result<Polygon> {
    let mut exterior = polygon.exterior().clone();
    let mut interiors: Vec<LineString> = polygon.interiors().to_vec();

    // Fix exterior ring orientation
    if !is_counter_clockwise(&exterior) {
        exterior = reverse_ring(&exterior);
    }

    // Fix interior ring orientations
    for interior in interiors.iter_mut() {
        if is_counter_clockwise(interior) {
            *interior = reverse_ring(interior);
        }
    }

    // Remove duplicate consecutive vertices
    exterior = remove_duplicate_vertices(&exterior);
    interiors = interiors
        .iter()
        .map(|ring| remove_duplicate_vertices(ring))
        .collect();

    // Ensure rings are closed
    exterior = ensure_closed(&exterior);
    interiors = interiors
        .iter()
        .map(|ring| ensure_closed(ring))
        .collect();

    Ok(Polygon::new(exterior, interiors))
}

/// Reverse the order of coordinates in a ring
fn reverse_ring(ring: &LineString) -> LineString {
    let coords: Vec<Coord> = ring.coords().rev().cloned().collect();
    LineString::from(coords)
}

/// Remove duplicate consecutive vertices
fn remove_duplicate_vertices(ring: &LineString) -> LineString {
    let coords: Vec<Coord> = ring.coords().cloned().collect();

    if coords.is_empty() {
        return ring.clone();
    }

    let mut cleaned = vec![coords[0]];

    for coord in coords.iter().skip(1) {
        if let Some(&last) = cleaned.last() {
            if !coords_equal(&last, coord) {
                cleaned.push(*coord);
            }
        }
    }

    LineString::from(cleaned)
}

/// Ensure a ring is closed
fn ensure_closed(ring: &LineString) -> LineString {
    let coords: Vec<Coord> = ring.coords().cloned().collect();

    if coords.len() < 2 {
        return ring.clone();
    }

    let mut closed = coords;

    if !coords_equal(&closed[0], closed.last().unwrap()) {
        closed.push(closed[0]);
    }

    LineString::from(closed)
}

/// Remove spikes from a polygon
pub fn remove_spikes(polygon: &Polygon, angle_threshold: f64) -> Result<Polygon> {
    let exterior = remove_spikes_from_ring(polygon.exterior(), angle_threshold);

    let interiors: Vec<LineString> = polygon
        .interiors()
        .iter()
        .map(|ring| remove_spikes_from_ring(ring, angle_threshold))
        .collect();

    Ok(Polygon::new(exterior, interiors))
}

/// Remove spikes from a ring
fn remove_spikes_from_ring(ring: &LineString, angle_threshold: f64) -> LineString {
    let coords: Vec<Coord> = ring.coords().cloned().collect();

    if coords.len() < 4 {
        return ring.clone();
    }

    let mut cleaned = vec![coords[0]];

    for i in 1..coords.len() - 1 {
        let p0 = coords[i - 1];
        let p1 = coords[i];
        let p2 = coords[i + 1];

        let angle = calculate_angle(&p0, &p1, &p2);

        // Keep vertex if angle is not too sharp
        if angle.abs() > angle_threshold {
            cleaned.push(p1);
        }
    }

    // Add last point if different from first
    if let Some(&last) = coords.last() {
        if !coords_equal(&cleaned[0], &last) {
            cleaned.push(last);
        } else if cleaned.len() > 1 {
            // Ensure closed
            cleaned.push(cleaned[0]);
        }
    }

    LineString::from(cleaned)
}

/// Calculate angle at vertex p1
fn calculate_angle(p0: &Coord, p1: &Coord, p2: &Coord) -> f64 {
    let v1x = p0.x - p1.x;
    let v1y = p0.y - p1.y;
    let v2x = p2.x - p1.x;
    let v2y = p2.y - p1.y;

    let dot = v1x * v2x + v1y * v2y;
    let cross = v1x * v2y - v1y * v2x;

    cross.atan2(dot)
}

/// Simplify and clean a polygon
pub fn clean_polygon(polygon: &Polygon, tolerance: f64) -> Result<Polygon> {
    use geo::Simplify;

    // First repair
    let repaired = repair_polygon(polygon)?;

    // Then simplify
    let simplified = repaired.simplify(&tolerance);

    Ok(simplified)
}

/// Check if a point is within valid coordinate bounds
pub fn validate_coordinates(point: &Point, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> bool {
    point.x() >= min_x && point.x() <= max_x && point.y() >= min_y && point.y() <= max_y
}

/// Find and report duplicate points in a collection
pub fn find_duplicate_points(points: &[Point], tolerance: f64) -> Vec<Vec<usize>> {
    let mut duplicates = Vec::new();
    let mut processed = HashSet::new();

    for i in 0..points.len() {
        if processed.contains(&i) {
            continue;
        }

        let mut group = vec![i];

        for j in (i + 1)..points.len() {
            if processed.contains(&j) {
                continue;
            }

            if points[i].euclidean_distance(&points[j]) <= tolerance {
                group.push(j);
                processed.insert(j);
            }
        }

        if group.len() > 1 {
            duplicates.push(group);
            processed.insert(i);
        }
    }

    duplicates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_line() {
        let line = LineString::from(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]);
        let result = validate_line(&line);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_line_insufficient_vertices() {
        let line = LineString::from(vec![(0.0, 0.0)]);
        let result = validate_line(&line);
        assert!(!result.is_valid);
        assert!(result.has_errors());
    }

    #[test]
    fn test_validate_polygon() {
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
        assert!(result.is_valid || result.has_warnings()); // May have orientation warning
    }

    #[test]
    fn test_is_counter_clockwise() {
        let ccw_ring = LineString::from(vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ]);
        assert!(is_counter_clockwise(&ccw_ring));
    }

    #[test]
    fn test_repair_polygon() {
        // Create polygon with clockwise exterior (wrong orientation)
        let polygon = Polygon::new(
            LineString::from(vec![
                (0.0, 0.0),
                (0.0, 10.0),
                (10.0, 10.0),
                (10.0, 0.0),
                (0.0, 0.0),
            ]),
            vec![],
        );

        let repaired = repair_polygon(&polygon).unwrap();
        assert!(is_counter_clockwise(repaired.exterior()));
    }

    #[test]
    fn test_remove_duplicate_vertices() {
        let ring = LineString::from(vec![
            (0.0, 0.0),
            (0.0, 0.0), // duplicate
            (10.0, 0.0),
            (10.0, 0.0), // duplicate
            (0.0, 0.0),
        ]);

        let cleaned = remove_duplicate_vertices(&ring);
        assert!(cleaned.coords_count() < ring.coords_count());
    }

    #[test]
    fn test_validate_coordinates() {
        let point = Point::new(5.0, 5.0);
        assert!(validate_coordinates(&point, 0.0, 0.0, 10.0, 10.0));
        assert!(!validate_coordinates(&point, 0.0, 0.0, 4.0, 4.0));
    }

    #[test]
    fn test_find_duplicate_points() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(10.0, 10.0),
        ];

        let duplicates = find_duplicate_points(&points, 1e-10);
        assert_eq!(duplicates.len(), 2);
    }
}
