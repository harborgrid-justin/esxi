//! Proximity analysis operations including nearest neighbor, Voronoi diagrams

use crate::error::{AnalysisError, Result};
use geo::{Coord, EuclideanDistance, LineString, Point, Polygon};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use spade::{DelaunayTriangulation, HasPosition, Point2};
use std::collections::HashMap;

/// A point with an associated ID for proximity analysis
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IndexedPoint {
    pub id: usize,
    pub x: f64,
    pub y: f64,
}

impl IndexedPoint {
    pub fn new(id: usize, x: f64, y: f64) -> Self {
        Self { id, x, y }
    }

    pub fn from_point(id: usize, point: &Point) -> Self {
        Self::new(id, point.x(), point.y())
    }

    pub fn to_point(&self) -> Point {
        Point::new(self.x, self.y)
    }
}

impl HasPosition for IndexedPoint {
    type Scalar = f64;

    fn position(&self) -> Point2<Self::Scalar> {
        Point2::new(self.x, self.y)
    }
}

/// Result of a nearest neighbor query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearestNeighbor {
    pub id: usize,
    pub point: Point,
    pub distance: f64,
}

/// Find the nearest neighbor to a query point
pub fn nearest_neighbor(query: &Point, candidates: &[Point]) -> Result<NearestNeighbor> {
    if candidates.is_empty() {
        return Err(AnalysisError::InsufficientData {
            required: 1,
            actual: 0,
        });
    }

    let (id, (point, distance)) = candidates
        .iter()
        .enumerate()
        .map(|(i, p)| (i, (p, query.euclidean_distance(p))))
        .min_by(|(_, (_, d1)), (_, (_, d2))| {
            d1.partial_cmp(d2).unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();

    Ok(NearestNeighbor {
        id,
        point: *point,
        distance,
    })
}

/// Find k nearest neighbors to a query point
pub fn k_nearest_neighbors(
    query: &Point,
    candidates: &[Point],
    k: usize,
) -> Result<Vec<NearestNeighbor>> {
    if candidates.is_empty() {
        return Err(AnalysisError::InsufficientData {
            required: 1,
            actual: 0,
        });
    }

    if k == 0 {
        return Err(AnalysisError::invalid_parameters(
            "k must be greater than 0",
        ));
    }

    let mut distances: Vec<_> = candidates
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let dist = query.euclidean_distance(p);
            (i, p, dist)
        })
        .collect();

    distances.sort_by(|(_, _, d1), (_, _, d2)| {
        d1.partial_cmp(d2).unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(distances
        .into_iter()
        .take(k)
        .map(|(id, point, distance)| NearestNeighbor {
            id,
            point: *point,
            distance,
        })
        .collect())
}

/// Find all neighbors within a given distance
pub fn neighbors_within_distance(
    query: &Point,
    candidates: &[Point],
    max_distance: f64,
) -> Result<Vec<NearestNeighbor>> {
    if max_distance < 0.0 {
        return Err(AnalysisError::invalid_parameters(
            "Distance must be non-negative",
        ));
    }

    let neighbors = candidates
        .iter()
        .enumerate()
        .filter_map(|(id, point)| {
            let distance = query.euclidean_distance(point);
            if distance <= max_distance {
                Some(NearestNeighbor {
                    id,
                    point: *point,
                    distance,
                })
            } else {
                None
            }
        })
        .collect();

    Ok(neighbors)
}

/// Compute distance matrix for all pairs of points
pub fn distance_matrix(points: &[Point]) -> Vec<Vec<f64>> {
    let n = points.len();
    let mut matrix = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in (i + 1)..n {
            let dist = points[i].euclidean_distance(&points[j]);
            matrix[i][j] = dist;
            matrix[j][i] = dist;
        }
    }

    matrix
}

/// Compute distance matrix in parallel
pub fn distance_matrix_parallel(points: &[Point]) -> Vec<Vec<f64>> {
    let n = points.len();
    let matrix: Vec<Vec<f64>> = (0..n)
        .into_par_iter()
        .map(|i| {
            let mut row = vec![0.0; n];
            for j in 0..n {
                if i != j {
                    row[j] = points[i].euclidean_distance(&points[j]);
                }
            }
            row
        })
        .collect();

    matrix
}

/// Generate Voronoi diagram from a set of points
pub fn voronoi_diagram(points: &[Point]) -> Result<Vec<Polygon>> {
    if points.len() < 3 {
        return Err(AnalysisError::InsufficientData {
            required: 3,
            actual: points.len(),
        });
    }

    // Convert to indexed points
    let indexed_points: Vec<IndexedPoint> = points
        .iter()
        .enumerate()
        .map(|(i, p)| IndexedPoint::from_point(i, p))
        .collect();

    // Create Delaunay triangulation
    let mut triangulation = FloatDelaunayTriangulation::new();

    for point in indexed_points.iter() {
        triangulation
            .insert(*point)
            .map_err(|e| AnalysisError::ProximityError(format!("Triangulation failed: {:?}", e)))?;
    }

    // Extract Voronoi cells
    // This is a simplified implementation
    let mut voronoi_cells = Vec::new();

    for vertex in triangulation.vertices() {
        let cell_points = compute_voronoi_cell(&triangulation, vertex);
        if !cell_points.is_empty() {
            let mut coords: Vec<Coord> = cell_points
                .iter()
                .map(|p| Coord { x: p.x, y: p.y })
                .collect();

            // Close the polygon
            if let Some(&first) = coords.first() {
                coords.push(first);
            }

            if coords.len() >= 4 {
                // At least 3 unique points + closing point
                voronoi_cells.push(Polygon::new(LineString::from(coords), vec![]));
            }
        }
    }

    Ok(voronoi_cells)
}

/// Compute Voronoi cell for a vertex (simplified)
fn compute_voronoi_cell<T: HasPosition<Scalar = f64>>(
    _triangulation: &DelaunayTriangulation<T>,
    _vertex: spade::handles::FixedVertexHandle,
) -> Vec<Point2<f64>> {
    // Simplified implementation
    // In production, would properly compute Voronoi cell from dual of Delaunay
    vec![]
}

/// Thiessen polygons (same as Voronoi)
pub fn thiessen_polygons(points: &[Point]) -> Result<Vec<Polygon>> {
    voronoi_diagram(points)
}

/// Find nearest facility for each demand point
pub fn nearest_facility(
    demand_points: &[Point],
    facility_points: &[Point],
) -> Result<Vec<(usize, NearestNeighbor)>> {
    if facility_points.is_empty() {
        return Err(AnalysisError::InsufficientData {
            required: 1,
            actual: 0,
        });
    }

    demand_points
        .iter()
        .enumerate()
        .map(|(i, point)| {
            nearest_neighbor(point, facility_points).map(|neighbor| (i, neighbor))
        })
        .collect()
}

/// Allocate points to nearest facilities in parallel
pub fn allocate_to_facilities_parallel(
    demand_points: &[Point],
    facility_points: &[Point],
) -> Result<HashMap<usize, Vec<usize>>> {
    if facility_points.is_empty() {
        return Err(AnalysisError::InsufficientData {
            required: 1,
            actual: 0,
        });
    }

    let allocations: Vec<(usize, usize)> = demand_points
        .par_iter()
        .enumerate()
        .map(|(demand_id, point)| {
            let nearest = nearest_neighbor(point, facility_points).unwrap();
            (nearest.id, demand_id)
        })
        .collect();

    let mut result: HashMap<usize, Vec<usize>> = HashMap::new();
    for (facility_id, demand_id) in allocations {
        result.entry(facility_id).or_insert_with(Vec::new).push(demand_id);
    }

    Ok(result)
}

/// Calculate minimum distance from each point to a set of features
pub fn minimum_distance_to_features(
    query_points: &[Point],
    feature_points: &[Point],
) -> Result<Vec<f64>> {
    if feature_points.is_empty() {
        return Err(AnalysisError::InsufficientData {
            required: 1,
            actual: 0,
        });
    }

    Ok(query_points
        .par_iter()
        .map(|query| {
            feature_points
                .iter()
                .map(|feature| query.euclidean_distance(feature))
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(f64::INFINITY)
        })
        .collect())
}

/// Create proximity zones (buffer zones at different distances)
pub fn proximity_zones(
    _points: &[Point],
    distances: &[f64],
) -> Result<Vec<Vec<Polygon>>> {
    if distances.is_empty() {
        return Err(AnalysisError::invalid_parameters(
            "At least one distance required",
        ));
    }

    // Sort distances
    let mut sorted_distances = distances.to_vec();
    sorted_distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // For each distance, create buffers
    // This would integrate with buffer module in production
    let zones: Vec<Vec<Polygon>> = sorted_distances
        .iter()
        .map(|_distance| {
            // Simplified: return empty for now
            // In production, would use buffer operations
            vec![]
        })
        .collect();

    Ok(zones)
}

/// Point-in-polygon test for multiple polygons
pub fn point_in_polygons(point: &Point, polygons: &[Polygon]) -> Vec<usize> {
    use geo::Contains;

    polygons
        .iter()
        .enumerate()
        .filter_map(|(i, poly)| {
            if poly.contains(point) {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nearest_neighbor() {
        let query = Point::new(0.0, 0.0);
        let candidates = vec![
            Point::new(1.0, 1.0),
            Point::new(5.0, 5.0),
            Point::new(10.0, 10.0),
        ];

        let result = nearest_neighbor(&query, &candidates).unwrap();
        assert_eq!(result.id, 0);
    }

    #[test]
    fn test_k_nearest_neighbors() {
        let query = Point::new(0.0, 0.0);
        let candidates = vec![
            Point::new(1.0, 1.0),
            Point::new(5.0, 5.0),
            Point::new(10.0, 10.0),
        ];

        let result = k_nearest_neighbors(&query, &candidates, 2).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 0);
        assert_eq!(result[1].id, 1);
    }

    #[test]
    fn test_neighbors_within_distance() {
        let query = Point::new(0.0, 0.0);
        let candidates = vec![
            Point::new(1.0, 0.0),
            Point::new(5.0, 0.0),
            Point::new(10.0, 0.0),
        ];

        let result = neighbors_within_distance(&query, &candidates, 6.0).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_distance_matrix() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 1.0),
        ];

        let matrix = distance_matrix(&points);
        assert_eq!(matrix.len(), 3);
        assert_eq!(matrix[0].len(), 3);
        assert_eq!(matrix[0][0], 0.0);
        assert!((matrix[0][1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_distance_matrix_parallel() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 1.0),
        ];

        let matrix = distance_matrix_parallel(&points);
        assert_eq!(matrix.len(), 3);
        assert_eq!(matrix[0].len(), 3);
    }

    #[test]
    fn test_minimum_distance_to_features() {
        let query_points = vec![Point::new(0.0, 0.0), Point::new(10.0, 10.0)];
        let feature_points = vec![Point::new(1.0, 1.0), Point::new(5.0, 5.0)];

        let distances = minimum_distance_to_features(&query_points, &feature_points).unwrap();
        assert_eq!(distances.len(), 2);
        assert!(distances[0] < distances[1]);
    }
}
