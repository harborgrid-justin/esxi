//! Water indices
//!
//! Spectral indices for water body detection and monitoring.

use crate::error::Result;
use crate::MultiBandImage;
use super::{IndexResult, normalized_difference};

/// Water indices calculator
pub struct WaterIndices;

impl WaterIndices {
    /// NDWI - Normalized Difference Water Index (McFeeters 1996)
    ///
    /// NDWI = (Green - NIR) / (Green + NIR)
    ///
    /// Positive values indicate water presence
    pub fn ndwi(image: &MultiBandImage, green_band: usize, nir_band: usize) -> Result<IndexResult> {
        let green = image.band(green_band)
            .ok_or_else(|| crate::error::ImageryError::InvalidBand {
                band: green_band,
                total: image.bands.len(),
            })?;

        let nir = image.band(nir_band)
            .ok_or_else(|| crate::error::ImageryError::InvalidBand {
                band: nir_band,
                total: image.bands.len(),
            })?;

        let values = normalized_difference(green, nir)?;

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "NDWI".to_string(),
        })
    }

    /// MNDWI - Modified Normalized Difference Water Index (Xu 2006)
    ///
    /// MNDWI = (Green - SWIR) / (Green + SWIR)
    ///
    /// Better for built-up areas, uses SWIR instead of NIR
    pub fn mndwi(image: &MultiBandImage, green_band: usize, swir_band: usize) -> Result<IndexResult> {
        let green = image.band(green_band)
            .ok_or_else(|| crate::error::ImageryError::InvalidBand {
                band: green_band,
                total: image.bands.len(),
            })?;

        let swir = image.band(swir_band)
            .ok_or_else(|| crate::error::ImageryError::InvalidBand {
                band: swir_band,
                total: image.bands.len(),
            })?;

        let values = normalized_difference(green, swir)?;

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "MNDWI".to_string(),
        })
    }

    /// AWEI - Automated Water Extraction Index
    ///
    /// AWEInsh = 4 * (Green - SWIR1) - (0.25*NIR + 2.75*SWIR2)
    /// (non-shadow version)
    pub fn awei_nsh(
        image: &MultiBandImage,
        green_band: usize,
        nir_band: usize,
        swir1_band: usize,
        swir2_band: usize,
    ) -> Result<IndexResult> {
        let green = image.band(green_band).unwrap();
        let nir = image.band(nir_band).unwrap();
        let swir1 = image.band(swir1_band).unwrap();
        let swir2 = image.band(swir2_band).unwrap();

        let values: Vec<f32> = green.iter()
            .zip(nir.iter())
            .zip(swir1.iter())
            .zip(swir2.iter())
            .map(|(((&g, &n), &s1), &s2)| {
                4.0 * (g - s1) - (0.25 * n + 2.75 * s2)
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "AWEInsh".to_string(),
        })
    }

    /// WRI - Water Ratio Index
    ///
    /// WRI = (Green + Red) / (NIR + SWIR)
    pub fn wri(
        image: &MultiBandImage,
        green_band: usize,
        red_band: usize,
        nir_band: usize,
        swir_band: usize,
    ) -> Result<IndexResult> {
        let green = image.band(green_band).unwrap();
        let red = image.band(red_band).unwrap();
        let nir = image.band(nir_band).unwrap();
        let swir = image.band(swir_band).unwrap();

        let values: Vec<f32> = green.iter()
            .zip(red.iter())
            .zip(nir.iter())
            .zip(swir.iter())
            .map(|(((&g, &r), &n), &s)| {
                let denominator = n + s;
                if denominator.abs() < 1e-10 {
                    0.0
                } else {
                    (g + r) / denominator
                }
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "WRI".to_string(),
        })
    }

    /// NDMI - Normalized Difference Moisture Index
    ///
    /// NDMI = (NIR - SWIR) / (NIR + SWIR)
    ///
    /// Useful for vegetation moisture content and water stress
    pub fn ndmi(image: &MultiBandImage, nir_band: usize, swir_band: usize) -> Result<IndexResult> {
        let nir = image.band(nir_band).unwrap();
        let swir = image.band(swir_band).unwrap();

        let values = normalized_difference(nir, swir)?;

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "NDMI".to_string(),
        })
    }

    /// SWI - Surface Water Index
    ///
    /// SWI = (Blue - NIR) / (Blue + NIR)
    pub fn swi(image: &MultiBandImage, blue_band: usize, nir_band: usize) -> Result<IndexResult> {
        let blue = image.band(blue_band).unwrap();
        let nir = image.band(nir_band).unwrap();

        let values = normalized_difference(blue, nir)?;

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "SWI".to_string(),
        })
    }

    /// Calculate water probability mask
    ///
    /// Combines multiple water indices for robust water detection
    pub fn water_mask(
        image: &MultiBandImage,
        green_band: usize,
        nir_band: usize,
        swir_band: usize,
        threshold: f32,
    ) -> Result<IndexResult> {
        let ndwi = Self::ndwi(image, green_band, nir_band)?;
        let mndwi = Self::mndwi(image, green_band, swir_band)?;

        // Combine indices - water if both are positive
        let values: Vec<f32> = ndwi.values.iter()
            .zip(mndwi.values.iter())
            .map(|(&n, &m)| {
                if n > threshold && m > threshold {
                    1.0 // Water
                } else {
                    0.0 // Not water
                }
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "WaterMask".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_ndwi_calculation() {
        let metadata = ImageMetadata {
            width: 2,
            height: 2,
            bands: 4,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["Blue".to_string(), "Green".to_string(), "Red".to_string(), "NIR".to_string()],
        };

        let mut image = MultiBandImage::new(metadata, DataType::UInt8);

        // Water has high green, low NIR
        image.bands[1] = vec![100.0, 110.0, 120.0, 130.0]; // Green
        image.bands[3] = vec![30.0, 35.0, 40.0, 45.0]; // NIR

        let ndwi = WaterIndices::ndwi(&image, 1, 3).unwrap();

        assert_eq!(ndwi.values.len(), 4);
        // All values should be positive for water
        assert!(ndwi.values.iter().all(|&v| v > 0.0));
    }

    #[test]
    fn test_water_mask() {
        let metadata = ImageMetadata {
            width: 2,
            height: 2,
            bands: 5,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec![
                "Blue".to_string(),
                "Green".to_string(),
                "Red".to_string(),
                "NIR".to_string(),
                "SWIR".to_string(),
            ],
        };

        let mut image = MultiBandImage::new(metadata, DataType::UInt8);

        // Water pixels
        image.bands[1] = vec![100.0, 110.0, 50.0, 60.0]; // Green
        image.bands[3] = vec![30.0, 35.0, 150.0, 160.0]; // NIR (low for water)
        image.bands[4] = vec![20.0, 25.0, 140.0, 150.0]; // SWIR (low for water)

        let mask = WaterIndices::water_mask(&image, 1, 3, 4, 0.0).unwrap();

        assert_eq!(mask.values.len(), 4);
        assert_eq!(mask.values[0], 1.0); // Water
        assert_eq!(mask.values[1], 1.0); // Water
        assert_eq!(mask.values[2], 0.0); // Not water
        assert_eq!(mask.values[3], 0.0); // Not water
    }
}
