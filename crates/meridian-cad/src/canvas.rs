//! CAD Canvas with multi-layer support, viewport management, and coordinate transforms
//!
//! The canvas is the primary drawing surface for the CAD engine. It manages multiple
//! layers, viewport transformations, and world-to-screen coordinate conversions.

use nalgebra::{Matrix3, Point2, Vector2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::primitives::{Arc, Bezier, Color, Ellipse, Line, Point, Polygon, Spline};
use crate::{CadError, CadResult};

/// Main CAD canvas containing layers and viewport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub id: Uuid,
    pub name: String,
    pub layers: HashMap<Uuid, Layer>,
    pub layer_order: Vec<Uuid>, // Z-order (bottom to top)
    pub active_layer: Option<Uuid>,
    pub viewport: Viewport,
    pub metadata: CanvasMetadata,
}

impl Canvas {
    /// Create a new canvas
    pub fn new(name: impl Into<String>) -> Self {
        let mut canvas = Self {
            id: Uuid::new_v4(),
            name: name.into(),
            layers: HashMap::new(),
            layer_order: Vec::new(),
            active_layer: None,
            viewport: Viewport::default(),
            metadata: CanvasMetadata::default(),
        };

        // Add default layer
        let layer = Layer::new("Layer 1", LayerStyle::default());
        let layer_id = layer.id;
        canvas.layers.insert(layer_id, layer);
        canvas.layer_order.push(layer_id);
        canvas.active_layer = Some(layer_id);

        canvas
    }

    /// Add a new layer
    pub fn add_layer(&mut self, name: impl Into<String>, style: LayerStyle) -> Uuid {
        let layer = Layer::new(name, style);
        let id = layer.id;
        self.layers.insert(id, layer);
        self.layer_order.push(id);
        id
    }

    /// Remove a layer
    pub fn remove_layer(&mut self, id: Uuid) -> CadResult<()> {
        if self.layers.len() <= 1 {
            return Err(CadError::LayerNotFound(
                "Cannot remove the last layer".into(),
            ));
        }

        self.layers
            .remove(&id)
            .ok_or_else(|| CadError::LayerNotFound(format!("Layer {} not found", id)))?;

        self.layer_order.retain(|&layer_id| layer_id != id);

        if self.active_layer == Some(id) {
            self.active_layer = self.layer_order.first().copied();
        }

        Ok(())
    }

    /// Get a layer by ID
    pub fn get_layer(&self, id: Uuid) -> CadResult<&Layer> {
        self.layers
            .get(&id)
            .ok_or_else(|| CadError::LayerNotFound(format!("Layer {} not found", id)))
    }

    /// Get a mutable layer by ID
    pub fn get_layer_mut(&mut self, id: Uuid) -> CadResult<&mut Layer> {
        self.layers
            .get_mut(&id)
            .ok_or_else(|| CadError::LayerNotFound(format!("Layer {} not found", id)))
    }

    /// Get the active layer
    pub fn active_layer(&self) -> CadResult<&Layer> {
        let id = self
            .active_layer
            .ok_or_else(|| CadError::LayerNotFound("No active layer".into()))?;
        self.get_layer(id)
    }

    /// Get the active layer (mutable)
    pub fn active_layer_mut(&mut self) -> CadResult<&mut Layer> {
        let id = self
            .active_layer
            .ok_or_else(|| CadError::LayerNotFound("No active layer".into()))?;
        self.get_layer_mut(id)
    }

    /// Set active layer
    pub fn set_active_layer(&mut self, id: Uuid) -> CadResult<()> {
        if !self.layers.contains_key(&id) {
            return Err(CadError::LayerNotFound(format!("Layer {} not found", id)));
        }
        self.active_layer = Some(id);
        Ok(())
    }

    /// Move layer in z-order
    pub fn move_layer(&mut self, id: Uuid, new_index: usize) -> CadResult<()> {
        if !self.layers.contains_key(&id) {
            return Err(CadError::LayerNotFound(format!("Layer {} not found", id)));
        }

        self.layer_order.retain(|&layer_id| layer_id != id);
        let index = new_index.min(self.layer_order.len());
        self.layer_order.insert(index, id);

        Ok(())
    }

