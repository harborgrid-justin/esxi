//! Orthorectification
//!
//! Geometric correction to remove terrain distortion and sensor perspective.

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;
use crate::processing::ResamplingMethod;

/// Orthorectification operations
pub struct Orthorectification;

impl Orthorectification {
    /// Perform orthorectification using RPC (Rational Polynomial Coefficients)
    pub fn rpc(
        image: &MultiBandImage,
        rpc_params: &RpcParameters,
        dem: &[f32],
        output_width: u32,
        output_height: u32,
    ) -> Result<MultiBandImage> {
        let mut output_meta = image.metadata.clone();
        output_meta.width = output_width;
        output_meta.height = output_height;

        let mut output = MultiBandImage::new(output_meta, image.data_type);

        // For each output pixel, calculate corresponding input pixel using RPC
        for y in 0..output_height {
            for x in 0..output_width {
                // Convert pixel to geographic coordinates
                let (lon, lat) = Self::pixel_to_geo(x, y, &image.metadata)?;

                // Get elevation from DEM
                let elevation = Self::sample_dem(dem, lon, lat, &image.metadata)?;

                // Apply RPC transformation
                let (src_x, src_y) = rpc_params.forward(lon, lat, elevation);

                // Resample from source image
                if src_x >= 0.0 && src_x < image.metadata.width as f32 &&
                   src_y >= 0.0 && src_y < image.metadata.height as f32 {
                    for band_idx in 0..image.bands.len() {
                        let value = Self::bilinear_interpolate(
                            &image.bands[band_idx],
                            image.metadata.width,
                            src_x,
                            src_y,
                        );

                        let out_idx = (y * output_width + x) as usize;
                        output.bands[band_idx][out_idx] = value;
                    }
                }
            }
        }

        Ok(output)
    }

    /// Orthorectification using rigorous sensor model
    pub fn rigorous_model(
        image: &MultiBandImage,
        sensor_params: &SensorParameters,
        dem: &[f32],
        output_width: u32,
        output_height: u32,
    ) -> Result<MultiBandImage> {
        // Use collinearity equations for frame cameras
        // or push-broom model for line scanners

        let mut output_meta = image.metadata.clone();
        output_meta.width = output_width;
        output_meta.height = output_height;

        let output = MultiBandImage::new(output_meta, image.data_type);

        // Placeholder: would implement rigorous sensor model
        Ok(output)
    }

    /// Simple terrain correction without full orthorectification
    pub fn terrain_correction(
        image: &MultiBandImage,
        dem: &[f32],
        sensor_azimuth: f32,
        sensor_elevation: f32,
    ) -> Result<MultiBandImage> {
        let mut output = image.clone();

        // Apply relief displacement correction
        // Displacement = (h * tan(θ)) / pixel_size
        // where h is elevation difference from reference

        let width = image.metadata.width as usize;

        for y in 0..image.metadata.height as usize {
            for x in 0..width {
                let idx = y * width + x;
                let elevation = dem[idx];

                // Calculate displacement
                let displacement_x = elevation * sensor_azimuth.to_radians().tan();
                let displacement_y = elevation * sensor_elevation.to_radians().tan();

                // Sample from displaced location
                let src_x = x as f32 + displacement_x;
                let src_y = y as f32 + displacement_y;

                if src_x >= 0.0 && src_x < image.metadata.width as f32 &&
                   src_y >= 0.0 && src_y < image.metadata.height as f32 {
                    for band_idx in 0..image.bands.len() {
                        let value = Self::bilinear_interpolate(
                            &image.bands[band_idx],
                            image.metadata.width,
                            src_x,
                            src_y,
                        );
                        output.bands[band_idx][idx] = value;
                    }
                }
            }
        }

        Ok(output)
    }

    /// Bilinear interpolation
    fn bilinear_interpolate(data: &[f32], width: u32, x: f32, y: f32) -> f32 {
        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let x1 = (x0 + 1).min(width - 1);
        let y1 = (y0 + 1).min((data.len() / width as usize - 1) as u32);

        let dx = x - x0 as f32;
        let dy = y - y0 as f32;

        let idx00 = (y0 * width + x0) as usize;
        let idx10 = (y0 * width + x1) as usize;
        let idx01 = (y1 * width + x0) as usize;
        let idx11 = (y1 * width + x1) as usize;

        let v00 = data.get(idx00).copied().unwrap_or(0.0);
        let v10 = data.get(idx10).copied().unwrap_or(0.0);
        let v01 = data.get(idx01).copied().unwrap_or(0.0);
        let v11 = data.get(idx11).copied().unwrap_or(0.0);

        let v0 = v00 * (1.0 - dx) + v10 * dx;
        let v1 = v01 * (1.0 - dx) + v11 * dx;

        v0 * (1.0 - dy) + v1 * dy
    }

    /// Convert pixel coordinates to geographic coordinates
    fn pixel_to_geo(x: u32, y: u32, metadata: &crate::ImageMetadata) -> Result<(f64, f64)> {
        if let Some(geo_transform) = &metadata.geo_transform {
            let lon = geo_transform[0] + x as f64 * geo_transform[1] + y as f64 * geo_transform[2];
            let lat = geo_transform[3] + x as f64 * geo_transform[4] + y as f64 * geo_transform[5];
            Ok((lon, lat))
        } else {
            Err(ImageryError::Metadata("No geotransform available".to_string()))
        }
    }

