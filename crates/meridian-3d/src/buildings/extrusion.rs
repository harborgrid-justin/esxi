//! Building footprint extrusion

use crate::{Error, Result, Vertex};
use super::BuildingModel;
use glam::{Vec2, Vec3, Vec4};

/// Parameters for footprint extrusion
#[derive(Debug, Clone)]
pub struct ExtrusionParams {
    /// Building height
    pub height: f32,

    /// Floor height (for generating floors)
    pub floor_height: f32,

    /// Roof type
    pub roof_type: RoofType,

    /// Roof height (for pitched roofs)
    pub roof_height: f32,

    /// Generate UVs for facade texturing
    pub generate_uvs: bool,

    /// Inset walls from footprint edge
    pub wall_inset: f32,
}

impl Default for ExtrusionParams {
    fn default() -> Self {
        Self {
            height: 10.0,
            floor_height: 3.0,
            roof_type: RoofType::Flat,
            roof_height: 0.0,
            generate_uvs: true,
            wall_inset: 0.0,
        }
    }
}

/// Type of roof
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoofType {
    /// Flat roof
    Flat,
    /// Gabled roof (peaked)
    Gabled,
    /// Hipped roof
    Hipped,
    /// Pyramid roof
    Pyramid,
}

/// Footprint extruder for creating 3D buildings from 2D polygons
pub struct FootprintExtruder {
    /// Options
    options: ExtrusionOptions,
}

/// Extrusion options
struct ExtrusionOptions {
    /// Generate smooth normals at corners
    smooth_corners: bool,

    /// Cap the bottom
    cap_bottom: bool,

    /// Cap the top
    cap_top: bool,
}

impl Default for ExtrusionOptions {
    fn default() -> Self {
        Self {
            smooth_corners: false,
            cap_bottom: true,
            cap_top: true,
        }
    }
}

impl FootprintExtruder {
    /// Create a new footprint extruder
    pub fn new() -> Self {
        Self {
            options: ExtrusionOptions::default(),
        }
    }

    /// Extrude a footprint polygon into a 3D building
    pub fn extrude(&self, footprint: &[Vec2], params: ExtrusionParams) -> Result<BuildingModel> {
        if footprint.len() < 3 {
            return Err(Error::mesh("Footprint must have at least 3 vertices"));
        }

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Apply wall inset if needed
        let footprint = if params.wall_inset > 0.0 {
            self.inset_polygon(footprint, params.wall_inset)?
        } else {
            footprint.to_vec()
        };

        // Generate walls
        self.generate_walls(&footprint, &params, &mut vertices, &mut indices);

        // Generate bottom cap
        if self.options.cap_bottom {
            self.generate_bottom_cap(&footprint, &mut vertices, &mut indices);
        }

        // Generate roof/top
        match params.roof_type {
            RoofType::Flat => {
                if self.options.cap_top {
                    self.generate_flat_roof(&footprint, params.height, &mut vertices, &mut indices);
                }
            }
            RoofType::Gabled => {
                self.generate_gabled_roof(&footprint, params.height, params.roof_height, &mut vertices, &mut indices);
            }
            RoofType::Hipped => {
                self.generate_hipped_roof(&footprint, params.height, params.roof_height, &mut vertices, &mut indices);
            }
            RoofType::Pyramid => {
                self.generate_pyramid_roof(&footprint, params.height, params.roof_height, &mut vertices, &mut indices);
            }
        }

        BuildingModel::from_geometry(vertices, indices)
    }

