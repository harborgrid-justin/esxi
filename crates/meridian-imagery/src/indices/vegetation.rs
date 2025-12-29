//! Vegetation indices
//!
//! Spectral indices for vegetation analysis and monitoring.

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;
use super::{IndexResult, normalized_difference};

/// Vegetation indices calculator
pub struct VegetationIndices;

impl VegetationIndices {
    /// NDVI - Normalized Difference Vegetation Index
    ///
    /// NDVI = (NIR - Red) / (NIR + Red)
    ///
    /// Range: -1 to 1 (typically 0.2-0.8 for vegetation)
    pub fn ndvi(image: &MultiBandImage, nir_band: usize, red_band: usize) -> Result<IndexResult> {
        Self::validate_bands(image, nir_band, red_band)?;

        let nir = image.band(nir_band).unwrap();
        let red = image.band(red_band).unwrap();

        let values = normalized_difference(nir, red)?;

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "NDVI".to_string(),
        })
    }

    /// EVI - Enhanced Vegetation Index
    ///
    /// EVI = G * (NIR - Red) / (NIR + C1*Red - C2*Blue + L)
    /// where G=2.5, C1=6, C2=7.5, L=1
    pub fn evi(
        image: &MultiBandImage,
        nir_band: usize,
        red_band: usize,
        blue_band: usize,
    ) -> Result<IndexResult> {
        Self::validate_bands(image, nir_band, red_band)?;
        Self::validate_bands(image, nir_band, blue_band)?;

        let nir = image.band(nir_band).unwrap();
        let red = image.band(red_band).unwrap();
        let blue = image.band(blue_band).unwrap();

        const G: f32 = 2.5;
        const C1: f32 = 6.0;
        const C2: f32 = 7.5;
        const L: f32 = 1.0;

        let values: Vec<f32> = nir.iter()
            .zip(red.iter())
            .zip(blue.iter())
            .map(|((&n, &r), &b)| {
                let denominator = n + C1 * r - C2 * b + L;
                if denominator.abs() < 1e-10 {
                    0.0
                } else {
                    G * (n - r) / denominator
                }
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "EVI".to_string(),
        })
    }

    /// SAVI - Soil Adjusted Vegetation Index
    ///
    /// SAVI = ((NIR - Red) / (NIR + Red + L)) * (1 + L)
    /// where L is soil brightness correction factor (typically 0.5)
    pub fn savi(
        image: &MultiBandImage,
        nir_band: usize,
        red_band: usize,
        l_factor: f32,
    ) -> Result<IndexResult> {
        Self::validate_bands(image, nir_band, red_band)?;

        let nir = image.band(nir_band).unwrap();
        let red = image.band(red_band).unwrap();

        let values: Vec<f32> = nir.iter()
            .zip(red.iter())
            .map(|(&n, &r)| {
                let denominator = n + r + l_factor;
                if denominator.abs() < 1e-10 {
                    0.0
                } else {
                    ((n - r) / denominator) * (1.0 + l_factor)
                }
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "SAVI".to_string(),
        })
    }

    /// NDRE - Normalized Difference Red Edge
    ///
    /// NDRE = (NIR - RedEdge) / (NIR + RedEdge)
    ///
    /// Useful for crops and precision agriculture
    pub fn ndre(
        image: &MultiBandImage,
        nir_band: usize,
        red_edge_band: usize,
    ) -> Result<IndexResult> {
        Self::validate_bands(image, nir_band, red_edge_band)?;

        let nir = image.band(nir_band).unwrap();
        let red_edge = image.band(red_edge_band).unwrap();

        let values = normalized_difference(nir, red_edge)?;

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "NDRE".to_string(),
        })
    }

    /// GNDVI - Green Normalized Difference Vegetation Index
    ///
    /// GNDVI = (NIR - Green) / (NIR + Green)
    ///
    /// More sensitive to chlorophyll content
    pub fn gndvi(
        image: &MultiBandImage,
        nir_band: usize,
        green_band: usize,
    ) -> Result<IndexResult> {
        Self::validate_bands(image, nir_band, green_band)?;

        let nir = image.band(nir_band).unwrap();
        let green = image.band(green_band).unwrap();

        let values = normalized_difference(nir, green)?;

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "GNDVI".to_string(),
        })
    }

    /// ARVI - Atmospherically Resistant Vegetation Index
    ///
    /// ARVI = (NIR - (2*Red - Blue)) / (NIR + (2*Red - Blue))
    ///
    /// Minimizes atmospheric effects
    pub fn arvi(
        image: &MultiBandImage,
        nir_band: usize,
        red_band: usize,
        blue_band: usize,
    ) -> Result<IndexResult> {
        Self::validate_bands(image, nir_band, red_band)?;
        Self::validate_bands(image, nir_band, blue_band)?;

        let nir = image.band(nir_band).unwrap();
        let red = image.band(red_band).unwrap();
        let blue = image.band(blue_band).unwrap();

        let values: Vec<f32> = nir.iter()
            .zip(red.iter())
            .zip(blue.iter())
            .map(|((&n, &r), &b)| {
                let rb = 2.0 * r - b;
                let denominator = n + rb;
                if denominator.abs() < 1e-10 {
                    0.0
                } else {
                    (n - rb) / denominator
                }
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "ARVI".to_string(),
        })
    }

    /// MSAVI - Modified Soil Adjusted Vegetation Index
    ///
    /// MSAVI = (2*NIR + 1 - sqrt((2*NIR + 1)^2 - 8*(NIR - Red))) / 2
    pub fn msavi(image: &MultiBandImage, nir_band: usize, red_band: usize) -> Result<IndexResult> {
        Self::validate_bands(image, nir_band, red_band)?;

        let nir = image.band(nir_band).unwrap();
        let red = image.band(red_band).unwrap();

        let values: Vec<f32> = nir.iter()
            .zip(red.iter())
            .map(|(&n, &r)| {
                let a = 2.0 * n + 1.0;
                let b = a * a - 8.0 * (n - r);
                if b < 0.0 {
                    0.0
                } else {
                    (a - b.sqrt()) / 2.0
                }
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "MSAVI".to_string(),
        })
    }

    /// LAI - Leaf Area Index (empirical from NDVI)
    ///
    /// LAI ≈ 3.618 * EVI - 0.118
    pub fn lai_from_evi(
        image: &MultiBandImage,
        nir_band: usize,
        red_band: usize,
        blue_band: usize,
    ) -> Result<IndexResult> {
        let evi = Self::evi(image, nir_band, red_band, blue_band)?;

        let values: Vec<f32> = evi.values.iter()
            .map(|&e| (3.618 * e - 0.118).max(0.0))
            .collect();

        Ok(IndexResult {
            values,
            width: evi.width,
            height: evi.height,
            name: "LAI".to_string(),
        })
    }

    /// Validate band indices
    fn validate_bands(image: &MultiBandImage, band1: usize, band2: usize) -> Result<()> {
        if band1 >= image.bands.len() {
            return Err(ImageryError::InvalidBand {
                band: band1,
                total: image.bands.len(),
            });
        }
        if band2 >= image.bands.len() {
            return Err(ImageryError::InvalidBand {
                band: band2,
                total: image.bands.len(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_ndvi_calculation() {
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

        // Set some test values
        image.bands[2] = vec![50.0, 60.0, 70.0, 80.0]; // Red
        image.bands[3] = vec![150.0, 160.0, 170.0, 180.0]; // NIR

        let ndvi = VegetationIndices::ndvi(&image, 3, 2).unwrap();

        assert_eq!(ndvi.values.len(), 4);
        assert!(ndvi.values[0] > 0.0 && ndvi.values[0] < 1.0);
    }

    #[test]
    fn test_evi_calculation() {
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

        image.bands[0] = vec![30.0, 35.0, 40.0, 45.0]; // Blue
        image.bands[2] = vec![50.0, 60.0, 70.0, 80.0]; // Red
        image.bands[3] = vec![150.0, 160.0, 170.0, 180.0]; // NIR

        let evi = VegetationIndices::evi(&image, 3, 2, 0).unwrap();

        assert_eq!(evi.values.len(), 4);
    }
}
