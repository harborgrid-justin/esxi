//! 3D building model loading (glTF)

use crate::{Error, Result, Vertex};
use glam::{Vec2, Vec3, Quat};
use std::path::Path;

/// 3D building model
pub struct BuildingModel {
    /// Vertex data
    vertices: Vec<Vertex>,

    /// Index data
    indices: Vec<u32>,

    /// Model bounds
    bounds: (Vec3, Vec3),

    /// Materials
    materials: Vec<Material>,
}

impl BuildingModel {
    /// Create a building model from geometry
    pub fn from_geometry(vertices: Vec<Vertex>, indices: Vec<u32>) -> Result<Self> {
        if vertices.is_empty() {
            return Err(Error::mesh("Empty vertex buffer"));
        }

        // Calculate bounds
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);

        for vertex in &vertices {
            let pos = Vec3::from(vertex.position);
            min = min.min(pos);
            max = max.max(pos);
        }

        Ok(Self {
            vertices,
            indices,
            bounds: (min, max),
            materials: Vec::new(),
        })
    }

    /// Get vertices
    pub fn vertices(&self) -> &[Vertex] {
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

    /// Get bounds
    pub fn bounds(&self) -> (Vec3, Vec3) {
        self.bounds
    }

    /// Add a material
    pub fn add_material(&mut self, material: Material) {
        self.materials.push(material);
    }

    /// Get materials
    pub fn materials(&self) -> &[Material] {
        &self.materials
    }
}

/// Material definition
#[derive(Debug, Clone)]
pub struct Material {
    /// Material name
    pub name: String,

    /// Base color
    pub base_color: [f32; 4],

    /// Metallic factor
    pub metallic: f32,

    /// Roughness factor
    pub roughness: f32,

    /// Emissive color
    pub emissive: [f32; 3],
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            base_color: [0.8, 0.8, 0.8, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
        }
    }
}

/// glTF model loader
pub struct ModelLoader;

impl ModelLoader {
    /// Load a glTF model
    pub async fn load_gltf(path: impl AsRef<Path>) -> Result<BuildingModel> {
        let (document, buffers, _images) = gltf::import(path)?;

        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut materials = Vec::new();

        // Load materials
        for material in document.materials() {
            let pbr = material.pbr_metallic_roughness();

            materials.push(Material {
                name: material.name().unwrap_or("Unnamed").to_string(),
                base_color: pbr.base_color_factor(),
                metallic: pbr.metallic_factor(),
                roughness: pbr.roughness_factor(),
                emissive: material.emissive_factor(),
            });
        }

        // Load meshes
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                // Read positions
                let positions: Vec<[f32; 3]> = if let Some(iter) = reader.read_positions() {
                    iter.collect()
                } else {
                    continue;
                };

                // Read normals
                let normals: Vec<[f32; 3]> = if let Some(iter) = reader.read_normals() {
                    iter.collect()
                } else {
                    vec![[0.0, 1.0, 0.0]; positions.len()]
                };

                // Read texture coordinates
                let tex_coords: Vec<[f32; 2]> = if let Some(iter) = reader.read_tex_coords(0) {
                    iter.into_f32().collect()
                } else {
                    vec![[0.0, 0.0]; positions.len()]
                };

                // Read tangents (or generate default)
                let tangents: Vec<[f32; 4]> = if let Some(iter) = reader.read_tangents() {
                    iter.collect()
                } else {
                    vec![[1.0, 0.0, 0.0, 1.0]; positions.len()]
                };

                // Create vertices
                let vertex_offset = all_vertices.len() as u32;

                for i in 0..positions.len() {
                    all_vertices.push(Vertex {
                        position: positions[i],
                        normal: normals[i],
                        tex_coords: tex_coords[i],
                        tangent: tangents[i],
                    });
                }

                // Read indices
                if let Some(indices_reader) = reader.read_indices() {
                    for index in indices_reader.into_u32() {
                        all_indices.push(vertex_offset + index);
                    }
                }
            }
        }

        let mut model = BuildingModel::from_geometry(all_vertices, all_indices)?;

        for material in materials {
            model.add_material(material);
        }

        Ok(model)
    }

    /// Load from OBJ file (simplified)
    pub async fn load_obj(path: impl AsRef<Path>) -> Result<BuildingModel> {
        // OBJ loading would go here
        // For now, return a placeholder
        Err(Error::not_found("OBJ loading not implemented"))
    }

    /// Create a simple cube model (for testing)
    pub fn create_cube(size: f32) -> BuildingModel {
        let s = size / 2.0;

        let vertices = vec![
            // Front face
            Vertex {
                position: [-s, -s, s],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [0.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [s, -s, s],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [1.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [s, s, s],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [1.0, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-s, s, s],
                normal: [0.0, 0.0, 1.0],
                tex_coords: [0.0, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            // Add other faces...
        ];

        let indices = vec![
            0, 1, 2, 0, 2, 3, // Front
        ];

        BuildingModel::from_geometry(vertices, indices).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_model_creation() {
        let vertices = vec![
            Vertex {
                position: [0.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [1.0, 0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, 1.0, 0.0],
                normal: [0.0, 1.0, 0.0],
                tex_coords: [0.5, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            },
        ];

        let indices = vec![0, 1, 2];

        let model = BuildingModel::from_geometry(vertices, indices).unwrap();

        assert_eq!(model.vertex_count(), 3);
        assert_eq!(model.index_count(), 3);
    }

    #[test]
    fn test_cube_model() {
        let cube = ModelLoader::create_cube(10.0);

        assert!(cube.vertex_count() > 0);
        assert!(cube.index_count() > 0);
    }

    #[test]
    fn test_material_default() {
        let material = Material::default();

        assert_eq!(material.name, "Default");
        assert_eq!(material.base_color, [0.8, 0.8, 0.8, 1.0]);
    }
}
