//! Spectral indices
//!
//! Calculate various spectral indices from multi-band imagery for
//! feature extraction and analysis.

pub mod vegetation;
pub mod water;
pub mod built_up;
pub mod custom;

pub use vegetation::VegetationIndices;
pub use water::WaterIndices;
pub use built_up::BuiltUpIndices;
pub use custom::BandMath;

use crate::error::Result;
use crate::MultiBandImage;

/// Generic index calculation result
#[derive(Debug, Clone)]
pub struct IndexResult {
    /// Index values (one per pixel)
    pub values: Vec<f32>,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Index name
    pub name: String,
}

impl IndexResult {
    /// Get value at pixel coordinates
    pub fn get(&self, x: u32, y: u32) -> Option<f32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize;
        self.values.get(idx).copied()
    }

    /// Convert to single-band image
    pub fn to_image(&self) -> MultiBandImage {
        let metadata = crate::ImageMetadata {
            width: self.width,
            height: self.height,
            bands: 1,
            bits_per_sample: 32,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec![self.name.clone()],
        };

        let mut image = MultiBandImage::new(metadata, crate::DataType::Float32);
        image.bands[0] = self.values.clone();
        image
    }
}

/// Helper function to calculate normalized difference
pub fn normalized_difference(band1: &[f32], band2: &[f32]) -> Result<Vec<f32>> {
    if band1.len() != band2.len() {
        return Err(crate::error::ImageryError::InvalidDimensions(
            "Bands must have same size".to_string()
        ));
    }

    let result = band1.iter()
        .zip(band2.iter())
        .map(|(b1, b2)| {
            let sum = b1 + b2;
            if sum.abs() < 1e-10 {
                0.0
            } else {
                (b1 - b2) / sum
            }
        })
        .collect();

    Ok(result)
}

/// Helper function to calculate ratio
pub fn ratio(band1: &[f32], band2: &[f32]) -> Result<Vec<f32>> {
    if band1.len() != band2.len() {
        return Err(crate::error::ImageryError::InvalidDimensions(
            "Bands must have same size".to_string()
        ));
    }

    let result = band1.iter()
        .zip(band2.iter())
        .map(|(b1, b2)| {
            if b2.abs() < 1e-10 {
                0.0
            } else {
                b1 / b2
            }
        })
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_difference() {
        let band1 = vec![100.0, 150.0, 200.0];
        let band2 = vec![50.0, 50.0, 100.0];

        let result = normalized_difference(&band1, &band2).unwrap();

        assert!((result[0] - 0.333).abs() < 0.01); // (100-50)/(100+50)
        assert_eq!(result[1], 0.5); // (150-50)/(150+50)
        assert!((result[2] - 0.333).abs() < 0.01); // (200-100)/(200+100)
    }

    #[test]
    fn test_ratio() {
        let band1 = vec![100.0, 150.0, 200.0];
        let band2 = vec![50.0, 50.0, 100.0];

        let result = ratio(&band1, &band2).unwrap();

        assert_eq!(result[0], 2.0); // 100/50
        assert_eq!(result[1], 3.0); // 150/50
        assert_eq!(result[2], 2.0); // 200/100
    }
}
