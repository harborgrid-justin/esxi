//! Evaluation metrics for spatial ML models

use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Confusion matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfusionMatrix {
    /// Matrix values
    pub matrix: Array2<usize>,

    /// Number of classes
    pub num_classes: usize,

    /// Class labels
    pub class_labels: Option<Vec<String>>,
}

impl ConfusionMatrix {
    /// Compute confusion matrix
    pub fn compute(
        y_true: &Array1<usize>,
        y_pred: &Array1<usize>,
        num_classes: usize,
    ) -> Result<Self> {
        if y_true.len() != y_pred.len() {
            return Err(MlError::InvalidInput(
                "True and predicted labels must have same length".to_string(),
            ));
        }

        let mut matrix = Array2::zeros((num_classes, num_classes));

        for (i, &true_label) in y_true.iter().enumerate() {
            let pred_label = y_pred[i];

            if true_label >= num_classes || pred_label >= num_classes {
                return Err(MlError::InvalidInput(format!(
                    "Label {} exceeds number of classes {}",
                    true_label.max(pred_label),
                    num_classes
                )));
            }

            matrix[[true_label, pred_label]] += 1;
        }

        Ok(Self {
            matrix,
            num_classes,
            class_labels: None,
        })
    }

    /// Get total number of samples
    pub fn total(&self) -> usize {
        self.matrix.sum()
    }

    /// Get true positives for a class
    pub fn true_positives(&self, class: usize) -> usize {
        self.matrix[[class, class]]
    }

    /// Get false positives for a class
    pub fn false_positives(&self, class: usize) -> usize {
        self.matrix.column(class).sum() - self.true_positives(class)
    }

    /// Get false negatives for a class
    pub fn false_negatives(&self, class: usize) -> usize {
        self.matrix.row(class).sum() - self.true_positives(class)
    }

    /// Get true negatives for a class
    pub fn true_negatives(&self, class: usize) -> usize {
        self.total()
            - self.true_positives(class)
            - self.false_positives(class)
            - self.false_negatives(class)
    }

    /// Compute accuracy
    pub fn accuracy(&self) -> f64 {
        let correct: usize = (0..self.num_classes)
            .map(|i| self.true_positives(i))
            .sum();
        correct as f64 / self.total() as f64
    }

    /// Normalize confusion matrix
    pub fn normalize(&self) -> Array2<f64> {
        let mut normalized = Array2::zeros((self.num_classes, self.num_classes));

        for i in 0..self.num_classes {
            let row_sum: usize = self.matrix.row(i).sum();
            if row_sum > 0 {
                for j in 0..self.num_classes {
                    normalized[[i, j]] = self.matrix[[i, j]] as f64 / row_sum as f64;
                }
            }
        }

        normalized
    }
}

