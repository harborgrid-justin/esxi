//! Vector Primitives for CAD Engine
//!
//! This module provides fundamental geometric primitives used throughout the CAD engine.
//! All primitives support serialization, high-precision arithmetic, and geometric operations.

use nalgebra::{Point2, Vector2};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use uuid::Uuid;

use crate::{CadError, CadResult};

/// 2D Point with high-precision coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64, // For 3D support in future versions
}

impl Point {
    /// Create a new 2D point
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y, z: 0.0 }
    }

    /// Create a new 3D point
    pub fn new_3d(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Create point at origin
    pub fn origin() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Calculate distance to another point
    pub fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }

    /// Calculate 2D distance (ignoring z)
    pub fn distance_2d(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Get midpoint between two points
    pub fn midpoint(&self, other: &Point) -> Point {
        Point::new_3d(
            (self.x + other.x) / 2.0,
            (self.y + other.y) / 2.0,
            (self.z + other.z) / 2.0,
        )
    }

    /// Convert to nalgebra Point2
    pub fn to_nalgebra(&self) -> Point2<f64> {
        Point2::new(self.x, self.y)
    }

    /// Convert to nalgebra Vector2
    pub fn to_vector(&self) -> Vector2<f64> {
        Vector2::new(self.x, self.y)
    }

    /// Rotate point around origin by angle (radians)
    pub fn rotate(&self, angle: f64) -> Point {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Point::new(
            self.x * cos_a - self.y * sin_a,
            self.x * sin_a + self.y * cos_a,
        )
    }

    /// Rotate point around a center point
    pub fn rotate_around(&self, center: &Point, angle: f64) -> Point {
        let translated = Point::new(self.x - center.x, self.y - center.y);
        let rotated = translated.rotate(angle);
        Point::new(rotated.x + center.x, rotated.y + center.y)
    }

    /// Scale point from origin
    pub fn scale(&self, factor: f64) -> Point {
        Point::new_3d(self.x * factor, self.y * factor, self.z * factor)
    }

    /// Translate point by vector
    pub fn translate(&self, dx: f64, dy: f64) -> Point {
        Point::new_3d(self.x + dx, self.y + dy, self.z)
    }
}

impl From<(f64, f64)> for Point {
    fn from((x, y): (f64, f64)) -> Self {
        Point::new(x, y)
    }
}

impl From<Point2<f64>> for Point {
    fn from(p: Point2<f64>) -> Self {
        Point::new(p.x, p.y)
    }
}

/// Straight line segment between two points
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub id: Uuid,
    pub start: Point,
    pub end: Point,
    pub style: LineStyle,
}

impl Line {
    /// Create a new line
    pub fn new(start: Point, end: Point) -> Self {
        Self {
            id: Uuid::new_v4(),
            start,
            end,
            style: LineStyle::default(),
        }
    }

    /// Get line length
    pub fn length(&self) -> f64 {
        self.start.distance(&self.end)
    }

    /// Get line direction vector (normalized)
    pub fn direction(&self) -> Vector2<f64> {
        let v = Vector2::new(self.end.x - self.start.x, self.end.y - self.start.y);
        v.normalize()
    }

    /// Get angle of line in radians
    pub fn angle(&self) -> f64 {
        (self.end.y - self.start.y).atan2(self.end.x - self.start.x)
    }

    /// Get point at parameter t (0.0 to 1.0)
    pub fn point_at(&self, t: f64) -> Point {
        Point::new(
            self.start.x + t * (self.end.x - self.start.x),
            self.start.y + t * (self.end.y - self.start.y),
        )
    }

    /// Get midpoint of line
    pub fn midpoint(&self) -> Point {
        self.start.midpoint(&self.end)
    }

    /// Check if point is on line (within tolerance)
    pub fn contains_point(&self, point: &Point, tolerance: f64) -> bool {
        let dist = self.distance_to_point(point);
        dist < tolerance
    }

