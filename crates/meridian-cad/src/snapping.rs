//! Snapping system for precise drawing
//!
//! This module provides grid snapping, object snapping, and smart guides
//! for accurate CAD drawing operations.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

use crate::canvas::{Entity, Layer};
use crate::primitives::{Line, Point};
use crate::{CadError, CadResult};

/// Snap result containing the snapped point and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapResult {
    /// The snapped point
    pub point: Point,

    /// Type of snap that occurred
    pub snap_type: SnapType,

    /// Distance from original point to snapped point
    pub distance: f64,

    /// Optional reference entity
    pub reference_entity: Option<uuid::Uuid>,

    /// Display message for user feedback
    pub message: String,
}

impl SnapResult {
    /// Create a new snap result
    pub fn new(point: Point, snap_type: SnapType) -> Self {
        Self {
            point,
            snap_type,
            distance: 0.0,
            reference_entity: None,
            message: snap_type.description().to_string(),
        }
    }

    /// Create with reference entity
    pub fn with_reference(mut self, entity_id: uuid::Uuid) -> Self {
        self.reference_entity = Some(entity_id);
        self
    }

    /// Set distance
    pub fn with_distance(mut self, distance: f64) -> Self {
        self.distance = distance;
        self
    }
}

/// Types of snapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapType {
    /// No snap occurred
    None,

    /// Snapped to grid
    Grid,

    /// Snapped to endpoint of line/arc
    Endpoint,

    /// Snapped to midpoint of line/arc
    Midpoint,

    /// Snapped to center of circle/arc
    Center,

    /// Snapped to intersection of two entities
    Intersection,

    /// Snapped perpendicular to line
    Perpendicular,

    /// Snapped to nearest point on entity
    Nearest,

    /// Snapped to tangent point
    Tangent,

    /// Snapped to quadrant of circle
    Quadrant,

    /// Snapped to extension of line
    Extension,

    /// Snapped using smart guide
    SmartGuide,
}

impl SnapType {
    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            SnapType::None => "No snap",
            SnapType::Grid => "Grid",
            SnapType::Endpoint => "Endpoint",
            SnapType::Midpoint => "Midpoint",
            SnapType::Center => "Center",
            SnapType::Intersection => "Intersection",
            SnapType::Perpendicular => "Perpendicular",
            SnapType::Nearest => "Nearest",
            SnapType::Tangent => "Tangent",
            SnapType::Quadrant => "Quadrant",
            SnapType::Extension => "Extension",
            SnapType::SmartGuide => "Smart Guide",
        }
    }

    /// Get priority (higher = more important)
    pub fn priority(&self) -> i32 {
        match self {
            SnapType::None => 0,
            SnapType::Grid => 1,
            SnapType::Nearest => 2,
            SnapType::SmartGuide => 3,
            SnapType::Extension => 4,
            SnapType::Tangent => 5,
            SnapType::Perpendicular => 6,
            SnapType::Quadrant => 7,
            SnapType::Center => 8,
            SnapType::Midpoint => 9,
            SnapType::Intersection => 10,
            SnapType::Endpoint => 11,
        }
    }
}

/// Grid snapping configuration and methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridSnap {
    pub enabled: bool,
    pub spacing: f64,
    pub subdivisions: u32,
    pub angle_snap: bool,
    pub angle_increment: f64, // degrees
}

impl Default for GridSnap {
    fn default() -> Self {
        Self {
            enabled: true,
            spacing: 10.0,
            subdivisions: 10,
            angle_snap: false,
            angle_increment: 15.0,
        }
    }
}

impl GridSnap {
    /// Create a new grid snap configuration
    pub fn new(spacing: f64) -> Self {
        Self {
            spacing,
            ..Default::default()
        }
    }

