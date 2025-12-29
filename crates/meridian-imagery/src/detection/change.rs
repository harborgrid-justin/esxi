//! Change detection algorithms

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;
use super::{DetectionResult, DetectionMetadata};

/// Change detection methods
pub struct ChangeDetection;

impl ChangeDetection {
    /// Image differencing - simple subtraction
    pub fn image_differencing(
        before: &MultiBandImage,
        after: &MultiBandImage,
        threshold: f32,
    ) -> Result<DetectionResult> {
        Self::validate_images(before, after)?;

        let size = (before.metadata.width * before.metadata.height) as usize;
        let mut mask = vec![0u8; size];
        let mut count = 0;

        // Calculate difference for each band and combine
        for pixel_idx in 0..size {
            let mut total_diff = 0.0;

            for band_idx in 0..before.bands.len() {
                let diff = (after.bands[band_idx][pixel_idx] - before.bands[band_idx][pixel_idx]).abs();
                total_diff += diff;
            }

            let avg_diff = total_diff / before.bands.len() as f32;

            if avg_diff > threshold {
                mask[pixel_idx] = 1;
                count += 1;
            }
        }

        Ok(DetectionResult {
            mask,
            width: before.metadata.width,
            height: before.metadata.height,
            metadata: DetectionMetadata {
                algorithm: "Image Differencing".to_string(),
                threshold,
                count,
            },
        })
    }

    /// Change Vector Analysis (CVA)
    pub fn change_vector_analysis(
        before: &MultiBandImage,
        after: &MultiBandImage,
        threshold: f32,
    ) -> Result<DetectionResult> {
        Self::validate_images(before, after)?;

        let size = (before.metadata.width * before.metadata.height) as usize;
        let mut mask = vec![0u8; size];
        let mut count = 0;

        for pixel_idx in 0..size {
            // Calculate magnitude of change vector
            let mut magnitude = 0.0;

            for band_idx in 0..before.bands.len() {
                let diff = after.bands[band_idx][pixel_idx] - before.bands[band_idx][pixel_idx];
                magnitude += diff * diff;
            }

            magnitude = magnitude.sqrt();

            if magnitude > threshold {
                mask[pixel_idx] = 1;
                count += 1;
            }
        }

        Ok(DetectionResult {
            mask,
            width: before.metadata.width,
            height: before.metadata.height,
            metadata: DetectionMetadata {
                algorithm: "Change Vector Analysis".to_string(),
                threshold,
                count,
            },
        })
    }

    /// Principal Component Analysis (PCA) change detection
    pub fn pca_change_detection(
        before: &MultiBandImage,
        after: &MultiBandImage,
        threshold: f32,
    ) -> Result<DetectionResult> {
        Self::validate_images(before, after)?;

        // Simplified PCA-based change detection
        // Full implementation would perform PCA on stacked images

        Self::change_vector_analysis(before, after, threshold)
    }

    /// Normalized Difference change detection
    pub fn normalized_difference(
        before: &MultiBandImage,
        after: &MultiBandImage,
        band: usize,
        threshold: f32,
    ) -> Result<DetectionResult> {
        Self::validate_images(before, after)?;

        if band >= before.bands.len() {
            return Err(ImageryError::InvalidBand {
                band,
                total: before.bands.len(),
            });
        }

        let size = (before.metadata.width * before.metadata.height) as usize;
        let mut mask = vec![0u8; size];
        let mut count = 0;

        for pixel_idx in 0..size {
            let b1 = before.bands[band][pixel_idx];
            let b2 = after.bands[band][pixel_idx];

            let sum = b1 + b2;
            let nd = if sum.abs() < 1e-10 {
                0.0
            } else {
                (b2 - b1) / sum
            };

            if nd.abs() > threshold {
                mask[pixel_idx] = 1;
                count += 1;
            }
        }

        Ok(DetectionResult {
            mask,
            width: before.metadata.width,
            height: before.metadata.height,
            metadata: DetectionMetadata {
                algorithm: "Normalized Difference".to_string(),
                threshold,
                count,
            },
        })
    }

    /// Multivariate Alteration Detection (MAD)
    pub fn multivariate_alteration_detection(
        before: &MultiBandImage,
        after: &MultiBandImage,
        threshold: f32,
    ) -> Result<DetectionResult> {
        Self::validate_images(before, after)?;

        // MAD uses canonical correlation analysis
        // Simplified implementation using CVA
        Self::change_vector_analysis(before, after, threshold)
    }

    /// Post-classification comparison
    pub fn post_classification_comparison(
        before_labels: &[u32],
        after_labels: &[u32],
        width: u32,
        height: u32,
    ) -> Result<DetectionResult> {
        if before_labels.len() != after_labels.len() {
            return Err(ImageryError::InvalidDimensions(
                "Label arrays must have same size".to_string()
            ));
        }

        let size = before_labels.len();
        let mut mask = vec![0u8; size];
        let mut count = 0;

        for i in 0..size {
            if before_labels[i] != after_labels[i] {
                mask[i] = 1;
                count += 1;
            }
        }

        Ok(DetectionResult {
            mask,
            width,
            height,
            metadata: DetectionMetadata {
                algorithm: "Post-Classification Comparison".to_string(),
                threshold: 0.0,
                count,
            },
        })
    }

    /// Validate images for change detection
    fn validate_images(before: &MultiBandImage, after: &MultiBandImage) -> Result<()> {
        if before.metadata.width != after.metadata.width ||
           before.metadata.height != after.metadata.height {
            return Err(ImageryError::InvalidDimensions(
                "Images must have same dimensions".to_string()
            ));
        }

        if before.bands.len() != after.bands.len() {
            return Err(ImageryError::InvalidParameter(
                "Images must have same number of bands".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_image_differencing() {
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

        let mut before = MultiBandImage::new(metadata.clone(), DataType::UInt8);
        let mut after = MultiBandImage::new(metadata, DataType::UInt8);

        before.bands[0].fill(100.0);
        after.bands[0].fill(150.0);

        let result = ChangeDetection::image_differencing(&before, &after, 40.0);
        assert!(result.is_ok());

        let detection = result.unwrap();
        assert_eq!(detection.count, 100); // All pixels changed
    }
}
