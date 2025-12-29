//! # Meridian Imagery
//!
//! Satellite and aerial imagery processing for Meridian GIS Platform v0.2.5.
//!
//! ## Features
//!
//! - **Multi-format Support**: GeoTIFF, COG, JPEG2000, NITF
//! - **Image Processing**: Radiometric correction, atmospheric correction, orthorectification
//! - **Spectral Indices**: NDVI, EVI, NDWI, MNDWI, NDBI
//! - **Classification**: Supervised and unsupervised classification
//! - **Object Detection**: Change detection, image segmentation
//! - **STAC Integration**: Catalog search and metadata management
//! - **Streaming Processing**: Memory-efficient windowed and parallel processing
//! - **Cloud Optimized**: COG support with HTTP range requests
//!
//! ## Example
//!
//! ```rust,no_run
//! use meridian_imagery::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load a GeoTIFF image
//! let mut image = GeoTiffReader::open("satellite.tif")?;
//!
//! // Calculate NDVI
//! let ndvi = VegetationIndices::ndvi(&image, 3, 2)?;
//!
//! // Apply atmospheric correction
//! let corrected = AtmosphericCorrection::dos1(&mut image)?;
//!
//! // Export as Cloud Optimized GeoTIFF
//! CogWriter::new("output.tif")
//!     .with_compression(Compression::Deflate)
//!     .write(&corrected)?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs, rust_2018_idioms)]
#![allow(dead_code, unused_variables, unused_imports)]

pub mod error;
pub mod format;
pub mod processing;
pub mod indices;
pub mod classification;
pub mod detection;
pub mod catalog;
pub mod streaming;
pub mod export;

pub use error::{ImageryError, Result};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::error::{ImageryError, Result};
    pub use crate::format::{
        GeoTiffReader, GeoTiffWriter,
        CogReader, CogWriter,
        ImageFormat, Compression,
    };
    pub use crate::processing::{
        RadiometricCorrection,
        AtmosphericCorrection,
        Pansharpening,
        Orthorectification,
        MosaicBuilder,
    };
    pub use crate::indices::{
        VegetationIndices,
        WaterIndices,
        BuiltUpIndices,
        BandMath,
    };
    pub use crate::classification::{
        SupervisedClassifier,
        UnsupervisedClassifier,
        MaximumLikelihood,
        KMeans,
    };
    pub use crate::detection::{
        ChangeDetection,
        ImageSegmentation,
        ObjectDetector,
    };
    pub use crate::catalog::{
        StacCatalog,
        ImagerySearch,
        StacItem,
    };
    pub use crate::streaming::{
        WindowedProcessor,
        ParallelProcessor,
        StreamingReader,
    };
    pub use crate::export::{
        TilePyramid,
        PreviewGenerator,
        ExportOptions,
    };
}

/// Common image metadata structure
#[derive(Debug, Clone)]
pub struct ImageMetadata {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Number of bands
    pub bands: u32,
    /// Bits per sample
    pub bits_per_sample: u16,
    /// Geographic transform [x_origin, pixel_width, 0, y_origin, 0, -pixel_height]
    pub geo_transform: Option<[f64; 6]>,
    /// Coordinate Reference System (WKT or EPSG code)
    pub crs: Option<String>,
    /// No data value
    pub no_data: Option<f64>,
    /// Band names
    pub band_names: Vec<String>,
}

/// Band data type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    /// Unsigned 8-bit integer
    UInt8,
    /// Unsigned 16-bit integer
    UInt16,
    /// Signed 16-bit integer
    Int16,
    /// Unsigned 32-bit integer
    UInt32,
    /// Signed 32-bit integer
    Int32,
    /// 32-bit floating point
    Float32,
    /// 64-bit floating point
    Float64,
}

impl DataType {
    /// Get the size in bytes
    pub fn size(&self) -> usize {
        match self {
            DataType::UInt8 => 1,
            DataType::UInt16 | DataType::Int16 => 2,
            DataType::UInt32 | DataType::Int32 | DataType::Float32 => 4,
            DataType::Float64 => 8,
        }
    }
}

/// Multi-band image data
#[derive(Debug, Clone)]
pub struct MultiBandImage {
    /// Image metadata
    pub metadata: ImageMetadata,
    /// Band data (each band is flattened row-major)
    pub bands: Vec<Vec<f32>>,
    /// Data type
    pub data_type: DataType,
}

impl MultiBandImage {
    /// Create a new multi-band image
    pub fn new(metadata: ImageMetadata, data_type: DataType) -> Self {
        let size = (metadata.width * metadata.height) as usize;
        let bands = vec![vec![0.0; size]; metadata.bands as usize];

        Self {
            metadata,
            bands,
            data_type,
        }
    }

    /// Get pixel value at (x, y) for a specific band
    pub fn get_pixel(&self, band: usize, x: u32, y: u32) -> Option<f32> {
        if band >= self.bands.len() || x >= self.metadata.width || y >= self.metadata.height {
            return None;
        }

        let idx = (y * self.metadata.width + x) as usize;
        Some(self.bands[band][idx])
    }

    /// Set pixel value at (x, y) for a specific band
    pub fn set_pixel(&mut self, band: usize, x: u32, y: u32, value: f32) -> Result<()> {
        if band >= self.bands.len() || x >= self.metadata.width || y >= self.metadata.height {
            return Err(ImageryError::InvalidBand {
                band,
                total: self.bands.len(),
            });
        }

        let idx = (y * self.metadata.width + x) as usize;
        self.bands[band][idx] = value;
        Ok(())
    }

    /// Get a reference to a band's data
    pub fn band(&self, index: usize) -> Option<&[f32]> {
        self.bands.get(index).map(|v| v.as_slice())
    }

    /// Get a mutable reference to a band's data
    pub fn band_mut(&mut self, index: usize) -> Option<&mut [f32]> {
        self.bands.get_mut(index).map(|v| v.as_mut_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiband_image_creation() {
        let metadata = ImageMetadata {
            width: 100,
            height: 100,
            bands: 3,
            bits_per_sample: 16,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()],
        };

        let image = MultiBandImage::new(metadata, DataType::UInt16);
        assert_eq!(image.bands.len(), 3);
        assert_eq!(image.bands[0].len(), 10000);
    }

    #[test]
    fn test_pixel_operations() {
        let metadata = ImageMetadata {
            width: 10,
            height: 10,
            bands: 1,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["Gray".to_string()],
        };

        let mut image = MultiBandImage::new(metadata, DataType::UInt8);

        image.set_pixel(0, 5, 5, 255.0).unwrap();
        assert_eq!(image.get_pixel(0, 5, 5), Some(255.0));
    }
}
