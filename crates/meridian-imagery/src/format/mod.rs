//! Image format handlers
//!
//! Support for various geospatial imagery formats including:
//! - GeoTIFF: Standard georeferenced TIFF
//! - COG: Cloud Optimized GeoTIFF
//! - JPEG2000: Wavelet-based compression
//! - NITF: National Imagery Transmission Format

pub mod geotiff;
pub mod cog;
pub mod jpeg2000;
pub mod nitf;

pub use geotiff::{GeoTiffReader, GeoTiffWriter};
pub use cog::{CogReader, CogWriter};

use crate::error::Result;
use crate::MultiBandImage;

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// GeoTIFF format
    GeoTiff,
    /// Cloud Optimized GeoTIFF
    Cog,
    /// JPEG2000
    Jpeg2000,
    /// NITF (National Imagery Transmission Format)
    Nitf,
    /// PNG
    Png,
    /// JPEG
    Jpeg,
}

impl ImageFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "tif" | "tiff" => Some(Self::GeoTiff),
            "jp2" | "j2k" => Some(Self::Jpeg2000),
            "ntf" | "nitf" => Some(Self::Nitf),
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            _ => None,
        }
    }

    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::GeoTiff => "tif",
            Self::Cog => "tif",
            Self::Jpeg2000 => "jp2",
            Self::Nitf => "ntf",
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }
}

/// Compression methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    /// No compression
    None,
    /// DEFLATE compression (ZIP)
    Deflate,
    /// LZW compression
    Lzw,
    /// JPEG compression
    Jpeg {
        /// Quality (1-100)
        quality: u8,
    },
    /// JPEG2000 compression
    Jpeg2000 {
        /// Compression ratio
        ratio: u8,
    },
}

impl Default for Compression {
    fn default() -> Self {
        Self::Deflate
    }
}

/// Image reader trait
pub trait ImageReader {
    /// Open an image file
    fn open(path: impl AsRef<std::path::Path>) -> Result<Self>
    where
        Self: Sized;

    /// Read the full image
    fn read(&mut self) -> Result<MultiBandImage>;

    /// Read a specific region
    fn read_region(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<MultiBandImage>;

    /// Get image metadata
    fn metadata(&self) -> &crate::ImageMetadata;
}

/// Image writer trait
pub trait ImageWriter {
    /// Create a new writer
    fn new(path: impl AsRef<std::path::Path>) -> Result<Self>
    where
        Self: Sized;

    /// Write an image
    fn write(&mut self, image: &MultiBandImage) -> Result<()>;

    /// Set compression method
    fn with_compression(&mut self, compression: Compression) -> &mut Self;
}
