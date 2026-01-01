//! Gzip compression compatibility layer
//!
//! Universal compression format with wide compatibility.

use crate::error::{CompressionError, Result};
use crate::stats::{CompressionStats, Timer};
use crate::Compressor;
use async_trait::async_trait;
use flate2::write::{GzEncoder, GzDecoder};
use flate2::{Compression, GzBuilder};
use std::io::{Read, Write};

/// Gzip compression configuration
#[derive(Debug, Clone)]
pub struct GzipConfig {
    /// Compression level (0-9)
    pub level: u32,
    /// Enable CRC32 checksum
    pub enable_crc: bool,
    /// Set modification time
    pub mtime: Option<u32>,
    /// Set filename in header
    pub filename: Option<String>,
    /// Set comment in header
    pub comment: Option<String>,
}

impl Default for GzipConfig {
    fn default() -> Self {
        Self {
            level: 6,
            enable_crc: true,
            mtime: None,
            filename: None,
            comment: None,
        }
    }
}

/// Gzip compressor
pub struct GzipCompressor {
    config: GzipConfig,
}

impl Default for GzipCompressor {
    fn default() -> Self {
        Self {
            config: GzipConfig::default(),
        }
    }
}

impl GzipCompressor {
    /// Create a new Gzip compressor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom configuration
    pub fn with_config(config: GzipConfig) -> Self {
        Self { config }
    }

    /// Create with specific compression level
    pub fn with_level(level: u32) -> Self {
        Self {
            config: GzipConfig {
                level: level.min(9),
                ..Default::default()
            },
        }
    }

    /// Set filename in gzip header
    pub fn with_filename(mut self, filename: String) -> Self {
        self.config.filename = Some(filename);
        self
    }

    /// Set comment in gzip header
    pub fn with_comment(mut self, comment: String) -> Self {
        self.config.comment = Some(comment);
        self
    }

    /// Compress with statistics
    pub fn compress_with_stats(&self, data: &[u8]) -> Result<(Vec<u8>, CompressionStats)> {
        let timer = Timer::start("gzip_compress");
        let compressed = self.compress(data)?;
        let elapsed = timer.stop();

        let stats = CompressionStats::new(
            data.len(),
            compressed.len(),
            elapsed,
            "gzip",
        )
        .with_level(self.config.level as i32);

        Ok((compressed, stats))
    }

    /// Create encoder with configuration
    fn create_encoder<W: Write>(&self, writer: W) -> GzEncoder<W> {
        let compression = Compression::new(self.config.level);
        let mut builder = GzBuilder::new();

        if let Some(ref filename) = self.config.filename {
            builder.filename(filename.as_bytes());
        }

        if let Some(ref comment) = self.config.comment {
            builder.comment(comment.as_bytes());
        }

        if let Some(mtime) = self.config.mtime {
            builder.mtime(mtime);
        }

        builder.write(writer, compression)
    }

    /// Compress stream
    pub fn compress_stream<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<usize> {
        let mut encoder = self.create_encoder(writer);

        let bytes_written = std::io::copy(reader, &mut encoder)
            .map_err(|e| CompressionError::gzip(e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| CompressionError::gzip(e.to_string()))?;

        Ok(bytes_written as usize)
    }

    /// Decompress stream
    pub fn decompress_stream<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<usize> {
        let mut decoder = GzDecoder::new(writer);

        let bytes_read = std::io::copy(reader, &mut decoder)
            .map_err(|e| CompressionError::gzip(e.to_string()))?;

        decoder
            .finish()
            .map_err(|e| CompressionError::gzip(e.to_string()))?;

        Ok(bytes_read as usize)
    }

    /// Validate gzip header
    pub fn validate_header(data: &[u8]) -> bool {
        data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
    }
}

#[async_trait]
impl Compressor for GzipCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = self.create_encoder(Vec::new());

        encoder
            .write_all(data)
            .map_err(|e| CompressionError::gzip(e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| CompressionError::gzip(e.to_string()))
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !Self::validate_header(data) {
            return Err(CompressionError::gzip(
                "Invalid gzip header".to_string(),
            ));
        }

        use flate2::read::GzDecoder;
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| CompressionError::gzip(e.to_string()))?;

