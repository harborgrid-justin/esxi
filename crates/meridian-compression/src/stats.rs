//! Compression statistics and benchmarking
//!
//! Enterprise-grade metrics collection and performance analysis for compression operations.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use dashmap::DashMap;

/// Compression statistics for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// Original uncompressed size in bytes
    pub original_size: usize,

    /// Compressed size in bytes
    pub compressed_size: usize,

    /// Compression ratio (original_size / compressed_size)
    pub compression_ratio: f64,

    /// Space savings percentage
    pub space_savings_percent: f64,

    /// Time taken for compression
    pub compression_time: Duration,

    /// Compression throughput (MB/s)
    pub throughput_mbps: f64,

    /// Algorithm used
    pub algorithm: String,

    /// Compression level
    pub compression_level: Option<i32>,

    /// Dictionary ID if used
    pub dictionary_id: Option<String>,

    /// Timestamp
    pub timestamp: std::time::SystemTime,
}

impl CompressionStats {
    /// Create new compression statistics
    pub fn new(
        original_size: usize,
        compressed_size: usize,
        compression_time: Duration,
        algorithm: impl Into<String>,
    ) -> Self {
        let compression_ratio = if compressed_size > 0 {
            original_size as f64 / compressed_size as f64
        } else {
            0.0
        };

        let space_savings_percent = if original_size > 0 {
            ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
        } else {
            0.0
        };

        let throughput_mbps = if compression_time.as_secs_f64() > 0.0 {
            (original_size as f64 / 1_000_000.0) / compression_time.as_secs_f64()
        } else {
            0.0
        };

        Self {
            original_size,
            compressed_size,
            compression_ratio,
            space_savings_percent,
            compression_time,
            throughput_mbps,
            algorithm: algorithm.into(),
            compression_level: None,
            dictionary_id: None,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Set compression level
    pub fn with_level(mut self, level: i32) -> Self {
        self.compression_level = Some(level);
        self
    }

    /// Set dictionary ID
    pub fn with_dictionary(mut self, dict_id: impl Into<String>) -> Self {
        self.dictionary_id = Some(dict_id.into());
        self
    }

    /// Check if compression was effective
    pub fn is_effective(&self) -> bool {
        self.compression_ratio > 1.0
    }

    /// Get compression efficiency score (0.0 to 100.0)
    pub fn efficiency_score(&self) -> f64 {
        // Weighted score: 70% ratio, 30% throughput
        let ratio_score = (self.compression_ratio - 1.0).min(10.0) / 10.0 * 70.0;
        let throughput_score = self.throughput_mbps.min(1000.0) / 1000.0 * 30.0;
        ratio_score + throughput_score
    }
}

/// Aggregate statistics for multiple compression operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AggregateStats {
    /// Total number of operations
    pub total_operations: u64,

    /// Total bytes processed (original)
    pub total_bytes_original: u64,

    /// Total bytes compressed
    pub total_bytes_compressed: u64,

    /// Average compression ratio
    pub avg_compression_ratio: f64,

    /// Average throughput (MB/s)
    pub avg_throughput_mbps: f64,

    /// Total compression time
    pub total_compression_time: Duration,

    /// Min compression ratio
    pub min_compression_ratio: f64,

    /// Max compression ratio
    pub max_compression_ratio: f64,

    /// Stats by algorithm
    pub by_algorithm: DashMap<String, AlgorithmStats>,
}

/// Statistics for a specific compression algorithm
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlgorithmStats {
    pub operations: u64,
    pub total_original_bytes: u64,
    pub total_compressed_bytes: u64,
    pub avg_ratio: f64,
    pub avg_throughput: f64,
    pub total_time: Duration,
}

/// Statistics collector with thread-safe operations
#[derive(Debug, Clone)]
pub struct StatsCollector {
    stats: Arc<RwLock<AggregateStats>>,
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsCollector {
    /// Create a new statistics collector
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(AggregateStats {
                min_compression_ratio: f64::MAX,
                max_compression_ratio: f64::MIN,
                ..Default::default()
            })),
        }
    }

    /// Record compression statistics
    pub fn record(&self, stat: &CompressionStats) {
        let mut stats = self.stats.write();

        stats.total_operations += 1;
        stats.total_bytes_original += stat.original_size as u64;
        stats.total_bytes_compressed += stat.compressed_size as u64;
        stats.total_compression_time += stat.compression_time;

        // Update averages
        stats.avg_compression_ratio = (stats.avg_compression_ratio
            * (stats.total_operations - 1) as f64
            + stat.compression_ratio)
            / stats.total_operations as f64;

        stats.avg_throughput_mbps = (stats.avg_throughput_mbps
            * (stats.total_operations - 1) as f64
            + stat.throughput_mbps)
            / stats.total_operations as f64;

        // Update min/max
        stats.min_compression_ratio = stats.min_compression_ratio.min(stat.compression_ratio);
        stats.max_compression_ratio = stats.max_compression_ratio.max(stat.compression_ratio);

        // Update per-algorithm stats
        stats.by_algorithm
            .entry(stat.algorithm.clone())
            .and_modify(|algo_stats| {
                algo_stats.operations += 1;
                algo_stats.total_original_bytes += stat.original_size as u64;
                algo_stats.total_compressed_bytes += stat.compressed_size as u64;
                algo_stats.total_time += stat.compression_time;
                algo_stats.avg_ratio = (algo_stats.avg_ratio
                    * (algo_stats.operations - 1) as f64
                    + stat.compression_ratio)
                    / algo_stats.operations as f64;
                algo_stats.avg_throughput = (algo_stats.avg_throughput
                    * (algo_stats.operations - 1) as f64
                    + stat.throughput_mbps)
                    / algo_stats.operations as f64;
            })
            .or_insert_with(|| AlgorithmStats {
                operations: 1,
                total_original_bytes: stat.original_size as u64,
                total_compressed_bytes: stat.compressed_size as u64,
                avg_ratio: stat.compression_ratio,
                avg_throughput: stat.throughput_mbps,
                total_time: stat.compression_time,
            });
    }

    /// Get current aggregate statistics
    pub fn get_aggregate(&self) -> AggregateStats {
        self.stats.read().clone()
    }

    /// Reset all statistics
    pub fn reset(&self) {
        let mut stats = self.stats.write();
        *stats = AggregateStats {
            min_compression_ratio: f64::MAX,
            max_compression_ratio: f64::MIN,
            ..Default::default()
        };
    }

    /// Get statistics for a specific algorithm
    pub fn get_algorithm_stats(&self, algorithm: &str) -> Option<AlgorithmStats> {
        let stats = self.stats.read();
        stats.by_algorithm.get(algorithm).map(|s| s.clone())
    }
}

