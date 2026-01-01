//! Normalization transformations
//!
//! Provides various normalization strategies for numerical data

use super::{FittableTransform, TransformStats};
use crate::pipeline::{BaseNode, NodeType, PipelineNode};
use crate::{Error, Result};
use ndarray::{Array1, ArrayD, Axis};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Standard scaler (z-score normalization)
///
/// Transforms features to have zero mean and unit variance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardScaler {
    #[serde(flatten)]
    base: BaseNode,

    /// Mean values for each feature
    mean: Option<Array1<f32>>,

    /// Standard deviation for each feature
    std: Option<Array1<f32>>,

    /// Whether the scaler has been fitted
    fitted: bool,

    /// Transformation statistics
    stats: TransformStats,
}

impl StandardScaler {
    /// Create a new standard scaler
    pub fn new() -> Self {
        Self {
            base: BaseNode::new("StandardScaler", NodeType::Transform),
            mean: None,
            std: None,
            fitted: false,
            stats: TransformStats::default(),
        }
    }

    /// Get the mean values
    pub fn mean(&self) -> Option<&Array1<f32>> {
        self.mean.as_ref()
    }

    /// Get the standard deviation values
    pub fn std(&self) -> Option<&Array1<f32>> {
        self.std.as_ref()
    }
}

impl Default for StandardScaler {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineNode for StandardScaler {
    fn id(&self) -> Uuid {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn node_type(&self) -> NodeType {
        self.base.node_type
    }

    fn execute(&self, input: ArrayD<f32>) -> Result<ArrayD<f32>> {
        if !self.fitted {
            return Err(Error::transform("StandardScaler must be fitted before transform"));
        }

        self.transform(input)
    }

    fn validate(&self) -> Result<()> {
        if !self.fitted {
            return Err(Error::transform("StandardScaler not fitted"));
        }
        Ok(())
    }
}

impl FittableTransform for StandardScaler {
    fn fit(&mut self, data: &ArrayD<f32>) -> Result<()> {
        if data.ndim() < 2 {
            return Err(Error::transform("Data must be at least 2-dimensional"));
        }

        let n_samples = data.len_of(Axis(0));
        let n_features = data.len_of(Axis(1));

        // Calculate mean for each feature
        let mean = data.mean_axis(Axis(0))
            .ok_or_else(|| Error::transform("Failed to calculate mean"))?;

        // Calculate standard deviation for each feature
        let centered = data.to_owned() - &mean;
        let variance = (&centered * &centered).mean_axis(Axis(0))
            .ok_or_else(|| Error::transform("Failed to calculate variance"))?;
        let std = variance.mapv(|v| v.sqrt().max(1e-8)); // Avoid division by zero

        // Convert to Array1
        let mean_arr = Array1::from_vec(mean.into_iter().collect());
        let std_arr = Array1::from_vec(std.into_iter().collect());

        self.mean = Some(mean_arr);
        self.std = Some(std_arr);
        self.fitted = true;
        self.stats.samples = n_samples;
        self.stats.features = n_features;

        Ok(())
    }

