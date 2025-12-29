//! Feature scaling and normalization

use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use ndarray::{Array1, Array2, Axis};
use serde::{Deserialize, Serialize};

/// Feature scaler types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScalerType {
    /// Min-max normalization to [0, 1]
    MinMax,

    /// Standardization (z-score normalization)
    Standard,

    /// Robust scaling using median and IQR
    Robust,

    /// Max absolute scaling to [-1, 1]
    MaxAbs,

    /// No scaling
    None,
}

/// Configuration for feature scaling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalerConfig {
    /// Scaler type
    pub scaler_type: ScalerType,

    /// Feature range for MinMax scaler
    pub feature_range: (f64, f64),

    /// Whether to handle outliers
    pub clip_outliers: bool,

    /// Outlier threshold in standard deviations
    pub outlier_threshold: f64,
}

impl Default for ScalerConfig {
    fn default() -> Self {
        Self {
            scaler_type: ScalerType::Standard,
            feature_range: (0.0, 1.0),
            clip_outliers: false,
            outlier_threshold: 3.0,
        }
    }
}

/// Feature scaler for normalization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureScaler {
    /// Scaler configuration
    config: ScalerConfig,

    /// Fitted parameters per feature
    params: Option<Vec<ScalerParams>>,

    /// Whether the scaler has been fitted
    fitted: bool,
}

/// Parameters for each feature
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScalerParams {
    /// Minimum value (for MinMax)
    min: f64,

    /// Maximum value (for MinMax, MaxAbs)
    max: f64,

    /// Mean value (for Standard)
    mean: f64,

    /// Standard deviation (for Standard)
    std: f64,

    /// Median (for Robust)
    median: f64,

    /// Interquartile range (for Robust)
    iqr: f64,
}

