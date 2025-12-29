//! Camera movement controller for smooth animations and interactions.

use super::Camera;
use glam::Vec2;
use std::time::Duration;

/// Camera animation state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    /// No animation running.
    Idle,
    /// Camera is animating.
    Animating,
    /// Animation is easing out.
    EasingOut,
}

/// Easing function for smooth animations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EasingFunction {
    /// Linear interpolation.
    Linear,
    /// Ease in (accelerate).
    EaseIn,
    /// Ease out (decelerate).
    EaseOut,
    /// Ease in-out (accelerate then decelerate).
    EaseInOut,
    /// Exponential ease out.
    ExpoOut,
}

impl EasingFunction {
    /// Apply the easing function to a normalized time value (0.0 to 1.0).
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => t * t,
            EasingFunction::EaseOut => t * (2.0 - t),
            EasingFunction::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            EasingFunction::ExpoOut => {
                if t >= 1.0 {
                    1.0
                } else {
                    1.0 - 2_f32.powf(-10.0 * t)
                }
            }
        }
    }
}

/// Animation target for camera movement.
#[derive(Debug, Clone)]
struct AnimationTarget {
    /// Target center position.
    target_center: Vec2,
    /// Target zoom level.
    target_zoom: f32,
    /// Target bearing.
    target_bearing: f32,
    /// Target pitch.
    target_pitch: f32,
    /// Animation duration.
    duration: Duration,
    /// Elapsed time.
    elapsed: Duration,
    /// Easing function.
    easing: EasingFunction,
}

/// Camera controller for smooth movement and animations.
pub struct CameraController {
    /// Current animation target.
    animation: Option<AnimationTarget>,
    /// Inertia settings for smooth panning.
    inertia_enabled: bool,
    /// Current velocity for inertia.
    velocity: Vec2,
    /// Inertia decay factor (0.0 to 1.0).
    inertia_decay: f32,
    /// Minimum velocity threshold.
    min_velocity: f32,
}

impl CameraController {
    /// Create a new camera controller.
    pub fn new() -> Self {
        Self {
            animation: None,
            inertia_enabled: true,
            velocity: Vec2::ZERO,
            inertia_decay: 0.9,
            min_velocity: 0.01,
        }
    }

    /// Enable or disable inertia.
    pub fn set_inertia_enabled(&mut self, enabled: bool) {
        self.inertia_enabled = enabled;
    }

    /// Animate the camera to a new position.
    pub fn animate_to(
        &mut self,
        center: Vec2,
        zoom: f32,
        bearing: f32,
        pitch: f32,
        duration: Duration,
        easing: EasingFunction,
    ) {
        self.animation = Some(AnimationTarget {
            target_center: center,
            target_zoom: zoom,
            target_bearing: bearing,
            target_pitch: pitch,
            duration,
            elapsed: Duration::ZERO,
            easing,
        });

        // Stop inertia when starting animation
        self.velocity = Vec2::ZERO;
    }

    /// Fly to a position with default easing.
    pub fn fly_to(&mut self, center: Vec2, zoom: f32, duration: Duration) {
        self.animate_to(
            center,
            zoom,
            0.0,
            0.0,
            duration,
            EasingFunction::ExpoOut,
        );
    }

    /// Update the camera controller.
    pub fn update(&mut self, camera: &mut Camera, delta_time: Duration) -> AnimationState {
        let mut state = AnimationState::Idle;

        // Update animation
        if let Some(ref mut anim) = self.animation {
            anim.elapsed += delta_time;
            let t = (anim.elapsed.as_secs_f32() / anim.duration.as_secs_f32()).min(1.0);
            let eased_t = anim.easing.apply(t);

            // Store original values
            let start_center = camera.center();
            let start_zoom = camera.zoom;
            let start_bearing = camera.bearing;
            let start_pitch = camera.pitch;

            // Interpolate camera values
            let new_center = start_center.lerp(anim.target_center, eased_t);
            camera.set_center(new_center.x, new_center.y);
            camera.zoom = start_zoom + (anim.target_zoom - start_zoom) * eased_t;
            camera.bearing = start_bearing + (anim.target_bearing - start_bearing) * eased_t;
            camera.pitch = start_pitch + (anim.target_pitch - start_pitch) * eased_t;
            camera.update_position();

            state = if t >= 1.0 {
                AnimationState::EasingOut
            } else {
                AnimationState::Animating
            };

            // Remove animation when complete
            if t >= 1.0 {
                self.animation = None;
            }
        }

        // Update inertia
        if self.inertia_enabled && self.velocity.length() > self.min_velocity {
            camera.pan(self.velocity * delta_time.as_secs_f32());
            self.velocity *= self.inertia_decay;

            if state == AnimationState::Idle {
                state = AnimationState::EasingOut;
            }
        } else if self.velocity.length() <= self.min_velocity {
            self.velocity = Vec2::ZERO;
        }

        state
    }

