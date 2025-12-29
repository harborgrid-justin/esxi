//! Tile bounds calculation in various coordinate systems

use crate::error::{Error, Result};
use crate::tile::coordinate::TileCoordinate;
use crate::tile::{lon_to_mercator_x, lat_to_mercator_y, EARTH_CIRCUMFERENCE};
use serde::{Deserialize, Serialize};

/// Tile bounds in geographic coordinates (WGS84)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TileBounds {
    /// Western longitude
    pub west: f64,
    /// Southern latitude
    pub south: f64,
    /// Eastern longitude
    pub east: f64,
    /// Northern latitude
    pub north: f64,
}

impl TileBounds {
    /// Create new tile bounds
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        Self {
            west,
            south,
            east,
            north,
        }
    }

    /// Create from tile coordinate
    pub fn from_tile(tile: &TileCoordinate) -> Self {
        tile.bounds()
    }

    /// Validate bounds
    pub fn validate(&self) -> Result<()> {
        if self.west > self.east {
            return Err(Error::InvalidBounds(format!(
                "West ({}) must be <= East ({})",
                self.west, self.east
            )));
        }
        if self.south > self.north {
            return Err(Error::InvalidBounds(format!(
                "South ({}) must be <= North ({})",
                self.south, self.north
            )));
        }
        if self.west < -180.0 || self.east > 180.0 {
            return Err(Error::InvalidBounds(format!(
                "Longitude out of range: [{}, {}]",
                self.west, self.east
            )));
        }
        if self.south < -85.051129 || self.north > 85.051129 {
            return Err(Error::InvalidBounds(format!(
                "Latitude out of range: [{}, {}]",
                self.south, self.north
            )));
        }
        Ok(())
    }

    /// Get bounds in Web Mercator coordinates
    pub fn to_mercator(&self) -> MercatorBounds {
        MercatorBounds {
            min_x: lon_to_mercator_x(self.west),
            min_y: lat_to_mercator_y(self.south),
            max_x: lon_to_mercator_x(self.east),
            max_y: lat_to_mercator_y(self.north),
        }
    }

    /// Get width in degrees
    pub fn width(&self) -> f64 {
        self.east - self.west
    }

    /// Get height in degrees
    pub fn height(&self) -> f64 {
        self.north - self.south
    }

    /// Get center point
    pub fn center(&self) -> (f64, f64) {
        (
            (self.west + self.east) / 2.0,
            (self.south + self.north) / 2.0,
        )
    }

    /// Check if bounds contain a point
    pub fn contains(&self, lon: f64, lat: f64) -> bool {
        lon >= self.west && lon <= self.east && lat >= self.south && lat <= self.north
    }

    /// Check if bounds intersect another bounds
    pub fn intersects(&self, other: &TileBounds) -> bool {
        !(self.east < other.west
            || self.west > other.east
            || self.north < other.south
            || self.south > other.north)
    }

    /// Get intersection with another bounds
    pub fn intersection(&self, other: &TileBounds) -> Option<TileBounds> {
        if !self.intersects(other) {
            return None;
        }

        Some(TileBounds::new(
            self.west.max(other.west),
            self.south.max(other.south),
            self.east.min(other.east),
            self.north.min(other.north),
        ))
    }

    /// Expand bounds by a margin (in degrees)
    pub fn expand(&self, margin: f64) -> TileBounds {
        TileBounds::new(
            (self.west - margin).max(-180.0),
            (self.south - margin).max(-85.051129),
            (self.east + margin).min(180.0),
            (self.north + margin).min(85.051129),
        )
    }

    /// Convert to array [west, south, east, north]
    pub fn to_array(&self) -> [f64; 4] {
        [self.west, self.south, self.east, self.north]
    }

    /// Create from array [west, south, east, north]
    pub fn from_array(arr: [f64; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl Default for TileBounds {
    fn default() -> Self {
        // World bounds
        Self::new(-180.0, -85.051129, 180.0, 85.051129)
    }
}

/// Tile bounds in Web Mercator coordinates (EPSG:3857)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MercatorBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl MercatorBounds {
    /// Create new mercator bounds
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Create from tile coordinate
    pub fn from_tile(tile: &TileCoordinate) -> Self {
        let tile_size = EARTH_CIRCUMFERENCE / (1 << tile.z) as f64;
        let half = EARTH_CIRCUMFERENCE / 2.0;

        let min_x = tile.x as f64 * tile_size - half;
        let max_x = (tile.x + 1) as f64 * tile_size - half;
        let min_y = half - (tile.y + 1) as f64 * tile_size;
        let max_y = half - tile.y as f64 * tile_size;

        Self::new(min_x, min_y, max_x, max_y)
    }

    /// Get width in meters
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Get height in meters
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Get center point in meters
    pub fn center(&self) -> (f64, f64) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    /// Check if bounds contain a point
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    /// Expand bounds by margin (in meters)
    pub fn expand(&self, margin: f64) -> Self {
        Self::new(
            self.min_x - margin,
            self.min_y - margin,
            self.max_x + margin,
            self.max_y + margin,
        )
    }

    /// Convert to array [min_x, min_y, max_x, max_y]
    pub fn to_array(&self) -> [f64; 4] {
        [self.min_x, self.min_y, self.max_x, self.max_y]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_bounds() {
        let bounds = TileBounds::new(-180.0, -85.0, 180.0, 85.0);
        assert!(bounds.validate().is_ok());
        assert_eq!(bounds.width(), 360.0);
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = TileBounds::new(-10.0, -10.0, 10.0, 10.0);
        assert!(bounds.contains(0.0, 0.0));
        assert!(!bounds.contains(20.0, 0.0));
    }

    #[test]
    fn test_bounds_intersection() {
        let b1 = TileBounds::new(0.0, 0.0, 10.0, 10.0);
        let b2 = TileBounds::new(5.0, 5.0, 15.0, 15.0);

        let intersection = b1.intersection(&b2).unwrap();
        assert_eq!(intersection.west, 5.0);
        assert_eq!(intersection.south, 5.0);
        assert_eq!(intersection.east, 10.0);
        assert_eq!(intersection.north, 10.0);
    }

    #[test]
    fn test_mercator_bounds() {
        let tile = TileCoordinate::new(1, 0, 0);
        let bounds = MercatorBounds::from_tile(&tile);
        assert!(bounds.width() > 0.0);
        assert!(bounds.height() > 0.0);
    }
}
