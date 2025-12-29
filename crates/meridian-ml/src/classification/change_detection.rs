//! Change detection algorithms

use crate::classification::{Classifier, ClassificationResult};
use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Types of change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// No change detected
    NoChange = 0,

    /// Minor change
    Minor = 1,

    /// Moderate change
    Moderate = 2,

    /// Major change
    Major = 3,

    /// Land cover transition
    Transition = 4,
}

impl ChangeType {
    /// Get change type name
    pub fn name(&self) -> &str {
        match self {
            Self::NoChange => "No Change",
            Self::Minor => "Minor Change",
            Self::Moderate => "Moderate Change",
            Self::Major => "Major Change",
            Self::Transition => "Transition",
        }
    }

    /// Convert from magnitude
    pub fn from_magnitude(magnitude: f64, thresholds: &[f64; 3]) -> Self {
        if magnitude < thresholds[0] {
            Self::NoChange
        } else if magnitude < thresholds[1] {
            Self::Minor
        } else if magnitude < thresholds[2] {
            Self::Moderate
        } else {
            Self::Major
        }
    }
}

/// Change detection method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeDetectionMethod {
    /// Image differencing
    Differencing,

    /// Change Vector Analysis
    CVA,

    /// Principal Component Analysis
    PCA,

    /// Post-classification comparison
    PostClassification,

    /// Deep learning based
    DeepLearning,
}

/// Change detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDetectionConfig {
    /// Detection method
    pub method: ChangeDetectionMethod,

    /// Change magnitude thresholds [minor, moderate, major]
    pub thresholds: [f64; 3],

    /// Minimum change area (pixels)
    pub min_change_area: usize,

    /// Apply morphological operations
    pub apply_morphology: bool,

    /// Confidence threshold
    pub confidence_threshold: f64,
}

impl Default for ChangeDetectionConfig {
    fn default() -> Self {
        Self {
            method: ChangeDetectionMethod::CVA,
            thresholds: [0.1, 0.3, 0.5],
            min_change_area: 9,
            apply_morphology: true,
            confidence_threshold: 0.7,
        }
    }
}

/// Change detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeDetectionResult {
    /// Change type per pixel
    pub change_type: Vec<ChangeType>,

    /// Change magnitude
    pub magnitude: Vec<f64>,

    /// Confidence scores
    pub confidence: Vec<f64>,

    /// Change direction (for CVA)
    pub direction: Option<Vec<f64>>,
}

/// Change detector
pub struct ChangeDetector {
    /// Configuration
    config: ChangeDetectionConfig,

    /// Reference classifier (for post-classification)
    classifier: Option<Box<dyn Classifier>>,
}

