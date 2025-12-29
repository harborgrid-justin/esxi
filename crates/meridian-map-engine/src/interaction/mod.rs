//! User interaction system for touch/mouse gestures and feature picking.

pub mod gestures;
pub mod picker;

use glam::Vec2;

/// Input event type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    /// Mouse/touch down.
    PointerDown {
        position: Vec2,
        button: PointerButton,
    },
    /// Mouse/touch move.
    PointerMove { position: Vec2 },
    /// Mouse/touch up.
    PointerUp {
        position: Vec2,
        button: PointerButton,
    },
    /// Mouse wheel scroll.
    Wheel { delta: f32 },
    /// Pinch gesture (two-finger zoom).
    Pinch { scale: f32, center: Vec2 },
    /// Rotate gesture.
    Rotate { angle: f32, center: Vec2 },
    /// Keyboard key press.
    KeyPress { key: KeyCode },
    /// Keyboard key release.
    KeyRelease { key: KeyCode },
}

/// Pointer button type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    /// Left mouse button or primary touch.
    Primary,
    /// Right mouse button.
    Secondary,
    /// Middle mouse button.
    Middle,
}

/// Keyboard key codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    /// Arrow keys.
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    /// Plus/minus for zoom.
    Plus,
    Minus,
    /// Space bar.
    Space,
    /// Escape.
    Escape,
    /// Other key.
    Other,
}

/// Interaction state tracking.
pub struct InteractionState {
    /// Current pointer position.
    pub pointer_position: Option<Vec2>,
    /// Whether pointer is down.
    pub pointer_down: bool,
    /// Pointer down position (for drag calculations).
    pub pointer_down_position: Option<Vec2>,
    /// Whether a drag is in progress.
    pub is_dragging: bool,
    /// Active touches (for multi-touch).
    pub active_touches: Vec<Touch>,
    /// Keyboard modifiers.
    pub modifiers: KeyboardModifiers,
}

impl InteractionState {
    /// Create a new interaction state.
    pub fn new() -> Self {
        Self {
            pointer_position: None,
            pointer_down: false,
            pointer_down_position: None,
            is_dragging: false,
            active_touches: Vec::new(),
            modifiers: KeyboardModifiers::default(),
        }
    }

    /// Handle an input event.
    pub fn handle_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::PointerDown { position, .. } => {
                self.pointer_down = true;
                self.pointer_down_position = Some(position);
                self.pointer_position = Some(position);
            }
            InputEvent::PointerMove { position } => {
                self.pointer_position = Some(position);

                if self.pointer_down {
                    if let Some(down_pos) = self.pointer_down_position {
                        let distance = position.distance(down_pos);
                        if distance > 5.0 {
                            // Threshold for drag
                            self.is_dragging = true;
                        }
                    }
                }
            }
            InputEvent::PointerUp { position, .. } => {
                self.pointer_down = false;
                self.is_dragging = false;
                self.pointer_position = Some(position);
                self.pointer_down_position = None;
            }
            _ => {}
        }
    }

    /// Get the current drag delta.
    pub fn drag_delta(&self) -> Option<Vec2> {
        if self.is_dragging {
            if let (Some(current), Some(down)) = (self.pointer_position, self.pointer_down_position)
            {
                Some(current - down)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Reset interaction state.
    pub fn reset(&mut self) {
        self.pointer_down = false;
        self.is_dragging = false;
        self.pointer_down_position = None;
        self.active_touches.clear();
    }
}

impl Default for InteractionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Touch point for multi-touch gestures.
#[derive(Debug, Clone, Copy)]
pub struct Touch {
    /// Touch identifier.
    pub id: u64,
    /// Touch position.
    pub position: Vec2,
    /// Previous position (for delta calculations).
    pub previous_position: Vec2,
}

impl Touch {
    /// Create a new touch.
    pub fn new(id: u64, position: Vec2) -> Self {
        Self {
            id,
            position,
            previous_position: position,
        }
    }

    /// Update touch position.
    pub fn update(&mut self, position: Vec2) {
        self.previous_position = self.position;
        self.position = position;
    }

    /// Get touch delta.
    pub fn delta(&self) -> Vec2 {
        self.position - self.previous_position
    }
}

/// Keyboard modifiers.
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyboardModifiers {
    /// Shift key is pressed.
    pub shift: bool,
    /// Control key is pressed.
    pub ctrl: bool,
    /// Alt key is pressed.
    pub alt: bool,
    /// Command/Super key is pressed.
    pub meta: bool,
}

/// Click/tap detection.
pub struct ClickDetector {
    /// Maximum time for a click in milliseconds.
    max_click_duration: std::time::Duration,
    /// Maximum distance for a click in pixels.
    max_click_distance: f32,
    /// Click start time.
    click_start_time: Option<std::time::Instant>,
    /// Click start position.
    click_start_position: Option<Vec2>,
}

impl ClickDetector {
    /// Create a new click detector.
    pub fn new() -> Self {
        Self {
            max_click_duration: std::time::Duration::from_millis(300),
            max_click_distance: 10.0,
            click_start_time: None,
            click_start_position: None,
        }
    }

    /// Start click detection.
    pub fn start(&mut self, position: Vec2) {
        self.click_start_time = Some(std::time::Instant::now());
        self.click_start_position = Some(position);
    }

    /// End click detection and check if it was a valid click.
    pub fn end(&mut self, position: Vec2) -> bool {
        if let (Some(start_time), Some(start_pos)) =
            (self.click_start_time, self.click_start_position)
        {
            let duration = start_time.elapsed();
            let distance = position.distance(start_pos);

            let is_click = duration <= self.max_click_duration
                && distance <= self.max_click_distance;

            self.click_start_time = None;
            self.click_start_position = None;

            is_click
        } else {
            false
        }
    }

    /// Reset click detection.
    pub fn reset(&mut self) {
        self.click_start_time = None;
        self.click_start_position = None;
    }
}

impl Default for ClickDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_state() {
        let mut state = InteractionState::new();
        assert!(!state.pointer_down);
        assert!(!state.is_dragging);

        state.handle_event(InputEvent::PointerDown {
            position: Vec2::new(0.0, 0.0),
            button: PointerButton::Primary,
        });
        assert!(state.pointer_down);
        assert!(!state.is_dragging);

        state.handle_event(InputEvent::PointerMove {
            position: Vec2::new(10.0, 10.0),
        });
        assert!(state.is_dragging);

        state.handle_event(InputEvent::PointerUp {
            position: Vec2::new(10.0, 10.0),
            button: PointerButton::Primary,
        });
        assert!(!state.pointer_down);
        assert!(!state.is_dragging);
    }

    #[test]
    fn test_touch() {
        let mut touch = Touch::new(1, Vec2::new(0.0, 0.0));
        assert_eq!(touch.delta(), Vec2::ZERO);

        touch.update(Vec2::new(5.0, 5.0));
        assert_eq!(touch.delta(), Vec2::new(5.0, 5.0));
    }

    #[test]
    fn test_click_detector() {
        let mut detector = ClickDetector::new();

        detector.start(Vec2::new(0.0, 0.0));
        let is_click = detector.end(Vec2::new(2.0, 2.0));
        assert!(is_click);

        detector.start(Vec2::new(0.0, 0.0));
        let is_not_click = detector.end(Vec2::new(100.0, 100.0));
        assert!(!is_not_click);
    }
}
