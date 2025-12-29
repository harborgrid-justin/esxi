//! Classification algorithms for spatial data

pub mod land_cover;
pub mod change_detection;

pub use land_cover::{LandCoverClassifier, LandCoverClass};
pub use change_detection::{ChangeDetector, ChangeType};

use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use ndarray::Array1;
use serde::{Deserialize, Serialize};

/// Base trait for classifiers
pub trait Classifier: Send + Sync {
    /// Train the classifier
    fn train(&mut self, features: &FeatureSet, labels: &Array1<usize>) -> Result<()>;

    /// Predict class labels
    fn predict(&self, features: &FeatureSet) -> Result<Array1<usize>>;

    /// Predict class probabilities
    fn predict_proba(&self, features: &FeatureSet) -> Result<Vec<Vec<f64>>>;

    /// Get the number of classes
    fn num_classes(&self) -> usize;

    /// Check if the classifier is trained
    fn is_trained(&self) -> bool;
}

/// Classification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationConfig {
    /// Number of classes
    pub num_classes: usize,

    /// Class weights for imbalanced datasets
    pub class_weights: Option<Vec<f64>>,

    /// Whether to use spatial cross-validation
    pub spatial_cv: bool,

    /// Number of cross-validation folds
    pub cv_folds: usize,

    /// Random seed for reproducibility
    pub random_seed: Option<u64>,
}

impl Default for ClassificationConfig {
    fn default() -> Self {
        Self {
            num_classes: 2,
            class_weights: None,
            spatial_cv: true,
            cv_folds: 5,
            random_seed: Some(42),
        }
    }
}

/// Classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// Predicted class labels
    pub labels: Array1<usize>,

    /// Class probabilities
    pub probabilities: Vec<Vec<f64>>,

    /// Confidence scores
    pub confidence: Array1<f64>,
}

impl ClassificationResult {
    /// Create a new classification result
    pub fn new(labels: Array1<usize>, probabilities: Vec<Vec<f64>>) -> Self {
        // Calculate confidence as max probability
        let confidence = Array1::from_vec(
            probabilities
                .iter()
                .map(|probs| probs.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
                .collect(),
        );

        Self {
            labels,
            probabilities,
            confidence,
        }
    }

    /// Get the number of predictions
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// Check if result is empty
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }

    /// Get predictions with confidence above threshold
    pub fn filter_by_confidence(&self, threshold: f64) -> Vec<usize> {
        self.confidence
            .iter()
            .enumerate()
            .filter(|(_, &conf)| conf >= threshold)
            .map(|(idx, _)| idx)
            .collect()
    }
}

/// Random forest classifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomForestClassifier {
    /// Number of trees
    n_estimators: usize,

    /// Maximum depth
    max_depth: Option<usize>,

    /// Minimum samples to split
    min_samples_split: usize,

    /// Number of features to consider
    max_features: Option<usize>,

    /// Whether the model is trained
    trained: bool,

    /// Number of classes
    num_classes: usize,
}

impl RandomForestClassifier {
    /// Create a new random forest classifier
    pub fn new(n_estimators: usize) -> Self {
        Self {
            n_estimators,
            max_depth: None,
            min_samples_split: 2,
            max_features: None,
            trained: false,
            num_classes: 0,
        }
    }

    /// Set maximum depth
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// Set minimum samples to split
    pub fn with_min_samples_split(mut self, min_samples: usize) -> Self {
        self.min_samples_split = min_samples;
        self
    }

    /// Set maximum features to consider
    pub fn with_max_features(mut self, max_features: usize) -> Self {
        self.max_features = Some(max_features);
        self
    }
}

impl Classifier for RandomForestClassifier {
    fn train(&mut self, features: &FeatureSet, labels: &Array1<usize>) -> Result<()> {
        if features.num_samples() != labels.len() {
            return Err(MlError::InvalidInput(
                "Number of features and labels must match".to_string(),
            ));
        }

        // Determine number of classes
        self.num_classes = labels.iter().max().map(|&x| x + 1).unwrap_or(0);

        if self.num_classes < 2 {
            return Err(MlError::InvalidInput(
                "At least 2 classes required".to_string(),
            ));
        }

        // Training logic would go here
        // For now, just mark as trained
        self.trained = true;

        Ok(())
    }

    fn predict(&self, features: &FeatureSet) -> Result<Array1<usize>> {
        if !self.trained {
            return Err(MlError::Model("Model not trained".to_string()));
        }

        // Placeholder: return zeros
        Ok(Array1::zeros(features.num_samples()))
    }

    fn predict_proba(&self, features: &FeatureSet) -> Result<Vec<Vec<f64>>> {
        if !self.trained {
            return Err(MlError::Model("Model not trained".to_string()));
        }

        // Placeholder: return uniform probabilities
        let n_samples = features.num_samples();
        let prob = 1.0 / self.num_classes as f64;
        Ok(vec![vec![prob; self.num_classes]; n_samples])
    }

    fn num_classes(&self) -> usize {
        self.num_classes
    }

    fn is_trained(&self) -> bool {
        self.trained
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_random_forest_creation() {
        let rf = RandomForestClassifier::new(100);
        assert_eq!(rf.n_estimators, 100);
        assert!(!rf.is_trained());
    }

    #[test]
    fn test_classification_result() {
        let labels = Array1::from_vec(vec![0, 1, 0, 1]);
        let probabilities = vec![
            vec![0.8, 0.2],
            vec![0.3, 0.7],
            vec![0.9, 0.1],
            vec![0.2, 0.8],
        ];

        let result = ClassificationResult::new(labels, probabilities);
        assert_eq!(result.len(), 4);
        assert!(result.confidence[0] >= 0.8);
    }
}
