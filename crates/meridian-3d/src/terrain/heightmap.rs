//! Heightmap processing and generation

use crate::{Error, Result};
use glam::{Vec2, Vec3};
use image::{DynamicImage, GrayImage};
use std::path::Path;

/// Heightmap data source
pub trait HeightmapSource: Send + Sync {
    /// Sample elevation at a position
    fn sample(&self, x: f32, z: f32) -> f32;

    /// Get heightmap dimensions
    fn dimensions(&self) -> (usize, usize);

    /// Get elevation range
    fn elevation_range(&self) -> (f32, f32);
}

/// 2D heightmap for terrain elevation data
pub struct Heightmap {
    /// Raw elevation data
    data: Vec<f32>,

    /// Width in pixels/samples
    width: usize,

    /// Height in pixels/samples
    height: usize,

    /// Minimum elevation value
    min_elevation: f32,

    /// Maximum elevation value
    max_elevation: f32,

    /// Scale factor for elevation values
    elevation_scale: f32,
}

impl Heightmap {
    /// Create a heightmap from a file (GeoTIFF, PNG, etc.)
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let img = image::open(path.as_ref())?;

        Self::from_image(img)
    }

    /// Create a heightmap from an image
    pub fn from_image(img: DynamicImage) -> Result<Self> {
        let gray = img.to_luma16();
        let (width, height) = gray.dimensions();

        let mut data = Vec::with_capacity((width * height) as usize);
        let mut min_elevation = f32::INFINITY;
        let mut max_elevation = f32::NEG_INFINITY;

        for pixel in gray.pixels() {
            let value = pixel.0[0] as f32 / u16::MAX as f32;
            data.push(value);
            min_elevation = min_elevation.min(value);
            max_elevation = max_elevation.max(value);
        }

        Ok(Self {
            data,
            width: width as usize,
            height: height as usize,
            min_elevation,
            max_elevation,
            elevation_scale: 1.0,
        })
    }

    /// Create a procedural heightmap using noise
    pub fn procedural(width: usize, height: usize, seed: u64) -> Self {
        let mut data = Vec::with_capacity(width * height);

        // Simple procedural generation using sine waves
        // In a real implementation, use Perlin/Simplex noise
        let mut min_elevation = f32::INFINITY;
        let mut max_elevation = f32::NEG_INFINITY;

        for y in 0..height {
            for x in 0..width {
                let fx = x as f32 / width as f32;
                let fy = y as f32 / height as f32;

                // Multi-octave noise simulation
                let value = Self::noise(fx, fy, seed)
                    + 0.5 * Self::noise(fx * 2.0, fy * 2.0, seed + 1)
                    + 0.25 * Self::noise(fx * 4.0, fy * 4.0, seed + 2);

                let normalized = (value + 1.75) / 3.5; // Normalize to 0..1

                data.push(normalized);
                min_elevation = min_elevation.min(normalized);
                max_elevation = max_elevation.max(normalized);
            }
        }

        Self {
            data,
            width,
            height,
            min_elevation,
            max_elevation,
            elevation_scale: 1.0,
        }
    }

    /// Simple noise function (placeholder for real noise)
    fn noise(x: f32, y: f32, seed: u64) -> f32 {
        let s = seed as f32 * 0.1;
        (x * 10.0 + s).sin() * (y * 10.0 + s).cos()
    }

    /// Create a flat heightmap
    pub fn flat(width: usize, height: usize, elevation: f32) -> Self {
        let data = vec![elevation; width * height];

        Self {
            data,
            width,
            height,
            min_elevation: elevation,
            max_elevation: elevation,
            elevation_scale: 1.0,
        }
    }

    /// Get heightmap width
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get heightmap height
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get minimum elevation
    pub fn min_elevation(&self) -> f32 {
        self.min_elevation * self.elevation_scale
    }

    /// Get maximum elevation
    pub fn max_elevation(&self) -> f32 {
        self.max_elevation * self.elevation_scale
    }

    /// Set elevation scale factor
    pub fn set_elevation_scale(&mut self, scale: f32) {
        self.elevation_scale = scale;
    }

    /// Sample elevation at a position (with bilinear interpolation)
    pub fn sample(&self, x: f32, z: f32) -> f32 {
        // Clamp to heightmap bounds
        let x = x.clamp(0.0, self.width as f32 - 1.0);
        let z = z.clamp(0.0, self.height as f32 - 1.0);

        // Get integer and fractional parts
        let x0 = x.floor() as usize;
        let z0 = z.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let z1 = (z0 + 1).min(self.height - 1);

        let fx = x - x0 as f32;
        let fz = z - z0 as f32;

        // Bilinear interpolation
        let h00 = self.get_pixel(x0, z0);
        let h10 = self.get_pixel(x1, z0);
        let h01 = self.get_pixel(x0, z1);
        let h11 = self.get_pixel(x1, z1);

        let h0 = h00 * (1.0 - fx) + h10 * fx;
        let h1 = h01 * (1.0 - fx) + h11 * fx;

        (h0 * (1.0 - fz) + h1 * fz) * self.elevation_scale
    }

    /// Get elevation at a specific pixel
    fn get_pixel(&self, x: usize, z: usize) -> f32 {
        self.data[z * self.width + x]
    }

    /// Calculate normal at a position
    pub fn calculate_normal(&self, x: f32, z: f32) -> Vec3 {
        let epsilon = 1.0;

        let h_center = self.sample(x, z);
        let h_right = self.sample(x + epsilon, z);
        let h_up = self.sample(x, z + epsilon);

        let tangent_x = Vec3::new(epsilon, h_right - h_center, 0.0);
        let tangent_z = Vec3::new(0.0, h_up - h_center, epsilon);

        tangent_z.cross(tangent_x).normalize()
    }

    /// Get raw data
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Get mutable raw data
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Apply a filter to the heightmap (smoothing, sharpening, etc.)
    pub fn apply_filter(&mut self, filter: HeightmapFilter) {
        match filter {
            HeightmapFilter::Smooth(radius) => self.smooth(radius),
            HeightmapFilter::Erosion(iterations) => self.hydraulic_erosion(iterations),
        }
    }

    /// Smooth the heightmap using box blur
    fn smooth(&mut self, radius: usize) {
        let mut new_data = self.data.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                let mut sum = 0.0;
                let mut count = 0;

                for dy in -(radius as isize)..=radius as isize {
                    for dx in -(radius as isize)..=radius as isize {
                        let nx = (x as isize + dx).clamp(0, self.width as isize - 1) as usize;
                        let ny = (y as isize + dy).clamp(0, self.height as isize - 1) as usize;

                        sum += self.get_pixel(nx, ny);
                        count += 1;
                    }
                }

                new_data[y * self.width + x] = sum / count as f32;
            }
        }

        self.data = new_data;
        self.recalculate_range();
    }

    /// Simple hydraulic erosion simulation
    fn hydraulic_erosion(&mut self, iterations: u32) {
        // Simplified erosion simulation
        for _ in 0..iterations {
            // This would be a full hydraulic erosion algorithm
            // For now, just apply a slight smoothing
            self.smooth(1);
        }
    }

    /// Recalculate min/max elevation
    fn recalculate_range(&mut self) {
        self.min_elevation = self.data.iter().copied().fold(f32::INFINITY, f32::min);
        self.max_elevation = self.data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    }

    /// Export to image
    pub fn to_image(&self) -> GrayImage {
        let mut img = GrayImage::new(self.width as u32, self.height as u32);

        for (i, &value) in self.data.iter().enumerate() {
            let x = (i % self.width) as u32;
            let y = (i / self.width) as u32;

            // Normalize to 0-255 range
            let normalized = ((value - self.min_elevation) / (self.max_elevation - self.min_elevation))
                .clamp(0.0, 1.0);
            let pixel_value = (normalized * 255.0) as u8;

            img.put_pixel(x, y, image::Luma([pixel_value]));
        }

        img
    }
}

