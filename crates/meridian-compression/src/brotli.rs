//! Brotli compression for web assets
//!
//! Optimized for HTTP compression with excellent ratios for text and web content.

use crate::error::{CompressionError, Result};
use crate::stats::{CompressionStats, Timer};
use crate::Compressor;
use async_trait::async_trait;
use brotli::{enc::BrotliEncoderParams, BrotliCompress, BrotliDecompress};
use std::io::{Read, Write};

/// Brotli compression quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrotliQuality {
    /// Fastest compression (level 0-1)
    Fastest,
    /// Fast compression (level 2-4)
    Fast,
    /// Balanced compression (level 5-7)
    Balanced,
    /// Best compression (level 8-9)
    Best,
    /// Maximum compression (level 10-11)
    Maximum,
}

impl BrotliQuality {
    /// Get numeric quality level
    pub fn level(&self) -> u32 {
        match self {
            Self::Fastest => 1,
            Self::Fast => 4,
            Self::Balanced => 6,
            Self::Best => 9,
            Self::Maximum => 11,
        }
    }
}

/// Brotli compression mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrotliMode {
    /// Generic compression mode
    Generic,
    /// Text compression mode
    Text,
    /// Font compression mode
    Font,
}

/// Brotli compression configuration
#[derive(Debug, Clone)]
pub struct BrotliConfig {
    /// Compression quality (0-11)
    pub quality: u32,
    /// Window size (10-24)
    pub window_size: u32,
    /// Block size (16-24)
    pub block_size: u32,
    /// Compression mode
    pub mode: BrotliMode,
}

impl Default for BrotliConfig {
    fn default() -> Self {
        Self {
            quality: 6,
            window_size: 22,
            block_size: 0, // auto
            mode: BrotliMode::Generic,
        }
    }
}

/// Brotli compressor optimized for web assets
pub struct BrotliCompressor {
    config: BrotliConfig,
}

impl Default for BrotliCompressor {
    fn default() -> Self {
        Self {
            config: BrotliConfig::default(),
        }
    }
}

impl BrotliCompressor {
    /// Create a new Brotli compressor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom configuration
    pub fn with_config(config: BrotliConfig) -> Self {
        Self { config }
    }

    /// Create with specific quality level
    pub fn with_level(quality: u32) -> Self {
        Self {
            config: BrotliConfig {
                quality: quality.min(11),
                ..Default::default()
            },
        }
    }

    /// Create with quality preset
    pub fn with_quality(quality: BrotliQuality) -> Self {
        Self::with_level(quality.level())
    }

    /// Create optimized for text compression
    pub fn for_text() -> Self {
        Self {
            config: BrotliConfig {
                quality: 9,
                mode: BrotliMode::Text,
                ..Default::default()
            },
        }
    }

    /// Create optimized for font compression
    pub fn for_font() -> Self {
        Self {
            config: BrotliConfig {
                quality: 11,
                mode: BrotliMode::Font,
                ..Default::default()
            },
        }
    }

    /// Compress with statistics
    pub fn compress_with_stats(&self, data: &[u8]) -> Result<(Vec<u8>, CompressionStats)> {
        let timer = Timer::start("brotli_compress");
        let compressed = self.compress(data)?;
        let elapsed = timer.stop();

        let stats = CompressionStats::new(
            data.len(),
            compressed.len(),
            elapsed,
            "brotli",
        )
        .with_level(self.config.quality as i32);

        Ok((compressed, stats))
    }

    /// Create encoder parameters
    fn create_params(&self) -> BrotliEncoderParams {
        let mut params = BrotliEncoderParams::default();
        params.quality = self.config.quality as i32;
        params.lgwin = self.config.window_size as i32;

        if self.config.block_size > 0 {
            params.lgblock = self.config.block_size as i32;
        }

        params.mode = match self.config.mode {
            BrotliMode::Generic => brotli::enc::BrotliEncoderMode::BROTLI_MODE_GENERIC,
            BrotliMode::Text => brotli::enc::BrotliEncoderMode::BROTLI_MODE_TEXT,
            BrotliMode::Font => brotli::enc::BrotliEncoderMode::BROTLI_MODE_FONT,
        };

        params
    }

    /// Compress stream
    pub fn compress_stream<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<usize> {
        let params = self.create_params();
        let mut input_buffer = Vec::new();

        reader
            .read_to_end(&mut input_buffer)
            .map_err(CompressionError::from)?;

        let original_size = input_buffer.len();

        BrotliCompress(
            &mut input_buffer.as_slice(),
            writer,
            &params,
        )
        .map_err(|e| CompressionError::brotli(e.to_string()))?;

        Ok(original_size)
    }

