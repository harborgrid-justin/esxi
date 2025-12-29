//! Fog and haze effects

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Type of fog
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FogType {
    /// Linear fog (based on distance)
    Linear,
    /// Exponential fog
    Exponential,
    /// Exponential squared fog
    ExponentialSquared,
    /// Height-based fog
    Height,
}

/// Fog parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FogParameters {
    /// Fog type
    pub fog_type: FogType,

    /// Fog color
    pub color: Vec3,

    /// Fog density
    pub density: f32,

    /// Start distance (for linear fog)
    pub start: f32,

    /// End distance (for linear fog)
    pub end: f32,

    /// Height falloff (for height fog)
    pub height_falloff: f32,

    /// Base height (for height fog)
    pub base_height: f32,
}

impl Default for FogParameters {
    fn default() -> Self {
        Self {
            fog_type: FogType::Exponential,
            color: Vec3::new(0.7, 0.8, 0.9),
            density: 0.001,
            start: 10.0,
            end: 100.0,
            height_falloff: 0.01,
            base_height: 0.0,
        }
    }
}

/// Fog effect
pub struct FogEffect {
    /// Fog parameters
    params: FogParameters,
}

impl FogEffect {
    /// Create a new fog effect
    pub fn new(params: FogParameters) -> Self {
        Self { params }
    }

    /// Update fog
    pub fn update(&mut self, time: f32) {
        // Fog can change over time (e.g., animated density)
    }

    /// Calculate fog factor at a distance
    pub fn calculate_fog_factor(&self, distance: f32, camera_height: f32, point_height: f32) -> f32 {
        match self.params.fog_type {
            FogType::Linear => self.linear_fog(distance),
            FogType::Exponential => self.exponential_fog(distance),
            FogType::ExponentialSquared => self.exponential_squared_fog(distance),
            FogType::Height => self.height_fog(distance, camera_height, point_height),
        }
    }

    /// Linear fog calculation
    fn linear_fog(&self, distance: f32) -> f32 {
        ((self.params.end - distance) / (self.params.end - self.params.start)).clamp(0.0, 1.0)
    }

    /// Exponential fog calculation
    fn exponential_fog(&self, distance: f32) -> f32 {
        (-self.params.density * distance).exp()
    }

    /// Exponential squared fog calculation
    fn exponential_squared_fog(&self, distance: f32) -> f32 {
        let factor = self.params.density * distance;
        (-factor * factor).exp()
    }

    /// Height-based fog calculation
    fn height_fog(&self, distance: f32, camera_height: f32, point_height: f32) -> f32 {
        let height_diff = (point_height - self.params.base_height).max(0.0);
        let height_factor = (-self.params.height_falloff * height_diff).exp();

        let base_fog = self.exponential_fog(distance);
        base_fog * height_factor
    }

    /// Apply fog to a color
    pub fn apply_fog(&self, color: Vec3, distance: f32, camera_height: f32, point_height: f32) -> Vec3 {
        let fog_factor = self.calculate_fog_factor(distance, camera_height, point_height);
        color.lerp(self.params.color, 1.0 - fog_factor)
    }

    /// Get parameters
    pub fn params(&self) -> &FogParameters {
        &self.params
    }

    /// Set parameters
    pub fn set_params(&mut self, params: FogParameters) {
        self.params = params;
    }

    /// Get fog color
    pub fn color(&self) -> Vec3 {
        self.params.color
    }

    /// Set fog color
    pub fn set_color(&mut self, color: Vec3) {
        self.params.color = color;
    }

    /// Get fog density
    pub fn density(&self) -> f32 {
        self.params.density
    }

    /// Set fog density
    pub fn set_density(&mut self, density: f32) {
        self.params.density = density.max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fog_creation() {
        let fog = FogEffect::new(FogParameters::default());
        assert_eq!(fog.params().fog_type, FogType::Exponential);
    }

    #[test]
    fn test_linear_fog() {
        let params = FogParameters {
            fog_type: FogType::Linear,
            start: 10.0,
            end: 100.0,
            ..Default::default()
        };

        let fog = FogEffect::new(params);

        // At start, fog factor should be 1.0 (no fog)
        let factor_start = fog.calculate_fog_factor(10.0, 0.0, 0.0);
        assert!((factor_start - 1.0).abs() < 0.01);

        // At end, fog factor should be 0.0 (full fog)
        let factor_end = fog.calculate_fog_factor(100.0, 0.0, 0.0);
        assert!((factor_end - 0.0).abs() < 0.01);

        // At middle, fog factor should be around 0.5
        let factor_mid = fog.calculate_fog_factor(55.0, 0.0, 0.0);
        assert!((factor_mid - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_exponential_fog() {
        let params = FogParameters {
            fog_type: FogType::Exponential,
            density: 0.1,
            ..Default::default()
        };

        let fog = FogEffect::new(params);

        let factor_near = fog.calculate_fog_factor(1.0, 0.0, 0.0);
        let factor_far = fog.calculate_fog_factor(10.0, 0.0, 0.0);

        // Fog should increase with distance
        assert!(factor_near > factor_far);
    }

    #[test]
    fn test_fog_application() {
        let fog = FogEffect::new(FogParameters {
            color: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        });

        let original_color = Vec3::new(1.0, 0.0, 0.0);
        let fogged_color = fog.apply_fog(original_color, 50.0, 0.0, 0.0);

        // Color should be blended towards fog color
        assert!(fogged_color.x >= original_color.x);
    }
}