    /// Calculate perpendicular distance from point to line
    pub fn distance_to_point(&self, point: &Point) -> f64 {
        let line_vec = Vector2::new(self.end.x - self.start.x, self.end.y - self.start.y);
        let point_vec = Vector2::new(point.x - self.start.x, point.y - self.start.y);

        let line_length = line_vec.norm();
        if line_length < f64::EPSILON {
            return point.distance(&self.start);
        }

        let cross = line_vec.x * point_vec.y - line_vec.y * point_vec.x;
        cross.abs() / line_length
    }

    /// Get intersection point with another line
    pub fn intersection(&self, other: &Line) -> Option<Point> {
        let x1 = self.start.x;
        let y1 = self.start.y;
        let x2 = self.end.x;
        let y2 = self.end.y;
        let x3 = other.start.x;
        let y3 = other.start.y;
        let x4 = other.end.x;
        let y4 = other.end.y;

        let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        if denom.abs() < f64::EPSILON {
            return None; // Lines are parallel
        }

        let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
        let u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;

        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            Some(Point::new(x1 + t * (x2 - x1), y1 + t * (y2 - y1)))
        } else {
            None
        }
    }
}

/// Circular arc defined by center, radius, and angle range
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arc {
    pub id: Uuid,
    pub center: Point,
    pub radius: f64,
    pub start_angle: f64, // radians
    pub end_angle: f64,   // radians
    pub style: LineStyle,
}

impl Arc {
    /// Create a new arc
    pub fn new(center: Point, radius: f64, start_angle: f64, end_angle: f64) -> CadResult<Self> {
        if radius <= 0.0 {
            return Err(CadError::InvalidGeometry("Arc radius must be positive".into()));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            center,
            radius,
            start_angle,
            end_angle,
            style: LineStyle::default(),
        })
    }

    /// Create a full circle
    pub fn circle(center: Point, radius: f64) -> CadResult<Self> {
        Self::new(center, radius, 0.0, 2.0 * PI)
    }

    /// Get arc length
    pub fn length(&self) -> f64 {
        let angle_span = (self.end_angle - self.start_angle).abs();
        self.radius * angle_span
    }

    /// Get point at angle
    pub fn point_at_angle(&self, angle: f64) -> Point {
        Point::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
        )
    }

    /// Get start point
    pub fn start_point(&self) -> Point {
        self.point_at_angle(self.start_angle)
    }

    /// Get end point
    pub fn end_point(&self) -> Point {
        self.point_at_angle(self.end_angle)
    }

    /// Get point at parameter t (0.0 to 1.0)
    pub fn point_at(&self, t: f64) -> Point {
        let angle = self.start_angle + t * (self.end_angle - self.start_angle);
        self.point_at_angle(angle)
    }

    /// Check if point is on arc (within tolerance)
    pub fn contains_point(&self, point: &Point, tolerance: f64) -> bool {
        let dist_to_center = self.center.distance(point);
        let radius_diff = (dist_to_center - self.radius).abs();

        if radius_diff > tolerance {
            return false;
        }

        let angle = (point.y - self.center.y).atan2(point.x - self.center.x);
        let normalized_angle = if angle < self.start_angle {
            angle + 2.0 * PI
        } else {
            angle
        };

        normalized_angle >= self.start_angle && normalized_angle <= self.end_angle
    }
}

/// Cubic Bezier curve
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bezier {
    pub id: Uuid,
    pub p0: Point, // Start point
    pub p1: Point, // First control point
    pub p2: Point, // Second control point
    pub p3: Point, // End point
    pub style: LineStyle,
}

impl Bezier {
    /// Create a new cubic Bezier curve
    pub fn new(p0: Point, p1: Point, p2: Point, p3: Point) -> Self {
        Self {
            id: Uuid::new_v4(),
            p0,
            p1,
            p2,
            p3,
            style: LineStyle::default(),
        }
    }

    /// Get point at parameter t (0.0 to 1.0) using De Casteljau's algorithm
    pub fn point_at(&self, t: f64) -> Point {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        Point::new(
            mt3 * self.p0.x + 3.0 * mt2 * t * self.p1.x + 3.0 * mt * t2 * self.p2.x + t3 * self.p3.x,
            mt3 * self.p0.y + 3.0 * mt2 * t * self.p1.y + 3.0 * mt * t2 * self.p2.y + t3 * self.p3.y,
        )
    }

