//! Geometry transformation operations including simplification and smoothing

use crate::error::{AnalysisError, Result};
use geo::{Coord, EuclideanDistance, LineString, Point, Polygon, Simplify};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Simplification algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimplificationAlgorithm {
    /// Douglas-Peucker algorithm
    DouglasPeucker,
    /// Visvalingam-Whyatt algorithm
    VisvalingamWhyatt,
    /// Vertex reduction by distance
    VertexReduction,
}

/// Simplify a line string using Douglas-Peucker algorithm
pub fn simplify_line_douglas_peucker(line: &LineString, tolerance: f64) -> Result<LineString> {
    if tolerance < 0.0 {
        return Err(AnalysisError::invalid_parameters(
            "Tolerance must be non-negative",
        ));
    }

    // Use geo's built-in simplify
    Ok(line.simplify(&tolerance))
}

/// Simplify a polygon using Douglas-Peucker algorithm
pub fn simplify_polygon_douglas_peucker(polygon: &Polygon, tolerance: f64) -> Result<Polygon> {
    if tolerance < 0.0 {
        return Err(AnalysisError::invalid_parameters(
            "Tolerance must be non-negative",
        ));
    }

    Ok(polygon.simplify(&tolerance))
}

/// Simplify by reducing vertices based on minimum distance
pub fn simplify_by_vertex_reduction(line: &LineString, min_distance: f64) -> Result<LineString> {
    if min_distance < 0.0 {
        return Err(AnalysisError::invalid_parameters(
            "Minimum distance must be non-negative",
        ));
    }

    let coords: Vec<Coord> = line.coords().cloned().collect();

    if coords.len() <= 2 {
        return Ok(line.clone());
    }

    let mut simplified = vec![coords[0]];
    let mut last_kept = coords[0];

    for coord in coords.iter().skip(1) {
        let dist = Point::from(last_kept).euclidean_distance(&Point::from(*coord));

        if dist >= min_distance {
            simplified.push(*coord);
            last_kept = *coord;
        }
    }

    // Always keep the last point
    if let Some(&last) = coords.last() {
        if simplified.last() != Some(&last) {
            simplified.push(last);
        }
    }

    Ok(LineString::from(simplified))
}

/// Smoothing algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SmoothingAlgorithm {
    /// Moving average smoothing
    MovingAverage,
    /// Bezier curve smoothing
    Bezier,
    /// Chaikin's algorithm
    Chaikin,
}

/// Smooth a line using moving average
pub fn smooth_line_moving_average(line: &LineString, window_size: usize) -> Result<LineString> {
    if window_size < 2 {
        return Err(AnalysisError::invalid_parameters(
            "Window size must be at least 2",
        ));
    }

    let coords: Vec<Coord> = line.coords().cloned().collect();

    if coords.len() < window_size {
        return Ok(line.clone());
    }

    let mut smoothed = Vec::new();

    for i in 0..coords.len() {
        let start = i.saturating_sub(window_size / 2);
        let end = (i + window_size / 2 + 1).min(coords.len());

        let window = &coords[start..end];
        let avg_x: f64 = window.iter().map(|c| c.x).sum::<f64>() / window.len() as f64;
        let avg_y: f64 = window.iter().map(|c| c.y).sum::<f64>() / window.len() as f64;

        smoothed.push(Coord { x: avg_x, y: avg_y });
    }

    Ok(LineString::from(smoothed))
}

/// Smooth a line using Chaikin's algorithm
pub fn smooth_line_chaikin(line: &LineString, iterations: usize) -> Result<LineString> {
    let coords: Vec<Coord> = line.coords().cloned().collect();

    if coords.len() < 2 {
        return Ok(line.clone());
    }

    let mut current = coords;

    for _ in 0..iterations {
        let mut next = Vec::new();

        // Keep first point
        next.push(current[0]);

        for i in 0..current.len() - 1 {
            let p0 = current[i];
            let p1 = current[i + 1];

            // Create two new points at 1/4 and 3/4 along the segment
            let q = Coord {
                x: 0.75 * p0.x + 0.25 * p1.x,
                y: 0.75 * p0.y + 0.25 * p1.y,
            };

            let r = Coord {
                x: 0.25 * p0.x + 0.75 * p1.x,
                y: 0.25 * p0.y + 0.75 * p1.y,
            };

            next.push(q);
            next.push(r);
        }

        // Keep last point
        if let Some(&last) = current.last() {
            next.push(last);
        }

        current = next;
    }

    Ok(LineString::from(current))
}