impl std::fmt::Display for ConfusionMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Predicted →")?;
        for i in 0..self.num_classes {
            for j in 0..self.num_classes {
                write!(f, "{:6} ", self.matrix[[i, j]])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Classification metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationMetrics {
    /// Accuracy
    pub accuracy: f64,

    /// Precision (macro-averaged)
    pub precision_macro: f64,

    /// Recall (macro-averaged)
    pub recall_macro: f64,

    /// F1 score (macro-averaged)
    pub f1_macro: f64,

    /// Per-class precision
    pub precision_per_class: Vec<f64>,

    /// Per-class recall
    pub recall_per_class: Vec<f64>,

    /// Per-class F1 score
    pub f1_per_class: Vec<f64>,

    /// Cohen's Kappa
    pub kappa: Option<f64>,
}

impl ClassificationMetrics {
    /// Compute metrics from confusion matrix
    pub fn from_confusion_matrix(cm: &ConfusionMatrix) -> Self {
        let num_classes = cm.num_classes;

        let mut precision_per_class = Vec::new();
        let mut recall_per_class = Vec::new();
        let mut f1_per_class = Vec::new();

        for class in 0..num_classes {
            let tp = cm.true_positives(class) as f64;
            let fp = cm.false_positives(class) as f64;
            let fn_count = cm.false_negatives(class) as f64;

            let precision = if tp + fp > 0.0 {
                tp / (tp + fp)
            } else {
                0.0
            };

            let recall = if tp + fn_count > 0.0 {
                tp / (tp + fn_count)
            } else {
                0.0
            };

            let f1 = if precision + recall > 0.0 {
                2.0 * precision * recall / (precision + recall)
            } else {
                0.0
            };

            precision_per_class.push(precision);
            recall_per_class.push(recall);
            f1_per_class.push(f1);
        }

        let precision_macro = precision_per_class.iter().sum::<f64>() / num_classes as f64;
        let recall_macro = recall_per_class.iter().sum::<f64>() / num_classes as f64;
        let f1_macro = f1_per_class.iter().sum::<f64>() / num_classes as f64;

        let accuracy = cm.accuracy();

        // Compute Cohen's Kappa
        let kappa = Self::compute_kappa(cm);

        Self {
            accuracy,
            precision_macro,
            recall_macro,
            f1_macro,
            precision_per_class,
            recall_per_class,
            f1_per_class,
            kappa,
        }
    }

    /// Compute Cohen's Kappa
    fn compute_kappa(cm: &ConfusionMatrix) -> Option<f64> {
        let total = cm.total() as f64;
        if total == 0.0 {
            return None;
        }

        let po = cm.accuracy(); // Observed agreement

        // Expected agreement
        let mut pe = 0.0;
        for i in 0..cm.num_classes {
            let row_sum: usize = cm.matrix.row(i).sum();
            let col_sum: usize = cm.matrix.column(i).sum();
            pe += (row_sum as f64 * col_sum as f64) / (total * total);
        }

        if pe == 1.0 {
            return None;
        }

        Some((po - pe) / (1.0 - pe))
    }
}

/// Regression metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionMetrics {
    /// Mean Squared Error
    pub mse: f64,

    /// Root Mean Squared Error
    pub rmse: f64,

    /// Mean Absolute Error
    pub mae: f64,

    /// R² (coefficient of determination)
    pub r2: f64,

    /// Mean Absolute Percentage Error
    pub mape: Option<f64>,

    /// Adjusted R²
    pub adjusted_r2: Option<f64>,
}

impl RegressionMetrics {
    /// Compute regression metrics
    pub fn compute(y_true: &Array1<f64>, y_pred: &Array1<f64>) -> Result<Self> {
        if y_true.len() != y_pred.len() {
            return Err(MlError::InvalidInput(
                "True and predicted values must have same length".to_string(),
            ));
        }

        let n = y_true.len() as f64;

        // MSE
        let mse = y_true
            .iter()
            .zip(y_pred.iter())
            .map(|(&yt, &yp)| (yt - yp).powi(2))
            .sum::<f64>()
            / n;

        // RMSE
        let rmse = mse.sqrt();

        // MAE
        let mae = y_true
            .iter()
            .zip(y_pred.iter())
            .map(|(&yt, &yp)| (yt - yp).abs())
            .sum::<f64>()
            / n;

        // R²
        let mean_y = y_true.mean().unwrap_or(0.0);
        let ss_tot = y_true.iter().map(|&y| (y - mean_y).powi(2)).sum::<f64>();
        let ss_res = y_true
            .iter()
            .zip(y_pred.iter())
            .map(|(&yt, &yp)| (yt - yp).powi(2))
            .sum::<f64>();

        let r2 = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        // MAPE (only if no zeros in y_true)
        let has_zeros = y_true.iter().any(|&y| y == 0.0);
        let mape = if !has_zeros {
            Some(
                y_true
                    .iter()
                    .zip(y_pred.iter())
                    .map(|(&yt, &yp)| ((yt - yp) / yt).abs())
                    .sum::<f64>()
                    / n
                    * 100.0,
            )
        } else {
            None
        };

        Ok(Self {
            mse,
            rmse,
            mae,
            r2,
            mape,
            adjusted_r2: None,
        })
    }
}

/// Cross-validation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossValidationStrategy {
    /// K-fold cross-validation
    KFold { k: usize },

    /// Stratified K-fold
    StratifiedKFold { k: usize },

    /// Leave-one-out
    LeaveOneOut,

    /// Time series split
    TimeSeriesSplit { n_splits: usize },
}

/// Cross-validator
pub struct CrossValidator {
    /// Strategy
    strategy: CrossValidationStrategy,

    /// Random seed
    random_seed: Option<u64>,
}

impl CrossValidator {
    /// Create a new cross-validator
    pub fn new(strategy: CrossValidationStrategy) -> Self {
        Self {
            strategy,
            random_seed: Some(42),
        }
    }

    /// K-fold cross-validation
    pub fn kfold(k: usize) -> Self {
        Self::new(CrossValidationStrategy::KFold { k })
    }

