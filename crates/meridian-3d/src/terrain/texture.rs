//! Terrain texturing and material splatting

use crate::{Error, Result};
use glam::{Vec2, Vec3, Vec4};
use image::DynamicImage;
use std::path::Path;
use wgpu::{Device, Texture, TextureView};

/// Terrain texture layer
#[derive(Debug, Clone)]
pub struct TerrainTextureLayer {
    /// Layer name
    pub name: String,

    /// Diffuse/albedo texture
    pub albedo: Option<DynamicImage>,

    /// Normal map
    pub normal: Option<DynamicImage>,

    /// Roughness map
    pub roughness: Option<DynamicImage>,

    /// Height/displacement map
    pub height: Option<DynamicImage>,

    /// UV scale
    pub uv_scale: f32,

    /// Blend mode
    pub blend_mode: BlendMode,
}

impl TerrainTextureLayer {
    /// Create a new texture layer
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            albedo: None,
            normal: None,
            roughness: None,
            height: None,
            uv_scale: 1.0,
            blend_mode: BlendMode::Height,
        }
    }

    /// Load albedo texture
    pub async fn with_albedo(mut self, path: impl AsRef<Path>) -> Result<Self> {
        self.albedo = Some(image::open(path)?);
        Ok(self)
    }

    /// Load normal map
    pub async fn with_normal(mut self, path: impl AsRef<Path>) -> Result<Self> {
        self.normal = Some(image::open(path)?);
        Ok(self)
    }

    /// Set UV scale
    pub fn with_uv_scale(mut self, scale: f32) -> Self {
        self.uv_scale = scale;
        self
    }
}

/// Texture blend mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Blend based on height/elevation
    Height,
    /// Blend based on slope
    Slope,
    /// Manual blend using splat map
    Splat,
    /// Triplanar projection
    Triplanar,
}

/// Texture splatting system for terrain
pub struct TextureSplatting {
    /// Texture layers
    layers: Vec<TerrainTextureLayer>,

    /// Splat map (controls layer blending)
    splat_map: Option<DynamicImage>,

    /// GPU textures
    gpu_textures: Vec<Option<Texture>>,
}

impl TextureSplatting {
    /// Create a new texture splatting system
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            splat_map: None,
            gpu_textures: Vec::new(),
        }
    }

    /// Add a texture layer
    pub fn add_layer(&mut self, layer: TerrainTextureLayer) {
        self.layers.push(layer);
    }

    /// Set the splat map
    pub fn set_splat_map(&mut self, splat_map: DynamicImage) {
        self.splat_map = Some(splat_map);
    }

    /// Get layer count
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Initialize GPU textures
    pub fn init_gpu_resources(&mut self, device: &Device) -> Result<()> {
        self.gpu_textures.clear();

        for layer in &self.layers {
            // Upload albedo texture
            if let Some(ref albedo) = layer.albedo {
                let texture = Self::create_texture_from_image(device, albedo)?;
                self.gpu_textures.push(Some(texture));
            } else {
                self.gpu_textures.push(None);
            }
        }

        Ok(())
    }

    /// Create a GPU texture from an image
    fn create_texture_from_image(device: &Device, image: &DynamicImage) -> Result<Texture> {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Terrain Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        Ok(texture)
    }

    /// Calculate blend weights for a terrain vertex
    pub fn calculate_blend_weights(&self, position: Vec3, normal: Vec3, height: f32) -> Vec4 {
        let mut weights = Vec4::ZERO;

        for (i, layer) in self.layers.iter().enumerate().take(4) {
            let weight = match layer.blend_mode {
                BlendMode::Height => self.height_blend(height, i),
                BlendMode::Slope => self.slope_blend(normal, i),
                BlendMode::Splat => self.splat_blend(position, i),
                BlendMode::Triplanar => 0.0, // Handled in shader
            };

            match i {
                0 => weights.x = weight,
                1 => weights.y = weight,
                2 => weights.z = weight,
                3 => weights.w = weight,
                _ => {}
            }
        }

        // Normalize weights
        let sum = weights.x + weights.y + weights.z + weights.w;
        if sum > 0.0 {
            weights /= sum;
        }

        weights
    }

    /// Calculate blend weight based on height
    fn height_blend(&self, height: f32, layer_index: usize) -> f32 {
        // Simple height-based blending
        let layer_count = self.layers.len().min(4);
        let height_per_layer = 1.0 / layer_count as f32;
        let layer_start = layer_index as f32 * height_per_layer;
        let layer_end = layer_start + height_per_layer;

        let normalized_height = height.clamp(0.0, 1.0);

        if normalized_height >= layer_start && normalized_height < layer_end {
            ((normalized_height - layer_start) / height_per_layer).min(1.0)
        } else {
            0.0
        }
    }

    /// Calculate blend weight based on slope
    fn slope_blend(&self, normal: Vec3, layer_index: usize) -> f32 {
        let slope = normal.y.acos(); // Angle from vertical

        // Different layers for different slopes
        match layer_index {
            0 => (1.0 - slope / std::f32::consts::FRAC_PI_2).max(0.0), // Flat areas
            1 => {
                let mid = std::f32::consts::FRAC_PI_4;
                1.0 - ((slope - mid).abs() / mid).min(1.0)
            } // Medium slopes
            2 => (slope / std::f32::consts::FRAC_PI_2).min(1.0), // Steep slopes
            _ => 0.0,
        }
    }

    /// Calculate blend weight from splat map
    fn splat_blend(&self, position: Vec3, layer_index: usize) -> f32 {
        if let Some(ref splat_map) = self.splat_map {
            // Sample splat map
            // TODO: Implement proper splat map sampling
            0.0
        } else {
            0.0
        }
    }
}

