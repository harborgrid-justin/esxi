//! Touch and mouse gesture recognition for map interactions.

use super::{InputEvent, PointerButton, Touch};
use glam::Vec2;
use std::time::{Duration, Instant};

/// Recognized gesture type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    /// Single tap/click.
    Tap { position: Vec2 },
    /// Double tap/click.
    DoubleTap { position: Vec2 },
    /// Long press/hold.
    LongPress { position: Vec2 },
    /// Pan/drag.
    Pan { delta: Vec2, velocity: Vec2 },
    /// Pinch zoom.
    Pinch { scale: f32, center: Vec2 },
    /// Rotate.
    Rotate { angle: f32, center: Vec2 },
    /// Two-finger pan.
    TwoFingerPan { delta: Vec2 },
    /// Mouse wheel zoom.
    Wheel { delta: f32, position: Vec2 },
}

/// Gesture recognizer for detecting complex gestures.
pub struct GestureRecognizer {
    /// Configuration.
    config: GestureConfig,
    /// Current touches (for multi-touch).
    touches: Vec<Touch>,
    /// Last tap time (for double-tap detection).
    last_tap_time: Option<Instant>,
    /// Last tap position.
    last_tap_position: Option<Vec2>,
    /// Long press start time.
    long_press_start: Option<Instant>,
    /// Initial pinch distance.
    initial_pinch_distance: Option<f32>,
    /// Initial rotation angle.
    initial_rotation_angle: Option<f32>,
    /// Pan start position.
    pan_start_position: Option<Vec2>,
    /// Pan velocity for inertia.
    pan_velocity: Vec2,
    /// Last pan time for velocity calculation.
    last_pan_time: Option<Instant>,
}

impl GestureRecognizer {
    /// Create a new gesture recognizer.
    pub fn new(config: GestureConfig) -> Self {
        Self {
            config,
            touches: Vec::new(),
            last_tap_time: None,
            last_tap_position: None,
            long_press_start: None,
            initial_pinch_distance: None,
            initial_rotation_angle: None,
            pan_start_position: None,
            pan_velocity: Vec2::ZERO,
            last_pan_time: None,
        }
    }

    /// Process an input event and recognize gestures.
    pub fn process_event(&mut self, event: InputEvent) -> Option<Gesture> {
        match event {
            InputEvent::PointerDown { position, button } => {
                self.handle_pointer_down(position, button)
            }
            InputEvent::PointerMove { position } => self.handle_pointer_move(position),
            InputEvent::PointerUp { position, button } => {
                self.handle_pointer_up(position, button)
            }
            InputEvent::Wheel { delta } => self.handle_wheel(delta),
            _ => None,
        }
    }

    /// Handle pointer down event.
    fn handle_pointer_down(&mut self, position: Vec2, button: PointerButton) -> Option<Gesture> {
        if button == PointerButton::Primary {
            self.pan_start_position = Some(position);
            self.long_press_start = Some(Instant::now());
            self.last_pan_time = Some(Instant::now());
        }
        None
    }

    /// Handle pointer move event.
    fn handle_pointer_move(&mut self, position: Vec2) -> Option<Gesture> {
        // Check for long press first
        if let Some(start_time) = self.long_press_start {
            if start_time.elapsed() >= self.config.long_press_duration {
                if let Some(start_pos) = self.pan_start_position {
                    let distance = position.distance(start_pos);
                    if distance < self.config.tap_distance_threshold {
                        self.long_press_start = None;
                        return Some(Gesture::LongPress { position });
                    }
                }
            }
        }

        // Handle pan
        if let Some(start_pos) = self.pan_start_position {
            let delta = position - start_pos;

            // Calculate velocity
            if let Some(last_time) = self.last_pan_time {
                let dt = last_time.elapsed().as_secs_f32();
                if dt > 0.0 {
                    self.pan_velocity = delta / dt;
                }
            }

            self.pan_start_position = Some(position);
            self.last_pan_time = Some(Instant::now());

            return Some(Gesture::Pan {
                delta,
                velocity: self.pan_velocity,
            });
        }

        // Handle multi-touch gestures
        if self.touches.len() >= 2 {
            return self.handle_multi_touch();
        }

        None
    }

    /// Handle pointer up event.
    fn handle_pointer_up(&mut self, position: Vec2, _button: PointerButton) -> Option<Gesture> {
        self.long_press_start = None;

        // Check for tap
        if let Some(start_pos) = self.pan_start_position {
            let distance = position.distance(start_pos);

            if distance < self.config.tap_distance_threshold {
                // Check for double tap
                if let (Some(last_time), Some(last_pos)) =
                    (self.last_tap_time, self.last_tap_position)
                {
                    let time_diff = last_time.elapsed();
                    let pos_diff = position.distance(last_pos);

                    if time_diff <= self.config.double_tap_interval
                        && pos_diff < self.config.tap_distance_threshold
                    {
                        self.last_tap_time = None;
                        self.last_tap_position = None;
                        return Some(Gesture::DoubleTap { position });
                    }
                }

                // Single tap
                self.last_tap_time = Some(Instant::now());
                self.last_tap_position = Some(position);
                self.pan_start_position = None;
                return Some(Gesture::Tap { position });
            }
        }

        self.pan_start_position = None;
        None
    }

    /// Handle mouse wheel event.
    fn handle_wheel(&mut self, delta: f32) -> Option<Gesture> {
        Some(Gesture::Wheel {
            delta,
            position: Vec2::ZERO, // Would need to track mouse position
        })
    }

