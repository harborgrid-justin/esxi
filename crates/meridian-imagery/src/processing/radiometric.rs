//! Radiometric corrections
//!
//! Convert digital numbers to physical units and correct sensor artifacts.

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;

/// Radiometric correction operations
pub struct RadiometricCorrection;

impl RadiometricCorrection {
    /// Apply gain and offset correction
    ///
    /// Converts DN (Digital Numbers) to radiance or reflectance:
    /// `value = gain * DN + offset`
    pub fn gain_offset(image: &mut MultiBandImage, gains: &[f32], offsets: &[f32]) -> Result<()> {
        if gains.len() != image.bands.len() || offsets.len() != image.bands.len() {
            return Err(ImageryError::InvalidParameter(
                "Gains and offsets must match number of bands".to_string()
            ));
        }

        for (band_idx, band) in image.bands.iter_mut().enumerate() {
            let gain = gains[band_idx];
            let offset = offsets[band_idx];

            for pixel in band.iter_mut() {
                *pixel = gain * (*pixel) + offset;
            }
        }

        Ok(())
    }

    /// Convert DN to Top of Atmosphere (TOA) radiance
    pub fn to_radiance(
        image: &mut MultiBandImage,
        gains: &[f32],
        offsets: &[f32],
    ) -> Result<()> {
        Self::gain_offset(image, gains, offsets)
    }

    /// Convert TOA radiance to TOA reflectance
    pub fn to_reflectance(
        image: &mut MultiBandImage,
        solar_zenith: f32,
        sun_earth_distance: f32,
        esun: &[f32],
    ) -> Result<()> {
        if esun.len() != image.bands.len() {
            return Err(ImageryError::InvalidParameter(
                "ESUN values must match number of bands".to_string()
            ));
        }

        let cos_zenith = solar_zenith.to_radians().cos();
        let distance_factor = sun_earth_distance * sun_earth_distance;

        for (band_idx, band) in image.bands.iter_mut().enumerate() {
            let esun_value = esun[band_idx];
            let factor = std::f32::consts::PI * distance_factor / (esun_value * cos_zenith);

            for pixel in band.iter_mut() {
                *pixel = (*pixel) * factor;
            }
        }

        Ok(())
    }

    /// Dark object subtraction (simple haze removal)
    pub fn dark_object_subtraction(image: &mut MultiBandImage) -> Result<()> {
        for band in image.bands.iter_mut() {
            // Find minimum value (dark object)
            let min_value = band.iter()
                .filter(|v| v.is_finite() && **v > 0.0)
                .copied()
                .fold(f32::INFINITY, f32::min);

            // Subtract from all pixels
            if min_value.is_finite() {
                for pixel in band.iter_mut() {
                    *pixel = (*pixel - min_value).max(0.0);
                }
            }
        }

        Ok(())
    }

    /// Hot pixel removal (remove outliers)
    pub fn remove_hot_pixels(
        image: &mut MultiBandImage,
        threshold_sigma: f32,
    ) -> Result<()> {
        for band in image.bands.iter_mut() {
            // Calculate mean and standard deviation
            let (mean, std_dev) = Self::calculate_stats(band);

            let threshold = mean + threshold_sigma * std_dev;

            // Replace hot pixels with mean
            for pixel in band.iter_mut() {
                if *pixel > threshold {
                    *pixel = mean;
                }
            }
        }

        Ok(())
    }

    /// Dead pixel removal (replace with neighbor average)
    pub fn remove_dead_pixels(
        image: &mut MultiBandImage,
        threshold: f32,
    ) -> Result<()> {
        let width = image.metadata.width;
        let height = image.metadata.height;

        for band_idx in 0..image.bands.len() {
            let mut corrections = Vec::new();

            for y in 1..(height - 1) {
                for x in 1..(width - 1) {
                    let idx = (y * width + x) as usize;
                    let value = image.bands[band_idx][idx];

                    // Check if pixel is dead (too low)
                    if value < threshold {
                        // Calculate neighbor average
                        let mut sum = 0.0;
                        let mut count = 0;

                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                if dx == 0 && dy == 0 {
                                    continue;
                                }

                                let nx = (x as i32 + dx) as usize;
                                let ny = (y as i32 + dy) as usize;
                                let nidx = ny * width as usize + nx;

                                sum += image.bands[band_idx][nidx];
                                count += 1;
                            }
                        }

                        corrections.push((idx, sum / count as f32));
                    }
                }
            }

            // Apply corrections
            for (idx, value) in corrections {
                image.bands[band_idx][idx] = value;
            }
        }

        Ok(())
    }

    /// Calculate mean and standard deviation
    fn calculate_stats(data: &[f32]) -> (f32, f32) {
        let n = data.len() as f32;
        let mean = data.iter().sum::<f32>() / n;

        let variance = data.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / n;

        (mean, variance.sqrt())
    }

    /// Apply flat field correction
    pub fn flat_field_correction(
        image: &mut MultiBandImage,
        flat_field: &MultiBandImage,
    ) -> Result<()> {
        if image.metadata.width != flat_field.metadata.width ||
           image.metadata.height != flat_field.metadata.height ||
           image.bands.len() != flat_field.bands.len() {
            return Err(ImageryError::InvalidDimensions(
                "Image and flat field dimensions must match".to_string()
            ));
        }

        for band_idx in 0..image.bands.len() {
            for (pixel, flat) in image.bands[band_idx].iter_mut()
                .zip(flat_field.bands[band_idx].iter()) {
                if *flat > 0.0 {
                    *pixel = *pixel / *flat;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_gain_offset_correction() {
        let metadata = ImageMetadata {
            width: 10,
            height: 10,
            bands: 1,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["Band1".to_string()],
        };

        let mut image = MultiBandImage::new(metadata, DataType::UInt8);
        image.bands[0].fill(100.0);

        let gains = vec![0.5];
        let offsets = vec![10.0];

        RadiometricCorrection::gain_offset(&mut image, &gains, &offsets).unwrap();

        assert_eq!(image.bands[0][0], 60.0); // 0.5 * 100 + 10
    }

    #[test]
    fn test_dark_object_subtraction() {
        let metadata = ImageMetadata {
            width: 10,
            height: 10,
            bands: 1,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["Band1".to_string()],
        };

        let mut image = MultiBandImage::new(metadata, DataType::UInt8);
        image.bands[0][0] = 10.0; // Minimum
        image.bands[0][1] = 50.0;
        image.bands[0][2] = 100.0;

        RadiometricCorrection::dark_object_subtraction(&mut image).unwrap();

        assert_eq!(image.bands[0][0], 0.0);
        assert_eq!(image.bands[0][1], 40.0);
        assert_eq!(image.bands[0][2], 90.0);
    }
}
