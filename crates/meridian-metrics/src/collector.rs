//! Metric collection engine for gathering and managing metrics.

use crate::error::{MetricsError, Result};
use crate::types::*;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn};

/// Configuration for the metrics collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Enable collection
    pub enabled: bool,

    /// Collection interval in seconds
    pub interval_secs: u64,

    /// Maximum number of metrics to store
    pub max_metrics: usize,

    /// Enable GIS-specific metrics
    pub enable_gis_metrics: bool,

    /// Buffer size for metric batching
    pub buffer_size: usize,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_secs: 15,
            max_metrics: 10000,
            enable_gis_metrics: true,
            buffer_size: 1000,
        }
    }
}

/// The main metrics collector
pub struct MetricsCollector {
    config: CollectorConfig,
    counters: Arc<DashMap<String, Counter>>,
    gauges: Arc<DashMap<String, Gauge>>,
    histograms: Arc<DashMap<String, HistogramMetric>>,
    summaries: Arc<DashMap<String, Summary>>,
    gis_metrics: Arc<RwLock<Vec<GisMetricType>>>,
    start_time: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: CollectorConfig) -> Self {
        info!("Initializing metrics collector with config: {:?}", config);

        Self {
            config,
            counters: Arc::new(DashMap::new()),
            gauges: Arc::new(DashMap::new()),
            histograms: Arc::new(DashMap::new()),
            summaries: Arc::new(DashMap::new()),
            gis_metrics: Arc::new(RwLock::new(Vec::new())),
            start_time: Instant::now(),
        }
    }

    /// Create a collector with default configuration
    pub fn default() -> Self {
        Self::new(CollectorConfig::default())
    }

    /// Register a new counter
    pub fn register_counter<S: Into<String>>(&self, name: S, help: S) -> Result<Counter> {
        let name = name.into();

        if self.counters.len() >= self.config.max_metrics {
            return Err(MetricsError::collection("Maximum metrics limit reached"));
        }

        let counter = Counter::new(name.clone(), help.into());
        self.counters.insert(name.clone(), counter.clone());

        debug!("Registered counter: {}", name);
        Ok(counter)
    }

    /// Register a counter with labels
    pub fn register_counter_with_labels<S: Into<String>>(
        &self,
        name: S,
        help: S,
        labels: Labels,
    ) -> Result<Counter> {
        let name = name.into();
        let key = self.make_key(&name, &labels);

        if self.counters.len() >= self.config.max_metrics {
            return Err(MetricsError::collection("Maximum metrics limit reached"));
        }

        let counter = Counter::with_labels(name.clone(), help.into(), labels);
        self.counters.insert(key.clone(), counter.clone());

        debug!("Registered counter with labels: {}", key);
        Ok(counter)
    }

    /// Get or create a counter
    pub fn counter<S: Into<String>>(&self, name: S) -> Result<Counter> {
        let name = name.into();

        if let Some(counter) = self.counters.get(&name) {
            return Ok(counter.clone());
        }

        self.register_counter(name.clone(), format!("Auto-registered counter: {}", name))
    }

    /// Register a new gauge
    pub fn register_gauge<S: Into<String>>(&self, name: S, help: S) -> Result<Gauge> {
        let name = name.into();

        if self.gauges.len() >= self.config.max_metrics {
            return Err(MetricsError::collection("Maximum metrics limit reached"));
        }

        let gauge = Gauge::new(name.clone(), help.into());
        self.gauges.insert(name.clone(), gauge.clone());

        debug!("Registered gauge: {}", name);
        Ok(gauge)
    }

    /// Register a gauge with labels
    pub fn register_gauge_with_labels<S: Into<String>>(
        &self,
        name: S,
        help: S,
        labels: Labels,
    ) -> Result<Gauge> {
        let name = name.into();
        let key = self.make_key(&name, &labels);

        if self.gauges.len() >= self.config.max_metrics {
            return Err(MetricsError::collection("Maximum metrics limit reached"));
        }

        let gauge = Gauge::with_labels(name.clone(), help.into(), labels);
        self.gauges.insert(key.clone(), gauge.clone());

        debug!("Registered gauge with labels: {}", key);
        Ok(gauge)
    }

    /// Get or create a gauge
    pub fn gauge<S: Into<String>>(&self, name: S) -> Result<Gauge> {
        let name = name.into();

        if let Some(gauge) = self.gauges.get(&name) {
            return Ok(gauge.clone());
        }

        self.register_gauge(name.clone(), format!("Auto-registered gauge: {}", name))
    }

    /// Register a new histogram
    pub fn register_histogram<S: Into<String>>(&self, name: S, help: S) -> Result<HistogramMetric> {
        let name = name.into();

        if self.histograms.len() >= self.config.max_metrics {
            return Err(MetricsError::collection("Maximum metrics limit reached"));
        }

        let histogram = HistogramMetric::new(name.clone(), help.into())?;
        self.histograms.insert(name.clone(), histogram.clone());

        debug!("Registered histogram: {}", name);
        Ok(histogram)
    }

    /// Register a histogram with labels
    pub fn register_histogram_with_labels<S: Into<String>>(
        &self,
        name: S,
        help: S,
        labels: Labels,
    ) -> Result<HistogramMetric> {
        let name = name.into();
        let key = self.make_key(&name, &labels);

        if self.histograms.len() >= self.config.max_metrics {
            return Err(MetricsError::collection("Maximum metrics limit reached"));
        }

        let histogram = HistogramMetric::with_labels(name.clone(), help.into(), labels)?;
        self.histograms.insert(key.clone(), histogram.clone());

        debug!("Registered histogram with labels: {}", key);
        Ok(histogram)
    }

    /// Get or create a histogram
    pub fn histogram<S: Into<String>>(&self, name: S) -> Result<HistogramMetric> {
        let name = name.into();

        if let Some(histogram) = self.histograms.get(&name) {
            return Ok(histogram.clone());
        }

        self.register_histogram(name.clone(), format!("Auto-registered histogram: {}", name))
    }

    /// Register a new summary
    pub fn register_summary<S: Into<String>>(&self, name: S, help: S) -> Result<Summary> {
        let name = name.into();

        if self.summaries.len() >= self.config.max_metrics {
            return Err(MetricsError::collection("Maximum metrics limit reached"));
        }

        let summary = Summary::new(name.clone(), help.into());
        self.summaries.insert(name.clone(), summary.clone());

        debug!("Registered summary: {}", name);
        Ok(summary)
    }

    /// Register a summary with labels
    pub fn register_summary_with_labels<S: Into<String>>(
        &self,
        name: S,
        help: S,
        labels: Labels,
    ) -> Result<Summary> {
        let name = name.into();
        let key = self.make_key(&name, &labels);

        if self.summaries.len() >= self.config.max_metrics {
            return Err(MetricsError::collection("Maximum metrics limit reached"));
        }

        let summary = Summary::with_labels(name.clone(), help.into(), labels);
        self.summaries.insert(key.clone(), summary.clone());

        debug!("Registered summary with labels: {}", key);
        Ok(summary)
    }

    /// Get or create a summary
    pub fn summary<S: Into<String>>(&self, name: S) -> Result<Summary> {
        let name = name.into();

        if let Some(summary) = self.summaries.get(&name) {
            return Ok(summary.clone());
        }

        self.register_summary(name.clone(), format!("Auto-registered summary: {}", name))
    }

    /// Record a GIS-specific metric
    pub fn record_gis_metric(&self, metric: GisMetricType) -> Result<()> {
        if !self.config.enable_gis_metrics {
            return Ok(());
        }

        let mut metrics = self.gis_metrics.write();

        // Limit buffer size
        if metrics.len() >= self.config.buffer_size {
            warn!("GIS metrics buffer full, removing oldest entries");
            let len = metrics.len();
            metrics.drain(0..len / 2);
        }

        metrics.push(metric);
        Ok(())
    }

    /// Get all GIS metrics
    pub fn get_gis_metrics(&self) -> Vec<GisMetricType> {
        self.gis_metrics.read().clone()
    }

    /// Clear GIS metrics
    pub fn clear_gis_metrics(&self) {
        self.gis_metrics.write().clear();
    }

    /// Collect all metric snapshots
    pub fn collect_snapshots(&self) -> Vec<MetricSnapshot> {
        let mut snapshots = Vec::new();
        let timestamp = Utc::now();

        // Collect counters
        for entry in self.counters.iter() {
            let counter = entry.value();
            snapshots.push(MetricSnapshot {
                name: counter.name().to_string(),
                help: counter.help().to_string(),
                labels: counter.labels().clone(),
                value: MetricValue::Counter {
                    value: counter.get(),
                },
                timestamp,
            });
        }

        // Collect gauges
        for entry in self.gauges.iter() {
            let gauge = entry.value();
            snapshots.push(MetricSnapshot {
                name: gauge.name().to_string(),
                help: gauge.help().to_string(),
                labels: gauge.labels().clone(),
                value: MetricValue::Gauge {
                    value: gauge.get(),
                },
                timestamp,
            });
        }

        // Collect histograms
        for entry in self.histograms.iter() {
            let histogram = entry.value();
            let stats = SummaryStats {
                count: histogram.count(),
                sum: histogram.mean() * histogram.count() as f64,
                min: histogram.min() as f64,
                max: histogram.max() as f64,
                mean: histogram.mean(),
                std_dev: histogram.std_dev(),
                p50: histogram.percentile(0.5) as f64,
                p90: histogram.percentile(0.9) as f64,
                p95: histogram.percentile(0.95) as f64,
                p99: histogram.percentile(0.99) as f64,
            };

            snapshots.push(MetricSnapshot {
                name: histogram.name().to_string(),
                help: histogram.help().to_string(),
                labels: histogram.labels().clone(),
                value: MetricValue::Histogram { stats },
                timestamp,
            });
        }

        // Collect summaries
        for entry in self.summaries.iter() {
            let summary = entry.value();
            snapshots.push(MetricSnapshot {
                name: summary.name().to_string(),
                help: summary.help().to_string(),
                labels: summary.labels().clone(),
                value: MetricValue::Summary {
                    stats: summary.get_stats(),
                },
                timestamp,
            });
        }

        snapshots
    }

    /// Get metrics count
    pub fn metrics_count(&self) -> usize {
        self.counters.len() + self.gauges.len() + self.histograms.len() + self.summaries.len()
    }

    /// Get uptime duration
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Reset all metrics
    pub fn reset_all(&self) {
        for entry in self.counters.iter() {
            entry.value().reset();
        }

        for entry in self.histograms.iter() {
            entry.value().reset();
        }

        for entry in self.summaries.iter() {
            entry.value().reset();
        }

        self.gis_metrics.write().clear();

        info!("All metrics reset");
    }

    /// Make a unique key from name and labels
    fn make_key(&self, name: &str, labels: &Labels) -> String {
        let mut key = name.to_string();
        if !labels.is_empty() {
            let mut label_pairs: Vec<_> = labels.iter().collect();
            label_pairs.sort_by_key(|(k, _)| *k);
            for (k, v) in label_pairs {
                key.push_str(&format!(",{}={}", k, v));
            }
        }
        key
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            counters: Arc::clone(&self.counters),
            gauges: Arc::clone(&self.gauges),
            histograms: Arc::clone(&self.histograms),
            summaries: Arc::clone(&self.summaries),
            gis_metrics: Arc::clone(&self.gis_metrics),
            start_time: self.start_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_creation() {
        let collector = MetricsCollector::default();
        assert_eq!(collector.metrics_count(), 0);
    }

    #[test]
    fn test_register_counter() {
        let collector = MetricsCollector::default();
        let counter = collector.register_counter("test_counter", "Test").unwrap();

        counter.inc();
        assert_eq!(counter.get(), 1);
        assert_eq!(collector.metrics_count(), 1);
    }

    #[test]
    fn test_register_gauge() {
        let collector = MetricsCollector::default();
        let gauge = collector.register_gauge("test_gauge", "Test").unwrap();

        gauge.set(42.0);
        assert_eq!(gauge.get(), 42.0);
        assert_eq!(collector.metrics_count(), 1);
    }

    #[test]
    fn test_collect_snapshots() {
        let collector = MetricsCollector::default();

        let counter = collector.register_counter("test_counter", "Test").unwrap();
        counter.add(5);

        let gauge = collector.register_gauge("test_gauge", "Test").unwrap();
        gauge.set(3.14);

        let snapshots = collector.collect_snapshots();
        assert_eq!(snapshots.len(), 2);
    }

    #[test]
    fn test_gis_metrics() {
        let collector = MetricsCollector::default();

        let metric = GisMetricType::QueryLatency(123.45);
        collector.record_gis_metric(metric.clone()).unwrap();

        let metrics = collector.get_gis_metrics();
        assert_eq!(metrics.len(), 1);

        collector.clear_gis_metrics();
        assert_eq!(collector.get_gis_metrics().len(), 0);
    }

    #[test]
    fn test_labels() {
        let collector = MetricsCollector::default();

        let mut labels = HashMap::new();
        labels.insert("tenant".to_string(), "test".to_string());

        let counter = collector
            .register_counter_with_labels("requests", "Request count", labels)
            .unwrap();

        counter.inc();
        assert_eq!(counter.get(), 1);
    }
}
