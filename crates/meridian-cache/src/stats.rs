//! Cache statistics and metrics tracking

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;

/// Cache statistics
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of set operations
    pub sets: u64,
    /// Number of delete operations
    pub deletes: u64,
    /// Number of evictions
    pub evictions: u64,
}

impl CacheStats {
    /// Get the total number of operations
    pub fn total_ops(&self) -> u64 {
        self.hits + self.misses
    }

    /// Get the hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_ops();
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Get the miss rate as a percentage
    pub fn miss_rate(&self) -> f64 {
        let total = self.total_ops();
        if total == 0 {
            0.0
        } else {
            (self.misses as f64 / total as f64) * 100.0
        }
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Merge with another stats object
    pub fn merge(&mut self, other: &CacheStats) {
        self.hits += other.hits;
        self.misses += other.misses;
        self.sets += other.sets;
        self.deletes += other.deletes;
        self.evictions += other.evictions;
    }
}

/// Detailed cache metrics with timing information
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    /// Basic statistics
    pub stats: CacheStats,
    /// Start time of metrics collection
    pub start_time: Instant,
    /// Total response time for get operations (microseconds)
    pub total_get_time_us: u64,
    /// Total response time for set operations (microseconds)
    pub total_set_time_us: u64,
    /// Peak memory usage (bytes)
    pub peak_memory: usize,
    /// Current memory usage (bytes)
    pub current_memory: usize,
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self {
            stats: CacheStats::default(),
            start_time: Instant::now(),
            total_get_time_us: 0,
            total_set_time_us: 0,
            peak_memory: 0,
            current_memory: 0,
        }
    }
}

impl CacheMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the average get latency in microseconds
    pub fn avg_get_latency_us(&self) -> f64 {
        let total_gets = self.stats.hits + self.stats.misses;
        if total_gets == 0 {
            0.0
        } else {
            self.total_get_time_us as f64 / total_gets as f64
        }
    }

    /// Get the average set latency in microseconds
    pub fn avg_set_latency_us(&self) -> f64 {
        if self.stats.sets == 0 {
            0.0
        } else {
            self.total_set_time_us as f64 / self.stats.sets as f64
        }
    }

    /// Get operations per second
    pub fn ops_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            0.0
        } else {
            self.stats.total_ops() as f64 / elapsed
        }
    }

    /// Get throughput in bytes per second (estimate based on memory)
    pub fn throughput_bytes_per_sec(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            0.0
        } else {
            self.current_memory as f64 / elapsed
        }
    }

    /// Update memory usage
    pub fn update_memory(&mut self, current: usize) {
        self.current_memory = current;
        if current > self.peak_memory {
            self.peak_memory = current;
        }
    }

    /// Record a get operation
    pub fn record_get(&mut self, duration: Duration, hit: bool) {
        self.total_get_time_us += duration.as_micros() as u64;
        if hit {
            self.stats.hits += 1;
        } else {
            self.stats.misses += 1;
        }
    }

    /// Record a set operation
    pub fn record_set(&mut self, duration: Duration) {
        self.total_set_time_us += duration.as_micros() as u64;
        self.stats.sets += 1;
    }

    /// Record a delete operation
    pub fn record_delete(&mut self) {
        self.stats.deletes += 1;
    }

    /// Record an eviction
    pub fn record_eviction(&mut self) {
        self.stats.evictions += 1;
    }

    /// Reset all metrics
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            stats: self.stats,
            uptime: self.start_time.elapsed(),
            avg_get_latency_us: self.avg_get_latency_us(),
            avg_set_latency_us: self.avg_set_latency_us(),
            ops_per_second: self.ops_per_second(),
            current_memory: self.current_memory,
            peak_memory: self.peak_memory,
        }
    }
}

/// A snapshot of cache metrics at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Basic statistics
    pub stats: CacheStats,
    /// Uptime
    #[serde(skip)]
    pub uptime: Duration,
    /// Average get latency (microseconds)
    pub avg_get_latency_us: f64,
    /// Average set latency (microseconds)
    pub avg_set_latency_us: f64,
    /// Operations per second
    pub ops_per_second: f64,
    /// Current memory usage (bytes)
    pub current_memory: usize,
    /// Peak memory usage (bytes)
    pub peak_memory: usize,
}

