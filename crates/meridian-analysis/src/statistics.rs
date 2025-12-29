//! Spatial statistics and point pattern analysis

use crate::error::{AnalysisError, Result};
use geo::{Area, EuclideanDistance, Point, Polygon};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Point pattern statistics result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointPatternStats {
    pub count: usize,
    pub density: f64,
    pub mean_center: Point,
    pub standard_distance: f64,
    pub mean_nearest_neighbor_distance: f64,
    pub nearest_neighbor_index: f64,
}

/// Calculate basic point pattern statistics
pub fn point_pattern_analysis(points: &[Point], study_area: &Polygon) -> Result<PointPatternStats> {
    if points.is_empty() {
        return Err(AnalysisError::InsufficientData {
            required: 1,
            actual: 0,
        });
    }

    let count = points.len();
    let area = study_area.unsigned_area();

    if area == 0.0 {
        return Err(AnalysisError::invalid_parameters("Study area has zero area"));
    }

    let density = count as f64 / area;

    // Mean center
    let mean_center = calculate_mean_center(points);

    // Standard distance
    let standard_distance = calculate_standard_distance(points, &mean_center);

    // Nearest neighbor analysis
    let mean_nn_dist = mean_nearest_neighbor_distance(points)?;

    // Expected mean distance under random distribution
    let expected_mean_dist = 1.0 / (2.0 * density.sqrt());

    let nearest_neighbor_index = if expected_mean_dist > 0.0 {
        mean_nn_dist / expected_mean_dist
    } else {
        0.0
    };

    Ok(PointPatternStats {
        count,
        density,
        mean_center,
        standard_distance,
        mean_nearest_neighbor_distance: mean_nn_dist,
        nearest_neighbor_index,
    })
}

/// Calculate mean center of points
pub fn calculate_mean_center(points: &[Point]) -> Point {
    if points.is_empty() {
        return Point::new(0.0, 0.0);
    }

    let sum_x: f64 = points.iter().map(|p| p.x()).sum();
    let sum_y: f64 = points.iter().map(|p| p.y()).sum();

    Point::new(sum_x / points.len() as f64, sum_y / points.len() as f64)
}

/// Calculate weighted mean center
pub fn weighted_mean_center(points: &[Point], weights: &[f64]) -> Result<Point> {
    if points.len() != weights.len() {
        return Err(AnalysisError::invalid_parameters(
            "Points and weights must have same length",
        ));
    }

    if points.is_empty() {
        return Err(AnalysisError::InsufficientData {
            required: 1,
            actual: 0,
        });
    }

    let total_weight: f64 = weights.iter().sum();

    if total_weight == 0.0 {
        return Err(AnalysisError::invalid_parameters("Total weight is zero"));
    }

    let sum_x: f64 = points
        .iter()
        .zip(weights)
        .map(|(p, w)| p.x() * w)
        .sum();

    let sum_y: f64 = points
        .iter()
        .zip(weights)
        .map(|(p, w)| p.y() * w)
        .sum();

    Ok(Point::new(sum_x / total_weight, sum_y / total_weight))
}

/// Calculate standard distance (spatial standard deviation)
pub fn calculate_standard_distance(points: &[Point], center: &Point) -> f64 {
    if points.is_empty() {
        return 0.0;
    }

    let sum_squared_dist: f64 = points
        .iter()
        .map(|p| {
            let dist = center.euclidean_distance(p);
            dist * dist
        })
        .sum();

    (sum_squared_dist / points.len() as f64).sqrt()
}

/// Calculate mean nearest neighbor distance
pub fn mean_nearest_neighbor_distance(points: &[Point]) -> Result<f64> {
    if points.len() < 2 {
        return Err(AnalysisError::InsufficientData {
            required: 2,
            actual: points.len(),
        });
    }

    let distances: Vec<f64> = points
        .par_iter()
        .map(|point| {
            points
                .iter()
                .filter(|&other| other != point)
                .map(|other| point.euclidean_distance(other))
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0)
        })
        .collect();

    let mean = distances.iter().sum::<f64>() / distances.len() as f64;
    Ok(mean)
}

/// Hot spot analysis using Getis-Ord Gi* statistic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotSpot {
    pub point_id: usize,
    pub gi_star: f64,
    pub z_score: f64,
    pub p_value: f64,
    pub classification: HotSpotClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HotSpotClass {
    HotSpot99,    // 99% confidence
    HotSpot95,    // 95% confidence
    HotSpot90,    // 90% confidence
    NotSignificant,
    ColdSpot90,   // 90% confidence
    ColdSpot95,   // 95% confidence
    ColdSpot99,   // 99% confidence
}