    /// Get all entities across all layers
    pub fn all_entities(&self) -> Vec<&Entity> {
        let mut entities = Vec::new();
        for layer_id in &self.layer_order {
            if let Some(layer) = self.layers.get(layer_id) {
                if layer.visible {
                    entities.extend(&layer.entities);
                }
            }
        }
        entities
    }

    /// Get bounding box of all visible entities
    pub fn bounds(&self) -> Option<(Point, Point)> {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        let mut has_entities = false;

        for entity in self.all_entities() {
            has_entities = true;
            let (min, max) = entity.bounds();
            min_x = min_x.min(min.x);
            min_y = min_y.min(min.y);
            max_x = max_x.max(max.x);
            max_y = max_y.max(max.y);
        }

        if has_entities {
            Some((Point::new(min_x, min_y), Point::new(max_x, max_y)))
        } else {
            None
        }
    }

    /// Zoom to fit all entities
    pub fn zoom_to_fit(&mut self) {
        if let Some((min, max)) = self.bounds() {
            self.viewport.zoom_to_rect(min, max);
        }
    }

    /// Clear all layers
    pub fn clear(&mut self) {
        for layer in self.layers.values_mut() {
            layer.entities.clear();
        }
    }
}

/// A drawing layer in the canvas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: Uuid,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub style: LayerStyle,
    pub entities: Vec<Entity>,
}

impl Layer {
    /// Create a new layer
    pub fn new(name: impl Into<String>, style: LayerStyle) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            visible: true,
            locked: false,
            style,
            entities: Vec::new(),
        }
    }

    /// Add an entity to the layer
    pub fn add_entity(&mut self, entity: Entity) -> CadResult<Uuid> {
        if self.locked {
            return Err(CadError::LayerNotFound("Layer is locked".into()));
        }
        let id = entity.id();
        self.entities.push(entity);
        Ok(id)
    }

    /// Remove an entity from the layer
    pub fn remove_entity(&mut self, id: Uuid) -> CadResult<()> {
        if self.locked {
            return Err(CadError::LayerNotFound("Layer is locked".into()));
        }

        let index = self
            .entities
            .iter()
            .position(|e| e.id() == id)
            .ok_or_else(|| CadError::InvalidGeometry(format!("Entity {} not found", id)))?;

        self.entities.remove(index);
        Ok(())
    }

    /// Get entity by ID
    pub fn get_entity(&self, id: Uuid) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id() == id)
    }

    /// Get entity by ID (mutable)
    pub fn get_entity_mut(&mut self, id: Uuid) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id() == id)
    }

    /// Find entities at a point (within tolerance)
    pub fn entities_at_point(&self, point: &Point, tolerance: f64) -> Vec<&Entity> {
        self.entities
            .iter()
            .filter(|e| e.contains_point(point, tolerance))
            .collect()
    }

    /// Find entities within a rectangular region
    pub fn entities_in_rect(&self, min: &Point, max: &Point) -> Vec<&Entity> {
        self.entities
            .iter()
            .filter(|e| {
                let (entity_min, entity_max) = e.bounds();
                entity_min.x <= max.x
                    && entity_max.x >= min.x
                    && entity_min.y <= max.y
                    && entity_max.y >= min.y
            })
            .collect()
    }
}

/// Visual style for a layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStyle {
    pub color: Color,
    pub line_width: f64,
    pub opacity: f64,
}

impl Default for LayerStyle {
    fn default() -> Self {
        Self {
            color: Color::black(),
            line_width: 1.0,
            opacity: 1.0,
        }
    }
}

/// CAD entity (wrapper for all primitive types)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Entity {
    Line(Line),
    Arc(Arc),
    Bezier(Bezier),
    Spline(Spline),
    Polygon(Polygon),
    Ellipse(Ellipse),
    Text(TextEntity),
    Dimension(DimensionEntity),
}

