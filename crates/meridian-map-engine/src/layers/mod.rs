//! Layer rendering system for different types of map content.

pub mod label;
pub mod marker;
pub mod raster;
pub mod vector;

use crate::camera::Camera;
use crate::error::Result;
use crate::renderer::Renderer;
use crate::style::StyleSpec;
use slotmap::{new_key_type, SlotMap};
use std::collections::HashMap;

new_key_type! {
    /// Unique identifier for a layer.
    pub struct LayerId;
}

/// Layer visibility state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Layer is visible.
    Visible,
    /// Layer is hidden.
    Hidden,
}

/// Layer type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerType {
    /// Vector layer (points, lines, polygons).
    Vector,
    /// Raster/tile layer.
    Raster,
    /// Text label layer.
    Label,
    /// Marker/icon layer.
    Marker,
}

/// Common layer properties.
#[derive(Debug, Clone)]
pub struct LayerProperties {
    /// Layer name.
    pub name: String,
    /// Layer type.
    pub layer_type: LayerType,
    /// Visibility state.
    pub visibility: Visibility,
    /// Minimum zoom level for visibility.
    pub min_zoom: f32,
    /// Maximum zoom level for visibility.
    pub max_zoom: f32,
    /// Layer opacity (0.0 to 1.0).
    pub opacity: f32,
    /// Z-index for layer ordering.
    pub z_index: i32,
}

impl LayerProperties {
    /// Create new layer properties.
    pub fn new(name: impl Into<String>, layer_type: LayerType) -> Self {
        Self {
            name: name.into(),
            layer_type,
            visibility: Visibility::Visible,
            min_zoom: 0.0,
            max_zoom: 22.0,
            opacity: 1.0,
            z_index: 0,
        }
    }

    /// Check if the layer should be visible at the given zoom level.
    pub fn is_visible_at_zoom(&self, zoom: f32) -> bool {
        self.visibility == Visibility::Visible && zoom >= self.min_zoom && zoom <= self.max_zoom
    }
}

/// Trait for renderable layers.
pub trait Layer: Send + Sync {
    /// Get layer properties.
    fn properties(&self) -> &LayerProperties;

    /// Get mutable layer properties.
    fn properties_mut(&mut self) -> &mut LayerProperties;

    /// Update the layer (e.g., load new data, update animations).
    fn update(&mut self, camera: &Camera, delta_time: f32) -> Result<()>;

    /// Render the layer.
    fn render(&self, renderer: &mut Renderer, camera: &Camera) -> Result<()>;

    /// Get layer type.
    fn layer_type(&self) -> LayerType {
        self.properties().layer_type
    }

    /// Check if layer is visible.
    fn is_visible(&self) -> bool {
        self.properties().visibility == Visibility::Visible
    }

    /// Set layer visibility.
    fn set_visibility(&mut self, visibility: Visibility) {
        self.properties_mut().visibility = visibility;
    }

    /// Get layer opacity.
    fn opacity(&self) -> f32 {
        self.properties().opacity
    }

    /// Set layer opacity.
    fn set_opacity(&mut self, opacity: f32) {
        self.properties_mut().opacity = opacity.clamp(0.0, 1.0);
    }
}

/// Manages multiple map layers.
pub struct LayerManager {
    /// All layers keyed by LayerId.
    layers: SlotMap<LayerId, Box<dyn Layer>>,
    /// Layer order (indices into layers SlotMap).
    layer_order: Vec<LayerId>,
    /// Layer lookup by name.
    layer_by_name: HashMap<String, LayerId>,
}

impl LayerManager {
    /// Create a new layer manager.
    pub fn new() -> Self {
        Self {
            layers: SlotMap::with_key(),
            layer_order: Vec::new(),
            layer_by_name: HashMap::new(),
        }
    }

    /// Add a layer to the manager.
    pub fn add_layer(&mut self, layer: Box<dyn Layer>) -> LayerId {
        let name = layer.properties().name.clone();
        let id = self.layers.insert(layer);
        self.layer_order.push(id);
        self.layer_by_name.insert(name, id);

        // Sort by z-index
        self.sort_layers();

        id
    }

