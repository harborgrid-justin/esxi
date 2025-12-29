//! Tile extent management for MVT encoding

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// Tile extent defines the coordinate space for features within a tile
///
/// The MVT specification uses a coordinate system where features are encoded
/// as integers within a defined extent (typically 4096x4096).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileExtent {
    /// Extent value (e.g., 4096 for MVT standard)
    pub extent: u32,
}

impl TileExtent {
    /// Create a new tile extent
    pub fn new(extent: u32) -> Self {
        Self { extent }
    }

    /// Get the default MVT extent (4096)
    pub fn default_mvt() -> Self {
        Self::new(crate::DEFAULT_EXTENT)
    }

    /// Convert geographic coordinate to tile coordinate
    pub fn geo_to_tile(&self, value: f64, min: f64, max: f64) -> i32 {
        let range = max - min;
        if range == 0.0 {
            return 0;
        }
        let normalized = (value - min) / range;
        (normalized * self.extent as f64).round() as i32
    }

    /// Convert tile coordinate to geographic coordinate
    pub fn tile_to_geo(&self, value: i32, min: f64, max: f64) -> f64 {
        let range = max - min;
        let normalized = value as f64 / self.extent as f64;
        min + normalized * range
    }

    /// Get the resolution (units per pixel) for this extent
    pub fn resolution(&self, tile_size_meters: f64) -> f64 {
        tile_size_meters / self.extent as f64
    }

    /// Validate a coordinate is within the extent
    pub fn validate_coordinate(&self, coord: i32) -> Result<()> {
        let max = self.extent as i32;
        if coord < 0 || coord > max {
            return Err(Error::InvalidCoordinate(format!(
                "Coordinate {} out of extent range [0, {}]",
                coord, max
            )));
        }
        Ok(())
    }

    /// Clamp a coordinate to the extent
    pub fn clamp_coordinate(&self, coord: i32) -> i32 {
        coord.max(0).min(self.extent as i32)
    }
}

impl Default for TileExtent {
    fn default() -> Self {
        Self::default_mvt()
    }
}

/// Helper for converting between coordinate systems
pub struct ExtentConverter {
    extent: TileExtent,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
    scale_x: f64,
    scale_y: f64,
}

impl ExtentConverter {
    /// Create a new extent converter
    pub fn new(extent: TileExtent, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        let range_x = max_x - min_x;
        let range_y = max_y - min_y;

        let scale_x = if range_x > 0.0 {
            extent.extent as f64 / range_x
        } else {
            1.0
        };

        let scale_y = if range_y > 0.0 {
            extent.extent as f64 / range_y
        } else {
            1.0
        };

        Self {
            extent,
            min_x,
            min_y,
            max_x,
            max_y,
            scale_x,
            scale_y,
        }
    }

    /// Convert geographic X to tile coordinate
    pub fn x_to_tile(&self, x: f64) -> i32 {
        ((x - self.min_x) * self.scale_x).round() as i32
    }

    /// Convert geographic Y to tile coordinate
    pub fn y_to_tile(&self, y: f64) -> i32 {
        ((y - self.min_y) * self.scale_y).round() as i32
    }

    /// Convert point (x, y) to tile coordinates
    pub fn point_to_tile(&self, x: f64, y: f64) -> (i32, i32) {
        (self.x_to_tile(x), self.y_to_tile(y))
    }

    /// Convert tile X to geographic coordinate
    pub fn x_to_geo(&self, x: i32) -> f64 {
        self.min_x + (x as f64 / self.scale_x)
    }

    /// Convert tile Y to geographic coordinate
    pub fn y_to_geo(&self, y: i32) -> f64 {
        self.min_y + (y as f64 / self.scale_y)
    }

    /// Convert tile point to geographic coordinates
    pub fn point_to_geo(&self, x: i32, y: i32) -> (f64, f64) {
        (self.x_to_geo(x), self.y_to_geo(y))
    }

    /// Get the extent
    pub fn extent(&self) -> &TileExtent {
        &self.extent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_extent() {
        let extent = TileExtent::new(4096);
        assert_eq!(extent.extent, 4096);

        let coord = extent.geo_to_tile(0.5, 0.0, 1.0);
        assert_eq!(coord, 2048);

        let geo = extent.tile_to_geo(2048, 0.0, 1.0);
        assert!((geo - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_extent_converter() {
        let extent = TileExtent::new(4096);
        let converter = ExtentConverter::new(extent, 0.0, 0.0, 100.0, 100.0);

        let (x, y) = converter.point_to_tile(50.0, 50.0);
        assert_eq!(x, 2048);
        assert_eq!(y, 2048);

        let (x2, y2) = converter.point_to_geo(x, y);
        assert!((x2 - 50.0).abs() < 1e-6);
        assert!((y2 - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_coordinate() {
        let extent = TileExtent::new(4096);
        assert_eq!(extent.clamp_coordinate(-100), 0);
        assert_eq!(extent.clamp_coordinate(5000), 4096);
        assert_eq!(extent.clamp_coordinate(2048), 2048);
    }
}
