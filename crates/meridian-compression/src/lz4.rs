//! LZ4 compression with streaming support
//!
//! Fast compression algorithm optimized for speed with good compression ratios.

use crate::error::{CompressionError, Result};
use crate::stats::{CompressionStats, Timer};
use crate::Compressor;
use async_trait::async_trait;
use bytes::Bytes;
use std::io::{Read, Write};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// LZ4 compression configuration
#[derive(Debug, Clone)]
pub struct Lz4Config {
    /// Compression level (1-12)
    pub level: i32,
    /// Enable block independence
    pub block_independent: bool,
    /// Block size in bytes
    pub block_size: usize,
    /// Enable checksum
    pub enable_checksum: bool,
}

impl Default for Lz4Config {
    fn default() -> Self {
        Self {
            level: 4,
            block_independent: true,
            block_size: 4 * 1024 * 1024, // 4 MB
            enable_checksum: true,
        }
    }
}

/// LZ4 compressor with streaming support
pub struct Lz4Compressor {
    config: Lz4Config,
}

impl Default for Lz4Compressor {
    fn default() -> Self {
        Self {
            config: Lz4Config::default(),
        }
    }
}

impl Lz4Compressor {
    /// Create a new LZ4 compressor with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom configuration
    pub fn with_config(config: Lz4Config) -> Self {
        Self { config }
    }

    /// Create with specific compression level
    pub fn with_level(level: i32) -> Self {
        Self {
            config: Lz4Config {
                level,
                ..Default::default()
            },
        }
    }

    /// Compress with statistics
    pub fn compress_with_stats(&self, data: &[u8]) -> Result<(Vec<u8>, CompressionStats)> {
        let timer = Timer::start("lz4_compress");
        let compressed = self.compress(data)?;
        let elapsed = timer.stop();

        let stats = CompressionStats::new(
            data.len(),
            compressed.len(),
            elapsed,
            "lz4",
        )
        .with_level(self.config.level);

        Ok((compressed, stats))
    }

    /// Stream compress from reader to writer
    pub fn compress_stream<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<usize> {
        let mut encoder = lz4::Encoder::new(writer)
            .map_err(|e| CompressionError::lz4(e.to_string()))?;

        let bytes_written = std::io::copy(reader, &mut encoder)
            .map_err(|e| CompressionError::lz4(e.to_string()))?;

        let (_writer, result) = encoder.finish();
        result.map_err(|e| CompressionError::lz4(e.to_string()))?;

        Ok(bytes_written as usize)
    }

    /// Async stream compress
    pub async fn compress_stream_async<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<usize> {
        let mut buffer = vec![0u8; self.config.block_size];
        let mut total_bytes = 0;

        // Create a temporary buffer to collect all data
        let mut all_data = Vec::new();

        loop {
            let bytes_read = reader
                .read(&mut buffer)
                .await
                .map_err(CompressionError::from)?;

            if bytes_read == 0 {
                break;
            }

            all_data.extend_from_slice(&buffer[..bytes_read]);
            total_bytes += bytes_read;
        }

        // Compress the collected data
        let compressed = self.compress(&all_data)?;

        // Write compressed data
        writer
            .write_all(&compressed)
            .await
            .map_err(CompressionError::from)?;

        writer.flush().await.map_err(CompressionError::from)?;

        Ok(total_bytes)
    }

    /// Decompress with statistics
    pub fn decompress_with_stats(&self, data: &[u8]) -> Result<(Vec<u8>, CompressionStats)> {
        let timer = Timer::start("lz4_decompress");
        let decompressed = self.decompress(data)?;
        let elapsed = timer.stop();

        let stats = CompressionStats::new(
            decompressed.len(),
            data.len(),
            elapsed,
            "lz4_decompress",
        );

        Ok((decompressed, stats))
    }
}

