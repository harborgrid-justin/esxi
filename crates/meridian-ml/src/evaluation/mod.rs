//! Model evaluation and validation

pub mod metrics;

pub use metrics::{
    SpatialMetrics, ConfusionMatrix, ClassificationMetrics,
    RegressionMetrics, CrossValidator, SpatialCrossValidator,
};

use crate::error::{MlError, Result};
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Metric values
    pub metrics: HashMap<String, f64>,

    /// Per-class metrics (for classification)
    pub per_class_metrics: Option<HashMap<usize, HashMap<String, f64>>>,

    /// Confusion matrix (for classification)
    pub confusion_matrix: Option<ConfusionMatrix>,
}

impl EvaluationResult {
    /// Create a new evaluation result
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            per_class_metrics: None,
            confusion_matrix: None,
        }
    }

    /// Add a metric
    pub fn add_metric(&mut self, name: String, value: f64) {
        self.metrics.insert(name, value);
    }

    /// Get a metric value
    pub fn get_metric(&self, name: &str) -> Option<f64> {
        self.metrics.get(name).copied()
    }

    /// Set confusion matrix
    pub fn set_confusion_matrix(&mut self, cm: ConfusionMatrix) {
        self.confusion_matrix = Some(cm);
    }

    /// Pretty print results
    pub fn summary(&self) -> String {
        let mut summary = String::from("Evaluation Results:\n");
        summary.push_str("==================\n");

        for (name, value) in &self.metrics {
            summary.push_str(&format!("{}: {:.4}\n", name, value));
        }

        if let Some(ref cm) = self.confusion_matrix {
            summary.push_str("\nConfusion Matrix:\n");
            summary.push_str(&cm.to_string());
        }

        summary
    }
}

impl Default for EvaluationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Model evaluator
pub struct Evaluator {
    /// Cache computed metrics
    cache: HashMap<String, f64>,
}

impl Evaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Evaluate classification model
    pub fn evaluate_classification(
        &mut self,
        y_true: &Array1<usize>,
        y_pred: &Array1<usize>,
        num_classes: usize,
    ) -> Result<EvaluationResult> {
        if y_true.len() != y_pred.len() {
            return Err(MlError::InvalidInput(
                "True and predicted labels must have same length".to_string(),
            ));
        }

        let mut result = EvaluationResult::new();

        // Compute confusion matrix
        let cm = ConfusionMatrix::compute(y_true, y_pred, num_classes)?;
        result.set_confusion_matrix(cm.clone());

        // Compute metrics
        let metrics = ClassificationMetrics::from_confusion_matrix(&cm);

        result.add_metric("accuracy".to_string(), metrics.accuracy);
        result.add_metric("precision_macro".to_string(), metrics.precision_macro);
        result.add_metric("recall_macro".to_string(), metrics.recall_macro);
        result.add_metric("f1_macro".to_string(), metrics.f1_macro);

        if let Some(kappa) = metrics.kappa {
            result.add_metric("kappa".to_string(), kappa);
        }

        Ok(result)
    }

    /// Evaluate regression model
    pub fn evaluate_regression(
        &mut self,
        y_true: &Array1<f64>,
        y_pred: &Array1<f64>,
    ) -> Result<EvaluationResult> {
        if y_true.len() != y_pred.len() {
            return Err(MlError::InvalidInput(
                "True and predicted values must have same length".to_string(),
            ));
        }

        let mut result = EvaluationResult::new();

        let metrics = RegressionMetrics::compute(y_true, y_pred)?;

        result.add_metric("mse".to_string(), metrics.mse);
        result.add_metric("rmse".to_string(), metrics.rmse);
        result.add_metric("mae".to_string(), metrics.mae);
        result.add_metric("r2".to_string(), metrics.r2);

        if let Some(mape) = metrics.mape {
            result.add_metric("mape".to_string(), mape);
        }

        Ok(result)
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluation_result() {
        let mut result = EvaluationResult::new();
        result.add_metric("accuracy".to_string(), 0.95);

        assert_eq!(result.get_metric("accuracy"), Some(0.95));
    }

    #[test]
    fn test_evaluator() {
        let evaluator = Evaluator::new();
        assert_eq!(evaluator.cache.len(), 0);
    }
}
