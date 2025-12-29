//! Pipeline performance metrics and monitoring.

use crate::BatchStats;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Pipeline performance metrics.
pub struct PipelineMetrics {
    /// Start time of the pipeline.
    start_time: RwLock<Option<Instant>>,
    /// End time of the pipeline.
    end_time: RwLock<Option<Instant>>,
    /// Stage-level metrics.
    stage_metrics: RwLock<HashMap<String, StageMetrics>>,
    /// Overall batch statistics.
    batch_stats: RwLock<BatchStats>,
    /// Custom counters.
    counters: RwLock<HashMap<String, u64>>,
    /// Custom gauges.
    gauges: RwLock<HashMap<String, f64>>,
}

/// Metrics for a pipeline stage.
#[derive(Debug, Clone)]
pub struct StageMetrics {
    /// Stage name.
    pub name: String,
    /// Number of records processed.
    pub records_processed: u64,
    /// Number of records failed.
    pub records_failed: u64,
    /// Processing duration.
    pub duration: Duration,
    /// Start time.
    pub start_time: Option<Instant>,
    /// End time.
    pub end_time: Option<Instant>,
}

impl StageMetrics {
    /// Create new stage metrics.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            records_processed: 0,
            records_failed: 0,
            duration: Duration::ZERO,
            start_time: None,
            end_time: None,
        }
    }

    /// Record processing start.
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Record processing end.
    pub fn end(&mut self) {
        self.end_time = Some(Instant::now());
        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            self.duration = end.duration_since(start);
        }
    }

    /// Get throughput in records per second.
    pub fn throughput(&self) -> f64 {
        if self.duration.as_secs_f64() == 0.0 {
            0.0
        } else {
            self.records_processed as f64 / self.duration.as_secs_f64()
        }
    }

    /// Get success rate as percentage.
    pub fn success_rate(&self) -> f64 {
        if self.records_processed == 0 {
            0.0
        } else {
            let successful = self.records_processed - self.records_failed;
            (successful as f64 / self.records_processed as f64) * 100.0
        }
    }
}

impl PipelineMetrics {
    /// Create new pipeline metrics.
    pub fn new() -> Self {
        Self {
            start_time: RwLock::new(None),
            end_time: RwLock::new(None),
            stage_metrics: RwLock::new(HashMap::new()),
            batch_stats: RwLock::new(BatchStats::new()),
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
        }
    }

    /// Record pipeline start.
    pub fn start(&self) {
        *self.start_time.write() = Some(Instant::now());
    }

    /// Record pipeline end.
    pub fn end(&self) {
        *self.end_time.write() = Some(Instant::now());
    }

    /// Get pipeline duration.
    pub fn duration(&self) -> Option<Duration> {
        let start = *self.start_time.read();
        let end = *self.end_time.read();

        match (start, end) {
            (Some(s), Some(e)) => Some(e.duration_since(s)),
            (Some(s), None) => Some(Instant::now().duration_since(s)),
            _ => None,
        }
    }

    /// Get or create stage metrics.
    pub fn get_stage(&self, stage_name: &str) -> StageMetrics {
        let metrics = self.stage_metrics.read();
        metrics
            .get(stage_name)
            .cloned()
            .unwrap_or_else(|| StageMetrics::new(stage_name))
    }

    /// Update stage metrics.
    pub fn update_stage(&self, stage_name: impl Into<String>, metrics: StageMetrics) {
        let mut stages = self.stage_metrics.write();
        stages.insert(stage_name.into(), metrics);
    }

    /// Record records processed for a stage.
    pub fn record_processed(&self, stage_name: &str, count: u64) {
        let mut metrics = self.get_stage(stage_name);
        metrics.records_processed += count;
        self.update_stage(stage_name, metrics);

        // Update overall stats
        let mut stats = self.batch_stats.write();
        stats.records_processed += count as usize;
    }

    /// Record failed records for a stage.
    pub fn record_failed(&self, stage_name: &str, count: u64) {
        let mut metrics = self.get_stage(stage_name);
        metrics.records_failed += count;
        self.update_stage(stage_name, metrics);

        // Update overall stats
        let mut stats = self.batch_stats.write();
        stats.records_failed += count as usize;
    }

