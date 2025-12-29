//! Terrain mesh generation

use crate::{Error, Result, Vertex};
use super::{Heightmap, HeightmapSource};
use glam::{Vec2, Vec3, Vec4};
use std::sync::Arc;

/// Vertex specifically for terrain rendering
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    /// Position in 3D space
    pub position: [f32; 3],
    /// Normal vector
    pub normal: [f32; 3],
    /// Texture coordinates
    pub tex_coords: [f32; 2],
    /// Terrain-specific data (slope, height, etc.)
    pub terrain_data: [f32; 4],
}

impl MeshVertex {
    /// Create a new mesh vertex
    pub fn new(position: Vec3, normal: Vec3, tex_coords: Vec2) -> Self {
        Self {
            position: position.into(),
            normal: normal.into(),
            tex_coords: tex_coords.into(),
            terrain_data: [0.0; 4],
        }
    }

    /// Convert to standard Vertex
    pub fn to_vertex(&self) -> Vertex {
        Vertex {
            position: self.position,
            normal: self.normal,
            tex_coords: self.tex_coords,
            tangent: [1.0, 0.0, 0.0, 1.0],
        }
    }
}

/// Terrain mesh at a specific LOD level
pub struct TerrainMesh {
    /// Vertex data
    vertices: Vec<MeshVertex>,

    /// Index data (triangles)
    indices: Vec<u32>,

    /// Resolution (vertices per side)
    resolution: usize,

    /// Mesh bounds
    bounds: (Vec3, Vec3),
}

impl TerrainMesh {
    /// Generate a terrain mesh from a heightmap
    pub fn from_heightmap(heightmap: Arc<Heightmap>, resolution: usize) -> Result<Self> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let width = heightmap.width() as f32;
        let height = heightmap.height() as f32;

        let step_x = width / (resolution - 1) as f32;
        let step_z = height / (resolution - 1) as f32;

        let mut min_bounds = Vec3::splat(f32::INFINITY);
        let mut max_bounds = Vec3::splat(f32::NEG_INFINITY);

        // Generate vertices
        for z in 0..resolution {
            for x in 0..resolution {
                let world_x = x as f32 * step_x;
                let world_z = z as f32 * step_z;

                let elevation = heightmap.sample(world_x, world_z);
                let position = Vec3::new(world_x, elevation, world_z);

                // Calculate normal
                let normal = heightmap.calculate_normal(world_x, world_z);

                // Texture coordinates
                let tex_coords = Vec2::new(
                    x as f32 / (resolution - 1) as f32,
                    z as f32 / (resolution - 1) as f32,
                );

                let mut vertex = MeshVertex::new(position, normal, tex_coords);

                // Store terrain-specific data
                let slope = normal.y.acos(); // Angle from up vector
                vertex.terrain_data = [
                    elevation,
                    slope,
                    0.0, // Reserved
                    0.0, // Reserved
                ];

                vertices.push(vertex);

                // Update bounds
                min_bounds = min_bounds.min(position);
                max_bounds = max_bounds.max(position);
            }
        }

        // Generate indices (triangles)
        for z in 0..resolution - 1 {
            for x in 0..resolution - 1 {
                let top_left = (z * resolution + x) as u32;
                let top_right = top_left + 1;
                let bottom_left = top_left + resolution as u32;
                let bottom_right = bottom_left + 1;

                // First triangle
                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

                // Second triangle
                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }

        Ok(Self {
            vertices,
            indices,
            resolution,
            bounds: (min_bounds, max_bounds),
        })
    }

    /// Create a flat plane mesh (for testing)
    pub fn plane(width: f32, height: f32, resolution: usize) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let step_x = width / (resolution - 1) as f32;
        let step_z = height / (resolution - 1) as f32;

        // Generate vertices
        for z in 0..resolution {
            for x in 0..resolution {
                let world_x = x as f32 * step_x;
                let world_z = z as f32 * step_z;

                let position = Vec3::new(world_x, 0.0, world_z);
                let normal = Vec3::Y;
                let tex_coords = Vec2::new(
                    x as f32 / (resolution - 1) as f32,
                    z as f32 / (resolution - 1) as f32,
                );

                vertices.push(MeshVertex::new(position, normal, tex_coords));
            }
        }

        // Generate indices
        for z in 0..resolution - 1 {
            for x in 0..resolution - 1 {
                let top_left = (z * resolution + x) as u32;
                let top_right = top_left + 1;
                let bottom_left = top_left + resolution as u32;
                let bottom_right = bottom_left + 1;

                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }

        Self {
            vertices,
            indices,
            resolution,
            bounds: (Vec3::ZERO, Vec3::new(width, 0.0, height)),
        }
    }

    /// Get vertices
    pub fn vertices(&self) -> &[MeshVertex] {
        &self.vertices
    }

    /// Get indices
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get index count
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Get mesh bounds
    pub fn bounds(&self) -> (Vec3, Vec3) {
        self.bounds
    }

    /// Get resolution
    pub fn resolution(&self) -> usize {
        self.resolution
    }

    /// Optimize mesh (reduce vertex count, improve cache coherency)
    pub fn optimize(&mut self) -> Result<()> {
        // Use meshopt for optimization
        // TODO: Implement mesh optimization
        Ok(())
    }

    /// Recalculate normals (smooth shading)
    pub fn recalculate_normals(&mut self) {
        // Reset normals to zero
        for vertex in &mut self.vertices {
            vertex.normal = [0.0, 0.0, 0.0];
        }

        // Accumulate face normals
        for i in (0..self.indices.len()).step_by(3) {
            let i0 = self.indices[i] as usize;
            let i1 = self.indices[i + 1] as usize;
            let i2 = self.indices[i + 2] as usize;

            let v0 = Vec3::from(self.vertices[i0].position);
            let v1 = Vec3::from(self.vertices[i1].position);
            let v2 = Vec3::from(self.vertices[i2].position);

            let normal = (v1 - v0).cross(v2 - v0);

            for idx in [i0, i1, i2] {
                let n = Vec3::from(self.vertices[idx].normal);
                self.vertices[idx].normal = (n + normal).into();
            }
        }

        // Normalize
        for vertex in &mut self.vertices {
            let normal = Vec3::from(vertex.normal).normalize();
            vertex.normal = normal.into();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_mesh() {
        let mesh = TerrainMesh::plane(100.0, 100.0, 10);

        assert_eq!(mesh.vertex_count(), 100); // 10x10 grid
        assert_eq!(mesh.triangle_count(), 162); // 9x9 quads * 2 triangles
        assert_eq!(mesh.resolution(), 10);
    }

    #[test]
    fn test_mesh_bounds() {
        let mesh = TerrainMesh::plane(100.0, 50.0, 10);
        let (min, max) = mesh.bounds();

        assert_eq!(min, Vec3::ZERO);
        assert_eq!(max, Vec3::new(100.0, 0.0, 50.0));
    }
}
