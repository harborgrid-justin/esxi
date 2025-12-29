//! Core rendering module for the map engine.

pub mod buffer;
pub mod pipeline;
pub mod shader;
pub mod texture;

use crate::error::{MapEngineError, Result};
use std::sync::Arc;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

/// Configuration for the renderer.
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Target frames per second.
    pub target_fps: u32,
    /// Enable VSync.
    pub vsync: bool,
    /// Maximum texture size.
    pub max_texture_size: u32,
    /// Enable MSAA (Multi-Sample Anti-Aliasing).
    pub msaa_samples: u32,
    /// Enable anisotropic filtering.
    pub anisotropic_filtering: u16,
    /// Maximum number of draw calls per frame.
    pub max_draw_calls: u32,
    /// Enable GPU instancing.
    pub enable_instancing: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            vsync: true,
            max_texture_size: 4096,
            msaa_samples: 4,
            anisotropic_filtering: 16,
            max_draw_calls: 1000,
            enable_instancing: true,
        }
    }
}

/// Main renderer for the map engine.
pub struct Renderer {
    /// WebGPU device.
    device: Arc<Device>,
    /// Command queue.
    queue: Arc<Queue>,
    /// Surface configuration.
    surface_config: SurfaceConfiguration,
    /// Renderer configuration.
    config: RendererConfig,
    /// Render pipeline manager.
    pipeline_manager: pipeline::PipelineManager,
    /// Shader manager.
    shader_manager: shader::ShaderManager,
    /// Texture atlas manager.
    texture_manager: texture::TextureManager,
    /// Buffer manager.
    buffer_manager: buffer::BufferManager,
    /// Frame counter for performance tracking.
    frame_count: u64,
}

impl Renderer {
    /// Create a new renderer instance.
    pub async fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        surface_config: SurfaceConfiguration,
        config: RendererConfig,
    ) -> Result<Self> {
        let shader_manager = shader::ShaderManager::new(device.clone());
        let pipeline_manager = pipeline::PipelineManager::new(device.clone(), &surface_config)?;
        let texture_manager = texture::TextureManager::new(
            device.clone(),
            queue.clone(),
            config.max_texture_size,
        )?;
        let buffer_manager = buffer::BufferManager::new(device.clone());

        Ok(Self {
            device,
            queue,
            surface_config,
            config,
            pipeline_manager,
            shader_manager,
            texture_manager,
            buffer_manager,
            frame_count: 0,
        })
    }

    /// Get reference to the device.
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get reference to the queue.
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Get reference to the surface configuration.
    pub fn surface_config(&self) -> &SurfaceConfiguration {
        &self.surface_config
    }

    /// Get reference to the renderer configuration.
    pub fn config(&self) -> &RendererConfig {
        &self.config
    }

    /// Get mutable reference to the pipeline manager.
    pub fn pipeline_manager_mut(&mut self) -> &mut pipeline::PipelineManager {
        &mut self.pipeline_manager
    }

    /// Get reference to the shader manager.
    pub fn shader_manager(&self) -> &shader::ShaderManager {
        &self.shader_manager
    }

    /// Get mutable reference to the texture manager.
    pub fn texture_manager_mut(&mut self) -> &mut texture::TextureManager {
        &mut self.texture_manager
    }

    /// Get mutable reference to the buffer manager.
    pub fn buffer_manager_mut(&mut self) -> &mut buffer::BufferManager {
        &mut self.buffer_manager
    }

    /// Resize the surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
    }

    /// Begin a new frame.
    pub fn begin_frame(&mut self) -> FrameContext {
        self.frame_count += 1;
        FrameContext {
            frame_number: self.frame_count,
            device: self.device.clone(),
            queue: self.queue.clone(),
        }
    }

    /// Get current frame count.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}

/// Context for a single frame rendering.
pub struct FrameContext {
    /// Current frame number.
    pub frame_number: u64,
    /// Device reference.
    pub device: Arc<Device>,
    /// Queue reference.
    pub queue: Arc<Queue>,
}

/// Vertex type for map rendering.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    /// Position in 2D map coordinates.
    pub position: [f32; 2],
    /// Texture coordinates.
    pub tex_coords: [f32; 2],
    /// Vertex color (RGBA).
    pub color: [f32; 4],
}

impl Vertex {
    /// Get the vertex buffer layout.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Tex coords
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Instance data for GPU instancing.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    /// Model matrix (4x4).
    pub transform: [[f32; 4]; 4],
    /// Instance color multiplier.
    pub color: [f32; 4],
}

impl InstanceData {
    /// Get the instance buffer layout.
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Transform matrix (4 vec4s)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 2) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 3) as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 4]>() * 4) as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