/// Perform hot spot analysis
pub fn hot_spot_analysis(
    points: &[Point],
    values: &[f64],
    distance_threshold: f64,
) -> Result<Vec<HotSpot>> {
    if points.len() != values.len() {
        return Err(AnalysisError::invalid_parameters(
            "Points and values must have same length",
        ));
    }

    if points.len() < 3 {
        return Err(AnalysisError::InsufficientData {
            required: 3,
            actual: points.len(),
        });
    }

    let results: Vec<HotSpot> = (0..points.len())
        .into_par_iter()
        .map(|i| {
            let gi_star = calculate_getis_ord_gi_star(i, points, values, distance_threshold);
            let z_score = gi_star; // Simplified
            let p_value = calculate_p_value(z_score);
            let classification = classify_hot_spot(z_score);

            HotSpot {
                point_id: i,
                gi_star,
                z_score,
                p_value,
                classification,
            }
        })
        .collect();

    Ok(results)
}

/// Calculate Getis-Ord Gi* statistic
fn calculate_getis_ord_gi_star(
    index: usize,
    points: &[Point],
    values: &[f64],
    distance_threshold: f64,
) -> f64 {
    let focal_point = &points[index];
    let mut sum_weighted = 0.0;
    let mut sum_weights = 0.0;

    for (j, point) in points.iter().enumerate() {
        let dist = focal_point.euclidean_distance(point);

        if dist <= distance_threshold {
            let weight = 1.0; // Binary weight for simplicity
            sum_weighted += values[j] * weight;
            sum_weights += weight;
        }
    }

    if sum_weights == 0.0 {
        return 0.0;
    }

    // Simplified Gi* calculation
    let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;
    let variance: f64 = values
        .iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;

    let std_dev = variance.sqrt();

    if std_dev == 0.0 {
        return 0.0;
    }

    let expected = mean * sum_weights;
    let gi_star = (sum_weighted - expected) / (std_dev * (sum_weights as f64).sqrt());

    gi_star
}

/// Calculate p-value from z-score (simplified)
fn calculate_p_value(z_score: f64) -> f64 {
    // Simplified p-value calculation
    // In production, would use proper statistical distribution
    let abs_z = z_score.abs();

    if abs_z >= 2.576 {
        0.01
    } else if abs_z >= 1.96 {
        0.05
    } else if abs_z >= 1.645 {
        0.10
    } else {
        1.0
    }
}

/// Classify hot spot based on z-score
fn classify_hot_spot(z_score: f64) -> HotSpotClass {
    if z_score >= 2.576 {
        HotSpotClass::HotSpot99
    } else if z_score >= 1.96 {
        HotSpotClass::HotSpot95
    } else if z_score >= 1.645 {
        HotSpotClass::HotSpot90
    } else if z_score <= -2.576 {
        HotSpotClass::ColdSpot99
    } else if z_score <= -1.96 {
        HotSpotClass::ColdSpot95
    } else if z_score <= -1.645 {
        HotSpotClass::ColdSpot90
    } else {
        HotSpotClass::NotSignificant
    }
}

/// Spatial autocorrelation using Moran's I
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoransI {
    pub i: f64,
    pub expected_i: f64,
    pub variance: f64,
    pub z_score: f64,
    pub p_value: f64,
}

/// Calculate Moran's I statistic
pub fn morans_i(points: &[Point], values: &[f64], distance_threshold: f64) -> Result<MoransI> {
    if points.len() != values.len() {
        return Err(AnalysisError::invalid_parameters(
            "Points and values must have same length",
        ));
    }

    if points.len() < 3 {
        return Err(AnalysisError::InsufficientData {
            required: 3,
            actual: points.len(),
        });
    }

    let n = points.len();
    let mean: f64 = values.iter().sum::<f64>() / n as f64;

    // Build spatial weights matrix
    let mut weights = vec![vec![0.0; n]; n];
    let mut total_weight = 0.0;

    for i in 0..n {
        for j in 0..n {
            if i != j {
                let dist = points[i].euclidean_distance(&points[j]);
                if dist <= distance_threshold {
                    weights[i][j] = 1.0;
                    total_weight += 1.0;
                }
            }
        }
    }

    if total_weight == 0.0 {
        return Err(AnalysisError::StatisticsError(
            "No spatial relationships found within threshold".to_string(),
        ));
    }

    // Calculate Moran's I
    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for i in 0..n {
        for j in 0..n {
            numerator += weights[i][j] * (values[i] - mean) * (values[j] - mean);
        }
        denominator += (values[i] - mean).powi(2);
    }

    let i_stat = (n as f64 / total_weight) * (numerator / denominator);

    // Expected value and variance
    let expected_i = -1.0 / (n as f64 - 1.0);
    let variance_i = 1.0 / (n as f64 - 1.0); // Simplified

    let z_score = (i_stat - expected_i) / variance_i.sqrt();
    let p_value = calculate_p_value(z_score);

    Ok(MoransI {
        i: i_stat,
        expected_i,
        variance: variance_i,
        z_score,
        p_value,
    })
}

/// K-means clustering for spatial points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub id: usize,
    pub center: Point,
    pub members: Vec<usize>,
}

