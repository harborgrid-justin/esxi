//! Marker and icon rendering for point-of-interest visualization.

use super::{Layer, LayerProperties, LayerType};
use crate::camera::Camera;
use crate::error::Result;
use crate::renderer::{buffer::BufferHandle, texture::TextureHandle, InstanceData, Renderer};
use glam::{Mat4, Vec2, Vec3};
use std::collections::HashMap;

/// Marker icon definition.
#[derive(Debug, Clone)]
pub struct MarkerIcon {
    /// Icon name/identifier.
    pub name: String,
    /// Texture handle for the icon.
    pub texture: TextureHandle,
    /// Icon size in pixels.
    pub size: Vec2,
    /// Icon anchor point (0.0-1.0, where 0.5,0.5 is center).
    pub anchor: Vec2,
}

impl MarkerIcon {
    /// Create a new marker icon.
    pub fn new(name: impl Into<String>, texture: TextureHandle, size: Vec2) -> Self {
        Self {
            name: name.into(),
            texture,
            size,
            anchor: Vec2::new(0.5, 0.5),
        }
    }

    /// Set the anchor point.
    pub fn with_anchor(mut self, anchor: Vec2) -> Self {
        self.anchor = anchor;
        self
    }
}

/// A marker instance on the map.
#[derive(Debug, Clone)]
pub struct Marker {
    /// Marker ID.
    pub id: u64,
    /// Position in world coordinates.
    pub position: Vec2,
    /// Icon name to use.
    pub icon_name: String,
    /// Scale factor (1.0 = original size).
    pub scale: f32,
    /// Rotation in degrees.
    pub rotation: f32,
    /// Color tint (RGBA, white = no tint).
    pub color: [f32; 4],
    /// Z-order offset for this marker.
    pub z_offset: f32,
    /// Whether the marker is interactive (clickable).
    pub interactive: bool,
    /// Custom data attached to the marker.
    pub data: serde_json::Value,
}

impl Marker {
    /// Create a new marker.
    pub fn new(id: u64, position: Vec2, icon_name: impl Into<String>) -> Self {
        Self {
            id,
            position,
            icon_name: icon_name.into(),
            scale: 1.0,
            rotation: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            z_offset: 0.0,
            interactive: true,
            data: serde_json::Value::Null,
        }
    }

    /// Set the scale.
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Set the rotation.
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set the color tint.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Set the z-offset.
    pub fn with_z_offset(mut self, z_offset: f32) -> Self {
        self.z_offset = z_offset;
        self
    }

    /// Set custom data.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Set interactivity.
    pub fn with_interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }
}

/// Marker layer for rendering icons and symbols.
pub struct MarkerLayer {
    /// Layer properties.
    properties: LayerProperties,
    /// Available marker icons.
    icons: HashMap<String, MarkerIcon>,
    /// Markers in this layer.
    markers: Vec<Marker>,
    /// Instance buffer for GPU instancing.
    instance_buffer: Option<BufferHandle>,
    /// Vertex buffer for marker quad.
    vertex_buffer: Option<BufferHandle>,
    /// Index buffer for marker quad.
    index_buffer: Option<BufferHandle>,
    /// Whether buffers need to be rebuilt.
    dirty: bool,
}

impl MarkerLayer {
    /// Create a new marker layer.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            properties: LayerProperties::new(name, LayerType::Marker),
            icons: HashMap::new(),
            markers: Vec::new(),
            instance_buffer: None,
            vertex_buffer: None,
            index_buffer: None,
            dirty: true,
        }
    }

    /// Register a marker icon.
    pub fn register_icon(&mut self, icon: MarkerIcon) {
        self.icons.insert(icon.name.clone(), icon);
    }

    /// Add a marker to the layer.
    pub fn add_marker(&mut self, marker: Marker) {
        self.markers.push(marker);
        self.dirty = true;
    }

    /// Add multiple markers.
    pub fn add_markers(&mut self, markers: Vec<Marker>) {
        self.markers.extend(markers);
        self.dirty = true;
    }

    /// Remove a marker by ID.
    pub fn remove_marker(&mut self, id: u64) -> Option<Marker> {
        if let Some(index) = self.markers.iter().position(|m| m.id == id) {
            self.dirty = true;
            Some(self.markers.remove(index))
        } else {
            None
        }
    }

    /// Clear all markers.
    pub fn clear_markers(&mut self) {
        self.markers.clear();
        self.dirty = true;
    }

    /// Get the number of markers.
    pub fn marker_count(&self) -> usize {
        self.markers.len()
    }

    /// Get a marker by ID.
    pub fn get_marker(&self, id: u64) -> Option<&Marker> {
        self.markers.iter().find(|m| m.id == id)
    }

    /// Get a mutable marker by ID.
    pub fn get_marker_mut(&mut self, id: u64) -> Option<&mut Marker> {
        self.dirty = true;
        self.markers.iter_mut().find(|m| m.id == id)
    }

    /// Build instance buffer for GPU instancing.
    fn build_instance_buffer(&mut self, renderer: &mut Renderer) -> Result<()> {
        let mut instances = Vec::new();

        for marker in &self.markers {
            if let Some(icon) = self.icons.get(&marker.icon_name) {
                // Calculate transform matrix for this marker instance
                let translation = Mat4::from_translation(Vec3::new(
                    marker.position.x,
                    marker.position.y,
                    marker.z_offset,
                ));

                let rotation = Mat4::from_rotation_z(marker.rotation.to_radians());

                let scale = Mat4::from_scale(Vec3::new(
                    icon.size.x * marker.scale,
                    icon.size.y * marker.scale,
                    1.0,
                ));

                let transform = translation * rotation * scale;

                instances.push(InstanceData {
                    transform: transform.to_cols_array_2d(),
                    color: marker.color,
                });
            }
        }

        // Create instance buffer
        if !instances.is_empty() {
            let instance_data = bytemuck::cast_slice(&instances);
            self.instance_buffer = Some(renderer.buffer_manager_mut().create_instance_buffer(
                instance_data,
                std::mem::size_of::<InstanceData>() as u32,
                true, // Dynamic for updates
            )?);
        }

        self.dirty = false;
        Ok(())
    }

    /// Get markers within a screen-space rectangle (for picking).
    pub fn get_markers_in_rect(&self, min: Vec2, max: Vec2) -> Vec<&Marker> {
        self.markers
            .iter()
            .filter(|m| {
                m.position.x >= min.x
                    && m.position.x <= max.x
                    && m.position.y >= min.y
                    && m.position.y <= max.y
            })
            .collect()
    }

    /// Find the closest marker to a screen position.
    pub fn find_closest_marker(&self, position: Vec2, max_distance: f32) -> Option<&Marker> {
        self.markers
            .iter()
            .filter(|m| m.interactive)
            .map(|m| (m, m.position.distance(position)))
            .filter(|(_, dist)| *dist <= max_distance)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(m, _)| m)
    }
}

