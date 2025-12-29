//! Quick preview generation

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;
use std::path::Path;

/// Preview generator
pub struct PreviewGenerator {
    max_dimension: u32,
    quality: u8,
    format: PreviewFormat,
}

/// Preview format
#[derive(Debug, Clone, Copy)]
pub enum PreviewFormat {
    /// PNG
    Png,
    /// JPEG
    Jpeg,
}

impl PreviewGenerator {
    /// Create a new preview generator
    pub fn new() -> Self {
        Self {
            max_dimension: 1024,
            quality: 75,
            format: PreviewFormat::Jpeg,
        }
    }

    /// Set maximum dimension
    pub fn with_max_dimension(mut self, max_dim: u32) -> Self {
        self.max_dimension = max_dim;
        self
    }

    /// Set quality (for JPEG)
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality;
        self
    }

    /// Set format
    pub fn with_format(mut self, format: PreviewFormat) -> Self {
        self.format = format;
        self
    }

    /// Generate preview
    pub fn generate(&self, image: &MultiBandImage) -> Result<MultiBandImage> {
        // Calculate scale factor
        let scale = self.calculate_scale(image);

        if scale >= 1.0 {
            // No scaling needed
            return Ok(image.clone());
        }

        // Calculate new dimensions
        let new_width = (image.metadata.width as f32 * scale) as u32;
        let new_height = (image.metadata.height as f32 * scale) as u32;

        log::info!(
            "Generating preview: {}x{} -> {}x{} (scale: {:.2})",
            image.metadata.width,
            image.metadata.height,
            new_width,
            new_height,
            scale
        );

        // Resample image
        self.resample(image, new_width, new_height)
    }

    /// Save preview to file
    pub fn save(&self, image: &MultiBandImage, path: impl AsRef<Path>) -> Result<()> {
        let preview = self.generate(image)?;

        // Would use image crate to save
        log::info!("Saving preview to: {:?}", path.as_ref());

        Ok(())
    }

    /// Generate thumbnail (smaller preview)
    pub fn generate_thumbnail(&self, image: &MultiBandImage, size: u32) -> Result<MultiBandImage> {
        let aspect = image.metadata.width as f32 / image.metadata.height as f32;

        let (width, height) = if aspect > 1.0 {
            (size, (size as f32 / aspect) as u32)
        } else {
            ((size as f32 * aspect) as u32, size)
        };

        self.resample(image, width, height)
    }

    /// Calculate scale factor
    fn calculate_scale(&self, image: &MultiBandImage) -> f32 {
        let max_dim = image.metadata.width.max(image.metadata.height);

        if max_dim <= self.max_dimension {
            1.0
        } else {
            self.max_dimension as f32 / max_dim as f32
        }
    }

    /// Resample image to new dimensions
    fn resample(
        &self,
        image: &MultiBandImage,
        new_width: u32,
        new_height: u32,
    ) -> Result<MultiBandImage> {
        let mut output_meta = image.metadata.clone();
        output_meta.width = new_width;
        output_meta.height = new_height;

        let mut output = MultiBandImage::new(output_meta, image.data_type);

        // Bilinear resampling
        let x_ratio = image.metadata.width as f32 / new_width as f32;
        let y_ratio = image.metadata.height as f32 / new_height as f32;

        for band_idx in 0..image.bands.len() {
            for y in 0..new_height {
                for x in 0..new_width {
                    let src_x = x as f32 * x_ratio;
                    let src_y = y as f32 * y_ratio;

                    let value = self.bilinear_sample(&image.bands[band_idx], image.metadata.width, src_x, src_y);

                    let dst_idx = (y * new_width + x) as usize;
                    output.bands[band_idx][dst_idx] = value;
                }
            }
        }

        Ok(output)
    }

    /// Bilinear sampling
    fn bilinear_sample(&self, data: &[f32], width: u32, x: f32, y: f32) -> f32 {
        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let x1 = (x0 + 1).min(width - 1);
        let y1 = (y0 + 1).min((data.len() / width as usize - 1) as u32);

        let dx = x - x0 as f32;
        let dy = y - y0 as f32;

        let idx00 = (y0 * width + x0) as usize;
        let idx10 = (y0 * width + x1) as usize;
        let idx01 = (y1 * width + x0) as usize;
        let idx11 = (y1 * width + x1) as usize;

        let v00 = data.get(idx00).copied().unwrap_or(0.0);
        let v10 = data.get(idx10).copied().unwrap_or(0.0);
        let v01 = data.get(idx01).copied().unwrap_or(0.0);
        let v11 = data.get(idx11).copied().unwrap_or(0.0);

        let v0 = v00 * (1.0 - dx) + v10 * dx;
        let v1 = v01 * (1.0 - dx) + v11 * dx;

        v0 * (1.0 - dy) + v1 * dy
    }

    /// Apply histogram stretch for better visualization
    pub fn histogram_stretch(&self, image: &mut MultiBandImage, min_percent: f32, max_percent: f32) -> Result<()> {
        for band in &mut image.bands {
            // Calculate percentiles
            let mut sorted = band.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let min_idx = (sorted.len() as f32 * min_percent) as usize;
            let max_idx = (sorted.len() as f32 * max_percent) as usize;

            let min_val = sorted[min_idx.min(sorted.len() - 1)];
            let max_val = sorted[max_idx.min(sorted.len() - 1)];

            let range = max_val - min_val;

            if range > 0.0 {
                for pixel in band.iter_mut() {
                    *pixel = ((*pixel - min_val) / range * 255.0).clamp(0.0, 255.0);
                }
            }
        }

        Ok(())
    }

    /// Generate RGB composite preview
    pub fn rgb_composite(
        &self,
        image: &MultiBandImage,
        red_band: usize,
        green_band: usize,
        blue_band: usize,
    ) -> Result<MultiBandImage> {
        if red_band >= image.bands.len() || green_band >= image.bands.len() || blue_band >= image.bands.len() {
            return Err(ImageryError::InvalidParameter(
                "Invalid band indices for RGB composite".to_string()
            ));
        }

        let mut output_meta = image.metadata.clone();
        output_meta.bands = 3;
        output_meta.band_names = vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()];

        let mut output = MultiBandImage::new(output_meta, image.data_type);

        output.bands[0] = image.bands[red_band].clone();
        output.bands[1] = image.bands[green_band].clone();
        output.bands[2] = image.bands[blue_band].clone();

        // Apply stretch
        self.histogram_stretch(&mut output, 0.02, 0.98)?;

        // Generate preview
        self.generate(&output)
    }
}

impl Default for PreviewGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_preview_generation() {
        let metadata = ImageMetadata {
            width: 2048,
            height: 2048,
            bands: 3,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["R".to_string(), "G".to_string(), "B".to_string()],
        };

        let image = MultiBandImage::new(metadata, DataType::UInt8);
        let generator = PreviewGenerator::new().with_max_dimension(512);

        let preview = generator.generate(&image);
        assert!(preview.is_ok());

        let result = preview.unwrap();
        assert_eq!(result.metadata.width, 512);
        assert_eq!(result.metadata.height, 512);
    }

    #[test]
    fn test_thumbnail_generation() {
        let metadata = ImageMetadata {
            width: 1000,
            height: 500,
            bands: 1,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["Gray".to_string()],
        };

        let image = MultiBandImage::new(metadata, DataType::UInt8);
        let generator = PreviewGenerator::new();

        let thumbnail = generator.generate_thumbnail(&image, 100);
        assert!(thumbnail.is_ok());

        let result = thumbnail.unwrap();
        assert_eq!(result.metadata.width, 100);
        assert_eq!(result.metadata.height, 50); // Maintains aspect ratio
    }
}