    /// Remove a layer by ID.
    pub fn remove_layer(&mut self, id: LayerId) -> Option<Box<dyn Layer>> {
        if let Some(layer) = self.layers.remove(id) {
            let name = layer.properties().name.clone();
            self.layer_order.retain(|&lid| lid != id);
            self.layer_by_name.remove(&name);
            Some(layer)
        } else {
            None
        }
    }

    /// Get a layer by ID.
    pub fn get_layer(&self, id: LayerId) -> Option<&dyn Layer> {
        self.layers.get(id).map(|b| b.as_ref())
    }

    /// Get a mutable layer by ID.
    pub fn get_layer_mut(&mut self, id: LayerId) -> Option<&mut dyn Layer> {
        self.layers.get_mut(id).map(|b| b.as_mut())
    }

    /// Get a layer by name.
    pub fn get_layer_by_name(&self, name: &str) -> Option<&dyn Layer> {
        self.layer_by_name
            .get(name)
            .and_then(|&id| self.get_layer(id))
    }

    /// Get a mutable layer by name.
    pub fn get_layer_by_name_mut(&mut self, name: &str) -> Option<&mut dyn Layer> {
        self.layer_by_name
            .get(name)
            .copied()
            .and_then(move |id| self.get_layer_mut(id))
    }

    /// Update all layers.
    pub fn update_all(&mut self, camera: &Camera, delta_time: f32) -> Result<()> {
        for &id in &self.layer_order {
            if let Some(layer) = self.layers.get_mut(id) {
                layer.update(camera, delta_time)?;
            }
        }
        Ok(())
    }

    /// Render all visible layers.
    pub fn render_all(&mut self, renderer: &mut Renderer, camera: &Camera) -> Result<()> {
        for &id in &self.layer_order {
            if let Some(layer) = self.layers.get(id) {
                if layer.is_visible() && layer.properties().is_visible_at_zoom(camera.zoom) {
                    layer.render(renderer, camera)?;
                }
            }
        }
        Ok(())
    }

    /// Sort layers by z-index.
    fn sort_layers(&mut self) {
        self.layer_order.sort_by(|&a, &b| {
            let z_a = self.layers.get(a).map(|l| l.properties().z_index).unwrap_or(0);
            let z_b = self.layers.get(b).map(|l| l.properties().z_index).unwrap_or(0);
            z_a.cmp(&z_b)
        });
    }

    /// Move a layer to a new z-index.
    pub fn set_layer_z_index(&mut self, id: LayerId, z_index: i32) {
        if let Some(layer) = self.layers.get_mut(id) {
            layer.properties_mut().z_index = z_index;
            self.sort_layers();
        }
    }

    /// Get the number of layers.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Get layer statistics.
    pub fn stats(&self) -> LayerStats {
        let mut stats = LayerStats::default();

        for layer in self.layers.values() {
            match layer.layer_type() {
                LayerType::Vector => stats.vector_layers += 1,
                LayerType::Raster => stats.raster_layers += 1,
                LayerType::Label => stats.label_layers += 1,
                LayerType::Marker => stats.marker_layers += 1,
            }

            if layer.is_visible() {
                stats.visible_layers += 1;
            }
        }

        stats
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about layers.
#[derive(Debug, Clone, Default)]
pub struct LayerStats {
    /// Number of vector layers.
    pub vector_layers: usize,
    /// Number of raster layers.
    pub raster_layers: usize,
    /// Number of label layers.
    pub label_layers: usize,
    /// Number of marker layers.
    pub marker_layers: usize,
    /// Number of visible layers.
    pub visible_layers: usize,
}

impl LayerStats {
    /// Get total number of layers.
    pub fn total_layers(&self) -> usize {
        self.vector_layers + self.raster_layers + self.label_layers + self.marker_layers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_properties() {
        let props = LayerProperties::new("test_layer", LayerType::Vector);
        assert_eq!(props.name, "test_layer");
        assert_eq!(props.layer_type, LayerType::Vector);
        assert!(props.is_visible_at_zoom(10.0));
    }

    #[test]
    fn test_visibility_at_zoom() {
        let mut props = LayerProperties::new("test_layer", LayerType::Vector);
        props.min_zoom = 5.0;
        props.max_zoom = 15.0;

        assert!(!props.is_visible_at_zoom(3.0));
        assert!(props.is_visible_at_zoom(10.0));
        assert!(!props.is_visible_at_zoom(20.0));
    }
}
