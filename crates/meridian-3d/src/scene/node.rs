//! Scene node hierarchy

use super::{Transform, BoundingBox};
use crate::{Camera, Error, Result};
use glam::Mat4;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;
use wgpu::{CommandEncoder, Device, Queue};

/// Unique identifier for scene nodes
pub type NodeId = Uuid;

/// Scene node in the hierarchy
pub struct SceneNode {
    /// Unique identifier
    id: NodeId,

    /// Node name
    name: String,

    /// Local transform
    transform: Transform,

    /// World transform (computed from parent chain)
    world_transform: Mat4,

    /// Parent node
    parent: Option<Arc<RwLock<SceneNode>>>,

    /// Child nodes
    children: Vec<Arc<RwLock<SceneNode>>>,

    /// Node type and data
    node_type: NodeType,

    /// Local bounding box
    bounds: Option<BoundingBox>,

    /// Visibility flag
    visible: bool,

    /// User data (arbitrary key-value storage)
    user_data: std::collections::HashMap<String, String>,
}

/// Type of scene node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// Empty group node
    Group,
    /// Terrain mesh
    Terrain,
    /// Building model
    Building,
    /// Generic 3D model
    Model,
    /// Light source
    Light,
    /// Camera
    Camera,
    /// Billboard/sprite
    Billboard,
    /// Particle system
    Particles,
    /// Custom node type
    Custom,
}

impl SceneNode {
    /// Create a new scene node
    pub fn new(name: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            transform: Transform::identity(),
            world_transform: Mat4::IDENTITY,
            parent: None,
            children: Vec::new(),
            node_type,
            bounds: None,
            visible: true,
            user_data: std::collections::HashMap::new(),
        }
    }

    /// Create the root node
    pub fn root() -> Self {
        Self::new("Root", NodeType::Group)
    }

    /// Get the node ID
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Get the node name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the node name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    /// Get the node type
    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    /// Get the local transform
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    /// Get mutable local transform
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    /// Set the local transform
    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    /// Get the world transform
    pub fn world_transform(&self) -> Mat4 {
        self.world_transform
    }

    /// Get bounding box
    pub fn bounds(&self) -> Option<BoundingBox> {
        self.bounds
    }

    /// Set bounding box
    pub fn set_bounds(&mut self, bounds: BoundingBox) {
        self.bounds = Some(bounds);
    }

    /// Get visibility
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Set visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Add a child node
    pub fn add_child(&mut self, child: Arc<RwLock<SceneNode>>) {
        self.children.push(child);
    }

    /// Remove a child node by ID
    pub fn remove_child(&mut self, id: NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|c| c.read().id == id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get children
    pub fn children(&self) -> &[Arc<RwLock<SceneNode>>] {
        &self.children
    }

    /// Update node and propagate transforms
    pub fn update(&mut self, delta_time: f32) {
        // Update local transform (for animations, etc.)
        // This is where we'd update any animations

        // Compute world transform
        self.world_transform = if let Some(ref parent) = self.parent {
            parent.read().world_transform * self.transform.matrix()
        } else {
            self.transform.matrix()
        };

        // Update children
        for child in &self.children {
            child.write().update(delta_time);
        }
    }

    /// Render this node and its children recursively
    pub fn render_recursive(
        &self,
        camera: &Camera,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
    ) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        // Frustum culling
        if let Some(bounds) = self.bounds {
            let world_bounds = bounds.transform(&self.world_transform);
            if !self.is_visible_in_frustum(camera, &world_bounds) {
                return Ok(());
            }
        }

        // Render this node based on type
        match self.node_type {
            NodeType::Terrain => {
                // Terrain rendering handled by TerrainRenderer
            }
            NodeType::Building => {
                // Building rendering handled by BuildingRenderer
            }
            NodeType::Model => {
                // Model rendering
            }
            _ => {
                // Other types
            }
        }

        // Render children
        for child in &self.children {
            child.read().render_recursive(camera, device, queue, encoder)?;
        }

        Ok(())
    }

    /// Check if node is visible in camera frustum
    fn is_visible_in_frustum(&self, camera: &Camera, world_bounds: &BoundingBox) -> bool {
        // Simplified frustum culling
        // TODO: Implement proper frustum planes checking
        true
    }

    /// Collect visible nodes recursively
    pub fn collect_visible(&self, camera: &Camera, result: &mut Vec<Arc<RwLock<SceneNode>>>) {
        if !self.visible {
            return;
        }

        // Check frustum culling
        if let Some(bounds) = self.bounds {
            let world_bounds = bounds.transform(&self.world_transform);
            if !self.is_visible_in_frustum(camera, &world_bounds) {
                return;
            }
        }

        // Add self (we need to clone the Arc, but we don't have access to it here)
        // This method should be called from Scene which has the Arc

        // Recurse to children
        for child in &self.children {
            child.read().collect_visible(camera, result);
        }
    }

    /// Collect nodes by type recursively
    pub fn collect_by_type(
        &self,
        node_type: NodeType,
        result: &mut Vec<Arc<RwLock<SceneNode>>>,
    ) {
        // Similar issue as collect_visible
        // This should be refactored

        for child in &self.children {
            child.read().collect_by_type(node_type, result);
        }
    }

    /// Set user data
    pub fn set_user_data(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.user_data.insert(key.into(), value.into());
    }

    /// Get user data
    pub fn get_user_data(&self, key: &str) -> Option<&str> {
        self.user_data.get(key).map(|s| s.as_str())
    }

    /// Find child by name (recursive)
    pub fn find_by_name(&self, name: &str) -> Option<Arc<RwLock<SceneNode>>> {
        if self.name == name {
            // Can't return self here without Arc
            return None;
        }

        for child in &self.children {
            if child.read().name == name {
                return Some(child.clone());
            }

            if let Some(found) = child.read().find_by_name(name) {
                return Some(found);
            }
        }

        None
    }

    /// Get node depth in hierarchy
    pub fn depth(&self) -> usize {
        if let Some(ref parent) = self.parent {
            parent.read().depth() + 1
        } else {
            0
        }
    }

    /// Get total number of descendants
    pub fn descendant_count(&self) -> usize {
        let mut count = self.children.len();
        for child in &self.children {
            count += child.read().descendant_count();
        }
        count
    }
}

