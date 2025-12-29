//! Camera system for map viewing and navigation.

pub mod controller;
pub mod projection;

use glam::{Mat4, Vec2, Vec3};

/// Camera for viewing the map.
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera position in world coordinates.
    pub position: Vec3,
    /// Camera target (look-at point).
    pub target: Vec3,
    /// Up vector.
    pub up: Vec3,
    /// Zoom level (0-22 for typical web mercator).
    pub zoom: f32,
    /// Camera pitch angle in degrees (0-60 for 2.5D).
    pub pitch: f32,
    /// Camera bearing/rotation in degrees (0-360).
    pub bearing: f32,
    /// Field of view in degrees (for perspective projection).
    pub fov: f32,
    /// Near clipping plane.
    pub near: f32,
    /// Far clipping plane.
    pub far: f32,
}

impl Camera {
    /// Create a new camera with default settings.
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 1000.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            zoom: 0.0,
            pitch: 0.0,
            bearing: 0.0,
            fov: 45.0,
            near: 0.1,
            far: 10000.0,
        }
    }

    /// Create a camera at a specific position and zoom level.
    pub fn at_position(lon: f32, lat: f32, zoom: f32) -> Self {
        let mut camera = Self::new();
        camera.set_center(lon, lat);
        camera.zoom = zoom;
        camera.update_position();
        camera
    }

    /// Set the camera center position.
    pub fn set_center(&mut self, lon: f32, lat: f32) {
        self.target = Vec3::new(lon, lat, 0.0);
    }

    /// Get the camera center position.
    pub fn center(&self) -> Vec2 {
        Vec2::new(self.target.x, self.target.y)
    }

    /// Update camera position based on zoom, pitch, and bearing.
    pub fn update_position(&mut self) {
        let distance = Self::zoom_to_distance(self.zoom);

        // Calculate position based on pitch and bearing
        let pitch_rad = self.pitch.to_radians();
        let bearing_rad = self.bearing.to_radians();

        let x = self.target.x + distance * pitch_rad.cos() * bearing_rad.sin();
        let y = self.target.y + distance * pitch_rad.cos() * bearing_rad.cos();
        let z = distance * pitch_rad.sin().max(0.1);

        self.position = Vec3::new(x, y, z);
    }

    /// Convert zoom level to camera distance.
    fn zoom_to_distance(zoom: f32) -> f32 {
        // Web Mercator-style zoom calculation
        let base_distance = 40000000.0; // ~Earth's circumference in units
        base_distance / (2_f32.powf(zoom))
    }

    /// Build the view matrix.
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, self.up)
    }

    /// Build the projection matrix.
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        if self.pitch > 0.0 {
            // Perspective projection for 2.5D view
            Mat4::perspective_rh(
                self.fov.to_radians(),
                aspect_ratio,
                self.near,
                self.far,
            )
        } else {
            // Orthographic projection for 2D view
            let distance = Self::zoom_to_distance(self.zoom);
            let height = distance * 0.5;
            let width = height * aspect_ratio;

            Mat4::orthographic_rh(
                -width,
                width,
                -height,
                height,
                self.near,
                self.far,
            )
        }
    }

    /// Build the view-projection matrix.
    pub fn view_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        self.projection_matrix(aspect_ratio) * self.view_matrix()
    }

    /// Pan the camera by a delta in screen space.
    pub fn pan(&mut self, delta: Vec2) {
        let scale = Self::zoom_to_distance(self.zoom) * 0.001;
        self.target.x += delta.x * scale;
        self.target.y += delta.y * scale;
        self.update_position();
    }

    /// Zoom the camera by a delta.
    pub fn zoom_by(&mut self, delta: f32) {
        self.zoom = (self.zoom + delta).clamp(0.0, 22.0);
        self.update_position();
    }

    /// Rotate the camera by a delta in degrees.
    pub fn rotate(&mut self, delta: f32) {
        self.bearing = (self.bearing + delta) % 360.0;
        if self.bearing < 0.0 {
            self.bearing += 360.0;
        }
        self.update_position();
    }

    /// Tilt the camera by a delta in degrees.
    pub fn tilt(&mut self, delta: f32) {
        self.pitch = (self.pitch + delta).clamp(0.0, 60.0);
        self.update_position();
    }

    /// Reset camera to default view.
    pub fn reset(&mut self) {
        self.position = Vec3::new(0.0, 0.0, 1000.0);
        self.target = Vec3::ZERO;
        self.zoom = 0.0;
        self.pitch = 0.0;
        self.bearing = 0.0;
        self.update_position();
    }

    /// Check if the camera is in 2.5D mode.
    pub fn is_2_5d(&self) -> bool {
        self.pitch > 0.0
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// Uniform data for camera/view matrices.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    /// View-projection matrix.
    pub view_proj: [[f32; 4]; 4],
    /// Camera position.
    pub camera_pos: [f32; 4],
    /// Zoom level.
    pub zoom: f32,
    /// Padding for alignment.
    pub _padding: [f32; 3],
}

impl CameraUniform {
    /// Create a new camera uniform from a camera.
    pub fn from_camera(camera: &Camera, aspect_ratio: f32) -> Self {
        Self {
            view_proj: camera.view_projection_matrix(aspect_ratio).to_cols_array_2d(),
            camera_pos: [camera.position.x, camera.position.y, camera.position.z, 1.0],
            zoom: camera.zoom,
            _padding: [0.0; 3],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new();
        assert_eq!(camera.zoom, 0.0);
        assert_eq!(camera.pitch, 0.0);
        assert_eq!(camera.bearing, 0.0);
    }

    #[test]
    fn test_camera_at_position() {
        let camera = Camera::at_position(10.0, 20.0, 5.0);
        assert_eq!(camera.center(), Vec2::new(10.0, 20.0));
        assert_eq!(camera.zoom, 5.0);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera::new();
        camera.zoom_by(5.0);
        assert_eq!(camera.zoom, 5.0);
        camera.zoom_by(-2.0);
        assert_eq!(camera.zoom, 3.0);

        // Test clamping
        camera.zoom_by(100.0);
        assert_eq!(camera.zoom, 22.0);
        camera.zoom_by(-100.0);
        assert_eq!(camera.zoom, 0.0);
    }

    #[test]
    fn test_camera_2_5d_check() {
        let mut camera = Camera::new();
        assert!(!camera.is_2_5d());

        camera.tilt(30.0);
        assert!(camera.is_2_5d());
    }
}
