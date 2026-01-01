//! Data transformation modules
//!
//! Provides various data transformation operations for ML pipelines

pub mod normalize;
pub mod encode;
pub mod impute;
pub mod feature;

pub use normalize::{StandardScaler, MinMaxScaler, RobustScaler};
pub use encode::{CategoricalEncoder, OneHotEncoder, LabelEncoder};
pub use impute::{MissingValueImputer, ImputeStrategy};
pub use feature::{FeatureEngineer, FeatureSelector};

use crate::pipeline::{PipelineNode, NodeType, BaseNode};
use crate::Result;
use ndarray::ArrayD;

/// Trait for stateful transformers that need to be fit before transform
pub trait FittableTransform: PipelineNode {
    /// Fit the transformer to training data
    fn fit(&mut self, data: &ArrayD<f32>) -> Result<()>;

    /// Transform data using fitted parameters
    fn transform(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>>;

    /// Fit and transform in one step
    fn fit_transform(&mut self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        self.fit(&data)?;
        self.transform(data)
    }

    /// Check if the transformer has been fitted
    fn is_fitted(&self) -> bool;
}

/// Statistics for data transformations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransformStats {
    /// Number of samples processed
    pub samples: usize,

    /// Number of features
    pub features: usize,

    /// Number of missing values handled
    pub missing_values: usize,

    /// Number of outliers detected
    pub outliers: usize,
}

impl Default for TransformStats {
    fn default() -> Self {
        Self {
            samples: 0,
            features: 0,
            missing_values: 0,
            outliers: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_stats_default() {
        let stats = TransformStats::default();
        assert_eq!(stats.samples, 0);
        assert_eq!(stats.features, 0);
    }
}