/// Builder for creating scene nodes
pub struct NodeBuilder {
    name: String,
    node_type: NodeType,
    transform: Transform,
    visible: bool,
    bounds: Option<BoundingBox>,
}

impl NodeBuilder {
    /// Create a new node builder
    pub fn new(name: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            name: name.into(),
            node_type,
            transform: Transform::identity(),
            visible: true,
            bounds: None,
        }
    }

    /// Set the transform
    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set bounds
    pub fn bounds(mut self, bounds: BoundingBox) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Build the scene node
    pub fn build(self) -> SceneNode {
        let mut node = SceneNode::new(self.name, self.node_type);
        node.set_transform(self.transform);
        node.set_visible(self.visible);
        if let Some(bounds) = self.bounds {
            node.set_bounds(bounds);
        }
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = SceneNode::new("Test Node", NodeType::Group);
        assert_eq!(node.name(), "Test Node");
        assert_eq!(node.node_type(), NodeType::Group);
        assert!(node.is_visible());
    }

    #[test]
    fn test_node_builder() {
        let node = NodeBuilder::new("Test", NodeType::Building)
            .visible(false)
            .build();

        assert_eq!(node.name(), "Test");
        assert!(!node.is_visible());
    }

    #[test]
    fn test_node_hierarchy() {
        let mut parent = SceneNode::new("Parent", NodeType::Group);
        let child = Arc::new(RwLock::new(SceneNode::new("Child", NodeType::Model)));

        parent.add_child(child.clone());
        assert_eq!(parent.children().len(), 1);
        assert_eq!(parent.descendant_count(), 1);
    }
}