    /// Start a stage.
    pub fn start_stage(&self, stage_name: &str) {
        let mut metrics = self.get_stage(stage_name);
        metrics.start();
        self.update_stage(stage_name, metrics);
    }

    /// End a stage.
    pub fn end_stage(&self, stage_name: &str) {
        let mut metrics = self.get_stage(stage_name);
        metrics.end();
        self.update_stage(stage_name, metrics);
    }

    /// Get all stage metrics.
    pub fn get_all_stages(&self) -> Vec<StageMetrics> {
        self.stage_metrics.read().values().cloned().collect()
    }

    /// Increment a counter.
    pub fn increment_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.write();
        *counters.entry(name.to_string()).or_insert(0) += value;
    }

    /// Set a gauge value.
    pub fn set_gauge(&self, name: &str, value: f64) {
        let mut gauges = self.gauges.write();
        gauges.insert(name.to_string(), value);
    }

    /// Get counter value.
    pub fn get_counter(&self, name: &str) -> u64 {
        *self.counters.read().get(name).unwrap_or(&0)
    }

    /// Get gauge value.
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.read().get(name).copied()
    }

    /// Get overall batch statistics.
    pub fn get_stats(&self) -> BatchStats {
        self.batch_stats.read().clone()
    }

    /// Update batch statistics.
    pub fn update_stats(&self, stats: BatchStats) {
        let mut current_stats = self.batch_stats.write();
        current_stats.merge(&stats);
    }

    /// Export metrics as JSON.
    pub fn to_json(&self) -> serde_json::Value {
        let stages: Vec<_> = self
            .get_all_stages()
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "records_processed": s.records_processed,
                    "records_failed": s.records_failed,
                    "duration_secs": s.duration.as_secs_f64(),
                    "throughput": s.throughput(),
                    "success_rate": s.success_rate(),
                })
            })
            .collect();

        let counters: HashMap<_, _> = self.counters.read().clone();
        let gauges: HashMap<_, _> = self.gauges.read().clone();

        serde_json::json!({
            "duration_secs": self.duration().map(|d| d.as_secs_f64()),
            "stages": stages,
            "counters": counters,
            "gauges": gauges,
            "stats": {
                "records_processed": self.get_stats().records_processed,
                "records_filtered": self.get_stats().records_filtered,
                "records_failed": self.get_stats().records_failed,
                "success_rate": self.get_stats().success_rate(),
                "throughput": self.get_stats().throughput(),
            }
        })
    }
}

impl Default for PipelineMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_pipeline_metrics() {
        let metrics = PipelineMetrics::new();

        metrics.start();
        thread::sleep(Duration::from_millis(10));
        metrics.end();

        let duration = metrics.duration();
        assert!(duration.is_some());
        assert!(duration.unwrap() >= Duration::from_millis(10));
    }

    #[test]
    fn test_stage_metrics() {
        let metrics = PipelineMetrics::new();

        metrics.start_stage("transform");
        metrics.record_processed("transform", 100);
        metrics.record_failed("transform", 5);
        metrics.end_stage("transform");

        let stage = metrics.get_stage("transform");
        assert_eq!(stage.records_processed, 100);
        assert_eq!(stage.records_failed, 5);
        assert_eq!(stage.success_rate(), 95.0);
    }

    #[test]
    fn test_counters_and_gauges() {
        let metrics = PipelineMetrics::new();

        metrics.increment_counter("api_calls", 5);
        metrics.increment_counter("api_calls", 3);
        metrics.set_gauge("memory_usage", 256.5);

        assert_eq!(metrics.get_counter("api_calls"), 8);
        assert_eq!(metrics.get_gauge("memory_usage"), Some(256.5));
    }

    #[test]
    fn test_metrics_json_export() {
        let metrics = PipelineMetrics::new();
        metrics.start();
        metrics.record_processed("source", 100);
        metrics.end();

        let json = metrics.to_json();
        assert!(json.get("duration_secs").is_some());
        assert!(json.get("stats").is_some());
    }
}
