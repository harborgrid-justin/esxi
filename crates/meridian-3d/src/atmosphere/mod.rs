//! Atmospheric effects for realistic outdoor rendering
//!
//! Provides:
//! - Physically-based sky rendering (Rayleigh/Mie scattering)
//! - Fog and haze effects
//! - Weather visualization (rain, snow, clouds)
//! - Time-of-day atmosphere changes

pub mod sky;
pub mod fog;
pub mod weather;

pub use sky::{SkyRenderer, SkyParameters, SkyModel};
pub use fog::{FogEffect, FogParameters, FogType};
pub use weather::{WeatherSystem, WeatherType, WeatherParameters};

use crate::{Camera, Error, Result};
use glam::Vec3;
use wgpu::{Device, Queue, RenderPass};

/// Complete atmosphere system
pub struct AtmosphereSystem {
    /// Sky renderer
    sky: SkyRenderer,

    /// Fog effect
    fog: Option<FogEffect>,

    /// Weather system
    weather: Option<WeatherSystem>,
}

impl AtmosphereSystem {
    /// Create a new atmosphere system
    pub fn new() -> Self {
        Self {
            sky: SkyRenderer::new(SkyParameters::default()),
            fog: None,
            weather: None,
        }
    }

    /// Create atmosphere with all features
    pub fn full(sky_params: SkyParameters) -> Self {
        Self {
            sky: SkyRenderer::new(sky_params),
            fog: Some(FogEffect::new(FogParameters::default())),
            weather: Some(WeatherSystem::new()),
        }
    }

    /// Get the sky renderer
    pub fn sky(&self) -> &SkyRenderer {
        &self.sky
    }

    /// Get mutable sky renderer
    pub fn sky_mut(&mut self) -> &mut SkyRenderer {
        &mut self.sky
    }

    /// Enable fog
    pub fn enable_fog(&mut self, params: FogParameters) {
        self.fog = Some(FogEffect::new(params));
    }

    /// Disable fog
    pub fn disable_fog(&mut self) {
        self.fog = None;
    }

    /// Get fog effect
    pub fn fog(&self) -> Option<&FogEffect> {
        self.fog.as_ref()
    }

    /// Get mutable fog effect
    pub fn fog_mut(&mut self) -> Option<&mut FogEffect> {
        self.fog.as_mut()
    }

    /// Enable weather
    pub fn enable_weather(&mut self) {
        self.weather = Some(WeatherSystem::new());
    }

    /// Disable weather
    pub fn disable_weather(&mut self) {
        self.weather = None;
    }

    /// Get weather system
    pub fn weather(&self) -> Option<&WeatherSystem> {
        self.weather.as_ref()
    }

    /// Get mutable weather system
    pub fn weather_mut(&mut self) -> Option<&mut WeatherSystem> {
        self.weather.as_mut()
    }

    /// Update atmosphere based on time and sun position
    pub fn update(&mut self, time: f32, sun_direction: Vec3) {
        self.sky.update(time, sun_direction);

        if let Some(ref mut fog) = self.fog {
            fog.update(time);
        }

        if let Some(ref mut weather) = self.weather {
            weather.update(time);
        }
    }

    /// Render the atmosphere
    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera: &Camera) -> Result<()> {
        // Render sky
        self.sky.render(render_pass, camera)?;

        // Fog is applied as post-process or in shaders

        // Render weather effects
        if let Some(ref weather) = self.weather {
            weather.render(render_pass, camera)?;
        }

        Ok(())
    }
}

impl Default for AtmosphereSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atmosphere_creation() {
        let atmosphere = AtmosphereSystem::new();
        assert!(atmosphere.fog().is_none());
        assert!(atmosphere.weather().is_none());
    }

    #[test]
    fn test_fog_enable() {
        let mut atmosphere = AtmosphereSystem::new();
        atmosphere.enable_fog(FogParameters::default());
        assert!(atmosphere.fog().is_some());

        atmosphere.disable_fog();
        assert!(atmosphere.fog().is_none());
    }

    #[test]
    fn test_weather_enable() {
        let mut atmosphere = AtmosphereSystem::new();
        atmosphere.enable_weather();
        assert!(atmosphere.weather().is_some());

        atmosphere.disable_weather();
        assert!(atmosphere.weather().is_none());
    }
}
