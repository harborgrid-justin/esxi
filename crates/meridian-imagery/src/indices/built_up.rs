//! Built-up area indices
//!
//! Spectral indices for urban and built-up area detection.

use crate::error::Result;
use crate::MultiBandImage;
use super::{IndexResult, normalized_difference};

/// Built-up area indices calculator
pub struct BuiltUpIndices;

impl BuiltUpIndices {
    /// NDBI - Normalized Difference Built-up Index
    ///
    /// NDBI = (SWIR - NIR) / (SWIR + NIR)
    ///
    /// Positive values indicate built-up areas
    pub fn ndbi(image: &MultiBandImage, swir_band: usize, nir_band: usize) -> Result<IndexResult> {
        let swir = image.band(swir_band)
            .ok_or_else(|| crate::error::ImageryError::InvalidBand {
                band: swir_band,
                total: image.bands.len(),
            })?;

        let nir = image.band(nir_band)
            .ok_or_else(|| crate::error::ImageryError::InvalidBand {
                band: nir_band,
                total: image.bands.len(),
            })?;

        let values = normalized_difference(swir, nir)?;

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "NDBI".to_string(),
        })
    }

    /// UI - Urban Index
    ///
    /// UI = (SWIR2 - NIR) / (SWIR2 + NIR)
    ///
    /// Similar to NDBI but uses SWIR2 band
    pub fn ui(image: &MultiBandImage, swir2_band: usize, nir_band: usize) -> Result<IndexResult> {
        Self::ndbi(image, swir2_band, nir_band).map(|mut result| {
            result.name = "UI".to_string();
            result
        })
    }

    /// IBI - Index-based Built-up Index
    ///
    /// IBI = (NDBI - (SAVI + MNDWI)/2) / (NDBI + (SAVI + MNDWI)/2)
    ///
    /// Combines built-up, vegetation, and water indices
    pub fn ibi(
        image: &MultiBandImage,
        green_band: usize,
        red_band: usize,
        nir_band: usize,
        swir_band: usize,
    ) -> Result<IndexResult> {
        // Calculate NDBI
        let ndbi = Self::ndbi(image, swir_band, nir_band)?;

        // Calculate SAVI (simplified with L=0.5)
        let nir = image.band(nir_band).unwrap();
        let red = image.band(red_band).unwrap();
        let savi: Vec<f32> = nir.iter()
            .zip(red.iter())
            .map(|(&n, &r)| {
                let denom = n + r + 0.5;
                if denom.abs() < 1e-10 {
                    0.0
                } else {
                    ((n - r) / denom) * 1.5
                }
            })
            .collect();

        // Calculate MNDWI
        let green = image.band(green_band).unwrap();
        let swir = image.band(swir_band).unwrap();
        let mndwi = normalized_difference(green, swir)?;

        // Calculate IBI
        let values: Vec<f32> = ndbi.values.iter()
            .zip(savi.iter())
            .zip(mndwi.iter())
            .map(|((&nb, &sv), &mw)| {
                let term = (sv + mw) / 2.0;
                let denom = nb + term;
                if denom.abs() < 1e-10 {
                    0.0
                } else {
                    (nb - term) / denom
                }
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "IBI".to_string(),
        })
    }

    /// BAEI - Built-up Area Extraction Index
    ///
    /// BAEI = (Red + 0.3) / (Green + SWIR)
    pub fn baei(
        image: &MultiBandImage,
        red_band: usize,
        green_band: usize,
        swir_band: usize,
    ) -> Result<IndexResult> {
        let red = image.band(red_band).unwrap();
        let green = image.band(green_band).unwrap();
        let swir = image.band(swir_band).unwrap();

        let values: Vec<f32> = red.iter()
            .zip(green.iter())
            .zip(swir.iter())
            .map(|((&r, &g), &s)| {
                let denom = g + s;
                if denom.abs() < 1e-10 {
                    0.0
                } else {
                    (r + 0.3) / denom
                }
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "BAEI".to_string(),
        })
    }

    /// NDBaI - Normalized Difference Bareness Index
    ///
    /// NDBaI = (SWIR - TIR) / (SWIR + TIR)
    ///
    /// Uses thermal infrared for bare soil/urban detection
    pub fn ndbai(
        image: &MultiBandImage,
        swir_band: usize,
        tir_band: usize,
    ) -> Result<IndexResult> {
        let swir = image.band(swir_band).unwrap();
        let tir = image.band(tir_band).unwrap();

        let values = normalized_difference(swir, tir)?;

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "NDBaI".to_string(),
        })
    }

    /// EBBI - Enhanced Built-Up and Bareness Index
    ///
    /// EBBI = (SWIR - NIR) / (10 * sqrt(SWIR + TIR))
    pub fn ebbi(
        image: &MultiBandImage,
        swir_band: usize,
        nir_band: usize,
        tir_band: usize,
    ) -> Result<IndexResult> {
        let swir = image.band(swir_band).unwrap();
        let nir = image.band(nir_band).unwrap();
        let tir = image.band(tir_band).unwrap();

        let values: Vec<f32> = swir.iter()
            .zip(nir.iter())
            .zip(tir.iter())
            .map(|((&s, &n), &t)| {
                let denom = 10.0 * (s + t).sqrt();
                if denom.abs() < 1e-10 {
                    0.0
                } else {
                    (s - n) / denom
                }
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "EBBI".to_string(),
        })
    }

    /// BUI - Built-up Index
    ///
    /// BUI = (Red - NIR) / (Red + NIR) - (SWIR - NIR) / (SWIR + NIR)
    pub fn bui(
        image: &MultiBandImage,
        red_band: usize,
        nir_band: usize,
        swir_band: usize,
    ) -> Result<IndexResult> {
        let red = image.band(red_band).unwrap();
        let nir = image.band(nir_band).unwrap();
        let swir = image.band(swir_band).unwrap();

        let values: Vec<f32> = red.iter()
            .zip(nir.iter())
            .zip(swir.iter())
            .map(|((&r, &n), &s)| {
                let term1 = {
                    let denom = r + n;
                    if denom.abs() < 1e-10 {
                        0.0
                    } else {
                        (r - n) / denom
                    }
                };

                let term2 = {
                    let denom = s + n;
                    if denom.abs() < 1e-10 {
                        0.0
                    } else {
                        (s - n) / denom
                    }
                };

                term1 - term2
            })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "BUI".to_string(),
        })
    }

    /// Urban area mask
    ///
    /// Binary mask of urban areas based on NDBI threshold
    pub fn urban_mask(
        image: &MultiBandImage,
        swir_band: usize,
        nir_band: usize,
        threshold: f32,
    ) -> Result<IndexResult> {
        let ndbi = Self::ndbi(image, swir_band, nir_band)?;

        let values: Vec<f32> = ndbi.values.iter()
            .map(|&v| if v > threshold { 1.0 } else { 0.0 })
            .collect();

        Ok(IndexResult {
            values,
            width: image.metadata.width,
            height: image.metadata.height,
            name: "UrbanMask".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_ndbi_calculation() {
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

        // Built-up has high SWIR, moderate NIR
        image.bands[3] = vec![50.0, 60.0, 70.0, 80.0]; // NIR
        image.bands[4] = vec![150.0, 160.0, 170.0, 180.0]; // SWIR

        let ndbi = BuiltUpIndices::ndbi(&image, 4, 3).unwrap();

        assert_eq!(ndbi.values.len(), 4);
        // All values should be positive for built-up
        assert!(ndbi.values.iter().all(|&v| v > 0.0));
    }

    #[test]
    fn test_urban_mask() {
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

        // Mix of urban and non-urban pixels
        image.bands[3] = vec![50.0, 60.0, 150.0, 160.0]; // NIR
        image.bands[4] = vec![150.0, 160.0, 100.0, 110.0]; // SWIR

        let mask = BuiltUpIndices::urban_mask(&image, 4, 3, 0.2).unwrap();

        assert_eq!(mask.values.len(), 4);
        assert!(mask.values[0] > 0.0); // Urban
        assert!(mask.values[1] > 0.0); // Urban
    }
}
