//! Meridian Compression - Advanced Compression Algorithms Suite
//!
//! Enterprise-grade compression library for $983M SaaS Platform v0.5
//!
//! # Features
//!
//! - **Multiple Algorithms**: LZ4, Zstandard, Brotli, Gzip, Snappy
//! - **Streaming Support**: Efficient handling of large files
//! - **Dictionary Training**: Optimized compression for similar data
//! - **Adaptive Compression**: Automatic algorithm selection
//! - **Pipeline Processing**: Chain multiple compression stages
//! - **Delta Compression**: Efficient versioning support
//! - **Comprehensive Stats**: Detailed performance metrics
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use meridian_compression::{Compressor, CompressionAlgorithm};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let data = b"Hello, Enterprise SaaS Platform!";
//!
//! // Simple compression
//! let compressed = Compressor::compress(data, CompressionAlgorithm::Lz4)?;
//! let decompressed = Compressor::decompress(&compressed, CompressionAlgorithm::Lz4)?;
//!
//! assert_eq!(data.as_slice(), decompressed.as_slice());
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod stats;
pub mod lz4;
pub mod zstd;
pub mod brotli;
pub mod gzip;
pub mod snappy;
pub mod dictionary;
pub mod delta;
pub mod streaming;
pub mod adaptive;
pub mod pipeline;

// Re-exports
pub use error::{CompressionError, Result};
pub use stats::{CompressionStats, StatsCollector, BenchmarkRunner};

use async_trait::async_trait;
use bytes::Bytes;
use std::fmt;

/// Compression algorithm enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CompressionAlgorithm {
    /// LZ4 - Fast compression with good ratio
    Lz4,
    /// Zstandard - Excellent ratio and speed balance
    Zstd,
    /// Brotli - Best for web assets
    Brotli,
    /// Gzip - Universal compatibility
    Gzip,
    /// Snappy - Real-time compression
    Snappy,
    /// Adaptive - Automatically select best algorithm
    Adaptive,
}

impl fmt::Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lz4 => write!(f, "LZ4"),
            Self::Zstd => write!(f, "Zstandard"),
            Self::Brotli => write!(f, "Brotli"),
            Self::Gzip => write!(f, "Gzip"),
            Self::Snappy => write!(f, "Snappy"),
            Self::Adaptive => write!(f, "Adaptive"),
        }
    }
}

impl CompressionAlgorithm {
    /// Get recommended compression level range for this algorithm
    pub fn level_range(&self) -> (i32, i32) {
        match self {
            Self::Lz4 => (1, 12),
            Self::Zstd => (1, 22),
            Self::Brotli => (0, 11),
            Self::Gzip => (0, 9),
            Self::Snappy => (0, 0), // Snappy has no compression level
            Self::Adaptive => (1, 9),
        }
    }

    /// Get default compression level
    pub fn default_level(&self) -> i32 {
        match self {
            Self::Lz4 => 4,
            Self::Zstd => 3,
            Self::Brotli => 6,
            Self::Gzip => 6,
            Self::Snappy => 0,
            Self::Adaptive => 6,
        }
    }

    /// Check if algorithm supports streaming
    pub fn supports_streaming(&self) -> bool {
        !matches!(self, Self::Snappy)
    }

    /// Check if algorithm supports dictionary training
    pub fn supports_dictionary(&self) -> bool {
        matches!(self, Self::Zstd)
    }
}

/// Core compression trait implemented by all algorithms
#[async_trait]
pub trait Compressor: Send + Sync {
    /// Compress data synchronously
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Decompress data synchronously
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Compress data asynchronously
    async fn compress_async(&self, data: Bytes) -> Result<Bytes> {
        let data_vec = data.to_vec();
        let compressed = tokio::task::spawn_blocking(move || {
            self.compress(&data_vec)
        })
        .await
        .map_err(|e| CompressionError::custom("Async compression failed", e.to_string()))??;

        Ok(Bytes::from(compressed))
    }

    /// Decompress data asynchronously
    async fn decompress_async(&self, data: Bytes) -> Result<Bytes> {
        let data_vec = data.to_vec();
        let decompressed = tokio::task::spawn_blocking(move || {
            self.decompress(&data_vec)
        })
        .await
        .map_err(|e| CompressionError::custom("Async decompression failed", e.to_string()))??;

        Ok(Bytes::from(decompressed))
    }

    /// Get compression algorithm name
    fn algorithm(&self) -> &str;

    /// Get compression level if applicable
    fn level(&self) -> Option<i32> {
        None
    }
}

/// Multi-stage compression pipeline trait
#[async_trait]
pub trait CompressionPipeline: Send + Sync {
    /// Add a compression stage to the pipeline
    fn add_stage(&mut self, compressor: Box<dyn Compressor>) -> Result<()>;

    /// Execute the pipeline on data
    async fn execute(&self, data: Bytes) -> Result<Bytes>;

    /// Execute the pipeline in reverse (decompression)
    async fn execute_reverse(&self, data: Bytes) -> Result<Bytes>;

    /// Get number of stages in pipeline
    fn stage_count(&self) -> usize;

    /// Clear all stages
    fn clear(&mut self);
}