/// Performance benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub algorithm: String,
    pub level: Option<i32>,
    pub sample_size: usize,
    pub iterations: u32,
    pub min_time: Duration,
    pub max_time: Duration,
    pub avg_time: Duration,
    pub median_time: Duration,
    pub std_dev: Duration,
    pub throughput_mbps: f64,
    pub compression_ratio: f64,
}

/// Benchmark runner for comparing compression algorithms
pub struct BenchmarkRunner {
    sample_data: Vec<u8>,
    iterations: u32,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner with sample data
    pub fn new(sample_data: Vec<u8>, iterations: u32) -> Self {
        Self {
            sample_data,
            iterations,
        }
    }

    /// Run a benchmark with a compression function
    pub fn run<F>(&self, algorithm: &str, compress_fn: F) -> BenchmarkResult
    where
        F: Fn(&[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>>,
    {
        let mut times = Vec::with_capacity(self.iterations as usize);
        let mut compressed_size = 0;

        for _ in 0..self.iterations {
            let start = Instant::now();
            let compressed = compress_fn(&self.sample_data)
                .expect("Compression should succeed in benchmark");
            let elapsed = start.elapsed();

            times.push(elapsed);
            compressed_size = compressed.len();
        }

        times.sort();

        let min_time = *times.first().unwrap();
        let max_time = *times.last().unwrap();
        let avg_time = times.iter().sum::<Duration>() / self.iterations;
        let median_time = times[times.len() / 2];

        // Calculate standard deviation
        let variance: f64 = times.iter()
            .map(|t| {
                let diff = t.as_secs_f64() - avg_time.as_secs_f64();
                diff * diff
            })
            .sum::<f64>() / self.iterations as f64;
        let std_dev = Duration::from_secs_f64(variance.sqrt());

        let throughput_mbps = (self.sample_data.len() as f64 / 1_000_000.0)
            / avg_time.as_secs_f64();

        let compression_ratio = self.sample_data.len() as f64 / compressed_size as f64;

        BenchmarkResult {
            algorithm: algorithm.to_string(),
            level: None,
            sample_size: self.sample_data.len(),
            iterations: self.iterations,
            min_time,
            max_time,
            avg_time,
            median_time,
            std_dev,
            throughput_mbps,
            compression_ratio,
        }
    }

    /// Compare multiple algorithms
    pub fn compare<F>(&self, algorithms: Vec<(&str, F)>) -> Vec<BenchmarkResult>
    where
        F: Fn(&[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>>,
    {
        algorithms
            .into_iter()
            .map(|(name, compress_fn)| self.run(name, compress_fn))
            .collect()
    }
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    /// Start a new timer
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            name: name.into(),
        }
    }

    /// Stop the timer and return elapsed duration
    pub fn stop(self) -> Duration {
        let elapsed = self.start.elapsed();
        tracing::debug!(
            timer = %self.name,
            elapsed_ms = elapsed.as_millis(),
            "Timer stopped"
        );
        elapsed
    }

    /// Get elapsed time without stopping
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_stats() {
        let stats = CompressionStats::new(
            1000,
            500,
            Duration::from_millis(100),
            "lz4",
        );

        assert_eq!(stats.compression_ratio, 2.0);
        assert_eq!(stats.space_savings_percent, 50.0);
        assert!(stats.is_effective());
    }

    #[test]
    fn test_stats_collector() {
        let collector = StatsCollector::new();

        let stat1 = CompressionStats::new(1000, 500, Duration::from_millis(100), "lz4");
        let stat2 = CompressionStats::new(2000, 1000, Duration::from_millis(200), "zstd");

        collector.record(&stat1);
        collector.record(&stat2);

        let aggregate = collector.get_aggregate();
        assert_eq!(aggregate.total_operations, 2);
        assert_eq!(aggregate.total_bytes_original, 3000);
    }

    #[test]
    fn test_efficiency_score() {
        let stats = CompressionStats::new(
            1000,
            500,
            Duration::from_millis(10),
            "test",
        );

        let score = stats.efficiency_score();
        assert!(score > 0.0 && score <= 100.0);
    }
}
