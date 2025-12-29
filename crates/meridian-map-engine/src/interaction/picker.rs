//! GPU-accelerated feature picking for identifying map features under the cursor.

use crate::camera::Camera;
use crate::error::Result;
use crate::renderer::Renderer;
use glam::Vec2;
use std::collections::HashMap;

/// Feature pick result.
#[derive(Debug, Clone)]
pub struct PickResult {
    /// Feature ID that was picked.
    pub feature_id: u64,
    /// Layer name.
    pub layer_name: String,
    /// Screen position where feature was picked.
    pub screen_position: Vec2,
    /// World position (if available).
    pub world_position: Option<Vec2>,
    /// Feature properties.
    pub properties: Option<serde_json::Value>,
}

/// GPU-based feature picker using color-coded rendering.
pub struct FeaturePicker {
    /// Picking framebuffer width.
    width: u32,
    /// Picking framebuffer height.
    height: u32,
    /// Feature ID to color mapping.
    feature_colors: HashMap<u64, [u8; 4]>,
    /// Color to feature ID mapping.
    color_to_feature: HashMap<[u8; 4], u64>,
    /// Next color ID for encoding.
    next_color_id: u32,
}

impl FeaturePicker {
    /// Create a new feature picker.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            feature_colors: HashMap::new(),
            color_to_feature: HashMap::new(),
            next_color_id: 1, // Start at 1 (0 is background)
        }
    }

    /// Register a feature for picking.
    pub fn register_feature(&mut self, feature_id: u64) -> [u8; 4] {
        if let Some(&color) = self.feature_colors.get(&feature_id) {
            return color;
        }

        let color = Self::encode_id_to_color(self.next_color_id);
        self.next_color_id += 1;

        self.feature_colors.insert(feature_id, color);
        self.color_to_feature.insert(color, feature_id);

        color
    }

    /// Encode an ID to a unique color.
    fn encode_id_to_color(id: u32) -> [u8; 4] {
        [
            ((id >> 16) & 0xFF) as u8,
            ((id >> 8) & 0xFF) as u8,
            (id & 0xFF) as u8,
            255,
        ]
    }

    /// Decode a color back to an ID.
    fn decode_color_to_id(color: [u8; 4]) -> u32 {
        ((color[0] as u32) << 16) | ((color[1] as u32) << 8) | (color[2] as u32)
    }

    /// Pick features at a screen position.
    pub fn pick_at_position(
        &self,
        renderer: &Renderer,
        camera: &Camera,
        screen_pos: Vec2,
    ) -> Option<PickResult> {
        // Note: In a complete implementation, this would:
        // 1. Render all features to an off-screen buffer using unique colors
        // 2. Read the pixel at screen_pos from the buffer
        // 3. Decode the color to get the feature ID

        // For now, return None as this requires actual GPU rendering
        None
    }

    /// Pick features in a screen rectangle.
    pub fn pick_in_rect(
        &self,
        renderer: &Renderer,
        camera: &Camera,
        rect_min: Vec2,
        rect_max: Vec2,
    ) -> Vec<PickResult> {
        // Note: In a complete implementation, this would:
        // 1. Render all features to an off-screen buffer using unique colors
        // 2. Read all pixels in the rectangle
        // 3. Decode colors to get feature IDs
        // 4. Return unique features

        Vec::new()
    }

    /// Clear registered features.
    pub fn clear(&mut self) {
        self.feature_colors.clear();
        self.color_to_feature.clear();
        self.next_color_id = 1;
    }

    /// Resize the picking framebuffer.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

/// CPU-based spatial index picker for simple geometric queries.
pub struct SpatialPicker {
    /// Features indexed by their bounding boxes.
    features: Vec<SpatialFeature>,
}

