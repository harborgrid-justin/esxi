//! Image mosaicking
//!
//! Seamlessly combine multiple overlapping images into a single composite.

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;
use std::collections::HashMap;

/// Mosaic builder for combining multiple images
pub struct MosaicBuilder {
    images: Vec<MultiBandImage>,
    blend_method: BlendMethod,
    feathering_width: u32,
    color_balance: bool,
}

/// Blending methods for mosaics
#[derive(Debug, Clone, Copy)]
pub enum BlendMethod {
    /// Simple averaging in overlap areas
    Average,
    /// Maximum value in overlap areas
    Maximum,
    /// Minimum value in overlap areas
    Minimum,
    /// First image takes precedence
    First,
    /// Last image takes precedence
    Last,
    /// Distance-weighted blending
    DistanceWeighted,
    /// Multi-band blending (Laplacian pyramid)
    MultiBand,
}

impl MosaicBuilder {
    /// Create a new mosaic builder
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            blend_method: BlendMethod::DistanceWeighted,
            feathering_width: 50,
            color_balance: true,
        }
    }

    /// Add an image to the mosaic
    pub fn add_image(&mut self, image: MultiBandImage) -> &mut Self {
        self.images.push(image);
        self
    }

    /// Set blending method
    pub fn with_blend_method(&mut self, method: BlendMethod) -> &mut Self {
        self.blend_method = method;
        self
    }

    /// Set feathering width for blending
    pub fn with_feathering(&mut self, width: u32) -> &mut Self {
        self.feathering_width = width;
        self
    }

    /// Enable/disable color balancing
    pub fn with_color_balance(&mut self, enable: bool) -> &mut Self {
        self.color_balance = enable;
        self
    }

    /// Build the mosaic
    pub fn build(&self) -> Result<MultiBandImage> {
        if self.images.is_empty() {
            return Err(ImageryError::InvalidParameter(
                "No images added to mosaic".to_string()
            ));
        }

        // Calculate output bounds
        let (min_x, min_y, max_x, max_y) = self.calculate_bounds()?;

        let output_width = (max_x - min_x) as u32;
        let output_height = (max_y - min_y) as u32;

        let mut output_meta = self.images[0].metadata.clone();
        output_meta.width = output_width;
        output_meta.height = output_height;

        // Update geotransform if available
        if let Some(ref mut gt) = output_meta.geo_transform {
            gt[0] = min_x;
            gt[3] = max_y;
        }

        let mut output = MultiBandImage::new(output_meta, self.images[0].data_type);

        // Color balance images if enabled
        let images = if self.color_balance {
            self.balance_colors(&self.images)?
        } else {
            self.images.clone()
        };

        // Blend images
        match self.blend_method {
            BlendMethod::Average => self.blend_average(&images, &mut output, min_x, min_y)?,
            BlendMethod::Maximum => self.blend_max(&images, &mut output, min_x, min_y)?,
            BlendMethod::Minimum => self.blend_min(&images, &mut output, min_x, min_y)?,
            BlendMethod::First => self.blend_first(&images, &mut output, min_x, min_y)?,
            BlendMethod::Last => self.blend_last(&images, &mut output, min_x, min_y)?,
            BlendMethod::DistanceWeighted => self.blend_distance_weighted(&images, &mut output, min_x, min_y)?,
            BlendMethod::MultiBand => self.blend_multiband(&images, &mut output, min_x, min_y)?,
        }

        Ok(output)
    }

    /// Calculate mosaic bounds
    fn calculate_bounds(&self) -> Result<(f64, f64, f64, f64)> {
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for image in &self.images {
            if let Some(gt) = &image.metadata.geo_transform {
                let x1 = gt[0];
                let y1 = gt[3];
                let x2 = gt[0] + image.metadata.width as f64 * gt[1];
                let y2 = gt[3] + image.metadata.height as f64 * gt[5];

                min_x = min_x.min(x1);
                min_y = min_y.min(y2);
                max_x = max_x.max(x2);
                max_y = max_y.max(y1);
            }
        }

        Ok((min_x, min_y, max_x, max_y))
    }

    /// Average blending
    fn blend_average(
        &self,
        images: &[MultiBandImage],
        output: &mut MultiBandImage,
        offset_x: f64,
        offset_y: f64,
    ) -> Result<()> {
        let mut counts = vec![0u32; (output.metadata.width * output.metadata.height) as usize];

        for image in images {
            self.add_image_to_output(image, output, &mut counts, offset_x, offset_y, |sum, val, _| sum + val)?;
        }

        // Divide by count
        for band in output.bands.iter_mut() {
            for (pixel, &count) in band.iter_mut().zip(counts.iter()) {
                if count > 0 {
                    *pixel /= count as f32;
                }
            }
        }

        Ok(())
    }

    /// Maximum value blending
    fn blend_max(
        &self,
        images: &[MultiBandImage],
        output: &mut MultiBandImage,
        offset_x: f64,
        offset_y: f64,
    ) -> Result<()> {
        for image in images {
            let mut counts = vec![0u32; (output.metadata.width * output.metadata.height) as usize];
            self.add_image_to_output(image, output, &mut counts, offset_x, offset_y, |cur, val, _| cur.max(val))?;
        }
        Ok(())
    }

    /// Minimum value blending
    fn blend_min(
        &self,
        images: &[MultiBandImage],
        output: &mut MultiBandImage,
        offset_x: f64,
        offset_y: f64,
    ) -> Result<()> {
        // Initialize with maximum values
        for band in output.bands.iter_mut() {
            band.fill(f32::MAX);
        }

        for image in images {
            let mut counts = vec![0u32; (output.metadata.width * output.metadata.height) as usize];
            self.add_image_to_output(image, output, &mut counts, offset_x, offset_y, |cur, val, _| cur.min(val))?;
        }

        // Replace MAX values with 0
        for band in output.bands.iter_mut() {
            for pixel in band.iter_mut() {
                if *pixel == f32::MAX {
                    *pixel = 0.0;
                }
            }
        }

        Ok(())
    }

    /// First image priority blending
    fn blend_first(
        &self,
        images: &[MultiBandImage],
        output: &mut MultiBandImage,
        offset_x: f64,
        offset_y: f64,
    ) -> Result<()> {
        let mut filled = vec![false; (output.metadata.width * output.metadata.height) as usize];

        for image in images {
            // Only write to unfilled pixels
            let mut counts = vec![0u32; filled.len()];
            self.add_image_to_output(image, output, &mut counts, offset_x, offset_y, |cur, val, idx| {
                if !filled[idx] {
                    filled[idx] = true;
                    val
                } else {
                    cur
                }
            })?;
        }

        Ok(())
    }

    /// Last image priority blending
    fn blend_last(
        &self,
        images: &[MultiBandImage],
        output: &mut MultiBandImage,
        offset_x: f64,
        offset_y: f64,
    ) -> Result<()> {
        for image in images {
            let mut counts = vec![0u32; (output.metadata.width * output.metadata.height) as usize];
            self.add_image_to_output(image, output, &mut counts, offset_x, offset_y, |_, val, _| val)?;
        }
        Ok(())
    }

    /// Distance-weighted blending
    fn blend_distance_weighted(
        &self,
        images: &[MultiBandImage],
        output: &mut MultiBandImage,
        offset_x: f64,
        offset_y: f64,
    ) -> Result<()> {
        // Calculate distance transforms for each image
        let mut weighted_sum = vec![vec![0.0; (output.metadata.width * output.metadata.height) as usize]; output.bands.len()];
        let mut weight_sum = vec![0.0; (output.metadata.width * output.metadata.height) as usize];

        for image in images {
            let distance_map = self.calculate_distance_map(image);

            // Add weighted values
            if let Some(gt) = &image.metadata.geo_transform {
                let img_offset_x = ((gt[0] - offset_x) / gt[1]) as i32;
                let img_offset_y = ((gt[3] - offset_y) / gt[5]) as i32;

                for y in 0..image.metadata.height {
                    for x in 0..image.metadata.width {
                        let out_x = x as i32 + img_offset_x;
                        let out_y = y as i32 + img_offset_y;

                        if out_x >= 0 && out_x < output.metadata.width as i32 &&
                           out_y >= 0 && out_y < output.metadata.height as i32 {
                            let src_idx = (y * image.metadata.width + x) as usize;
                            let dst_idx = (out_y as u32 * output.metadata.width + out_x as u32) as usize;

                            let weight = distance_map[src_idx];

                            for (band_idx, band) in image.bands.iter().enumerate() {
                                weighted_sum[band_idx][dst_idx] += band[src_idx] * weight;
                            }
                            weight_sum[dst_idx] += weight;
                        }
                    }
                }
            }
        }

        // Normalize by total weight
        for band_idx in 0..output.bands.len() {
            for (idx, pixel) in output.bands[band_idx].iter_mut().enumerate() {
                if weight_sum[idx] > 0.0 {
                    *pixel = weighted_sum[band_idx][idx] / weight_sum[idx];
                }
            }
        }

        Ok(())
    }

    /// Multi-band blending (placeholder)
    fn blend_multiband(
        &self,
        images: &[MultiBandImage],
        output: &mut MultiBandImage,
        offset_x: f64,
        offset_y: f64,
    ) -> Result<()> {
        // Multi-band blending using Laplacian pyramids
        // Placeholder: would implement full pyramid blending
        self.blend_distance_weighted(images, output, offset_x, offset_y)
    }

    /// Helper to add image to output with custom blending function
    fn add_image_to_output<F>(
        &self,
        image: &MultiBandImage,
        output: &mut MultiBandImage,
        counts: &mut [u32],
        offset_x: f64,
        offset_y: f64,
        blend_fn: F,
    ) -> Result<()>
    where
        F: Fn(f32, f32, usize) -> f32,
    {
        if let Some(gt) = &image.metadata.geo_transform {
            let img_offset_x = ((gt[0] - offset_x) / gt[1]) as i32;
            let img_offset_y = ((gt[3] - offset_y) / gt[5]) as i32;

            for y in 0..image.metadata.height {
                for x in 0..image.metadata.width {
                    let out_x = x as i32 + img_offset_x;
                    let out_y = y as i32 + img_offset_y;

                    if out_x >= 0 && out_x < output.metadata.width as i32 &&
                       out_y >= 0 && out_y < output.metadata.height as i32 {
                        let src_idx = (y * image.metadata.width + x) as usize;
                        let dst_idx = (out_y as u32 * output.metadata.width + out_x as u32) as usize;

                        for (band_idx, band) in image.bands.iter().enumerate() {
                            let current = output.bands[band_idx][dst_idx];
                            let new_value = band[src_idx];
                            output.bands[band_idx][dst_idx] = blend_fn(current, new_value, dst_idx);
                        }

                        counts[dst_idx] += 1;
                    }
                }
            }
        }

        Ok(())
    }

    /// Calculate distance transform (distance from edge)
    fn calculate_distance_map(&self, image: &MultiBandImage) -> Vec<f32> {
        let width = image.metadata.width as usize;
        let height = image.metadata.height as usize;
        let mut distances = vec![0.0; width * height];

        for y in 0..height {
            for x in 0..width {
                // Simple distance: minimum distance to any edge
                let dx = x.min(width - 1 - x) as f32;
                let dy = y.min(height - 1 - y) as f32;
                distances[y * width + x] = dx.min(dy);
            }
        }

        // Normalize
        let max_dist = distances.iter().copied().fold(0.0f32, f32::max);
        if max_dist > 0.0 {
            for d in distances.iter_mut() {
                *d /= max_dist;
            }
        }

        distances
    }

    /// Balance colors across images using histogram matching
    fn balance_colors(&self, images: &[MultiBandImage]) -> Result<Vec<MultiBandImage>> {
        if images.is_empty() {
            return Ok(vec![]);
        }

        // Use first image as reference
        let reference = &images[0];
        let mut balanced = vec![reference.clone()];

        for image in images.iter().skip(1) {
            let mut adjusted = image.clone();

            // Match histogram for each band
            for band_idx in 0..image.bands.len().min(reference.bands.len()) {
                Self::histogram_match(
                    &mut adjusted.bands[band_idx],
                    &reference.bands[band_idx],
                );
            }

            balanced.push(adjusted);
        }

        Ok(balanced)
    }

    /// Histogram matching
    fn histogram_match(source: &mut [f32], reference: &[f32]) {
        // Calculate CDFs
        let source_cdf = Self::calculate_cdf(source);
        let reference_cdf = Self::calculate_cdf(reference);

        // Create lookup table
        let mut lut = [0.0; 256];
        for i in 0..256 {
            let source_val = source_cdf[i];
            let mut closest_idx = 0;
            let mut min_diff = f32::MAX;

            for (j, &ref_val) in reference_cdf.iter().enumerate() {
                let diff = (source_val - ref_val).abs();
                if diff < min_diff {
                    min_diff = diff;
                    closest_idx = j;
                }
            }

            lut[i] = closest_idx as f32;
        }

        // Apply lookup table
        for pixel in source.iter_mut() {
            let idx = (*pixel).min(255.0).max(0.0) as usize;
            *pixel = lut[idx];
        }
    }

    /// Calculate cumulative distribution function
    fn calculate_cdf(data: &[f32]) -> Vec<f32> {
        let mut histogram = vec![0u32; 256];

        // Build histogram
        for &value in data {
            let bin = (value.min(255.0).max(0.0)) as usize;
            histogram[bin] += 1;
        }

        // Calculate CDF
        let mut cdf = vec![0.0; 256];
        let total = data.len() as f32;
        let mut cumsum = 0u32;

        for (i, &count) in histogram.iter().enumerate() {
            cumsum += count;
            cdf[i] = cumsum as f32 / total;
        }

        cdf
    }
}

impl Default for MosaicBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_mosaic_builder() {
        let mut builder = MosaicBuilder::new();
        assert_eq!(builder.images.len(), 0);

        let metadata = ImageMetadata {
            width: 100,
            height: 100,
            bands: 3,
            bits_per_sample: 8,
            geo_transform: Some([0.0, 1.0, 0.0, 100.0, 0.0, -1.0]),
            crs: None,
            no_data: None,
            band_names: vec!["R".to_string(), "G".to_string(), "B".to_string()],
        };

        let image = MultiBandImage::new(metadata, DataType::UInt8);
        builder.add_image(image);

        assert_eq!(builder.images.len(), 1);
    }
}