/// Perform k-means clustering
pub fn k_means_clustering(points: &[Point], k: usize, max_iterations: usize) -> Result<Vec<Cluster>> {
    if k == 0 {
        return Err(AnalysisError::invalid_parameters("k must be greater than 0"));
    }

    if points.len() < k {
        return Err(AnalysisError::InsufficientData {
            required: k,
            actual: points.len(),
        });
    }

    // Initialize cluster centers (simple: take first k points)
    let mut centers: Vec<Point> = points.iter().take(k).cloned().collect();
    let mut assignments = vec![0; points.len()];

    for _iteration in 0..max_iterations {
        let mut changed = false;

        // Assignment step
        for (i, point) in points.iter().enumerate() {
            let closest = centers
                .iter()
                .enumerate()
                .min_by(|(_, c1), (_, c2)| {
                    let d1 = point.euclidean_distance(*c1);
                    let d2 = point.euclidean_distance(*c2);
                    d1.partial_cmp(&d2).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx)
                .unwrap();

            if assignments[i] != closest {
                assignments[i] = closest;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        // Update step
        for cluster_id in 0..k {
            let cluster_points: Vec<&Point> = points
                .iter()
                .enumerate()
                .filter(|(i, _)| assignments[*i] == cluster_id)
                .map(|(_, p)| p)
                .collect();

            if !cluster_points.is_empty() {
                centers[cluster_id] = calculate_mean_center(&cluster_points.iter().map(|&&p| p).collect::<Vec<_>>());
            }
        }
    }

    // Build result
    let clusters: Vec<Cluster> = (0..k)
        .map(|cluster_id| {
            let members: Vec<usize> = assignments
                .iter()
                .enumerate()
                .filter(|(_, &c)| c == cluster_id)
                .map(|(i, _)| i)
                .collect();

            Cluster {
                id: cluster_id,
                center: centers[cluster_id],
                members,
            }
        })
        .collect();

    Ok(clusters)
}

/// DBSCAN clustering
pub fn dbscan_clustering(points: &[Point], epsilon: f64, min_points: usize) -> Result<Vec<Cluster>> {
    if epsilon <= 0.0 {
        return Err(AnalysisError::invalid_parameters("Epsilon must be positive"));
    }

    let n = points.len();
    let mut labels = vec![-1; n]; // -1 = unclassified, -2 = noise
    let mut cluster_id = 0;

    for i in 0..n {
        if labels[i] != -1 {
            continue;
        }

        let neighbors = find_neighbors(points, i, epsilon);

        if neighbors.len() < min_points {
            labels[i] = -2; // Mark as noise
            continue;
        }

        // Start new cluster
        labels[i] = cluster_id;
        let mut seed_set = neighbors;

        while let Some(j) = seed_set.pop() {
            if labels[j] == -2 {
                labels[j] = cluster_id;
            }

            if labels[j] != -1 {
                continue;
            }

            labels[j] = cluster_id;

            let neighbors_j = find_neighbors(points, j, epsilon);
            if neighbors_j.len() >= min_points {
                seed_set.extend(neighbors_j);
            }
        }

        cluster_id += 1;
    }

    // Build clusters
    let mut clusters = Vec::new();
    for cid in 0..cluster_id {
        let members: Vec<usize> = labels
            .iter()
            .enumerate()
            .filter(|(_, &label)| label == cid)
            .map(|(i, _)| i)
            .collect();

        if !members.is_empty() {
            let cluster_points: Vec<Point> = members.iter().map(|&i| points[i]).collect();
            let center = calculate_mean_center(&cluster_points);

            clusters.push(Cluster {
                id: cid as usize,
                center,
                members,
            });
        }
    }

    Ok(clusters)
}

/// Find neighbors within epsilon distance
fn find_neighbors(points: &[Point], index: usize, epsilon: f64) -> Vec<usize> {
    points
        .iter()
        .enumerate()
        .filter(|(i, point)| {
            *i != index && points[index].euclidean_distance(*point) <= epsilon
        })
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::LineString;

    #[test]
    fn test_mean_center() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];

        let center = calculate_mean_center(&points);
        assert!((center.x() - 5.0).abs() < 1e-10);
        assert!((center.y() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_weighted_mean_center() {
        let points = vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)];
        let weights = vec![1.0, 1.0];

        let center = weighted_mean_center(&points, &weights).unwrap();
        assert!((center.x() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_standard_distance() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ];

        let center = calculate_mean_center(&points);
        let std_dist = calculate_standard_distance(&points, &center);
        assert!(std_dist > 0.0);
    }

    #[test]
    fn test_k_means_clustering() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(10.0, 10.0),
            Point::new(11.0, 11.0),
        ];

        let clusters = k_means_clustering(&points, 2, 10).unwrap();
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn test_hot_spot_classification() {
        assert_eq!(classify_hot_spot(3.0), HotSpotClass::HotSpot99);
        assert_eq!(classify_hot_spot(2.0), HotSpotClass::HotSpot95);
        assert_eq!(classify_hot_spot(-3.0), HotSpotClass::ColdSpot99);
        assert_eq!(classify_hot_spot(0.0), HotSpotClass::NotSignificant);
    }
}