impl SpatialPicker {
    /// Create a new spatial picker.
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
        }
    }

    /// Add a feature to the spatial index.
    pub fn add_feature(&mut self, feature: SpatialFeature) {
        self.features.push(feature);
    }

    /// Pick features at a point.
    pub fn pick_at_point(&self, point: Vec2, tolerance: f32) -> Vec<&SpatialFeature> {
        self.features
            .iter()
            .filter(|f| {
                let bounds = f.bounds;
                point.x >= bounds.min.x - tolerance
                    && point.x <= bounds.max.x + tolerance
                    && point.y >= bounds.min.y - tolerance
                    && point.y <= bounds.max.y + tolerance
            })
            .collect()
    }

    /// Pick features in a bounding box.
    pub fn pick_in_bbox(&self, bbox: BoundingBox) -> Vec<&SpatialFeature> {
        self.features
            .iter()
            .filter(|f| bbox.intersects(&f.bounds))
            .collect()
    }

    /// Clear all features.
    pub fn clear(&mut self) {
        self.features.clear();
    }

    /// Get the number of indexed features.
    pub fn feature_count(&self) -> usize {
        self.features.len()
    }
}

impl Default for SpatialPicker {
    fn default() -> Self {
        Self::new()
    }
}

/// Feature with spatial bounds for picking.
#[derive(Debug, Clone)]
pub struct SpatialFeature {
    /// Feature ID.
    pub id: u64,
    /// Layer name.
    pub layer_name: String,
    /// Bounding box.
    pub bounds: BoundingBox,
    /// Feature geometry type.
    pub geometry_type: GeometryType,
    /// Feature properties.
    pub properties: Option<serde_json::Value>,
}

/// Geometry type for spatial features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryType {
    /// Point geometry.
    Point,
    /// Line geometry.
    Line,
    /// Polygon geometry.
    Polygon,
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    /// Minimum point (bottom-left).
    pub min: Vec2,
    /// Maximum point (top-right).
    pub max: Vec2,
}

impl BoundingBox {
    /// Create a new bounding box.
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Create a bounding box from a center point and size.
    pub fn from_center(center: Vec2, size: Vec2) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    /// Check if this bounding box contains a point.
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Check if this bounding box intersects another.
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        !(self.max.x < other.min.x
            || self.min.x > other.max.x
            || self.max.y < other.min.y
            || self.min.y > other.max.y)
    }

    /// Get the center of the bounding box.
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    /// Get the size of the bounding box.
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Expand the bounding box by a margin.
    pub fn expand(&self, margin: f32) -> BoundingBox {
        BoundingBox {
            min: self.min - Vec2::splat(margin),
            max: self.max + Vec2::splat(margin),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_picker_color_encoding() {
        let color = FeaturePicker::encode_id_to_color(123456);
        let id = FeaturePicker::decode_color_to_id(color);
        assert_eq!(id, 123456);
    }

    #[test]
    fn test_feature_picker_registration() {
        let mut picker = FeaturePicker::new(800, 600);

        let color1 = picker.register_feature(1);
        let color2 = picker.register_feature(2);

        assert_ne!(color1, color2);
        assert_eq!(picker.feature_colors.len(), 2);
    }

    #[test]
    fn test_bounding_box_contains() {
        let bbox = BoundingBox::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));

        assert!(bbox.contains(Vec2::new(5.0, 5.0)));
        assert!(!bbox.contains(Vec2::new(15.0, 15.0)));
    }

    #[test]
    fn test_bounding_box_intersects() {
        let bbox1 = BoundingBox::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        let bbox2 = BoundingBox::new(Vec2::new(5.0, 5.0), Vec2::new(15.0, 15.0));
        let bbox3 = BoundingBox::new(Vec2::new(20.0, 20.0), Vec2::new(30.0, 30.0));

        assert!(bbox1.intersects(&bbox2));
        assert!(!bbox1.intersects(&bbox3));
    }

    #[test]
    fn test_spatial_picker() {
        let mut picker = SpatialPicker::new();

        let feature = SpatialFeature {
            id: 1,
            layer_name: "test".to_string(),
            bounds: BoundingBox::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0)),
            geometry_type: GeometryType::Point,
            properties: None,
        };

        picker.add_feature(feature);
        assert_eq!(picker.feature_count(), 1);

        let results = picker.pick_at_point(Vec2::new(5.0, 5.0), 1.0);
        assert_eq!(results.len(), 1);
    }
}
