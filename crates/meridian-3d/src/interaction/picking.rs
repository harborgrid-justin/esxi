//! Ray-based 3D object picking

use crate::{Camera, Scene, Error, Result};
use crate::scene::{SceneNode, NodeId};
use glam::{Vec2, Vec3, Mat4};
use std::sync::Arc;
use parking_lot::RwLock;

/// Ray for raycasting
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Ray origin
    pub origin: Vec3,

    /// Ray direction (normalized)
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Get point along ray at distance t
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    /// Create a ray from screen coordinates
    pub fn from_screen(screen_pos: Vec2, screen_size: Vec2, camera: &Camera) -> Self {
        // Convert screen coordinates to NDC
        let ndc_x = (2.0 * screen_pos.x) / screen_size.x - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / screen_size.y;

        // Convert NDC to world space
        let inv_vp = camera.view_projection_matrix().inverse();

        let near_point = inv_vp.transform_point3(Vec3::new(ndc_x, ndc_y, -1.0));
        let far_point = inv_vp.transform_point3(Vec3::new(ndc_x, ndc_y, 1.0));

        let direction = (far_point - near_point).normalize();

        Self::new(camera.position, direction)
    }
}

/// Result of a picking operation
#[derive(Debug, Clone)]
pub struct PickResult {
    /// Hit node ID
    pub node_id: NodeId,

    /// Hit position in world space
    pub position: Vec3,

    /// Hit normal
    pub normal: Vec3,

    /// Distance from ray origin
    pub distance: f32,
}

/// 3D object picker using raycasting
pub struct Picker {
    /// Maximum pick distance
    max_distance: f32,
}

impl Picker {
    /// Create a new picker
    pub fn new() -> Self {
        Self {
            max_distance: 10000.0,
        }
    }

    /// Pick objects from screen coordinates
    pub fn pick_screen(
        &self,
        screen_pos: Vec2,
        screen_size: Vec2,
        camera: &Camera,
        scene: &Scene,
    ) -> Option<PickResult> {
        let ray = Ray::from_screen(screen_pos, screen_size, camera);
        self.pick_ray(ray, scene)
    }

    /// Pick objects using a ray
    pub fn pick_ray(&self, ray: Ray, scene: &Scene) -> Option<PickResult> {
        let mut closest_hit: Option<PickResult> = None;

        // Get all visible nodes
        let nodes = scene.get_visible_nodes();

        for node in nodes {
            let node_read = node.read();

            if let Some(bounds) = node_read.bounds() {
                if let Some(hit) = self.ray_box_intersection(&ray, &bounds) {
                    if hit.distance < self.max_distance {
                        if let Some(ref current_closest) = closest_hit {
                            if hit.distance < current_closest.distance {
                                closest_hit = Some(PickResult {
                                    node_id: node_read.id(),
                                    position: hit.position,
                                    normal: hit.normal,
                                    distance: hit.distance,
                                });
                            }
                        } else {
                            closest_hit = Some(PickResult {
                                node_id: node_read.id(),
                                position: hit.position,
                                normal: hit.normal,
                                distance: hit.distance,
                            });
                        }
                    }
                }
            }
        }

        closest_hit
    }

    /// Ray-box intersection test
    fn ray_box_intersection(
        &self,
        ray: &Ray,
        bounds: &crate::scene::BoundingBox,
    ) -> Option<PickResult> {
        let inv_dir = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );

        let t1 = (bounds.min.x - ray.origin.x) * inv_dir.x;
        let t2 = (bounds.max.x - ray.origin.x) * inv_dir.x;
        let t3 = (bounds.min.y - ray.origin.y) * inv_dir.y;
        let t4 = (bounds.max.y - ray.origin.y) * inv_dir.y;
        let t5 = (bounds.min.z - ray.origin.z) * inv_dir.z;
        let t6 = (bounds.max.z - ray.origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            return None;
        }

        let distance = if tmin < 0.0 { tmax } else { tmin };
        let position = ray.at(distance);

        // Calculate normal (simplified - use face normal)
        let center = bounds.center();
        let normal = (position - center).normalize();

        Some(PickResult {
            node_id: uuid::Uuid::nil(),
            position,
            normal,
            distance,
        })
    }

    /// Set maximum pick distance
    pub fn set_max_distance(&mut self, distance: f32) {
        self.max_distance = distance.max(0.1);
    }

    /// Get maximum pick distance
    pub fn max_distance(&self) -> f32 {
        self.max_distance
    }
}

impl Default for Picker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_creation() {
        let ray = Ray::new(Vec3::ZERO, Vec3::NEG_Z);
        assert_eq!(ray.origin, Vec3::ZERO);
        assert!((ray.direction.length() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ray_at() {
        let ray = Ray::new(Vec3::ZERO, Vec3::NEG_Z);
        let point = ray.at(10.0);
        assert_eq!(point, Vec3::new(0.0, 0.0, -10.0));
    }

    #[test]
    fn test_picker_creation() {
        let picker = Picker::new();
        assert_eq!(picker.max_distance(), 10000.0);
    }
}
