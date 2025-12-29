//! Sky rendering with atmospheric scattering

use crate::{Camera, Error, Result};
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
use wgpu::{Device, Queue, RenderPass};

/// Sky rendering model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkyModel {
    /// Simple gradient
    Gradient,
    /// Preetham atmospheric scattering
    Preetham,
    /// Hosek-Wilkie atmospheric scattering
    HosekWilkie,
    /// Procedural clouds
    Procedural,
}

/// Sky parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyParameters {
    /// Sky model to use
    #[serde(skip)]
    pub model: SkyModel,

    /// Turbidity (atmospheric haziness)
    pub turbidity: f32,

    /// Ground albedo
    pub ground_albedo: Vec3,

    /// Sun intensity
    pub sun_intensity: f32,

    /// Rayleigh scattering coefficient
    pub rayleigh_coefficient: f32,

    /// Mie scattering coefficient
    pub mie_coefficient: f32,

    /// Mie directional factor (anisotropy)
    pub mie_directional_g: f32,
}

impl Default for SkyParameters {
    fn default() -> Self {
        Self {
            model: SkyModel::Preetham,
            turbidity: 2.0,
            ground_albedo: Vec3::new(0.3, 0.3, 0.3),
            sun_intensity: 1.0,
            rayleigh_coefficient: 1.0,
            mie_coefficient: 1.0,
            mie_directional_g: 0.76,
        }
    }
}

/// Sky renderer
pub struct SkyRenderer {
    /// Sky parameters
    params: SkyParameters,

    /// Current sun direction
    sun_direction: Vec3,

    /// Zenith color
    zenith_color: Vec3,

    /// Horizon color
    horizon_color: Vec3,

    /// Ground color
    ground_color: Vec3,
}

impl SkyRenderer {
    /// Create a new sky renderer
    pub fn new(params: SkyParameters) -> Self {
        let mut renderer = Self {
            params,
            sun_direction: Vec3::new(0.0, -0.707, 0.707),
            zenith_color: Vec3::new(0.2, 0.5, 1.0),
            horizon_color: Vec3::new(0.7, 0.8, 1.0),
            ground_color: Vec3::new(0.3, 0.3, 0.3),
        };

        renderer.calculate_sky_colors();
        renderer
    }

    /// Update sky based on sun position
    pub fn update(&mut self, time: f32, sun_direction: Vec3) {
        self.sun_direction = sun_direction.normalize();
        self.calculate_sky_colors();
    }

    /// Calculate sky colors based on sun position
    fn calculate_sky_colors(&mut self) {
        let sun_elevation = (-self.sun_direction.y).asin();

        match self.params.model {
            SkyModel::Gradient => {
                self.calculate_gradient_colors(sun_elevation);
            }
            SkyModel::Preetham => {
                self.calculate_preetham_colors(sun_elevation);
            }
            SkyModel::HosekWilkie => {
                self.calculate_hosek_wilkie_colors(sun_elevation);
            }
            SkyModel::Procedural => {
                self.calculate_procedural_colors(sun_elevation);
            }
        }
    }

    /// Simple gradient sky
    fn calculate_gradient_colors(&mut self, sun_elevation: f32) {
        let day_factor = (sun_elevation / (std::f32::consts::FRAC_PI_2)).clamp(0.0, 1.0);

        // Day colors
        let day_zenith = Vec3::new(0.2, 0.5, 1.0);
        let day_horizon = Vec3::new(0.7, 0.8, 1.0);

        // Sunset colors
        let sunset_zenith = Vec3::new(0.3, 0.2, 0.5);
        let sunset_horizon = Vec3::new(1.0, 0.5, 0.3);

        // Night colors
        let night_zenith = Vec3::new(0.0, 0.0, 0.05);
        let night_horizon = Vec3::new(0.0, 0.0, 0.1);

        if day_factor > 0.5 {
            // Day
            self.zenith_color = day_zenith;
            self.horizon_color = day_horizon;
        } else if day_factor > 0.1 {
            // Sunset/sunrise
            let t = (day_factor - 0.1) / 0.4;
            self.zenith_color = sunset_zenith.lerp(day_zenith, t);
            self.horizon_color = sunset_horizon.lerp(day_horizon, t);
        } else {
            // Night
            let t = day_factor / 0.1;
            self.zenith_color = night_zenith.lerp(sunset_zenith, t);
            self.horizon_color = night_horizon.lerp(sunset_horizon, t);
        }

        self.ground_color = self.params.ground_albedo;
    }

    /// Preetham atmospheric scattering model
    fn calculate_preetham_colors(&mut self, sun_elevation: f32) {
        // Simplified Preetham model
        // Full implementation would calculate perez function
        self.calculate_gradient_colors(sun_elevation);
    }

    /// Hosek-Wilkie atmospheric scattering model
    fn calculate_hosek_wilkie_colors(&mut self, sun_elevation: f32) {
        // Simplified Hosek-Wilkie model
        self.calculate_gradient_colors(sun_elevation);
    }

    /// Procedural sky with clouds
    fn calculate_procedural_colors(&mut self, sun_elevation: f32) {
        self.calculate_gradient_colors(sun_elevation);
    }

    /// Sample sky color in a direction
    pub fn sample_sky(&self, direction: Vec3) -> Vec3 {
        let up_factor = direction.y.max(0.0);

        if direction.y >= 0.0 {
            // Sky hemisphere
            self.horizon_color.lerp(self.zenith_color, up_factor)
        } else {
            // Ground hemisphere
            self.ground_color
        }
    }

    /// Render the sky
    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera: &Camera) -> Result<()> {
        // Sky rendering would happen here
        // Typically renders a large sphere or uses a fragment shader
        Ok(())
    }

    /// Get parameters
    pub fn params(&self) -> &SkyParameters {
        &self.params
    }

    /// Set parameters
    pub fn set_params(&mut self, params: SkyParameters) {
        self.params = params;
        self.calculate_sky_colors();
    }

    /// Get zenith color
    pub fn zenith_color(&self) -> Vec3 {
        self.zenith_color
    }

    /// Get horizon color
    pub fn horizon_color(&self) -> Vec3 {
        self.horizon_color
    }

    /// Get ground color
    pub fn ground_color(&self) -> Vec3 {
        self.ground_color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sky_renderer() {
        let renderer = SkyRenderer::new(SkyParameters::default());
        assert!(renderer.zenith_color().length() > 0.0);
    }

    #[test]
    fn test_sky_sampling() {
        let renderer = SkyRenderer::new(SkyParameters::default());

        let zenith_sample = renderer.sample_sky(Vec3::Y);
        let horizon_sample = renderer.sample_sky(Vec3::X);
        let ground_sample = renderer.sample_sky(Vec3::NEG_Y);

        // Zenith should be different from horizon
        assert_ne!(zenith_sample, horizon_sample);
        // Ground should be ground color
        assert_eq!(ground_sample, renderer.ground_color());
    }
}