    /// Generate train/test splits
    pub fn split(&self, n_samples: usize) -> Vec<(Vec<usize>, Vec<usize>)> {
        match self.strategy {
            CrossValidationStrategy::KFold { k } => self.kfold_split(n_samples, k),
            CrossValidationStrategy::LeaveOneOut => self.loo_split(n_samples),
            _ => vec![], // Placeholder for other strategies
        }
    }

    /// K-fold split implementation
    fn kfold_split(&self, n_samples: usize, k: usize) -> Vec<(Vec<usize>, Vec<usize>)> {
        let fold_size = n_samples / k;
        let mut splits = Vec::new();

        for fold in 0..k {
            let test_start = fold * fold_size;
            let test_end = if fold == k - 1 {
                n_samples
            } else {
                (fold + 1) * fold_size
            };

            let test_indices: Vec<usize> = (test_start..test_end).collect();
            let train_indices: Vec<usize> = (0..test_start)
                .chain(test_end..n_samples)
                .collect();

            splits.push((train_indices, test_indices));
        }

        splits
    }

    /// Leave-one-out split
    fn loo_split(&self, n_samples: usize) -> Vec<(Vec<usize>, Vec<usize>)> {
        (0..n_samples)
            .map(|i| {
                let test_indices = vec![i];
                let train_indices: Vec<usize> = (0..i).chain((i + 1)..n_samples).collect();
                (train_indices, test_indices)
            })
            .collect()
    }
}

/// Spatial cross-validator
pub struct SpatialCrossValidator {
    /// Number of folds
    n_folds: usize,

    /// Buffer distance for spatial separation
    buffer_distance: f64,
}

impl SpatialCrossValidator {
    /// Create a new spatial cross-validator
    pub fn new(n_folds: usize, buffer_distance: f64) -> Self {
        Self {
            n_folds,
            buffer_distance,
        }
    }

    /// Generate spatial cross-validation splits
    pub fn split(
        &self,
        coords: &Array2<f64>,
    ) -> Result<Vec<(Vec<usize>, Vec<usize>)>> {
        let n_samples = coords.nrows();

        // Simplified spatial split - divide space into grid cells
        // In production, use proper spatial clustering

        let mut splits = Vec::new();
        let fold_size = n_samples / self.n_folds;

        for fold in 0..self.n_folds {
            let test_start = fold * fold_size;
            let test_end = if fold == self.n_folds - 1 {
                n_samples
            } else {
                (fold + 1) * fold_size
            };

            let test_indices: Vec<usize> = (test_start..test_end).collect();
            let train_indices: Vec<usize> = (0..test_start)
                .chain(test_end..n_samples)
                .collect();

            splits.push((train_indices, test_indices));
        }

        Ok(splits)
    }
}

/// Spatial accuracy metrics
pub struct SpatialMetrics;

impl SpatialMetrics {
    /// Compute spatial autocorrelation of errors
    pub fn error_autocorrelation(
        residuals: &Array1<f64>,
        coords: &Array2<f64>,
    ) -> Result<f64> {
        if residuals.len() != coords.nrows() {
            return Err(MlError::InvalidInput(
                "Residuals and coordinates must have same length".to_string(),
            ));
        }

        // Use Moran's I on residuals
        use crate::features::spatial::morans_i;

        let coords_vec: Vec<(f64, f64)> = coords
            .rows()
            .into_iter()
            .map(|row| (row[0], row[1]))
            .collect();

        let residuals_vec: Vec<f64> = residuals.iter().cloned().collect();

        morans_i(&residuals_vec, &coords_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confusion_matrix() {
        let y_true = Array1::from_vec(vec![0, 1, 2, 0, 1, 2]);
        let y_pred = Array1::from_vec(vec![0, 2, 2, 0, 1, 1]);

        let cm = ConfusionMatrix::compute(&y_true, &y_pred, 3).unwrap();
        assert_eq!(cm.num_classes, 3);
        assert_eq!(cm.total(), 6);
        assert_eq!(cm.true_positives(0), 2);
    }

    #[test]
    fn test_regression_metrics() {
        let y_true = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let y_pred = Array1::from_vec(vec![1.1, 2.1, 2.9, 4.1, 4.9]);

        let metrics = RegressionMetrics::compute(&y_true, &y_pred).unwrap();
        assert!(metrics.rmse < 0.2);
        assert!(metrics.r2 > 0.95);
    }

    #[test]
    fn test_cross_validator() {
        let cv = CrossValidator::kfold(5);
        let splits = cv.split(100);

        assert_eq!(splits.len(), 5);
        for (train, test) in &splits {
            assert_eq!(train.len() + test.len(), 100);
        }
    }
}