    fn transform(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        let mean = self.mean.as_ref()
            .ok_or_else(|| Error::transform("Scaler not fitted"))?;
        let std = self.std.as_ref()
            .ok_or_else(|| Error::transform("Scaler not fitted"))?;

        let normalized = (data - mean) / std;
        Ok(normalized)
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

/// Min-Max scaler
///
/// Scales features to a specified range [min, max]
#[derive(Debug, Clone)]
pub struct MinMaxScaler {
    base: BaseNode,
    min_val: f32,
    max_val: f32,
    data_min: Option<Array1<f32>>,
    data_max: Option<Array1<f32>>,
    fitted: bool,
}

impl MinMaxScaler {
    /// Create a new min-max scaler
    pub fn new(min_val: f32, max_val: f32) -> Self {
        Self {
            base: BaseNode::new("MinMaxScaler", NodeType::Transform),
            min_val,
            max_val,
            data_min: None,
            data_max: None,
            fitted: false,
        }
    }

    /// Create a scaler for [0, 1] range
    pub fn zero_one() -> Self {
        Self::new(0.0, 1.0)
    }
}

impl PipelineNode for MinMaxScaler {
    fn id(&self) -> Uuid {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn node_type(&self) -> NodeType {
        self.base.node_type
    }

    fn execute(&self, input: ArrayD<f32>) -> Result<ArrayD<f32>> {
        if !self.fitted {
            return Err(Error::transform("MinMaxScaler must be fitted before transform"));
        }
        self.transform(input)
    }
}

impl FittableTransform for MinMaxScaler {
    fn fit(&mut self, data: &ArrayD<f32>) -> Result<()> {
        if data.ndim() < 2 {
            return Err(Error::transform("Data must be at least 2-dimensional"));
        }

        // Find min and max for each feature
        let data_min_vals = data.map_axis(Axis(0), |col| {
            col.iter().copied().fold(f32::INFINITY, f32::min)
        });

        let data_max_vals = data.map_axis(Axis(0), |col| {
            col.iter().copied().fold(f32::NEG_INFINITY, f32::max)
        });

        // Convert to Array1
        let min_arr = Array1::from_vec(data_min_vals.into_iter().collect());
        let max_arr = Array1::from_vec(data_max_vals.into_iter().collect());

        self.data_min = Some(min_arr);
        self.data_max = Some(max_arr);
        self.fitted = true;

        Ok(())
    }

    fn transform(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        let data_min = self.data_min.as_ref()
            .ok_or_else(|| Error::transform("Scaler not fitted"))?;
        let data_max = self.data_max.as_ref()
            .ok_or_else(|| Error::transform("Scaler not fitted"))?;

        let range = data_max - data_min;
        let range = range.mapv(|v| if v == 0.0 { 1.0 } else { v });

        let normalized = ((data - data_min) / &range) * (self.max_val - self.min_val) + self.min_val;

        Ok(normalized)
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

/// Robust scaler using median and IQR
///
/// More robust to outliers than StandardScaler
#[derive(Debug, Clone)]
pub struct RobustScaler {
    base: BaseNode,
    median: Option<Array1<f32>>,
    iqr: Option<Array1<f32>>,
    fitted: bool,
}

impl RobustScaler {
    /// Create a new robust scaler
    pub fn new() -> Self {
        Self {
            base: BaseNode::new("RobustScaler", NodeType::Transform),
            median: None,
            iqr: None,
            fitted: false,
        }
    }
}

impl Default for RobustScaler {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineNode for RobustScaler {
    fn id(&self) -> Uuid {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn node_type(&self) -> NodeType {
        self.base.node_type
    }

    fn execute(&self, input: ArrayD<f32>) -> Result<ArrayD<f32>> {
        if !self.fitted {
            return Err(Error::transform("RobustScaler must be fitted before transform"));
        }
        self.transform(input)
    }
}

impl FittableTransform for RobustScaler {
    fn fit(&mut self, data: &ArrayD<f32>) -> Result<()> {
        if data.ndim() < 2 {
            return Err(Error::transform("Data must be at least 2-dimensional"));
        }

        // Calculate median and IQR for each feature
        let n_features = data.len_of(Axis(1));
        let mut medians = Array1::zeros(n_features);
        let mut iqrs = Array1::zeros(n_features);

        for feat_idx in 0..n_features {
            let mut column: Vec<f32> = data.index_axis(Axis(1), feat_idx).iter().copied().collect();
            column.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let n = column.len();
            let q1_idx = n / 4;
            let q2_idx = n / 2;
            let q3_idx = 3 * n / 4;

            medians[feat_idx] = column[q2_idx];
            iqrs[feat_idx] = (column[q3_idx] - column[q1_idx]).max(1e-8);
        }

        self.median = Some(medians);
        self.iqr = Some(iqrs);
        self.fitted = true;

        Ok(())
    }

    fn transform(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        let median = self.median.as_ref()
            .ok_or_else(|| Error::transform("Scaler not fitted"))?;
        let iqr = self.iqr.as_ref()
            .ok_or_else(|| Error::transform("Scaler not fitted"))?;

        let scaled = (data - median) / iqr;
        Ok(scaled)
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_standard_scaler_creation() {
        let scaler = StandardScaler::new();
        assert!(!scaler.is_fitted());
        assert_eq!(scaler.name(), "StandardScaler");
    }

    #[test]
    fn test_minmax_scaler_zero_one() {
        let scaler = MinMaxScaler::zero_one();
        assert_eq!(scaler.min_val, 0.0);
        assert_eq!(scaler.max_val, 1.0);
    }

    #[test]
    fn test_robust_scaler_creation() {
        let scaler = RobustScaler::new();
        assert!(!scaler.is_fitted());
    }
}
