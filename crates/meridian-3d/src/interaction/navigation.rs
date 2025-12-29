//! 3D camera navigation and controls

use crate::{Camera, Error, Result};
use glam::{Vec2, Vec3, Quat};
use rapier3d::prelude::*;

/// Camera navigation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraMode {
    /// Orbit around a target point
    Orbit,
    /// Free-flying camera
    Fly,
    /// First-person camera
    FirstPerson,
    /// Walk mode (constrained to ground)
    Walk,
}

/// Navigation input state
#[derive(Debug, Default, Clone)]
pub struct NavigationInput {
    /// Mouse delta (x, y)
    pub mouse_delta: Vec2,

    /// Mouse scroll delta
    pub scroll_delta: f32,

    /// Movement input (forward/back, left/right, up/down)
    pub movement: Vec3,

    /// Modifier keys
    pub shift_pressed: bool,
    pub ctrl_pressed: bool,
    pub alt_pressed: bool,
}

/// Camera navigation controller
pub struct NavigationController {
    /// Current camera mode
    mode: CameraMode,

    /// Orbit target point
    orbit_target: Vec3,

    /// Orbit distance
    orbit_distance: f32,

    /// Orbit angles (yaw, pitch)
    orbit_angles: Vec2,

    /// Movement speed
    move_speed: f32,

    /// Rotation speed (sensitivity)
    rotation_speed: f32,

    /// Zoom speed
    zoom_speed: f32,

    /// Smooth damping factor
    damping: f32,

    /// Physics-based navigation
    use_physics: bool,

    /// Physics rigid body
    physics_body: Option<RigidBodyHandle>,
}

impl NavigationController {
    /// Create a new navigation controller
    pub fn new(mode: CameraMode) -> Self {
        Self {
            mode,
            orbit_target: Vec3::ZERO,
            orbit_distance: 100.0,
            orbit_angles: Vec2::new(0.0, 45.0_f32.to_radians()),
            move_speed: 10.0,
            rotation_speed: 0.003,
            zoom_speed: 0.1,
            damping: 0.9,
            use_physics: false,
            physics_body: None,
        }
    }

    /// Update camera based on input
    pub fn update(&mut self, camera: &mut Camera, input: &NavigationInput, delta_time: f32) {
        match self.mode {
            CameraMode::Orbit => self.update_orbit(camera, input, delta_time),
            CameraMode::Fly => self.update_fly(camera, input, delta_time),
            CameraMode::FirstPerson => self.update_first_person(camera, input, delta_time),
            CameraMode::Walk => self.update_walk(camera, input, delta_time),
        }
    }

    /// Update orbit camera
    fn update_orbit(&mut self, camera: &mut Camera, input: &NavigationInput, delta_time: f32) {
        // Rotate around target with mouse
        if input.mouse_delta.length() > 0.0 {
            self.orbit_angles.x += input.mouse_delta.x * self.rotation_speed;
            self.orbit_angles.y -= input.mouse_delta.y * self.rotation_speed;

            // Clamp pitch
            self.orbit_angles.y = self.orbit_angles.y.clamp(-1.5, 1.5);
        }

        // Zoom with scroll
        if input.scroll_delta != 0.0 {
            self.orbit_distance *= 1.0 - input.scroll_delta * self.zoom_speed;
            self.orbit_distance = self.orbit_distance.clamp(1.0, 10000.0);
        }

        // Pan target with Shift + drag
        if input.shift_pressed && input.mouse_delta.length() > 0.0 {
            let right = camera.right();
            let up = camera.up();

            self.orbit_target += right * -input.mouse_delta.x * self.move_speed * 0.1;
            self.orbit_target += up * input.mouse_delta.y * self.move_speed * 0.1;
        }

        // Calculate camera position
        let offset = Vec3::new(
            self.orbit_angles.y.cos() * self.orbit_angles.x.sin(),
            self.orbit_angles.y.sin(),
            self.orbit_angles.y.cos() * self.orbit_angles.x.cos(),
        ) * self.orbit_distance;

        camera.position = self.orbit_target + offset;
        camera.target = self.orbit_target;
    }