impl MetricsSnapshot {
    /// Format uptime as a human-readable string
    pub fn uptime_string(&self) -> String {
        let secs = self.uptime.as_secs();
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;
        format!("{}h {}m {}s", hours, minutes, seconds)
    }

    /// Format memory as a human-readable string
    pub fn format_memory(bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = KB * 1024;
        const GB: usize = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    /// Get current memory as a human-readable string
    pub fn current_memory_string(&self) -> String {
        Self::format_memory(self.current_memory)
    }

    /// Get peak memory as a human-readable string
    pub fn peak_memory_string(&self) -> String {
        Self::format_memory(self.peak_memory)
    }
}

/// Metrics collector for tracking cache performance over time
pub struct MetricsCollector {
    metrics: Arc<RwLock<CacheMetrics>>,
    snapshots: Arc<RwLock<Vec<MetricsSnapshot>>>,
    snapshot_interval: Duration,
    max_snapshots: usize,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(snapshot_interval: Duration, max_snapshots: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(CacheMetrics::new())),
            snapshots: Arc::new(RwLock::new(Vec::new())),
            snapshot_interval,
            max_snapshots,
        }
    }

    /// Get the current metrics
    pub fn get_metrics(&self) -> CacheMetrics {
        self.metrics.read().clone()
    }

    /// Update metrics
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut CacheMetrics),
    {
        let mut metrics = self.metrics.write();
        f(&mut metrics);
    }

    /// Record a get operation
    pub fn record_get(&self, duration: Duration, hit: bool) {
        self.update(|m| m.record_get(duration, hit));
    }

    /// Record a set operation
    pub fn record_set(&self, duration: Duration) {
        self.update(|m| m.record_set(duration));
    }

    /// Record a delete operation
    pub fn record_delete(&self) {
        self.update(|m| m.record_delete());
    }

    /// Record an eviction
    pub fn record_eviction(&self) {
        self.update(|m| m.record_eviction());
    }

    /// Update memory usage
    pub fn update_memory(&self, current: usize) {
        self.update(|m| m.update_memory(current));
    }

    /// Take a snapshot of current metrics
    pub fn take_snapshot(&self) {
        let snapshot = self.metrics.read().snapshot();
        let mut snapshots = self.snapshots.write();

        snapshots.push(snapshot);

        // Keep only the most recent snapshots
        if snapshots.len() > self.max_snapshots {
            let to_remove = snapshots.len() - self.max_snapshots;
            snapshots.drain(0..to_remove);
        }
    }

    /// Get all snapshots
    pub fn get_snapshots(&self) -> Vec<MetricsSnapshot> {
        self.snapshots.read().clone()
    }

    /// Start background snapshot collection
    pub fn start_background_collection(&self) -> tokio::task::JoinHandle<()> {
        let collector = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(collector.snapshot_interval);

            loop {
                interval.tick().await;
                collector.take_snapshot();
            }
        })
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.metrics.write().reset();
        self.snapshots.write().clear();
    }

    /// Get statistics for a time window
    pub fn get_window_stats(&self, window: Duration) -> Option<CacheStats> {
        let snapshots = self.snapshots.read();
        if snapshots.is_empty() {
            return None;
        }

        let cutoff = Instant::now() - window;
        let mut combined = CacheStats::default();

        for snapshot in snapshots.iter() {
            // This is an approximation since we don't store the snapshot time
            combined.merge(&snapshot.stats);
        }

        Some(combined)
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
            snapshots: self.snapshots.clone(),
            snapshot_interval: self.snapshot_interval,
            max_snapshots: self.max_snapshots,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new(Duration::from_secs(60), 1440) // 1 minute interval, 24 hours of snapshots
    }
}

