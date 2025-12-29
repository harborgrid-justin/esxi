//! Predictive modeling for spatial data

pub mod time_series;
pub mod anomaly;

pub use time_series::{SpatialTimeSeries, ForecastResult};
pub use anomaly::{AnomalyDetector, AnomalyType, AnomalyResult};

use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use ndarray::Array1;
use serde::{Deserialize, Serialize};

/// Base trait for predictors
pub trait Predictor: Send + Sync {
    /// Train the predictor
    fn train(&mut self, features: &FeatureSet, targets: &Array1<f64>) -> Result<()>;

    /// Predict future values
    fn predict(&self, features: &FeatureSet) -> Result<Array1<f64>>;

    /// Predict with uncertainty estimates
    fn predict_with_uncertainty(&self, features: &FeatureSet) -> Result<PredictionResult>;

    /// Check if the predictor is trained
    fn is_trained(&self) -> bool;
}

/// Prediction result with uncertainty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// Predicted values
    pub predictions: Array1<f64>,

    /// Lower confidence bound
    pub lower_bound: Option<Array1<f64>>,

    /// Upper confidence bound
    pub upper_bound: Option<Array1<f64>>,

    /// Prediction variance
    pub variance: Option<Array1<f64>>,

    /// Confidence level (e.g., 0.95)
    pub confidence_level: f64,
}

impl PredictionResult {
    /// Create a new prediction result
    pub fn new(predictions: Array1<f64>, confidence_level: f64) -> Self {
        Self {
            predictions,
            lower_bound: None,
            upper_bound: None,
            variance: None,
            confidence_level,
        }
    }

    /// Add confidence bounds
    pub fn with_bounds(mut self, lower: Array1<f64>, upper: Array1<f64>) -> Self {
        self.lower_bound = Some(lower);
        self.upper_bound = Some(upper);
        self
    }

    /// Add variance
    pub fn with_variance(mut self, variance: Array1<f64>) -> Self {
        self.variance = Some(variance);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prediction_result() {
        let predictions = Array1::from_vec(vec![1.0, 2.0, 3.0]);
        let result = PredictionResult::new(predictions, 0.95);
        assert_eq!(result.confidence_level, 0.95);
    }
}
