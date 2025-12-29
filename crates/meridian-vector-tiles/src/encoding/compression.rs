//! Tile compression (Gzip and Brotli)

use crate::error::{Error, Result};
use flate2::write::{GzEncoder, GzDecoder};
use flate2::Compression;
use std::io::Write;

/// Compress data using gzip
pub fn compress_gzip(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data)
        .map_err(|e| Error::compression(format!("Gzip compression failed: {}", e)))?;
    encoder
        .finish()
        .map_err(|e| Error::compression(format!("Gzip finish failed: {}", e)))
}

/// Decompress gzip data
pub fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(Vec::new());
    decoder
        .write_all(data)
        .map_err(|e| Error::decompression(format!("Gzip decompression failed: {}", e)))?;
    decoder
        .finish()
        .map_err(|e| Error::decompression(format!("Gzip finish failed: {}", e)))
}

/// Compress data using brotli
pub fn compress_brotli(data: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut writer = brotli::CompressorWriter::new(&mut output, 4096, 11, 22);
    writer
        .write_all(data)
        .map_err(|e| Error::compression(format!("Brotli compression failed: {}", e)))?;
    drop(writer);
    Ok(output)
}

/// Decompress brotli data
pub fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut reader = brotli::Decompressor::new(data, 4096);
    std::io::copy(&mut reader, &mut output)
        .map_err(|e| Error::decompression(format!("Brotli decompression failed: {}", e)))?;
    Ok(output)
}

/// Compression level for gzip
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GzipLevel {
    /// No compression
    None,
    /// Fast compression
    Fast,
    /// Default compression
    Default,
    /// Best compression
    Best,
    /// Custom level (0-9)
    Level(u32),
}

impl From<GzipLevel> for Compression {
    fn from(level: GzipLevel) -> Self {
        match level {
            GzipLevel::None => Compression::none(),
            GzipLevel::Fast => Compression::fast(),
            GzipLevel::Default => Compression::default(),
            GzipLevel::Best => Compression::best(),
            GzipLevel::Level(n) => Compression::new(n),
        }
    }
}

/// Compress data using gzip with custom level
pub fn compress_gzip_level(data: &[u8], level: GzipLevel) -> Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), level.into());
    encoder
        .write_all(data)
        .map_err(|e| Error::compression(format!("Gzip compression failed: {}", e)))?;
    encoder
        .finish()
        .map_err(|e| Error::compression(format!("Gzip finish failed: {}", e)))
}

/// Brotli compression quality (0-11)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrotliQuality(pub u32);

impl BrotliQuality {
    pub const MIN: u32 = 0;
    pub const MAX: u32 = 11;
    pub const DEFAULT: u32 = 6;

    pub fn new(quality: u32) -> Self {
        Self(quality.clamp(Self::MIN, Self::MAX))
    }
}

impl Default for BrotliQuality {
    fn default() -> Self {
        Self::new(Self::DEFAULT)
    }
}

/// Compress data using brotli with custom quality
pub fn compress_brotli_quality(data: &[u8], quality: BrotliQuality) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut writer = brotli::CompressorWriter::new(&mut output, 4096, quality.0, 22);
    writer
        .write_all(data)
        .map_err(|e| Error::compression(format!("Brotli compression failed: {}", e)))?;
    drop(writer);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gzip_roundtrip() {
        let data = b"Hello, World! This is a test of gzip compression.";
        let compressed = compress_gzip(data).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = decompress_gzip(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_brotli_roundtrip() {
        let data = b"Hello, World! This is a test of brotli compression.";
        let compressed = compress_brotli(data).unwrap();
        assert!(compressed.len() < data.len());

        let decompressed = decompress_brotli(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_gzip_levels() {
        let data = b"Test data for compression levels";

        let fast = compress_gzip_level(data, GzipLevel::Fast).unwrap();
        let best = compress_gzip_level(data, GzipLevel::Best).unwrap();

        // Best compression should be smaller or equal
        assert!(best.len() <= fast.len());
    }

    #[test]
    fn test_brotli_quality() {
        let quality = BrotliQuality::new(15); // Should clamp to 11
        assert_eq!(quality.0, 11);

        let quality = BrotliQuality::default();
        assert_eq!(quality.0, 6);
    }
}
