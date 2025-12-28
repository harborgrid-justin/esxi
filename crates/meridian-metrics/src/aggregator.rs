//! Metrics aggregation and rollup strategies.

use crate::error::{MetricsError, Result};
use crate::types::{MetricSnapshot, MetricValue, SummaryStats};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Time window for aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeWindow {
    /// 1 minute window
    Minute,
    /// 5 minute window
    FiveMinutes,
    /// 15 minute window
    FifteenMinutes,
    /// 1 hour window
    Hour,
    /// 1 day window
    Day,
    /// 1 week window
    Week,
}

impl TimeWindow {
    /// Get duration for this window
    pub fn duration(&self) -> Duration {
        match self {
            Self::Minute => Duration::minutes(1),
            Self::FiveMinutes => Duration::minutes(5),
            Self::FifteenMinutes => Duration::minutes(15),
            Self::Hour => Duration::hours(1),
            Self::Day => Duration::days(1),
            Self::Week => Duration::weeks(1),
        }
    }

    /// Get duration in seconds
    pub fn seconds(&self) -> i64 {
        self.duration().num_seconds()
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Minute => "1m",
            Self::FiveMinutes => "5m",
            Self::FifteenMinutes => "15m",
            Self::Hour => "1h",
            Self::Day => "1d",
            Self::Week => "1w",
        }
    }
}

/// Aggregation function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AggregationFunc {
    /// Sum of all values
    Sum,
    /// Average of all values
    Average,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// Count of values
    Count,
    /// First value in window
    First,
    /// Last value in window
    Last,
    /// Median value
    Median,
    /// 95th percentile
    P95,
    /// 99th percentile
    P99,
}

impl AggregationFunc {
    /// Apply aggregation to a set of values
    pub fn apply(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        match self {
            Self::Sum => values.iter().sum(),
            Self::Average => values.iter().sum::<f64>() / values.len() as f64,
            Self::Min => values
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min),
            Self::Max => values
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max),
            Self::Count => values.len() as f64,
            Self::First => values[0],
            Self::Last => values[values.len() - 1],
            Self::Median => {
                let mut sorted = values.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mid = sorted.len() / 2;
                if sorted.len() % 2 == 0 {
                    (sorted[mid - 1] + sorted[mid]) / 2.0
                } else {
                    sorted[mid]
                }
            }
            Self::P95 => {
                let mut sorted = values.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let idx = ((sorted.len() as f64 * 0.95) as usize).min(sorted.len() - 1);
                sorted[idx]
            }
            Self::P99 => {
                let mut sorted = values.to_vec();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let idx = ((sorted.len() as f64 * 0.99) as usize).min(sorted.len() - 1);
                sorted[idx]
            }
        }
    }
}

/// Aggregated metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedPoint {
    /// Metric name
    pub metric_name: String,

    /// Labels
    pub labels: HashMap<String, String>,

    /// Aggregated value
    pub value: f64,

    /// Aggregation function used
    pub function: AggregationFunc,

    /// Time window
    pub window: TimeWindow,

    /// Window start time
    pub window_start: DateTime<Utc>,

    /// Window end time
    pub window_end: DateTime<Utc>,

    /// Number of samples in this aggregation
    pub sample_count: usize,
}

/// Rollup rule for automatic aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollupRule {
    /// Rule name
    pub name: String,

    /// Metric name pattern (supports wildcards)
    pub metric_pattern: String,

    /// Aggregation function
    pub function: AggregationFunc,

    /// Time window
    pub window: TimeWindow,

    /// Retention period for rollups
    pub retention_secs: i64,

    /// Enabled flag
    pub enabled: bool,
}

impl RollupRule {
    /// Check if a metric matches this rule
    pub fn matches(&self, metric_name: &str) -> bool {
        if self.metric_pattern == "*" {
            return true;
        }

        if self.metric_pattern.ends_with('*') {
            metric_name.starts_with(&self.metric_pattern[..self.metric_pattern.len() - 1])
        } else {
            metric_name == self.metric_pattern
        }
    }
}

/// Aggregation buffer for a specific metric and window
struct AggregationBuffer {
    metric_name: String,
    labels: HashMap<String, String>,
    window: TimeWindow,
    function: AggregationFunc,
    values: VecDeque<(DateTime<Utc>, f64)>,
    max_points: usize,
}

impl AggregationBuffer {
    fn new(
        metric_name: String,
        labels: HashMap<String, String>,
        window: TimeWindow,
        function: AggregationFunc,
        max_points: usize,
    ) -> Self {
        Self {
            metric_name,
            labels,
            window,
            function,
            values: VecDeque::new(),
            max_points,
        }
    }

    fn add(&mut self, timestamp: DateTime<Utc>, value: f64) {
        self.values.push_back((timestamp, value));

        // Trim old values beyond max_points
        while self.values.len() > self.max_points {
            self.values.pop_front();
        }
    }

