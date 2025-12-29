//! # Meridian 3D
//!
//! 3D terrain and building visualization for Meridian GIS Platform.
//!
//! This crate provides GPU-accelerated 3D rendering capabilities including:
//! - Dynamic terrain rendering with LOD
//! - Building extrusion and procedural generation
//! - Real-time shadow mapping and PBR materials
//! - Atmospheric effects (sky, fog, weather)
//! - 3D analysis tools (viewshed, shadow analysis)
//! - Interactive 3D navigation
//! - Export capabilities for screenshots and video
//!
//! ## Architecture
//!
//! The crate is built on WebGPU (wgpu) for cross-platform GPU rendering,
//! with support for desktop and web targets.
//!
//! ## Example
//!
//! ```rust,no_run
//! use meridian_3d::{Scene, TerrainRenderer, RenderContext};
//!
//! async fn render_terrain() -> Result<(), Box<dyn std::error::Error>> {
//!     let ctx = RenderContext::new().await?;
//!     let mut scene = Scene::new();
//!     let terrain = TerrainRenderer::from_heightmap("elevation.tif").await?;
//!     scene.add_terrain(terrain);
//!
//!     // Render loop
//!     ctx.render(&scene)?;
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod error;
pub mod scene;
pub mod terrain;
pub mod buildings;
pub mod lighting;
pub mod atmosphere;
pub mod analysis;
pub mod interaction;
pub mod export;

// Re-exports
pub use error::{Error, Result};
pub use scene::{Scene, SceneNode, Transform};
pub use terrain::{TerrainRenderer, TerrainMesh, HeightmapSource};
pub use buildings::{BuildingRenderer, BuildingModel, ProceduralBuilding};
pub use lighting::{LightingSystem, SunLight, PbrMaterial};
pub use atmosphere::{SkyRenderer, FogEffect, WeatherSystem};
pub use analysis::{ViewshedAnalysis, ShadowAnalysis, FlythroughPath};
pub use interaction::{Picker, NavigationController, CameraMode};
pub use export::{ScreenshotExporter, VideoRecorder};

use glam::{Mat4, Vec3, Vec2, Quat};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, Adapter, Instance};
use std::sync::Arc;
use parking_lot::RwLock;

/// Core rendering context managing GPU resources
pub struct RenderContext {
    /// WebGPU instance
    pub instance: Instance,
    /// GPU adapter
    pub adapter: Adapter,
    /// Logical device
    pub device: Device,
    /// Command queue
    pub queue: Queue,
    /// Surface for rendering (optional for offscreen)
    pub surface: Option<Surface<'static>>,
    /// Surface configuration
    pub config: Option<SurfaceConfiguration>,
}

impl RenderContext {
    /// Create a new render context with default settings
    pub async fn new() -> Result<Self> {
        Self::with_backends(wgpu::Backends::all()).await
    }

    /// Create a render context with specific backends
    pub async fn with_backends(backends: wgpu::Backends) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(Error::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Meridian 3D Device"),
                    required_features: wgpu::Features::TEXTURE_COMPRESSION_BC
                        | wgpu::Features::DEPTH_CLIP_CONTROL
                        | wgpu::Features::MULTI_DRAW_INDIRECT
                        | wgpu::Features::PUSH_CONSTANTS,
                    required_limits: wgpu::Limits {
                        max_push_constant_size: 128,
                        ..Default::default()
                    },
                },
                None,
            )
            .await?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: None,
            config: None,
        })
    }

    /// Attach a surface for window rendering
    pub fn attach_surface(&mut self, surface: Surface<'static>, width: u32, height: u32) {
        let config = surface
            .get_default_config(&self.adapter, width, height)
            .expect("Surface incompatible with adapter");

        surface.configure(&self.device, &config);
        self.surface = Some(surface);
        self.config = Some(config);
    }

    /// Render a scene to the attached surface or offscreen
    pub fn render(&self, scene: &Scene) -> Result<()> {
        if let (Some(surface), Some(config)) = (&self.surface, &self.config) {
            let output = surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                },
            );

            scene.render(&self.device, &self.queue, &mut encoder, &view)?;

            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        } else {
            // Offscreen rendering
            let mut encoder = self.device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("Offscreen Encoder"),
                },
            );

            // Create offscreen texture
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Offscreen Texture"),
                size: wgpu::Extent3d {
                    width: 1920,
                    height: 1080,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            scene.render(&self.device, &self.queue, &mut encoder, &view)?;

            self.queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }

    /// Resize the rendering surface
    pub fn resize(&mut self, width: u32, height: u32) {
        if let (Some(surface), Some(config)) = (&self.surface, &mut self.config) {
            config.width = width;
            config.height = height;
            surface.configure(&self.device, config);
        }
    }
}

