//! Drawing and editing tools for the CAD engine
//!
//! This module provides professional CAD tools including pen, rectangle, circle,
//! text, dimensions, and measurement tools.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::canvas::{DimensionEntity, Entity, TextEntity};
use crate::primitives::{Arc, Bezier, Ellipse, Line, Point, Polygon};
use crate::{CadError, CadResult};

/// Trait for all CAD tools
pub trait Tool {
    /// Get tool name
    fn name(&self) -> &str;

    /// Get tool description
    fn description(&self) -> &str;

    /// Tool configuration
    fn config(&self) -> &ToolConfig;

    /// Called when tool is activated
    fn activate(&mut self) {}

    /// Called when tool is deactivated
    fn deactivate(&mut self) {}

    /// Handle mouse down event
    fn on_mouse_down(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        Ok(None)
    }

    /// Handle mouse move event
    fn on_mouse_move(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        Ok(None)
    }

    /// Handle mouse up event
    fn on_mouse_up(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        Ok(None)
    }

    /// Handle key press event
    fn on_key_press(&mut self, key: &str) -> CadResult<Option<ToolAction>> {
        Ok(None)
    }

    /// Reset tool state
    fn reset(&mut self);
}

/// Tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub cursor: CursorType,
    pub snap_to_grid: bool,
    pub snap_to_objects: bool,
    pub show_preview: bool,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            cursor: CursorType::Crosshair,
            snap_to_grid: true,
            snap_to_objects: true,
            show_preview: true,
        }
    }
}

/// Cursor types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorType {
    Arrow,
    Crosshair,
    Hand,
    Text,
    Move,
    Resize,
}

/// Actions that tools can produce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolAction {
    CreateEntity(Entity),
    ModifyEntity { id: Uuid, entity: Entity },
    DeleteEntity(Uuid),
    ShowPreview(Entity),
    ClearPreview,
    SetStatus(String),
}

/// Pen tool for freehand drawing
#[derive(Debug, Clone)]
pub struct PenTool {
    config: ToolConfig,
    points: Vec<Point>,
    min_distance: f64,
    smoothing: bool,
}

impl PenTool {
    pub fn new() -> Self {
        Self {
            config: ToolConfig::default(),
            points: Vec::new(),
            min_distance: 2.0,
            smoothing: true,
        }
    }

    pub fn with_smoothing(mut self, smoothing: bool) -> Self {
        self.smoothing = smoothing;
        self
    }

    fn create_bezier_curve(&self) -> CadResult<Entity> {
        if self.points.len() < 2 {
            return Err(CadError::ToolError(
                "Need at least 2 points to create curve".into(),
            ));
        }

        if self.points.len() == 2 {
            // Simple line
            return Ok(Entity::Line(Line::new(self.points[0], self.points[1])));
        }

        // Create smooth Bezier curves through points
        let mut beziers = Vec::new();

        for i in 0..self.points.len() - 1 {
            let p0 = self.points[i];
            let p3 = self.points[i + 1];

            // Calculate control points for smooth curve
            let direction = Point::new(p3.x - p0.x, p3.y - p0.y);
            let p1 = Point::new(p0.x + direction.x / 3.0, p0.y + direction.y / 3.0);
            let p2 = Point::new(p0.x + 2.0 * direction.x / 3.0, p0.y + 2.0 * direction.y / 3.0);

            beziers.push(Bezier::new(p0, p1, p2, p3));
        }

        // For now, return the first bezier
        // In a real implementation, this would return a path or compound entity
        Ok(Entity::Bezier(beziers[0].clone()))
    }
}

impl Default for PenTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for PenTool {
    fn name(&self) -> &str {
        "Pen"
    }

    fn description(&self) -> &str {
        "Draw freehand curves and paths"
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }

    fn on_mouse_down(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        self.points.clear();
        self.points.push(point);
        Ok(Some(ToolAction::SetStatus(
            "Drawing... Release to finish".into(),
        )))
    }

    fn on_mouse_move(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if self.points.is_empty() {
            return Ok(None);
        }

        // Only add point if it's far enough from the last point
        if let Some(last) = self.points.last() {
            if last.distance(&point) >= self.min_distance {
                self.points.push(point);
            }
        }

        Ok(None)
    }

    fn on_mouse_up(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if self.points.is_empty() {
            return Ok(None);
        }

        self.points.push(point);
        let entity = self.create_bezier_curve()?;
        self.points.clear();

        Ok(Some(ToolAction::CreateEntity(entity)))
    }