impl Entity {
    /// Get entity ID
    pub fn id(&self) -> Uuid {
        match self {
            Entity::Line(l) => l.id,
            Entity::Arc(a) => a.id,
            Entity::Bezier(b) => b.id,
            Entity::Spline(s) => s.id,
            Entity::Polygon(p) => p.id,
            Entity::Ellipse(e) => e.id,
            Entity::Text(t) => t.id,
            Entity::Dimension(d) => d.id,
        }
    }

    /// Get entity bounding box
    pub fn bounds(&self) -> (Point, Point) {
        match self {
            Entity::Line(l) => {
                let min_x = l.start.x.min(l.end.x);
                let min_y = l.start.y.min(l.end.y);
                let max_x = l.start.x.max(l.end.x);
                let max_y = l.start.y.max(l.end.y);
                (Point::new(min_x, min_y), Point::new(max_x, max_y))
            }
            Entity::Arc(a) => {
                let r = a.radius;
                (
                    Point::new(a.center.x - r, a.center.y - r),
                    Point::new(a.center.x + r, a.center.y + r),
                )
            }
            Entity::Bezier(b) => {
                let mut min_x = b.p0.x.min(b.p1.x).min(b.p2.x).min(b.p3.x);
                let mut min_y = b.p0.y.min(b.p1.y).min(b.p2.y).min(b.p3.y);
                let mut max_x = b.p0.x.max(b.p1.x).max(b.p2.x).max(b.p3.x);
                let mut max_y = b.p0.y.max(b.p1.y).max(b.p2.y).max(b.p3.y);

                // Sample curve for tighter bounds
                for i in 0..=10 {
                    let t = i as f64 / 10.0;
                    let p = b.point_at(t);
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }

                (Point::new(min_x, min_y), Point::new(max_x, max_y))
            }
            Entity::Spline(s) => s.bounds(),
            Entity::Polygon(p) => p.bounds(),
            Entity::Ellipse(e) => e.bounds(),
            Entity::Text(t) => (t.position, t.position), // Simplified
            Entity::Dimension(d) => (d.start, d.end),    // Simplified
        }
    }

    /// Check if entity contains a point (within tolerance)
    pub fn contains_point(&self, point: &Point, tolerance: f64) -> bool {
        match self {
            Entity::Line(l) => l.contains_point(point, tolerance),
            Entity::Arc(a) => a.contains_point(point, tolerance),
            Entity::Polygon(p) => p.contains_point(point),
            Entity::Ellipse(e) => e.contains_point(point),
            _ => {
                // Simplified: check if point is within bounding box + tolerance
                let (min, max) = self.bounds();
                point.x >= min.x - tolerance
                    && point.x <= max.x + tolerance
                    && point.y >= min.y - tolerance
                    && point.y <= max.y + tolerance
            }
        }
    }

    /// Transform entity by matrix
    pub fn transform(&mut self, matrix: &Matrix3<f64>) {
        match self {
            Entity::Line(l) => {
                l.start = transform_point(&l.start, matrix);
                l.end = transform_point(&l.end, matrix);
            }
            Entity::Arc(a) => {
                a.center = transform_point(&a.center, matrix);
                // Note: This is simplified - proper arc transform requires decomposition
            }
            Entity::Bezier(b) => {
                b.p0 = transform_point(&b.p0, matrix);
                b.p1 = transform_point(&b.p1, matrix);
                b.p2 = transform_point(&b.p2, matrix);
                b.p3 = transform_point(&b.p3, matrix);
            }
            Entity::Spline(s) => {
                for point in &mut s.control_points {
                    *point = transform_point(point, matrix);
                }
            }
            Entity::Polygon(p) => {
                for vertex in &mut p.vertices {
                    *vertex = transform_point(vertex, matrix);
                }
            }
            Entity::Ellipse(e) => {
                e.center = transform_point(&e.center, matrix);
                // Note: This is simplified - proper ellipse transform requires decomposition
            }
            Entity::Text(t) => {
                t.position = transform_point(&t.position, matrix);
            }
            Entity::Dimension(d) => {
                d.start = transform_point(&d.start, matrix);
                d.end = transform_point(&d.end, matrix);
            }
        }
    }
}

