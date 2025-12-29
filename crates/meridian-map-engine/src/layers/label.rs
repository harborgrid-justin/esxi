//! Text label rendering for map annotations.

use super::{Layer, LayerProperties, LayerType};
use crate::camera::Camera;
use crate::error::Result;
use crate::renderer::Renderer;
use glam::Vec2;

/// Text anchor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAnchor {
    /// Top-left corner.
    TopLeft,
    /// Top-center.
    Top,
    /// Top-right corner.
    TopRight,
    /// Left-center.
    Left,
    /// Center.
    Center,
    /// Right-center.
    Right,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-center.
    Bottom,
    /// Bottom-right corner.
    BottomRight,
}

/// A text label feature.
#[derive(Debug, Clone)]
pub struct TextLabel {
    /// Label ID.
    pub id: u64,
    /// Label text.
    pub text: String,
    /// Position in world coordinates.
    pub position: Vec2,
    /// Font size in pixels.
    pub font_size: f32,
    /// Text color (RGBA).
    pub color: [f32; 4],
    /// Text anchor.
    pub anchor: TextAnchor,
    /// Rotation in degrees.
    pub rotation: f32,
    /// Priority for label placement (higher = more important).
    pub priority: f32,
    /// Maximum width before wrapping.
    pub max_width: Option<f32>,
    /// Halo (outline) color.
    pub halo_color: Option<[f32; 4]>,
    /// Halo width in pixels.
    pub halo_width: f32,
}

impl TextLabel {
    /// Create a new text label.
    pub fn new(id: u64, text: impl Into<String>, position: Vec2) -> Self {
        Self {
            id,
            text: text.into(),
            position,
            font_size: 16.0,
            color: [0.0, 0.0, 0.0, 1.0],
            anchor: TextAnchor::Center,
            rotation: 0.0,
            priority: 0.0,
            max_width: None,
            halo_color: Some([1.0, 1.0, 1.0, 1.0]),
            halo_width: 2.0,
        }
    }

    /// Set font size.
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set text color.
    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    /// Set text anchor.
    pub fn with_anchor(mut self, anchor: TextAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set rotation.
    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: f32) -> Self {
        self.priority = priority;
        self
    }

    /// Set halo color and width.
    pub fn with_halo(mut self, color: [f32; 4], width: f32) -> Self {
        self.halo_color = Some(color);
        self.halo_width = width;
        self
    }
}

/// Label layer for rendering text annotations.
pub struct LabelLayer {
    /// Layer properties.
    properties: LayerProperties,
    /// Labels in this layer.
    labels: Vec<TextLabel>,
    /// Font family name.
    font_family: String,
    /// Whether to perform collision detection.
    collision_detection: bool,
    /// Minimum zoom for label visibility.
    label_min_zoom: f32,
}

