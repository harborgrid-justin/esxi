//! Image segmentation algorithms

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;
use super::{DetectionResult, DetectionMetadata};

/// Image segmentation methods
pub struct ImageSegmentation;

impl ImageSegmentation {
    /// Threshold-based segmentation
    pub fn threshold(
        image: &MultiBandImage,
        band: usize,
        threshold: f32,
    ) -> Result<DetectionResult> {
        if band >= image.bands.len() {
            return Err(ImageryError::InvalidBand {
                band,
                total: image.bands.len(),
            });
        }

        let size = (image.metadata.width * image.metadata.height) as usize;
        let mut mask = vec![0u8; size];
        let mut count = 0;

        for (idx, &value) in image.bands[band].iter().enumerate() {
            if value > threshold {
                mask[idx] = 1;
                count += 1;
            }
        }

        Ok(DetectionResult {
            mask,
            width: image.metadata.width,
            height: image.metadata.height,
            metadata: DetectionMetadata {
                algorithm: "Threshold".to_string(),
                threshold,
                count,
            },
        })
    }

    /// Region growing segmentation
    pub fn region_growing(
        image: &MultiBandImage,
        seed_points: &[(u32, u32)],
        similarity_threshold: f32,
    ) -> Result<DetectionResult> {
        let width = image.metadata.width as usize;
        let height = image.metadata.height as usize;
        let size = width * height;

        let mut mask = vec![0u8; size];
        let mut visited = vec![false; size];

        for (seed_idx, &(seed_x, seed_y)) in seed_points.iter().enumerate() {
            if seed_x >= image.metadata.width || seed_y >= image.metadata.height {
                continue;
            }

            let mut queue = vec![(seed_x, seed_y)];
            let seed_idx_flat = (seed_y * image.metadata.width + seed_x) as usize;

            // Get seed pixel values
            let mut seed_values = Vec::with_capacity(image.bands.len());
            for band in &image.bands {
                seed_values.push(band[seed_idx_flat]);
            }

            while let Some((x, y)) = queue.pop() {
                let idx = (y * image.metadata.width + x) as usize;

                if visited[idx] {
                    continue;
                }

                visited[idx] = true;

                // Check similarity
                let mut similar = true;
                for (band_idx, band) in image.bands.iter().enumerate() {
                    if (band[idx] - seed_values[band_idx]).abs() > similarity_threshold {
                        similar = false;
                        break;
                    }
                }

                if similar {
                    mask[idx] = (seed_idx + 1) as u8;

                    // Add neighbors
                    if x > 0 {
                        queue.push((x - 1, y));
                    }
                    if x < image.metadata.width - 1 {
                        queue.push((x + 1, y));
                    }
                    if y > 0 {
                        queue.push((x, y - 1));
                    }
                    if y < image.metadata.height - 1 {
                        queue.push((x, y + 1));
                    }
                }
            }
        }

        let count = mask.iter().filter(|&&v| v > 0).count();

        Ok(DetectionResult {
            mask,
            width: image.metadata.width,
            height: image.metadata.height,
            metadata: DetectionMetadata {
                algorithm: "Region Growing".to_string(),
                threshold: similarity_threshold,
                count,
            },
        })
    }

    /// Watershed segmentation (simplified)
    pub fn watershed(
        image: &MultiBandImage,
        band: usize,
    ) -> Result<DetectionResult> {
        if band >= image.bands.len() {
            return Err(ImageryError::InvalidBand {
                band,
                total: image.bands.len(),
            });
        }

        // Simplified watershed - use threshold-based approach
        // Full implementation would use gradient images and flooding

        let data = &image.bands[band];
        let mean = data.iter().sum::<f32>() / data.len() as f32;

        Self::threshold(image, band, mean)
    }

    /// Edge-based segmentation using gradient
    pub fn edge_detection(
        image: &MultiBandImage,
        band: usize,
        threshold: f32,
    ) -> Result<DetectionResult> {
        if band >= image.bands.len() {
            return Err(ImageryError::InvalidBand {
                band,
                total: image.bands.len(),
            });
        }

        let width = image.metadata.width as usize;
        let height = image.metadata.height as usize;
        let size = width * height;

        let mut mask = vec![0u8; size];
        let mut count = 0;

        let data = &image.bands[band];

        // Simple Sobel edge detection
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let idx = y * width + x;

                // Sobel kernels
                let gx = -data[idx - width - 1] - 2.0 * data[idx - 1] - data[idx + width - 1]
                       + data[idx - width + 1] + 2.0 * data[idx + 1] + data[idx + width + 1];

                let gy = -data[idx - width - 1] - 2.0 * data[idx - width] - data[idx - width + 1]
                       + data[idx + width - 1] + 2.0 * data[idx + width] + data[idx + width + 1];

                let magnitude = (gx * gx + gy * gy).sqrt();

                if magnitude > threshold {
                    mask[idx] = 1;
                    count += 1;
                }
            }
        }

        Ok(DetectionResult {
            mask,
            width: image.metadata.width,
            height: image.metadata.height,
            metadata: DetectionMetadata {
                algorithm: "Edge Detection".to_string(),
                threshold,
                count,
            },
        })
    }

    /// Morphological operations - erosion
    pub fn erosion(mask: &mut DetectionResult, kernel_size: usize) -> Result<()> {
        let width = mask.width as usize;
        let height = mask.height as usize;
        let mut new_mask = mask.mask.clone();

        let offset = kernel_size / 2;

        for y in offset..(height - offset) {
            for x in offset..(width - offset) {
                let idx = y * width + x;

                // Check if all neighbors are 1
                let mut all_set = true;
                for dy in 0..kernel_size {
                    for dx in 0..kernel_size {
                        let ny = y - offset + dy;
                        let nx = x - offset + dx;
                        let nidx = ny * width + nx;

                        if mask.mask[nidx] == 0 {
                            all_set = false;
                            break;
                        }
                    }
                    if !all_set {
                        break;
                    }
                }

                new_mask[idx] = if all_set { 1 } else { 0 };
            }
        }

        mask.mask = new_mask;
        mask.metadata.count = mask.mask.iter().filter(|&&v| v > 0).count();

        Ok(())
    }

    /// Morphological operations - dilation
    pub fn dilation(mask: &mut DetectionResult, kernel_size: usize) -> Result<()> {
        let width = mask.width as usize;
        let height = mask.height as usize;
        let mut new_mask = mask.mask.clone();

        let offset = kernel_size / 2;

        for y in offset..(height - offset) {
            for x in offset..(width - offset) {
                let idx = y * width + x;

                // Check if any neighbor is 1
                let mut any_set = false;
                for dy in 0..kernel_size {
                    for dx in 0..kernel_size {
                        let ny = y - offset + dy;
                        let nx = x - offset + dx;
                        let nidx = ny * width + nx;

                        if mask.mask[nidx] == 1 {
                            any_set = true;
                            break;
                        }
                    }
                    if any_set {
                        break;
                    }
                }

                new_mask[idx] = if any_set { 1 } else { 0 };
            }
        }

        mask.mask = new_mask;
        mask.metadata.count = mask.mask.iter().filter(|&&v| v > 0).count();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_threshold_segmentation() {
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

        // Half pixels above threshold, half below
        for i in 0..50 {
            image.bands[0][i] = 100.0;
        }
        for i in 50..100 {
            image.bands[0][i] = 200.0;
        }

        let result = ImageSegmentation::threshold(&image, 0, 150.0);
        assert!(result.is_ok());

        let segmentation = result.unwrap();
        assert_eq!(segmentation.count, 50);
    }
}