/// Text entity for annotations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextEntity {
    pub id: Uuid,
    pub position: Point,
    pub text: String,
    pub font_size: f64,
    pub font_family: String,
    pub color: Color,
    pub rotation: f64,
    pub alignment: TextAlignment,
}

impl TextEntity {
    pub fn new(position: Point, text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            position,
            text: text.into(),
            font_size: 12.0,
            font_family: "Arial".into(),
            color: Color::black(),
            rotation: 0.0,
            alignment: TextAlignment::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

/// Dimension entity for measurements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionEntity {
    pub id: Uuid,
    pub start: Point,
    pub end: Point,
    pub offset: f64,
    pub text_override: Option<String>,
    pub style: DimensionStyle,
}

impl DimensionEntity {
    pub fn new(start: Point, end: Point, offset: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            start,
            end,
            offset,
            text_override: None,
            style: DimensionStyle::default(),
        }
    }

    pub fn length(&self) -> f64 {
        self.start.distance(&self.end)
    }

    pub fn text(&self) -> String {
        self.text_override
            .clone()
            .unwrap_or_else(|| format!("{:.2}", self.length()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionStyle {
    pub arrow_size: f64,
    pub text_height: f64,
    pub color: Color,
}

impl Default for DimensionStyle {
    fn default() -> Self {
        Self {
            arrow_size: 5.0,
            text_height: 10.0,
            color: Color::black(),
        }
    }
}

/// Viewport for world-to-screen coordinate transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub center: Point,     // World space center
    pub zoom: f64,         // Zoom level (1.0 = 100%)
    pub rotation: f64,     // Rotation in radians
    pub width: f64,        // Viewport width in pixels
    pub height: f64,       // Viewport height in pixels
    pub min_zoom: f64,     // Minimum zoom level
    pub max_zoom: f64,     // Maximum zoom level
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            center: Point::origin(),
            zoom: 1.0,
            rotation: 0.0,
            width: 1920.0,
            height: 1080.0,
            min_zoom: 0.01,
            max_zoom: 100.0,
        }
    }
}

impl Viewport {
    /// Create a new viewport
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, point: &Point) -> Point2<f64> {
        let matrix = self.transform_matrix();
        let p = Point2::new(point.x, point.y);
        matrix.transform_point(&p)
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen: &Point2<f64>) -> Point {
        let matrix = self.inverse_transform_matrix();
        let p = matrix.transform_point(screen);
        Point::new(p.x, p.y)
    }

    /// Get world-to-screen transformation matrix
    pub fn transform_matrix(&self) -> Matrix3<f64> {
        // Translate to origin
        let translate_to_origin =
            Matrix3::new_translation(&Vector2::new(-self.center.x, -self.center.y));

        // Scale (zoom)
        let scale = Matrix3::new_nonuniform_scaling(&Vector2::new(self.zoom, self.zoom));

        // Rotate
        let rotate = Matrix3::new_rotation(self.rotation);

        // Translate to screen center
        let translate_to_screen =
            Matrix3::new_translation(&Vector2::new(self.width / 2.0, self.height / 2.0));

        translate_to_screen * rotate * scale * translate_to_origin
    }

    /// Get screen-to-world transformation matrix
    pub fn inverse_transform_matrix(&self) -> Matrix3<f64> {
        self.transform_matrix()
            .try_inverse()
            .unwrap_or_else(Matrix3::identity)
    }

