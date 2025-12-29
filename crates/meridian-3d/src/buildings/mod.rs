//! Building rendering and 3D model management
//!
//! Supports:
//! - Footprint extrusion for simple buildings
//! - glTF model loading for detailed buildings
//! - Procedural building generation
//! - Facade texturing and LOD

pub mod extrusion;
pub mod model;
pub mod procedural;
pub mod facade;

pub use extrusion::{FootprintExtruder, ExtrusionParams};
pub use model::{BuildingModel, ModelLoader};
pub use procedural::{ProceduralBuilding, BuildingGenerator, BuildingStyle};
pub use facade::{FacadeTexture, FacadePattern};

use crate::{Camera, Error, Result, Vertex};
use glam::{Vec2, Vec3};
use std::sync::Arc;
use wgpu::{Device, Queue, Buffer, RenderPass};

/// Main building renderer
pub struct BuildingRenderer {
    /// Building models
    models: Vec<BuildingModel>,

    /// GPU buffers
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,

    /// Render pipeline
    pipeline: Option<wgpu::RenderPipeline>,
}

impl BuildingRenderer {
    /// Create a new building renderer
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            vertex_buffer: None,
            index_buffer: None,
            pipeline: None,
        }
    }

    /// Add a building model
    pub fn add_model(&mut self, model: BuildingModel) {
        self.models.push(model);
    }

    /// Load a building from a glTF file
    pub async fn load_gltf(&mut self, path: &str) -> Result<()> {
        let model = ModelLoader::load_gltf(path).await?;
        self.models.push(model);
        Ok(())
    }

    /// Generate a procedural building
    pub fn generate_procedural(
        &mut self,
        footprint: &[Vec2],
        height: f32,
        style: BuildingStyle,
    ) -> Result<()> {
        let building = BuildingGenerator::generate(footprint, height, style)?;
        self.models.push(building.into());
        Ok(())
    }

    /// Extrude a building from footprint
    pub fn extrude_footprint(
        &mut self,
        footprint: &[Vec2],
        params: ExtrusionParams,
    ) -> Result<()> {
        let extruder = FootprintExtruder::new();
        let model = extruder.extrude(footprint, params)?;
        self.models.push(model);
        Ok(())
    }

    /// Initialize GPU resources
    pub fn init_gpu_resources(&mut self, device: &Device) -> Result<()> {
        // Combine all model vertices and indices
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();

        for model in &self.models {
            let vertex_offset = all_vertices.len() as u32;
            all_vertices.extend_from_slice(model.vertices());

            for &index in model.indices() {
                all_indices.push(index + vertex_offset);
            }
        }

        // Create GPU buffers
        if !all_vertices.is_empty() {
            use wgpu::util::DeviceExt;

            self.vertex_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Building Vertex Buffer"),
                    contents: bytemuck::cast_slice(&all_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ));

            self.index_buffer = Some(device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Building Index Buffer"),
                    contents: bytemuck::cast_slice(&all_indices),
                    usage: wgpu::BufferUsages::INDEX,
                },
            ));
        }

        Ok(())
    }

    /// Render all buildings
    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>) -> Result<()> {
        if let (Some(ref vbuf), Some(ref ibuf)) = (&self.vertex_buffer, &self.index_buffer) {
            render_pass.set_vertex_buffer(0, vbuf.slice(..));
            render_pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);

            let total_indices: u32 = self.models.iter()
                .map(|m| m.index_count() as u32)
                .sum();

            if total_indices > 0 {
                render_pass.draw_indexed(0..total_indices, 0, 0..1);
            }
        }

        Ok(())
    }

    /// Get building count
    pub fn count(&self) -> usize {
        self.models.len()
    }

    /// Clear all buildings
    pub fn clear(&mut self) {
        self.models.clear();
        self.vertex_buffer = None;
        self.index_buffer = None;
    }
}

impl Default for BuildingRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Building metadata
#[derive(Debug, Clone)]
pub struct BuildingMetadata {
    /// Building ID
    pub id: String,

    /// Building name
    pub name: Option<String>,

    /// Address
    pub address: Option<String>,

    /// Building type (residential, commercial, etc.)
    pub building_type: Option<String>,

    /// Height in meters
    pub height: f32,

    /// Number of floors
    pub floors: Option<u32>,

    /// Year built
    pub year_built: Option<u32>,

    /// Custom properties
    pub properties: std::collections::HashMap<String, String>,
}

impl BuildingMetadata {
    /// Create new building metadata
    pub fn new(id: impl Into<String>, height: f32) -> Self {
        Self {
            id: id.into(),
            name: None,
            address: None,
            building_type: None,
            height,
            floors: None,
            year_built: None,
            properties: std::collections::HashMap::new(),
        }
    }

    /// Set building name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set building type
    pub fn with_type(mut self, building_type: impl Into<String>) -> Self {
        self.building_type = Some(building_type.into());
        self
    }

    /// Set number of floors
    pub fn with_floors(mut self, floors: u32) -> Self {
        self.floors = Some(floors);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_renderer_creation() {
        let renderer = BuildingRenderer::new();
        assert_eq!(renderer.count(), 0);
    }

    #[test]
    fn test_building_metadata() {
        let metadata = BuildingMetadata::new("building_001", 25.0)
            .with_name("Empire State Building")
            .with_type("Commercial")
            .with_floors(102);

        assert_eq!(metadata.id, "building_001");
        assert_eq!(metadata.height, 25.0);
        assert_eq!(metadata.floors, Some(102));
    }
}
