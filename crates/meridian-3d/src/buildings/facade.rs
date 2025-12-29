//! Building facade texturing and detail

use crate::{Error, Result};
use image::{DynamicImage, RgbaImage};
use glam::{Vec2, Vec3, Vec4};

/// Facade texture for building walls
pub struct FacadeTexture {
    /// Texture image
    texture: DynamicImage,

    /// Texture width in meters
    width_meters: f32,

    /// Texture height in meters
    height_meters: f32,

    /// Pattern type
    pattern: FacadePattern,
}

impl FacadeTexture {
    /// Create a new facade texture
    pub fn new(texture: DynamicImage, width_meters: f32, height_meters: f32) -> Self {
        Self {
            texture,
            width_meters,
            height_meters,
            pattern: FacadePattern::Uniform,
        }
    }

    /// Create a procedural brick facade
    pub fn brick(width: u32, height: u32) -> Self {
        let texture = Self::generate_brick_texture(width, height);
        Self::new(texture, 3.0, 3.0) // 3m x 3m tile
    }

    /// Create a procedural glass facade
    pub fn glass(width: u32, height: u32) -> Self {
        let texture = Self::generate_glass_texture(width, height);
        Self::new(texture, 5.0, 3.0) // 5m wide, 3m per floor
    }

    /// Create a concrete facade
    pub fn concrete(width: u32, height: u32) -> Self {
        let texture = Self::generate_concrete_texture(width, height);
        Self::new(texture, 4.0, 4.0)
    }

    /// Create a stone facade
    pub fn stone(width: u32, height: u32) -> Self {
        let texture = Self::generate_stone_texture(width, height);
        Self::new(texture, 2.0, 2.0)
    }

    /// Generate a brick texture
    fn generate_brick_texture(width: u32, height: u32) -> DynamicImage {
        let mut img = RgbaImage::new(width, height);

        let brick_width = width / 8;
        let brick_height = height / 16;
        let mortar_width = width / 64;

        for y in 0..height {
            for x in 0..width {
                let brick_row = y / brick_height;
                let offset = if brick_row % 2 == 0 { 0 } else { brick_width / 2 };

                let local_x = (x + offset) % (brick_width + mortar_width);
                let local_y = y % (brick_height + mortar_width);

                let is_mortar = local_x < mortar_width || local_y < mortar_width;

                let color = if is_mortar {
                    // Mortar - light gray
                    [200, 200, 200, 255]
                } else {
                    // Brick - reddish brown with variation
                    let variation = ((x + y) % 20) as u8;
                    [150 + variation, 70 + variation / 2, 50, 255]
                };

                img.put_pixel(x, y, image::Rgba(color));
            }
        }

        DynamicImage::ImageRgba8(img)
    }

    /// Generate a glass facade texture
    fn generate_glass_texture(width: u32, height: u32) -> DynamicImage {
        let mut img = RgbaImage::new(width, height);

        let window_width = width / 4;
        let window_height = height / 3;
        let frame_width = width / 32;

        for y in 0..height {
            for x in 0..width {
                let local_x = x % window_width;
                let local_y = y % window_height;

                let is_frame = local_x < frame_width
                    || local_x > window_width - frame_width
                    || local_y < frame_width
                    || local_y > window_height - frame_width;

                let color = if is_frame {
                    // Window frame - dark gray
                    [60, 60, 60, 255]
                } else {
                    // Glass - light blue with reflection
                    let reflection = ((x + y) % 40) as u8;
                    [150 + reflection, 180 + reflection, 220 + reflection, 255]
                };

                img.put_pixel(x, y, image::Rgba(color));
            }
        }

        DynamicImage::ImageRgba8(img)
    }

    /// Generate a concrete texture
    fn generate_concrete_texture(width: u32, height: u32) -> DynamicImage {
        let mut img = RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                // Concrete - gray with noise
                let noise = ((x as f32 * 0.1).sin() + (y as f32 * 0.1).cos()) * 10.0;
                let base = 140.0 + noise;

                let gray = base.clamp(0.0, 255.0) as u8;
                img.put_pixel(x, y, image::Rgba([gray, gray, gray, 255]));
            }
        }

        DynamicImage::ImageRgba8(img)
    }

    /// Generate a stone texture
    fn generate_stone_texture(width: u32, height: u32) -> DynamicImage {
        let mut img = RgbaImage::new(width, height);

        let stone_size = width / 4;

        for y in 0..height {
            for x in 0..width {
                let stone_x = x / stone_size;
                let stone_y = y / stone_size;

                // Stone blocks with variation
                let variation = ((stone_x + stone_y * 7) % 30) as u8;
                let base = 120 + variation;

                // Add some texture within each stone
                let texture = ((x % stone_size) as f32 * 0.2).sin() * 10.0;
                let final_value = (base as f32 + texture).clamp(0.0, 255.0) as u8;

                img.put_pixel(x, y, image::Rgba([final_value, final_value - 10, final_value - 20, 255]));
            }
        }

        DynamicImage::ImageRgba8(img)
    }

    /// Get the texture image
    pub fn texture(&self) -> &DynamicImage {
        &self.texture
    }

    /// Get width in meters
    pub fn width_meters(&self) -> f32 {
        self.width_meters
    }

    /// Get height in meters
    pub fn height_meters(&self) -> f32 {
        self.height_meters
    }

    /// Set the facade pattern
    pub fn with_pattern(mut self, pattern: FacadePattern) -> Self {
        self.pattern = pattern;
        self
    }

    /// Get the pattern
    pub fn pattern(&self) -> FacadePattern {
        self.pattern
    }
}

