//! Flythrough animation and camera path interpolation

use crate::{Camera, Transform, Error, Result};
use glam::{Vec3, Quat};
use serde::{Deserialize, Serialize};

/// Keyframe for flythrough animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlythroughKeyframe {
    /// Time in seconds
    pub time: f32,

    /// Camera position
    pub position: Vec3,

    /// Camera target/look-at
    pub target: Vec3,

    /// Field of view
    pub fov: f32,

    /// Easing function
    pub easing: EasingFunction,
}

/// Easing function for smooth interpolation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EasingFunction {
    /// Linear interpolation
    Linear,
    /// Ease in (slow start)
    EaseIn,
    /// Ease out (slow end)
    EaseOut,
    /// Ease in-out (slow start and end)
    EaseInOut,
}

impl EasingFunction {
    /// Apply easing to a value [0, 1]
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

/// Flythrough path with keyframes
pub struct FlythroughPath {
    /// Keyframes
    keyframes: Vec<FlythroughKeyframe>,

    /// Total duration
    duration: f32,

    /// Loop the animation
    looping: bool,
}

impl FlythroughPath {
    /// Create a new flythrough path
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
            duration: 0.0,
            looping: false,
        }
    }

    /// Add a keyframe
    pub fn add_keyframe(&mut self, keyframe: FlythroughKeyframe) {
        self.duration = self.duration.max(keyframe.time);
        self.keyframes.push(keyframe);
        self.keyframes.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    }

    /// Get keyframes
    pub fn keyframes(&self) -> &[FlythroughKeyframe] {
        &self.keyframes
    }

    /// Get duration
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Set looping
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }

    /// Sample camera at a specific time
    pub fn sample(&self, time: f32) -> Option<Camera> {
        if self.keyframes.is_empty() {
            return None;
        }

        let time = if self.looping {
            time % self.duration
        } else {
            time.clamp(0.0, self.duration)
        };

        // Find surrounding keyframes
        let mut prev_idx = 0;
        let mut next_idx = 0;

        for (i, keyframe) in self.keyframes.iter().enumerate() {
            if keyframe.time <= time {
                prev_idx = i;
            }
            if keyframe.time >= time {
                next_idx = i;
                break;
            }
        }

        if prev_idx == next_idx {
            // Exact keyframe
            let kf = &self.keyframes[prev_idx];
            Some(Camera::new(kf.position, kf.target, 16.0 / 9.0))
        } else {
            // Interpolate between keyframes
            let prev = &self.keyframes[prev_idx];
            let next = &self.keyframes[next_idx];

            let t = (time - prev.time) / (next.time - prev.time);
            let t_eased = next.easing.apply(t);

            let position = prev.position.lerp(next.position, t_eased);
            let target = prev.target.lerp(next.target, t_eased);
            let fov = prev.fov + (next.fov - prev.fov) * t_eased;

            let mut camera = Camera::new(position, target, 16.0 / 9.0);
            camera.fov = fov;

            Some(camera)
        }
    }
}

impl Default for FlythroughPath {
    fn default() -> Self {
        Self::new()
    }
}

/// Flythrough animator
pub struct FlythroughAnimator {
    /// Current path
    path: Option<FlythroughPath>,

    /// Current time
    current_time: f32,

    /// Playing state
    playing: bool,

    /// Playback speed
    speed: f32,
}

impl FlythroughAnimator {
    /// Create a new flythrough animator
    pub fn new() -> Self {
        Self {
            path: None,
            current_time: 0.0,
            playing: false,
            speed: 1.0,
        }
    }

    /// Set the flythrough path
    pub fn set_path(&mut self, path: FlythroughPath) {
        self.path = Some(path);
        self.current_time = 0.0;
    }

    /// Play the animation
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Stop and reset
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_time = 0.0;
    }

    /// Seek to a specific time
    pub fn seek(&mut self, time: f32) {
        self.current_time = time;
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.1);
    }

    /// Update animation
    pub fn update(&mut self, delta_time: f32) -> Option<Camera> {
        if !self.playing {
            return self.current_camera();
        }

        self.current_time += delta_time * self.speed;

        if let Some(ref path) = self.path {
            if !path.looping && self.current_time >= path.duration() {
                self.playing = false;
                self.current_time = path.duration();
            }
        }

        self.current_camera()
    }

    /// Get current camera
    pub fn current_camera(&self) -> Option<Camera> {
        self.path.as_ref().and_then(|path| path.sample(self.current_time))
    }

    /// Get current time
    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    /// Check if playing
    pub fn is_playing(&self) -> bool {
        self.playing
    }
}

impl Default for FlythroughAnimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flythrough_path() {
        let mut path = FlythroughPath::new();

        path.add_keyframe(FlythroughKeyframe {
            time: 0.0,
            position: Vec3::ZERO,
            target: Vec3::new(0.0, 0.0, -10.0),
            fov: 60.0_f32.to_radians(),
            easing: EasingFunction::Linear,
        });

        path.add_keyframe(FlythroughKeyframe {
            time: 5.0,
            position: Vec3::new(10.0, 0.0, 0.0),
            target: Vec3::new(10.0, 0.0, -10.0),
            fov: 60.0_f32.to_radians(),
            easing: EasingFunction::Linear,
        });

        assert_eq!(path.duration(), 5.0);
        assert_eq!(path.keyframes().len(), 2);

        let camera = path.sample(2.5).unwrap();
        assert!((camera.position.x - 5.0).abs() < 0.1);
    }

    #[test]
    fn test_easing_functions() {
        assert_eq!(EasingFunction::Linear.apply(0.5), 0.5);

        let ease_in = EasingFunction::EaseIn.apply(0.5);
        assert!(ease_in < 0.5);

        let ease_out = EasingFunction::EaseOut.apply(0.5);
        assert!(ease_out > 0.5);
    }

    #[test]
    fn test_animator() {
        let mut animator = FlythroughAnimator::new();
        assert!(!animator.is_playing());

        let mut path = FlythroughPath::new();
        path.add_keyframe(FlythroughKeyframe {
            time: 0.0,
            position: Vec3::ZERO,
            target: Vec3::NEG_Z,
            fov: 60.0_f32.to_radians(),
            easing: EasingFunction::Linear,
        });

        animator.set_path(path);
        animator.play();

        assert!(animator.is_playing());

        animator.update(1.0);
        assert_eq!(animator.current_time(), 1.0);
    }
}