impl ChangeDetector {
    /// Create a new change detector
    pub fn new() -> Self {
        Self {
            config: ChangeDetectionConfig::default(),
            classifier: None,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ChangeDetectionConfig) -> Self {
        Self {
            config,
            classifier: None,
        }
    }

    /// Set change detection method
    pub fn with_method(mut self, method: ChangeDetectionMethod) -> Self {
        self.config.method = method;
        self
    }

    /// Set thresholds
    pub fn with_thresholds(mut self, thresholds: [f64; 3]) -> Self {
        self.config.thresholds = thresholds;
        self
    }

    /// Set classifier for post-classification method
    pub fn with_classifier(mut self, classifier: Box<dyn Classifier>) -> Self {
        self.classifier = Some(classifier);
        self
    }

    /// Detect changes between two time periods
    pub fn detect(
        &self,
        before: &FeatureSet,
        after: &FeatureSet,
    ) -> Result<ChangeDetectionResult> {
        if before.num_samples() != after.num_samples() {
            return Err(MlError::InvalidInput(
                "Before and after must have same number of samples".to_string(),
            ));
        }

        if before.num_features() != after.num_features() {
            return Err(MlError::InvalidInput(
                "Before and after must have same number of features".to_string(),
            ));
        }

        match self.config.method {
            ChangeDetectionMethod::Differencing => self.detect_by_differencing(before, after),
            ChangeDetectionMethod::CVA => self.detect_by_cva(before, after),
            ChangeDetectionMethod::PCA => self.detect_by_pca(before, after),
            ChangeDetectionMethod::PostClassification => {
                self.detect_by_post_classification(before, after)
            }
            ChangeDetectionMethod::DeepLearning => Err(MlError::Model(
                "Deep learning method not yet implemented".to_string(),
            )),
        }
    }

    /// Image differencing method
    fn detect_by_differencing(
        &self,
        before: &FeatureSet,
        after: &FeatureSet,
    ) -> Result<ChangeDetectionResult> {
        let diff = &after.features - &before.features;

        // Calculate magnitude as Euclidean distance
        let magnitude: Vec<f64> = diff
            .rows()
            .into_iter()
            .map(|row| row.iter().map(|&x| x * x).sum::<f64>().sqrt())
            .collect();

        let change_type: Vec<ChangeType> = magnitude
            .iter()
            .map(|&mag| ChangeType::from_magnitude(mag, &self.config.thresholds))
            .collect();

        let confidence = magnitude
            .iter()
            .map(|&mag| (mag / self.config.thresholds[2]).min(1.0))
            .collect();

        Ok(ChangeDetectionResult {
            change_type,
            magnitude,
            confidence,
            direction: None,
        })
    }

    /// Change Vector Analysis (CVA)
    fn detect_by_cva(
        &self,
        before: &FeatureSet,
        after: &FeatureSet,
    ) -> Result<ChangeDetectionResult> {
        let diff = &after.features - &before.features;

        let mut magnitude = Vec::new();
        let mut direction = Vec::new();

        for row in diff.rows() {
            // Magnitude
            let mag = row.iter().map(|&x| x * x).sum::<f64>().sqrt();
            magnitude.push(mag);

            // Direction (angle in feature space)
            // For 2D case: arctan(y/x), for higher dimensions use first two components
            let dir = if row.len() >= 2 {
                row[1].atan2(row[0])
            } else {
                0.0
            };
            direction.push(dir);
        }

        let change_type: Vec<ChangeType> = magnitude
            .iter()
            .map(|&mag| ChangeType::from_magnitude(mag, &self.config.thresholds))
            .collect();

        let confidence = magnitude
            .iter()
            .map(|&mag| (mag / self.config.thresholds[2]).min(1.0))
            .collect();

        Ok(ChangeDetectionResult {
            change_type,
            magnitude,
            confidence,
            direction: Some(direction),
        })
    }

    /// PCA-based change detection
    fn detect_by_pca(
        &self,
        before: &FeatureSet,
        after: &FeatureSet,
    ) -> Result<ChangeDetectionResult> {
        // Simplified PCA-based change detection
        // A full implementation would compute PCA on the difference image
        // and use the first principal component for change detection

        let diff = &after.features - &before.features;

        // For now, use simple magnitude calculation
        let magnitude: Vec<f64> = diff
            .rows()
            .into_iter()
            .map(|row| row.iter().map(|&x| x * x).sum::<f64>().sqrt())
            .collect();

        let change_type: Vec<ChangeType> = magnitude
            .iter()
            .map(|&mag| ChangeType::from_magnitude(mag, &self.config.thresholds))
            .collect();

        let confidence = magnitude
            .iter()
            .map(|&mag| (mag / self.config.thresholds[2]).min(1.0))
            .collect();

        Ok(ChangeDetectionResult {
            change_type,
            magnitude,
            confidence,
            direction: None,
        })
    }

    /// Post-classification comparison
    fn detect_by_post_classification(
        &self,
        before: &FeatureSet,
        after: &FeatureSet,
    ) -> Result<ChangeDetectionResult> {
        let classifier = self
            .classifier
            .as_ref()
            .ok_or_else(|| MlError::Model("Classifier not set".to_string()))?;

        if !classifier.is_trained() {
            return Err(MlError::Model("Classifier not trained".to_string()));
        }

        // Classify both time periods
        let before_classes = classifier.predict(before)?;
        let after_classes = classifier.predict(after)?;

        // Compare classifications
        let mut change_type = Vec::new();
        let mut magnitude = Vec::new();
        let mut confidence = Vec::new();

        for i in 0..before_classes.len() {
            if before_classes[i] != after_classes[i] {
                change_type.push(ChangeType::Transition);
                magnitude.push(1.0);
                confidence.push(1.0);
            } else {
                change_type.push(ChangeType::NoChange);
                magnitude.push(0.0);
                confidence.push(1.0);
            }
        }

        Ok(ChangeDetectionResult {
            change_type,
            magnitude,
            confidence,
            direction: None,
        })
    }

    /// Post-process change detection results
    pub fn post_process(&self, result: ChangeDetectionResult, width: usize, height: usize) -> Result<ChangeDetectionResult> {
        if result.change_type.len() != width * height {
            return Err(MlError::InvalidInput(
                "Result size doesn't match dimensions".to_string(),
            ));
        }

        let mut processed = result.clone();

        // Filter by minimum change area
        if self.config.min_change_area > 1 {
            processed = self.filter_small_changes(processed, width, height);
        }

        // Apply morphological operations
        if self.config.apply_morphology {
            processed = self.apply_morphology(processed, width, height);
        }

        Ok(processed)
    }

    /// Filter small change regions
    fn filter_small_changes(&self, result: ChangeDetectionResult, width: usize, height: usize) -> ChangeDetectionResult {
        // Simplified implementation - a full version would use connected components
        result
    }

    /// Apply morphological operations
    fn apply_morphology(&self, result: ChangeDetectionResult, width: usize, height: usize) -> ChangeDetectionResult {
        // Simplified implementation - would apply erosion/dilation
        result
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_change_type() {
        assert_eq!(ChangeType::NoChange.name(), "No Change");
        assert_eq!(ChangeType::from_magnitude(0.05, &[0.1, 0.3, 0.5]), ChangeType::NoChange);
        assert_eq!(ChangeType::from_magnitude(0.2, &[0.1, 0.3, 0.5]), ChangeType::Minor);
    }

    #[test]
    fn test_change_detector() {
        let detector = ChangeDetector::new();
        assert_eq!(detector.config.method, ChangeDetectionMethod::CVA);
    }
}