impl FeatureScaler {
    /// Create a new feature scaler
    pub fn new(scaler_type: ScalerType) -> Self {
        Self {
            config: ScalerConfig {
                scaler_type,
                ..Default::default()
            },
            params: None,
            fitted: false,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ScalerConfig) -> Self {
        Self {
            config,
            params: None,
            fitted: false,
        }
    }

    /// Set feature range for MinMax scaler
    pub fn with_feature_range(mut self, min: f64, max: f64) -> Self {
        self.config.feature_range = (min, max);
        self
    }

    /// Enable outlier clipping
    pub fn with_outlier_clipping(mut self, threshold: f64) -> Self {
        self.config.clip_outliers = true;
        self.config.outlier_threshold = threshold;
        self
    }

    /// Fit the scaler to data
    pub fn fit(&mut self, features: &Array2<f64>) -> Result<()> {
        if self.config.scaler_type == ScalerType::None {
            self.fitted = true;
            return Ok(());
        }

        let n_features = features.ncols();
        let mut params = Vec::with_capacity(n_features);

        for col_idx in 0..n_features {
            let column = features.column(col_idx);
            let param = self.compute_params(&column)?;
            params.push(param);
        }

        self.params = Some(params);
        self.fitted = true;

        Ok(())
    }

    /// Compute parameters for a single feature
    fn compute_params(&self, values: &ndarray::ArrayView1<f64>) -> Result<ScalerParams> {
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean = values.mean().unwrap_or(0.0);
        let std = values.std(0.0);

        // Calculate median and IQR
        let mut sorted: Vec<f64> = values.iter().cloned().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = sorted.len();
        let median = if n % 2 == 0 {
            (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
        } else {
            sorted[n / 2]
        };

        let q1_idx = n / 4;
        let q3_idx = 3 * n / 4;
        let q1 = sorted[q1_idx];
        let q3 = sorted[q3_idx];
        let iqr = q3 - q1;

        Ok(ScalerParams {
            min,
            max,
            mean,
            std,
            median,
            iqr,
        })
    }

    /// Transform features
    pub fn transform(&self, features: &Array2<f64>) -> Result<Array2<f64>> {
        if !self.fitted {
            return Err(MlError::Model("Scaler not fitted".to_string()));
        }

        if self.config.scaler_type == ScalerType::None {
            return Ok(features.clone());
        }

        let params = self.params.as_ref().ok_or_else(|| {
            MlError::Model("Scaler parameters not available".to_string())
        })?;

        if features.ncols() != params.len() {
            return Err(MlError::InvalidFeatureDimensions {
                expected: params.len(),
                actual: features.ncols(),
            });
        }

        let mut transformed = features.clone();

        for (col_idx, param) in params.iter().enumerate() {
            let mut column = transformed.column_mut(col_idx);

            match self.config.scaler_type {
                ScalerType::MinMax => {
                    let range = param.max - param.min;
                    if range > 0.0 {
                        let (min_out, max_out) = self.config.feature_range;
                        let scale = (max_out - min_out) / range;
                        column.mapv_inplace(|x| (x - param.min) * scale + min_out);
                    }
                }
                ScalerType::Standard => {
                    if param.std > 0.0 {
                        column.mapv_inplace(|x| (x - param.mean) / param.std);
                    }
                }
                ScalerType::Robust => {
                    if param.iqr > 0.0 {
                        column.mapv_inplace(|x| (x - param.median) / param.iqr);
                    }
                }
                ScalerType::MaxAbs => {
                    let max_abs = param.max.abs().max(param.min.abs());
                    if max_abs > 0.0 {
                        column.mapv_inplace(|x| x / max_abs);
                    }
                }
                ScalerType::None => {}
            }

            // Clip outliers if enabled
            if self.config.clip_outliers {
                let threshold = self.config.outlier_threshold;
                column.mapv_inplace(|x| x.max(-threshold).min(threshold));
            }
        }

        Ok(transformed)
    }

    /// Fit and transform in one step
    pub fn fit_transform(&mut self, features: &Array2<f64>) -> Result<Array2<f64>> {
        self.fit(features)?;
        self.transform(features)
    }

    /// Inverse transform (denormalize)
    pub fn inverse_transform(&self, features: &Array2<f64>) -> Result<Array2<f64>> {
        if !self.fitted {
            return Err(MlError::Model("Scaler not fitted".to_string()));
        }

        if self.config.scaler_type == ScalerType::None {
            return Ok(features.clone());
        }

        let params = self.params.as_ref().ok_or_else(|| {
            MlError::Model("Scaler parameters not available".to_string())
        })?;

        if features.ncols() != params.len() {
            return Err(MlError::InvalidFeatureDimensions {
                expected: params.len(),
                actual: features.ncols(),
            });
        }

        let mut inverted = features.clone();

        for (col_idx, param) in params.iter().enumerate() {
            let mut column = inverted.column_mut(col_idx);

            match self.config.scaler_type {
                ScalerType::MinMax => {
                    let range = param.max - param.min;
                    if range > 0.0 {
                        let (min_out, max_out) = self.config.feature_range;
                        let scale = range / (max_out - min_out);
                        column.mapv_inplace(|x| (x - min_out) * scale + param.min);
                    }
                }
                ScalerType::Standard => {
                    if param.std > 0.0 {
                        column.mapv_inplace(|x| x * param.std + param.mean);
                    }
                }
                ScalerType::Robust => {
                    if param.iqr > 0.0 {
                        column.mapv_inplace(|x| x * param.iqr + param.median);
                    }
                }
                ScalerType::MaxAbs => {
                    let max_abs = param.max.abs().max(param.min.abs());
                    if max_abs > 0.0 {
                        column.mapv_inplace(|x| x * max_abs);
                    }
                }
                ScalerType::None => {}
            }
        }

        Ok(inverted)
    }

    /// Scale a FeatureSet
    pub fn transform_feature_set(&self, feature_set: &FeatureSet) -> Result<FeatureSet> {
        let scaled_features = self.transform(&feature_set.features)?;
        Ok(FeatureSet {
            features: scaled_features,
            names: feature_set.names.clone(),
            sample_ids: feature_set.sample_ids.clone(),
            metadata: feature_set.metadata.clone(),
        })
    }

    /// Check if scaler is fitted
    pub fn is_fitted(&self) -> bool {
        self.fitted
    }

    /// Get scaler type
    pub fn scaler_type(&self) -> ScalerType {
        self.config.scaler_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use ndarray::arr2;

    #[test]
    fn test_minmax_scaler() {
        let data = arr2(&[[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]);

        let mut scaler = FeatureScaler::new(ScalerType::MinMax);
        let scaled = scaler.fit_transform(&data).unwrap();

        assert_relative_eq!(scaled[[0, 0]], 0.0, epsilon = 1e-10);
        assert_relative_eq!(scaled[[2, 0]], 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_standard_scaler() {
        let data = arr2(&[[1.0], [2.0], [3.0], [4.0], [5.0]]);

        let mut scaler = FeatureScaler::new(ScalerType::Standard);
        let scaled = scaler.fit_transform(&data).unwrap();

        // Mean should be ~0, std should be ~1
        let mean = scaled.mean().unwrap();
        let std = scaled.std(0.0);

        assert_relative_eq!(mean, 0.0, epsilon = 1e-10);
        assert_relative_eq!(std, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_inverse_transform() {
        let data = arr2(&[[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]);

        let mut scaler = FeatureScaler::new(ScalerType::MinMax);
        let scaled = scaler.fit_transform(&data).unwrap();
        let inverted = scaler.inverse_transform(&scaled).unwrap();

        for i in 0..data.nrows() {
            for j in 0..data.ncols() {
                assert_relative_eq!(data[[i, j]], inverted[[i, j]], epsilon = 1e-10);
            }
        }
    }
}
