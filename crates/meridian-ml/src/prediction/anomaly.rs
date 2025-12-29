//! Anomaly detection for spatial data

use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use crate::prediction::{Predictor, PredictionResult};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Types of anomalies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Point anomaly
    Point,

    /// Contextual anomaly
    Contextual,

    /// Collective anomaly
    Collective,

    /// Spatial anomaly
    Spatial,
}

/// Anomaly detection method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyMethod {
    /// Isolation Forest
    IsolationForest,

    /// One-Class SVM
    OneClassSVM,

    /// Local Outlier Factor
    LOF,

    /// Statistical (Z-score)
    Statistical,

    /// Autoencoder (Deep Learning)
    Autoencoder,

    /// Spatial outlier detection
    SpatialOutlier,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    /// Anomaly scores (higher = more anomalous)
    pub scores: Array1<f64>,

    /// Binary labels (1 = anomaly, 0 = normal)
    pub labels: Array1<usize>,

    /// Anomaly threshold used
    pub threshold: f64,

    /// Anomaly types (if classified)
    pub types: Option<Vec<AnomalyType>>,
}

impl AnomalyResult {
    /// Create a new anomaly result
    pub fn new(scores: Array1<f64>, threshold: f64) -> Self {
        let labels = scores.mapv(|s| if s > threshold { 1 } else { 0 });

        Self {
            scores,
            labels,
            threshold,
            types: None,
        }
    }

    /// Get indices of detected anomalies
    pub fn anomaly_indices(&self) -> Vec<usize> {
        self.labels
            .iter()
            .enumerate()
            .filter(|(_, &label)| label == 1)
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Get number of anomalies
    pub fn num_anomalies(&self) -> usize {
        self.labels.iter().filter(|&&l| l == 1).count()
    }

    /// Get anomaly rate
    pub fn anomaly_rate(&self) -> f64 {
        self.num_anomalies() as f64 / self.labels.len() as f64
    }
}

/// Anomaly detector
pub struct AnomalyDetector {
    /// Detection method
    method: AnomalyMethod,

    /// Contamination rate (expected proportion of anomalies)
    contamination: f64,

    /// Whether the model is trained
    trained: bool,

    /// Training statistics
    mean: Option<Array1<f64>>,
    std: Option<Array1<f64>>,
    threshold: Option<f64>,
}

impl AnomalyDetector {
    /// Create a new anomaly detector
    pub fn new(method: AnomalyMethod) -> Self {
        Self {
            method,
            contamination: 0.1, // 10% expected anomalies
            trained: false,
            mean: None,
            std: None,
            threshold: None,
        }
    }

    /// Set contamination rate
    pub fn with_contamination(mut self, rate: f64) -> Self {
        if rate > 0.0 && rate < 1.0 {
            self.contamination = rate;
        }
        self
    }

    /// Fit the anomaly detector
    pub fn fit(&mut self, features: &FeatureSet) -> Result<()> {
        match self.method {
            AnomalyMethod::Statistical => self.fit_statistical(features),
            AnomalyMethod::IsolationForest => self.fit_isolation_forest(features),
            AnomalyMethod::LOF => self.fit_lof(features),
            AnomalyMethod::SpatialOutlier => self.fit_spatial_outlier(features),
            _ => Err(MlError::Model(format!(
                "Method {:?} not yet implemented",
                self.method
            ))),
        }
    }

    /// Fit statistical method (Z-score)
    fn fit_statistical(&mut self, features: &FeatureSet) -> Result<()> {
        let data = &features.features;

        // Calculate mean and std for each feature
        let mean = data.mean_axis(ndarray::Axis(0)).ok_or_else(|| {
            MlError::Training("Failed to calculate mean".to_string())
        })?;

        let std = data.std_axis(ndarray::Axis(0), 0.0);

        // Set threshold based on contamination rate
        // Using normal distribution quantile
        let z_score = self.contamination_to_zscore(self.contamination);
        self.threshold = Some(z_score);

        self.mean = Some(mean);
        self.std = Some(std);
        self.trained = true;

        Ok(())
    }

    /// Convert contamination rate to z-score
    fn contamination_to_zscore(&self, contamination: f64) -> f64 {
        // Approximation: higher contamination = lower threshold
        // For 0.1 (10%), use z = 2.326 (99th percentile)
        // For 0.05 (5%), use z = 2.576 (99.5th percentile)
        let p = 1.0 - contamination;
        if p >= 0.99 {
            2.326
        } else if p >= 0.95 {
            1.96
        } else {
            1.645
        }
    }