        Ok(decompressed)
    }

    fn algorithm(&self) -> &str {
        "gzip"
    }

    fn level(&self) -> Option<i32> {
        Some(self.config.level as i32)
    }
}

/// Deflate compressor (raw deflate without gzip wrapper)
pub struct DeflateCompressor {
    level: u32,
}

impl Default for DeflateCompressor {
    fn default() -> Self {
        Self { level: 6 }
    }
}

impl DeflateCompressor {
    /// Create a new deflate compressor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific level
    pub fn with_level(level: u32) -> Self {
        Self { level: level.min(9) }
    }
}

#[async_trait]
impl Compressor for DeflateCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::write::DeflateEncoder;

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(self.level));

        encoder
            .write_all(data)
            .map_err(|e| CompressionError::gzip(e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| CompressionError::gzip(e.to_string()))
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::DeflateDecoder;

        let mut decoder = DeflateDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| CompressionError::gzip(e.to_string()))?;

        Ok(decompressed)
    }

    fn algorithm(&self) -> &str {
        "deflate"
    }

    fn level(&self) -> Option<i32> {
        Some(self.level as i32)
    }
}

/// Zlib compressor (deflate with zlib wrapper)
pub struct ZlibCompressor {
    level: u32,
}

impl Default for ZlibCompressor {
    fn default() -> Self {
        Self { level: 6 }
    }
}

impl ZlibCompressor {
    /// Create a new zlib compressor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with specific level
    pub fn with_level(level: u32) -> Self {
        Self { level: level.min(9) }
    }
}

#[async_trait]
impl Compressor for ZlibCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::write::ZlibEncoder;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(self.level));

        encoder
            .write_all(data)
            .map_err(|e| CompressionError::gzip(e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| CompressionError::gzip(e.to_string()))
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::ZlibDecoder;

        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| CompressionError::gzip(e.to_string()))?;

        Ok(decompressed)
    }

    fn algorithm(&self) -> &str {
        "zlib"
    }

    fn level(&self) -> Option<i32> {
        Some(self.level as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gzip_compress_decompress() {
        let compressor = GzipCompressor::new();
        let data = b"Hello, Gzip compression!";

        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.as_slice(), decompressed.as_slice());
        assert!(GzipCompressor::validate_header(&compressed));
    }

    #[test]
    fn test_gzip_levels() {
        let data = b"Test data" as &[u8];
        let data = data.repeat(1000);

        let low = GzipCompressor::with_level(1);
        let high = GzipCompressor::with_level(9);

        let compressed_low = low.compress(&data).unwrap();
        let compressed_high = high.compress(&data).unwrap();

        assert!(compressed_high.len() <= compressed_low.len());
    }

    #[test]
    fn test_gzip_with_metadata() {
        let compressor = GzipCompressor::new()
            .with_filename("test.txt".to_string())
            .with_comment("Test file".to_string());

        let data = b"Data with metadata";
        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_deflate_compress_decompress() {
        let compressor = DeflateCompressor::new();
        let data = b"Deflate compression test";

        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_zlib_compress_decompress() {
        let compressor = ZlibCompressor::new();
        let data = b"Zlib compression test";

        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[tokio::test]
    async fn test_gzip_async() {
        let compressor = GzipCompressor::new();
        let data = bytes::Bytes::from("Async gzip test");

        let compressed = compressor.compress_async(data.clone()).await.unwrap();
        let decompressed = compressor.decompress_async(compressed).await.unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_format_compatibility() {
        let data = b"Test data";

        // Each format should decompress its own compression
        let gzip = GzipCompressor::new();
        let deflate = DeflateCompressor::new();
        let zlib = ZlibCompressor::new();

        let gzip_compressed = gzip.compress(data).unwrap();
        let deflate_compressed = deflate.compress(data).unwrap();
        let zlib_compressed = zlib.compress(data).unwrap();

        assert_eq!(gzip.decompress(&gzip_compressed).unwrap(), data);
        assert_eq!(deflate.decompress(&deflate_compressed).unwrap(), data);
        assert_eq!(zlib.decompress(&zlib_compressed).unwrap(), data);

        // Formats should not be cross-compatible
        assert!(gzip.decompress(&deflate_compressed).is_err());
        assert!(deflate.decompress(&gzip_compressed).is_err());
    }
}
