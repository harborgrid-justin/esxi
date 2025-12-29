//! Texture atlas management for efficient texture binding.

use crate::error::{MapEngineError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Device, Extent3d, Queue, Texture, TextureView};

/// Texture handle for referencing loaded textures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(u64);

impl TextureHandle {
    /// Create a new texture handle.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the internal ID.
    pub fn id(&self) -> u64 {
        self.0
    }
}

/// Region within a texture atlas.
#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    /// X coordinate in the atlas.
    pub x: u32,
    /// Y coordinate in the atlas.
    pub y: u32,
    /// Width of the region.
    pub width: u32,
    /// Height of the region.
    pub height: u32,
    /// Normalized UV coordinates (min).
    pub uv_min: [f32; 2],
    /// Normalized UV coordinates (max).
    pub uv_max: [f32; 2],
}

impl AtlasRegion {
    /// Create a new atlas region.
    pub fn new(x: u32, y: u32, width: u32, height: u32, atlas_width: u32, atlas_height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            uv_min: [
                x as f32 / atlas_width as f32,
                y as f32 / atlas_height as f32,
            ],
            uv_max: [
                (x + width) as f32 / atlas_width as f32,
                (y + height) as f32 / atlas_height as f32,
            ],
        }
    }
}

/// Texture atlas for batching multiple textures into one.
pub struct TextureAtlas {
    texture: Texture,
    view: TextureView,
    width: u32,
    height: u32,
    regions: HashMap<TextureHandle, AtlasRegion>,
    next_x: u32,
    next_y: u32,
    row_height: u32,
}

impl TextureAtlas {
    /// Create a new texture atlas.
    pub fn new(device: &Device, width: u32, height: u32, label: &str) -> Self {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            width,
            height,
            regions: HashMap::new(),
            next_x: 0,
            next_y: 0,
            row_height: 0,
        }
    }

    /// Add an image to the atlas.
    pub fn add_image(
        &mut self,
        queue: &Queue,
        handle: TextureHandle,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<AtlasRegion> {
        // Check if we need to move to next row
        if self.next_x + width > self.width {
            self.next_x = 0;
            self.next_y += self.row_height;
            self.row_height = 0;
        }

        // Check if atlas is full
        if self.next_y + height > self.height {
            return Err(MapEngineError::Texture(
                "Texture atlas is full".to_string(),
            ));
        }

        let region = AtlasRegion::new(
            self.next_x,
            self.next_y,
            width,
            height,
            self.width,
            self.height,
        );

        // Upload texture data
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: region.x,
                    y: region.y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Update position for next texture
        self.next_x += width;
        self.row_height = self.row_height.max(height);

        self.regions.insert(handle, region);

        Ok(region)
    }

    /// Get a region by handle.
    pub fn get_region(&self, handle: TextureHandle) -> Option<&AtlasRegion> {
        self.regions.get(&handle)
    }

    /// Get the texture view.
    pub fn view(&self) -> &TextureView {
        &self.view
    }

    /// Get atlas dimensions.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the number of regions in the atlas.
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Calculate the fill percentage of the atlas.
    pub fn fill_percentage(&self) -> f32 {
        let used_pixels: u32 = self
            .regions
            .values()
            .map(|r| r.width * r.height)
            .sum();
        let total_pixels = self.width * self.height;
        (used_pixels as f32 / total_pixels as f32) * 100.0
    }
}

/// Manages multiple texture atlases and individual textures.
pub struct TextureManager {
    device: Arc<Device>,
    queue: Arc<Queue>,
    atlases: Vec<TextureAtlas>,
    standalone_textures: HashMap<TextureHandle, (Texture, TextureView)>,
    sampler: wgpu::Sampler,
    max_texture_size: u32,
    next_handle_id: u64,
}

impl TextureManager {
    /// Create a new texture manager.
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, max_texture_size: u32) -> Result<Self> {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: 16,
            border_color: None,
        });

        Ok(Self {
            device,
            queue,
            atlases: Vec::new(),
            standalone_textures: HashMap::new(),
            sampler,
            max_texture_size,
            next_handle_id: 0,
        })
    }

    /// Create a new texture atlas.
    pub fn create_atlas(&mut self, width: u32, height: u32) -> usize {
        let atlas = TextureAtlas::new(
            &self.device,
            width.min(self.max_texture_size),
            height.min(self.max_texture_size),
            &format!("Texture Atlas {}", self.atlases.len()),
        );
        self.atlases.push(atlas);
        self.atlases.len() - 1
    }

    /// Add an image to an atlas.
    pub fn add_to_atlas(
        &mut self,
        atlas_index: usize,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(TextureHandle, AtlasRegion)> {
        let handle = self.generate_handle();

        let atlas = self
            .atlases
            .get_mut(atlas_index)
            .ok_or_else(|| MapEngineError::Texture("Atlas not found".to_string()))?;

        let region = atlas.add_image(&self.queue, handle, data, width, height)?;

        Ok((handle, region))
    }

    /// Create a standalone texture (not in an atlas).
    pub fn create_texture(
        &mut self,
        data: &[u8],
        width: u32,
        height: u32,
        label: &str,
    ) -> Result<TextureHandle> {
        let handle = self.generate_handle();

        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.standalone_textures.insert(handle, (texture, view));

        Ok(handle)
    }

    /// Get an atlas by index.
    pub fn get_atlas(&self, index: usize) -> Option<&TextureAtlas> {
        self.atlases.get(index)
    }

    /// Get a standalone texture view.
    pub fn get_texture_view(&self, handle: TextureHandle) -> Option<&TextureView> {
        self.standalone_textures.get(&handle).map(|(_, view)| view)
    }

    /// Get the default sampler.
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    /// Generate a unique texture handle.
    fn generate_handle(&mut self) -> TextureHandle {
        let handle = TextureHandle::new(self.next_handle_id);
        self.next_handle_id += 1;
        handle
    }

    /// Get texture manager statistics.
    pub fn stats(&self) -> TextureStats {
        TextureStats {
            atlas_count: self.atlases.len(),
            standalone_texture_count: self.standalone_textures.len(),
            total_atlas_regions: self.atlases.iter().map(|a| a.region_count()).sum(),
        }
    }
}

/// Statistics about texture memory usage.
#[derive(Debug, Clone)]
pub struct TextureStats {
    /// Number of texture atlases.
    pub atlas_count: usize,
    /// Number of standalone textures.
    pub standalone_texture_count: usize,
    /// Total number of regions across all atlases.
    pub total_atlas_regions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_region() {
        let region = AtlasRegion::new(0, 0, 256, 256, 1024, 1024);
        assert_eq!(region.x, 0);
        assert_eq!(region.y, 0);
        assert_eq!(region.width, 256);
        assert_eq!(region.height, 256);
        assert_eq!(region.uv_min, [0.0, 0.0]);
        assert_eq!(region.uv_max, [0.25, 0.25]);
    }

    #[test]
    fn test_texture_handle() {
        let handle1 = TextureHandle::new(0);
        let handle2 = TextureHandle::new(1);
        assert_ne!(handle1, handle2);
        assert_eq!(handle1.id(), 0);
        assert_eq!(handle2.id(), 1);
    }
}
