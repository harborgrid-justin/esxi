//! Level of detail (LOD) management for terrain

use crate::{Camera, Result};
use super::TerrainBounds;
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// LOD level configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LodLevel {
    /// LOD level index (0 = highest detail)
    pub level: u32,

    /// Mesh resolution (vertices per side)
    pub resolution: usize,

    /// Distance threshold for this LOD level
    pub distance: f32,
}

impl LodLevel {
    /// Create a new LOD level
    pub fn new(level: u32, resolution: usize, distance: f32) -> Self {
        Self {
            level,
            resolution,
            distance,
        }
    }
}

/// LOD system settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodSettings {
    /// Number of LOD levels
    pub num_levels: u32,

    /// Base resolution for highest detail level
    pub base_resolution: usize,

    /// Distance multiplier between LOD levels
    pub distance_multiplier: f32,

    /// Base distance for LOD transitions
    pub base_distance: f32,

    /// Enable smooth LOD transitions (morphing)
    pub smooth_transitions: bool,

    /// Morphing region size (as percentage of LOD distance)
    pub morph_region: f32,
}

impl Default for LodSettings {
    fn default() -> Self {
        Self {
            num_levels: 5,
            base_resolution: 256,
            distance_multiplier: 2.0,
            base_distance: 100.0,
            smooth_transitions: true,
            morph_region: 0.2,
        }
    }
}

/// LOD manager for dynamic level-of-detail selection
pub struct LodManager {
    /// LOD settings
    settings: LodSettings,

    /// Precomputed LOD levels
    levels: Vec<LodLevel>,

    /// Currently active LOD level
    active_level: u32,

    /// LOD transition factor (0.0 to 1.0) for morphing
    transition_factor: f32,
}

impl LodManager {
    /// Create a new LOD manager
    pub fn new(settings: LodSettings) -> Self {
        let levels = Self::compute_lod_levels(&settings);
        let active_level = levels.last().map(|l| l.level).unwrap_or(0);

        Self {
            settings,
            levels,
            active_level,
            transition_factor: 0.0,
        }
    }

    /// Compute LOD levels from settings
    fn compute_lod_levels(settings: &LodSettings) -> Vec<LodLevel> {
        let mut levels = Vec::new();

        for i in 0..settings.num_levels {
            let resolution = settings.base_resolution / (1 << i).max(1);
            let distance = settings.base_distance * settings.distance_multiplier.powi(i as i32);

            levels.push(LodLevel::new(i, resolution.max(4), distance));
        }

        levels
    }

    /// Update LOD based on camera position
    pub fn update(&mut self, camera_pos: Vec3, terrain_bounds: &TerrainBounds) {
        let terrain_center = terrain_bounds.center();
        let distance = camera_pos.distance(terrain_center);

        // Select appropriate LOD level
        let mut new_level = self.levels.last().unwrap().level;

        for level in &self.levels {
            if distance < level.distance {
                new_level = level.level;
                break;
            }
        }

        // Calculate transition factor for morphing
        if self.settings.smooth_transitions {
            if let Some(current_lod) = self.levels.get(new_level as usize) {
                let morph_start = current_lod.distance * (1.0 - self.settings.morph_region);
                let morph_end = current_lod.distance;

                if distance >= morph_start && distance < morph_end {
                    self.transition_factor = (distance - morph_start) / (morph_end - morph_start);
                } else {
                    self.transition_factor = 0.0;
                }
            }
        }

        self.active_level = new_level;
    }

    /// Get the currently active LOD level
    pub fn active_level(&self) -> &LodLevel {
        &self.levels[self.active_level as usize]
    }

    /// Get all LOD levels
    pub fn levels(&self) -> &[LodLevel] {
        &self.levels
    }

    /// Get transition factor for morphing
    pub fn transition_factor(&self) -> f32 {
        self.transition_factor
    }

    /// Get LOD level by index
    pub fn get_level(&self, index: u32) -> Option<&LodLevel> {
        self.levels.get(index as usize)
    }

    /// Calculate LOD level for a specific distance
    pub fn calculate_lod_for_distance(&self, distance: f32) -> &LodLevel {
        for level in &self.levels {
            if distance < level.distance {
                return level;
            }
        }
        self.levels.last().unwrap()
    }

    /// Get settings
    pub fn settings(&self) -> &LodSettings {
        &self.settings
    }

    /// Update settings and recompute levels
    pub fn update_settings(&mut self, settings: LodSettings) {
        self.settings = settings;
        self.levels = Self::compute_lod_levels(&self.settings);
    }
}