/// Camera for 3D scene navigation
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera position in world space
    pub position: Vec3,
    /// Camera target/look-at point
    pub target: Vec3,
    /// Up vector
    pub up: Vec3,
    /// Field of view in radians
    pub fov: f32,
    /// Aspect ratio (width/height)
    pub aspect: f32,
    /// Near clipping plane
    pub near: f32,
    /// Far clipping plane
    pub far: f32,
}

impl Camera {
    /// Create a new perspective camera
    pub fn new(position: Vec3, target: Vec3, aspect: f32) -> Self {
        Self {
            position,
            target,
            up: Vec3::Y,
            fov: 60.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 100000.0,
        }
    }

    /// Get the view matrix
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Get the projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    /// Get combined view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Get camera direction
    pub fn direction(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    /// Get camera right vector
    pub fn right(&self) -> Vec3 {
        self.direction().cross(self.up).normalize()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(
            Vec3::new(0.0, 100.0, 100.0),
            Vec3::ZERO,
            16.0 / 9.0,
        )
    }
}

/// Vertex format for 3D rendering
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    /// Position in 3D space
    pub position: [f32; 3],
    /// Normal vector
    pub normal: [f32; 3],
    /// Texture coordinates
    pub tex_coords: [f32; 2],
    /// Tangent vector for normal mapping
    pub tangent: [f32; 4],
}

impl Vertex {
    /// Vertex buffer layout descriptor
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // TexCoords
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Tangent
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Common utilities
pub mod utils {
    use super::*;

    /// Convert geographic coordinates to 3D world coordinates
    pub fn geo_to_world(lon: f64, lat: f64, elevation: f64, origin: (f64, f64)) -> Vec3 {
        const METERS_PER_DEGREE: f64 = 111_320.0;

        let x = (lon - origin.0) * METERS_PER_DEGREE * lat.to_radians().cos();
        let y = elevation;
        let z = (lat - origin.1) * METERS_PER_DEGREE;

        Vec3::new(x as f32, y as f32, z as f32)
    }

    /// Convert world coordinates back to geographic
    pub fn world_to_geo(pos: Vec3, origin: (f64, f64)) -> (f64, f64, f64) {
        const METERS_PER_DEGREE: f64 = 111_320.0;

        let lat = origin.1 + (pos.z as f64 / METERS_PER_DEGREE);
        let lon = origin.0 + (pos.x as f64 / (METERS_PER_DEGREE * lat.to_radians().cos()));
        let elevation = pos.y as f64;

        (lon, lat, elevation)
    }

    /// Calculate surface normal from three vertices
    pub fn calculate_normal(v0: Vec3, v1: Vec3, v2: Vec3) -> Vec3 {
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        edge1.cross(edge2).normalize()
    }

    /// Calculate tangent vector for normal mapping
    pub fn calculate_tangent(
        v0: Vec3, v1: Vec3, v2: Vec3,
        uv0: Vec2, uv1: Vec2, uv2: Vec2,
    ) -> glam::Vec4 {
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        let tangent = Vec3::new(
            f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
        ).normalize();

        glam::Vec4::new(tangent.x, tangent.y, tangent.z, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_matrices() {
        let camera = Camera::default();
        let view = camera.view_matrix();
        let proj = camera.projection_matrix();
        let vp = camera.view_projection_matrix();

        assert_eq!(vp, proj * view);
    }

    #[test]
    fn test_geo_conversion() {
        let origin = (-122.4194, 37.7749); // San Francisco
        let world_pos = utils::geo_to_world(-122.4194, 37.7749, 10.0, origin);
        assert!((world_pos.x - 0.0).abs() < 0.1);
        assert!((world_pos.y - 10.0).abs() < 0.1);
        assert!((world_pos.z - 0.0).abs() < 0.1);

        let (lon, lat, elev) = utils::world_to_geo(world_pos, origin);
        assert!((lon - -122.4194).abs() < 0.0001);
        assert!((lat - 37.7749).abs() < 0.0001);
        assert!((elev - 10.0).abs() < 0.1);
    }
}
