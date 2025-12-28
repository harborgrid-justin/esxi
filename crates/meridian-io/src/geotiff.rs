//! GeoTIFF raster support

use crate::error::{IoError, Result};
use crate::traits::Metadata;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tiff::decoder::{Decoder, DecodingResult};
use tiff::tags::Tag;

/// GeoTIFF metadata and georeferencing information
#[derive(Debug, Clone)]
pub struct GeoTiffInfo {
    /// Image width in pixels
    pub width: u32,

    /// Image height in pixels
    pub height: u32,

    /// Number of bands/channels
    pub bands: u16,

    /// Bits per sample
    pub bits_per_sample: Vec<u16>,

    /// Compression method
    pub compression: Option<String>,

    /// Photometric interpretation
    pub photometric: Option<String>,

    /// GeoTIFF tags (ModelPixelScale, ModelTiepoint, etc.)
    pub geo_tags: HashMap<String, Vec<f64>>,

    /// Coordinate reference system (from GeoKey directory)
    pub crs: Option<String>,

    /// Affine transformation parameters [a, b, c, d, e, f]
    /// x = a + b*col + c*row
    /// y = d + e*col + f*row
    pub transform: Option<[f64; 6]>,

    /// Bounding box [min_x, min_y, max_x, max_y]
    pub bbox: Option<[f64; 4]>,
}

/// GeoTIFF reader
pub struct GeoTiffReader;

impl GeoTiffReader {
    /// Create a new GeoTIFF reader
    pub fn new() -> Self {
        Self
    }

    /// Read GeoTIFF metadata and georeferencing
    pub fn read_info(&self, path: &Path) -> Result<GeoTiffInfo> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut decoder = Decoder::new(reader)?;

        let (width, height) = decoder.dimensions()?;
        let colortype = decoder.colortype()?;

        // Get bits per sample
        let bits_per_sample = match decoder.find_tag(Tag::BitsPerSample) {
            Ok(Some(tiff::decoder::Value::Unsigned(v))) => {
                v.iter().map(|&x| x as u16).collect()
            }
            _ => vec![8], // Default to 8 bits
        };

        // Determine number of bands
        let bands = match colortype {
            tiff::ColorType::Gray(_) => 1,
            tiff::ColorType::RGB(_) => 3,
            tiff::ColorType::RGBA(_) => 4,
            tiff::ColorType::CMYK(_) => 4,
            _ => 1,
        };

        // Get compression
        let compression = match decoder.find_tag(Tag::Compression) {
            Ok(Some(tiff::decoder::Value::Unsigned(v))) => {
                Some(match v[0] {
                    1 => "None",
                    5 => "LZW",
                    7 => "JPEG",
                    8 => "Deflate",
                    32773 => "PackBits",
                    _ => "Unknown",
                }.to_string())
            }
            _ => None,
        };

        // Get photometric interpretation
        let photometric = match decoder.find_tag(Tag::PhotometricInterpretation) {
            Ok(Some(tiff::decoder::Value::Unsigned(v))) => {
                Some(match v[0] {
                    0 => "WhiteIsZero",
                    1 => "BlackIsZero",
                    2 => "RGB",
                    3 => "Palette",
                    4 => "TransparencyMask",
                    5 => "CMYK",
                    6 => "YCbCr",
                    _ => "Unknown",
                }.to_string())
            }
            _ => None,
        };

        let mut geo_tags = HashMap::new();
        let mut transform = None;
        let mut bbox = None;

        // Read GeoTIFF tags
        // ModelPixelScaleTag (33550)
        if let Ok(Some(tiff::decoder::Value::Double(scale))) = decoder.find_tag(Tag::Unknown(33550)) {
            geo_tags.insert("ModelPixelScale".to_string(), scale.clone());

            // Read ModelTiepointTag (33922) for georeferencing
            if let Ok(Some(tiff::decoder::Value::Double(tiepoint))) = decoder.find_tag(Tag::Unknown(33922)) {
                geo_tags.insert("ModelTiepoint".to_string(), tiepoint.clone());

                // Calculate transform from tiepoint and scale
                // Tiepoint format: [I, J, K, X, Y, Z, ...]
                // Scale format: [ScaleX, ScaleY, ScaleZ]
                if tiepoint.len() >= 6 && scale.len() >= 2 {
                    let i = tiepoint[0];
                    let j = tiepoint[1];
                    let x = tiepoint[3];
                    let y = tiepoint[4];
                    let scale_x = scale[0];
                    let scale_y = -scale[1].abs(); // Y scale is typically negative

                    // Calculate upper-left corner
                    let origin_x = x - i * scale_x;
                    let origin_y = y - j * scale_y;

                    transform = Some([
                        origin_x,  // a: x origin
                        scale_x,   // b: x pixel size
                        0.0,       // c: x rotation
                        origin_y,  // d: y origin
                        0.0,       // e: y rotation
                        scale_y,   // f: y pixel size
                    ]);

                    // Calculate bounding box
                    let max_x = origin_x + (width as f64) * scale_x;
                    let max_y = origin_y + (height as f64) * scale_y;

                    bbox = Some([
                        origin_x.min(max_x),
                        origin_y.min(max_y),
                        origin_x.max(max_x),
                        origin_y.max(max_y),
                    ]);
                }
            }
        }