/// Quadtree-based LOD for large terrains
pub struct QuadtreeLod {
    /// Root node
    root: QuadtreeNode,

    /// Maximum depth
    max_depth: u32,

    /// Split distance threshold
    split_distance: f32,
}

/// Quadtree node for terrain LOD
struct QuadtreeNode {
    /// Bounds of this node
    bounds: (Vec3, Vec3),

    /// LOD level
    level: u32,

    /// Children (NW, NE, SW, SE)
    children: Option<Box<[QuadtreeNode; 4]>>,

    /// Is this node active for rendering?
    active: bool,
}

impl QuadtreeLod {
    /// Create a new quadtree LOD system
    pub fn new(terrain_bounds: TerrainBounds, max_depth: u32) -> Self {
        let root = QuadtreeNode {
            bounds: (terrain_bounds.min, terrain_bounds.max),
            level: 0,
            children: None,
            active: true,
        };

        Self {
            root,
            max_depth,
            split_distance: 100.0,
        }
    }

    /// Update quadtree based on camera position
    pub fn update(&mut self, camera_pos: Vec3) {
        self.update_node(&mut self.root, camera_pos, 0);
    }

    /// Recursively update nodes
    fn update_node(&mut self, node: &mut QuadtreeNode, camera_pos: Vec3, depth: u32) {
        let center = (node.bounds.0 + node.bounds.1) * 0.5;
        let distance = camera_pos.distance(center);

        // Should we split this node?
        let should_split = distance < self.split_distance && depth < self.max_depth;

        if should_split && node.children.is_none() {
            // Split node
            node.children = Some(Box::new(Self::create_children(&node.bounds)));
        } else if !should_split && node.children.is_some() {
            // Merge node
            node.children = None;
        }

        // Update children recursively
        if let Some(ref mut children) = node.children {
            for child in children.iter_mut() {
                self.update_node(child, camera_pos, depth + 1);
            }
        }
    }

    /// Create four child nodes
    fn create_children(bounds: &(Vec3, Vec3)) -> [QuadtreeNode; 4] {
        let (min, max) = bounds;
        let center = (*min + *max) * 0.5;

        [
            // Northwest
            QuadtreeNode {
                bounds: (*min, Vec3::new(center.x, max.y, center.z)),
                level: 1,
                children: None,
                active: true,
            },
            // Northeast
            QuadtreeNode {
                bounds: (Vec3::new(center.x, min.y, min.z), Vec3::new(max.x, max.y, center.z)),
                level: 1,
                children: None,
                active: true,
            },
            // Southwest
            QuadtreeNode {
                bounds: (Vec3::new(min.x, min.y, center.z), Vec3::new(center.x, max.y, max.z)),
                level: 1,
                children: None,
                active: true,
            },
            // Southeast
            QuadtreeNode {
                bounds: (center, *max),
                level: 1,
                children: None,
                active: true,
            },
        ]
    }

    /// Get all active leaf nodes for rendering
    pub fn get_active_nodes(&self) -> Vec<(Vec3, Vec3, u32)> {
        let mut nodes = Vec::new();
        self.collect_active_nodes(&self.root, &mut nodes);
        nodes
    }

    /// Collect active leaf nodes
    fn collect_active_nodes(&self, node: &QuadtreeNode, result: &mut Vec<(Vec3, Vec3, u32)>) {
        if let Some(ref children) = node.children {
            for child in children.iter() {
                self.collect_active_nodes(child, result);
            }
        } else if node.active {
            result.push((node.bounds.0, node.bounds.1, node.level));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_manager() {
        let settings = LodSettings::default();
        let manager = LodManager::new(settings);

        assert_eq!(manager.levels().len(), 5);
        assert_eq!(manager.active_level().level, 4); // Starts at lowest detail
    }

    #[test]
    fn test_lod_selection() {
        let settings = LodSettings {
            num_levels: 3,
            base_resolution: 256,
            distance_multiplier: 2.0,
            base_distance: 100.0,
            smooth_transitions: false,
            morph_region: 0.2,
        };

        let manager = LodManager::new(settings);

        // Close distance should select LOD 0
        let lod = manager.calculate_lod_for_distance(50.0);
        assert_eq!(lod.level, 0);

        // Far distance should select LOD 2
        let lod = manager.calculate_lod_for_distance(500.0);
        assert_eq!(lod.level, 2);
    }

    #[test]
    fn test_lod_levels() {
        let settings = LodSettings::default();
        let levels = LodManager::compute_lod_levels(&settings);

        assert_eq!(levels[0].resolution, 256);
        assert_eq!(levels[1].resolution, 128);
        assert_eq!(levels[2].resolution, 64);
    }
}