impl HeightmapSource for Heightmap {
    fn sample(&self, x: f32, z: f32) -> f32 {
        self.sample(x, z)
    }

    fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn elevation_range(&self) -> (f32, f32) {
        (self.min_elevation(), self.max_elevation())
    }
}

/// Heightmap processing filters
pub enum HeightmapFilter {
    /// Smooth the heightmap
    Smooth(usize),
    /// Apply hydraulic erosion
    Erosion(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_heightmap() {
        let heightmap = Heightmap::flat(100, 100, 10.0);

        assert_eq!(heightmap.width(), 100);
        assert_eq!(heightmap.height(), 100);
        assert_eq!(heightmap.min_elevation(), 10.0);
        assert_eq!(heightmap.max_elevation(), 10.0);
        assert_eq!(heightmap.sample(50.0, 50.0), 10.0);
    }

    #[test]
    fn test_procedural_heightmap() {
        let heightmap = Heightmap::procedural(100, 100, 42);

        assert_eq!(heightmap.width(), 100);
        assert_eq!(heightmap.height(), 100);
        assert!(heightmap.min_elevation() < heightmap.max_elevation());
    }

    #[test]
    fn test_heightmap_sampling() {
        let mut heightmap = Heightmap::flat(10, 10, 0.0);
        heightmap.data_mut()[55] = 10.0; // Set middle pixel

        let sampled = heightmap.sample(5.0, 5.0);
        assert_eq!(sampled, 10.0);
    }

    #[test]
    fn test_normal_calculation() {
        let heightmap = Heightmap::flat(100, 100, 0.0);
        let normal = heightmap.calculate_normal(50.0, 50.0);

        // Flat surface should have upward normal
        assert!((normal.y - 1.0).abs() < 0.1);
    }
}
