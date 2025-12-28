//! Buffer analysis operations for points, lines, and polygons

use crate::error::{AnalysisError, Result};
use geo::{
    Coord, LineString, MultiPolygon,
    Point, Polygon,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// End cap style for line buffers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapStyle {
    /// Rounded end caps (default)
    Round,
    /// Flat end caps
    Flat,
    /// Square end caps
    Square,
}

/// Join style for connecting buffer segments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JoinStyle {
    /// Rounded joins (default)
    Round,
    /// Mitered joins
    Miter,
    /// Beveled joins
    Bevel,
}

/// Buffer parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferParams {
    /// Buffer distance
    pub distance: f64,
    /// Number of segments for rounded portions (higher = smoother)
    pub quadrant_segments: usize,
    /// End cap style for line buffers
    pub cap_style: CapStyle,
    /// Join style for connecting segments
    pub join_style: JoinStyle,
    /// Miter limit ratio (for miter joins)
    pub miter_limit: f64,
    /// Single-sided buffer (for lines/polygons)
    pub single_sided: bool,
}

impl Default for BufferParams {
    fn default() -> Self {
        Self {
            distance: 0.0,
            quadrant_segments: 8,
            cap_style: CapStyle::Round,
            join_style: JoinStyle::Round,
            miter_limit: 5.0,
            single_sided: false,
        }
    }
}

impl BufferParams {
    /// Create new buffer parameters with a distance
    pub fn new(distance: f64) -> Self {
        Self {
            distance,
            ..Default::default()
        }
    }

    /// Set quadrant segments
    pub fn quadrant_segments(mut self, segments: usize) -> Self {
        self.quadrant_segments = segments.max(1);
        self
    }

    /// Set cap style
    pub fn cap_style(mut self, style: CapStyle) -> Self {
        self.cap_style = style;
        self
    }

    /// Set join style
    pub fn join_style(mut self, style: JoinStyle) -> Self {
        self.join_style = style;
        self
    }

    /// Set miter limit
    pub fn miter_limit(mut self, limit: f64) -> Self {
        self.miter_limit = limit.max(1.0);
        self
    }

    /// Set single-sided buffer
    pub fn single_sided(mut self, single_sided: bool) -> Self {
        self.single_sided = single_sided;
        self
    }

    /// Validate parameters
    pub fn validate(&self) -> Result<()> {
        if self.distance.is_nan() || self.distance.is_infinite() {
            return Err(AnalysisError::invalid_parameters(
                "Buffer distance must be finite",
            ));
        }
        if self.quadrant_segments == 0 {
            return Err(AnalysisError::invalid_parameters(
                "Quadrant segments must be at least 1",
            ));
        }
        if self.miter_limit < 1.0 {
            return Err(AnalysisError::invalid_parameters(
                "Miter limit must be at least 1.0",
            ));
        }
        Ok(())
    }
}

/// Buffer a point to create a circular polygon
pub fn buffer_point(point: &Point, params: &BufferParams) -> Result<Polygon> {
    params.validate()?;

    if params.distance == 0.0 {
        return Err(AnalysisError::BufferError(
            "Cannot create zero-distance buffer".to_string(),
        ));
    }

    let distance = params.distance.abs();
    let num_points = params.quadrant_segments * 4;
    let angle_step = 2.0 * PI / num_points as f64;

    let coords: Vec<Coord> = (0..=num_points)
        .map(|i| {
            let angle = i as f64 * angle_step;
            Coord {
                x: point.x() + distance * angle.cos(),
                y: point.y() + distance * angle.sin(),
            }
        })
        .collect();

    Ok(Polygon::new(LineString::from(coords), vec![]))
}

/// Buffer multiple points in parallel
pub fn buffer_points(points: &[Point], params: &BufferParams) -> Result<Vec<Polygon>> {
    params.validate()?;

    points
        .par_iter()
        .map(|point| buffer_point(point, params))
        .collect()
}

/// Buffer a line string
pub fn buffer_line(line: &LineString, params: &BufferParams) -> Result<Polygon> {
    params.validate()?;

    if line.coords_count() < 2 {
        return Err(AnalysisError::invalid_geometry(
            "Line must have at least 2 points",
        ));
    }

    if params.distance == 0.0 {
        return Err(AnalysisError::BufferError(
            "Cannot create zero-distance buffer".to_string(),
        ));
    }

    let distance = params.distance;
    let mut left_coords = Vec::new();
    let mut right_coords = Vec::new();

    // Generate offset points on both sides of the line
    let coords: Vec<Coord> = line.coords().cloned().collect();

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

            left_coords.push(Coord {
                x: p1.x + offset_x,
                y: p1.y + offset_y,
            });
            right_coords.push(Coord {
                x: p1.x - offset_x,
                y: p1.y - offset_y,
            });

            if i == coords.len() - 2 {
                left_coords.push(Coord {
                    x: p2.x + offset_x,
                    y: p2.y + offset_y,
                });
                right_coords.push(Coord {
                    x: p2.x - offset_x,
                    y: p2.y - offset_y,
                });
            }
        }
    }

    // Add end caps
    if !params.single_sided {
        match params.cap_style {
            CapStyle::Round => {
                let end_cap = generate_round_cap(
                    coords.last().unwrap(),
                    &coords[coords.len() - 2],
                    distance,
                    params.quadrant_segments,
                );
                left_coords.extend(end_cap);

                let start_cap = generate_round_cap(
                    &coords[0],
                    &coords[1],
                    -distance,
                    params.quadrant_segments,
                );
                right_coords.extend(start_cap);
            }
            CapStyle::Flat => {
                // No additional points needed
            }
            CapStyle::Square => {
                // Extend by distance at ends
                if let (Some(&last), Some(&second_last)) =
                    (coords.last(), coords.get(coords.len() - 2))
                {
                    let dx = last.x - second_last.x;
                    let dy = last.y - second_last.y;
                    let length = (dx * dx + dy * dy).sqrt();
                    if length > 0.0 {
                        let ext_x = dx / length * distance;
                        let ext_y = dy / length * distance;
                        if let Some(last_left) = left_coords.last_mut() {
                            last_left.x += ext_x;
                            last_left.y += ext_y;
                        }
                    }
                }
            }
        }
    }

    // Combine left and right sides
    right_coords.reverse();
    left_coords.extend(right_coords);

    if !left_coords.is_empty() {
        left_coords.push(left_coords[0]);
    }

    Ok(Polygon::new(LineString::from(left_coords), vec![]))
}

