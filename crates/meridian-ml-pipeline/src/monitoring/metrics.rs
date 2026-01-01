//! Model performance metrics collection
//!
//! Provides comprehensive metrics tracking for ML models in production

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Classification metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationMetrics {
    /// Accuracy
    pub accuracy: f64,

    /// Precision
    pub precision: f64,

    /// Recall
    pub recall: f64,

    /// F1 score
    pub f1_score: f64,

    /// Area under ROC curve
    pub auc_roc: Option<f64>,

    /// Confusion matrix
    pub confusion_matrix: Vec<Vec<usize>>,

    /// Per-class metrics
    pub per_class: HashMap<String, ClassMetrics>,
}

impl Default for ClassificationMetrics {
    fn default() -> Self {
        Self {
            accuracy: 0.0,
            precision: 0.0,
            recall: 0.0,
            f1_score: 0.0,
            auc_roc: None,
            confusion_matrix: Vec::new(),
            per_class: HashMap::new(),
        }
    }
}

/// Per-class metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassMetrics {
    /// Class precision
    pub precision: f64,

    /// Class recall
    pub recall: f64,

    /// Class F1 score
    pub f1_score: f64,

    /// Support (number of samples)
    pub support: usize,
}

/// Regression metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionMetrics {
    /// Mean Absolute Error
    pub mae: f64,

    /// Mean Squared Error
    pub mse: f64,

    /// Root Mean Squared Error
    pub rmse: f64,

    /// R-squared score
    pub r2_score: f64,

    /// Mean Absolute Percentage Error
    pub mape: f64,

    /// Median Absolute Error
    pub median_ae: f64,
}

impl Default for RegressionMetrics {
    fn default() -> Self {
        Self {
            mae: 0.0,
            mse: 0.0,
            rmse: 0.0,
            r2_score: 0.0,
            mape: 0.0,
            median_ae: 0.0,
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average inference time in milliseconds
    pub avg_inference_time_ms: f64,

    /// P50 latency in milliseconds
    pub p50_latency_ms: f64,

    /// P95 latency in milliseconds
    pub p95_latency_ms: f64,

    /// P99 latency in milliseconds
    pub p99_latency_ms: f64,

    /// Throughput (requests per second)
    pub throughput_rps: f64,

    /// Memory usage in MB
    pub memory_mb: f64,

    /// CPU utilization (0.0 to 1.0)
    pub cpu_utilization: f64,

    /// GPU utilization (0.0 to 1.0)
    pub gpu_utilization: Option<f64>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_inference_time_ms: 0.0,
            p50_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            throughput_rps: 0.0,
            memory_mb: 0.0,
            cpu_utilization: 0.0,
            gpu_utilization: None,
        }
    }
}