    /// Decompress stream
    pub fn decompress_stream<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<usize> {
        let mut input_buffer = Vec::new();
        reader
            .read_to_end(&mut input_buffer)
            .map_err(CompressionError::from)?;

        let bytes_written = BrotliDecompress(&mut input_buffer.as_slice(), writer)
            .map_err(|e| CompressionError::brotli(e.to_string()))?;

        Ok(bytes_written)
    }
}

#[async_trait]
impl Compressor for BrotliCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        let params = self.create_params();

        BrotliCompress(&mut data.as_ref(), &mut output, &params)
            .map_err(|e| CompressionError::brotli(e.to_string()))?;

        Ok(output)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut output = Vec::new();

        BrotliDecompress(&mut data.as_ref(), &mut output)
            .map_err(|e| CompressionError::brotli(e.to_string()))?;

        Ok(output)
    }

    fn algorithm(&self) -> &str {
        "brotli"
    }

    fn level(&self) -> Option<i32> {
        Some(self.config.quality as i32)
    }
}

/// Brotli compressor optimized for HTTP responses
pub struct BrotliHttpCompressor {
    text_compressor: BrotliCompressor,
    generic_compressor: BrotliCompressor,
}

impl Default for BrotliHttpCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl BrotliHttpCompressor {
    /// Create a new HTTP compressor
    pub fn new() -> Self {
        Self {
            text_compressor: BrotliCompressor::for_text(),
            generic_compressor: BrotliCompressor::with_level(6),
        }
    }

    /// Compress based on content type
    pub fn compress_by_content_type(
        &self,
        data: &[u8],
        content_type: &str,
    ) -> Result<Vec<u8>> {
        if content_type.starts_with("text/")
            || content_type.contains("javascript")
            || content_type.contains("json")
            || content_type.contains("xml")
        {
            self.text_compressor.compress(data)
        } else if content_type.contains("font") {
            BrotliCompressor::for_font().compress(data)
        } else {
            self.generic_compressor.compress(data)
        }
    }

    /// Check if content type should be compressed
    pub fn should_compress(content_type: &str) -> bool {
        matches!(
            content_type,
            ct if ct.starts_with("text/")
                || ct.contains("javascript")
                || ct.contains("json")
                || ct.contains("xml")
                || ct.contains("svg")
                || ct.contains("font")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brotli_compress_decompress() {
        let compressor = BrotliCompressor::new();
        let data = b"Hello, Brotli compression! This is optimized for web assets.";

        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.as_slice(), decompressed.as_slice());
        assert!(compressed.len() < data.len());
    }

    #[test]
    fn test_brotli_quality_levels() {
        let data = b"Test data for Brotli compression" as &[u8];
        let data = data.repeat(100);

        let fast = BrotliCompressor::with_quality(BrotliQuality::Fast);
        let best = BrotliCompressor::with_quality(BrotliQuality::Best);

        let compressed_fast = fast.compress(&data).unwrap();
        let compressed_best = best.compress(&data).unwrap();

        // Higher quality should result in better compression
        assert!(compressed_best.len() <= compressed_fast.len());
    }

    #[test]
    fn test_brotli_text_mode() {
        let compressor = BrotliCompressor::for_text();
        let text = b"The quick brown fox jumps over the lazy dog. " as &[u8];
        let data = text.repeat(50);

        let compressed = compressor.compress(&data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data, decompressed);
        assert!(compressed.len() < data.len() / 2);
    }

    #[test]
    fn test_http_compressor() {
        let compressor = BrotliHttpCompressor::new();

        // Test HTML compression
        let html = b"<!DOCTYPE html><html><body>Hello</body></html>";
        let compressed = compressor
            .compress_by_content_type(html, "text/html")
            .unwrap();
        assert!(compressed.len() < html.len());

        // Test JSON compression
        let json = br#"{"key": "value", "array": [1, 2, 3]}"#;
        let compressed = compressor
            .compress_by_content_type(json, "application/json")
            .unwrap();
        assert!(compressed.len() < json.len());
    }

    #[test]
    fn test_should_compress() {
        assert!(BrotliHttpCompressor::should_compress("text/html"));
        assert!(BrotliHttpCompressor::should_compress("application/javascript"));
        assert!(BrotliHttpCompressor::should_compress("application/json"));
        assert!(!BrotliHttpCompressor::should_compress("image/jpeg"));
        assert!(!BrotliHttpCompressor::should_compress("video/mp4"));
    }

    #[tokio::test]
    async fn test_brotli_async() {
        let compressor = BrotliCompressor::new();
        let data = bytes::Bytes::from("Async Brotli compression test");

        let compressed = compressor.compress_async(data.clone()).await.unwrap();
        let decompressed = compressor.decompress_async(compressed).await.unwrap();

        assert_eq!(data, decompressed);
    }
}