impl Default for TextureSplatting {
    fn default() -> Self {
        Self::new()
    }
}

/// Procedural texture generator for terrain
pub struct ProceduralTexture;

impl ProceduralTexture {
    /// Generate a grass texture
    pub fn grass(width: u32, height: u32) -> DynamicImage {
        let mut img = image::RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                // Simple green color with some variation
                let noise = ((x + y) as f32 * 0.1).sin() * 0.2 + 0.8;
                let r = (80.0 * noise) as u8;
                let g = (120.0 * noise) as u8;
                let b = (40.0 * noise) as u8;

                img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
            }
        }

        DynamicImage::ImageRgba8(img)
    }

    /// Generate a rock texture
    pub fn rock(width: u32, height: u32) -> DynamicImage {
        let mut img = image::RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let noise = ((x as f32 * 0.1).sin() + (y as f32 * 0.1).cos()) * 0.3 + 0.7;
                let gray = (100.0 * noise) as u8;

                img.put_pixel(x, y, image::Rgba([gray, gray, gray, 255]));
            }
        }

        DynamicImage::ImageRgba8(img)
    }

    /// Generate a sand texture
    pub fn sand(width: u32, height: u32) -> DynamicImage {
        let mut img = image::RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let noise = ((x + y) as f32 * 0.15).sin() * 0.1 + 0.9;
                let r = (230.0 * noise) as u8;
                let g = (210.0 * noise) as u8;
                let b = (170.0 * noise) as u8;

                img.put_pixel(x, y, image::Rgba([r, g, b, 255]));
            }
        }

        DynamicImage::ImageRgba8(img)
    }

    /// Generate a snow texture
    pub fn snow(width: u32, height: u32) -> DynamicImage {
        let mut img = image::RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let noise = ((x as f32 * 0.2).sin() * (y as f32 * 0.2).cos()) * 0.05 + 0.95;
                let value = (250.0 * noise) as u8;

                img.put_pixel(x, y, image::Rgba([value, value, value, 255]));
            }
        }

        DynamicImage::ImageRgba8(img)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_layer_creation() {
        let layer = TerrainTextureLayer::new("Grass")
            .with_uv_scale(2.0);

        assert_eq!(layer.name, "Grass");
        assert_eq!(layer.uv_scale, 2.0);
    }

    #[test]
    fn test_texture_splatting() {
        let mut splatting = TextureSplatting::new();

        let layer1 = TerrainTextureLayer::new("Layer1");
        let layer2 = TerrainTextureLayer::new("Layer2");

        splatting.add_layer(layer1);
        splatting.add_layer(layer2);

        assert_eq!(splatting.layer_count(), 2);
    }

    #[test]
    fn test_procedural_textures() {
        let grass = ProceduralTexture::grass(64, 64);
        assert_eq!(grass.width(), 64);
        assert_eq!(grass.height(), 64);

        let rock = ProceduralTexture::rock(64, 64);
        assert_eq!(rock.width(), 64);

        let sand = ProceduralTexture::sand(64, 64);
        assert_eq!(sand.width(), 64);

        let snow = ProceduralTexture::snow(64, 64);
        assert_eq!(snow.width(), 64);
    }

    #[test]
    fn test_blend_weight_calculation() {
        let splatting = TextureSplatting::new();

        let weights = splatting.calculate_blend_weights(
            Vec3::ZERO,
            Vec3::Y,
            0.5,
        );

        // Weights should sum to 1.0 or be all zeros
        let sum = weights.x + weights.y + weights.z + weights.w;
        assert!(sum == 0.0 || (sum - 1.0).abs() < 0.01);
    }
}