/// Facade pattern/layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FacadePattern {
    /// Uniform texture across entire facade
    Uniform,

    /// Regular grid pattern (windows)
    Grid {
        /// Columns per unit width
        columns: u32,
        /// Rows per unit height
        rows: u32,
    },

    /// Alternating pattern
    Alternating,

    /// Randomized pattern
    Random,
}

/// Facade detail generator
pub struct FacadeDetailGenerator;

impl FacadeDetailGenerator {
    /// Generate window cutouts for a facade
    pub fn generate_windows(
        wall_width: f32,
        wall_height: f32,
        window_width: f32,
        window_height: f32,
        spacing: f32,
    ) -> Vec<WindowRect> {
        let mut windows = Vec::new();

        let floor_count = (wall_height / (window_height + spacing)).floor() as u32;
        let window_count_per_floor = (wall_width / (window_width + spacing)).floor() as u32;

        for floor in 0..floor_count {
            let y = spacing + floor as f32 * (window_height + spacing);

            for window_idx in 0..window_count_per_floor {
                let x = spacing + window_idx as f32 * (window_width + spacing);

                windows.push(WindowRect {
                    x,
                    y,
                    width: window_width,
                    height: window_height,
                });
            }
        }

        windows
    }

    /// Generate balconies
    pub fn generate_balconies(
        wall_width: f32,
        wall_height: f32,
        balcony_width: f32,
        balcony_depth: f32,
        floor_height: f32,
    ) -> Vec<BalconyRect> {
        let mut balconies = Vec::new();

        let floor_count = (wall_height / floor_height).floor() as u32;

        // Add balconies to every other floor
        for floor in (1..floor_count).step_by(2) {
            let y = floor as f32 * floor_height;
            let x = (wall_width - balcony_width) / 2.0;

            balconies.push(BalconyRect {
                x,
                y,
                width: balcony_width,
                depth: balcony_depth,
            });
        }

        balconies
    }

    /// Generate door openings
    pub fn generate_doors(
        wall_width: f32,
        door_width: f32,
        door_height: f32,
    ) -> Vec<DoorRect> {
        vec![DoorRect {
            x: (wall_width - door_width) / 2.0,
            y: 0.0,
            width: door_width,
            height: door_height,
        }]
    }
}

/// Window rectangle definition
#[derive(Debug, Clone, Copy)]
pub struct WindowRect {
    /// X position on facade
    pub x: f32,
    /// Y position on facade
    pub y: f32,
    /// Window width
    pub width: f32,
    /// Window height
    pub height: f32,
}

/// Balcony rectangle definition
#[derive(Debug, Clone, Copy)]
pub struct BalconyRect {
    /// X position on facade
    pub x: f32,
    /// Y position on facade
    pub y: f32,
    /// Balcony width
    pub width: f32,
    /// Balcony depth (protrusion)
    pub depth: f32,
}

/// Door rectangle definition
#[derive(Debug, Clone, Copy)]
pub struct DoorRect {
    /// X position on facade
    pub x: f32,
    /// Y position on facade
    pub y: f32,
    /// Door width
    pub width: f32,
    /// Door height
    pub height: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facade_textures() {
        let brick = FacadeTexture::brick(256, 256);
        assert_eq!(brick.texture().width(), 256);

        let glass = FacadeTexture::glass(256, 256);
        assert_eq!(glass.texture().width(), 256);

        let concrete = FacadeTexture::concrete(256, 256);
        assert_eq!(concrete.texture().width(), 256);

        let stone = FacadeTexture::stone(256, 256);
        assert_eq!(stone.texture().width(), 256);
    }

    #[test]
    fn test_window_generation() {
        let windows = FacadeDetailGenerator::generate_windows(
            20.0, // wall width
            30.0, // wall height
            2.0,  // window width
            2.5,  // window height
            0.5,  // spacing
        );

        assert!(windows.len() > 0);

        // Check that windows are within bounds
        for window in &windows {
            assert!(window.x >= 0.0 && window.x + window.width <= 20.0);
            assert!(window.y >= 0.0 && window.y + window.height <= 30.0);
        }
    }

    #[test]
    fn test_balcony_generation() {
        let balconies = FacadeDetailGenerator::generate_balconies(
            15.0, // wall width
            30.0, // wall height
            5.0,  // balcony width
            1.5,  // balcony depth
            3.0,  // floor height
        );

        assert!(balconies.len() > 0);
    }

    #[test]
    fn test_door_generation() {
        let doors = FacadeDetailGenerator::generate_doors(
            10.0, // wall width
            2.0,  // door width
            2.5,  // door height
        );

        assert_eq!(doors.len(), 1);
        assert!(doors[0].x >= 0.0);
    }

    #[test]
    fn test_facade_pattern() {
        let facade = FacadeTexture::brick(256, 256)
            .with_pattern(FacadePattern::Grid { columns: 4, rows: 8 });

        assert_eq!(
            facade.pattern(),
            FacadePattern::Grid { columns: 4, rows: 8 }
        );
    }
}