#[async_trait]
impl Compressor for Lz4Compressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut compressed = Vec::new();

        {
            let mut encoder = lz4::Encoder::new(&mut compressed)
                .map_err(|e| CompressionError::lz4(e.to_string()))?;

            encoder
                .write_all(data)
                .map_err(|e| CompressionError::lz4(e.to_string()))?;

            let (_output, result) = encoder.finish();
            result.map_err(|e| CompressionError::lz4(e.to_string()))?;
        }

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = lz4::Decoder::new(data)
            .map_err(|e| CompressionError::lz4(e.to_string()))?;

        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| CompressionError::lz4(e.to_string()))?;

        Ok(decompressed)
    }

    fn algorithm(&self) -> &str {
        "lz4"
    }

    fn level(&self) -> Option<i32> {
        Some(self.config.level)
    }
}

/// LZ4 block compressor for independent blocks
pub struct Lz4BlockCompressor {
    level: i32,
}

impl Default for Lz4BlockCompressor {
    fn default() -> Self {
        Self { level: 4 }
    }
}

impl Lz4BlockCompressor {
    /// Create a new block compressor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific level
    pub fn with_level(level: i32) -> Self {
        Self { level }
    }

    /// Compress a single block
    pub fn compress_block(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Use lz4-sys for block compression if available
        // For now, fall back to frame compression
        let compressor = Lz4Compressor::with_level(self.level);
        compressor.compress(data)
    }

    /// Decompress a single block
    pub fn decompress_block(&self, data: &[u8], original_size: usize) -> Result<Vec<u8>> {
        let compressor = Lz4Compressor::default();
        let decompressed = compressor.decompress(data)?;

        if decompressed.len() != original_size {
            return Err(CompressionError::lz4(format!(
                "Decompressed size {} doesn't match expected size {}",
                decompressed.len(),
                original_size
            )));
        }

        Ok(decompressed)
    }
}

/// LZ4 HC (High Compression) variant
pub struct Lz4HcCompressor {
    level: i32,
}

impl Default for Lz4HcCompressor {
    fn default() -> Self {
        Self { level: 9 }
    }
}

impl Lz4HcCompressor {
    /// Create new HC compressor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific level (1-12)
    pub fn with_level(level: i32) -> Self {
        Self { level }
    }
}

#[async_trait]
impl Compressor for Lz4HcCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        // LZ4 HC uses higher compression levels
        let compressor = Lz4Compressor::with_level(self.level.max(9));
        compressor.compress(data)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let compressor = Lz4Compressor::default();
        compressor.decompress(data)
    }

    fn algorithm(&self) -> &str {
        "lz4_hc"
    }

    fn level(&self) -> Option<i32> {
        Some(self.level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz4_compress_decompress() {
        let compressor = Lz4Compressor::new();
        let data = b"Hello, World! This is a test of LZ4 compression.";

        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_lz4_with_stats() {
        let compressor = Lz4Compressor::new();
        let data = b"Test data for statistics collection" as &[u8];
        let data = data.repeat(100);

        let (compressed, stats) = compressor.compress_with_stats(&data).unwrap();

        assert!(stats.is_effective());
        assert!(stats.compression_ratio > 1.0);
        assert_eq!(stats.algorithm, "lz4");
    }

    #[test]
    fn test_lz4_levels() {
        let data = b"Test data" as &[u8];
        let data = data.repeat(1000);

        let low = Lz4Compressor::with_level(1);
        let high = Lz4Compressor::with_level(12);

        let compressed_low = low.compress(&data).unwrap();
        let compressed_high = high.compress(&data).unwrap();

        // Higher compression should result in smaller size
        assert!(compressed_high.len() <= compressed_low.len());
    }

    #[tokio::test]
    async fn test_lz4_async() {
        let compressor = Lz4Compressor::new();
        let data = Bytes::from("Async compression test data");

        let compressed = compressor.compress_async(data.clone()).await.unwrap();
        let decompressed = compressor.decompress_async(compressed).await.unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_lz4_hc() {
        let compressor = Lz4HcCompressor::new();
        let data = b"High compression test data" as &[u8];
        let data = data.repeat(100);

        let compressed = compressor.compress(&data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data, decompressed);
        assert!(compressed.len() < data.len());
    }
}