/// Densify a line by adding points at regular intervals
pub fn densify_line(line: &LineString, max_segment_length: f64) -> Result<LineString> {
    if max_segment_length <= 0.0 {
        return Err(AnalysisError::invalid_parameters(
            "Max segment length must be positive",
        ));
    }

    let coords: Vec<Coord> = line.coords().cloned().collect();

    if coords.len() < 2 {
        return Ok(line.clone());
    }

    let mut densified = vec![coords[0]];

    for i in 0..coords.len() - 1 {
        let p0 = coords[i];
        let p1 = coords[i + 1];

        let segment_length = Point::from(p0).euclidean_distance(&Point::from(p1));
        let num_segments = (segment_length / max_segment_length).ceil() as usize;

        if num_segments > 1 {
            for j in 1..num_segments {
                let t = j as f64 / num_segments as f64;
                let new_point = Coord {
                    x: p0.x + t * (p1.x - p0.x),
                    y: p0.y + t * (p1.y - p0.y),
                };
                densified.push(new_point);
            }
        }

        densified.push(p1);
    }

    Ok(LineString::from(densified))
}

/// Densify a polygon
pub fn densify_polygon(polygon: &Polygon, max_segment_length: f64) -> Result<Polygon> {
    let exterior = densify_line(polygon.exterior(), max_segment_length)?;

    let interiors: Result<Vec<LineString>> = polygon
        .interiors()
        .iter()
        .map(|ring| densify_line(ring, max_segment_length))
        .collect();

    Ok(Polygon::new(exterior, interiors?))
}

/// Generalize a line by removing vertices that don't contribute to shape
pub fn generalize_line(line: &LineString, tolerance: f64) -> Result<LineString> {
    simplify_line_douglas_peucker(line, tolerance)
}

/// Calculate bend simplification for lines (remove small bends)
pub fn remove_small_bends(line: &LineString, min_area: f64) -> Result<LineString> {
    if min_area < 0.0 {
        return Err(AnalysisError::invalid_parameters(
            "Minimum area must be non-negative",
        ));
    }

    let coords: Vec<Coord> = line.coords().cloned().collect();

    if coords.len() < 3 {
        return Ok(line.clone());
    }

    let mut simplified = vec![coords[0]];
    let mut i = 1;

    while i < coords.len() - 1 {
        let p0 = *simplified.last().unwrap();
        let p1 = coords[i];
        let p2 = coords[i + 1];

        // Calculate triangle area
        let area = triangle_area(&p0, &p1, &p2).abs();

        if area < min_area {
            // Skip this vertex
            i += 1;
        } else {
            simplified.push(p1);
            i += 1;
        }
    }

    // Always keep the last point
    if let Some(&last) = coords.last() {
        simplified.push(last);
    }

    Ok(LineString::from(simplified))
}

/// Calculate area of a triangle
fn triangle_area(p0: &Coord, p1: &Coord, p2: &Coord) -> f64 {
    0.5 * ((p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y))
}

/// Offset a line by a distance (parallel line)
pub fn offset_line(line: &LineString, distance: f64) -> Result<LineString> {
    let coords: Vec<Coord> = line.coords().cloned().collect();

    if coords.len() < 2 {
        return Err(AnalysisError::invalid_geometry(
            "Line must have at least 2 points",
        ));
    }

    let mut offset_coords = Vec::new();

    for i in 0..coords.len() - 1 {
        let p1 = coords[i];
        let p2 = coords[i + 1];

        // Calculate perpendicular offset
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let length = (dx * dx + dy * dy).sqrt();

        if length > 0.0 {
            let offset_x = -dy / length * distance;
            let offset_y = dx / length * distance;

            offset_coords.push(Coord {
                x: p1.x + offset_x,
                y: p1.y + offset_y,
            });

            if i == coords.len() - 2 {
                offset_coords.push(Coord {
                    x: p2.x + offset_x,
                    y: p2.y + offset_y,
                });
            }
        }
    }

    Ok(LineString::from(offset_coords))
}

/// Reverse the direction of a line
pub fn reverse_line(line: &LineString) -> LineString {
    let coords: Vec<Coord> = line.coords().rev().cloned().collect();
    LineString::from(coords)
}

/// Split a line at a specific point
pub fn split_line_at_point(line: &LineString, point: &Point) -> Result<Vec<LineString>> {
    let coords: Vec<Coord> = line.coords().cloned().collect();

    if coords.len() < 2 {
        return Err(AnalysisError::invalid_geometry(
            "Line must have at least 2 points",
        ));
    }

    // Find closest segment
    let mut min_dist = f64::INFINITY;
    let mut split_index = 0;

    for i in 0..coords.len() - 1 {
        let p1 = Point::from(coords[i]);
        let p2 = Point::from(coords[i + 1]);

        let dist = point_to_segment_distance(point, &p1, &p2);

        if dist < min_dist {
            min_dist = dist;
            split_index = i;
        }
    }

    // Create two new lines
    let mut first_coords: Vec<Coord> = coords[..=split_index].to_vec();
    first_coords.push(point.0.into());

    let mut second_coords = vec![point.0.into()];
    second_coords.extend_from_slice(&coords[split_index + 1..]);

    Ok(vec![
        LineString::from(first_coords),
        LineString::from(second_coords),
    ])
}

