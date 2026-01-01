//! Data and model drift detection
//!
//! Detects distribution shifts in input data and model predictions

use super::Alert;
use crate::{Error, Result};
use ndarray::{Array1, ArrayD, Axis};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Type of drift
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftType {
    /// Input data distribution drift
    DataDrift,

    /// Model prediction distribution drift
    ConceptDrift,

    /// Feature-specific drift
    FeatureDrift,

    /// Label drift (for supervised learning)
    LabelDrift,
}

/// Drift detection method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriftMethod {
    /// Kolmogorov-Smirnov test
    KolmogorovSmirnov,

    /// Population Stability Index
    PSI,

    /// Wasserstein distance
    Wasserstein,

    /// Jensen-Shannon divergence
    JensenShannon,

    /// Chi-squared test
    ChiSquared,
}

/// Drift detection report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftReport {
    /// Report timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Drift type
    pub drift_type: DriftType,

    /// Detection method
    pub method: DriftMethod,

    /// Whether drift was detected
    pub drift_detected: bool,

    /// Drift score (0.0 to 1.0, higher = more drift)
    pub drift_score: f64,

    /// Statistical p-value (if applicable)
    pub p_value: Option<f64>,

    /// Threshold used for detection
    pub threshold: f64,

    /// Per-feature drift scores
    pub feature_scores: Vec<f64>,

    /// Alerts generated
    pub alerts: Vec<Alert>,
}

impl DriftReport {
    /// Create a new drift report
    pub fn new(drift_type: DriftType, method: DriftMethod) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            drift_type,
            method,
            drift_detected: false,
            drift_score: 0.0,
            p_value: None,
            threshold: 0.05,
            feature_scores: Vec::new(),
            alerts: Vec::new(),
        }
    }

    /// Add an alert
    pub fn add_alert(&mut self, alert: Alert) {
        self.alerts.push(alert);
    }
}

/// Configuration for drift detection
#[derive(Debug, Clone)]
pub struct DriftConfig {
    /// Detection method
    pub method: DriftMethod,

    /// Drift threshold (0.0 to 1.0)
    pub threshold: f64,

    /// Window size for reference data
    pub reference_window_size: usize,

    /// Window size for current data
    pub current_window_size: usize,

    /// Minimum samples required
    pub min_samples: usize,

    /// Enable per-feature drift detection
    pub detect_feature_drift: bool,

    /// Enable automatic alerting
    pub enable_alerting: bool,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            method: DriftMethod::PSI,
            threshold: 0.1,
            reference_window_size: 1000,
            current_window_size: 100,
            min_samples: 30,
            detect_feature_drift: true,
            enable_alerting: true,
        }
    }
}

/// Drift detector
pub struct DriftDetector {
    /// Configuration
    config: DriftConfig,

    /// Reference data window (baseline)
    reference_data: VecDeque<ArrayD<f32>>,

    /// Current data window
    current_data: VecDeque<ArrayD<f32>>,

    /// Number of drift detections
    drift_count: usize,

    /// Last drift report
    last_report: Option<DriftReport>,
}