    fn reset(&mut self) {
        self.points.clear();
    }
}

/// Rectangle tool
#[derive(Debug, Clone)]
pub struct RectangleTool {
    config: ToolConfig,
    start: Option<Point>,
    mode: RectangleMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RectangleMode {
    TwoCorners,
    CenterAndCorner,
}

impl RectangleTool {
    pub fn new() -> Self {
        Self {
            config: ToolConfig::default(),
            start: None,
            mode: RectangleMode::TwoCorners,
        }
    }

    pub fn with_mode(mut self, mode: RectangleMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn draw(&self, start: Point, end: Point) -> CadResult<Polygon> {
        let (min_x, max_x) = if start.x < end.x {
            (start.x, end.x)
        } else {
            (end.x, start.x)
        };

        let (min_y, max_y) = if start.y < end.y {
            (start.y, end.y)
        } else {
            (end.y, start.y)
        };

        Polygon::rectangle(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    fn create_rectangle(&self, start: Point, end: Point) -> CadResult<Entity> {
        match self.mode {
            RectangleMode::TwoCorners => Ok(Entity::Polygon(self.draw(start, end)?)),
            RectangleMode::CenterAndCorner => {
                let width = (end.x - start.x).abs() * 2.0;
                let height = (end.y - start.y).abs() * 2.0;
                let min_x = start.x - width / 2.0;
                let min_y = start.y - height / 2.0;
                Ok(Entity::Polygon(Polygon::rectangle(
                    min_x, min_y, width, height,
                )?))
            }
        }
    }
}

impl Default for RectangleTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for RectangleTool {
    fn name(&self) -> &str {
        "Rectangle"
    }

    fn description(&self) -> &str {
        "Draw rectangles and squares"
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }

    fn on_mouse_down(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        self.start = Some(point);
        Ok(Some(ToolAction::SetStatus("Drag to define rectangle".into())))
    }

    fn on_mouse_move(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if let Some(start) = self.start {
            let preview = self.create_rectangle(start, point)?;
            Ok(Some(ToolAction::ShowPreview(preview)))
        } else {
            Ok(None)
        }
    }

    fn on_mouse_up(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if let Some(start) = self.start.take() {
            let entity = self.create_rectangle(start, point)?;
            Ok(Some(ToolAction::CreateEntity(entity)))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.start = None;
    }
}

/// Circle tool
#[derive(Debug, Clone)]
pub struct CircleTool {
    config: ToolConfig,
    center: Option<Point>,
    mode: CircleMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircleMode {
    CenterAndRadius,
    TwoPoints,
    ThreePoints,
}

impl CircleTool {
    pub fn new() -> Self {
        Self {
            config: ToolConfig::default(),
            center: None,
            mode: CircleMode::CenterAndRadius,
        }
    }

    pub fn with_mode(mut self, mode: CircleMode) -> Self {
        self.mode = mode;
        self
    }

    fn create_circle(&self, center: Point, point: Point) -> CadResult<Entity> {
        let radius = center.distance(&point);
        Ok(Entity::Ellipse(Ellipse::circle(center, radius)?))
    }
}

impl Default for CircleTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for CircleTool {
    fn name(&self) -> &str {
        "Circle"
    }

    fn description(&self) -> &str {
        "Draw circles and arcs"
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }

    fn on_mouse_down(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        self.center = Some(point);
        Ok(Some(ToolAction::SetStatus("Define radius".into())))
    }

    fn on_mouse_move(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if let Some(center) = self.center {
            let preview = self.create_circle(center, point)?;
            Ok(Some(ToolAction::ShowPreview(preview)))
        } else {
            Ok(None)
        }
    }

    fn on_mouse_up(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if let Some(center) = self.center.take() {
            let entity = self.create_circle(center, point)?;
            Ok(Some(ToolAction::CreateEntity(entity)))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.center = None;
    }
}

/// Text tool for adding annotations
#[derive(Debug, Clone)]
pub struct TextTool {
    config: ToolConfig,
    position: Option<Point>,
    text: String,
    font_size: f64,
}

impl TextTool {
    pub fn new() -> Self {
        Self {
            config: ToolConfig {
                cursor: CursorType::Text,
                ..Default::default()
            },
            position: None,
            text: String::new(),
            font_size: 12.0,
        }
    }

    pub fn with_font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }
}

impl Default for TextTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for TextTool {
    fn name(&self) -> &str {
        "Text"
    }

    fn description(&self) -> &str {
        "Add text annotations"
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }

    fn on_mouse_down(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        self.position = Some(point);
        Ok(Some(ToolAction::SetStatus("Type text, press Enter to finish".into())))
    }

    fn on_key_press(&mut self, key: &str) -> CadResult<Option<ToolAction>> {
        if let Some(position) = self.position {
            match key {
                "Enter" => {
                    if !self.text.is_empty() {
                        let mut text_entity = TextEntity::new(position, self.text.clone());
                        text_entity.font_size = self.font_size;
                        self.text.clear();
                        self.position = None;
                        return Ok(Some(ToolAction::CreateEntity(Entity::Text(text_entity))));
                    }
                }
                "Backspace" => {
                    self.text.pop();
                }
                "Escape" => {
                    self.text.clear();
                    self.position = None;
                    return Ok(Some(ToolAction::ClearPreview));
                }
                _ => {
                    if key.len() == 1 {
                        self.text.push_str(key);
                    }
                }
            }
        }
        Ok(None)
    }

    fn reset(&mut self) {
        self.position = None;
        self.text.clear();
    }
}

/// Dimension tool for measurements with annotations
#[derive(Debug, Clone)]
pub struct DimensionTool {
    config: ToolConfig,
    start: Option<Point>,
    dimension_type: DimensionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionType {
    Linear,
    Horizontal,
    Vertical,
    Angular,
    Radial,
    Diameter,
}

impl DimensionTool {
    pub fn new() -> Self {
        Self {
            config: ToolConfig::default(),
            start: None,
            dimension_type: DimensionType::Linear,
        }
    }

    pub fn with_type(mut self, dim_type: DimensionType) -> Self {
        self.dimension_type = dim_type;
        self
    }

    fn create_dimension(&self, start: Point, end: Point) -> CadResult<Entity> {
        let offset = 20.0; // Default offset from measured line
        Ok(Entity::Dimension(DimensionEntity::new(start, end, offset)))
    }
}

impl Default for DimensionTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for DimensionTool {
    fn name(&self) -> &str {
        "Dimension"
    }

    fn description(&self) -> &str {
        "Add measurement dimensions"
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }

    fn on_mouse_down(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        self.start = Some(point);
        Ok(Some(ToolAction::SetStatus("Select second point".into())))
    }

    fn on_mouse_move(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if let Some(start) = self.start {
            let preview = self.create_dimension(start, point)?;
            Ok(Some(ToolAction::ShowPreview(preview)))
        } else {
            Ok(None)
        }
    }

    fn on_mouse_up(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if let Some(start) = self.start.take() {
            let entity = self.create_dimension(start, point)?;
            Ok(Some(ToolAction::CreateEntity(entity)))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.start = None;
    }
}

/// Measure tool for getting distances and angles without creating entities
#[derive(Debug, Clone)]
pub struct MeasureTool {
    config: ToolConfig,
    start: Option<Point>,
    measurements: Vec<Measurement>,
}

impl MeasureTool {
    pub fn new() -> Self {
        Self {
            config: ToolConfig::default(),
            start: None,
            measurements: Vec::new(),
        }
    }

    pub fn get_measurements(&self) -> &[Measurement] {
        &self.measurements
    }

    pub fn clear_measurements(&mut self) {
        self.measurements.clear();
    }
}

impl Default for MeasureTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for MeasureTool {
    fn name(&self) -> &str {
        "Measure"
    }

    fn description(&self) -> &str {
        "Measure distances and angles"
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }

    fn on_mouse_down(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        self.start = Some(point);
        Ok(Some(ToolAction::SetStatus("Select second point to measure".into())))
    }

    fn on_mouse_move(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if let Some(start) = self.start {
            let distance = start.distance(&point);
            let angle = (point.y - start.y).atan2(point.x - start.x).to_degrees();
            let status = format!("Distance: {:.2}, Angle: {:.2}°", distance, angle);
            Ok(Some(ToolAction::SetStatus(status)))
        } else {
            Ok(None)
        }
    }

    fn on_mouse_up(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if let Some(start) = self.start.take() {
            let distance = start.distance(&point);
            let angle = (point.y - start.y).atan2(point.x - start.x).to_degrees();

            let measurement = Measurement {
                start,
                end: point,
                distance,
                angle,
            };

            let status = format!(
                "Measured: {:.2} units at {:.2}°",
                measurement.distance, measurement.angle
            );

            self.measurements.push(measurement);
            Ok(Some(ToolAction::SetStatus(status)))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.start = None;
    }
}

/// Measurement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub start: Point,
    pub end: Point,
    pub distance: f64,
    pub angle: f64, // degrees
}

/// Line tool for drawing straight lines
#[derive(Debug, Clone)]
pub struct LineTool {
    config: ToolConfig,
    start: Option<Point>,
    chain_mode: bool,
}

impl LineTool {
    pub fn new() -> Self {
        Self {
            config: ToolConfig::default(),
            start: None,
            chain_mode: false,
        }
    }

    pub fn with_chain_mode(mut self, chain: bool) -> Self {
        self.chain_mode = chain;
        self
    }
}

impl Default for LineTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for LineTool {
    fn name(&self) -> &str {
        "Line"
    }

    fn description(&self) -> &str {
        "Draw straight lines"
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }

    fn on_mouse_down(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        self.start = Some(point);
        Ok(Some(ToolAction::SetStatus("Select end point".into())))
    }

    fn on_mouse_move(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if let Some(start) = self.start {
            let line = Line::new(start, point);
            Ok(Some(ToolAction::ShowPreview(Entity::Line(line))))
        } else {
            Ok(None)
        }
    }

    fn on_mouse_up(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        if let Some(start) = self.start {
            let line = Line::new(start, point);

            if self.chain_mode {
                // In chain mode, end point becomes start of next line
                self.start = Some(point);
            } else {
                self.start = None;
            }

            Ok(Some(ToolAction::CreateEntity(Entity::Line(line))))
        } else {
            Ok(None)
        }
    }

    fn on_key_press(&mut self, key: &str) -> CadResult<Option<ToolAction>> {
        if key == "Escape" {
            self.start = None;
            Ok(Some(ToolAction::ClearPreview))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.start = None;
    }
}

/// Arc tool for drawing circular arcs
#[derive(Debug, Clone)]
pub struct ArcTool {
    config: ToolConfig,
    state: ArcToolState,
}

#[derive(Debug, Clone)]
enum ArcToolState {
    Initial,
    CenterSelected(Point),
    StartSelected { center: Point, start: Point },
}

impl ArcTool {
    pub fn new() -> Self {
        Self {
            config: ToolConfig::default(),
            state: ArcToolState::Initial,
        }
    }
}

impl Default for ArcTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for ArcTool {
    fn name(&self) -> &str {
        "Arc"
    }

    fn description(&self) -> &str {
        "Draw circular arcs"
    }

    fn config(&self) -> &ToolConfig {
        &self.config
    }

    fn on_mouse_down(&mut self, point: Point) -> CadResult<Option<ToolAction>> {
        match self.state {
            ArcToolState::Initial => {
                self.state = ArcToolState::CenterSelected(point);
                Ok(Some(ToolAction::SetStatus("Select start point".into())))
            }
            ArcToolState::CenterSelected(center) => {
                self.state = ArcToolState::StartSelected {
                    center,
                    start: point,
                };
                Ok(Some(ToolAction::SetStatus("Select end point".into())))
            }
            ArcToolState::StartSelected { center, start } => {
                // Calculate angles
                let start_angle = (start.y - center.y).atan2(start.x - center.x);
                let end_angle = (point.y - center.y).atan2(point.x - center.x);
                let radius = center.distance(&start);

                let arc = Arc::new(center, radius, start_angle, end_angle)?;
                self.state = ArcToolState::Initial;

                Ok(Some(ToolAction::CreateEntity(Entity::Arc(arc))))
            }
        }
    }

    fn reset(&mut self) {
        self.state = ArcToolState::Initial;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_tool() {
        let tool = RectangleTool::new();
        let rect = tool.draw(Point::new(0.0, 0.0), Point::new(10.0, 5.0)).unwrap();
        assert_eq!(rect.area(), 50.0);
    }

    #[test]
    fn test_measurement() {
        let mut tool = MeasureTool::new();
        tool.on_mouse_down(Point::new(0.0, 0.0)).unwrap();
        tool.on_mouse_up(Point::new(3.0, 4.0)).unwrap();

        let measurements = tool.get_measurements();
        assert_eq!(measurements.len(), 1);
        assert_eq!(measurements[0].distance, 5.0);
    }
}