    /// Snap a point to the grid
    pub fn snap(&self, point: &Point) -> SnapResult {
        if !self.enabled {
            return SnapResult::new(*point, SnapType::None);
        }

        let snapped_x = (point.x / self.spacing).round() * self.spacing;
        let snapped_y = (point.y / self.spacing).round() * self.spacing;

        let snapped_point = Point::new(snapped_x, snapped_y);
        let distance = point.distance(&snapped_point);

        SnapResult::new(snapped_point, SnapType::Grid).with_distance(distance)
    }

    /// Snap angle to increment
    pub fn snap_angle(&self, angle_radians: f64) -> f64 {
        if !self.angle_snap {
            return angle_radians;
        }

        let angle_degrees = angle_radians.to_degrees();
        let increment = self.angle_increment;
        let snapped_degrees = (angle_degrees / increment).round() * increment;
        snapped_degrees.to_radians()
    }

    /// Get effective snap spacing (considering subdivisions)
    pub fn effective_spacing(&self) -> f64 {
        self.spacing / self.subdivisions as f64
    }
}

/// Object snapping configuration and methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectSnap {
    pub enabled: bool,
    pub snap_radius: f64,
    pub endpoint: bool,
    pub midpoint: bool,
    pub center: bool,
    pub intersection: bool,
    pub perpendicular: bool,
    pub nearest: bool,
    pub tangent: bool,
    pub quadrant: bool,
}

impl Default for ObjectSnap {
    fn default() -> Self {
        Self {
            enabled: true,
            snap_radius: 10.0,
            endpoint: true,
            midpoint: true,
            center: true,
            intersection: true,
            perpendicular: true,
            nearest: true,
            tangent: false,
            quadrant: false,
        }
    }
}

impl ObjectSnap {
    /// Create with all snaps enabled
    pub fn all() -> Self {
        Self {
            endpoint: true,
            midpoint: true,
            center: true,
            intersection: true,
            perpendicular: true,
            nearest: true,
            tangent: true,
            quadrant: true,
            ..Default::default()
        }
    }

    /// Create with only endpoint and midpoint
    pub fn basic() -> Self {
        Self {
            endpoint: true,
            midpoint: true,
            center: false,
            intersection: false,
            perpendicular: false,
            nearest: false,
            tangent: false,
            quadrant: false,
            ..Default::default()
        }
    }

    /// Find the best snap for a point near entities
    pub fn snap(&self, point: &Point, entities: &[&Entity]) -> SnapResult {
        if !self.enabled || entities.is_empty() {
            return SnapResult::new(*point, SnapType::None);
        }

        let mut candidates = Vec::new();

        // Collect all possible snap points
        for entity in entities {
            candidates.extend(self.get_snap_points(point, entity));
        }

        // Find closest snap within radius
        let mut best: Option<SnapResult> = None;

        for candidate in candidates {
            if candidate.distance <= self.snap_radius {
                if let Some(ref current_best) = best {
                    // Prefer higher priority snaps, or closer if same priority
                    if candidate.snap_type.priority() > current_best.snap_type.priority()
                        || (candidate.snap_type.priority() == current_best.snap_type.priority()
                            && candidate.distance < current_best.distance)
                    {
                        best = Some(candidate);
                    }
                } else {
                    best = Some(candidate);
                }
            }
        }

        best.unwrap_or_else(|| SnapResult::new(*point, SnapType::None))
    }