    /// Generate wall geometry
    fn generate_walls(
        &self,
        footprint: &[Vec2],
        params: &ExtrusionParams,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
    ) {
        let n = footprint.len();

        for i in 0..n {
            let p0 = footprint[i];
            let p1 = footprint[(i + 1) % n];

            // Wall quad vertices
            let bottom_left = Vec3::new(p0.x, 0.0, p0.y);
            let bottom_right = Vec3::new(p1.x, 0.0, p1.y);
            let top_left = Vec3::new(p0.x, params.height, p0.y);
            let top_right = Vec3::new(p1.x, params.height, p1.y);

            // Calculate normal (outward facing)
            let edge = p1 - p0;
            let normal_2d = Vec2::new(-edge.y, edge.x).normalize();
            let normal = Vec3::new(normal_2d.x, 0.0, normal_2d.y);

            // UVs for facade texturing
            let edge_length = edge.length();
            let u0 = 0.0;
            let u1 = edge_length / params.floor_height; // Scale UVs by floor height
            let v0 = 0.0;
            let v1 = params.height / params.floor_height;

            let base_idx = vertices.len() as u32;

            // Add vertices
            vertices.push(Vertex {
                position: bottom_left.into(),
                normal: normal.into(),
                tex_coords: [u0, v0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });

            vertices.push(Vertex {
                position: bottom_right.into(),
                normal: normal.into(),
                tex_coords: [u1, v0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });

            vertices.push(Vertex {
                position: top_right.into(),
                normal: normal.into(),
                tex_coords: [u1, v1],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });

            vertices.push(Vertex {
                position: top_left.into(),
                normal: normal.into(),
                tex_coords: [u0, v1],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });

            // Add indices (two triangles per wall)
            indices.extend_from_slice(&[
                base_idx, base_idx + 1, base_idx + 2,
                base_idx, base_idx + 2, base_idx + 3,
            ]);
        }
    }

    /// Generate bottom cap (floor)
    fn generate_bottom_cap(
        &self,
        footprint: &[Vec2],
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
    ) {
        let base_idx = vertices.len() as u32;

        // Triangulate footprint
        let triangles = self.triangulate_polygon(footprint);

        // Add vertices
        for &point in footprint {
            vertices.push(Vertex {
                position: [point.x, 0.0, point.y],
                normal: [0.0, -1.0, 0.0], // Downward normal
                tex_coords: [point.x, point.y],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });
        }

        // Add indices (reverse winding for bottom face)
        for tri in triangles {
            indices.push(base_idx + tri[2]);
            indices.push(base_idx + tri[1]);
            indices.push(base_idx + tri[0]);
        }
    }

    /// Generate flat roof
    fn generate_flat_roof(
        &self,
        footprint: &[Vec2],
        height: f32,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
    ) {
        let base_idx = vertices.len() as u32;
        let triangles = self.triangulate_polygon(footprint);

        for &point in footprint {
            vertices.push(Vertex {
                position: [point.x, height, point.y],
                normal: [0.0, 1.0, 0.0], // Upward normal
                tex_coords: [point.x, point.y],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });
        }

        for tri in triangles {
            indices.extend_from_slice(&[
                base_idx + tri[0],
                base_idx + tri[1],
                base_idx + tri[2],
            ]);
        }
    }

    /// Generate gabled roof
    fn generate_gabled_roof(
        &self,
        footprint: &[Vec2],
        base_height: f32,
        roof_height: f32,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
    ) {
        // Simplified gabled roof - find longest edge as ridge
        let center = self.polygon_center(footprint);
        let apex = Vec3::new(center.x, base_height + roof_height, center.y);

        // Create roof faces from apex to each edge
        let base_idx = vertices.len() as u32;
        let n = footprint.len();

        for i in 0..n {
            let p0 = footprint[i];
            let p1 = footprint[(i + 1) % n];

            let v0 = Vec3::new(p0.x, base_height, p0.y);
            let v1 = Vec3::new(p1.x, base_height, p1.y);

            let normal = (v1 - apex).cross(v0 - apex).normalize();

            vertices.push(Vertex {
                position: apex.into(),
                normal: normal.into(),
                tex_coords: [0.5, 1.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });

            vertices.push(Vertex {
                position: v0.into(),
                normal: normal.into(),
                tex_coords: [0.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });

            vertices.push(Vertex {
                position: v1.into(),
                normal: normal.into(),
                tex_coords: [1.0, 0.0],
                tangent: [1.0, 0.0, 0.0, 1.0],
            });

            indices.extend_from_slice(&[base_idx + i as u32 * 3, base_idx + i as u32 * 3 + 1, base_idx + i as u32 * 3 + 2]);
        }
    }

    /// Generate hipped roof
    fn generate_hipped_roof(
        &self,
        footprint: &[Vec2],
        base_height: f32,
        roof_height: f32,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
    ) {
        // Similar to gabled but with hip edges
        // For simplicity, use pyramid approach
        self.generate_pyramid_roof(footprint, base_height, roof_height, vertices, indices);
    }

    /// Generate pyramid roof
    fn generate_pyramid_roof(
        &self,
        footprint: &[Vec2],
        base_height: f32,
        roof_height: f32,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
    ) {
        let center = self.polygon_center(footprint);
        let apex = Vec3::new(center.x, base_height + roof_height, center.y);

        let base_idx = vertices.len() as u32;
        let n = footprint.len();

        for i in 0..n {
            let p0 = footprint[i];
            let p1 = footprint[(i + 1) % n];

            let v0 = Vec3::new(p0.x, base_height, p0.y);
            let v1 = Vec3::new(p1.x, base_height, p1.y);

            let normal = (v1 - apex).cross(v0 - apex).normalize();

            vertices.extend_from_slice(&[
                Vertex {
                    position: apex.into(),
                    normal: normal.into(),
                    tex_coords: [0.5, 1.0],
                    tangent: [1.0, 0.0, 0.0, 1.0],
                },
                Vertex {
                    position: v0.into(),
                    normal: normal.into(),
                    tex_coords: [0.0, 0.0],
                    tangent: [1.0, 0.0, 0.0, 1.0],
                },
                Vertex {
                    position: v1.into(),
                    normal: normal.into(),
                    tex_coords: [1.0, 0.0],
                    tangent: [1.0, 0.0, 0.0, 1.0],
                },
            ]);

            indices.extend_from_slice(&[base_idx + i as u32 * 3, base_idx + i as u32 * 3 + 1, base_idx + i as u32 * 3 + 2]);
        }
    }

    /// Triangulate a polygon using ear clipping
    fn triangulate_polygon(&self, polygon: &[Vec2]) -> Vec<[u32; 3]> {
        // Simplified triangulation - fan triangulation from first vertex
        let mut triangles = Vec::new();

        for i in 1..polygon.len() - 1 {
            triangles.push([0, i as u32, (i + 1) as u32]);
        }

        triangles
    }

    /// Calculate polygon center
    fn polygon_center(&self, polygon: &[Vec2]) -> Vec2 {
        let sum: Vec2 = polygon.iter().sum();
        sum / polygon.len() as f32
    }

    /// Inset a polygon by a given distance
    fn inset_polygon(&self, polygon: &[Vec2], distance: f32) -> Result<Vec<Vec2>> {
        // Simplified inset - move each vertex inward along angle bisector
        let n = polygon.len();
        let mut inset = Vec::with_capacity(n);

        for i in 0..n {
            let prev = polygon[(i + n - 1) % n];
            let curr = polygon[i];
            let next = polygon[(i + 1) % n];

            let v1 = (curr - prev).normalize();
            let v2 = (next - curr).normalize();
            let bisector = (v1 + v2).normalize();

            inset.push(curr - bisector * distance);
        }

        Ok(inset)
    }
}

impl Default for FootprintExtruder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footprint_extrusion() {
        let footprint = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];

        let params = ExtrusionParams {
            height: 20.0,
            ..Default::default()
        };

        let extruder = FootprintExtruder::new();
        let model = extruder.extrude(&footprint, params).unwrap();

        assert!(model.vertex_count() > 0);
        assert!(model.index_count() > 0);
    }

    #[test]
    fn test_roof_types() {
        let footprint = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ];

        let extruder = FootprintExtruder::new();

        for roof_type in [RoofType::Flat, RoofType::Gabled, RoofType::Pyramid] {
            let params = ExtrusionParams {
                height: 15.0,
                roof_type,
                roof_height: 5.0,
                ..Default::default()
            };

            let model = extruder.extrude(&footprint, params).unwrap();
            assert!(model.vertex_count() > 0);
        }
    }
}
