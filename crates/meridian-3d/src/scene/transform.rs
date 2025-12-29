//! 3D transformation system

use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// 3D transformation (position, rotation, scale)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// Position in 3D space
    pub position: Vec3,

    /// Rotation as quaternion
    pub rotation: Quat,

    /// Scale (non-uniform allowed)
    pub scale: Vec3,
}

impl Transform {
    /// Create a new transform
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Create an identity transform
    pub fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Create a transform from position only
    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Create a transform from rotation only
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        }
    }

    /// Create a transform from scale only
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale,
        }
    }

    /// Create a transform from Euler angles (in radians)
    pub fn from_euler(position: Vec3, pitch: f32, yaw: f32, roll: f32) -> Self {
        Self {
            position,
            rotation: Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, roll),
            scale: Vec3::ONE,
        }
    }

    /// Get the transformation matrix
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Get the inverse transformation matrix
    pub fn inverse_matrix(&self) -> Mat4 {
        self.matrix().inverse()
    }

    /// Transform a point
    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.matrix().transform_point3(point)
    }

    /// Transform a vector (ignores position)
    pub fn transform_vector(&self, vector: Vec3) -> Vec3 {
        self.matrix().transform_vector3(vector)
    }

    /// Get the forward direction
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// Get the right direction
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    /// Translate by a vector
    pub fn translate(&mut self, delta: Vec3) {
        self.position += delta;
    }

    /// Translate in local space
    pub fn translate_local(&mut self, delta: Vec3) {
        self.position += self.transform_vector(delta);
    }

    /// Rotate by a quaternion
    pub fn rotate(&mut self, rotation: Quat) {
        self.rotation = rotation * self.rotation;
    }

    /// Rotate around an axis
    pub fn rotate_axis(&mut self, axis: Vec3, angle: f32) {
        self.rotation = Quat::from_axis_angle(axis, angle) * self.rotation;
    }

    /// Rotate around X axis
    pub fn rotate_x(&mut self, angle: f32) {
        self.rotate_axis(Vec3::X, angle);
    }

    /// Rotate around Y axis
    pub fn rotate_y(&mut self, angle: f32) {
        self.rotate_axis(Vec3::Y, angle);
    }

    /// Rotate around Z axis
    pub fn rotate_z(&mut self, angle: f32) {
        self.rotate_axis(Vec3::Z, angle);
    }

    /// Look at a target position
    pub fn look_at(&mut self, target: Vec3, up: Vec3) {
        let direction = (target - self.position).normalize();
        if direction.length_squared() > 0.0 {
            self.rotation = look_at_rotation(direction, up);
        }
    }

    /// Scale uniformly
    pub fn scale_uniform(&mut self, scale: f32) {
        self.scale *= scale;
    }

    /// Interpolate between two transforms (LERP for position/scale, SLERP for rotation)
    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }

    /// Combine two transforms (this * other)
    pub fn mul_transform(&self, other: &Transform) -> Transform {
        Transform {
            position: self.transform_point(other.position),
            rotation: self.rotation * other.rotation,
            scale: self.scale * other.scale,
        }
    }

    /// Decompose a matrix into a transform
    pub fn from_matrix(matrix: Mat4) -> Self {
        let (scale, rotation, position) = matrix.to_scale_rotation_translation();
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Get Euler angles (pitch, yaw, roll) in radians
    pub fn to_euler(&self) -> (f32, f32, f32) {
        let (yaw, pitch, roll) = self.rotation.to_euler(glam::EulerRot::YXZ);
        (pitch, yaw, roll)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl std::ops::Mul for Transform {
    type Output = Transform;

    fn mul(self, rhs: Transform) -> Transform {
        self.mul_transform(&rhs)
    }
}

/// Helper function to create a look-at rotation
fn look_at_rotation(direction: Vec3, up: Vec3) -> Quat {
    let forward = -direction; // Negate because we look down -Z
    let right = up.cross(forward).normalize();
    let up = forward.cross(right);

    Quat::from_mat3(&glam::Mat3::from_cols(right, up, forward))
}

/// Transform builder for fluent API
pub struct TransformBuilder {
    transform: Transform,
}

impl TransformBuilder {
    /// Create a new transform builder
    pub fn new() -> Self {
        Self {
            transform: Transform::identity(),
        }
    }

    /// Set position
    pub fn position(mut self, position: Vec3) -> Self {
        self.transform.position = position;
        self
    }

    /// Set rotation from quaternion
    pub fn rotation(mut self, rotation: Quat) -> Self {
        self.transform.rotation = rotation;
        self
    }

    /// Set rotation from Euler angles
    pub fn euler(mut self, pitch: f32, yaw: f32, roll: f32) -> Self {
        self.transform.rotation = Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, roll);
        self
    }

    /// Set scale
    pub fn scale(mut self, scale: Vec3) -> Self {
        self.transform.scale = scale;
        self
    }

    /// Set uniform scale
    pub fn uniform_scale(mut self, scale: f32) -> Self {
        self.transform.scale = Vec3::splat(scale);
        self
    }

    /// Build the transform
    pub fn build(self) -> Transform {
        self.transform
    }
}

impl Default for TransformBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_identity_transform() {
        let transform = Transform::identity();
        assert_eq!(transform.matrix(), Mat4::IDENTITY);
    }

    #[test]
    fn test_transform_point() {
        let transform = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));
        let point = Vec3::ZERO;
        let transformed = transform.transform_point(point);
        assert_eq!(transformed, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_transform_rotation() {
        let mut transform = Transform::identity();
        transform.rotate_y(PI / 2.0); // 90 degrees

        let forward = transform.forward();
        assert!((forward.x - 1.0).abs() < 0.001);
        assert!(forward.y.abs() < 0.001);
        assert!(forward.z.abs() < 0.001);
    }

    #[test]
    fn test_transform_lerp() {
        let t1 = Transform::from_position(Vec3::ZERO);
        let t2 = Transform::from_position(Vec3::new(10.0, 0.0, 0.0));
        let lerped = t1.lerp(&t2, 0.5);
        assert_eq!(lerped.position, Vec3::new(5.0, 0.0, 0.0));
    }

    #[test]
    fn test_transform_builder() {
        let transform = TransformBuilder::new()
            .position(Vec3::new(1.0, 2.0, 3.0))
            .uniform_scale(2.0)
            .build();

        assert_eq!(transform.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(transform.scale, Vec3::splat(2.0));
    }

    #[test]
    fn test_matrix_decomposition() {
        let original = Transform::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from_rotation_y(PI / 4.0),
            Vec3::splat(2.0),
        );

        let matrix = original.matrix();
        let decomposed = Transform::from_matrix(matrix);

        assert!((decomposed.position - original.position).length() < 0.001);
        assert!((decomposed.scale - original.scale).length() < 0.001);
    }
}
