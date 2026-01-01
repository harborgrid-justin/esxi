//! Adaptive compression with automatic algorithm selection
//!
//! Intelligently selects the best compression algorithm based on data characteristics.

use crate::error::{CompressionError, Result};
use crate::stats::CompressionStats;
use crate::{CompressionAlgorithm, Compressor};
use crate::{
    lz4::Lz4Compressor,
    zstd::ZstdCompressor,
    brotli::BrotliCompressor,
    gzip::GzipCompressor,
    snappy::SnappyCompressor,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Data characteristics for algorithm selection
#[derive(Debug, Clone)]
pub struct DataCharacteristics {
    /// Size of the data in bytes
    pub size: usize,
    /// Estimated entropy (0.0 - 1.0)
    pub entropy: f64,
    /// Repetition ratio (0.0 - 1.0)
    pub repetition: f64,
    /// Data type hint
    pub data_type: DataType,
    /// Performance priority
    pub priority: CompressionPriority,
}

/// Data type hints for optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// Plain text
    Text,
    /// Binary data
    Binary,
    /// JSON/XML structured data
    Structured,
    /// Already compressed data
    Compressed,
    /// Multimedia data
    Multimedia,
    /// Unknown type
    Unknown,
}

/// Compression priority
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionPriority {
    /// Maximize speed
    Speed,
    /// Balance speed and ratio
    Balanced,
    /// Maximize compression ratio
    Ratio,
}

impl DataCharacteristics {
    /// Analyze data and extract characteristics
    pub fn analyze(data: &[u8]) -> Self {
        let size = data.len();
        let entropy = Self::calculate_entropy(data);
        let repetition = Self::calculate_repetition(data);
        let data_type = Self::detect_data_type(data);

        Self {
            size,
            entropy,
            repetition,
            data_type,
            priority: CompressionPriority::Balanced,
        }
    }

    /// Calculate Shannon entropy
    fn calculate_entropy(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut counts = [0u32; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in &counts {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy / 8.0 // Normalize to 0-1
    }

    /// Calculate repetition ratio
    fn calculate_repetition(data: &[u8]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let mut repetitions = 0;
        for i in 1..data.len().min(1000) {
            if data[i] == data[i - 1] {
                repetitions += 1;
            }
        }

        repetitions as f64 / data.len().min(1000) as f64
    }

    /// Detect probable data type
    fn detect_data_type(data: &[u8]) -> DataType {
        if data.is_empty() {
            return DataType::Unknown;
        }

        // Check for common compression headers
        if Self::is_likely_compressed(data) {
            return DataType::Compressed;
        }

        // Check for text (printable ASCII + common control chars)
        let printable_count = data.iter()
            .take(1000)
            .filter(|&&b| b == b'\n' || b == b'\r' || b == b'\t' || (b >= 32 && b < 127))
            .count();

        if printable_count > 900 {
            // Check for structured data
            if data.iter().any(|&b| b == b'{' || b == b'<') {
                return DataType::Structured;
            }
            return DataType::Text;
        }

        // Check for multimedia headers
        if Self::is_multimedia(data) {
            return DataType::Multimedia;
        }

        DataType::Binary
    }

    /// Check if data is likely already compressed
    fn is_likely_compressed(data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }

        // Check for common compression magic numbers
        matches!(
            &data[..2],
            b"\x1f\x8b" | // gzip
            b"\x42\x5a" | // bzip2
            b"\x50\x4b" | // zip
            b"\xfd\x37"    // xz
        ) || (data.len() >= 4 && &data[..4] == b"\x28\xb5\x2f\xfd") // zstd
    }

    /// Check if data is multimedia
    fn is_multimedia(data: &[u8]) -> bool {
        if data.len() < 4 {
            return false;
        }

        matches!(
            &data[..4],
            b"\xff\xd8\xff\xe0" | // JPEG
            b"\x89PNG" |           // PNG
            b"RIFF" |              // WAV/AVI
            b"ftyp"                // MP4
        ) || matches!(&data[..3], b"\xff\xfb" | b"ID3") // MP3
    }

    /// Set compression priority
    pub fn with_priority(mut self, priority: CompressionPriority) -> Self {
        self.priority = priority;
        self
    }
}

/// Adaptive compressor that selects best algorithm
pub struct AdaptiveCompressor {
    /// Enable algorithm benchmarking
    enable_benchmarking: bool,
    /// Maximum time for benchmarking (milliseconds)
    benchmark_timeout_ms: u64,
    /// Minimum data size for adaptive selection
    min_adaptive_size: usize,
}

impl Default for AdaptiveCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveCompressor {
    /// Create a new adaptive compressor
    pub fn new() -> Self {
        Self {
            enable_benchmarking: false,
            benchmark_timeout_ms: 100,
            min_adaptive_size: 1024, // 1 KB
        }
    }

