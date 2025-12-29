//! Custom band math operations
//!
//! Flexible band math for creating custom spectral indices and transformations.

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;
use super::IndexResult;

/// Custom band math calculator
pub struct BandMath;

impl BandMath {
    /// Apply a custom formula to bands
    ///
    /// Supports basic arithmetic operations on bands
    pub fn apply<F>(
        image: &MultiBandImage,
        band_indices: &[usize],
        operation: F,
        name: impl Into<String>,
    ) -> Result<IndexResult>
    where
        F: Fn(&[f32]) -> f32,
    {
        // Validate band indices
        for &idx in band_indices {
            if idx >= image.bands.len() {
                return Err(ImageryError::InvalidBand {
                    band: idx,
                    total: image.bands.len(),
                });
            }
        }

        let size = (image.metadata.width * image.metadata.height) as usize;
        let mut values = vec![0.0; size];

        // Extract band values for each pixel
        for pixel_idx in 0..size {
            let mut band_values = Vec::with_capacity(band_indices.len());
            for &band_idx in band_indices {
                band_values.push(image.bands[band_idx][pixel_idx]);
            }
            values[pixel_idx] = operation(&band_values);
        }

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: name.into(),
        })
    }

    /// Add two bands
    pub fn add(
        image: &MultiBandImage,
        band1: usize,
        band2: usize,
    ) -> Result<IndexResult> {
        Self::apply(image, &[band1, band2], |b| b[0] + b[1], "Add")
    }

    /// Subtract two bands
    pub fn subtract(
        image: &MultiBandImage,
        band1: usize,
        band2: usize,
    ) -> Result<IndexResult> {
        Self::apply(image, &[band1, band2], |b| b[0] - b[1], "Subtract")
    }

    /// Multiply two bands
    pub fn multiply(
        image: &MultiBandImage,
        band1: usize,
        band2: usize,
    ) -> Result<IndexResult> {
        Self::apply(image, &[band1, band2], |b| b[0] * b[1], "Multiply")
    }

    /// Divide two bands
    pub fn divide(
        image: &MultiBandImage,
        band1: usize,
        band2: usize,
    ) -> Result<IndexResult> {
        Self::apply(
            image,
            &[band1, band2],
            |b| if b[1].abs() < 1e-10 { 0.0 } else { b[0] / b[1] },
            "Divide",
        )
    }

    /// Scale a band by a constant
    pub fn scale(
        image: &MultiBandImage,
        band: usize,
        factor: f32,
    ) -> Result<IndexResult> {
        Self::apply(image, &[band], move |b| b[0] * factor, "Scale")
    }

    /// Apply offset to a band
    pub fn offset(
        image: &MultiBandImage,
        band: usize,
        offset: f32,
    ) -> Result<IndexResult> {
        Self::apply(image, &[band], move |b| b[0] + offset, "Offset")
    }

    /// Calculate band ratio
    pub fn ratio(
        image: &MultiBandImage,
        band1: usize,
        band2: usize,
    ) -> Result<IndexResult> {
        Self::divide(image, band1, band2)
    }

    /// Calculate normalized difference
    pub fn normalized_difference(
        image: &MultiBandImage,
        band1: usize,
        band2: usize,
    ) -> Result<IndexResult> {
        Self::apply(
            image,
            &[band1, band2],
            |b| {
                let sum = b[0] + b[1];
                if sum.abs() < 1e-10 {
                    0.0
                } else {
                    (b[0] - b[1]) / sum
                }
            },
            "NormalizedDiff",
        )
    }

    /// Calculate band average
    pub fn average(
        image: &MultiBandImage,
        band_indices: &[usize],
    ) -> Result<IndexResult> {
        let n = band_indices.len() as f32;
        Self::apply(
            image,
            band_indices,
            move |b| b.iter().sum::<f32>() / n,
            "Average",
        )
    }

    /// Calculate band maximum
    pub fn maximum(
        image: &MultiBandImage,
        band_indices: &[usize],
    ) -> Result<IndexResult> {
        Self::apply(
            image,
            band_indices,
            |b| b.iter().copied().fold(f32::NEG_INFINITY, f32::max),
            "Maximum",
        )
    }

    /// Calculate band minimum
    pub fn minimum(
        image: &MultiBandImage,
        band_indices: &[usize],
    ) -> Result<IndexResult> {
        Self::apply(
            image,
            band_indices,
            |b| b.iter().copied().fold(f32::INFINITY, f32::min),
            "Minimum",
        )
    }

    /// Apply threshold to create binary mask
    pub fn threshold(
        image: &MultiBandImage,
        band: usize,
        threshold: f32,
    ) -> Result<IndexResult> {
        Self::apply(
            image,
            &[band],
            move |b| if b[0] > threshold { 1.0 } else { 0.0 },
            "Threshold",
        )
    }

    /// Apply range threshold (inclusive)
    pub fn range_threshold(
        image: &MultiBandImage,
        band: usize,
        min: f32,
        max: f32,
    ) -> Result<IndexResult> {
        Self::apply(
            image,
            &[band],
            move |b| if b[0] >= min && b[0] <= max { 1.0 } else { 0.0 },
            "RangeThreshold",
        )
    }

    /// Apply power transformation
    pub fn power(
        image: &MultiBandImage,
        band: usize,
        exponent: f32,
    ) -> Result<IndexResult> {
        Self::apply(
            image,
            &[band],
            move |b| b[0].powf(exponent),
            "Power",
        )
    }

    /// Apply square root
    pub fn sqrt(
        image: &MultiBandImage,
        band: usize,
    ) -> Result<IndexResult> {
        Self::apply(
            image,
            &[band],
            |b| b[0].max(0.0).sqrt(),
            "SquareRoot",
        )
    }

    /// Apply logarithm (natural log)
    pub fn log(
        image: &MultiBandImage,
        band: usize,
    ) -> Result<IndexResult> {
        Self::apply(
            image,
            &[band],
            |b| if b[0] > 0.0 { b[0].ln() } else { 0.0 },
            "Log",
        )
    }

    /// Apply absolute value
    pub fn abs(
        image: &MultiBandImage,
        band: usize,
    ) -> Result<IndexResult> {
        Self::apply(
            image,
            &[band],
            |b| b[0].abs(),
            "Abs",
        )
    }

    /// Linear combination of bands: result = Σ(weight_i * band_i) + offset
    pub fn linear_combination(
        image: &MultiBandImage,
        band_indices: &[usize],
        weights: &[f32],
        offset: f32,
    ) -> Result<IndexResult> {
        if band_indices.len() != weights.len() {
            return Err(ImageryError::InvalidParameter(
                "Number of bands must match number of weights".to_string()
            ));
        }

        let weights_vec = weights.to_vec();
        Self::apply(
            image,
            band_indices,
            move |b| {
                b.iter()
                    .zip(weights_vec.iter())
                    .map(|(val, weight)| val * weight)
                    .sum::<f32>() + offset
            },
            "LinearCombination",
        )
    }

    /// Principal Component Analysis (first component)
    pub fn pca_first_component(
        image: &MultiBandImage,
        band_indices: &[usize],
    ) -> Result<IndexResult> {
        // Simplified PCA - just returns average of normalized bands
        // Full PCA would require covariance matrix calculation

        let mut normalized_bands = Vec::new();

        for &band_idx in band_indices {
            let band = image.band(band_idx).unwrap();

            // Normalize band
            let mean = band.iter().sum::<f32>() / band.len() as f32;
            let std = (band.iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f32>() / band.len() as f32)
                .sqrt();

            let normalized: Vec<f32> = band.iter()
                .map(|v| if std > 0.0 { (v - mean) / std } else { 0.0 })
                .collect();

            normalized_bands.push(normalized);
        }

        // Average normalized bands as first PC approximation
        let size = normalized_bands[0].len();
        let values: Vec<f32> = (0..size)
            .map(|i| {
                normalized_bands.iter()
                    .map(|b| b[i])
                    .sum::<f32>() / normalized_bands.len() as f32
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "PCA1".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_band_add() {
        let metadata = ImageMetadata {
            width: 2,
            height: 2,
            bands: 2,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["B1".to_string(), "B2".to_string()],
        };

        let mut image = MultiBandImage::new(metadata, DataType::UInt8);
        image.bands[0] = vec![10.0, 20.0, 30.0, 40.0];
        image.bands[1] = vec![5.0, 10.0, 15.0, 20.0];

        let result = BandMath::add(&image, 0, 1).unwrap();

        assert_eq!(result.values[0], 15.0);
        assert_eq!(result.values[1], 30.0);
        assert_eq!(result.values[2], 45.0);
        assert_eq!(result.values[3], 60.0);
    }

    #[test]
    fn test_threshold() {
        let metadata = ImageMetadata {
            width: 2,
            height: 2,
            bands: 1,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["B1".to_string()],
        };

        let mut image = MultiBandImage::new(metadata, DataType::UInt8);
        image.bands[0] = vec![10.0, 50.0, 100.0, 150.0];

        let result = BandMath::threshold(&image, 0, 75.0).unwrap();

        assert_eq!(result.values[0], 0.0);
        assert_eq!(result.values[1], 0.0);
        assert_eq!(result.values[2], 1.0);
        assert_eq!(result.values[3], 1.0);
    }

    #[test]
    fn test_linear_combination() {
        let metadata = ImageMetadata {
            width: 2,
            height: 2,
            bands: 3,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["B1".to_string(), "B2".to_string(), "B3".to_string()],
        };

        let mut image = MultiBandImage::new(metadata, DataType::UInt8);
        image.bands[0] = vec![10.0; 4];
        image.bands[1] = vec![20.0; 4];
        image.bands[2] = vec![30.0; 4];

        let result = BandMath::linear_combination(
            &image,
            &[0, 1, 2],
            &[0.5, 0.3, 0.2],
            10.0,
        ).unwrap();

        // 0.5*10 + 0.3*20 + 0.2*30 + 10 = 5 + 6 + 6 + 10 = 27
        assert_eq!(result.values[0], 27.0);
    }
}
