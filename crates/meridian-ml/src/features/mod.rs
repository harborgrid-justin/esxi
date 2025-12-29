//! Feature engineering and extraction

pub mod spatial;
pub mod raster;
pub mod temporal;
pub mod scaler;

pub use spatial::{SpatialFeatures, SpatialFeatureExtractor};
pub use raster::{RasterFeatures, RasterFeatureExtractor};
pub use temporal::{TemporalFeatures, TemporalFeatureExtractor};
pub use scaler::{FeatureScaler, ScalerType, ScalerConfig};

use crate::error::{MlError, Result};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Base trait for feature extraction
pub trait FeatureExtractor: Send + Sync {
    /// Extract features from input data
    fn extract(&self, input: &FeatureInput) -> Result<FeatureSet>;

    /// Get the expected number of features
    fn num_features(&self) -> usize;

    /// Get feature names
    fn feature_names(&self) -> Vec<String>;
}

/// Input data for feature extraction
#[derive(Debug, Clone)]
pub enum FeatureInput {
    /// Vector data (x, y coordinates)
    Vector(Vec<(f64, f64)>),

    /// Raster data (2D array)
    Raster(Array2<f64>),

    /// Time series data
    TimeSeries(Vec<(chrono::DateTime<chrono::Utc>, f64)>),

    /// Multi-dimensional array
    Array(ndarray::ArrayD<f64>),

    /// Custom data
    Custom(Box<dyn std::any::Any + Send + Sync>),
}

/// Extracted feature set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSet {
    /// Feature matrix (samples x features)
    pub features: Array2<f64>,

    /// Feature names
    pub names: Vec<String>,

    /// Sample identifiers
    pub sample_ids: Option<Vec<String>>,

    /// Feature metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl FeatureSet {
    /// Create a new feature set
    pub fn new(features: Array2<f64>, names: Vec<String>) -> Self {
        Self {
            features,
            names,
            sample_ids: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Get the number of samples
    pub fn num_samples(&self) -> usize {
        self.features.nrows()
    }

    /// Get the number of features
    pub fn num_features(&self) -> usize {
        self.features.ncols()
    }

    /// Get a single sample
    pub fn sample(&self, index: usize) -> Result<Array1<f64>> {
        if index >= self.num_samples() {
            return Err(MlError::InvalidInput(format!(
                "Sample index {} out of bounds (max: {})",
                index,
                self.num_samples() - 1
            )));
        }
        Ok(self.features.row(index).to_owned())
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Set sample IDs
    pub fn set_sample_ids(&mut self, ids: Vec<String>) -> Result<()> {
        if ids.len() != self.num_samples() {
            return Err(MlError::InvalidInput(format!(
                "Number of sample IDs ({}) does not match number of samples ({})",
                ids.len(),
                self.num_samples()
            )));
        }
        self.sample_ids = Some(ids);
        Ok(())
    }

    /// Select specific features by index
    pub fn select_features(&self, indices: &[usize]) -> Result<FeatureSet> {
        let mut selected_features = Array2::zeros((self.num_samples(), indices.len()));
        let mut selected_names = Vec::new();

        for (i, &idx) in indices.iter().enumerate() {
            if idx >= self.num_features() {
                return Err(MlError::InvalidInput(format!(
                    "Feature index {} out of bounds (max: {})",
                    idx,
                    self.num_features() - 1
                )));
            }
            selected_features.column_mut(i).assign(&self.features.column(idx));
            selected_names.push(self.names[idx].clone());
        }

        Ok(FeatureSet {
            features: selected_features,
            names: selected_names,
            sample_ids: self.sample_ids.clone(),
            metadata: self.metadata.clone(),
        })
    }

    /// Concatenate with another feature set
    pub fn concatenate(&self, other: &FeatureSet) -> Result<FeatureSet> {
        if self.num_samples() != other.num_samples() {
            return Err(MlError::InvalidInput(format!(
                "Cannot concatenate feature sets with different number of samples: {} vs {}",
                self.num_samples(),
                other.num_samples()
            )));
        }

        let combined = ndarray::concatenate![ndarray::Axis(1), self.features, other.features];
        let mut names = self.names.clone();
        names.extend(other.names.clone());

        Ok(FeatureSet {
            features: combined,
            names,
            sample_ids: self.sample_ids.clone(),
            metadata: self.metadata.clone(),
        })
    }

    /// Split into train and test sets
    pub fn train_test_split(&self, test_ratio: f64, shuffle: bool) -> Result<(FeatureSet, FeatureSet)> {
        if test_ratio <= 0.0 || test_ratio >= 1.0 {
            return Err(MlError::InvalidParameter {
                param: "test_ratio".to_string(),
                reason: "must be between 0 and 1".to_string(),
            });
        }

        let n_samples = self.num_samples();
        let n_test = (n_samples as f64 * test_ratio).round() as usize;
        let n_train = n_samples - n_test;

        let mut indices: Vec<usize> = (0..n_samples).collect();
        if shuffle {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            indices.shuffle(&mut rng);
        }

        let train_indices = &indices[..n_train];
        let test_indices = &indices[n_train..];

        let train = self.subset(train_indices)?;
        let test = self.subset(test_indices)?;

        Ok((train, test))
    }

    /// Create a subset of samples
    pub fn subset(&self, indices: &[usize]) -> Result<FeatureSet> {
        let mut subset_features = Array2::zeros((indices.len(), self.num_features()));
        let mut subset_ids = Vec::new();

        for (i, &idx) in indices.iter().enumerate() {
            if idx >= self.num_samples() {
                return Err(MlError::InvalidInput(format!(
                    "Sample index {} out of bounds (max: {})",
                    idx,
                    self.num_samples() - 1
                )));
            }
            subset_features.row_mut(i).assign(&self.features.row(idx));

            if let Some(ref ids) = self.sample_ids {
                subset_ids.push(ids[idx].clone());
            }
        }

        Ok(FeatureSet {
            features: subset_features,
            names: self.names.clone(),
            sample_ids: if subset_ids.is_empty() { None } else { Some(subset_ids) },
            metadata: self.metadata.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_feature_set_creation() {
        let features = arr2(&[[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]);
        let names = vec!["feature1".to_string(), "feature2".to_string()];
        let fs = FeatureSet::new(features, names);

        assert_eq!(fs.num_samples(), 3);
        assert_eq!(fs.num_features(), 2);
    }

    #[test]
    fn test_feature_selection() {
        let features = arr2(&[[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]);
        let names = vec!["f1".to_string(), "f2".to_string(), "f3".to_string()];
        let fs = FeatureSet::new(features, names);

        let selected = fs.select_features(&[0, 2]).unwrap();
        assert_eq!(selected.num_features(), 2);
        assert_eq!(selected.names, vec!["f1", "f3"]);
    }

    #[test]
    fn test_train_test_split() {
        let features = arr2(&[
            [1.0, 2.0],
            [3.0, 4.0],
            [5.0, 6.0],
            [7.0, 8.0],
            [9.0, 10.0],
        ]);
        let names = vec!["f1".to_string(), "f2".to_string()];
        let fs = FeatureSet::new(features, names);

        let (train, test) = fs.train_test_split(0.2, false).unwrap();
        assert_eq!(train.num_samples() + test.num_samples(), 5);
        assert_eq!(test.num_samples(), 1);
    }
}
