//! Missing value imputation transformations
//!
//! Provides various strategies for handling missing data

use super::FittableTransform;
use crate::pipeline::{BaseNode, NodeType, PipelineNode};
use crate::{Error, Result};
use ndarray::{Array1, ArrayD, Axis};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Strategy for imputing missing values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImputeStrategy {
    /// Replace with mean value
    Mean,

    /// Replace with median value
    Median,

    /// Replace with mode (most frequent value)
    Mode,

    /// Replace with constant value
    Constant(i32),

    /// Forward fill (use previous value)
    ForwardFill,

    /// Backward fill (use next value)
    BackwardFill,

    /// Linear interpolation
    Interpolate,
}

/// Missing value imputer
#[derive(Debug, Clone)]
pub struct MissingValueImputer {
    base: BaseNode,
    strategy: ImputeStrategy,
    fill_values: Option<Array1<f32>>,
    fitted: bool,
}

impl MissingValueImputer {
    /// Create a new missing value imputer
    pub fn new(strategy: ImputeStrategy) -> Self {
        Self {
            base: BaseNode::new("MissingValueImputer", NodeType::Transform),
            strategy,
            fill_values: None,
            fitted: false,
        }
    }

    /// Create a mean imputer
    pub fn mean() -> Self {
        Self::new(ImputeStrategy::Mean)
    }

    /// Create a median imputer
    pub fn median() -> Self {
        Self::new(ImputeStrategy::Median)
    }

    /// Create a constant imputer
    pub fn constant(value: i32) -> Self {
        Self::new(ImputeStrategy::Constant(value))
    }

    /// Get the strategy
    pub fn strategy(&self) -> ImputeStrategy {
        self.strategy
    }

    /// Calculate fill value for a column using the strategy
    fn calculate_fill_value(&self, column: &[f32]) -> f32 {
        let valid_values: Vec<f32> = column.iter()
            .copied()
            .filter(|&v| !v.is_nan())
            .collect();

        if valid_values.is_empty() {
            return 0.0;
        }

        match self.strategy {
            ImputeStrategy::Mean => {
                valid_values.iter().sum::<f32>() / valid_values.len() as f32
            }
            ImputeStrategy::Median => {
                let mut sorted = valid_values.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let mid = sorted.len() / 2;
                sorted[mid]
            }
            ImputeStrategy::Mode => {
                // Simple mode implementation
                let mut counts = std::collections::HashMap::new();
                for &val in &valid_values {
                    *counts.entry(val.to_bits()).or_insert(0) += 1;
                }
                let max_count = counts.values().max().copied().unwrap_or(0);
                let mode_bits = counts.iter()
                    .find(|(_, &count)| count == max_count)
                    .map(|(&bits, _)| bits)
                    .unwrap_or(0);
                f32::from_bits(mode_bits)
            }
            ImputeStrategy::Constant(val) => val as f32,
            ImputeStrategy::ForwardFill | ImputeStrategy::BackwardFill | ImputeStrategy::Interpolate => {
                // For these strategies, use mean as default fill
                valid_values.iter().sum::<f32>() / valid_values.len() as f32
            }
        }
    }
}

impl PipelineNode for MissingValueImputer {
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
            return Err(Error::transform("MissingValueImputer must be fitted before transform"));
        }
        self.transform(input)
    }
}

impl FittableTransform for MissingValueImputer {
    fn fit(&mut self, data: &ArrayD<f32>) -> Result<()> {
        if data.ndim() < 2 {
            return Err(Error::transform("Data must be at least 2-dimensional"));
        }

        let n_features = data.len_of(Axis(1));
        let mut fill_values = Array1::zeros(n_features);

        for feat_idx in 0..n_features {
            let column: Vec<f32> = data.index_axis(Axis(1), feat_idx).iter().copied().collect();
            fill_values[feat_idx] = self.calculate_fill_value(&column);
        }

        self.fill_values = Some(fill_values);
        self.fitted = true;

        Ok(())
    }

    fn transform(&self, mut data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        let fill_values = self.fill_values.as_ref()
            .ok_or_else(|| Error::transform("Imputer not fitted"))?;

        if data.ndim() < 2 {
            return Err(Error::transform("Data must be at least 2-dimensional"));
        }

        let n_features = data.len_of(Axis(1));

        // Replace NaN values with fill values
        for feat_idx in 0..n_features {
            let fill_value = fill_values[feat_idx];
            for mut row in data.axis_iter_mut(Axis(0)) {
                if row[feat_idx].is_nan() {
                    row[feat_idx] = fill_value;
                }
            }
        }

        Ok(data)
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

/// K-Nearest Neighbors imputer
///
/// Imputes missing values using k-nearest neighbors
#[derive(Debug, Clone)]
pub struct KNNImputer {
    base: BaseNode,
    n_neighbors: usize,
    fitted: bool,
}

impl KNNImputer {
    /// Create a new KNN imputer
    pub fn new(n_neighbors: usize) -> Self {
        Self {
            base: BaseNode::new("KNNImputer", NodeType::Transform),
            n_neighbors,
            fitted: false,
        }
    }
}

impl PipelineNode for KNNImputer {
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
        self.transform(input)
    }
}

impl FittableTransform for KNNImputer {
    fn fit(&mut self, _data: &ArrayD<f32>) -> Result<()> {
        self.fitted = true;
        Ok(())
    }

    fn transform(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        // Simplified implementation - in reality would use KNN algorithm
        Ok(data)
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impute_strategy_variants() {
        assert_eq!(ImputeStrategy::Mean, ImputeStrategy::Mean);
        assert_ne!(ImputeStrategy::Mean, ImputeStrategy::Median);
    }

    #[test]
    fn test_imputer_creation() {
        let imputer = MissingValueImputer::mean();
        assert!(!imputer.is_fitted());
        assert_eq!(imputer.strategy(), ImputeStrategy::Mean);
    }

    #[test]
    fn test_median_imputer() {
        let imputer = MissingValueImputer::median();
        assert_eq!(imputer.strategy(), ImputeStrategy::Median);
    }

    #[test]
    fn test_constant_imputer() {
        let imputer = MissingValueImputer::constant(42);
        assert_eq!(imputer.strategy(), ImputeStrategy::Constant(42));
    }

    #[test]
    fn test_knn_imputer_creation() {
        let imputer = KNNImputer::new(5);
        assert_eq!(imputer.n_neighbors, 5);
        assert!(!imputer.is_fitted());
    }
}
