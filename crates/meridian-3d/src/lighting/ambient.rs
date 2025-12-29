//! Ambient lighting and occlusion

use crate::{Error, Result};
use glam::{Vec3, Vec4};
use wgpu::{Device, Texture, TextureView};

/// Ambient light (global illumination)
#[derive(Debug, Clone)]
pub struct AmbientLight {
    /// Ambient color
    pub color: Vec3,

    /// Ambient intensity
    pub intensity: f32,

    /// Use image-based lighting
    pub use_ibl: bool,
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: Vec3::new(0.3, 0.3, 0.4), // Slight blue tint for sky
            intensity: 0.3,
            use_ibl: false,
        }
    }
}

impl AmbientLight {
    /// Create a new ambient light
    pub fn new(color: Vec3, intensity: f32) -> Self {
        Self {
            color,
            intensity,
            use_ibl: false,
        }
    }

    /// Get the final ambient color
    pub fn final_color(&self) -> Vec3 {
        self.color * self.intensity
    }

    /// Create ambient light from sky color
    pub fn from_sky(sky_color: Vec3, intensity: f32) -> Self {
        Self::new(sky_color, intensity)
    }
}

/// Ambient occlusion system
pub struct AmbientOcclusion {
    /// AO texture (screen-space or pre-baked)
    ao_texture: Option<Texture>,

    /// AO parameters
    params: AoParameters,

    /// Enable SSAO
    ssao_enabled: bool,
}

/// Ambient occlusion parameters
#[derive(Debug, Clone)]
pub struct AoParameters {
    /// AO strength/intensity
    pub strength: f32,

    /// AO radius (for SSAO)
    pub radius: f32,

    /// Number of samples (for SSAO)
    pub sample_count: u32,

    /// Bias to prevent acne
    pub bias: f32,
}

impl Default for AoParameters {
    fn default() -> Self {
        Self {
            strength: 1.0,
            radius: 0.5,
            sample_count: 16,
            bias: 0.025,
        }
    }
}

impl AmbientOcclusion {
    /// Create a new ambient occlusion system
    pub fn new(device: &Device) -> Result<Self> {
        Ok(Self {
            ao_texture: None,
            params: AoParameters::default(),
            ssao_enabled: true,
        })
    }

    /// Enable/disable SSAO
    pub fn set_ssao_enabled(&mut self, enabled: bool) {
        self.ssao_enabled = enabled;
    }

    /// Is SSAO enabled
    pub fn is_ssao_enabled(&self) -> bool {
        self.ssao_enabled
    }

    /// Get AO parameters
    pub fn params(&self) -> &AoParameters {
        &self.params
    }

    /// Get mutable AO parameters
    pub fn params_mut(&mut self) -> &mut AoParameters {
        &mut self.params
    }

    /// Set AO strength
    pub fn set_strength(&mut self, strength: f32) {
        self.params.strength = strength.clamp(0.0, 2.0);
    }

    /// Set AO radius
    pub fn set_radius(&mut self, radius: f32) {
        self.params.radius = radius.clamp(0.1, 2.0);
    }

    /// Render SSAO (Screen-Space Ambient Occlusion)
    pub fn render_ssao(
        &self,
        device: &Device,
        depth_texture: &TextureView,
        normal_texture: &TextureView,
    ) -> Result<Texture> {
        // SSAO rendering would happen here
        // For now, create a dummy texture

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Texture"),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        Ok(texture)
    }

    /// Sample AO value at a UV coordinate
    pub fn sample_ao(&self, uv: (f32, f32)) -> f32 {
        // If we have a pre-baked AO texture, sample it
        // For now, return default value
        1.0
    }

    /// Generate random sample kernel for SSAO
    pub fn generate_sample_kernel(&self) -> Vec<Vec3> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut kernel = Vec::with_capacity(self.params.sample_count as usize);

        for i in 0..self.params.sample_count {
            // Random sample in hemisphere
            let mut sample = Vec3::new(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(0.0..1.0),
            )
            .normalize();

            // Scale samples closer to origin
            let scale = i as f32 / self.params.sample_count as f32;
            let scale = 0.1 + scale * scale * 0.9;

            sample *= scale;
            kernel.push(sample);
        }

        kernel
    }

    /// Generate random noise texture for SSAO
    pub fn generate_noise_texture(&self, device: &Device, size: u32) -> Texture {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut noise_data = Vec::with_capacity((size * size * 4) as usize);

        for _ in 0..size * size {
            // Random rotation vector
            let x = rng.gen_range(-1.0..1.0_f32);
            let y = rng.gen_range(-1.0..1.0_f32);

            // Encode as RGBA (only using RG for 2D rotation)
            noise_data.push((x * 127.5 + 127.5) as u8);
            noise_data.push((y * 127.5 + 127.5) as u8);
            noise_data.push(0);
            noise_data.push(255);
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("SSAO Noise"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload noise data
        // queue.write_texture(...);

        texture
    }
}

/// Horizon-based ambient occlusion (HBAO)
pub struct HbaoRenderer {
    /// HBAO parameters
    params: HbaoParameters,
}

/// HBAO parameters
#[derive(Debug, Clone)]
pub struct HbaoParameters {
    /// Number of directions to sample
    pub directions: u32,

    /// Number of steps per direction
    pub steps: u32,

    /// AO radius
    pub radius: f32,

    /// AO strength
    pub strength: f32,

    /// Angle bias
    pub angle_bias: f32,
}

impl Default for HbaoParameters {
    fn default() -> Self {
        Self {
            directions: 8,
            steps: 4,
            radius: 0.5,
            strength: 1.0,
            angle_bias: 0.1,
        }
    }
}

impl HbaoRenderer {
    /// Create a new HBAO renderer
    pub fn new() -> Self {
        Self {
            params: HbaoParameters::default(),
        }
    }

    /// Get parameters
    pub fn params(&self) -> &HbaoParameters {
        &self.params
    }

    /// Get mutable parameters
    pub fn params_mut(&mut self) -> &mut HbaoParameters {
        &mut self.params
    }
}

impl Default for HbaoRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ambient_light() {
        let ambient = AmbientLight::default();

        assert_eq!(ambient.intensity, 0.3);

        let final_color = ambient.final_color();
        assert!(final_color.length() > 0.0);
    }

    #[test]
    fn test_ambient_from_sky() {
        let sky_color = Vec3::new(0.5, 0.7, 1.0);
        let ambient = AmbientLight::from_sky(sky_color, 0.5);

        assert_eq!(ambient.color, sky_color);
        assert_eq!(ambient.intensity, 0.5);
    }

    #[test]
    fn test_ao_parameters() {
        let params = AoParameters::default();

        assert_eq!(params.sample_count, 16);
        assert_eq!(params.strength, 1.0);
    }

    #[test]
    fn test_hbao_parameters() {
        let params = HbaoParameters::default();

        assert_eq!(params.directions, 8);
        assert_eq!(params.steps, 4);
    }
}