/// Buffer a polygon (expand or contract)
pub fn buffer_polygon(polygon: &Polygon, params: &BufferParams) -> Result<Polygon> {
    params.validate()?;

    if params.distance == 0.0 {
        return Ok(polygon.clone());
    }

    // Simple implementation: buffer the exterior ring and holes separately
    let exterior = polygon.exterior();
    let buffered_exterior = buffer_line(exterior, params)?;

    // For negative buffers (erosion), we would need more sophisticated handling
    // This is a simplified implementation
    Ok(buffered_exterior)
}

/// Generate a round cap at the end of a line
fn generate_round_cap(
    end_point: &Coord,
    prev_point: &Coord,
    distance: f64,
    quadrant_segments: usize,
) -> Vec<Coord> {
    let dx = end_point.x - prev_point.x;
    let dy = end_point.y - prev_point.y;
    let base_angle = dy.atan2(dx);

    let angle_step = PI / quadrant_segments as f64;
    let abs_distance = distance.abs();

    (0..=quadrant_segments)
        .map(|i| {
            let angle = if distance > 0.0 {
                base_angle + PI / 2.0 + i as f64 * angle_step
            } else {
                base_angle - PI / 2.0 + i as f64 * angle_step
            };

            Coord {
                x: end_point.x + abs_distance * angle.cos(),
                y: end_point.y + abs_distance * angle.sin(),
            }
        })
        .collect()
}

/// Create a variable-width buffer along a line
pub fn variable_buffer(
    line: &LineString,
    distances: &[f64],
    params: &BufferParams,
) -> Result<Polygon> {
    if line.coords_count() != distances.len() {
        return Err(AnalysisError::invalid_parameters(
            "Number of distances must match number of line points",
        ));
    }

    params.validate()?;

    if line.coords_count() < 2 {
        return Err(AnalysisError::invalid_geometry(
            "Line must have at least 2 points",
        ));
    }

    let mut left_coords = Vec::new();
    let mut right_coords = Vec::new();

    let coords: Vec<Coord> = line.coords().cloned().collect();

    for i in 0..coords.len() - 1 {
        let p1 = coords[i];
        let p2 = coords[i + 1];
        let d1 = distances[i].abs();
        let d2 = distances[i + 1].abs();

        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let length = (dx * dx + dy * dy).sqrt();

        if length > 0.0 {
            let offset_x1 = -dy / length * d1;
            let offset_y1 = dx / length * d1;
            let offset_x2 = -dy / length * d2;
            let offset_y2 = dx / length * d2;

            left_coords.push(Coord {
                x: p1.x + offset_x1,
                y: p1.y + offset_y1,
            });
            right_coords.push(Coord {
                x: p1.x - offset_x1,
                y: p1.y - offset_y1,
            });

            if i == coords.len() - 2 {
                left_coords.push(Coord {
                    x: p2.x + offset_x2,
                    y: p2.y + offset_y2,
                });
                right_coords.push(Coord {
                    x: p2.x - offset_x2,
                    y: p2.y - offset_y2,
                });
            }
        }
    }

    right_coords.reverse();
    left_coords.extend(right_coords);

    if !left_coords.is_empty() {
        left_coords.push(left_coords[0]);
    }

    Ok(Polygon::new(LineString::from(left_coords), vec![]))
}

/// Dissolve overlapping buffers into a single geometry
pub fn dissolve_buffers(buffers: &[Polygon]) -> Result<MultiPolygon> {
    if buffers.is_empty() {
        return Ok(MultiPolygon::new(vec![]));
    }

    // Simple implementation: return as multi-polygon
    // In production, would use geo-booleanop for actual union
    Ok(MultiPolygon::new(buffers.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_point() {
        let point = Point::new(0.0, 0.0);
        let params = BufferParams::new(10.0);
        let buffer = buffer_point(&point, &params).unwrap();

        assert!(buffer.exterior().coords_count() > 4);
    }

    #[test]
    fn test_buffer_params_validation() {
        let params = BufferParams::new(f64::NAN);
        assert!(params.validate().is_err());

        let params = BufferParams::new(10.0).quadrant_segments(0);
        assert!(params.validate().is_err());

        let params = BufferParams::new(10.0).miter_limit(0.5);
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_buffer_line() {
        let line = LineString::from(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]);
        let params = BufferParams::new(1.0);
        let buffer = buffer_line(&line, &params).unwrap();

        assert!(buffer.exterior().coords_count() > 3);
    }

    #[test]
    fn test_variable_buffer() {
        let line = LineString::from(vec![(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)]);
        let distances = vec![1.0, 2.0, 3.0];
        let params = BufferParams::default();
        let buffer = variable_buffer(&line, &distances, &params).unwrap();

        assert!(buffer.exterior().coords_count() > 3);
    }

    #[test]
    fn test_buffer_points_parallel() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(20.0, 20.0),
        ];
        let params = BufferParams::new(5.0);
        let buffers = buffer_points(&points, &params).unwrap();

        assert_eq!(buffers.len(), 3);
    }
}
