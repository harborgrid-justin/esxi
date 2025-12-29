//! Geographically Weighted Regression (GWR)

use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use crate::regression::{Regressor, RegressionResult};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Types of spatial regression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpatialRegressionType {
    /// Geographically Weighted Regression
    GWR,

    /// Spatial Lag Model
    SpatialLag,

    /// Spatial Error Model
    SpatialError,

    /// Geographically and Temporally Weighted Regression
    GTWR,
}

/// Kernel functions for GWR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelType {
    /// Gaussian kernel
    Gaussian,

    /// Exponential kernel
    Exponential,

    /// Bi-square kernel
    BiSquare,

    /// Tri-cube kernel
    TriCube,
}

/// Bandwidth selection method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BandwidthSelection {
    /// Fixed bandwidth
    Fixed(f64),

    /// Adaptive bandwidth (k nearest neighbors)
    Adaptive(usize),

    /// Cross-validation
    CrossValidation,

    /// AIC-based selection
    AIC,
}

/// GWR configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GWRConfig {
    /// Kernel type
    pub kernel: KernelType,

    /// Bandwidth selection
    pub bandwidth: BandwidthSelection,

    /// Whether to standardize features
    pub standardize: bool,
}

impl Default for GWRConfig {
    fn default() -> Self {
        Self {
            kernel: KernelType::Gaussian,
            bandwidth: BandwidthSelection::Adaptive(30),
            standardize: true,
        }
    }
}

/// Spatial regression model
pub struct SpatialRegression {
    /// Regression type
    regression_type: SpatialRegressionType,

    /// Configuration (for GWR)
    config: Option<GWRConfig>,

    /// Training coordinates
    training_coords: Option<Array2<f64>>,

    /// Local coefficients (for GWR)
    local_coefficients: Option<Array2<f64>>,

    /// Whether the model is trained
    trained: bool,
}

impl SpatialRegression {
    /// Create a new spatial regression model
    pub fn new(regression_type: SpatialRegressionType) -> Self {
        Self {
            regression_type,
            config: if matches!(regression_type, SpatialRegressionType::GWR) {
                Some(GWRConfig::default())
            } else {
                None
            },
            training_coords: None,
            local_coefficients: None,
            trained: false,
        }
    }

    /// Create a GWR model with custom configuration
    pub fn gwr_with_config(config: GWRConfig) -> Self {
        Self {
            regression_type: SpatialRegressionType::GWR,
            config: Some(config),
            training_coords: None,
            local_coefficients: None,
            trained: false,
        }
    }

    /// Set kernel type (for GWR)
    pub fn with_kernel(mut self, kernel: KernelType) -> Self {
        if let Some(ref mut config) = self.config {
            config.kernel = kernel;
        }
        self
    }

    /// Set bandwidth (for GWR)
    pub fn with_bandwidth(mut self, bandwidth: BandwidthSelection) -> Self {
        if let Some(ref mut config) = self.config {
            config.bandwidth = bandwidth;
        }
        self
    }

    /// Train with spatial coordinates
    pub fn train_with_coords(
        &mut self,
        features: &FeatureSet,
        targets: &Array1<f64>,
        coords: &Array2<f64>,
    ) -> Result<()> {
        if features.num_samples() != targets.len() {
            return Err(MlError::InvalidInput(
                "Number of features and targets must match".to_string(),
            ));
        }

        if coords.nrows() != features.num_samples() {
            return Err(MlError::InvalidInput(
                "Number of coordinates must match samples".to_string(),
            ));
        }

        self.training_coords = Some(coords.clone());

        match self.regression_type {
            SpatialRegressionType::GWR => self.train_gwr(features, targets, coords),
            SpatialRegressionType::SpatialLag => self.train_spatial_lag(features, targets, coords),
            SpatialRegressionType::SpatialError => {
                self.train_spatial_error(features, targets, coords)
            }
            SpatialRegressionType::GTWR => Err(MlError::Model(
                "GTWR requires temporal data".to_string(),
            )),
        }
    }

    /// Train GWR model
    fn train_gwr(
        &mut self,
        features: &FeatureSet,
        targets: &Array1<f64>,
        coords: &Array2<f64>,
    ) -> Result<()> {
        let n_samples = features.num_samples();
        let n_features = features.num_features();

        // Initialize local coefficients
        let mut local_coefs = Array2::zeros((n_samples, n_features + 1)); // +1 for intercept

        // For each sample location, fit a local regression
        for i in 0..n_samples {
            let weights = self.compute_weights(coords, i)?;

            // Weighted least squares
            // β_i = (X'WX)^-1 X'Wy
            // This is a simplified implementation

            // For now, store zeros
            // In production, implement proper weighted regression
        }

        self.local_coefficients = Some(local_coefs);
        self.trained = true;

        Ok(())
    }