/// Model metrics combining all metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Model identifier
    pub model_id: uuid::Uuid,

    /// Model name
    pub model_name: String,

    /// Model version
    pub model_version: String,

    /// Classification metrics (if applicable)
    pub classification: Option<ClassificationMetrics>,

    /// Regression metrics (if applicable)
    pub regression: Option<RegressionMetrics>,

    /// Performance metrics
    pub performance: PerformanceMetrics,

    /// Custom metrics
    pub custom: HashMap<String, f64>,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ModelMetrics {
    /// Create new model metrics
    pub fn new(model_id: uuid::Uuid, model_name: String, model_version: String) -> Self {
        Self {
            model_id,
            model_name,
            model_version,
            classification: None,
            regression: None,
            performance: PerformanceMetrics::default(),
            custom: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add a custom metric
    pub fn add_custom(&mut self, name: String, value: f64) {
        self.custom.insert(name, value);
    }

    /// Set classification metrics
    pub fn with_classification(mut self, metrics: ClassificationMetrics) -> Self {
        self.classification = Some(metrics);
        self
    }

    /// Set regression metrics
    pub fn with_regression(mut self, metrics: RegressionMetrics) -> Self {
        self.regression = Some(metrics);
        self
    }
}

/// Metrics collector
pub struct MetricsCollector {
    /// Stored metrics by model ID
    metrics: Arc<RwLock<HashMap<uuid::Uuid, Vec<ModelMetrics>>>>,

    /// Maximum metrics to store per model
    max_metrics_per_model: usize,

    /// Latency samples for percentile calculation
    latency_samples: Arc<RwLock<Vec<f64>>>,

    /// Maximum latency samples to keep
    max_latency_samples: usize,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(max_metrics_per_model: usize, max_latency_samples: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            max_metrics_per_model,
            latency_samples: Arc::new(RwLock::new(Vec::new())),
            max_latency_samples,
        }
    }

    /// Create with default settings
    pub fn with_defaults() -> Self {
        Self::new(1000, 10000)
    }

    /// Record model metrics
    pub fn record(&self, metrics: ModelMetrics) {
        let mut store = self.metrics.write();
        let model_metrics = store.entry(metrics.model_id).or_insert_with(Vec::new);

        model_metrics.push(metrics);

        // Limit stored metrics
        if model_metrics.len() > self.max_metrics_per_model {
            model_metrics.remove(0);
        }

        tracing::debug!("Recorded metrics for model {}", store.len());
    }

    /// Record latency sample
    pub fn record_latency(&self, latency_ms: f64) {
        let mut samples = self.latency_samples.write();
        samples.push(latency_ms);

        if samples.len() > self.max_latency_samples {
            samples.remove(0);
        }
    }

    /// Get metrics for a model
    pub fn get_metrics(&self, model_id: uuid::Uuid) -> Option<Vec<ModelMetrics>> {
        self.metrics.read().get(&model_id).cloned()
    }

    /// Get latest metrics for a model
    pub fn get_latest(&self, model_id: uuid::Uuid) -> Option<ModelMetrics> {
        self.metrics
            .read()
            .get(&model_id)
            .and_then(|metrics| metrics.last().cloned())
    }

    /// Calculate performance metrics from samples
    pub fn calculate_performance_metrics(&self) -> PerformanceMetrics {
        let samples = self.latency_samples.read();

        if samples.is_empty() {
            return PerformanceMetrics::default();
        }

        let mut sorted = samples.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let avg = samples.iter().sum::<f64>() / samples.len() as f64;
        let p50 = self.percentile(&sorted, 50.0);
        let p95 = self.percentile(&sorted, 95.0);
        let p99 = self.percentile(&sorted, 99.0);

        let throughput = if avg > 0.0 { 1000.0 / avg } else { 0.0 };

        PerformanceMetrics {
            avg_inference_time_ms: avg,
            p50_latency_ms: p50,
            p95_latency_ms: p95,
            p99_latency_ms: p99,
            throughput_rps: throughput,
            ..Default::default()
        }
    }

    /// Calculate percentile
    fn percentile(&self, sorted_samples: &[f64], percentile: f64) -> f64 {
        if sorted_samples.is_empty() {
            return 0.0;
        }

        let index = ((percentile / 100.0) * sorted_samples.len() as f64) as usize;
        let index = index.min(sorted_samples.len() - 1);
        sorted_samples[index]
    }

    /// Get all model IDs
    pub fn model_ids(&self) -> Vec<uuid::Uuid> {
        self.metrics.read().keys().copied().collect()
    }

    /// Clear all metrics
    pub fn clear(&self) {
        self.metrics.write().clear();
        self.latency_samples.write().clear();
    }

    /// Clear metrics for a specific model
    pub fn clear_model(&self, model_id: uuid::Uuid) {
        self.metrics.write().remove(&model_id);
    }

    /// Get summary statistics
    pub fn summary(&self) -> MetricsSummary {
        let store = self.metrics.read();
        let total_models = store.len();
        let total_metrics: usize = store.values().map(|v| v.len()).sum();

        MetricsSummary {
            total_models,
            total_metrics,
            latency_samples: self.latency_samples.read().len(),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Total number of models tracked
    pub total_models: usize,

    /// Total number of metric records
    pub total_metrics: usize,

    /// Number of latency samples
    pub latency_samples: usize,
}

/// Calculate classification metrics from predictions
pub fn calculate_classification_metrics(
    y_true: &[usize],
    y_pred: &[usize],
    n_classes: usize,
) -> ClassificationMetrics {
    if y_true.is_empty() || y_pred.is_empty() || y_true.len() != y_pred.len() {
        return ClassificationMetrics::default();
    }

    let n = y_true.len();

    // Build confusion matrix
    let mut confusion_matrix = vec![vec![0; n_classes]; n_classes];
    for (true_label, pred_label) in y_true.iter().zip(y_pred.iter()) {
        if *true_label < n_classes && *pred_label < n_classes {
            confusion_matrix[*true_label][*pred_label] += 1;
        }
    }

    // Calculate accuracy
    let correct = y_true.iter().zip(y_pred.iter()).filter(|(t, p)| t == p).count();
    let accuracy = correct as f64 / n as f64;

    // Calculate precision, recall, F1
    let mut total_precision = 0.0;
    let mut total_recall = 0.0;
    let mut total_f1 = 0.0;

    for class in 0..n_classes {
        let tp = confusion_matrix[class][class] as f64;
        let fp: f64 = (0..n_classes)
            .filter(|&i| i != class)
            .map(|i| confusion_matrix[i][class] as f64)
            .sum();
        let fn_: f64 = (0..n_classes)
            .filter(|&i| i != class)
            .map(|i| confusion_matrix[class][i] as f64)
            .sum();

        let precision = if tp + fp > 0.0 { tp / (tp + fp) } else { 0.0 };
        let recall = if tp + fn_ > 0.0 { tp / (tp + fn_) } else { 0.0 };
        let f1 = if precision + recall > 0.0 {
            2.0 * (precision * recall) / (precision + recall)
        } else {
            0.0
        };

        total_precision += precision;
        total_recall += recall;
        total_f1 += f1;
    }

    let precision = total_precision / n_classes as f64;
    let recall = total_recall / n_classes as f64;
    let f1_score = total_f1 / n_classes as f64;

    ClassificationMetrics {
        accuracy,
        precision,
        recall,
        f1_score,
        auc_roc: None,
        confusion_matrix,
        per_class: HashMap::new(),
    }
}

/// Calculate regression metrics from predictions
pub fn calculate_regression_metrics(y_true: &[f64], y_pred: &[f64]) -> RegressionMetrics {
    if y_true.is_empty() || y_pred.is_empty() || y_true.len() != y_pred.len() {
        return RegressionMetrics::default();
    }

    let n = y_true.len() as f64;

    // Calculate errors
    let errors: Vec<f64> = y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(t, p)| t - p)
        .collect();

    let abs_errors: Vec<f64> = errors.iter().map(|e| e.abs()).collect();

    // MAE
    let mae = abs_errors.iter().sum::<f64>() / n;

    // MSE and RMSE
    let mse = errors.iter().map(|e| e * e).sum::<f64>() / n;
    let rmse = mse.sqrt();

    // R2 score
    let y_mean = y_true.iter().sum::<f64>() / n;
    let ss_tot: f64 = y_true.iter().map(|y| (y - y_mean).powi(2)).sum();
    let ss_res: f64 = errors.iter().map(|e| e.powi(2)).sum();
    let r2_score = if ss_tot > 0.0 { 1.0 - (ss_res / ss_tot) } else { 0.0 };

    // MAPE
    let mape = abs_errors
        .iter()
        .zip(y_true.iter())
        .filter(|(_, t)| **t != 0.0)
        .map(|(e, t)| e / t.abs())
        .sum::<f64>()
        / n
        * 100.0;

    // Median AE
    let mut sorted_errors = abs_errors.clone();
    sorted_errors.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_ae = sorted_errors[sorted_errors.len() / 2];

    RegressionMetrics {
        mae,
        mse,
        rmse,
        r2_score,
        mape,
        median_ae,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_metrics_default() {
        let metrics = ClassificationMetrics::default();
        assert_eq!(metrics.accuracy, 0.0);
        assert_eq!(metrics.precision, 0.0);
    }

    #[test]
    fn test_regression_metrics_default() {
        let metrics = RegressionMetrics::default();
        assert_eq!(metrics.mae, 0.0);
        assert_eq!(metrics.rmse, 0.0);
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.avg_inference_time_ms, 0.0);
        assert_eq!(metrics.throughput_rps, 0.0);
    }

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::with_defaults();
        let summary = collector.summary();
        assert_eq!(summary.total_models, 0);
        assert_eq!(summary.total_metrics, 0);
    }

    #[test]
    fn test_calculate_classification_metrics() {
        let y_true = vec![0, 1, 1, 0, 1];
        let y_pred = vec![0, 1, 0, 0, 1];
        let metrics = calculate_classification_metrics(&y_true, &y_pred, 2);

        assert!(metrics.accuracy > 0.0);
        assert!(metrics.precision >= 0.0);
        assert!(metrics.recall >= 0.0);
    }

    #[test]
    fn test_calculate_regression_metrics() {
        let y_true = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y_pred = vec![1.1, 2.1, 2.9, 4.2, 4.8];
        let metrics = calculate_regression_metrics(&y_true, &y_pred);

        assert!(metrics.mae > 0.0);
        assert!(metrics.rmse > 0.0);
        assert!(metrics.r2_score > 0.0);
    }

    #[test]
    fn test_record_latency() {
        let collector = MetricsCollector::with_defaults();
        collector.record_latency(10.0);
        collector.record_latency(20.0);
        collector.record_latency(15.0);

        let perf = collector.calculate_performance_metrics();
        assert!(perf.avg_inference_time_ms > 0.0);
        assert!(perf.p50_latency_ms > 0.0);
    }
}
