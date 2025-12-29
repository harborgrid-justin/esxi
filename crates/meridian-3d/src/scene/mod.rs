//! Scene graph management system
//!
//! Provides hierarchical scene organization with transform propagation,
//! spatial culling, and efficient rendering.

pub mod node;
pub mod transform;

pub use node::{SceneNode, NodeId, NodeType};
pub use transform::Transform;

use crate::{Camera, Error, Result};
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use wgpu::{CommandEncoder, Device, Queue, TextureView};

/// Scene graph managing all 3D objects
pub struct Scene {
    /// Root node of the scene graph
    root: Arc<RwLock<SceneNode>>,

    /// Flat map of all nodes by ID for fast lookup
    nodes: Arc<RwLock<HashMap<NodeId, Arc<RwLock<SceneNode>>>>>,

    /// Active camera
    camera: Camera,

    /// Scene bounds for culling
    bounds: Option<BoundingBox>,

    /// Background color
    background_color: wgpu::Color,
}

impl Scene {
    /// Create a new empty scene
    pub fn new() -> Self {
        let root = Arc::new(RwLock::new(SceneNode::root()));
        let root_id = root.read().id();

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root.clone());

        Self {
            root,
            nodes: Arc::new(RwLock::new(nodes)),
            camera: Camera::default(),
            bounds: None,
            background_color: wgpu::Color {
                r: 0.53,
                g: 0.81,
                b: 0.92,
                a: 1.0,
            },
        }
    }

    /// Add a node to the scene
    pub fn add_node(&mut self, node: SceneNode) -> NodeId {
        let node_id = node.id();
        let node_arc = Arc::new(RwLock::new(node));

        self.nodes.write().insert(node_id, node_arc.clone());
        self.root.write().add_child(node_arc);

        node_id
    }

    /// Get a node by ID
    pub fn get_node(&self, id: NodeId) -> Option<Arc<RwLock<SceneNode>>> {
        self.nodes.read().get(&id).cloned()
    }

    /// Remove a node from the scene
    pub fn remove_node(&mut self, id: NodeId) -> Result<()> {
        if let Some(node) = self.nodes.write().remove(&id) {
            self.root.write().remove_child(id);
            Ok(())
        } else {
            Err(Error::not_found(format!("Node {} not found", id)))
        }
    }

    /// Get the active camera
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Get mutable camera reference
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Set the scene camera
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    /// Set background color
    pub fn set_background_color(&mut self, color: wgpu::Color) {
        self.background_color = color;
    }

    /// Update the scene (transform propagation, animations, etc.)
    pub fn update(&mut self, delta_time: f32) {
        self.root.write().update(delta_time);
        self.update_bounds();
    }

    /// Update scene bounds
    fn update_bounds(&mut self) {
        // TODO: Calculate from all nodes
        self.bounds = Some(BoundingBox::infinite());
    }

    /// Render the entire scene
    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
    ) -> Result<()> {
        // Create render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Scene Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.background_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        drop(render_pass);

        // Traverse and render scene graph
        self.root.read().render_recursive(&self.camera, device, queue, encoder)?;

        Ok(())
    }

    /// Get all visible nodes (frustum culling)
    pub fn get_visible_nodes(&self) -> Vec<Arc<RwLock<SceneNode>>> {
        let mut visible = Vec::new();
        self.root.read().collect_visible(&self.camera, &mut visible);
        visible
    }

    /// Query nodes by type
    pub fn query_nodes_by_type(&self, node_type: NodeType) -> Vec<Arc<RwLock<SceneNode>>> {
        let mut results = Vec::new();
        self.root.read().collect_by_type(node_type, &mut results);
        results
    }

    /// Get scene statistics
    pub fn stats(&self) -> SceneStats {
        SceneStats {
            total_nodes: self.nodes.read().len(),
            visible_nodes: self.get_visible_nodes().len(),
            bounds: self.bounds,
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    /// Minimum corner
    pub min: Vec3,
    /// Maximum corner
    pub max: Vec3,
}

impl BoundingBox {
    /// Create a new bounding box
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an infinite bounding box
    pub fn infinite() -> Self {
        Self {
            min: Vec3::splat(f32::NEG_INFINITY),
            max: Vec3::splat(f32::INFINITY),
        }
    }

    /// Create an empty (inverted) bounding box
    pub fn empty() -> Self {
        Self {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }

    /// Get the center of the bounding box
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the size of the bounding box
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Expand the bounding box to include a point
    pub fn expand(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    /// Merge with another bounding box
    pub fn merge(&mut self, other: &BoundingBox) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    /// Check if a point is inside the bounding box
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    /// Check if this bounding box intersects another
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    /// Transform the bounding box by a matrix
    pub fn transform(&self, mat: &Mat4) -> Self {
        let corners = [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ];

        let mut result = BoundingBox::empty();
        for corner in &corners {
            let transformed = mat.transform_point3(*corner);
            result.expand(transformed);
        }

        result
    }
}

/// Scene statistics
#[derive(Debug, Clone)]
pub struct SceneStats {
    /// Total number of nodes in scene
    pub total_nodes: usize,
    /// Number of visible nodes
    pub visible_nodes: usize,
    /// Scene bounds
    pub bounds: Option<BoundingBox>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_creation() {
        let scene = Scene::new();
        let stats = scene.stats();
        assert_eq!(stats.total_nodes, 1); // Root node
    }

    #[test]
    fn test_bounding_box() {
        let mut bbox = BoundingBox::new(
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
        );

        assert_eq!(bbox.center(), Vec3::ZERO);
        assert_eq!(bbox.size(), Vec3::splat(2.0));
        assert!(bbox.contains(Vec3::ZERO));
        assert!(!bbox.contains(Vec3::new(2.0, 0.0, 0.0)));

        bbox.expand(Vec3::new(2.0, 0.0, 0.0));
        assert_eq!(bbox.max.x, 2.0);
    }
}
