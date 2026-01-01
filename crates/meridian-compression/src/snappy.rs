//! Snappy compression for real-time compression
//!
//! Extremely fast compression optimized for speed over ratio.

use crate::error::{CompressionError, Result};
use crate::stats::{CompressionStats, Timer};
use crate::Compressor;
use async_trait::async_trait;
use snap::{read::FrameDecoder, write::FrameEncoder};
use std::io::{Read, Write};

/// Snappy compression configuration
#[derive(Debug, Clone)]
pub struct SnappyConfig {
    /// Use framing format (recommended for streaming)
    pub use_framing: bool,
    /// Maximum block size for framing
    pub max_block_size: usize,
}

impl Default for SnappyConfig {
    fn default() -> Self {
        Self {
            use_framing: true,
            max_block_size: 64 * 1024, // 64 KB
        }
    }
}

/// Snappy compressor optimized for speed
pub struct SnappyCompressor {
    config: SnappyConfig,
}

impl Default for SnappyCompressor {
    fn default() -> Self {
        Self {
            config: SnappyConfig::default(),
        }
    }
}

impl SnappyCompressor {
    /// Create a new Snappy compressor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom configuration
    pub fn with_config(config: SnappyConfig) -> Self {
        Self { config }
    }

    /// Create raw compressor (no framing)
    pub fn raw() -> Self {
        Self {
            config: SnappyConfig {
                use_framing: false,
                ..Default::default()
            },
        }
    }

    /// Compress with statistics
    pub fn compress_with_stats(&self, data: &[u8]) -> Result<(Vec<u8>, CompressionStats)> {
        let timer = Timer::start("snappy_compress");
        let compressed = self.compress(data)?;
        let elapsed = timer.stop();

        let stats = CompressionStats::new(
            data.len(),
            compressed.len(),
            elapsed,
            "snappy",
        );

        Ok((compressed, stats))
    }

    /// Compress raw (without framing)
    pub fn compress_raw(&self, data: &[u8]) -> Result<Vec<u8>> {
        let max_compressed_len = snap::raw::max_compress_len(data.len());
        let mut compressed = vec![0u8; max_compressed_len];

        let compressed_len = snap::raw::Encoder::new()
            .compress(data, &mut compressed)
            .map_err(|e| CompressionError::snappy(e.to_string()))?;

        compressed.truncate(compressed_len);
        Ok(compressed)
    }

    /// Decompress raw (without framing)
    pub fn decompress_raw(&self, data: &[u8]) -> Result<Vec<u8>> {
        let decompressed_len = snap::raw::decompress_len(data)
            .map_err(|e| CompressionError::snappy(e.to_string()))?;

        let mut decompressed = vec![0u8; decompressed_len];

        snap::raw::Decoder::new()
            .decompress(data, &mut decompressed)
            .map_err(|e| CompressionError::snappy(e.to_string()))?;

        Ok(decompressed)
    }

    /// Compress with framing format
    pub fn compress_framed(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = FrameEncoder::new(Vec::new());

        encoder
            .write_all(data)
            .map_err(|e| CompressionError::snappy(e.to_string()))?;

        encoder
            .into_inner()
            .map_err(|e| CompressionError::snappy(e.to_string()))
    }

    /// Decompress framing format
    pub fn decompress_framed(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = FrameDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| CompressionError::snappy(e.to_string()))?;

        Ok(decompressed)
    }

    /// Stream compress
    pub fn compress_stream<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<usize> {
        let mut encoder = FrameEncoder::new(writer);

        let bytes_written = std::io::copy(reader, &mut encoder)
            .map_err(|e| CompressionError::snappy(e.to_string()))?;

        encoder
            .into_inner()
            .map_err(|e| CompressionError::snappy(e.to_string()))?;

        Ok(bytes_written as usize)
    }

    /// Stream decompress
    pub fn decompress_stream<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<usize> {
        let mut decoder = FrameDecoder::new(reader);

        let bytes_written = std::io::copy(&mut decoder, writer)
            .map_err(|e| CompressionError::snappy(e.to_string()))?;

        Ok(bytes_written as usize)
    }

    /// Get maximum compressed length for input size
    pub fn max_compressed_len(input_len: usize) -> usize {
        snap::raw::max_compress_len(input_len)
    }
}

#[async_trait]
impl Compressor for SnappyCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if self.config.use_framing {
            self.compress_framed(data)
        } else {
            self.compress_raw(data)
        }
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Try framed format first, fall back to raw
        self.decompress_framed(data)
            .or_else(|_| self.decompress_raw(data))
    }

    fn algorithm(&self) -> &str {
        "snappy"
    }

    fn level(&self) -> Option<i32> {
        None // Snappy has no compression levels
    }
}

/// Snappy block compressor for parallel compression
pub struct SnappyBlockCompressor {
    block_size: usize,
}

impl Default for SnappyBlockCompressor {
    fn default() -> Self {
        Self {
            block_size: 64 * 1024, // 64 KB
        }
    }
}

impl SnappyBlockCompressor {
    /// Create a new block compressor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific block size
    pub fn with_block_size(block_size: usize) -> Self {
        Self { block_size }
    }

