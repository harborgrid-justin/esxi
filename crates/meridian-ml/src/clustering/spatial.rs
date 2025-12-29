//! Spatial clustering algorithms

use crate::clustering::{Clusterer, ClusteringResult};
use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Spatial clustering methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpatialClusteringMethod {
    /// DBSCAN (Density-Based Spatial Clustering)
    DBSCAN,

    /// OPTICS (Ordering Points To Identify Clustering Structure)
    OPTICS,

    /// Spatial K-means
    SpatialKMeans,

    /// Hierarchical spatial clustering
    Hierarchical,
}

/// DBSCAN parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DBSCANParams {
    /// Epsilon (neighborhood radius)
    pub eps: f64,

    /// Minimum points in neighborhood
    pub min_samples: usize,

    /// Distance metric
    pub metric: DistanceMetric,
}

impl Default for DBSCANParams {
    fn default() -> Self {
        Self {
            eps: 0.5,
            min_samples: 5,
            metric: DistanceMetric::Euclidean,
        }
    }
}

/// Distance metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    /// Euclidean distance
    Euclidean,

    /// Manhattan distance
    Manhattan,

    /// Haversine distance (for lat/lon)
    Haversine,
}

/// Spatial clustering
pub struct SpatialClusterer {
    /// Clustering method
    method: SpatialClusteringMethod,

    /// DBSCAN parameters
    dbscan_params: Option<DBSCANParams>,

    /// Cluster labels
    labels: Option<Array1<usize>>,

    /// Number of clusters found
    n_clusters: usize,

    /// Whether the model is fitted
    fitted: bool,
}

impl SpatialClusterer {
    /// Create a new spatial clusterer
    pub fn new(method: SpatialClusteringMethod) -> Self {
        Self {
            method,
            dbscan_params: if matches!(method, SpatialClusteringMethod::DBSCAN) {
                Some(DBSCANParams::default())
            } else {
                None
            },
            labels: None,
            n_clusters: 0,
            fitted: false,
        }
    }

    /// Create DBSCAN clusterer
    pub fn dbscan(eps: f64, min_samples: usize) -> Self {
        Self {
            method: SpatialClusteringMethod::DBSCAN,
            dbscan_params: Some(DBSCANParams {
                eps,
                min_samples,
                metric: DistanceMetric::Euclidean,
            }),
            labels: None,
            n_clusters: 0,
            fitted: false,
        }
    }

    /// Set distance metric
    pub fn with_metric(mut self, metric: DistanceMetric) -> Self {
        if let Some(ref mut params) = self.dbscan_params {
            params.metric = metric;
        }
        self
    }

    /// Fit with coordinates
    pub fn fit_with_coords(&mut self, coords: &Array2<f64>) -> Result<()> {
        match self.method {
            SpatialClusteringMethod::DBSCAN => self.fit_dbscan(coords),
            _ => Err(MlError::Model(format!(
                "Method {:?} not yet implemented",
                self.method
            ))),
        }
    }

    /// DBSCAN implementation
    fn fit_dbscan(&mut self, coords: &Array2<f64>) -> Result<()> {
        let params = self
            .dbscan_params
            .as_ref()
            .ok_or_else(|| MlError::Model("DBSCAN parameters not set".to_string()))?;

        let n_points = coords.nrows();
        let mut labels = Array1::from_elem(n_points, usize::MAX); // usize::MAX = unvisited
        let mut cluster_id = 0;

        for i in 0..n_points {
            if labels[i] != usize::MAX {
                continue; // Already processed
            }

            // Find neighbors
            let neighbors = self.find_neighbors(coords, i, params.eps, params.metric);

            if neighbors.len() < params.min_samples {
                labels[i] = usize::MAX - 1; // Mark as noise
                continue;
            }

            // Start new cluster
            self.expand_cluster(coords, &mut labels, i, &neighbors, cluster_id, params);
            cluster_id += 1;
        }

        self.n_clusters = cluster_id;
        self.labels = Some(labels);
        self.fitted = true;

        Ok(())
    }