    /// Get tangent vector at parameter t
    pub fn tangent_at(&self, t: f64) -> Vector2<f64> {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;

        let dx = 3.0 * mt2 * (self.p1.x - self.p0.x)
            + 6.0 * mt * t * (self.p2.x - self.p1.x)
            + 3.0 * t2 * (self.p3.x - self.p2.x);

        let dy = 3.0 * mt2 * (self.p1.y - self.p0.y)
            + 6.0 * mt * t * (self.p2.y - self.p1.y)
            + 3.0 * t2 * (self.p3.y - self.p2.y);

        Vector2::new(dx, dy).normalize()
    }

    /// Approximate curve length using adaptive subdivision
    pub fn length(&self, tolerance: f64) -> f64 {
        self.adaptive_length(0.0, 1.0, tolerance)
    }

    fn adaptive_length(&self, t0: f64, t1: f64, tolerance: f64) -> f64 {
        let p0 = self.point_at(t0);
        let p1 = self.point_at(t1);
        let pm = self.point_at((t0 + t1) / 2.0);

        let chord = p0.distance(&p1);
        let subdivided = p0.distance(&pm) + pm.distance(&p1);

        if (subdivided - chord).abs() < tolerance {
            subdivided
        } else {
            let mid = (t0 + t1) / 2.0;
            self.adaptive_length(t0, mid, tolerance) + self.adaptive_length(mid, t1, tolerance)
        }
    }

    /// Split curve at parameter t into two curves
    pub fn split(&self, t: f64) -> (Bezier, Bezier) {
        // De Casteljau's algorithm for splitting
        let p01 = Point::new(
            self.p0.x + t * (self.p1.x - self.p0.x),
            self.p0.y + t * (self.p1.y - self.p0.y),
        );
        let p12 = Point::new(
            self.p1.x + t * (self.p2.x - self.p1.x),
            self.p1.y + t * (self.p2.y - self.p1.y),
        );
        let p23 = Point::new(
            self.p2.x + t * (self.p3.x - self.p2.x),
            self.p2.y + t * (self.p3.y - self.p2.y),
        );

        let p012 = Point::new(p01.x + t * (p12.x - p01.x), p01.y + t * (p12.y - p01.y));
        let p123 = Point::new(p12.x + t * (p23.x - p12.x), p12.y + t * (p23.y - p12.y));

        let p0123 = Point::new(
            p012.x + t * (p123.x - p012.x),
            p012.y + t * (p123.y - p012.y),
        );

        (
            Bezier::new(self.p0, p01, p012, p0123),
            Bezier::new(p0123, p123, p23, self.p3),
        )
    }
}

/// B-Spline curve with control points
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spline {
    pub id: Uuid,
    pub control_points: Vec<Point>,
    pub degree: usize,
    pub knots: Vec<f64>,
    pub style: LineStyle,
}

impl Spline {
    /// Create a new B-spline curve
    pub fn new(control_points: Vec<Point>, degree: usize) -> CadResult<Self> {
        if control_points.len() < degree + 1 {
            return Err(CadError::InvalidGeometry(
                "Not enough control points for spline degree".into(),
            ));
        }

        let n = control_points.len();
        let m = n + degree + 1;
        let mut knots = vec![0.0; m];

        // Create uniform knot vector
        for i in 0..m {
            if i <= degree {
                knots[i] = 0.0;
            } else if i >= m - degree - 1 {
                knots[i] = 1.0;
            } else {
                knots[i] = (i - degree) as f64 / (m - 2 * degree - 1) as f64;
            }
        }

        Ok(Self {
            id: Uuid::new_v4(),
            control_points,
            degree,
            knots,
            style: LineStyle::default(),
        })
    }

    /// Evaluate spline at parameter t using Cox-de Boor recursion
    pub fn point_at(&self, t: f64) -> Point {
        let n = self.control_points.len();
        let mut points = self.control_points.clone();

        for r in 1..=self.degree {
            for i in (r..n).rev() {
                let knot_diff = self.knots[i + self.degree - r + 1] - self.knots[i];
                let alpha = if knot_diff.abs() < f64::EPSILON {
                    0.0
                } else {
                    (t - self.knots[i]) / knot_diff
                };

                points[i] = Point::new(
                    (1.0 - alpha) * points[i - 1].x + alpha * points[i].x,
                    (1.0 - alpha) * points[i - 1].y + alpha * points[i].y,
                );
            }
        }

        points[n - 1]
    }