impl Layer for MarkerLayer {
    fn properties(&self) -> &LayerProperties {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.properties
    }

    fn update(&mut self, _camera: &Camera, _delta_time: f32) -> Result<()> {
        // Update logic (e.g., animations, dynamic marker updates) goes here
        Ok(())
    }

    fn render(&self, renderer: &mut Renderer, _camera: &Camera) -> Result<()> {
        // Note: In a complete implementation, this would:
        // 1. Bind the marker rendering pipeline
        // 2. Group markers by icon texture
        // 3. Use GPU instancing to render all markers with the same icon in one draw call
        // 4. Render markers sorted by z-offset for proper layering

        if let Some(_instance_buffer) = self.instance_buffer {
            // Group markers by icon
            let mut icon_groups: HashMap<String, Vec<&Marker>> = HashMap::new();
            for marker in &self.markers {
                icon_groups
                    .entry(marker.icon_name.clone())
                    .or_insert_with(Vec::new)
                    .push(marker);
            }

            // Render each icon group with instancing
            for (icon_name, _markers) in icon_groups {
                if let Some(_icon) = self.icons.get(&icon_name) {
                    // Bind icon texture
                    // render_pass.set_bind_group(1, texture_bind_group, &[]);
                    // render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                    // render_pass.draw_indexed(0..6, 0, 0..instance_count);
                }
            }
        }

        Ok(())
    }
}

/// Marker clustering for managing many markers efficiently.
pub struct MarkerCluster {
    /// Center position of the cluster.
    pub center: Vec2,
    /// Number of markers in the cluster.
    pub count: usize,
    /// Markers in this cluster.
    pub markers: Vec<u64>,
}

impl MarkerCluster {
    /// Create a new marker cluster.
    pub fn new(center: Vec2) -> Self {
        Self {
            center,
            count: 0,
            markers: Vec::new(),
        }
    }

    /// Add a marker to the cluster.
    pub fn add_marker(&mut self, id: u64) {
        self.markers.push(id);
        self.count = self.markers.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_creation() {
        let marker = Marker::new(1, Vec2::new(0.0, 0.0), "pin");
        assert_eq!(marker.id, 1);
        assert_eq!(marker.icon_name, "pin");
        assert_eq!(marker.position, Vec2::new(0.0, 0.0));
        assert_eq!(marker.scale, 1.0);
    }

    #[test]
    fn test_marker_builder() {
        let marker = Marker::new(1, Vec2::ZERO, "pin")
            .with_scale(2.0)
            .with_rotation(45.0)
            .with_color([1.0, 0.0, 0.0, 1.0]);

        assert_eq!(marker.scale, 2.0);
        assert_eq!(marker.rotation, 45.0);
        assert_eq!(marker.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_marker_layer() {
        let mut layer = MarkerLayer::new("test_markers");
        assert_eq!(layer.marker_count(), 0);

        layer.add_marker(Marker::new(1, Vec2::new(0.0, 0.0), "pin"));
        layer.add_marker(Marker::new(2, Vec2::new(10.0, 10.0), "pin"));
        assert_eq!(layer.marker_count(), 2);

        layer.remove_marker(1);
        assert_eq!(layer.marker_count(), 1);

        layer.clear_markers();
        assert_eq!(layer.marker_count(), 0);
    }

    #[test]
    fn test_marker_cluster() {
        let mut cluster = MarkerCluster::new(Vec2::ZERO);
        assert_eq!(cluster.count, 0);

        cluster.add_marker(1);
        cluster.add_marker(2);
        assert_eq!(cluster.count, 2);
    }
}
