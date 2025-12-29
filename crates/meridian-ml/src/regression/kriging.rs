//! Kriging interpolation for spatial data

use crate::error::{MlError, Result};
use crate::regression::{Regressor, RegressionResult};
use crate::features::FeatureSet;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Types of kriging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KrigingType {
    /// Ordinary kriging
    Ordinary,

    /// Simple kriging
    Simple,

    /// Universal kriging
    Universal,

    /// Co-kriging
    CoKriging,
}

/// Variogram models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariogramModel {
    /// Linear variogram
    Linear,

    /// Spherical variogram
    Spherical,

    /// Exponential variogram
    Exponential,

    /// Gaussian variogram
    Gaussian,

    /// Power variogram
    Power,
}

/// Variogram parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variogram {
    /// Model type
    pub model: VariogramModel,

    /// Nugget effect
    pub nugget: f64,

    /// Sill (total variance)
    pub sill: f64,

    /// Range (correlation length)
    pub range: f64,
}

impl Variogram {
    /// Create a new variogram
    pub fn new(model: VariogramModel, nugget: f64, sill: f64, range: f64) -> Self {
        Self {
            model,
            nugget,
            sill,
            range,
        }
    }

    /// Compute variogram value at distance h
    pub fn value(&self, h: f64) -> f64 {
        if h == 0.0 {
            return 0.0;
        }

        let partial_sill = self.sill - self.nugget;

        let gamma = match self.model {
            VariogramModel::Linear => {
                self.nugget + partial_sill * (h / self.range).min(1.0)
            }
            VariogramModel::Spherical => {
                if h <= self.range {
                    self.nugget
                        + partial_sill
                            * (1.5 * h / self.range - 0.5 * (h / self.range).powi(3))
                } else {
                    self.sill
                }
            }
            VariogramModel::Exponential => {
                self.nugget + partial_sill * (1.0 - (-h / self.range).exp())
            }
            VariogramModel::Gaussian => {
                self.nugget + partial_sill * (1.0 - (-(h * h) / (self.range * self.range)).exp())
            }
            VariogramModel::Power => {
                self.nugget + partial_sill * h.powf(self.range)
            }
        };

        gamma
    }

    /// Compute covariance at distance h
    pub fn covariance(&self, h: f64) -> f64 {
        self.sill - self.value(h)
    }

    /// Fit variogram from data
    pub fn fit(distances: &[f64], semivariances: &[f64], model: VariogramModel) -> Result<Self> {
        if distances.len() != semivariances.len() {
            return Err(MlError::InvalidInput(
                "Distances and semivariances must have same length".to_string(),
            ));
        }

        if distances.is_empty() {
            return Err(MlError::EmptyDataset);
        }

        // Estimate parameters using simple methods
        // In production, use proper least squares fitting

        // Nugget: intercept (semivariance at small distances)
        let nugget = semivariances.iter().take(3).sum::<f64>() / 3.0.min(semivariances.len() as f64);

        // Sill: asymptotic value
        let sill = semivariances.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Range: distance where variogram reaches ~95% of sill
        let target = nugget + 0.95 * (sill - nugget);
        let mut range = distances[distances.len() / 2]; // Default to median distance

        for (i, &sv) in semivariances.iter().enumerate() {
            if sv >= target {
                range = distances[i];
                break;
            }
        }

        Ok(Self {
            model,
            nugget,
            sill,
            range,
        })
    }
}

/// Kriging interpolator
pub struct Kriging {
    /// Kriging type
    kriging_type: KrigingType,

    /// Variogram
    variogram: Option<Variogram>,

    /// Training data coordinates
    training_coords: Option<Array2<f64>>,

    /// Training values
    training_values: Option<Array1<f64>>,

    /// Mean (for simple kriging)
    mean: Option<f64>,

    /// Whether the model is trained
    trained: bool,
}

impl Kriging {
    /// Create a new kriging interpolator
    pub fn new(kriging_type: KrigingType) -> Self {
        Self {
            kriging_type,
            variogram: None,
            training_coords: None,
            training_values: None,
            mean: None,
            trained: false,
        }
    }

    /// Set variogram
    pub fn with_variogram(mut self, variogram: Variogram) -> Self {
        self.variogram = Some(variogram);
        self
    }

    /// Fit variogram from data
    pub fn fit_variogram(&mut self, coords: &Array2<f64>, values: &Array1<f64>, model: VariogramModel) -> Result<()> {
        // Compute empirical variogram
        let (distances, semivariances) = self.compute_empirical_variogram(coords, values)?;

        // Fit variogram model
        let variogram = Variogram::fit(&distances, &semivariances, model)?;
        self.variogram = Some(variogram);

        Ok(())
    }