    /// Get curve bounds
    pub fn bounds(&self) -> (Point, Point) {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for point in &self.control_points {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        (Point::new(min_x, min_y), Point::new(max_x, max_y))
    }
}

/// Closed polygon defined by vertices
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    pub id: Uuid,
    pub vertices: Vec<Point>,
    pub style: LineStyle,
    pub fill: Option<FillStyle>,
}

impl Polygon {
    /// Create a new polygon
    pub fn new(vertices: Vec<Point>) -> CadResult<Self> {
        if vertices.len() < 3 {
            return Err(CadError::InvalidGeometry(
                "Polygon must have at least 3 vertices".into(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            vertices,
            style: LineStyle::default(),
            fill: None,
        })
    }

    /// Create a rectangle
    pub fn rectangle(x: f64, y: f64, width: f64, height: f64) -> CadResult<Self> {
        let vertices = vec![
            Point::new(x, y),
            Point::new(x + width, y),
            Point::new(x + width, y + height),
            Point::new(x, y + height),
        ];
        Self::new(vertices)
    }

    /// Create a regular polygon
    pub fn regular(center: Point, radius: f64, sides: usize) -> CadResult<Self> {
        if sides < 3 {
            return Err(CadError::InvalidGeometry(
                "Regular polygon must have at least 3 sides".into(),
            ));
        }

        let mut vertices = Vec::with_capacity(sides);
        let angle_step = 2.0 * PI / sides as f64;

        for i in 0..sides {
            let angle = i as f64 * angle_step;
            vertices.push(Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ));
        }

        Self::new(vertices)
    }

    /// Calculate polygon area using shoelace formula
    pub fn area(&self) -> f64 {
        let n = self.vertices.len();
        let mut area = 0.0;

        for i in 0..n {
            let j = (i + 1) % n;
            area += self.vertices[i].x * self.vertices[j].y;
            area -= self.vertices[j].x * self.vertices[i].y;
        }

        area.abs() / 2.0
    }

    /// Calculate polygon perimeter
    pub fn perimeter(&self) -> f64 {
        let n = self.vertices.len();
        let mut perimeter = 0.0;

        for i in 0..n {
            let j = (i + 1) % n;
            perimeter += self.vertices[i].distance(&self.vertices[j]);
        }

        perimeter
    }

    /// Check if point is inside polygon using ray casting
    pub fn contains_point(&self, point: &Point) -> bool {
        let n = self.vertices.len();
        let mut inside = false;

        let mut j = n - 1;
        for i in 0..n {
            let vi = &self.vertices[i];
            let vj = &self.vertices[j];

            if ((vi.y > point.y) != (vj.y > point.y))
                && (point.x < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y) + vi.x)
            {
                inside = !inside;
            }
            j = i;
        }

        inside
    }

    /// Get polygon centroid
    pub fn centroid(&self) -> Point {
        let n = self.vertices.len() as f64;
        let sum_x: f64 = self.vertices.iter().map(|p| p.x).sum();
        let sum_y: f64 = self.vertices.iter().map(|p| p.y).sum();

        Point::new(sum_x / n, sum_y / n)
    }

    /// Get bounding box
    pub fn bounds(&self) -> (Point, Point) {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for point in &self.vertices {
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        (Point::new(min_x, min_y), Point::new(max_x, max_y))
    }
}

/// Ellipse defined by center, radii, and rotation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ellipse {
    pub id: Uuid,
    pub center: Point,
    pub radius_x: f64,
    pub radius_y: f64,
    pub rotation: f64, // radians
    pub style: LineStyle,
    pub fill: Option<FillStyle>,
}

