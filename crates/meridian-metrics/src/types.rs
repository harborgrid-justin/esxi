//! Custom metric types: Counters, Gauges, Histograms, and Summaries.

use crate::error::{MetricsError, Result};
use chrono::{DateTime, Utc};
use hdrhistogram::Histogram;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Labels for multi-dimensional metrics
pub type Labels = HashMap<String, String>;

/// A monotonically increasing counter
#[derive(Debug, Clone)]
pub struct Counter {
    name: String,
    help: String,
    value: Arc<AtomicU64>,
    labels: Labels,
    created_at: DateTime<Utc>,
}

impl Counter {
    /// Create a new counter
    pub fn new<S: Into<String>>(name: S, help: S) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            value: Arc::new(AtomicU64::new(0)),
            labels: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    /// Create a counter with labels
    pub fn with_labels<S: Into<String>>(name: S, help: S, labels: Labels) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            value: Arc::new(AtomicU64::new(0)),
            labels,
            created_at: Utc::now(),
        }
    }

    /// Increment the counter by 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the counter by a specific amount
    pub fn add(&self, delta: u64) {
        self.value.fetch_add(delta, Ordering::Relaxed);
    }

    /// Get the current value
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Get the metric name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the help text
    pub fn help(&self) -> &str {
        &self.help
    }

    /// Get the labels
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Reset the counter to zero
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

/// A gauge that can go up or down
#[derive(Debug, Clone)]
pub struct Gauge {
    name: String,
    help: String,
    value: Arc<RwLock<f64>>,
    labels: Labels,
    created_at: DateTime<Utc>,
}

impl Gauge {
    /// Create a new gauge
    pub fn new<S: Into<String>>(name: S, help: S) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            value: Arc::new(RwLock::new(0.0)),
            labels: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    /// Create a gauge with labels
    pub fn with_labels<S: Into<String>>(name: S, help: S, labels: Labels) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            value: Arc::new(RwLock::new(0.0)),
            labels,
            created_at: Utc::now(),
        }
    }

    /// Set the gauge to a specific value
    pub fn set(&self, val: f64) {
        *self.value.write() = val;
    }

    /// Increment the gauge
    pub fn inc(&self) {
        *self.value.write() += 1.0;
    }

    /// Decrement the gauge
    pub fn dec(&self) {
        *self.value.write() -= 1.0;
    }

    /// Add a value to the gauge
    pub fn add(&self, delta: f64) {
        *self.value.write() += delta;
    }

    /// Subtract a value from the gauge
    pub fn sub(&self, delta: f64) {
        *self.value.write() -= delta;
    }

    /// Get the current value
    pub fn get(&self) -> f64 {
        *self.value.read()
    }

    /// Get the metric name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the help text
    pub fn help(&self) -> &str {
        &self.help
    }

    /// Get the labels
    pub fn labels(&self) -> &Labels {
        &self.labels
    }
}

/// A histogram for observing distributions
#[derive(Debug)]
pub struct HistogramMetric {
    name: String,
    help: String,
    histogram: Arc<RwLock<Histogram<u64>>>,
    labels: Labels,
    created_at: DateTime<Utc>,
}

impl HistogramMetric {
    /// Create a new histogram
    pub fn new<S: Into<String>>(name: S, help: S) -> Result<Self> {
        let histogram = Histogram::new(3)
            .map_err(|e| MetricsError::invalid_metric(format!("Failed to create histogram: {}", e)))?;

        Ok(Self {
            name: name.into(),
            help: help.into(),
            histogram: Arc::new(RwLock::new(histogram)),
            labels: HashMap::new(),
            created_at: Utc::now(),
        })
    }

    /// Create a histogram with labels
    pub fn with_labels<S: Into<String>>(name: S, help: S, labels: Labels) -> Result<Self> {
        let histogram = Histogram::new(3)
            .map_err(|e| MetricsError::invalid_metric(format!("Failed to create histogram: {}", e)))?;

        Ok(Self {
            name: name.into(),
            help: help.into(),
            histogram: Arc::new(RwLock::new(histogram)),
            labels,
            created_at: Utc::now(),
        })
    }

    /// Observe a value
    pub fn observe(&self, value: u64) -> Result<()> {
        self.histogram
            .write()
            .record(value)
            .map_err(|e| MetricsError::collection(format!("Failed to record value: {}", e)))
    }

    /// Get percentile value
    pub fn percentile(&self, percentile: f64) -> u64 {
        self.histogram.read().value_at_quantile(percentile)
    }

    /// Get the mean value
    pub fn mean(&self) -> f64 {
        self.histogram.read().mean()
    }

    /// Get the standard deviation
    pub fn std_dev(&self) -> f64 {
        self.histogram.read().stdev()
    }

    /// Get the minimum value
    pub fn min(&self) -> u64 {
        self.histogram.read().min()
    }

    /// Get the maximum value
    pub fn max(&self) -> u64 {
        self.histogram.read().max()
    }

    /// Get the total count
    pub fn count(&self) -> u64 {
        self.histogram.read().len()
    }

    /// Get the metric name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the help text
    pub fn help(&self) -> &str {
        &self.help
    }

    /// Get the labels
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Reset the histogram
    pub fn reset(&self) {
        self.histogram.write().reset();
    }
}