        // Read ModelTransformationTag (34264) if available
        if let Ok(Some(tiff::decoder::Value::Double(matrix))) = decoder.find_tag(Tag::Unknown(34264)) {
            geo_tags.insert("ModelTransformation".to_string(), matrix.clone());

            // Extract transform from 4x4 transformation matrix
            if matrix.len() >= 12 {
                transform = Some([
                    matrix[3],  // a: translation X
                    matrix[0],  // b: scale X
                    matrix[1],  // c: rotation/shear X
                    matrix[7],  // d: translation Y
                    matrix[4],  // e: rotation/shear Y
                    matrix[5],  // f: scale Y
                ]);
            }
        }

        // TODO: Read GeoKey directory for CRS information (tag 34735)
        let crs = None; // Placeholder - would need GeoKey parsing

        Ok(GeoTiffInfo {
            width,
            height,
            bands,
            bits_per_sample,
            compression,
            photometric,
            geo_tags,
            crs,
            transform,
            bbox,
        })
    }

    /// Read raster data from a specific band
    pub fn read_band(&self, path: &Path, band: u16) -> Result<Vec<u8>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut decoder = Decoder::new(reader)?;

        // Read the image data
        let image_data = decoder.read_image()?;

        match image_data {
            DecodingResult::U8(data) => Ok(data),
            DecodingResult::U16(data) => {
                // Convert u16 to u8 (simple downsampling)
                Ok(data.iter().map(|&x| (x >> 8) as u8).collect())
            }
            DecodingResult::U32(data) => {
                // Convert u32 to u8
                Ok(data.iter().map(|&x| (x >> 24) as u8).collect())
            }
            DecodingResult::U64(data) => {
                // Convert u64 to u8
                Ok(data.iter().map(|&x| (x >> 56) as u8).collect())
            }
            _ => Err(IoError::GeoTiff("Unsupported TIFF data type".to_string())),
        }
    }

    /// Read all bands as separate vectors
    pub fn read_all_bands(&self, path: &Path) -> Result<Vec<Vec<u8>>> {
        let info = self.read_info(path)?;

        let mut bands = Vec::new();
        for band_idx in 0..info.bands {
            bands.push(self.read_band(path, band_idx)?);
        }

        Ok(bands)
    }

    /// Convert to common metadata format
    pub fn read_metadata(&self, path: &Path) -> Result<Metadata> {
        let info = self.read_info(path)?;

        let mut metadata = Metadata::new();
        metadata.layers.push("raster".to_string());
        metadata.crs = info.crs;
        metadata.geometry_types.push("Raster".to_string());

        if let Some(bbox) = info.bbox {
            metadata.bbox = Some(bbox.to_vec());
        }

        // Add raster-specific metadata
        metadata.additional.insert("width".to_string(), info.width.to_string());
        metadata.additional.insert("height".to_string(), info.height.to_string());
        metadata.additional.insert("bands".to_string(), info.bands.to_string());

        if let Some(ref compression) = info.compression {
            metadata.additional.insert("compression".to_string(), compression.clone());
        }

        if let Some(ref photometric) = info.photometric {
            metadata.additional.insert("photometric".to_string(), photometric.clone());
        }

        Ok(metadata)
    }
}

impl Default for GeoTiffReader {
    fn default() -> Self {
        Self::new()
    }
}

/// GeoTIFF writer
pub struct GeoTiffWriter;

impl GeoTiffWriter {
    /// Create a new GeoTIFF writer
    pub fn new() -> Self {
        Self
    }

    /// Write raster data as GeoTIFF
    pub fn write(
        &self,
        _path: &Path,
        _width: u32,
        _height: u32,
        _data: &[u8],
        _transform: Option<[f64; 6]>,
        _crs: Option<&str>,
    ) -> Result<()> {
        // TODO: Implement GeoTIFF writing
        Err(IoError::UnsupportedFormat("GeoTIFF writing not yet implemented".to_string()))
    }
}

impl Default for GeoTiffWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geotiff_reader_creation() {
        let _reader = GeoTiffReader::new();
    }

    #[test]
    fn test_geotiff_writer_creation() {
        let _writer = GeoTiffWriter::new();
    }
}