    /// Get all snap points for an entity
    fn get_snap_points(&self, cursor: &Point, entity: &Entity) -> Vec<SnapResult> {
        let mut snaps = Vec::new();

        match entity {
            Entity::Line(line) => {
                // Endpoints
                if self.endpoint {
                    snaps.push(
                        SnapResult::new(line.start, SnapType::Endpoint)
                            .with_reference(line.id)
                            .with_distance(cursor.distance(&line.start)),
                    );
                    snaps.push(
                        SnapResult::new(line.end, SnapType::Endpoint)
                            .with_reference(line.id)
                            .with_distance(cursor.distance(&line.end)),
                    );
                }

                // Midpoint
                if self.midpoint {
                    let mid = line.midpoint();
                    snaps.push(
                        SnapResult::new(mid, SnapType::Midpoint)
                            .with_reference(line.id)
                            .with_distance(cursor.distance(&mid)),
                    );
                }

                // Nearest point on line
                if self.nearest {
                    let nearest = self.nearest_point_on_line(cursor, line);
                    snaps.push(
                        SnapResult::new(nearest, SnapType::Nearest)
                            .with_reference(line.id)
                            .with_distance(cursor.distance(&nearest)),
                    );
                }
            }

            Entity::Arc(arc) => {
                // Endpoints
                if self.endpoint {
                    let start = arc.start_point();
                    let end = arc.end_point();
                    snaps.push(
                        SnapResult::new(start, SnapType::Endpoint)
                            .with_reference(arc.id)
                            .with_distance(cursor.distance(&start)),
                    );
                    snaps.push(
                        SnapResult::new(end, SnapType::Endpoint)
                            .with_reference(arc.id)
                            .with_distance(cursor.distance(&end)),
                    );
                }

                // Center
                if self.center {
                    snaps.push(
                        SnapResult::new(arc.center, SnapType::Center)
                            .with_reference(arc.id)
                            .with_distance(cursor.distance(&arc.center)),
                    );
                }

                // Quadrants
                if self.quadrant {
                    for angle in [0.0, PI / 2.0, PI, 3.0 * PI / 2.0] {
                        if angle >= arc.start_angle && angle <= arc.end_angle {
                            let quad = arc.point_at_angle(angle);
                            snaps.push(
                                SnapResult::new(quad, SnapType::Quadrant)
                                    .with_reference(arc.id)
                                    .with_distance(cursor.distance(&quad)),
                            );
                        }
                    }
                }
            }

            Entity::Ellipse(ellipse) => {
                // Center
                if self.center {
                    snaps.push(
                        SnapResult::new(ellipse.center, SnapType::Center)
                            .with_reference(ellipse.id)
                            .with_distance(cursor.distance(&ellipse.center)),
                    );
                }

                // Quadrants
                if self.quadrant {
                    for angle in [0.0, PI / 2.0, PI, 3.0 * PI / 2.0] {
                        let quad = ellipse.point_at_angle(angle);
                        snaps.push(
                            SnapResult::new(quad, SnapType::Quadrant)
                                .with_reference(ellipse.id)
                                .with_distance(cursor.distance(&quad)),
                        );
                    }
                }
            }

            Entity::Polygon(polygon) => {
                // Vertices (endpoints)
                if self.endpoint {
                    for vertex in &polygon.vertices {
                        snaps.push(
                            SnapResult::new(*vertex, SnapType::Endpoint)
                                .with_reference(polygon.id)
                                .with_distance(cursor.distance(vertex)),
                        );
                    }
                }

                // Midpoints of edges
                if self.midpoint {
                    let n = polygon.vertices.len();
                    for i in 0..n {
                        let j = (i + 1) % n;
                        let mid = polygon.vertices[i].midpoint(&polygon.vertices[j]);
                        snaps.push(
                            SnapResult::new(mid, SnapType::Midpoint)
                                .with_reference(polygon.id)
                                .with_distance(cursor.distance(&mid)),
                        );
                    }
                }

                // Center (centroid)
                if self.center {
                    let centroid = polygon.centroid();
                    snaps.push(
                        SnapResult::new(centroid, SnapType::Center)
                            .with_reference(polygon.id)
                            .with_distance(cursor.distance(&centroid)),
                    );
                }
            }

            _ => {
                // Handle other entity types
            }
        }

        snaps
    }