    /// Compress data in parallel blocks
    pub fn compress_parallel(&self, data: &[u8]) -> Result<Vec<Vec<u8>>> {
        use rayon::prelude::*;

        let blocks: Vec<&[u8]> = data.chunks(self.block_size).collect();

        blocks
            .par_iter()
            .map(|block| {
                let compressor = SnappyCompressor::raw();
                compressor.compress_raw(block)
            })
            .collect()
    }

    /// Decompress parallel blocks
    pub fn decompress_parallel(&self, blocks: Vec<Vec<u8>>) -> Result<Vec<u8>> {
        use rayon::prelude::*;

        let decompressed_blocks: Result<Vec<Vec<u8>>> = blocks
            .par_iter()
            .map(|block| {
                let compressor = SnappyCompressor::raw();
                compressor.decompress_raw(block)
            })
            .collect();

        let decompressed = decompressed_blocks?
            .into_iter()
            .flatten()
            .collect();

        Ok(decompressed)
    }
}

/// Real-time compressor optimized for minimal latency
pub struct RealtimeCompressor {
    compressor: SnappyCompressor,
    max_latency_us: u64,
}

impl RealtimeCompressor {
    /// Create a new real-time compressor
    pub fn new(max_latency_us: u64) -> Self {
        Self {
            compressor: SnappyCompressor::new(),
            max_latency_us,
        }
    }

    /// Compress with latency guarantee
    pub fn compress_with_deadline(&self, data: &[u8]) -> Result<Option<Vec<u8>>> {
        use std::time::Instant;

        let start = Instant::now();
        let compressed = self.compressor.compress(data)?;
        let elapsed = start.elapsed();

        if elapsed.as_micros() as u64 > self.max_latency_us {
            // Latency exceeded, return uncompressed
            Ok(None)
        } else {
            Ok(Some(compressed))
        }
    }

    /// Check if compression would meet latency target (estimate)
    pub fn can_compress_in_time(&self, data_size: usize) -> bool {
        // Snappy typically achieves 250-500 MB/s
        // Conservative estimate: 200 MB/s
        let estimated_us = (data_size as f64 / 200_000_000.0) * 1_000_000.0;
        estimated_us < self.max_latency_us as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snappy_compress_decompress() {
        let compressor = SnappyCompressor::new();
        let data = b"Hello, Snappy! Fast compression for real-time applications.";

        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_snappy_raw() {
        let compressor = SnappyCompressor::raw();
        let data = b"Raw Snappy compression test";

        let compressed = compressor.compress_raw(data).unwrap();
        let decompressed = compressor.decompress_raw(&compressed).unwrap();

        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_snappy_framed() {
        let compressor = SnappyCompressor::new();
        let data = b"Framed Snappy compression test" as &[u8];
        let data = data.repeat(100);

        let compressed = compressor.compress_framed(&data).unwrap();
        let decompressed = compressor.decompress_framed(&compressed).unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_snappy_speed() {
        let compressor = SnappyCompressor::new();
        let data = b"x" as &[u8];
        let data = data.repeat(10_000);

        let (compressed, stats) = compressor.compress_with_stats(&data).unwrap();

        // Snappy should be very fast
        assert!(stats.throughput_mbps > 10.0); // At least 10 MB/s
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_block_compressor() {
        let compressor = SnappyBlockCompressor::new();
        let data = b"Test data for block compression" as &[u8];
        let data = data.repeat(1000);

        let blocks = compressor.compress_parallel(&data).unwrap();
        assert!(!blocks.is_empty());

        let decompressed = compressor.decompress_parallel(blocks).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_realtime_compressor() {
        let compressor = RealtimeCompressor::new(1000); // 1ms deadline
        let data = b"Real-time compression test";

        let result = compressor.compress_with_deadline(data).unwrap();
        assert!(result.is_some());

        if let Some(compressed) = result {
            let decompressed = SnappyCompressor::new()
                .decompress(&compressed)
                .unwrap();
            assert_eq!(data.as_slice(), decompressed.as_slice());
        }
    }

    #[test]
    fn test_max_compressed_len() {
        let input_len = 1000;
        let max_len = SnappyCompressor::max_compressed_len(input_len);
        assert!(max_len >= input_len);
    }

    #[tokio::test]
    async fn test_snappy_async() {
        let compressor = SnappyCompressor::new();
        let data = bytes::Bytes::from("Async Snappy test");

        let compressed = compressor.compress_async(data.clone()).await.unwrap();
        let decompressed = compressor.decompress_async(compressed).await.unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_format_compatibility() {
        let raw_compressor = SnappyCompressor::raw();
        let framed_compressor = SnappyCompressor::new();

        let data = b"Test compatibility";

        let raw_compressed = raw_compressor.compress_raw(data).unwrap();
        let framed_compressed = framed_compressor.compress_framed(data).unwrap();

        // Formats should be different
        assert_ne!(raw_compressed, framed_compressed);

        // Each should decompress with appropriate method
        assert_eq!(
            raw_compressor.decompress_raw(&raw_compressed).unwrap(),
            data
        );
        assert_eq!(
            framed_compressor.decompress_framed(&framed_compressed).unwrap(),
            data
        );
    }
}