    /// Pan viewport by screen delta
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let delta_world = Vector2::new(dx, dy) / self.zoom;
        self.center.x -= delta_world.x;
        self.center.y -= delta_world.y;
    }

    /// Zoom viewport by factor around a screen point
    pub fn zoom_at(&mut self, factor: f64, screen_x: f64, screen_y: f64) {
        // Convert screen point to world before zoom
        let screen_point = Point2::new(screen_x, screen_y);
        let world_point_before = self.screen_to_world(&screen_point);

        // Apply zoom
        self.zoom *= factor;
        self.zoom = self.zoom.clamp(self.min_zoom, self.max_zoom);

        // Convert same screen point to world after zoom
        let world_point_after = self.screen_to_world(&screen_point);

        // Adjust center to keep world point under cursor
        self.center.x += world_point_before.x - world_point_after.x;
        self.center.y += world_point_before.y - world_point_after.y;
    }

    /// Zoom to fit a rectangular region
    pub fn zoom_to_rect(&mut self, min: Point, max: Point) {
        // Calculate center
        self.center = Point::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0);

        // Calculate zoom to fit
        let width = max.x - min.x;
        let height = max.y - min.y;

        if width > 0.0 && height > 0.0 {
            let zoom_x = self.width / width;
            let zoom_y = self.height / height;
            self.zoom = zoom_x.min(zoom_y) * 0.9; // 90% to add padding
            self.zoom = self.zoom.clamp(self.min_zoom, self.max_zoom);
        }
    }

    /// Get visible world bounds
    pub fn visible_bounds(&self) -> (Point, Point) {
        let top_left = self.screen_to_world(&Point2::new(0.0, 0.0));
        let bottom_right = self.screen_to_world(&Point2::new(self.width, self.height));

        (
            Point::new(top_left.x, top_left.y),
            Point::new(bottom_right.x, bottom_right.y),
        )
    }

    /// Reset viewport to default
    pub fn reset(&mut self) {
        self.center = Point::origin();
        self.zoom = 1.0;
        self.rotation = 0.0;
    }
}

/// Canvas metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub author: String,
    pub description: String,
    pub units: Units,
    pub grid_spacing: f64,
    pub snap_enabled: bool,
}

impl Default for CanvasMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            modified_at: now,
            author: String::new(),
            description: String::new(),
            units: Units::Millimeters,
            grid_spacing: 10.0,
            snap_enabled: true,
        }
    }
}

/// Drawing units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    Millimeters,
    Centimeters,
    Meters,
    Inches,
    Feet,
    Pixels,
}

impl Units {
    /// Get conversion factor to millimeters
    pub fn to_mm(&self) -> f64 {
        match self {
            Units::Millimeters => 1.0,
            Units::Centimeters => 10.0,
            Units::Meters => 1000.0,
            Units::Inches => 25.4,
            Units::Feet => 304.8,
            Units::Pixels => 0.264583, // Assuming 96 DPI
        }
    }
}

/// Helper function to transform a point by a matrix
fn transform_point(point: &Point, matrix: &Matrix3<f64>) -> Point {
    let p = Point2::new(point.x, point.y);
    let transformed = matrix.transform_point(&p);
    Point::new_3d(transformed.x, transformed.y, point.z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_creation() {
        let canvas = Canvas::new("Test Canvas");
        assert_eq!(canvas.name, "Test Canvas");
        assert_eq!(canvas.layers.len(), 1);
        assert!(canvas.active_layer.is_some());
    }

    #[test]
    fn test_layer_management() {
        let mut canvas = Canvas::new("Test");
        let layer_id = canvas.add_layer("Layer 2", LayerStyle::default());
        assert_eq!(canvas.layers.len(), 2);
        canvas.set_active_layer(layer_id).unwrap();
        assert_eq!(canvas.active_layer, Some(layer_id));
    }

    #[test]
    fn test_viewport_world_to_screen() {
        let viewport = Viewport::new(1920.0, 1080.0);
        let world_point = Point::new(100.0, 100.0);
        let screen_point = viewport.world_to_screen(&world_point);
        let back_to_world = viewport.screen_to_world(&screen_point);
        assert!((back_to_world.x - world_point.x).abs() < 0.001);
        assert!((back_to_world.y - world_point.y).abs() < 0.001);
    }
}