impl Clone for HistogramMetric {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            help: self.help.clone(),
            histogram: Arc::new(RwLock::new(self.histogram.read().clone())),
            labels: self.labels.clone(),
            created_at: self.created_at,
        }
    }
}

/// Summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStats {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub std_dev: f64,
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

/// A summary metric for calculating quantiles
#[derive(Debug, Clone)]
pub struct Summary {
    name: String,
    help: String,
    values: Arc<RwLock<Vec<f64>>>,
    labels: Labels,
    created_at: DateTime<Utc>,
    max_age_secs: u64,
}

impl Summary {
    /// Create a new summary
    pub fn new<S: Into<String>>(name: S, help: S) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            values: Arc::new(RwLock::new(Vec::new())),
            labels: HashMap::new(),
            created_at: Utc::now(),
            max_age_secs: 600, // 10 minutes default
        }
    }

    /// Create a summary with labels
    pub fn with_labels<S: Into<String>>(name: S, help: S, labels: Labels) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            values: Arc::new(RwLock::new(Vec::new())),
            labels,
            created_at: Utc::now(),
            max_age_secs: 600,
        }
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        self.values.write().push(value);
    }

    /// Get summary statistics
    pub fn get_stats(&self) -> SummaryStats {
        let mut values = self.values.write();

        if values.is_empty() {
            return SummaryStats {
                count: 0,
                sum: 0.0,
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                std_dev: 0.0,
                p50: 0.0,
                p90: 0.0,
                p95: 0.0,
                p99: 0.0,
            };
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let count = values.len() as u64;
        let sum: f64 = values.iter().sum();
        let mean = sum / count as f64;

        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        let percentile = |p: f64| -> f64 {
            let idx = ((p / 100.0) * (count - 1) as f64).round() as usize;
            values[idx.min(values.len() - 1)]
        };

        SummaryStats {
            count,
            sum,
            min: values[0],
            max: values[values.len() - 1],
            mean,
            std_dev,
            p50: percentile(50.0),
            p90: percentile(90.0),
            p95: percentile(95.0),
            p99: percentile(99.0),
        }
    }

    /// Get the metric name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the help text
    pub fn help(&self) -> &str {
        &self.help
    }

    /// Get the labels
    pub fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Reset the summary
    pub fn reset(&self) {
        self.values.write().clear();
    }
}

/// GIS-specific metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GisMetricType {
    /// Query execution latency in milliseconds
    QueryLatency(f64),

    /// Tile rendering time in milliseconds
    TileRenderTime {
        zoom: u8,
        x: u32,
        y: u32,
        duration_ms: f64,
    },

    /// Spatial operation duration
    SpatialOperation {
        operation: String,
        geometry_count: u64,
        duration_ms: f64,
    },

    /// Cache hit rate
    CacheHitRate {
        cache_type: String,
        hits: u64,
        misses: u64,
    },

    /// Data load time
    DataLoadTime {
        source: String,
        bytes: u64,
        duration_ms: f64,
    },
}

/// Metric value for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MetricValue {
    Counter { value: u64 },
    Gauge { value: f64 },
    Histogram { stats: SummaryStats },
    Summary { stats: SummaryStats },
}

/// Complete metric snapshot with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub name: String,
    pub help: String,
    pub labels: Labels,
    pub value: MetricValue,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter() {
        let counter = Counter::new("test_counter", "Test counter");
        assert_eq!(counter.get(), 0);

        counter.inc();
        assert_eq!(counter.get(), 1);

        counter.add(5);
        assert_eq!(counter.get(), 6);

        counter.reset();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test_gauge", "Test gauge");
        assert_eq!(gauge.get(), 0.0);

        gauge.set(10.5);
        assert_eq!(gauge.get(), 10.5);

        gauge.inc();
        assert_eq!(gauge.get(), 11.5);

        gauge.dec();
        assert_eq!(gauge.get(), 10.5);

        gauge.add(2.5);
        assert_eq!(gauge.get(), 13.0);

        gauge.sub(3.0);
        assert_eq!(gauge.get(), 10.0);
    }

    #[test]
    fn test_histogram() {
        let histogram = HistogramMetric::new("test_histogram", "Test histogram").unwrap();

        for i in 1..=100 {
            histogram.observe(i).unwrap();
        }

        assert_eq!(histogram.count(), 100);
        assert_eq!(histogram.min(), 1);
        assert_eq!(histogram.max(), 100);
        assert!(histogram.mean() > 49.0 && histogram.mean() < 51.0);
    }

    #[test]
    fn test_summary() {
        let summary = Summary::new("test_summary", "Test summary");

        for i in 1..=100 {
            summary.observe(i as f64);
        }

        let stats = summary.get_stats();
        assert_eq!(stats.count, 100);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 100.0);
        assert!(stats.mean > 49.0 && stats.mean < 51.0);
        assert!(stats.p50 > 48.0 && stats.p50 < 52.0);
        assert!(stats.p99 > 95.0);
    }

    #[test]
    fn test_counter_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("tenant".to_string(), "tenant1".to_string());
        labels.insert("region".to_string(), "us-west".to_string());

        let counter = Counter::with_labels("test_counter", "Test counter", labels.clone());
        assert_eq!(counter.labels().get("tenant").unwrap(), "tenant1");
        assert_eq!(counter.labels().get("region").unwrap(), "us-west");
    }
}