    fn aggregate(&self, window_start: DateTime<Utc>, window_end: DateTime<Utc>) -> Option<AggregatedPoint> {
        // Filter values within the window
        let window_values: Vec<f64> = self
            .values
            .iter()
            .filter(|(ts, _)| *ts >= window_start && *ts < window_end)
            .map(|(_, v)| *v)
            .collect();

        if window_values.is_empty() {
            return None;
        }

        let aggregated_value = self.function.apply(&window_values);

        Some(AggregatedPoint {
            metric_name: self.metric_name.clone(),
            labels: self.labels.clone(),
            value: aggregated_value,
            function: self.function,
            window: self.window,
            window_start,
            window_end,
            sample_count: window_values.len(),
        })
    }

    fn cleanup(&mut self, cutoff: DateTime<Utc>) {
        self.values.retain(|(ts, _)| *ts >= cutoff);
    }
}

/// Metrics aggregator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatorConfig {
    /// Enable aggregation
    pub enabled: bool,

    /// Maximum points per buffer
    pub max_points_per_buffer: usize,

    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,

    /// Default retention in seconds
    pub default_retention_secs: i64,
}

impl Default for AggregatorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_points_per_buffer: 10000,
            cleanup_interval_secs: 300,
            default_retention_secs: 86400 * 7, // 7 days
        }
    }
}

/// Metrics aggregator
pub struct MetricsAggregator {
    config: AggregatorConfig,
    buffers: Arc<RwLock<HashMap<String, AggregationBuffer>>>,
    rollup_rules: Arc<RwLock<Vec<RollupRule>>>,
    aggregated_points: Arc<RwLock<Vec<AggregatedPoint>>>,
}

impl MetricsAggregator {
    /// Create a new metrics aggregator
    pub fn new(config: AggregatorConfig) -> Self {
        info!("Initializing metrics aggregator");

        Self {
            config,
            buffers: Arc::new(RwLock::new(HashMap::new())),
            rollup_rules: Arc::new(RwLock::new(Vec::new())),
            aggregated_points: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(AggregatorConfig::default())
    }

    /// Add a rollup rule
    pub fn add_rule(&self, rule: RollupRule) {
        let mut rules = self.rollup_rules.write();
        info!("Adding rollup rule: {} for {}", rule.name, rule.metric_pattern);
        rules.push(rule);
    }

    /// Remove a rollup rule
    pub fn remove_rule(&self, name: &str) -> Option<RollupRule> {
        let mut rules = self.rollup_rules.write();
        if let Some(pos) = rules.iter().position(|r| r.name == name) {
            info!("Removing rollup rule: {}", name);
            Some(rules.remove(pos))
        } else {
            None
        }
    }

    /// Get all rollup rules
    pub fn rules(&self) -> Vec<RollupRule> {
        self.rollup_rules.read().clone()
    }

    /// Ingest a metric snapshot
    pub fn ingest(&self, snapshot: &MetricSnapshot) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let rules = self.rollup_rules.read();

        for rule in rules.iter() {
            if !rule.enabled || !rule.matches(&snapshot.name) {
                continue;
            }

            let value = match &snapshot.value {
                MetricValue::Counter { value } => *value as f64,
                MetricValue::Gauge { value } => *value,
                MetricValue::Histogram { stats } => stats.mean,
                MetricValue::Summary { stats } => stats.mean,
            };

            let key = self.make_buffer_key(&snapshot.name, &snapshot.labels, rule.window, rule.function);

            let mut buffers = self.buffers.write();

            let buffer = buffers.entry(key.clone()).or_insert_with(|| {
                AggregationBuffer::new(
                    snapshot.name.clone(),
                    snapshot.labels.clone(),
                    rule.window,
                    rule.function,
                    self.config.max_points_per_buffer,
                )
            });

            buffer.add(snapshot.timestamp, value);
        }

        Ok(())
    }

    /// Perform aggregation for a time window
    pub fn aggregate(&self, window: TimeWindow) -> Result<Vec<AggregatedPoint>> {
        let now = Utc::now();
        let window_duration = window.duration();
        let window_end = now;
        let window_start = now - window_duration;

        let mut results = Vec::new();
        let buffers = self.buffers.read();

        for buffer in buffers.values() {
            if buffer.window == window {
                if let Some(point) = buffer.aggregate(window_start, window_end) {
                    results.push(point);
                }
            }
        }

        // Store aggregated points
        if !results.is_empty() {
            let mut points = self.aggregated_points.write();
            points.extend(results.clone());

            debug!("Aggregated {} points for window {:?}", results.len(), window);
        }

        Ok(results)
    }

