//! Compression support for cached data

use bytes::Bytes;

use crate::error::{CacheError, CacheResult};

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// No compression
    None,
    /// LZ4 compression (fast)
    LZ4,
    /// Zstandard compression (high ratio)
    Zstd,
}

/// Compression level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// Fastest compression
    Fastest,
    /// Default compression
    Default,
    /// Best compression ratio
    Best,
    /// Custom level (algorithm-specific)
    Custom(i32),
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,
    /// Compression level
    pub level: CompressionLevel,
    /// Minimum size to compress (bytes)
    pub min_size: usize,
    /// Dictionary for training (Zstd only)
    pub dictionary: Option<Vec<u8>>,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::LZ4,
            level: CompressionLevel::Default,
            min_size: 1024, // 1 KB
            dictionary: None,
        }
    }
}

/// Compressor for cache data
pub struct Compressor {
    config: CompressionConfig,
}

impl Compressor {
    /// Create a new compressor with the given configuration
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// Create a compressor with LZ4 algorithm
    pub fn lz4() -> Self {
        Self::new(CompressionConfig {
            algorithm: CompressionAlgorithm::LZ4,
            ..Default::default()
        })
    }

    /// Create a compressor with Zstd algorithm
    pub fn zstd() -> Self {
        Self::new(CompressionConfig {
            algorithm: CompressionAlgorithm::Zstd,
            ..Default::default()
        })
    }

    /// Create a compressor with no compression
    pub fn none() -> Self {
        Self::new(CompressionConfig {
            algorithm: CompressionAlgorithm::None,
            ..Default::default()
        })
    }

    /// Compress data if it meets the minimum size threshold
    pub fn compress(&self, data: &[u8]) -> CacheResult<CompressedData> {
        // Check if compression is worth it
        if data.len() < self.config.min_size
            || self.config.algorithm == CompressionAlgorithm::None
        {
            return Ok(CompressedData {
                data: Bytes::copy_from_slice(data),
                algorithm: CompressionAlgorithm::None,
                original_size: data.len(),
            });
        }

        match self.config.algorithm {
            CompressionAlgorithm::None => Ok(CompressedData {
                data: Bytes::copy_from_slice(data),
                algorithm: CompressionAlgorithm::None,
                original_size: data.len(),
            }),
            CompressionAlgorithm::LZ4 => self.compress_lz4(data),
            CompressionAlgorithm::Zstd => self.compress_zstd(data),
        }
    }

    /// Decompress data
    pub fn decompress(&self, compressed: &CompressedData) -> CacheResult<Bytes> {
        match compressed.algorithm {
            CompressionAlgorithm::None => Ok(compressed.data.clone()),
            CompressionAlgorithm::LZ4 => self.decompress_lz4(compressed),
            CompressionAlgorithm::Zstd => self.decompress_zstd(compressed),
        }
    }

    /// Compress using LZ4
    fn compress_lz4(&self, data: &[u8]) -> CacheResult<CompressedData> {
        let compressed = lz4::block::compress(data, None, false)
            .map_err(|e| CacheError::Compression(format!("LZ4 compression failed: {}", e)))?;

        Ok(CompressedData {
            data: Bytes::from(compressed),
            algorithm: CompressionAlgorithm::LZ4,
            original_size: data.len(),
        })
    }

    /// Decompress using LZ4
    fn decompress_lz4(&self, compressed: &CompressedData) -> CacheResult<Bytes> {
        let decompressed = lz4::block::decompress(&compressed.data, Some(compressed.original_size as i32))
            .map_err(|e| CacheError::Decompression(format!("LZ4 decompression failed: {}", e)))?;

        Ok(Bytes::from(decompressed))
    }

    /// Compress using Zstd
    fn compress_zstd(&self, data: &[u8]) -> CacheResult<CompressedData> {
        let level = match self.config.level {
            CompressionLevel::Fastest => 1,
            CompressionLevel::Default => 3,
            CompressionLevel::Best => 19,
            CompressionLevel::Custom(l) => l,
        };

        let compressed = zstd::encode_all(data, level)
            .map_err(|e| CacheError::Compression(format!("Zstd compression failed: {}", e)))?;

        Ok(CompressedData {
            data: Bytes::from(compressed),
            algorithm: CompressionAlgorithm::Zstd,
            original_size: data.len(),
        })
    }

    /// Decompress using Zstd
    fn decompress_zstd(&self, compressed: &CompressedData) -> CacheResult<Bytes> {
        let decompressed = zstd::decode_all(&compressed.data[..])
            .map_err(|e| CacheError::Decompression(format!("Zstd decompression failed: {}", e)))?;

        Ok(Bytes::from(decompressed))
    }

    /// Get the compression ratio for given data
    pub fn get_compression_ratio(&self, original: &[u8], compressed: &CompressedData) -> f64 {
        if original.is_empty() {
            return 0.0;
        }
        compressed.data.len() as f64 / original.len() as f64
    }

    /// Check if compression is beneficial for given data
    pub fn is_compression_beneficial(&self, data: &[u8]) -> CacheResult<bool> {
        if data.len() < self.config.min_size {
            return Ok(false);
        }

        let compressed = self.compress(data)?;
        let ratio = self.get_compression_ratio(data, &compressed);

        // Consider compression beneficial if it reduces size by at least 10%
        Ok(ratio < 0.9)
    }
}

/// Compressed data with metadata
#[derive(Debug, Clone)]
pub struct CompressedData {
    /// Compressed data
    pub data: Bytes,
    /// Compression algorithm used
    pub algorithm: CompressionAlgorithm,
    /// Original uncompressed size
    pub original_size: usize,
}