    /// Find neighbors within epsilon distance
    fn find_neighbors(
        &self,
        coords: &Array2<f64>,
        point_idx: usize,
        eps: f64,
        metric: DistanceMetric,
    ) -> Vec<usize> {
        let mut neighbors = Vec::new();
        let point = coords.row(point_idx);

        for (i, other) in coords.rows().into_iter().enumerate() {
            let dist = self.distance(&point.to_owned(), &other.to_owned(), metric);
            if dist <= eps {
                neighbors.push(i);
            }
        }

        neighbors
    }

    /// Calculate distance between two points
    fn distance(&self, p1: &Array1<f64>, p2: &Array1<f64>, metric: DistanceMetric) -> f64 {
        match metric {
            DistanceMetric::Euclidean => {
                p1.iter()
                    .zip(p2.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt()
            }
            DistanceMetric::Manhattan => {
                p1.iter()
                    .zip(p2.iter())
                    .map(|(a, b)| (a - b).abs())
                    .sum::<f64>()
            }
            DistanceMetric::Haversine => {
                // Haversine formula for lat/lon
                let lat1 = p1[0].to_radians();
                let lon1 = p1[1].to_radians();
                let lat2 = p2[0].to_radians();
                let lon2 = p2[1].to_radians();

                let dlat = lat2 - lat1;
                let dlon = lon2 - lon1;

                let a = (dlat / 2.0).sin().powi(2)
                    + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
                let c = 2.0 * a.sqrt().asin();

                6371.0 * c // Earth radius in km
            }
        }
    }

    /// Expand cluster from seed point
    fn expand_cluster(
        &self,
        coords: &Array2<f64>,
        labels: &mut Array1<usize>,
        point_idx: usize,
        neighbors: &[usize],
        cluster_id: usize,
        params: &DBSCANParams,
    ) {
        labels[point_idx] = cluster_id;
        let mut seed_set: Vec<usize> = neighbors.to_vec();
        let mut i = 0;

        while i < seed_set.len() {
            let current_point = seed_set[i];

            if labels[current_point] == usize::MAX - 1 {
                // Was noise, add to cluster
                labels[current_point] = cluster_id;
            }

            if labels[current_point] != usize::MAX {
                i += 1;
                continue; // Already processed
            }

            labels[current_point] = cluster_id;

            // Find neighbors of current point
            let current_neighbors =
                self.find_neighbors(coords, current_point, params.eps, params.metric);

            if current_neighbors.len() >= params.min_samples {
                // Add new neighbors to seed set
                for &neighbor in &current_neighbors {
                    if !seed_set.contains(&neighbor) {
                        seed_set.push(neighbor);
                    }
                }
            }

            i += 1;
        }
    }

    /// Get cluster labels
    pub fn labels(&self) -> Option<&Array1<usize>> {
        self.labels.as_ref()
    }
}

impl Clusterer for SpatialClusterer {
    fn fit(&mut self, features: &FeatureSet) -> Result<()> {
        // Use first two features as coordinates
        if features.num_features() < 2 {
            return Err(MlError::InvalidInput(
                "At least 2 features required for spatial clustering".to_string(),
            ));
        }

        let coords = features.features.slice(ndarray::s![.., 0..2]).to_owned();
        self.fit_with_coords(&coords)
    }

    fn predict(&self, features: &FeatureSet) -> Result<Array1<usize>> {
        if !self.fitted {
            return Err(MlError::Model("Model not fitted".to_string()));
        }

        self.labels
            .clone()
            .ok_or_else(|| MlError::Model("No labels available".to_string()))
    }

    fn num_clusters(&self) -> usize {
        self.n_clusters
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_dbscan_creation() {
        let dbscan = SpatialClusterer::dbscan(0.5, 5);
        assert!(!dbscan.is_fitted());
    }

    #[test]
    fn test_distance_euclidean() {
        let clusterer = SpatialClusterer::new(SpatialClusteringMethod::DBSCAN);
        let p1 = Array1::from_vec(vec![0.0, 0.0]);
        let p2 = Array1::from_vec(vec![3.0, 4.0]);

        let dist = clusterer.distance(&p1, &p2, DistanceMetric::Euclidean);
        assert!((dist - 5.0).abs() < 1e-10);
    }
}