/// Calculate distance from point to line segment
fn point_to_segment_distance(point: &Point, p1: &Point, p2: &Point) -> f64 {
    let dx = p2.x() - p1.x();
    let dy = p2.y() - p1.y();
    let length_sq = dx * dx + dy * dy;

    if length_sq == 0.0 {
        return point.euclidean_distance(p1);
    }

    let t = ((point.x() - p1.x()) * dx + (point.y() - p1.y()) * dy) / length_sq;
    let t = t.clamp(0.0, 1.0);

    let closest = Point::new(p1.x() + t * dx, p1.y() + t * dy);
    point.euclidean_distance(&closest)
}

/// Merge consecutive line segments
pub fn merge_lines(lines: &[LineString]) -> Result<LineString> {
    if lines.is_empty() {
        return Err(AnalysisError::InsufficientData {
            required: 1,
            actual: 0,
        });
    }

    if lines.len() == 1 {
        return Ok(lines[0].clone());
    }

    let mut merged_coords = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let coords: Vec<Coord> = line.coords().cloned().collect();

        if i == 0 {
            merged_coords.extend(coords);
        } else {
            // Skip first coordinate if it matches the last coordinate of previous line
            let skip = if let (Some(&last), Some(&first)) = (merged_coords.last(), coords.first())
            {
                (last.x - first.x).abs() < 1e-10 && (last.y - first.y).abs() < 1e-10
            } else {
                false
            };

            if skip {
                merged_coords.extend(coords.iter().skip(1));
            } else {
                merged_coords.extend(coords);
            }
        }
    }

    Ok(LineString::from(merged_coords))
}

/// Simplify geometries in parallel
pub fn simplify_lines_parallel(
    lines: &[LineString],
    tolerance: f64,
) -> Result<Vec<LineString>> {
    lines
        .par_iter()
        .map(|line| simplify_line_douglas_peucker(line, tolerance))
        .collect()
}

/// Smooth geometries in parallel
pub fn smooth_lines_parallel(
    lines: &[LineString],
    window_size: usize,
) -> Result<Vec<LineString>> {
    lines
        .par_iter()
        .map(|line| smooth_line_moving_average(line, window_size))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_line() {
        let line = LineString::from(vec![
            (0.0, 0.0),
            (1.0, 0.1),
            (2.0, -0.1),
            (3.0, 0.0),
            (10.0, 0.0),
        ]);

        let simplified = simplify_line_douglas_peucker(&line, 0.5).unwrap();
        assert!(simplified.coords_count() <= line.coords_count());
    }

    #[test]
    fn test_vertex_reduction() {
        let line = LineString::from(vec![
            (0.0, 0.0),
            (0.1, 0.0),
            (0.2, 0.0),
            (10.0, 0.0),
        ]);

        let simplified = simplify_by_vertex_reduction(&line, 1.0).unwrap();
        assert!(simplified.coords_count() < line.coords_count());
    }

    #[test]
    fn test_smooth_moving_average() {
        let line = LineString::from(vec![
            (0.0, 0.0),
            (1.0, 1.0),
            (2.0, 0.0),
            (3.0, 1.0),
            (4.0, 0.0),
        ]);

        let smoothed = smooth_line_moving_average(&line, 3).unwrap();
        assert_eq!(smoothed.coords_count(), line.coords_count());
    }

    #[test]
    fn test_chaikin_smoothing() {
        let line = LineString::from(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]);

        let smoothed = smooth_line_chaikin(&line, 1).unwrap();
        assert!(smoothed.coords_count() > line.coords_count());
    }

    #[test]
    fn test_densify_line() {
        let line = LineString::from(vec![(0.0, 0.0), (10.0, 0.0)]);

        let densified = densify_line(&line, 2.0).unwrap();
        assert!(densified.coords_count() > line.coords_count());
    }

    #[test]
    fn test_reverse_line() {
        let line = LineString::from(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]);
        let reversed = reverse_line(&line);

        let orig_coords: Vec<Coord> = line.coords().cloned().collect();
        let rev_coords: Vec<Coord> = reversed.coords().cloned().collect();

        assert_eq!(orig_coords[0], rev_coords[rev_coords.len() - 1]);
        assert_eq!(orig_coords[orig_coords.len() - 1], rev_coords[0]);
    }

    #[test]
    fn test_merge_lines() {
        let line1 = LineString::from(vec![(0.0, 0.0), (5.0, 0.0)]);
        let line2 = LineString::from(vec![(5.0, 0.0), (10.0, 0.0)]);

        let merged = merge_lines(&[line1, line2]).unwrap();
        assert_eq!(merged.coords_count(), 3); // Shared point should not be duplicated
    }
}