    /// Handle multi-touch gestures (pinch, rotate).
    fn handle_multi_touch(&mut self) -> Option<Gesture> {
        if self.touches.len() != 2 {
            return None;
        }

        let touch1 = self.touches[0];
        let touch2 = self.touches[1];

        let current_distance = touch1.position.distance(touch2.position);
        let center = (touch1.position + touch2.position) * 0.5;

        // Pinch gesture
        if let Some(initial_distance) = self.initial_pinch_distance {
            let scale = current_distance / initial_distance;

            // Check if movement is significant enough
            if (scale - 1.0).abs() > 0.01 {
                return Some(Gesture::Pinch { scale, center });
            }
        } else {
            self.initial_pinch_distance = Some(current_distance);
        }

        // Rotation gesture
        let current_angle = (touch2.position - touch1.position).y.atan2(
            (touch2.position - touch1.position).x,
        );

        if let Some(initial_angle) = self.initial_rotation_angle {
            let angle_delta = current_angle - initial_angle;

            // Check if rotation is significant enough
            if angle_delta.abs() > 0.05 {
                return Some(Gesture::Rotate {
                    angle: angle_delta.to_degrees(),
                    center,
                });
            }
        } else {
            self.initial_rotation_angle = Some(current_angle);
        }

        None
    }

    /// Add a touch point.
    pub fn add_touch(&mut self, id: u64, position: Vec2) {
        self.touches.push(Touch::new(id, position));
    }

    /// Update a touch point.
    pub fn update_touch(&mut self, id: u64, position: Vec2) {
        if let Some(touch) = self.touches.iter_mut().find(|t| t.id == id) {
            touch.update(position);
        }
    }

    /// Remove a touch point.
    pub fn remove_touch(&mut self, id: u64) {
        self.touches.retain(|t| t.id != id);

        // Reset multi-touch state when touches end
        if self.touches.len() < 2 {
            self.initial_pinch_distance = None;
            self.initial_rotation_angle = None;
        }
    }

    /// Reset all gesture state.
    pub fn reset(&mut self) {
        self.touches.clear();
        self.last_tap_time = None;
        self.last_tap_position = None;
        self.long_press_start = None;
        self.initial_pinch_distance = None;
        self.initial_rotation_angle = None;
        self.pan_start_position = None;
        self.pan_velocity = Vec2::ZERO;
        self.last_pan_time = None;
    }
}

/// Gesture recognition configuration.
#[derive(Debug, Clone)]
pub struct GestureConfig {
    /// Maximum distance for tap detection (pixels).
    pub tap_distance_threshold: f32,
    /// Maximum time between taps for double-tap (milliseconds).
    pub double_tap_interval: Duration,
    /// Minimum duration for long press (milliseconds).
    pub long_press_duration: Duration,
    /// Minimum pinch scale change to trigger.
    pub pinch_threshold: f32,
    /// Minimum rotation angle to trigger (degrees).
    pub rotation_threshold: f32,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            tap_distance_threshold: 10.0,
            double_tap_interval: Duration::from_millis(300),
            long_press_duration: Duration::from_millis(500),
            pinch_threshold: 0.01,
            rotation_threshold: 5.0,
        }
    }
}

/// Gesture handler trait for responding to gestures.
pub trait GestureHandler {
    /// Handle a recognized gesture.
    fn handle_gesture(&mut self, gesture: Gesture);
}

/// Default gesture handler for map interactions.
pub struct MapGestureHandler {
    /// Enable pan gestures.
    pub pan_enabled: bool,
    /// Enable zoom gestures.
    pub zoom_enabled: bool,
    /// Enable rotate gestures.
    pub rotate_enabled: bool,
}

impl MapGestureHandler {
    /// Create a new map gesture handler.
    pub fn new() -> Self {
        Self {
            pan_enabled: true,
            zoom_enabled: true,
            rotate_enabled: true,
        }
    }
}

impl Default for MapGestureHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureHandler for MapGestureHandler {
    fn handle_gesture(&mut self, gesture: Gesture) {
        match gesture {
            Gesture::Pan { delta, .. } if self.pan_enabled => {
                // Apply pan to camera
                log::debug!("Pan gesture: {:?}", delta);
            }
            Gesture::Pinch { scale, .. } if self.zoom_enabled => {
                // Apply zoom to camera
                log::debug!("Pinch gesture: scale={}", scale);
            }
            Gesture::Rotate { angle, .. } if self.rotate_enabled => {
                // Apply rotation to camera
                log::debug!("Rotate gesture: angle={}", angle);
            }
            Gesture::Wheel { delta, .. } if self.zoom_enabled => {
                // Apply wheel zoom to camera
                log::debug!("Wheel gesture: delta={}", delta);
            }
            Gesture::DoubleTap { position } if self.zoom_enabled => {
                // Zoom in on double tap
                log::debug!("Double tap at: {:?}", position);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gesture_config() {
        let config = GestureConfig::default();
        assert_eq!(config.tap_distance_threshold, 10.0);
        assert_eq!(config.double_tap_interval, Duration::from_millis(300));
    }

    #[test]
    fn test_gesture_recognizer() {
        let mut recognizer = GestureRecognizer::new(GestureConfig::default());

        let gesture = recognizer.process_event(InputEvent::PointerDown {
            position: Vec2::new(0.0, 0.0),
            button: PointerButton::Primary,
        });
        assert!(gesture.is_none());

        let gesture = recognizer.process_event(InputEvent::PointerUp {
            position: Vec2::new(1.0, 1.0),
            button: PointerButton::Primary,
        });
        assert!(matches!(gesture, Some(Gesture::Tap { .. })));
    }

    #[test]
    fn test_map_gesture_handler() {
        let handler = MapGestureHandler::new();
        assert!(handler.pan_enabled);
        assert!(handler.zoom_enabled);
        assert!(handler.rotate_enabled);
    }
}