    /// Compute empirical variogram
    fn compute_empirical_variogram(
        &self,
        coords: &Array2<f64>,
        values: &Array1<f64>,
    ) -> Result<(Vec<f64>, Vec<f64>)> {
        let n = coords.nrows();
        let mut distances = Vec::new();
        let mut semivariances = Vec::new();

        // Compute all pairwise distances and squared differences
        for i in 0..n {
            for j in i + 1..n {
                let dx = coords[[i, 0]] - coords[[j, 0]];
                let dy = coords[[i, 1]] - coords[[j, 1]];
                let dist = (dx * dx + dy * dy).sqrt();

                let diff = values[i] - values[j];
                let semivar = 0.5 * diff * diff;

                distances.push(dist);
                semivariances.push(semivar);
            }
        }

        // Bin the data for empirical variogram
        let n_bins = 15;
        let max_dist = distances.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let bin_width = max_dist / n_bins as f64;

        let mut binned_distances = Vec::new();
        let mut binned_semivariances = Vec::new();

        for bin in 0..n_bins {
            let bin_start = bin as f64 * bin_width;
            let bin_end = (bin + 1) as f64 * bin_width;

            let mut bin_dists = Vec::new();
            let mut bin_semivars = Vec::new();

            for (i, &d) in distances.iter().enumerate() {
                if d >= bin_start && d < bin_end {
                    bin_dists.push(d);
                    bin_semivars.push(semivariances[i]);
                }
            }

            if !bin_dists.is_empty() {
                let avg_dist = bin_dists.iter().sum::<f64>() / bin_dists.len() as f64;
                let avg_semivar = bin_semivars.iter().sum::<f64>() / bin_semivars.len() as f64;

                binned_distances.push(avg_dist);
                binned_semivariances.push(avg_semivar);
            }
        }

        Ok((binned_distances, binned_semivariances))
    }

    /// Train kriging model
    pub fn train_with_coords(
        &mut self,
        coords: &Array2<f64>,
        values: &Array1<f64>,
    ) -> Result<()> {
        if coords.nrows() != values.len() {
            return Err(MlError::InvalidInput(
                "Number of coordinates must match values".to_string(),
            ));
        }

        self.training_coords = Some(coords.clone());
        self.training_values = Some(values.clone());

        // Fit variogram if not provided
        if self.variogram.is_none() {
            self.fit_variogram(coords, values, VariogramModel::Spherical)?;
        }

        // Calculate mean for simple kriging
        if matches!(self.kriging_type, KrigingType::Simple) {
            self.mean = Some(values.mean().unwrap_or(0.0));
        }

        self.trained = true;
        Ok(())
    }

    /// Predict at specific coordinates
    pub fn predict_at_coords(&self, coords: &Array2<f64>) -> Result<Array1<f64>> {
        if !self.trained {
            return Err(MlError::Model("Model not trained".to_string()));
        }

        let training_coords = self
            .training_coords
            .as_ref()
            .ok_or_else(|| MlError::Model("Training coordinates not available".to_string()))?;

        let training_values = self
            .training_values
            .as_ref()
            .ok_or_else(|| MlError::Model("Training values not available".to_string()))?;

        let variogram = self
            .variogram
            .as_ref()
            .ok_or_else(|| MlError::Model("Variogram not set".to_string()))?;

        let n_predict = coords.nrows();
        let n_train = training_coords.nrows();

        let mut predictions = Array1::zeros(n_predict);

        // For each prediction location
        for i in 0..n_predict {
            let pred_coord = coords.row(i);

            // Build kriging system
            let mut k_matrix = Array2::zeros((n_train + 1, n_train + 1));
            let mut k_vector = Array1::zeros(n_train + 1);

            // Fill covariance matrix
            for j in 0..n_train {
                for k in 0..n_train {
                    let dx = training_coords[[j, 0]] - training_coords[[k, 0]];
                    let dy = training_coords[[j, 1]] - training_coords[[k, 1]];
                    let dist = (dx * dx + dy * dy).sqrt();
                    k_matrix[[j, k]] = variogram.covariance(dist);
                }
                k_matrix[[j, n_train]] = 1.0;
                k_matrix[[n_train, j]] = 1.0;
            }

            // Fill covariance vector
            for j in 0..n_train {
                let dx = pred_coord[0] - training_coords[[j, 0]];
                let dy = pred_coord[1] - training_coords[[j, 1]];
                let dist = (dx * dx + dy * dy).sqrt();
                k_vector[j] = variogram.covariance(dist);
            }
            k_vector[n_train] = 1.0;

            // Solve kriging system: k_matrix * weights = k_vector
            // Simplified: use nearest neighbor for now
            // In production, use proper linear solver

            // Find nearest training point
            let mut min_dist = f64::INFINITY;
            let mut nearest_idx = 0;

            for j in 0..n_train {
                let dx = pred_coord[0] - training_coords[[j, 0]];
                let dy = pred_coord[1] - training_coords[[j, 1]];
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < min_dist {
                    min_dist = dist;
                    nearest_idx = j;
                }
            }

            predictions[i] = training_values[nearest_idx];
        }

        Ok(predictions)
    }
}

impl Regressor for Kriging {
    fn train(&mut self, _features: &FeatureSet, _targets: &Array1<f64>) -> Result<()> {
        Err(MlError::Model(
            "Kriging requires coordinates. Use train_with_coords()".to_string(),
        ))
    }

    fn predict(&self, _features: &FeatureSet) -> Result<Array1<f64>> {
        Err(MlError::Model(
            "Kriging requires coordinates. Use predict_at_coords()".to_string(),
        ))
    }

    fn predict_with_intervals(
        &self,
        _features: &FeatureSet,
        _confidence: f64,
    ) -> Result<RegressionResult> {
        Err(MlError::Model(
            "Kriging requires coordinates. Use predict_at_coords()".to_string(),
        ))
    }

    fn is_trained(&self) -> bool {
        self.trained
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variogram() {
        let var = Variogram::new(VariogramModel::Spherical, 0.1, 1.0, 10.0);
        assert_eq!(var.value(0.0), 0.0);
        assert!(var.value(5.0) > 0.1);
        assert!(var.value(20.0) <= 1.0);
    }

    #[test]
    fn test_kriging_creation() {
        let kriging = Kriging::new(KrigingType::Ordinary);
        assert!(!kriging.is_trained());
    }
}
