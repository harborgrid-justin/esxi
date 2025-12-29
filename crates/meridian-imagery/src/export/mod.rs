//! Export and output generation

pub mod tiles;
pub mod preview;

pub use tiles::TilePyramid;
pub use preview::PreviewGenerator;

use crate::error::Result;
use crate::MultiBandImage;
use crate::format::Compression;

/// Export options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Output format
    pub format: ExportFormat,
    /// Compression method
    pub compression: Compression,
    /// Quality (for lossy formats)
    pub quality: u8,
    /// Include metadata
    pub include_metadata: bool,
    /// Generate overviews
    pub generate_overviews: bool,
}

impl ExportOptions {
    /// Create default export options
    pub fn new(format: ExportFormat) -> Self {
        Self {
            format,
            compression: Compression::default(),
            quality: 85,
            include_metadata: true,
            generate_overviews: false,
        }
    }

    /// Set compression
    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.compression = compression;
        self
    }

    /// Set quality
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality;
        self
    }

    /// Enable/disable metadata
    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Enable/disable overviews
    pub fn with_overviews(mut self, generate: bool) -> Self {
        self.generate_overviews = generate;
        self
    }
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self::new(ExportFormat::GeoTiff)
    }
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// GeoTIFF
    GeoTiff,
    /// Cloud Optimized GeoTIFF
    Cog,
    /// PNG
    Png,
    /// JPEG
    Jpeg,
    /// JPEG2000
    Jpeg2000,
}

impl ExportFormat {
    /// Get file extension
    pub fn extension(&self) -> &str {
        match self {
            Self::GeoTiff | Self::Cog => "tif",
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Jpeg2000 => "jp2",
        }
    }
}