    /// Create with benchmarking enabled
    pub fn with_benchmarking(benchmark_timeout_ms: u64) -> Self {
        Self {
            enable_benchmarking: true,
            benchmark_timeout_ms,
            min_adaptive_size: 1024,
        }
    }

    /// Select best algorithm based on data characteristics
    pub fn select_algorithm(&self, characteristics: &DataCharacteristics) -> CompressionAlgorithm {
        // For very small data, use fast algorithm
        if characteristics.size < self.min_adaptive_size {
            return CompressionAlgorithm::Snappy;
        }

        // Already compressed data - use fast algorithm
        if characteristics.data_type == DataType::Compressed {
            return CompressionAlgorithm::Snappy;
        }

        // Multimedia data - usually doesn't compress well
        if characteristics.data_type == DataType::Multimedia {
            return CompressionAlgorithm::Snappy;
        }

        match characteristics.priority {
            CompressionPriority::Speed => {
                if characteristics.repetition > 0.3 {
                    CompressionAlgorithm::Lz4
                } else {
                    CompressionAlgorithm::Snappy
                }
            }
            CompressionPriority::Balanced => {
                if characteristics.data_type == DataType::Text
                    || characteristics.data_type == DataType::Structured
                {
                    CompressionAlgorithm::Zstd
                } else if characteristics.repetition > 0.2 {
                    CompressionAlgorithm::Lz4
                } else {
                    CompressionAlgorithm::Zstd
                }
            }
            CompressionPriority::Ratio => {
                if characteristics.data_type == DataType::Text
                    || characteristics.data_type == DataType::Structured
                {
                    CompressionAlgorithm::Brotli
                } else if characteristics.entropy < 0.7 {
                    CompressionAlgorithm::Zstd
                } else {
                    CompressionAlgorithm::Brotli
                }
            }
        }
    }

    /// Compress with automatically selected algorithm
    pub fn compress_with_best_algorithm(&self, data: &[u8]) -> Result<Vec<u8>> {
        let characteristics = DataCharacteristics::analyze(data);
        let algorithm = self.select_algorithm(&characteristics);

        self.compress_with_algorithm(data, algorithm)
    }

    /// Compress with specific algorithm
    fn compress_with_algorithm(
        &self,
        data: &[u8],
        algorithm: CompressionAlgorithm,
    ) -> Result<Vec<u8>> {
        match algorithm {
            CompressionAlgorithm::Lz4 => Lz4Compressor::default().compress(data),
            CompressionAlgorithm::Zstd => ZstdCompressor::default().compress(data),
            CompressionAlgorithm::Brotli => BrotliCompressor::default().compress(data),
            CompressionAlgorithm::Gzip => GzipCompressor::default().compress(data),
            CompressionAlgorithm::Snappy => SnappyCompressor::default().compress(data),
            CompressionAlgorithm::Adaptive => {
                self.compress_with_best_algorithm(data)
            }
        }
    }

    /// Benchmark multiple algorithms and select best
    pub fn compress_with_benchmark(&self, data: &[u8]) -> Result<AdaptiveCompressionResult> {
        let characteristics = DataCharacteristics::analyze(data);

        let algorithms = match characteristics.priority {
            CompressionPriority::Speed => {
                vec![
                    CompressionAlgorithm::Snappy,
                    CompressionAlgorithm::Lz4,
                ]
            }
            CompressionPriority::Balanced => {
                vec![
                    CompressionAlgorithm::Lz4,
                    CompressionAlgorithm::Zstd,
                    CompressionAlgorithm::Snappy,
                ]
            }
            CompressionPriority::Ratio => {
                vec![
                    CompressionAlgorithm::Zstd,
                    CompressionAlgorithm::Brotli,
                    CompressionAlgorithm::Gzip,
                ]
            }
        };

        let mut best_result: Option<(CompressionAlgorithm, Vec<u8>, CompressionStats)> = None;
        let mut best_score = 0.0;

        for algorithm in algorithms {
            let start = std::time::Instant::now();
            let compressed = match self.compress_with_algorithm(data, algorithm) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let elapsed = start.elapsed();

            if elapsed.as_millis() as u64 > self.benchmark_timeout_ms {
                continue; // Skip if too slow
            }

            let stats = CompressionStats::new(
                data.len(),
                compressed.len(),
                elapsed,
                format!("{:?}", algorithm),
            );

            // Calculate score based on priority
            let score = match characteristics.priority {
                CompressionPriority::Speed => {
                    // Prioritize throughput
                    stats.throughput_mbps * 0.7 + stats.compression_ratio * 0.3
                }
                CompressionPriority::Balanced => {
                    // Equal weight
                    stats.compression_ratio * 0.5
                        + (stats.throughput_mbps / 100.0).min(1.0) * 0.5
                }
                CompressionPriority::Ratio => {
                    // Prioritize ratio
                    stats.compression_ratio * 0.8
                        + (stats.throughput_mbps / 100.0).min(1.0) * 0.2
                }
            };

            if score > best_score {
                best_score = score;
                best_result = Some((algorithm, compressed, stats));
            }
        }

        let (algorithm, compressed, stats) = best_result.ok_or_else(|| {
            CompressionError::Adaptive("No algorithm succeeded".to_string())
        })?;

        Ok(AdaptiveCompressionResult {
            algorithm,
            compressed,
            stats,
            characteristics,
        })
    }
}

/// Result of adaptive compression
#[derive(Debug)]
pub struct AdaptiveCompressionResult {
    /// Selected algorithm
    pub algorithm: CompressionAlgorithm,
    /// Compressed data
    pub compressed: Vec<u8>,
    /// Compression statistics
    pub stats: CompressionStats,
    /// Data characteristics
    pub characteristics: DataCharacteristics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_characteristics() {
        let text_data = b"Hello, World! This is a test of adaptive compression.";
        let chars = DataCharacteristics::analyze(text_data);

        assert_eq!(chars.data_type, DataType::Text);
        assert!(chars.entropy > 0.0);
    }

