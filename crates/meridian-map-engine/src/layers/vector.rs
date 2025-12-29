//! Vector layer rendering for points, lines, and polygons.

use super::{Layer, LayerProperties, LayerType, Visibility};
use crate::camera::Camera;
use crate::error::Result;
use crate::renderer::{buffer::BufferHandle, Renderer, Vertex};
use crate::style::StyleSpec;
use glam::Vec2;

/// Geometry type for vector features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryType {
    /// Point geometry.
    Point,
    /// Line string geometry.
    LineString,
    /// Polygon geometry.
    Polygon,
}

/// A vector feature with geometry and properties.
#[derive(Debug, Clone)]
pub struct VectorFeature {
    /// Feature ID.
    pub id: u64,
    /// Geometry type.
    pub geometry_type: GeometryType,
    /// Vertices for the geometry.
    pub vertices: Vec<Vec2>,
    /// Feature properties (for data-driven styling).
    pub properties: serde_json::Value,
    /// Computed style color.
    pub color: [f32; 4],
}

impl VectorFeature {
    /// Create a new point feature.
    pub fn point(id: u64, position: Vec2, properties: serde_json::Value) -> Self {
        Self {
            id,
            geometry_type: GeometryType::Point,
            vertices: vec![position],
            properties,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Create a new line feature.
    pub fn line(id: u64, vertices: Vec<Vec2>, properties: serde_json::Value) -> Self {
        Self {
            id,
            geometry_type: GeometryType::LineString,
            vertices,
            properties,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Create a new polygon feature.
    pub fn polygon(id: u64, vertices: Vec<Vec2>, properties: serde_json::Value) -> Self {
        Self {
            id,
            geometry_type: GeometryType::Polygon,
            vertices,
            properties,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Set the feature color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }
}

/// Vector layer for rendering geometric features.
pub struct VectorLayer {
    /// Layer properties.
    properties: LayerProperties,
    /// Features in this layer.
    features: Vec<VectorFeature>,
    /// Style specification.
    style: Option<StyleSpec>,
    /// Vertex buffer handle.
    vertex_buffer: Option<BufferHandle>,
    /// Index buffer handle.
    index_buffer: Option<BufferHandle>,
    /// Whether buffers need to be rebuilt.
    dirty: bool,
    /// Line width in pixels.
    line_width: f32,
    /// Point size in pixels.
    point_size: f32,
}

impl VectorLayer {
    /// Create a new vector layer.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            properties: LayerProperties::new(name, LayerType::Vector),
            features: Vec::new(),
            style: None,
            vertex_buffer: None,
            index_buffer: None,
            dirty: true,
            line_width: 2.0,
            point_size: 8.0,
        }
    }

    /// Add a feature to the layer.
    pub fn add_feature(&mut self, feature: VectorFeature) {
        self.features.push(feature);
        self.dirty = true;
    }

    /// Add multiple features.
    pub fn add_features(&mut self, features: Vec<VectorFeature>) {
        self.features.extend(features);
        self.dirty = true;
    }

    /// Clear all features.
    pub fn clear_features(&mut self) {
        self.features.clear();
        self.dirty = true;
    }

    /// Get the number of features.
    pub fn feature_count(&self) -> usize {
        self.features.len()
    }

    /// Set the style specification.
    pub fn set_style(&mut self, style: StyleSpec) {
        self.style = Some(style);
        self.dirty = true;
    }

    /// Set line width.
    pub fn set_line_width(&mut self, width: f32) {
        self.line_width = width.max(0.1);
    }

    /// Set point size.
    pub fn set_point_size(&mut self, size: f32) {
        self.point_size = size.max(1.0);
    }

    /// Build vertex and index buffers from features.
    fn build_buffers(&mut self, renderer: &mut Renderer) -> Result<()> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for feature in &self.features {
            match feature.geometry_type {
                GeometryType::Point => {
                    self.build_point_geometry(feature, &mut vertices, &mut indices);
                }
                GeometryType::LineString => {
                    self.build_line_geometry(feature, &mut vertices, &mut indices);
                }
                GeometryType::Polygon => {
                    self.build_polygon_geometry(feature, &mut vertices, &mut indices);
                }
            }
        }

        // Create GPU buffers
        if !vertices.is_empty() {
            let vertex_data = bytemuck::cast_slice(&vertices);
            self.vertex_buffer = Some(renderer.buffer_manager_mut().create_vertex_buffer(
                vertex_data,
                std::mem::size_of::<Vertex>() as u32,
                false,
            )?);

            self.index_buffer = Some(
                renderer
                    .buffer_manager_mut()
                    .create_index_buffer(&indices, false)?,
            );
        }

        self.dirty = false;
        Ok(())
    }

    /// Build geometry for a point feature (as a quad).
    fn build_point_geometry(
        &self,
        feature: &VectorFeature,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
    ) {
        if let Some(&center) = feature.vertices.first() {
            let base_index = vertices.len() as u32;
            let half_size = self.point_size * 0.5;

            // Create quad vertices
            vertices.push(Vertex {
                position: [center.x - half_size, center.y - half_size],
                tex_coords: [0.0, 0.0],
                color: feature.color,
            });
            vertices.push(Vertex {
                position: [center.x + half_size, center.y - half_size],
                tex_coords: [1.0, 0.0],
                color: feature.color,
            });
            vertices.push(Vertex {
                position: [center.x + half_size, center.y + half_size],
                tex_coords: [1.0, 1.0],
                color: feature.color,
            });
            vertices.push(Vertex {
                position: [center.x - half_size, center.y + half_size],
                tex_coords: [0.0, 1.0],
                color: feature.color,
            });

            // Create quad indices (two triangles)
            indices.extend_from_slice(&[
                base_index,
                base_index + 1,
                base_index + 2,
                base_index,
                base_index + 2,
                base_index + 3,
            ]);
        }
    }

    /// Build geometry for a line feature.
    fn build_line_geometry(
        &self,
        feature: &VectorFeature,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
    ) {
        if feature.vertices.len() < 2 {
            return;
        }

        let base_index = vertices.len() as u32;
        let half_width = self.line_width * 0.5;

        // Create line segments with width
        for i in 0..feature.vertices.len() - 1 {
            let p0 = feature.vertices[i];
            let p1 = feature.vertices[i + 1];

            // Calculate perpendicular offset
            let dir = (p1 - p0).normalize();
            let perp = Vec2::new(-dir.y, dir.x) * half_width;

            // Create quad for this segment
            let idx = base_index + (i * 4) as u32;
            vertices.push(Vertex {
                position: [(p0 - perp).x, (p0 - perp).y],
                tex_coords: [0.0, 0.0],
                color: feature.color,
            });
            vertices.push(Vertex {
                position: [(p0 + perp).x, (p0 + perp).y],
                tex_coords: [1.0, 0.0],
                color: feature.color,
            });
            vertices.push(Vertex {
                position: [(p1 + perp).x, (p1 + perp).y],
                tex_coords: [1.0, 1.0],
                color: feature.color,
            });
            vertices.push(Vertex {
                position: [(p1 - perp).x, (p1 - perp).y],
                tex_coords: [0.0, 1.0],
                color: feature.color,
            });

            indices.extend_from_slice(&[idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]);
        }
    }

    /// Build geometry for a polygon feature.
    fn build_polygon_geometry(
        &self,
        feature: &VectorFeature,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
    ) {
        if feature.vertices.len() < 3 {
            return;
        }

        let base_index = vertices.len() as u32;

        // Add vertices
        for &vertex in &feature.vertices {
            vertices.push(Vertex {
                position: [vertex.x, vertex.y],
                tex_coords: [0.0, 0.0],
                color: feature.color,
            });
        }

        // Simple fan triangulation (works for convex polygons)
        for i in 1..feature.vertices.len() - 1 {
            indices.push(base_index);
            indices.push(base_index + i as u32);
            indices.push(base_index + i as u32 + 1);
        }
    }
}

impl Layer for VectorLayer {
    fn properties(&self) -> &LayerProperties {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.properties
    }

    fn update(&mut self, _camera: &Camera, _delta_time: f32) -> Result<()> {
        // Update logic (e.g., apply data-driven styling) goes here
        Ok(())
    }

    fn render(&self, renderer: &mut Renderer, _camera: &Camera) -> Result<()> {
        // Note: In a complete implementation, this would use the actual render pipeline
        // For now, this is a placeholder showing the structure

        if let (Some(_vertex_buffer), Some(_index_buffer)) =
            (self.vertex_buffer, self.index_buffer)
        {
            // Render the vector layer
            // render_pass.set_pipeline(&vector_pipeline);
            // render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            // render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
            // render_pass.draw_indexed(0..index_count, 0, 0..1);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_feature_creation() {
        let point = VectorFeature::point(1, Vec2::new(0.0, 0.0), serde_json::json!({}));
        assert_eq!(point.geometry_type, GeometryType::Point);
        assert_eq!(point.vertices.len(), 1);

        let line = VectorFeature::line(
            2,
            vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0)],
            serde_json::json!({}),
        );
        assert_eq!(line.geometry_type, GeometryType::LineString);
        assert_eq!(line.vertices.len(), 2);
    }

    #[test]
    fn test_vector_layer() {
        let mut layer = VectorLayer::new("test_vector");
        assert_eq!(layer.feature_count(), 0);

        layer.add_feature(VectorFeature::point(1, Vec2::new(0.0, 0.0), serde_json::json!({})));
        assert_eq!(layer.feature_count(), 1);

        layer.clear_features();
        assert_eq!(layer.feature_count(), 0);
    }
}