    /// Update fly camera
    fn update_fly(&mut self, camera: &mut Camera, input: &NavigationInput, delta_time: f32) {
        // Rotate camera with mouse
        if input.mouse_delta.length() > 0.0 {
            let yaw = -input.mouse_delta.x * self.rotation_speed;
            let pitch = -input.mouse_delta.y * self.rotation_speed;

            let direction = camera.direction();
            let right = camera.right();
            let up = Vec3::Y;

            // Apply rotation
            let yaw_quat = Quat::from_axis_angle(up, yaw);
            let pitch_quat = Quat::from_axis_angle(right, pitch);

            let new_direction = yaw_quat * pitch_quat * direction;
            camera.target = camera.position + new_direction;
        }

        // Move camera
        if input.movement.length() > 0.0 {
            let forward = camera.direction();
            let right = camera.right();
            let up = Vec3::Y;

            let speed = if input.shift_pressed {
                self.move_speed * 3.0
            } else {
                self.move_speed
            };

            camera.position += forward * input.movement.z * speed * delta_time;
            camera.position += right * input.movement.x * speed * delta_time;
            camera.position += up * input.movement.y * speed * delta_time;

            camera.target = camera.position + forward;
        }
    }

    /// Update first-person camera
    fn update_first_person(&mut self, camera: &mut Camera, input: &NavigationInput, delta_time: f32) {
        // Similar to fly, but movement is on XZ plane only
        self.update_fly(camera, input, delta_time);

        // Optionally constrain height
        // camera.position.y = camera.position.y.max(1.8); // Eye height
    }

    /// Update walk camera (ground-constrained)
    fn update_walk(&mut self, camera: &mut Camera, input: &NavigationInput, delta_time: f32) {
        // Similar to first-person but with collision detection
        self.update_first_person(camera, input, delta_time);

        // Physics-based movement would go here if enabled
        if self.use_physics {
            // Use rapier3d for collision detection
        }
    }

    /// Get camera mode
    pub fn mode(&self) -> CameraMode {
        self.mode
    }

    /// Set camera mode
    pub fn set_mode(&mut self, mode: CameraMode) {
        self.mode = mode;
    }

    /// Get orbit target
    pub fn orbit_target(&self) -> Vec3 {
        self.orbit_target
    }

    /// Set orbit target
    pub fn set_orbit_target(&mut self, target: Vec3) {
        self.orbit_target = target;
    }

    /// Get move speed
    pub fn move_speed(&self) -> f32 {
        self.move_speed
    }

    /// Set move speed
    pub fn set_move_speed(&mut self, speed: f32) {
        self.move_speed = speed.max(0.1);
    }

    /// Get rotation speed
    pub fn rotation_speed(&self) -> f32 {
        self.rotation_speed
    }

    /// Set rotation speed
    pub fn set_rotation_speed(&mut self, speed: f32) {
        self.rotation_speed = speed.max(0.0001);
    }

    /// Enable physics-based navigation
    pub fn enable_physics(&mut self, enable: bool) {
        self.use_physics = enable;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_controller() {
        let controller = NavigationController::new(CameraMode::Orbit);
        assert_eq!(controller.mode(), CameraMode::Orbit);
    }

    #[test]
    fn test_camera_modes() {
        let mut controller = NavigationController::new(CameraMode::Orbit);
        assert_eq!(controller.mode(), CameraMode::Orbit);

        controller.set_mode(CameraMode::Fly);
        assert_eq!(controller.mode(), CameraMode::Fly);

        controller.set_mode(CameraMode::FirstPerson);
        assert_eq!(controller.mode(), CameraMode::FirstPerson);
    }

    #[test]
    fn test_orbit_target() {
        let mut controller = NavigationController::new(CameraMode::Orbit);
        let target = Vec3::new(10.0, 0.0, 10.0);

        controller.set_orbit_target(target);
        assert_eq!(controller.orbit_target(), target);
    }

    #[test]
    fn test_speeds() {
        let mut controller = NavigationController::new(CameraMode::Fly);

        controller.set_move_speed(20.0);
        assert_eq!(controller.move_speed(), 20.0);

        controller.set_rotation_speed(0.005);
        assert_eq!(controller.rotation_speed(), 0.005);
    }
}