    /// Find nearest point on a line segment
    fn nearest_point_on_line(&self, point: &Point, line: &Line) -> Point {
        let line_vec = Point::new(line.end.x - line.start.x, line.end.y - line.start.y);
        let point_vec = Point::new(point.x - line.start.x, point.y - line.start.y);

        let line_length_sq = line_vec.x * line_vec.x + line_vec.y * line_vec.y;

        if line_length_sq < f64::EPSILON {
            return line.start;
        }

        let t = ((point_vec.x * line_vec.x + point_vec.y * line_vec.y) / line_length_sq)
            .clamp(0.0, 1.0);

        Point::new(
            line.start.x + t * line_vec.x,
            line.start.y + t * line_vec.y,
        )
    }

    /// Find perpendicular snap point
    pub fn snap_perpendicular(&self, from: &Point, to_line: &Line) -> Option<SnapResult> {
        if !self.perpendicular {
            return None;
        }

        let nearest = self.nearest_point_on_line(from, to_line);
        let distance = from.distance(&nearest);

        if distance <= self.snap_radius {
            Some(
                SnapResult::new(nearest, SnapType::Perpendicular)
                    .with_reference(to_line.id)
                    .with_distance(distance),
            )
        } else {
            None
        }
    }
}

/// Smart guides for alignment and distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartGuide {
    pub enabled: bool,
    pub tolerance: f64,
    pub show_horizontal: bool,
    pub show_vertical: bool,
    pub show_alignment: bool,
    pub show_distribution: bool,
}

impl Default for SmartGuide {
    fn default() -> Self {
        Self {
            enabled: true,
            tolerance: 2.0,
            show_horizontal: true,
            show_vertical: true,
            show_alignment: true,
            show_distribution: true,
        }
    }
}

impl SmartGuide {
    /// Find smart guide snaps
    pub fn snap(&self, point: &Point, reference_points: &[Point]) -> Vec<GuideResult> {
        if !self.enabled || reference_points.is_empty() {
            return Vec::new();
        }

        let mut guides = Vec::new();

        for ref_point in reference_points {
            // Horizontal alignment
            if self.show_horizontal && (point.y - ref_point.y).abs() < self.tolerance {
                guides.push(GuideResult {
                    guide_type: GuideType::Horizontal,
                    snap_point: Point::new(point.x, ref_point.y),
                    reference: *ref_point,
                    message: format!("Horizontal align: Y = {:.2}", ref_point.y),
                });
            }

            // Vertical alignment
            if self.show_vertical && (point.x - ref_point.x).abs() < self.tolerance {
                guides.push(GuideResult {
                    guide_type: GuideType::Vertical,
                    snap_point: Point::new(ref_point.x, point.y),
                    reference: *ref_point,
                    message: format!("Vertical align: X = {:.2}", ref_point.x),
                });
            }
        }

        // Check for alignment with multiple points
        if self.show_alignment && reference_points.len() >= 2 {
            guides.extend(self.find_alignment_guides(point, reference_points));
        }

        guides
    }

    /// Find alignment guides
    fn find_alignment_guides(&self, point: &Point, references: &[Point]) -> Vec<GuideResult> {
        let mut guides = Vec::new();

        // Check if point aligns horizontally with any two reference points
        for (i, p1) in references.iter().enumerate() {
            for p2 in references.iter().skip(i + 1) {
                if (p1.y - p2.y).abs() < self.tolerance && (point.y - p1.y).abs() < self.tolerance {
                    guides.push(GuideResult {
                        guide_type: GuideType::MultiAlignment,
                        snap_point: Point::new(point.x, p1.y),
                        reference: *p1,
                        message: "Aligned with multiple points".into(),
                    });
                }

                if (p1.x - p2.x).abs() < self.tolerance && (point.x - p1.x).abs() < self.tolerance {
                    guides.push(GuideResult {
                        guide_type: GuideType::MultiAlignment,
                        snap_point: Point::new(p1.x, point.y),
                        reference: *p1,
                        message: "Aligned with multiple points".into(),
                    });
                }
            }
        }

        guides
    }