impl LabelLayer {
    /// Create a new label layer.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            properties: LayerProperties::new(name, LayerType::Label),
            labels: Vec::new(),
            font_family: "Arial".to_string(),
            collision_detection: true,
            label_min_zoom: 0.0,
        }
    }

    /// Add a label to the layer.
    pub fn add_label(&mut self, label: TextLabel) {
        self.labels.push(label);
    }

    /// Add multiple labels.
    pub fn add_labels(&mut self, labels: Vec<TextLabel>) {
        self.labels.extend(labels);
    }

    /// Clear all labels.
    pub fn clear_labels(&mut self) {
        self.labels.clear();
    }

    /// Get the number of labels.
    pub fn label_count(&self) -> usize {
        self.labels.len()
    }

    /// Set font family.
    pub fn set_font_family(&mut self, family: impl Into<String>) {
        self.font_family = family.into();
    }

    /// Enable or disable collision detection.
    pub fn set_collision_detection(&mut self, enabled: bool) {
        self.collision_detection = enabled;
    }

    /// Perform collision detection to determine which labels to show.
    fn detect_collisions(&self, camera: &Camera) -> Vec<usize> {
        if !self.collision_detection {
            return (0..self.labels.len()).collect();
        }

        let mut visible_indices = Vec::new();
        let mut occupied_regions: Vec<(Vec2, Vec2)> = Vec::new();

        // Sort labels by priority (higher priority first)
        let mut sorted_labels: Vec<(usize, &TextLabel)> =
            self.labels.iter().enumerate().collect();
        sorted_labels.sort_by(|a, b| b.1.priority.partial_cmp(&a.1.priority).unwrap());

        for (index, label) in sorted_labels {
            // Estimate label bounds (simplified)
            let width = label.text.len() as f32 * label.font_size * 0.6;
            let height = label.font_size;

            let bounds_min = Vec2::new(
                label.position.x - width * 0.5,
                label.position.y - height * 0.5,
            );
            let bounds_max = Vec2::new(
                label.position.x + width * 0.5,
                label.position.y + height * 0.5,
            );

            // Check for collisions
            let mut collides = false;
            for (occupied_min, occupied_max) in &occupied_regions {
                if !(bounds_max.x < occupied_min.x
                    || bounds_min.x > occupied_max.x
                    || bounds_max.y < occupied_min.y
                    || bounds_min.y > occupied_max.y)
                {
                    collides = true;
                    break;
                }
            }

            if !collides {
                visible_indices.push(index);
                occupied_regions.push((bounds_min, bounds_max));
            }
        }

        visible_indices
    }
}

impl Layer for LabelLayer {
    fn properties(&self) -> &LayerProperties {
        &self.properties
    }

    fn properties_mut(&mut self) -> &mut LayerProperties {
        &mut self.properties
    }

    fn update(&mut self, camera: &Camera, _delta_time: f32) -> Result<()> {
        // Perform collision detection
        let _visible_labels = self.detect_collisions(camera);

        // In a full implementation, this would update vertex buffers
        // with only the visible labels

        Ok(())
    }

    fn render(&self, renderer: &mut Renderer, camera: &Camera) -> Result<()> {
        // Note: In a complete implementation, this would:
        // 1. Bind the text rendering pipeline
        // 2. For each visible label, render the text using a signed distance field (SDF) texture
        // 3. Render halos first, then the text itself

        let visible_indices = self.detect_collisions(camera);

        for &index in &visible_indices {
            let _label = &self.labels[index];
            // Render label text
            // This would involve:
            // - Converting text to glyph quads
            // - Using SDF texture atlas for font rendering
            // - Applying transformations for rotation and anchor
        }

        Ok(())
    }
}

/// Glyph information for text rendering.
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    /// Character code.
    pub char_code: u32,
    /// UV coordinates in font atlas.
    pub uv_min: Vec2,
    /// UV coordinates in font atlas.
    pub uv_max: Vec2,
    /// Glyph advance width.
    pub advance: f32,
    /// Glyph bearing (offset from baseline).
    pub bearing: Vec2,
    /// Glyph size.
    pub size: Vec2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_label_creation() {
        let label = TextLabel::new(1, "Test Label", Vec2::new(0.0, 0.0));
        assert_eq!(label.text, "Test Label");
        assert_eq!(label.position, Vec2::new(0.0, 0.0));
        assert_eq!(label.font_size, 16.0);
    }

    #[test]
    fn test_label_builder() {
        let label = TextLabel::new(1, "Test", Vec2::ZERO)
            .with_font_size(24.0)
            .with_color([1.0, 0.0, 0.0, 1.0])
            .with_priority(100.0);

        assert_eq!(label.font_size, 24.0);
        assert_eq!(label.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(label.priority, 100.0);
    }

    #[test]
    fn test_label_layer() {
        let mut layer = LabelLayer::new("test_labels");
        assert_eq!(layer.label_count(), 0);

        layer.add_label(TextLabel::new(1, "Label 1", Vec2::ZERO));
        layer.add_label(TextLabel::new(2, "Label 2", Vec2::new(10.0, 10.0)));
        assert_eq!(layer.label_count(), 2);

        layer.clear_labels();
        assert_eq!(layer.label_count(), 0);
    }
}
