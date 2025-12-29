//! Tile encoding modules

pub mod compression;
pub mod mvt;
pub mod pbf;

pub use compression::{compress_brotli, compress_gzip, decompress_brotli, decompress_gzip};
pub use mvt::MvtEncoder;

use crate::error::Result;

/// Compression format for tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionFormat {
    /// No compression
    None,
    /// Gzip compression
    Gzip,
    /// Brotli compression
    Brotli,
}

impl CompressionFormat {
    /// Get the content-encoding header value
    pub fn content_encoding(&self) -> Option<&'static str> {
        match self {
            CompressionFormat::None => None,
            CompressionFormat::Gzip => Some("gzip"),
            CompressionFormat::Brotli => Some("br"),
        }
    }

    /// Get the file extension
    pub fn extension(&self) -> &'static str {
        match self {
            CompressionFormat::None => "",
            CompressionFormat::Gzip => ".gz",
            CompressionFormat::Brotli => ".br",
        }
    }

    /// Parse from content-encoding header
    pub fn from_content_encoding(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "gzip" | "x-gzip" => Some(CompressionFormat::Gzip),
            "br" => Some(CompressionFormat::Brotli),
            "identity" | "" => Some(CompressionFormat::None),
            _ => None,
        }
    }
}

/// Compress data with the specified format
pub fn compress(data: &[u8], format: CompressionFormat) -> Result<Vec<u8>> {
    match format {
        CompressionFormat::None => Ok(data.to_vec()),
        CompressionFormat::Gzip => compress_gzip(data),
        CompressionFormat::Brotli => compress_brotli(data),
    }
}

/// Decompress data with the specified format
pub fn decompress(data: &[u8], format: CompressionFormat) -> Result<Vec<u8>> {
    match format {
        CompressionFormat::None => Ok(data.to_vec()),
        CompressionFormat::Gzip => decompress_gzip(data),
        CompressionFormat::Brotli => decompress_brotli(data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_format() {
        assert_eq!(
            CompressionFormat::Gzip.content_encoding(),
            Some("gzip")
        );
        assert_eq!(
            CompressionFormat::Brotli.content_encoding(),
            Some("br")
        );
        assert_eq!(CompressionFormat::None.content_encoding(), None);
    }

    #[test]
    fn test_from_content_encoding() {
        assert_eq!(
            CompressionFormat::from_content_encoding("gzip"),
            Some(CompressionFormat::Gzip)
        );
        assert_eq!(
            CompressionFormat::from_content_encoding("br"),
            Some(CompressionFormat::Brotli)
        );
    }
}