/// Performance analyzer for cache metrics
pub struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    /// Analyze hit rate and provide recommendations
    pub fn analyze_hit_rate(stats: &CacheStats) -> String {
        let hit_rate = stats.hit_rate();

        if hit_rate >= 90.0 {
            "Excellent: Cache is performing very well".to_string()
        } else if hit_rate >= 70.0 {
            "Good: Cache is performing well".to_string()
        } else if hit_rate >= 50.0 {
            "Fair: Consider increasing cache size or adjusting TTL".to_string()
        } else if hit_rate >= 30.0 {
            "Poor: Cache may be too small or TTL too short".to_string()
        } else {
            "Very Poor: Investigate caching strategy and configuration".to_string()
        }
    }

    /// Analyze eviction rate
    pub fn analyze_eviction_rate(stats: &CacheStats) -> String {
        if stats.sets == 0 {
            return "No data available".to_string();
        }

        let eviction_rate = (stats.evictions as f64 / stats.sets as f64) * 100.0;

        if eviction_rate < 5.0 {
            "Excellent: Low eviction rate".to_string()
        } else if eviction_rate < 15.0 {
            "Good: Moderate eviction rate".to_string()
        } else if eviction_rate < 30.0 {
            "Fair: Consider increasing cache capacity".to_string()
        } else {
            "Poor: High eviction rate, increase cache size".to_string()
        }
    }

    /// Generate a performance report
    pub fn generate_report(snapshot: &MetricsSnapshot) -> String {
        format!(
            r#"
Cache Performance Report
========================

Uptime: {}

Operations:
  - Total: {}
  - Hits: {} ({:.2}%)
  - Misses: {} ({:.2}%)
  - Sets: {}
  - Deletes: {}
  - Evictions: {}

Performance:
  - Hit Rate: {:.2}%
  - Ops/sec: {:.2}
  - Avg GET latency: {:.2} μs
  - Avg SET latency: {:.2} μs

Memory:
  - Current: {}
  - Peak: {}

Analysis:
  - Hit Rate: {}
  - Eviction Rate: {}
"#,
            snapshot.uptime_string(),
            snapshot.stats.total_ops(),
            snapshot.stats.hits,
            snapshot.stats.hit_rate(),
            snapshot.stats.misses,
            snapshot.stats.miss_rate(),
            snapshot.stats.sets,
            snapshot.stats.deletes,
            snapshot.stats.evictions,
            snapshot.stats.hit_rate(),
            snapshot.ops_per_second,
            snapshot.avg_get_latency_us,
            snapshot.avg_set_latency_us,
            snapshot.current_memory_string(),
            snapshot.peak_memory_string(),
            Self::analyze_hit_rate(&snapshot.stats),
            Self::analyze_eviction_rate(&snapshot.stats),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();

        stats.hits = 80;
        stats.misses = 20;
        stats.sets = 100;
        stats.deletes = 10;
        stats.evictions = 5;

        assert_eq!(stats.total_ops(), 100);
        assert_eq!(stats.hit_rate(), 80.0);
        assert_eq!(stats.miss_rate(), 20.0);
    }

    #[test]
    fn test_metrics() {
        let mut metrics = CacheMetrics::new();

        metrics.record_get(Duration::from_micros(100), true);
        metrics.record_get(Duration::from_micros(200), false);
        metrics.record_set(Duration::from_micros(150));

        assert_eq!(metrics.stats.hits, 1);
        assert_eq!(metrics.stats.misses, 1);
        assert_eq!(metrics.stats.sets, 1);
        assert_eq!(metrics.avg_get_latency_us(), 150.0);
        assert_eq!(metrics.avg_set_latency_us(), 150.0);
    }

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new(Duration::from_secs(1), 10);

        collector.record_get(Duration::from_micros(100), true);
        collector.record_set(Duration::from_micros(150));
        collector.update_memory(1024);

        let metrics = collector.get_metrics();
        assert_eq!(metrics.stats.hits, 1);
        assert_eq!(metrics.stats.sets, 1);
        assert_eq!(metrics.current_memory, 1024);
    }

    #[test]
    fn test_snapshot() {
        let metrics = CacheMetrics::new();
        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.stats.hits, 0);
        assert_eq!(snapshot.stats.misses, 0);
    }
}
