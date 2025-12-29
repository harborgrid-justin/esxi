//! Atmospheric correction
//!
//! Remove atmospheric effects from satellite imagery to recover surface reflectance.

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;

/// Atmospheric correction methods
pub struct AtmosphericCorrection;

impl AtmosphericCorrection {
    /// Dark Object Subtraction - Simple atmospheric correction
    ///
    /// DOS1: Assumes dark objects should have 1% reflectance
    pub fn dos1(image: &mut MultiBandImage) -> Result<()> {
        for band in image.bands.iter_mut() {
            // Find 1st percentile as atmospheric contribution
            let mut sorted: Vec<f32> = band.iter()
                .filter(|v| v.is_finite() && **v > 0.0)
                .copied()
                .collect();

            if sorted.is_empty() {
                continue;
            }

            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let percentile_idx = (sorted.len() as f32 * 0.01) as usize;
            let dark_value = sorted[percentile_idx.min(sorted.len() - 1)];

            // Subtract atmospheric contribution
            for pixel in band.iter_mut() {
                *pixel = (*pixel - dark_value).max(0.0);
            }
        }

        Ok(())
    }

    /// DOS2: Improved DOS with cosine correction
    pub fn dos2(
        image: &mut MultiBandImage,
        solar_zenith: f32,
    ) -> Result<()> {
        let cos_zenith = solar_zenith.to_radians().cos();

        for band in image.bands.iter_mut() {
            let mut sorted: Vec<f32> = band.iter()
                .filter(|v| v.is_finite() && **v > 0.0)
                .copied()
                .collect();

            if sorted.is_empty() {
                continue;
            }

            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let percentile_idx = (sorted.len() as f32 * 0.01) as usize;
            let dark_value = sorted[percentile_idx.min(sorted.len() - 1)];

            let atmospheric_correction = dark_value / cos_zenith;

            for pixel in band.iter_mut() {
                *pixel = (*pixel - atmospheric_correction).max(0.0);
            }
        }

        Ok(())
    }

    /// 6S (Second Simulation of Satellite Signal in the Solar Spectrum)
    /// Simplified implementation - full version requires extensive LUT
    pub fn six_s(
        image: &mut MultiBandImage,
        params: &SixSParams,
    ) -> Result<()> {
        // Simplified 6S correction formula:
        // ρ_surface = (ρ_TOA - ρ_atm) / (T↓ * T↑)
        // where:
        // - ρ_surface: surface reflectance
        // - ρ_TOA: top of atmosphere reflectance
        // - ρ_atm: atmospheric path radiance
        // - T↓: downward transmittance
        // - T↑: upward transmittance

        for band_idx in 0..image.bands.len() {
            let path_radiance = params.path_radiance.get(band_idx)
                .ok_or_else(|| ImageryError::InvalidParameter(
                    "Missing path radiance for band".to_string()
                ))?;

            let transmittance_down = params.transmittance_down.get(band_idx)
                .ok_or_else(|| ImageryError::InvalidParameter(
                    "Missing downward transmittance for band".to_string()
                ))?;

            let transmittance_up = params.transmittance_up.get(band_idx)
                .ok_or_else(|| ImageryError::InvalidParameter(
                    "Missing upward transmittance for band".to_string()
                ))?;

            let factor = transmittance_down * transmittance_up;

            for pixel in image.bands[band_idx].iter_mut() {
                *pixel = (*pixel - path_radiance) / factor;
                *pixel = pixel.max(0.0).min(1.0); // Clamp to valid reflectance range
            }
        }

        Ok(())
    }

    /// FLAASH (Fast Line-of-sight Atmospheric Analysis of Spectral Hypercubes)
    /// Simplified implementation
    pub fn flaash(
        image: &mut MultiBandImage,
        params: &FlaashParams,
    ) -> Result<()> {
        // FLAASH uses MODTRAN radiative transfer code
        // Simplified version using empirical parameters

        let cos_view = params.view_zenith.to_radians().cos();
        let cos_sun = params.solar_zenith.to_radians().cos();

        for band_idx in 0..image.bands.len() {
            let wavelength = params.wavelengths.get(band_idx)
                .ok_or_else(|| ImageryError::InvalidParameter(
                    "Missing wavelength for band".to_string()
                ))?;

            // Calculate atmospheric parameters based on wavelength
            let (absorption, scattering) = Self::calculate_atmospheric_params(
                *wavelength,
                params.visibility,
                params.aerosol_model,
            );

            for pixel in image.bands[band_idx].iter_mut() {
                // Simplified atmospheric correction
                let corrected = (*pixel - absorption) / (1.0 + scattering);
                *pixel = corrected.max(0.0).min(1.0);
            }
        }

        Ok(())
    }