/// Configuration for compression operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompressionConfig {
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,

    /// Compression level (algorithm-specific)
    pub level: i32,

    /// Enable checksum validation
    pub enable_checksum: bool,

    /// Buffer size for streaming operations
    pub buffer_size: usize,

    /// Enable parallel compression (if supported)
    pub enable_parallel: bool,

    /// Dictionary ID for dictionary-based compression
    pub dictionary_id: Option<String>,

    /// Maximum allowed compression time (milliseconds)
    pub max_compression_time_ms: Option<u64>,

    /// Collect detailed statistics
    pub collect_stats: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Zstd,
            level: 3,
            enable_checksum: true,
            buffer_size: 64 * 1024, // 64 KB
            enable_parallel: true,
            dictionary_id: None,
            max_compression_time_ms: None,
            collect_stats: true,
        }
    }
}

impl CompressionConfig {
    /// Create a new configuration with defaults
    pub fn new(algorithm: CompressionAlgorithm) -> Self {
        Self {
            algorithm,
            level: algorithm.default_level(),
            ..Default::default()
        }
    }

    /// Set compression level
    pub fn with_level(mut self, level: i32) -> Self {
        self.level = level;
        self
    }

    /// Set buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Enable or disable checksum
    pub fn with_checksum(mut self, enable: bool) -> Self {
        self.enable_checksum = enable;
        self
    }

    /// Set dictionary ID
    pub fn with_dictionary(mut self, dict_id: impl Into<String>) -> Self {
        self.dictionary_id = Some(dict_id.into());
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        let (min, max) = self.algorithm.level_range();
        if self.level < min || self.level > max {
            return Err(CompressionError::InvalidCompressionLevel {
                level: self.level,
                reason: format!(
                    "Level must be between {} and {} for {}",
                    min, max, self.algorithm
                ),
            });
        }

        if self.buffer_size == 0 {
            return Err(CompressionError::InvalidConfiguration(
                "Buffer size must be greater than 0".to_string(),
            ));
        }

        if self.dictionary_id.is_some() && !self.algorithm.supports_dictionary() {
            return Err(CompressionError::InvalidConfiguration(
                format!("{} does not support dictionary compression", self.algorithm),
            ));
        }

        Ok(())
    }
}

/// Unified compression facade for all algorithms
pub struct CompressionFacade;

impl CompressionFacade {
    /// Compress data with specified algorithm
    pub fn compress(
        data: &[u8],
        algorithm: CompressionAlgorithm,
    ) -> Result<Vec<u8>> {
        match algorithm {
            CompressionAlgorithm::Lz4 => lz4::Lz4Compressor::default().compress(data),
            CompressionAlgorithm::Zstd => zstd::ZstdCompressor::default().compress(data),
            CompressionAlgorithm::Brotli => brotli::BrotliCompressor::default().compress(data),
            CompressionAlgorithm::Gzip => gzip::GzipCompressor::default().compress(data),
            CompressionAlgorithm::Snappy => snappy::SnappyCompressor::default().compress(data),
            CompressionAlgorithm::Adaptive => {
                adaptive::AdaptiveCompressor::new().compress_with_best_algorithm(data)
            }
        }
    }

    /// Decompress data with specified algorithm
    pub fn decompress(
        data: &[u8],
        algorithm: CompressionAlgorithm,
    ) -> Result<Vec<u8>> {
        match algorithm {
            CompressionAlgorithm::Lz4 => lz4::Lz4Compressor::default().decompress(data),
            CompressionAlgorithm::Zstd => zstd::ZstdCompressor::default().decompress(data),
            CompressionAlgorithm::Brotli => brotli::BrotliCompressor::default().decompress(data),
            CompressionAlgorithm::Gzip => gzip::GzipCompressor::default().decompress(data),
            CompressionAlgorithm::Snappy => snappy::SnappyCompressor::default().decompress(data),
            CompressionAlgorithm::Adaptive => {
                Err(CompressionError::Adaptive(
                    "Cannot decompress with Adaptive - original algorithm unknown".to_string()
                ))
            }
        }
    }

    /// Compress with configuration
    pub fn compress_with_config(
        data: &[u8],
        config: &CompressionConfig,
    ) -> Result<Vec<u8>> {
        config.validate()?;

        match config.algorithm {
            CompressionAlgorithm::Lz4 => {
                lz4::Lz4Compressor::with_level(config.level).compress(data)
            }
            CompressionAlgorithm::Zstd => {
                zstd::ZstdCompressor::with_level(config.level).compress(data)
            }
            CompressionAlgorithm::Brotli => {
                brotli::BrotliCompressor::with_level(config.level as u32).compress(data)
            }
            CompressionAlgorithm::Gzip => {
                gzip::GzipCompressor::with_level(config.level as u32).compress(data)
            }
            CompressionAlgorithm::Snappy => {
                snappy::SnappyCompressor::default().compress(data)
            }
            CompressionAlgorithm::Adaptive => {
                adaptive::AdaptiveCompressor::new().compress_with_best_algorithm(data)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_display() {
        assert_eq!(CompressionAlgorithm::Lz4.to_string(), "LZ4");
        assert_eq!(CompressionAlgorithm::Zstd.to_string(), "Zstandard");
    }

    #[test]
    fn test_config_validation() {
        let config = CompressionConfig::new(CompressionAlgorithm::Zstd)
            .with_level(50);
        assert!(config.validate().is_err());

        let config = CompressionConfig::new(CompressionAlgorithm::Zstd)
            .with_level(10);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_algorithm_capabilities() {
        assert!(CompressionAlgorithm::Zstd.supports_streaming());
        assert!(CompressionAlgorithm::Zstd.supports_dictionary());
        assert!(!CompressionAlgorithm::Snappy.supports_streaming());
    }
}