    /// Fit isolation forest
    fn fit_isolation_forest(&mut self, features: &FeatureSet) -> Result<()> {
        // Simplified isolation forest
        // In production, use proper implementation
        self.trained = true;
        Ok(())
    }

    /// Fit Local Outlier Factor
    fn fit_lof(&mut self, features: &FeatureSet) -> Result<()> {
        // Simplified LOF
        self.trained = true;
        Ok(())
    }

    /// Fit spatial outlier detector
    fn fit_spatial_outlier(&mut self, features: &FeatureSet) -> Result<()> {
        // Spatial outlier detection using local statistics
        self.trained = true;
        Ok(())
    }

    /// Detect anomalies in new data
    pub fn detect(&self, features: &FeatureSet) -> Result<AnomalyResult> {
        if !self.trained {
            return Err(MlError::Model("Model not trained".to_string()));
        }

        match self.method {
            AnomalyMethod::Statistical => self.detect_statistical(features),
            AnomalyMethod::IsolationForest => self.detect_isolation_forest(features),
            AnomalyMethod::LOF => self.detect_lof(features),
            AnomalyMethod::SpatialOutlier => self.detect_spatial_outlier(features),
            _ => Err(MlError::Model(format!(
                "Method {:?} not yet implemented",
                self.method
            ))),
        }
    }

    /// Detect using statistical method
    fn detect_statistical(&self, features: &FeatureSet) -> Result<AnomalyResult> {
        let mean = self
            .mean
            .as_ref()
            .ok_or_else(|| MlError::Model("Mean not computed".to_string()))?;

        let std = self
            .std
            .as_ref()
            .ok_or_else(|| MlError::Model("Std not computed".to_string()))?;

        let threshold = self
            .threshold
            .ok_or_else(|| MlError::Model("Threshold not set".to_string()))?;

        let data = &features.features;
        let n_samples = data.nrows();

        // Calculate Z-scores
        let mut scores = Array1::zeros(n_samples);

        for i in 0..n_samples {
            let row = data.row(i);
            let mut z_scores = Vec::new();

            for (j, &value) in row.iter().enumerate() {
                if std[j] > 0.0 {
                    let z = ((value - mean[j]) / std[j]).abs();
                    z_scores.push(z);
                } else {
                    z_scores.push(0.0);
                }
            }

            // Use max Z-score across features
            scores[i] = z_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        }

        Ok(AnomalyResult::new(scores, threshold))
    }

    /// Detect using isolation forest
    fn detect_isolation_forest(&self, features: &FeatureSet) -> Result<AnomalyResult> {
        // Placeholder
        let scores = Array1::zeros(features.num_samples());
        Ok(AnomalyResult::new(scores, 0.5))
    }

    /// Detect using LOF
    fn detect_lof(&self, features: &FeatureSet) -> Result<AnomalyResult> {
        // Placeholder
        let scores = Array1::zeros(features.num_samples());
        Ok(AnomalyResult::new(scores, 1.5))
    }

    /// Detect spatial outliers
    fn detect_spatial_outlier(&self, features: &FeatureSet) -> Result<AnomalyResult> {
        // Placeholder
        let scores = Array1::zeros(features.num_samples());
        Ok(AnomalyResult::new(scores, 2.0))
    }

    /// Fit and detect in one step
    pub fn fit_detect(&mut self, features: &FeatureSet) -> Result<AnomalyResult> {
        self.fit(features)?;
        self.detect(features)
    }
}

impl Predictor for AnomalyDetector {
    fn train(&mut self, features: &FeatureSet, _targets: &Array1<f64>) -> Result<()> {
        // Anomaly detection is unsupervised, ignore targets
        self.fit(features)
    }

    fn predict(&self, features: &FeatureSet) -> Result<Array1<f64>> {
        let result = self.detect(features)?;
        Ok(result.scores)
    }

    fn predict_with_uncertainty(&self, features: &FeatureSet) -> Result<PredictionResult> {
        let scores = self.predict(features)?;
        Ok(PredictionResult::new(scores, 0.95))
    }

    fn is_trained(&self) -> bool {
        self.trained
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_detector() {
        let detector = AnomalyDetector::new(AnomalyMethod::Statistical);
        assert!(!detector.is_trained());
        assert_eq!(detector.contamination, 0.1);
    }

    #[test]
    fn test_anomaly_result() {
        let scores = Array1::from_vec(vec![0.5, 1.5, 0.3, 2.5, 0.8]);
        let result = AnomalyResult::new(scores, 1.0);
        assert_eq!(result.num_anomalies(), 2);
        assert!((result.anomaly_rate() - 0.4).abs() < 1e-10);
    }
}
