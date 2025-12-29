//! Viewshed analysis - visibility from a point

use crate::{Scene, Error, Result};
use glam::{Vec2, Vec3};
use std::collections::HashMap;

/// Viewshed analysis result
pub struct ViewshedResult {
    /// Visibility map (position -> visible)
    pub visibility_map: HashMap<(i32, i32), bool>,

    /// Observer position
    pub observer_position: Vec3,

    /// Maximum distance analyzed
    pub max_distance: f32,

    /// Resolution (meters per cell)
    pub resolution: f32,

    /// Visible area (square meters)
    pub visible_area: f32,
}

impl ViewshedResult {
    /// Check if a position is visible
    pub fn is_visible(&self, position: Vec2) -> bool {
        let cell_x = (position.x / self.resolution) as i32;
        let cell_y = (position.y / self.resolution) as i32;

        self.visibility_map.get(&(cell_x, cell_y)).copied().unwrap_or(false)
    }

    /// Get visibility percentage
    pub fn visibility_percentage(&self) -> f32 {
        let visible_count = self.visibility_map.values().filter(|&&v| v).count();
        let total_count = self.visibility_map.len();

        if total_count > 0 {
            visible_count as f32 / total_count as f32 * 100.0
        } else {
            0.0
        }
    }
}

/// Viewshed analyzer
pub struct ViewshedAnalysis {
    /// Resolution in meters
    resolution: f32,
}

impl ViewshedAnalysis {
    /// Create a new viewshed analyzer
    pub fn new() -> Self {
        Self {
            resolution: 1.0,
        }
    }

    /// Perform viewshed analysis
    pub fn analyze(
        &self,
        scene: &Scene,
        observer_position: Vec3,
        max_distance: f32,
    ) -> Result<ViewshedResult> {
        let mut visibility_map = HashMap::new();

        let cells = (max_distance / self.resolution) as i32;

        for x in -cells..=cells {
            for y in -cells..=cells {
                let world_x = observer_position.x + x as f32 * self.resolution;
                let world_z = observer_position.z + y as f32 * self.resolution;

                let target = Vec3::new(world_x, 0.0, world_z); // Assume ground level
                let distance = Vec2::new(x as f32, y as f32).length() * self.resolution;

                if distance <= max_distance {
                    let visible = self.is_point_visible(scene, observer_position, target);
                    visibility_map.insert((x, y), visible);
                }
            }
        }

        let visible_area = visibility_map.values().filter(|&&v| v).count() as f32
            * self.resolution * self.resolution;

        Ok(ViewshedResult {
            visibility_map,
            observer_position,
            max_distance,
            resolution: self.resolution,
            visible_area,
        })
    }

    /// Check if a point is visible from observer (line-of-sight)
    fn is_point_visible(&self, scene: &Scene, observer: Vec3, target: Vec3) -> bool {
        // Raycast from observer to target
        // For now, simplified - always visible
        // Real implementation would check terrain and building occlusion
        true
    }

    /// Set resolution
    pub fn set_resolution(&mut self, resolution: f32) {
        self.resolution = resolution.max(0.1);
    }
}

impl Default for ViewshedAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewshed_analysis() {
        let analyzer = ViewshedAnalysis::new();
        // Basic test
    }
}