    /// Calculate atmospheric parameters
    fn calculate_atmospheric_params(
        wavelength: f32,
        visibility: f32,
        aerosol_model: AerosolModel,
    ) -> (f32, f32) {
        // Simplified Rayleigh scattering (inversely proportional to λ^4)
        let rayleigh = 0.008735 * wavelength.powf(-4.08);

        // Aerosol scattering (depends on visibility and model)
        let aerosol_beta = match aerosol_model {
            AerosolModel::Rural => 3.91 / visibility,
            AerosolModel::Maritime => 3.91 / visibility * 0.8,
            AerosolModel::Urban => 3.91 / visibility * 1.3,
        };

        let aerosol = aerosol_beta * wavelength.powf(-1.3);

        let total_scattering = rayleigh + aerosol;
        let absorption = total_scattering * 0.1; // Simplified

        (absorption, total_scattering)
    }

    /// Apply topographic correction (for mountainous terrain)
    pub fn topographic_correction(
        image: &mut MultiBandImage,
        dem: &[f32],
        solar_zenith: f32,
        solar_azimuth: f32,
    ) -> Result<()> {
        if dem.len() != (image.metadata.width * image.metadata.height) as usize {
            return Err(ImageryError::InvalidDimensions(
                "DEM dimensions must match image".to_string()
            ));
        }

        let width = image.metadata.width as usize;

        // Calculate slope and aspect from DEM
        for band in image.bands.iter_mut() {
            for y in 1..(image.metadata.height as usize - 1) {
                for x in 1..(width - 1) {
                    let idx = y * width + x;

                    // Calculate slope using neighboring cells
                    let dz_dx = (dem[idx + 1] - dem[idx - 1]) / 2.0;
                    let dz_dy = (dem[idx + width] - dem[idx - width]) / 2.0;

                    let slope = (dz_dx * dz_dx + dz_dy * dz_dy).sqrt().atan();
                    let aspect = dz_dy.atan2(dz_dx);

                    // Calculate illumination angle
                    let cos_i = solar_zenith.to_radians().cos() * slope.cos() +
                                solar_zenith.to_radians().sin() * slope.sin() *
                                (solar_azimuth.to_radians() - aspect).cos();

                    // Apply correction (C-correction method)
                    if cos_i > 0.0 {
                        band[idx] = band[idx] * solar_zenith.to_radians().cos() / cos_i;
                    }
                }
            }
        }

        Ok(())
    }
}

/// 6S atmospheric correction parameters
#[derive(Debug, Clone)]
pub struct SixSParams {
    /// Atmospheric path radiance per band
    pub path_radiance: Vec<f32>,
    /// Downward transmittance per band
    pub transmittance_down: Vec<f32>,
    /// Upward transmittance per band
    pub transmittance_up: Vec<f32>,
}

/// FLAASH atmospheric correction parameters
#[derive(Debug, Clone)]
pub struct FlaashParams {
    /// Solar zenith angle (degrees)
    pub solar_zenith: f32,
    /// View zenith angle (degrees)
    pub view_zenith: f32,
    /// Visibility (km)
    pub visibility: f32,
    /// Aerosol model
    pub aerosol_model: AerosolModel,
    /// Band center wavelengths (micrometers)
    pub wavelengths: Vec<f32>,
}

/// Aerosol models
#[derive(Debug, Clone, Copy)]
pub enum AerosolModel {
    /// Rural aerosol
    Rural,
    /// Maritime aerosol
    Maritime,
    /// Urban aerosol
    Urban,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_dos1_correction() {
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
        for i in 0..100 {
            image.bands[0][i] = (i as f32 + 10.0);
        }

        AtmosphericCorrection::dos1(&mut image).unwrap();

        // First percentile should be near zero after correction
        assert!(image.bands[0][0] < 5.0);
    }
}