    /// Get aggregated points
    pub fn get_aggregated(&self, metric_name: Option<&str>, window: Option<TimeWindow>) -> Vec<AggregatedPoint> {
        let points = self.aggregated_points.read();

        points
            .iter()
            .filter(|p| {
                if let Some(name) = metric_name {
                    if &p.metric_name != name {
                        return false;
                    }
                }

                if let Some(w) = window {
                    if p.window != w {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    /// Cleanup old data
    pub fn cleanup(&self) -> Result<()> {
        let now = Utc::now();
        let rules = self.rollup_rules.read();

        // Build retention map
        let retention_map: HashMap<String, i64> = rules
            .iter()
            .map(|r| (r.name.clone(), r.retention_secs))
            .collect();

        // Cleanup buffers
        {
            let mut buffers = self.buffers.write();

            for buffer in buffers.values_mut() {
                let retention = retention_map
                    .values()
                    .max()
                    .copied()
                    .unwrap_or(self.config.default_retention_secs);

                let cutoff = now - Duration::seconds(retention);
                buffer.cleanup(cutoff);
            }
        }

        // Cleanup aggregated points
        {
            let mut points = self.aggregated_points.write();
            let before_len = points.len();

            let max_retention = retention_map
                .values()
                .max()
                .copied()
                .unwrap_or(self.config.default_retention_secs);

            let cutoff = now - Duration::seconds(max_retention);

            points.retain(|p| p.window_end >= cutoff);

            let removed = before_len - points.len();
            if removed > 0 {
                info!("Cleaned up {} old aggregated points", removed);
            }
        }

        Ok(())
    }

    /// Start background aggregation tasks
    pub async fn start_background_aggregation(self: Arc<Self>) {
        if !self.config.enabled {
            info!("Aggregation disabled");
            return;
        }

        // Spawn aggregation tasks for different windows
        let windows = vec![
            TimeWindow::Minute,
            TimeWindow::FiveMinutes,
            TimeWindow::Hour,
            TimeWindow::Day,
        ];

        for window in windows {
            let aggregator = Arc::clone(&self);

            tokio::spawn(async move {
                let mut ticker = tokio::time::interval(
                    std::time::Duration::from_secs(window.seconds() as u64)
                );

                loop {
                    ticker.tick().await;

                    if let Err(e) = aggregator.aggregate(window) {
                        warn!("Aggregation failed for window {:?}: {}", window, e);
                    }
                }
            });
        }

        // Spawn cleanup task
        let cleanup_interval = std::time::Duration::from_secs(self.config.cleanup_interval_secs);

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(cleanup_interval);

            loop {
                ticker.tick().await;

                if let Err(e) = self.cleanup() {
                    warn!("Cleanup failed: {}", e);
                }
            }
        });

        info!("Background aggregation tasks started");
    }

    /// Make a unique key for a buffer
    fn make_buffer_key(
        &self,
        metric_name: &str,
        labels: &HashMap<String, String>,
        window: TimeWindow,
        function: AggregationFunc,
    ) -> String {
        let mut key = format!("{}:{:?}:{:?}", metric_name, window, function);

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

impl Clone for MetricsAggregator {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            buffers: Arc::clone(&self.buffers),
            rollup_rules: Arc::clone(&self.rollup_rules),
            aggregated_points: Arc::clone(&self.aggregated_points),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_functions() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        assert_eq!(AggregationFunc::Sum.apply(&values), 15.0);
        assert_eq!(AggregationFunc::Average.apply(&values), 3.0);
        assert_eq!(AggregationFunc::Min.apply(&values), 1.0);
        assert_eq!(AggregationFunc::Max.apply(&values), 5.0);
        assert_eq!(AggregationFunc::Count.apply(&values), 5.0);
        assert_eq!(AggregationFunc::First.apply(&values), 1.0);
        assert_eq!(AggregationFunc::Last.apply(&values), 5.0);
        assert_eq!(AggregationFunc::Median.apply(&values), 3.0);
    }

    #[test]
    fn test_rollup_rule_matching() {
        let rule = RollupRule {
            name: "test".to_string(),
            metric_pattern: "query_*".to_string(),
            function: AggregationFunc::Average,
            window: TimeWindow::Minute,
            retention_secs: 3600,
            enabled: true,
        };

        assert!(rule.matches("query_latency"));
        assert!(rule.matches("query_count"));
        assert!(!rule.matches("other_metric"));
    }

    #[test]
    fn test_time_window() {
        assert_eq!(TimeWindow::Minute.seconds(), 60);
        assert_eq!(TimeWindow::Hour.seconds(), 3600);
        assert_eq!(TimeWindow::Day.seconds(), 86400);
    }

    #[tokio::test]
    async fn test_aggregator() {
        let aggregator = MetricsAggregator::default();

        let rule = RollupRule {
            name: "test_rule".to_string(),
            metric_pattern: "*".to_string(),
            function: AggregationFunc::Average,
            window: TimeWindow::Minute,
            retention_secs: 3600,
            enabled: true,
        };

        aggregator.add_rule(rule);

        let snapshot = MetricSnapshot {
            name: "test_metric".to_string(),
            help: "Test".to_string(),
            labels: HashMap::new(),
            value: MetricValue::Gauge { value: 42.0 },
            timestamp: Utc::now(),
        };

        aggregator.ingest(&snapshot).unwrap();

        // Wait a bit for aggregation
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