    /// Find distribution guides
    pub fn find_distribution(&self, points: &[Point]) -> Vec<GuideResult> {
        if !self.show_distribution || points.len() < 3 {
            return Vec::new();
        }

        let mut guides = Vec::new();

        // Sort by x coordinate
        let mut x_sorted = points.to_vec();
        x_sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

        // Check for equal spacing
        for i in 0..x_sorted.len() - 2 {
            let spacing1 = x_sorted[i + 1].x - x_sorted[i].x;
            let spacing2 = x_sorted[i + 2].x - x_sorted[i + 1].x;

            if (spacing1 - spacing2).abs() < self.tolerance {
                guides.push(GuideResult {
                    guide_type: GuideType::Distribution,
                    snap_point: x_sorted[i + 1],
                    reference: x_sorted[i],
                    message: format!("Equal spacing: {:.2}", spacing1),
                });
            }
        }

        guides
    }
}

/// Guide result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideResult {
    pub guide_type: GuideType,
    pub snap_point: Point,
    pub reference: Point,
    pub message: String,
}

/// Types of smart guides
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuideType {
    Horizontal,
    Vertical,
    Diagonal,
    MultiAlignment,
    Distribution,
    Spacing,
}

/// Combined snapping system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapSystem {
    pub grid: GridSnap,
    pub object: ObjectSnap,
    pub smart_guide: SmartGuide,
    pub priority_order: Vec<SnapType>,
}

impl Default for SnapSystem {
    fn default() -> Self {
        Self {
            grid: GridSnap::default(),
            object: ObjectSnap::default(),
            smart_guide: SmartGuide::default(),
            priority_order: vec![
                SnapType::Endpoint,
                SnapType::Intersection,
                SnapType::Midpoint,
                SnapType::Center,
                SnapType::Perpendicular,
                SnapType::Nearest,
                SnapType::Grid,
            ],
        }
    }
}

impl SnapSystem {
    /// Perform comprehensive snapping
    pub fn snap(&self, point: &Point, layer: &Layer) -> SnapResult {
        let entities: Vec<&Entity> = layer.entities.iter().collect();

        // Try object snap first (higher priority)
        let object_snap = self.object.snap(point, &entities);
        if object_snap.snap_type != SnapType::None {
            return object_snap;
        }

        // Then try grid snap
        self.grid.snap(point)
    }

    /// Snap with custom entity list
    pub fn snap_to_entities(&self, point: &Point, entities: &[&Entity]) -> SnapResult {
        // Try object snap first
        let object_snap = self.object.snap(point, entities);
        if object_snap.snap_type != SnapType::None {
            return object_snap;
        }

        // Fall back to grid snap
        self.grid.snap(point)
    }

    /// Get smart guides for a point
    pub fn get_guides(&self, point: &Point, reference_points: &[Point]) -> Vec<GuideResult> {
        self.smart_guide.snap(point, reference_points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_snap() {
        let grid = GridSnap::new(10.0);
        let point = Point::new(12.3, 7.8);
        let result = grid.snap(&point);

        assert_eq!(result.point.x, 10.0);
        assert_eq!(result.point.y, 10.0);
        assert_eq!(result.snap_type, SnapType::Grid);
    }

    #[test]
    fn test_object_snap_endpoint() {
        let mut object_snap = ObjectSnap::default();
        object_snap.snap_radius = 5.0;

        let line = Line::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0));
        let entities = vec![&Entity::Line(line)];

        let cursor = Point::new(0.5, 0.5);
        let result = object_snap.snap(&cursor, &entities);

        assert_eq!(result.snap_type, SnapType::Endpoint);
        assert_eq!(result.point.x, 0.0);
        assert_eq!(result.point.y, 0.0);
    }

    #[test]
    fn test_smart_guide_horizontal() {
        let guide = SmartGuide::default();
        let reference = vec![Point::new(5.0, 10.0)];
        let point = Point::new(15.0, 10.1);

        let guides = guide.snap(&point, &reference);
        assert!(!guides.is_empty());
        assert_eq!(guides[0].guide_type, GuideType::Horizontal);
    }
}