    #[test]
    fn test_binary_detection() {
        let binary_data = vec![0u8, 1, 2, 3, 255, 254, 253];
        let chars = DataCharacteristics::analyze(&binary_data);

        assert_eq!(chars.data_type, DataType::Binary);
    }

    #[test]
    fn test_compressed_detection() {
        let gzip_header = b"\x1f\x8b\x08\x00\x00\x00\x00\x00";
        let chars = DataCharacteristics::analyze(gzip_header);

        assert_eq!(chars.data_type, DataType::Compressed);
    }

    #[test]
    fn test_algorithm_selection() {
        let compressor = AdaptiveCompressor::new();

        // Small data -> Snappy
        let small_chars = DataCharacteristics {
            size: 500,
            entropy: 0.5,
            repetition: 0.1,
            data_type: DataType::Binary,
            priority: CompressionPriority::Balanced,
        };
        assert_eq!(
            compressor.select_algorithm(&small_chars),
            CompressionAlgorithm::Snappy
        );

        // Text with ratio priority -> Brotli
        let text_chars = DataCharacteristics {
            size: 10000,
            entropy: 0.6,
            repetition: 0.2,
            data_type: DataType::Text,
            priority: CompressionPriority::Ratio,
        };
        assert_eq!(
            compressor.select_algorithm(&text_chars),
            CompressionAlgorithm::Brotli
        );

        // Speed priority -> Snappy or LZ4
        let speed_chars = DataCharacteristics {
            size: 10000,
            entropy: 0.5,
            repetition: 0.1,
            data_type: DataType::Binary,
            priority: CompressionPriority::Speed,
        };
        let algo = compressor.select_algorithm(&speed_chars);
        assert!(
            algo == CompressionAlgorithm::Snappy || algo == CompressionAlgorithm::Lz4
        );
    }

    #[test]
    fn test_adaptive_compression() {
        let compressor = AdaptiveCompressor::new();

        let text_data = b"The quick brown fox jumps over the lazy dog. " as &[u8];
        let data = text_data.repeat(100);

        let compressed = compressor.compress_with_best_algorithm(&data).unwrap();

        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_benchmark_compression() {
        let compressor = AdaptiveCompressor::with_benchmarking(1000);

        let data = b"Test data for benchmarking" as &[u8];
        let data = data.repeat(100);

        let result = compressor.compress_with_benchmark(&data).unwrap();

        assert!(result.compressed.len() < data.len());
        assert!(result.stats.is_effective());
    }

    #[test]
    fn test_entropy_calculation() {
        // Uniform data - low entropy
        let uniform = vec![0u8; 1000];
        let chars = DataCharacteristics::analyze(&uniform);
        assert!(chars.entropy < 0.1);

        // Random data - high entropy
        let random: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let chars = DataCharacteristics::analyze(&random);
        assert!(chars.entropy > 0.3);
    }

    #[test]
    fn test_repetition_calculation() {
        // High repetition
        let repeated = vec![0u8; 1000];
        let chars = DataCharacteristics::analyze(&repeated);
        assert!(chars.repetition > 0.9);

        // Low repetition
        let varied: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let chars = DataCharacteristics::analyze(&varied);
        assert!(chars.repetition < 0.1);
    }
}
