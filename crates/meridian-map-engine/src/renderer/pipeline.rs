//! WebGPU render pipeline management.

use crate::error::{MapEngineError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{
    BindGroupLayout, ColorTargetState, Device, PipelineLayout, RenderPipeline,
    SurfaceConfiguration,
};

/// Pipeline identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineId {
    /// Base 2D rendering pipeline.
    Base,
    /// Vector layer rendering.
    Vector,
    /// Raster/tile rendering.
    Raster,
    /// Text/label rendering.
    Text,
    /// Marker/icon rendering.
    Marker,
    /// Feature picking pipeline.
    Picking,
}

/// Configuration for creating a render pipeline.
pub struct PipelineDescriptor {
    /// Pipeline identifier.
    pub id: PipelineId,
    /// Vertex shader module.
    pub vertex_shader: wgpu::ShaderModule,
    /// Fragment shader module.
    pub fragment_shader: wgpu::ShaderModule,
    /// Vertex buffer layouts.
    pub vertex_layouts: Vec<wgpu::VertexBufferLayout<'static>>,
    /// Bind group layouts.
    pub bind_group_layouts: Vec<BindGroupLayout>,
    /// Primitive topology.
    pub primitive: wgpu::PrimitiveState,
    /// Depth/stencil state.
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    /// Multisample state.
    pub multisample: wgpu::MultisampleState,
    /// Blend state.
    pub blend: Option<wgpu::BlendState>,
}

/// Manages render pipelines for different layer types.
pub struct PipelineManager {
    device: Arc<Device>,
    pipelines: HashMap<PipelineId, RenderPipeline>,
    layouts: HashMap<PipelineId, PipelineLayout>,
    bind_group_layouts: HashMap<String, BindGroupLayout>,
}

impl PipelineManager {
    /// Create a new pipeline manager.
    pub fn new(device: Arc<Device>, surface_config: &SurfaceConfiguration) -> Result<Self> {
        let mut manager = Self {
            device,
            pipelines: HashMap::new(),
            layouts: HashMap::new(),
            bind_group_layouts: HashMap::new(),
        };

        // Create common bind group layouts
        manager.create_common_bind_groups();

        Ok(manager)
    }

    /// Create common bind group layouts used across pipelines.
    fn create_common_bind_groups(&mut self) {
        // Camera/view uniform bind group
        let camera_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        self.bind_group_layouts
            .insert("camera".to_string(), camera_layout);

        // Texture sampler bind group
        let texture_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        self.bind_group_layouts
            .insert("texture".to_string(), texture_layout);

        // Style/uniforms bind group
        let style_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Style Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        self.bind_group_layouts
            .insert("style".to_string(), style_layout);
    }

    /// Create and register a new render pipeline.
    pub fn create_pipeline(&mut self, desc: PipelineDescriptor) -> Result<()> {
        let layout_refs: Vec<&BindGroupLayout> = desc.bind_group_layouts.iter().collect();

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{:?} Pipeline Layout", desc.id)),
                bind_group_layouts: &layout_refs,
                push_constant_ranges: &[],
            });

        let vertex_layouts: Vec<wgpu::VertexBufferLayout> = desc.vertex_layouts.clone();

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{:?} Render Pipeline", desc.id)),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &desc.vertex_shader,
                    entry_point: "vs_main",
                    buffers: &vertex_layouts,
                },
                primitive: desc.primitive,
                depth_stencil: desc.depth_stencil.clone(),
                multisample: desc.multisample,
                fragment: Some(wgpu::FragmentState {
                    module: &desc.fragment_shader,
                    entry_point: "fs_main",
                    targets: &[Some(ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: desc.blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        self.layouts.insert(desc.id, pipeline_layout);
        self.pipelines.insert(desc.id, pipeline);

        Ok(())
    }

    /// Get a render pipeline by ID.
    pub fn get_pipeline(&self, id: PipelineId) -> Option<&RenderPipeline> {
        self.pipelines.get(&id)
    }

    /// Get a pipeline layout by ID.
    pub fn get_layout(&self, id: PipelineId) -> Option<&PipelineLayout> {
        self.layouts.get(&id)
    }

    /// Get a bind group layout by name.
    pub fn get_bind_group_layout(&self, name: &str) -> Option<&BindGroupLayout> {
        self.bind_group_layouts.get(name)
    }

    /// Check if a pipeline exists.
    pub fn has_pipeline(&self, id: PipelineId) -> bool {
        self.pipelines.contains_key(&id)
    }

    /// Get the default primitive state for 2D rendering.
    pub fn default_primitive() -> wgpu::PrimitiveState {
        wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None, // No culling for 2D
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        }
    }

    /// Get the default blend state for alpha blending.
    pub fn default_blend() -> wgpu::BlendState {
        wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        }
    }

    /// Get the default multisample state.
    pub fn default_multisample(samples: u32) -> wgpu::MultisampleState {
        wgpu::MultisampleState {
            count: samples,
            mask: !0,
            alpha_to_coverage_enabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_id_uniqueness() {
        let ids = vec![
            PipelineId::Base,
            PipelineId::Vector,
            PipelineId::Raster,
            PipelineId::Text,
            PipelineId::Marker,
            PipelineId::Picking,
        ];

        for (i, id1) in ids.iter().enumerate() {
            for (j, id2) in ids.iter().enumerate() {
                if i == j {
                    assert_eq!(id1, id2);
                } else {
                    assert_ne!(id1, id2);
                }
            }
        }
    }
}