    /// Sample DEM at geographic coordinates
    fn sample_dem(dem: &[f32], lon: f64, lat: f64, metadata: &crate::ImageMetadata) -> Result<f32> {
        // Convert geographic coordinates to DEM pixel coordinates
        // and sample with interpolation

        if let Some(geo_transform) = &metadata.geo_transform {
            // Inverse geotransform
            let x = (lon - geo_transform[0]) / geo_transform[1];
            let y = (lat - geo_transform[3]) / geo_transform[5];

            if x >= 0.0 && x < metadata.width as f64 && y >= 0.0 && y < metadata.height as f64 {
                let idx = (y as u32 * metadata.width + x as u32) as usize;
                return Ok(dem.get(idx).copied().unwrap_or(0.0));
            }
        }

        Ok(0.0) // Default elevation
    }
}

/// RPC (Rational Polynomial Coefficients) parameters
#[derive(Debug, Clone)]
pub struct RpcParameters {
    /// Line numerator coefficients (20 coefficients)
    pub line_num_coeff: [f64; 20],
    /// Line denominator coefficients (20 coefficients)
    pub line_den_coeff: [f64; 20],
    /// Sample numerator coefficients (20 coefficients)
    pub samp_num_coeff: [f64; 20],
    /// Sample denominator coefficients (20 coefficients)
    pub samp_den_coeff: [f64; 20],
    /// Latitude offset
    pub lat_off: f64,
    /// Latitude scale
    pub lat_scale: f64,
    /// Longitude offset
    pub long_off: f64,
    /// Longitude scale
    pub long_scale: f64,
    /// Height offset
    pub height_off: f64,
    /// Height scale
    pub height_scale: f64,
    /// Line offset
    pub line_off: f64,
    /// Line scale
    pub line_scale: f64,
    /// Sample offset
    pub samp_off: f64,
    /// Sample scale
    pub samp_scale: f64,
}

impl RpcParameters {
    /// Forward RPC transformation (ground to image)
    pub fn forward(&self, lon: f64, lat: f64, height: f64) -> (f32, f32) {
        // Normalize coordinates
        let p = (lon - self.long_off) / self.long_scale;
        let l = (lat - self.lat_off) / self.lat_scale;
        let h = (height - self.height_off) / self.height_scale;

        // Calculate polynomial terms
        let line_num = Self::evaluate_polynomial(&self.line_num_coeff, p, l, h);
        let line_den = Self::evaluate_polynomial(&self.line_den_coeff, p, l, h);
        let samp_num = Self::evaluate_polynomial(&self.samp_num_coeff, p, l, h);
        let samp_den = Self::evaluate_polynomial(&self.samp_den_coeff, p, l, h);

        // Denormalize
        let line = (line_num / line_den) * self.line_scale + self.line_off;
        let samp = (samp_num / samp_den) * self.samp_scale + self.samp_off;

        (samp as f32, line as f32)
    }

    /// Evaluate RPC polynomial
    fn evaluate_polynomial(coeffs: &[f64; 20], p: f64, l: f64, h: f64) -> f64 {
        coeffs[0] +
        coeffs[1] * l +
        coeffs[2] * p +
        coeffs[3] * h +
        coeffs[4] * l * p +
        coeffs[5] * l * h +
        coeffs[6] * p * h +
        coeffs[7] * l * l +
        coeffs[8] * p * p +
        coeffs[9] * h * h +
        coeffs[10] * p * l * h +
        coeffs[11] * l * l * l +
        coeffs[12] * l * p * p +
        coeffs[13] * l * h * h +
        coeffs[14] * l * l * p +
        coeffs[15] * p * p * p +
        coeffs[16] * p * h * h +
        coeffs[17] * l * l * h +
        coeffs[18] * p * p * h +
        coeffs[19] * h * h * h
    }
}

/// Sensor model parameters
#[derive(Debug, Clone)]
pub struct SensorParameters {
    /// Sensor position (X, Y, Z in map coordinates)
    pub position: (f64, f64, f64),
    /// Sensor orientation (omega, phi, kappa in radians)
    pub orientation: (f64, f64, f64),
    /// Focal length (mm)
    pub focal_length: f64,
    /// Principal point (x, y in mm)
    pub principal_point: (f64, f64),
    /// Pixel size (mm)
    pub pixel_size: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_creation() {
        let rpc = RpcParameters {
            line_num_coeff: [0.0; 20],
            line_den_coeff: [1.0; 20],
            samp_num_coeff: [0.0; 20],
            samp_den_coeff: [1.0; 20],
            lat_off: 0.0,
            lat_scale: 1.0,
            long_off: 0.0,
            long_scale: 1.0,
            height_off: 0.0,
            height_scale: 1.0,
            line_off: 0.0,
            line_scale: 1.0,
            samp_off: 0.0,
            samp_scale: 1.0,
        };

        let (x, y) = rpc.forward(0.0, 0.0, 0.0);
        assert!(x.is_finite() && y.is_finite());
    }
}
