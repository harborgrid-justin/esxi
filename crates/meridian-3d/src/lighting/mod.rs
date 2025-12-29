//! Lighting system for realistic 3D rendering
//!
//! Provides:
//! - Physical Based Rendering (PBR) materials
//! - Real-time shadow mapping
//! - Sun/directional lighting with time-of-day
//! - Ambient occlusion
//! - Point and spot lights

pub mod sun;
pub mod ambient;
pub mod pbr;

pub use sun::{SunLight, SunParameters};
pub use ambient::{AmbientOcclusion, AmbientLight};
pub use pbr::{PbrMaterial, MaterialProperties};

use crate::{Camera, Error, Result};
use glam::{Vec3, Vec4, Mat4};
use wgpu::{Device, Queue, Texture, TextureView, Buffer};
use std::sync::Arc;
use parking_lot::RwLock;

/// Main lighting system managing all lights and shadows
pub struct LightingSystem {
    /// Sun/directional light
    sun: Option<SunLight>,

    /// Ambient light
    ambient: AmbientLight,

    /// Point lights
    point_lights: Vec<PointLight>,

    /// Spot lights
    spot_lights: Vec<SpotLight>,

    /// Shadow map for sun
    shadow_map: Option<ShadowMap>,

    /// Ambient occlusion
    ambient_occlusion: Option<AmbientOcclusion>,

    /// GPU buffer for light data
    light_buffer: Option<Buffer>,
}

impl LightingSystem {
    /// Create a new lighting system
    pub fn new() -> Self {
        Self {
            sun: None,
            ambient: AmbientLight::default(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            shadow_map: None,
            ambient_occlusion: None,
            light_buffer: None,
        }
    }

    /// Set the sun light
    pub fn set_sun(&mut self, sun: SunLight) {
        self.sun = Some(sun);
    }

    /// Get the sun light
    pub fn sun(&self) -> Option<&SunLight> {
        self.sun.as_ref()
    }

    /// Get mutable sun light
    pub fn sun_mut(&mut self) -> Option<&mut SunLight> {
        self.sun.as_mut()
    }

    /// Set ambient light
    pub fn set_ambient(&mut self, ambient: AmbientLight) {
        self.ambient = ambient;
    }

    /// Get ambient light
    pub fn ambient(&self) -> &AmbientLight {
        &self.ambient
    }

    /// Add a point light
    pub fn add_point_light(&mut self, light: PointLight) {
        self.point_lights.push(light);
    }

    /// Add a spot light
    pub fn add_spot_light(&mut self, light: SpotLight) {
        self.spot_lights.push(light);
    }

    /// Get all point lights
    pub fn point_lights(&self) -> &[PointLight] {
        &self.point_lights
    }

    /// Get all spot lights
    pub fn spot_lights(&self) -> &[SpotLight] {
        &self.spot_lights
    }

    /// Initialize shadow mapping
    pub fn init_shadow_mapping(&mut self, device: &Device, shadow_map_size: u32) -> Result<()> {
        self.shadow_map = Some(ShadowMap::new(device, shadow_map_size)?);
        Ok(())
    }

    /// Initialize ambient occlusion
    pub fn init_ambient_occlusion(&mut self, device: &Device) -> Result<()> {
        self.ambient_occlusion = Some(AmbientOcclusion::new(device)?);
        Ok(())
    }

    /// Update lighting system
    pub fn update(&mut self, time: f32, camera: &Camera) {
        // Update sun position based on time
        if let Some(ref mut sun) = self.sun {
            sun.update(time);
        }

        // Update shadow map if needed
        // ...
    }

    /// Render shadows
    pub fn render_shadows(&self, device: &Device, queue: &Queue) -> Result<()> {
        if let Some(ref shadow_map) = self.shadow_map {
            // Render shadow map pass
            // ...
        }
        Ok(())
    }

    /// Get light count
    pub fn light_count(&self) -> usize {
        let sun_count = if self.sun.is_some() { 1 } else { 0 };
        sun_count + self.point_lights.len() + self.spot_lights.len()
    }

    /// Clear all lights
    pub fn clear_lights(&mut self) {
        self.sun = None;
        self.point_lights.clear();
        self.spot_lights.clear();
    }
}

impl Default for LightingSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Point light (omnidirectional)
#[derive(Debug, Clone)]
pub struct PointLight {
    /// Light position
    pub position: Vec3,

    /// Light color (RGB)
    pub color: Vec3,