    /// Add velocity for inertia panning.
    pub fn add_velocity(&mut self, velocity: Vec2) {
        if self.inertia_enabled {
            self.velocity += velocity;
        }
    }

    /// Stop all animations and inertia.
    pub fn stop(&mut self) {
        self.animation = None;
        self.velocity = Vec2::ZERO;
    }

    /// Check if an animation is running.
    pub fn is_animating(&self) -> bool {
        self.animation.is_some()
    }

    /// Get the current animation progress (0.0 to 1.0).
    pub fn animation_progress(&self) -> f32 {
        if let Some(ref anim) = self.animation {
            (anim.elapsed.as_secs_f32() / anim.duration.as_secs_f32()).min(1.0)
        } else {
            1.0
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new()
    }
}

/// Preset camera animations.
pub struct CameraPresets;

impl CameraPresets {
    /// Standard animation duration.
    pub const STANDARD_DURATION: Duration = Duration::from_millis(500);

    /// Slow animation duration.
    pub const SLOW_DURATION: Duration = Duration::from_millis(1000);

    /// Fast animation duration.
    pub const FAST_DURATION: Duration = Duration::from_millis(250);

    /// Create a zoom-in animation.
    pub fn zoom_in(controller: &mut CameraController, camera: &Camera, levels: f32) {
        let center = camera.center();
        let new_zoom = (camera.zoom + levels).min(22.0);
        controller.animate_to(
            center,
            new_zoom,
            camera.bearing,
            camera.pitch,
            Self::STANDARD_DURATION,
            EasingFunction::EaseOut,
        );
    }

    /// Create a zoom-out animation.
    pub fn zoom_out(controller: &mut CameraController, camera: &Camera, levels: f32) {
        let center = camera.center();
        let new_zoom = (camera.zoom - levels).max(0.0);
        controller.animate_to(
            center,
            new_zoom,
            camera.bearing,
            camera.pitch,
            Self::STANDARD_DURATION,
            EasingFunction::EaseOut,
        );
    }

    /// Reset camera to default position.
    pub fn reset_north(controller: &mut CameraController, camera: &Camera) {
        let center = camera.center();
        controller.animate_to(
            center,
            camera.zoom,
            0.0,
            0.0,
            Self::STANDARD_DURATION,
            EasingFunction::EaseInOut,
        );
    }

    /// Rotate to specific bearing.
    pub fn rotate_to(controller: &mut CameraController, camera: &Camera, bearing: f32) {
        let center = camera.center();
        controller.animate_to(
            center,
            camera.zoom,
            bearing,
            camera.pitch,
            Self::STANDARD_DURATION,
            EasingFunction::EaseInOut,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_functions() {
        let linear = EasingFunction::Linear;
        assert_eq!(linear.apply(0.5), 0.5);

        let ease_in = EasingFunction::EaseIn;
        assert!(ease_in.apply(0.5) < 0.5);

        let ease_out = EasingFunction::EaseOut;
        assert!(ease_out.apply(0.5) > 0.5);
    }

    #[test]
    fn test_camera_controller() {
        let mut controller = CameraController::new();
        assert!(!controller.is_animating());

        controller.animate_to(
            Vec2::new(10.0, 20.0),
            5.0,
            0.0,
            0.0,
            Duration::from_secs(1),
            EasingFunction::Linear,
        );

        assert!(controller.is_animating());
        assert_eq!(controller.animation_progress(), 0.0);
    }

    #[test]
    fn test_controller_stop() {
        let mut controller = CameraController::new();
        controller.add_velocity(Vec2::new(10.0, 10.0));
        controller.stop();
        assert_eq!(controller.velocity, Vec2::ZERO);
    }
}
