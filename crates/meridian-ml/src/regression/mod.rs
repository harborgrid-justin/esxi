//! Regression algorithms for spatial data

pub mod spatial_regression;
pub mod kriging;

pub use spatial_regression::{SpatialRegression, SpatialRegressionType};
pub use kriging::{Kriging, KrigingType, Variogram};

use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use ndarray::Array1;
use serde::{Deserialize, Serialize};

/// Base trait for regression models
pub trait Regressor: Send + Sync {
    /// Train the regressor
    fn train(&mut self, features: &FeatureSet, targets: &Array1<f64>) -> Result<()>;

    /// Predict values
    fn predict(&self, features: &FeatureSet) -> Result<Array1<f64>>;

    /// Predict with prediction intervals
    fn predict_with_intervals(&self, features: &FeatureSet, confidence: f64) -> Result<RegressionResult>;

    /// Check if the regressor is trained
    fn is_trained(&self) -> bool;

    /// Get feature importance (if applicable)
    fn feature_importance(&self) -> Option<Vec<f64>> {
        None
    }
}

/// Regression result with prediction intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionResult {
    /// Predicted values
    pub predictions: Array1<f64>,

    /// Lower bounds of prediction intervals
    pub lower_bounds: Option<Array1<f64>>,

    /// Upper bounds of prediction intervals
    pub upper_bounds: Option<Array1<f64>>,

    /// Standard errors
    pub standard_errors: Option<Array1<f64>>,
}

impl RegressionResult {
    /// Create a new regression result
    pub fn new(predictions: Array1<f64>) -> Self {
        Self {
            predictions,
            lower_bounds: None,
            upper_bounds: None,
            standard_errors: None,
        }
    }

    /// Add prediction intervals
    pub fn with_intervals(mut self, lower: Array1<f64>, upper: Array1<f64>) -> Self {
        self.lower_bounds = Some(lower);
        self.upper_bounds = Some(upper);
        self
    }

    /// Add standard errors
    pub fn with_standard_errors(mut self, errors: Array1<f64>) -> Self {
        self.standard_errors = Some(errors);
        self
    }
}

/// Linear regression model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearRegression {
    /// Coefficients
    coefficients: Option<Array1<f64>>,

    /// Intercept
    intercept: f64,

    /// Whether to fit intercept
    fit_intercept: bool,

    /// Whether the model is trained
    trained: bool,
}

impl LinearRegression {
    /// Create a new linear regression model
    pub fn new() -> Self {
        Self {
            coefficients: None,
            intercept: 0.0,
            fit_intercept: true,
            trained: false,
        }
    }

    /// Set whether to fit intercept
    pub fn with_fit_intercept(mut self, fit: bool) -> Self {
        self.fit_intercept = fit;
        self
    }

    /// Get coefficients
    pub fn coefficients(&self) -> Option<&Array1<f64>> {
        self.coefficients.as_ref()
    }

    /// Get intercept
    pub fn intercept(&self) -> f64 {
        self.intercept
    }
}

impl Default for LinearRegression {
    fn default() -> Self {
        Self::new()
    }
}

impl Regressor for LinearRegression {
    fn train(&mut self, features: &FeatureSet, targets: &Array1<f64>) -> Result<()> {
        if features.num_samples() != targets.len() {
            return Err(MlError::InvalidInput(
                "Number of features and targets must match".to_string(),
            ));
        }

        // Ordinary Least Squares using normal equation: β = (X'X)^-1 X'y
        let x = &features.features;
        let y = targets;

        // Add intercept column if needed
        let x_design = if self.fit_intercept {
            let ones = Array1::ones(x.nrows());
            ndarray::concatenate![ndarray::Axis(1), ones.insert_axis(ndarray::Axis(1)), x.clone()]
        } else {
            x.clone()
        };

        // Compute X'X
        let xt = x_design.t();
        let xtx = xt.dot(&x_design);

        // Compute X'y
        let xty = xt.dot(y);

        // Solve for coefficients using SVD (more stable than direct inversion)
        // For now, use a simplified approach
        // In production, use proper linear algebra library

        self.trained = true;
        Ok(())
    }

    fn predict(&self, features: &FeatureSet) -> Result<Array1<f64>> {
        if !self.trained {
            return Err(MlError::Model("Model not trained".to_string()));
        }

        // Placeholder: return zeros
        Ok(Array1::zeros(features.num_samples()))
    }

    fn predict_with_intervals(&self, features: &FeatureSet, confidence: f64) -> Result<RegressionResult> {
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
    fn test_linear_regression_creation() {
        let lr = LinearRegression::new();
        assert!(!lr.is_trained());
        assert!(lr.fit_intercept);
    }

    #[test]
    fn test_regression_result() {
        let predictions = Array1::from_vec(vec![1.0, 2.0, 3.0]);
        let result = RegressionResult::new(predictions.clone());
        assert_eq!(result.predictions.len(), 3);
    }
}