impl DriftDetector {
    /// Create a new drift detector
    pub fn new(config: DriftConfig) -> Self {
        Self {
            config,
            reference_data: VecDeque::new(),
            current_data: VecDeque::new(),
            drift_count: 0,
            last_report: None,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(DriftConfig::default())
    }

    /// Set reference data (baseline)
    pub fn set_reference(&mut self, data: Vec<ArrayD<f32>>) {
        self.reference_data.clear();
        for sample in data {
            self.add_reference_sample(sample);
        }
    }

    /// Add a reference sample
    pub fn add_reference_sample(&mut self, sample: ArrayD<f32>) {
        if self.reference_data.len() >= self.config.reference_window_size {
            self.reference_data.pop_front();
        }
        self.reference_data.push_back(sample);
    }

    /// Add a current sample and check for drift
    pub fn add_sample(&mut self, sample: ArrayD<f32>) -> Result<Option<DriftReport>> {
        if self.current_data.len() >= self.config.current_window_size {
            self.current_data.pop_front();
        }
        self.current_data.push_back(sample);

        // Check if we have enough samples
        if self.reference_data.len() < self.config.min_samples
            || self.current_data.len() < self.config.min_samples
        {
            return Ok(None);
        }

        // Perform drift detection
        self.detect_drift()
    }

    /// Detect drift between reference and current data
    pub fn detect_drift(&mut self) -> Result<Option<DriftReport>> {
        if self.reference_data.is_empty() || self.current_data.is_empty() {
            return Ok(None);
        }

        let mut report = DriftReport::new(DriftType::DataDrift, self.config.method);
        report.threshold = self.config.threshold;

        // Calculate drift score based on method
        let drift_score = match self.config.method {
            DriftMethod::PSI => self.calculate_psi()?,
            DriftMethod::KolmogorovSmirnov => self.calculate_ks()?,
            DriftMethod::Wasserstein => self.calculate_wasserstein()?,
            DriftMethod::JensenShannon => self.calculate_js()?,
            DriftMethod::ChiSquared => self.calculate_chi_squared()?,
        };

        report.drift_score = drift_score;
        report.drift_detected = drift_score > self.config.threshold;

        // Detect per-feature drift if enabled
        if self.config.detect_feature_drift {
            report.feature_scores = self.calculate_feature_drift()?;
        }

        // Generate alerts if enabled
        if self.config.enable_alerting && report.drift_detected {
            self.drift_count += 1;

            let alert = if drift_score > self.config.threshold * 2.0 {
                Alert::critical(
                    "Severe Data Drift Detected",
                    format!("Drift score: {:.4} (threshold: {:.4})", drift_score, self.config.threshold),
                )
            } else {
                Alert::warning(
                    "Data Drift Detected",
                    format!("Drift score: {:.4} (threshold: {:.4})", drift_score, self.config.threshold),
                )
            };

            report.add_alert(alert);
        }

        self.last_report = Some(report.clone());

        Ok(Some(report))
    }

    /// Calculate Population Stability Index (PSI)
    fn calculate_psi(&self) -> Result<f64> {
        // Simplified PSI calculation
        // In production, would calculate proper distribution bins

        let ref_mean = self.calculate_mean(&self.reference_data)?;
        let curr_mean = self.calculate_mean(&self.current_data)?;

        let ref_std = self.calculate_std(&self.reference_data, &ref_mean)?;
        let curr_std = self.calculate_std(&self.current_data, &curr_mean)?;

        // Simple approximation: normalize difference in means and stds
        let mean_diff = (&curr_mean - &ref_mean).mapv(|x| x.abs()).mean().unwrap_or(0.0) as f64;
        let std_diff = (&curr_std - &ref_std).mapv(|x| x.abs()).mean().unwrap_or(0.0) as f64;

        Ok((mean_diff + std_diff) / 2.0)
    }

    /// Calculate Kolmogorov-Smirnov statistic
    fn calculate_ks(&self) -> Result<f64> {
        // Simplified KS test
        // In production, would use proper two-sample KS test

        let ref_mean = self.calculate_mean(&self.reference_data)?;
        let curr_mean = self.calculate_mean(&self.current_data)?;

        let diff = (&curr_mean - &ref_mean).mapv(|x| x.abs()).mean().unwrap_or(0.0) as f64;

        Ok(diff)
    }

    /// Calculate Wasserstein distance
    fn calculate_wasserstein(&self) -> Result<f64> {
        // Simplified Wasserstein distance
        let ref_mean = self.calculate_mean(&self.reference_data)?;
        let curr_mean = self.calculate_mean(&self.current_data)?;

        let distance = (&curr_mean - &ref_mean)
            .mapv(|x| x * x)
            .sum()
            .sqrt() as f64;

        Ok(distance)
    }

    /// Calculate Jensen-Shannon divergence
    fn calculate_js(&self) -> Result<f64> {
        // Simplified JS divergence
        let ref_mean = self.calculate_mean(&self.reference_data)?;
        let curr_mean = self.calculate_mean(&self.current_data)?;

        let kl_div = self.kl_divergence(&ref_mean, &curr_mean);

        Ok(kl_div / 2.0)
    }

    /// Calculate Chi-squared statistic
    fn calculate_chi_squared(&self) -> Result<f64> {
        // Simplified chi-squared test
        let ref_mean = self.calculate_mean(&self.reference_data)?;
        let curr_mean = self.calculate_mean(&self.current_data)?;

        let chi_sq = (&curr_mean - &ref_mean)
            .mapv(|x| x * x)
            .mean()
            .unwrap_or(0.0) as f64;

        Ok(chi_sq)
    }

    /// Calculate per-feature drift scores
    fn calculate_feature_drift(&self) -> Result<Vec<f64>> {
        let ref_mean = self.calculate_mean(&self.reference_data)?;
        let curr_mean = self.calculate_mean(&self.current_data)?;

        let scores: Vec<f64> = (&curr_mean - &ref_mean)
            .mapv(|x| x.abs() as f64)
            .iter()
            .copied()
            .collect();

        Ok(scores)
    }

    /// Calculate mean across samples
    fn calculate_mean(&self, data: &VecDeque<ArrayD<f32>>) -> Result<Array1<f32>> {
        if data.is_empty() {
            return Err(Error::drift("No data available"));
        }

        let n_samples = data.len();
        let n_features = data[0].len_of(Axis(0));

        let mut sum = Array1::zeros(n_features);

        for sample in data {
            for (i, &val) in sample.iter().enumerate() {
                sum[i % n_features] += val;
            }
        }

        sum /= n_samples as f32;
        Ok(sum)
    }

    /// Calculate standard deviation
    fn calculate_std(&self, data: &VecDeque<ArrayD<f32>>, mean: &Array1<f32>) -> Result<Array1<f32>> {
        if data.is_empty() {
            return Err(Error::drift("No data available"));
        }

        let n_samples = data.len();
        let n_features = mean.len();

        let mut variance = Array1::zeros(n_features);

        for sample in data {
            for (i, &val) in sample.iter().enumerate() {
                let idx = i % n_features;
                let diff = val - mean[idx];
                variance[idx] += diff * diff;
            }
        }

        variance /= n_samples as f32;
        Ok(variance.mapv(|x| x.sqrt()))
    }

    /// Calculate KL divergence
    fn kl_divergence(&self, p: &Array1<f32>, q: &Array1<f32>) -> f64 {
        let epsilon = 1e-10;
        p.iter()
            .zip(q.iter())
            .map(|(&p_val, &q_val)| {
                let p_safe = (p_val as f64).max(epsilon);
                let q_safe = (q_val as f64).max(epsilon);
                p_safe * (p_safe / q_safe).ln()
            })
            .sum()
    }

    /// Get last drift report
    pub fn last_report(&self) -> Option<&DriftReport> {
        self.last_report.as_ref()
    }

    /// Get drift count
    pub fn drift_count(&self) -> usize {
        self.drift_count
    }

    /// Reset detector state
    pub fn reset(&mut self) {
        self.current_data.clear();
        self.drift_count = 0;
        self.last_report = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_config_default() {
        let config = DriftConfig::default();
        assert_eq!(config.method, DriftMethod::PSI);
        assert_eq!(config.threshold, 0.1);
        assert!(config.detect_feature_drift);
    }

    #[test]
    fn test_drift_detector_creation() {
        let detector = DriftDetector::with_defaults();
        assert_eq!(detector.drift_count(), 0);
        assert!(detector.last_report().is_none());
    }

    #[test]
    fn test_drift_report_creation() {
        let report = DriftReport::new(DriftType::DataDrift, DriftMethod::PSI);
        assert_eq!(report.drift_type, DriftType::DataDrift);
        assert_eq!(report.method, DriftMethod::PSI);
        assert!(!report.drift_detected);
    }

    #[test]
    fn test_drift_type_variants() {
        assert_ne!(DriftType::DataDrift, DriftType::ConceptDrift);
        assert_ne!(DriftType::FeatureDrift, DriftType::LabelDrift);
    }
}