    /// Light intensity
    pub intensity: f32,

    /// Attenuation radius
    pub radius: f32,

    /// Cast shadows
    pub cast_shadows: bool,
}

impl PointLight {
    /// Create a new point light
    pub fn new(position: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            position,
            color,
            intensity,
            radius: 10.0,
            cast_shadows: false,
        }
    }

    /// Calculate attenuation at a distance
    pub fn attenuation(&self, distance: f32) -> f32 {
        let d = distance / self.radius;
        1.0 / (1.0 + d * d)
    }
}

/// Spot light (cone-shaped)
#[derive(Debug, Clone)]
pub struct SpotLight {
    /// Light position
    pub position: Vec3,

    /// Direction the light is pointing
    pub direction: Vec3,

    /// Light color (RGB)
    pub color: Vec3,

    /// Light intensity
    pub intensity: f32,

    /// Inner cone angle (radians)
    pub inner_angle: f32,

    /// Outer cone angle (radians)
    pub outer_angle: f32,

    /// Attenuation radius
    pub radius: f32,

    /// Cast shadows
    pub cast_shadows: bool,
}

impl SpotLight {
    /// Create a new spot light
    pub fn new(position: Vec3, direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            position,
            direction: direction.normalize(),
            color,
            intensity,
            inner_angle: 30.0_f32.to_radians(),
            outer_angle: 45.0_f32.to_radians(),
            radius: 20.0,
            cast_shadows: false,
        }
    }

    /// Calculate spotlight cone attenuation
    pub fn cone_attenuation(&self, light_to_point: Vec3) -> f32 {
        let cos_angle = self.direction.dot(light_to_point.normalize());
        let cos_inner = self.inner_angle.cos();
        let cos_outer = self.outer_angle.cos();

        if cos_angle > cos_inner {
            1.0
        } else if cos_angle > cos_outer {
            ((cos_angle - cos_outer) / (cos_inner - cos_outer)).powi(2)
        } else {
            0.0
        }
    }
}

/// Shadow map for directional/sun light
pub struct ShadowMap {
    /// Shadow map texture
    texture: Texture,

    /// Shadow map view
    view: TextureView,

    /// Shadow map size
    size: u32,

    /// Light space matrix
    light_matrix: Mat4,
}

impl ShadowMap {
    /// Create a new shadow map
    pub fn new(device: &Device, size: u32) -> Result<Self> {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            texture,
            view,
            size,
            light_matrix: Mat4::IDENTITY,
        })
    }

    /// Get the shadow map texture view
    pub fn view(&self) -> &TextureView {
        &self.view
    }

    /// Get the light space matrix
    pub fn light_matrix(&self) -> Mat4 {
        self.light_matrix
    }

    /// Update light space matrix
    pub fn update_light_matrix(&mut self, light_dir: Vec3, scene_center: Vec3, scene_radius: f32) {
        let light_pos = scene_center - light_dir * scene_radius * 2.0;

        let view = Mat4::look_at_rh(light_pos, scene_center, Vec3::Y);
        let proj = Mat4::orthographic_rh(
            -scene_radius,
            scene_radius,
            -scene_radius,
            scene_radius,
            0.1,
            scene_radius * 4.0,
        );

        self.light_matrix = proj * view;
    }

    /// Get shadow map size
    pub fn size(&self) -> u32 {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lighting_system() {
        let mut system = LightingSystem::new();

        assert_eq!(system.light_count(), 0);

        let point_light = PointLight::new(Vec3::new(0.0, 10.0, 0.0), Vec3::ONE, 1.0);
        system.add_point_light(point_light);

        assert_eq!(system.light_count(), 1);
    }

    #[test]
    fn test_point_light_attenuation() {
        let light = PointLight::new(Vec3::ZERO, Vec3::ONE, 1.0);

        let atten_near = light.attenuation(0.0);
        let atten_far = light.attenuation(light.radius);

        assert!(atten_near > atten_far);
        assert!(atten_near <= 1.0);
    }

    #[test]
    fn test_spot_light_cone() {
        let light = SpotLight::new(
            Vec3::ZERO,
            Vec3::NEG_Z,
            Vec3::ONE,
            1.0,
        );

        // Point directly in light direction
        let atten_center = light.cone_attenuation(Vec3::NEG_Z);
        assert_eq!(atten_center, 1.0);

        // Point outside cone
        let atten_outside = light.cone_attenuation(Vec3::X);
        assert_eq!(atten_outside, 0.0);
    }
}