    /// Compute spatial weights
    fn compute_weights(&self, coords: &Array2<f64>, target_idx: usize) -> Result<Array1<f64>> {
        let config = self
            .config
            .as_ref()
            .ok_or_else(|| MlError::Model("GWR config not set".to_string()))?;

        let target_coord = coords.row(target_idx);
        let n_samples = coords.nrows();
        let mut weights = Array1::zeros(n_samples);

        // Calculate distances
        let mut distances: Vec<(usize, f64)> = coords
            .rows()
            .into_iter()
            .enumerate()
            .map(|(idx, coord)| {
                let dx = coord[0] - target_coord[0];
                let dy = coord[1] - target_coord[1];
                let dist = (dx * dx + dy * dy).sqrt();
                (idx, dist)
            })
            .collect();

        // Get bandwidth
        let bandwidth = match config.bandwidth {
            BandwidthSelection::Fixed(bw) => bw,
            BandwidthSelection::Adaptive(k) => {
                distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                if k < distances.len() {
                    distances[k].1
                } else {
                    distances.last().map(|(_, d)| *d).unwrap_or(1.0)
                }
            }
            _ => 1.0, // Default bandwidth
        };

        // Compute kernel weights
        for (idx, dist) in distances {
            let w = self.kernel_function(dist, bandwidth, config.kernel);
            weights[idx] = w;
        }

        Ok(weights)
    }

    /// Kernel function
    fn kernel_function(&self, distance: f64, bandwidth: f64, kernel: KernelType) -> f64 {
        if bandwidth == 0.0 {
            return if distance == 0.0 { 1.0 } else { 0.0 };
        }

        let u = distance / bandwidth;

        match kernel {
            KernelType::Gaussian => (-0.5 * u * u).exp(),
            KernelType::Exponential => (-u).exp(),
            KernelType::BiSquare => {
                if u < 1.0 {
                    (1.0 - u * u).powi(2)
                } else {
                    0.0
                }
            }
            KernelType::TriCube => {
                if u < 1.0 {
                    (1.0 - u.powi(3)).powi(3)
                } else {
                    0.0
                }
            }
        }
    }

    /// Train spatial lag model
    fn train_spatial_lag(
        &mut self,
        features: &FeatureSet,
        targets: &Array1<f64>,
        coords: &Array2<f64>,
    ) -> Result<()> {
        // Simplified spatial lag model
        // Y = ρWY + Xβ + ε
        self.trained = true;
        Ok(())
    }

    /// Train spatial error model
    fn train_spatial_error(
        &mut self,
        features: &FeatureSet,
        targets: &Array1<f64>,
        coords: &Array2<f64>,
    ) -> Result<()> {
        // Simplified spatial error model
        // Y = Xβ + u, u = λWu + ε
        self.trained = true;
        Ok(())
    }

    /// Predict at specific coordinates
    pub fn predict_at_coords(
        &self,
        features: &FeatureSet,
        coords: &Array2<f64>,
    ) -> Result<Array1<f64>> {
        if !self.trained {
            return Err(MlError::Model("Model not trained".to_string()));
        }

        // Placeholder: return zeros
        Ok(Array1::zeros(features.num_samples()))
    }
}

impl Regressor for SpatialRegression {
    fn train(&mut self, features: &FeatureSet, targets: &Array1<f64>) -> Result<()> {
        Err(MlError::Model(
            "Spatial regression requires coordinates. Use train_with_coords()".to_string(),
        ))
    }

    fn predict(&self, features: &FeatureSet) -> Result<Array1<f64>> {
        if !self.trained {
            return Err(MlError::Model("Model not trained".to_string()));
        }

        Ok(Array1::zeros(features.num_samples()))
    }

    fn predict_with_intervals(
        &self,
        features: &FeatureSet,
        confidence: f64,
    ) -> Result<RegressionResult> {
        let predictions = self.predict(features)?;
        Ok(RegressionResult::new(predictions))
    }

    fn is_trained(&self) -> bool {
        self.trained
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_regression_creation() {
        let gwr = SpatialRegression::new(SpatialRegressionType::GWR);
        assert!(!gwr.is_trained());
    }

    #[test]
    fn test_kernel_function() {
        let gwr = SpatialRegression::new(SpatialRegressionType::GWR);
        let w = gwr.kernel_function(0.0, 1.0, KernelType::Gaussian);
        assert_eq!(w, 1.0);

        let w = gwr.kernel_function(1.0, 1.0, KernelType::BiSquare);
        assert_eq!(w, 0.0);
    }
}