impl CompressedData {
    /// Get the compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            return 0.0;
        }
        self.data.len() as f64 / self.original_size as f64
    }

    /// Get the space saved by compression
    pub fn space_saved(&self) -> usize {
        self.original_size.saturating_sub(self.data.len())
    }

    /// Get the space saved as a percentage
    pub fn space_saved_percent(&self) -> f64 {
        if self.original_size == 0 {
            return 0.0;
        }
        (self.space_saved() as f64 / self.original_size as f64) * 100.0
    }
}

/// Adaptive compressor that selects the best algorithm
pub struct AdaptiveCompressor {
    lz4: Compressor,
    zstd: Compressor,
    /// Size threshold to switch from LZ4 to Zstd
    zstd_threshold: usize,
}

impl AdaptiveCompressor {
    /// Create a new adaptive compressor
    pub fn new(zstd_threshold: usize) -> Self {
        Self {
            lz4: Compressor::lz4(),
            zstd: Compressor::zstd(),
            zstd_threshold,
        }
    }

    /// Compress using the best algorithm for the data
    pub fn compress(&self, data: &[u8]) -> CacheResult<CompressedData> {
        // Use LZ4 for small data (faster)
        // Use Zstd for large data (better compression)
        if data.len() < self.zstd_threshold {
            self.lz4.compress(data)
        } else {
            self.zstd.compress(data)
        }
    }

    /// Decompress data
    pub fn decompress(&self, compressed: &CompressedData) -> CacheResult<Bytes> {
        match compressed.algorithm {
            CompressionAlgorithm::LZ4 => self.lz4.decompress(compressed),
            CompressionAlgorithm::Zstd => self.zstd.decompress(compressed),
            CompressionAlgorithm::None => Ok(compressed.data.clone()),
        }
    }

    /// Try both algorithms and return the better result
    pub fn compress_best(&self, data: &[u8]) -> CacheResult<CompressedData> {
        let lz4_result = self.lz4.compress(data)?;
        let zstd_result = self.zstd.compress(data)?;

        if lz4_result.data.len() <= zstd_result.data.len() {
            Ok(lz4_result)
        } else {
            Ok(zstd_result)
        }
    }
}

impl Default for AdaptiveCompressor {
    fn default() -> Self {
        Self::new(64 * 1024) // 64 KB threshold
    }
}

/// Compression statistics
#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    /// Total bytes compressed
    pub total_bytes: usize,
    /// Total compressed size
    pub compressed_bytes: usize,
    /// Number of compressions
    pub count: usize,
}

impl CompressionStats {
    /// Record a compression operation
    pub fn record(&mut self, original_size: usize, compressed_size: usize) {
        self.total_bytes += original_size;
        self.compressed_bytes += compressed_size;
        self.count += 1;
    }

    /// Get the average compression ratio
    pub fn average_ratio(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        self.compressed_bytes as f64 / self.total_bytes as f64
    }

    /// Get total space saved
    pub fn total_saved(&self) -> usize {
        self.total_bytes.saturating_sub(self.compressed_bytes)
    }

    /// Get space saved as a percentage
    pub fn saved_percent(&self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        (self.total_saved() as f64 / self.total_bytes as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz4_compression() -> CacheResult<()> {
        let compressor = Compressor::lz4();
        let data = b"Hello, World! This is a test string that should compress well.".repeat(10);

        let compressed = compressor.compress(&data)?;
        assert!(compressed.data.len() < data.len());
        assert_eq!(compressed.algorithm, CompressionAlgorithm::LZ4);

        let decompressed = compressor.decompress(&compressed)?;
        assert_eq!(decompressed, data);

        Ok(())
    }

    #[test]
    fn test_zstd_compression() -> CacheResult<()> {
        let compressor = Compressor::zstd();
        let data = b"Hello, World! This is a test string that should compress well.".repeat(10);

        let compressed = compressor.compress(&data)?;
        assert!(compressed.data.len() < data.len());
        assert_eq!(compressed.algorithm, CompressionAlgorithm::Zstd);

        let decompressed = compressor.decompress(&compressed)?;
        assert_eq!(decompressed, data);

        Ok(())
    }

    #[test]
    fn test_no_compression_for_small_data() -> CacheResult<()> {
        let compressor = Compressor::new(CompressionConfig {
            algorithm: CompressionAlgorithm::LZ4,
            min_size: 1024,
            ..Default::default()
        });

        let data = b"Small data";
        let compressed = compressor.compress(data)?;

        assert_eq!(compressed.algorithm, CompressionAlgorithm::None);
        assert_eq!(compressed.data, data);

        Ok(())
    }

    #[test]
    fn test_adaptive_compressor() -> CacheResult<()> {
        let compressor = AdaptiveCompressor::default();

        // Small data should use LZ4
        let small_data = b"Small data".repeat(10);
        let small_compressed = compressor.compress(&small_data)?;
        assert_eq!(small_compressed.algorithm, CompressionAlgorithm::LZ4);

        // Large data should use Zstd
        let large_data = b"Large data ".repeat(10000);
        let large_compressed = compressor.compress(&large_data)?;
        assert_eq!(large_compressed.algorithm, CompressionAlgorithm::Zstd);

        Ok(())
    }

    #[test]
    fn test_compression_stats() {
        let mut stats = CompressionStats::default();

        stats.record(1000, 500);
        stats.record(2000, 1000);

        assert_eq!(stats.total_bytes, 3000);
        assert_eq!(stats.compressed_bytes, 1500);
        assert_eq!(stats.count, 2);
        assert_eq!(stats.average_ratio(), 0.5);
        assert_eq!(stats.total_saved(), 1500);
        assert_eq!(stats.saved_percent(), 50.0);
    }
}