impl Ellipse {
    /// Create a new ellipse
    pub fn new(center: Point, radius_x: f64, radius_y: f64, rotation: f64) -> CadResult<Self> {
        if radius_x <= 0.0 || radius_y <= 0.0 {
            return Err(CadError::InvalidGeometry(
                "Ellipse radii must be positive".into(),
            ));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            center,
            radius_x,
            radius_y,
            rotation,
            style: LineStyle::default(),
            fill: None,
        })
    }

    /// Create a circle (special case of ellipse)
    pub fn circle(center: Point, radius: f64) -> CadResult<Self> {
        Self::new(center, radius, radius, 0.0)
    }

    /// Get point on ellipse at angle (radians)
    pub fn point_at_angle(&self, angle: f64) -> Point {
        let x = self.radius_x * angle.cos();
        let y = self.radius_y * angle.sin();

        // Apply rotation
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        Point::new(
            self.center.x + x * cos_r - y * sin_r,
            self.center.y + x * sin_r + y * cos_r,
        )
    }

    /// Calculate ellipse area
    pub fn area(&self) -> f64 {
        PI * self.radius_x * self.radius_y
    }

    /// Calculate ellipse perimeter (Ramanujan approximation)
    pub fn perimeter(&self) -> f64 {
        let a = self.radius_x;
        let b = self.radius_y;
        let h = ((a - b).powi(2)) / ((a + b).powi(2));
        PI * (a + b) * (1.0 + (3.0 * h) / (10.0 + (4.0 - 3.0 * h).sqrt()))
    }

    /// Check if point is inside ellipse
    pub fn contains_point(&self, point: &Point) -> bool {
        // Translate and rotate point to ellipse coordinate system
        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;

        let cos_r = (-self.rotation).cos();
        let sin_r = (-self.rotation).sin();

        let x = dx * cos_r - dy * sin_r;
        let y = dx * sin_r + dy * cos_r;

        // Check ellipse equation
        (x * x) / (self.radius_x * self.radius_x) + (y * y) / (self.radius_y * self.radius_y) <= 1.0
    }

    /// Get bounding box
    pub fn bounds(&self) -> (Point, Point) {
        // For rotated ellipse, we need to find the extrema
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();

        let a = self.radius_x;
        let b = self.radius_y;

        let dx = (a * a * cos_r * cos_r + b * b * sin_r * sin_r).sqrt();
        let dy = (a * a * sin_r * sin_r + b * b * cos_r * cos_r).sqrt();

        (
            Point::new(self.center.x - dx, self.center.y - dy),
            Point::new(self.center.x + dx, self.center.y + dy),
        )
    }
}

/// Line style for rendering
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineStyle {
    pub color: Color,
    pub width: f64,
    pub dash_pattern: Vec<f64>,
}

impl Default for LineStyle {
    fn default() -> Self {
        Self {
            color: Color::black(),
            width: 1.0,
            dash_pattern: vec![],
        }
    }
}

/// Fill style for closed shapes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillStyle {
    pub color: Color,
    pub opacity: f64,
}

impl Default for FillStyle {
    fn default() -> Self {
        Self {
            color: Color::white(),
            opacity: 1.0,
        }
    }
}

/// RGBA Color
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub fn black() -> Self {
        Self::rgb(0, 0, 0)
    }

    pub fn white() -> Self {
        Self::rgb(255, 255, 255)
    }

    pub fn red() -> Self {
        Self::rgb(255, 0, 0)
    }

    pub fn green() -> Self {
        Self::rgb(0, 255, 0)
    }

    pub fn blue() -> Self {
        Self::rgb(0, 0, 255)
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert_eq!(p1.distance(&p2), 5.0);
    }

    #[test]
    fn test_line_length() {
        let line = Line::new(Point::new(0.0, 0.0), Point::new(3.0, 4.0));
        assert_eq!(line.length(), 5.0);
    }

    #[test]
    fn test_polygon_area() {
        let poly = Polygon::rectangle(0.0, 0.0, 10.0, 5.0).unwrap();
        assert_eq!(poly.area(), 50.0);
    }

    #[test]
    fn test_ellipse_area() {
        let ellipse = Ellipse::circle(Point::new(0.0, 0.0), 10.0).unwrap();
        assert!((ellipse.area() - 314.159).abs() < 0.01);
    }
}
