//! Tile coordinate systems (TMS, ZXY)

use crate::error::{Error, Result};
use crate::tile::{lon_to_mercator_x, lat_to_mercator_y, mercator_x_to_lon, mercator_y_to_lat};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Tile coordinate in ZXY format (Slippy Map)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoordinate {
    /// Zoom level (0-24)
    pub z: u8,
    /// Column (X coordinate)
    pub x: u32,
    /// Row (Y coordinate, top to bottom)
    pub y: u32,
}

impl TileCoordinate {
    /// Create a new tile coordinate
    pub fn new(z: u8, x: u32, y: u32) -> Self {
        Self { z, x, y }
    }

    /// Create tile coordinate from lon/lat
    pub fn from_lon_lat(lon: f64, lat: f64, z: u8) -> Result<Self> {
        if z > crate::MAX_ZOOM_LEVEL {
            return Err(Error::InvalidZoom {
                zoom: z,
                min: crate::MIN_ZOOM_LEVEL,
                max: crate::MAX_ZOOM_LEVEL,
            });
        }

        let n = 1u32 << z;
        let x = ((lon + 180.0) / 360.0 * n as f64).floor() as u32;
        let lat_rad = lat.to_radians();
        let y = ((1.0 - lat_rad.tan().asinh() / std::f64::consts::PI) / 2.0 * n as f64).floor() as u32;

        Ok(Self::new(z, x.min(n - 1), y.min(n - 1)))
    }

    /// Parse from Z/X/Y string format
    pub fn from_zxy_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 3 {
            return Err(Error::InvalidCoordinate(format!(
                "Invalid ZXY format: {}. Expected Z/X/Y",
                s
            )));
        }

        let z = parts[0].parse().map_err(|_| {
            Error::InvalidCoordinate(format!("Invalid zoom level: {}", parts[0]))
        })?;
        let x = parts[1].parse().map_err(|_| {
            Error::InvalidCoordinate(format!("Invalid X coordinate: {}", parts[1]))
        })?;
        let y = parts[2].parse().map_err(|_| {
            Error::InvalidCoordinate(format!("Invalid Y coordinate: {}", parts[2]))
        })?;

        let coord = Self::new(z, x, y);
        coord.validate()?;
        Ok(coord)
    }

    /// Validate the tile coordinate
    pub fn validate(&self) -> Result<()> {
        if self.z > crate::MAX_ZOOM_LEVEL {
            return Err(Error::InvalidZoom {
                zoom: self.z,
                min: crate::MIN_ZOOM_LEVEL,
                max: crate::MAX_ZOOM_LEVEL,
            });
        }

        let max_coord = 1u32 << self.z;
        if self.x >= max_coord || self.y >= max_coord {
            return Err(Error::InvalidCoordinate(format!(
                "Coordinate ({}, {}) out of range for zoom {}. Max: {}",
                self.x,
                self.y,
                self.z,
                max_coord - 1
            )));
        }

        Ok(())
    }

    /// Convert to TMS (Y inverted) coordinate
    pub fn to_tms(&self) -> Self {
        let max_y = (1u32 << self.z) - 1;
        Self::new(self.z, self.x, max_y - self.y)
    }

    /// Get parent tile at lower zoom
    pub fn parent(&self) -> Option<Self> {
        if self.z == 0 {
            return None;
        }
        Some(Self::new(self.z - 1, self.x / 2, self.y / 2))
    }

    /// Get four child tiles at higher zoom
    pub fn children(&self) -> Option<[Self; 4]> {
        if self.z >= crate::MAX_ZOOM_LEVEL {
            return None;
        }

        let z = self.z + 1;
        let x = self.x * 2;
        let y = self.y * 2;

        Some([
            Self::new(z, x, y),
            Self::new(z, x + 1, y),
            Self::new(z, x, y + 1),
            Self::new(z, x + 1, y + 1),
        ])
    }

    /// Get siblings (including self)
    pub fn siblings(&self) -> [Self; 4] {
        if let Some(parent) = self.parent() {
            parent.children().unwrap()
        } else {
            [*self, *self, *self, *self]
        }
    }

    /// Get the tile's bounding box in lon/lat
    pub fn bounds(&self) -> TileBounds {
        let n = (1u32 << self.z) as f64;

        let west = self.x as f64 / n * 360.0 - 180.0;
        let east = (self.x + 1) as f64 / n * 360.0 - 180.0;

        let north = Self::y_to_lat(self.y, self.z);
        let south = Self::y_to_lat(self.y + 1, self.z);

        TileBounds::new(west, south, east, north)
    }

    /// Helper: Convert Y coordinate to latitude
    fn y_to_lat(y: u32, z: u8) -> f64 {
        let n = (1u32 << z) as f64;
        let lat_rad = ((std::f64::consts::PI * (1.0 - 2.0 * y as f64 / n)).sinh()).atan();
        lat_rad.to_degrees()
    }

    /// Get the tile's center point in lon/lat
    pub fn center(&self) -> (f64, f64) {
        let bounds = self.bounds();
        (
            (bounds.west + bounds.east) / 2.0,
            (bounds.north + bounds.south) / 2.0,
        )
    }

    /// Format as Z/X/Y string
    pub fn to_zxy_string(&self) -> String {
        format!("{}/{}/{}", self.z, self.x, self.y)
    }

    /// Get QuadKey (Microsoft Bing Maps format)
    pub fn to_quadkey(&self) -> String {
        let mut quadkey = String::new();
        for i in (0..self.z).rev() {
            let mut digit = 0;
            let mask = 1 << i;
            if (self.x & mask) != 0 {
                digit += 1;
            }
            if (self.y & mask) != 0 {
                digit += 2;
            }
            quadkey.push_str(&digit.to_string());
        }
        quadkey
    }

    /// Create from QuadKey
    pub fn from_quadkey(quadkey: &str) -> Result<Self> {
        let z = quadkey.len() as u8;
        let mut x = 0u32;
        let mut y = 0u32;

        for (i, c) in quadkey.chars().enumerate() {
            let mask = 1 << (z - 1 - i as u8);
            match c {
                '0' => {}
                '1' => x |= mask,
                '2' => y |= mask,
                '3' => {
                    x |= mask;
                    y |= mask;
                }
                _ => {
                    return Err(Error::InvalidCoordinate(format!(
                        "Invalid quadkey character: {}",
                        c
                    )))
                }
            }
        }

        Ok(Self::new(z, x, y))
    }
}

impl fmt::Display for TileCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.z, self.x, self.y)
    }
}

/// Tile bounds in geographic coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TileBounds {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

impl TileBounds {
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        Self { west, south, east, north }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_creation() {
        let tile = TileCoordinate::new(10, 512, 384);
        assert_eq!(tile.z, 10);
        assert_eq!(tile.x, 512);
        assert_eq!(tile.y, 384);
    }

    #[test]
    fn test_from_lon_lat() {
        let tile = TileCoordinate::from_lon_lat(-122.4194, 37.7749, 10).unwrap();
        assert_eq!(tile.z, 10);
    }

    #[test]
    fn test_parent_child() {
        let tile = TileCoordinate::new(10, 512, 384);
        let parent = tile.parent().unwrap();
        assert_eq!(parent.z, 9);
        assert_eq!(parent.x, 256);
        assert_eq!(parent.y, 192);

        let children = parent.children().unwrap();
        assert!(children.contains(&tile));
    }

    #[test]
    fn test_quadkey() {
        let tile = TileCoordinate::new(3, 3, 5);
        let quadkey = tile.to_quadkey();
        let tile2 = TileCoordinate::from_quadkey(&quadkey).unwrap();
        assert_eq!(tile, tile2);
    }

    #[test]
    fn test_zxy_string() {
        let tile = TileCoordinate::new(10, 512, 384);
        let s = tile.to_zxy_string();
        assert_eq!(s, "10/512/384");

        let tile2 = TileCoordinate::from_zxy_str(&s).unwrap();
        assert_eq!(tile, tile2);
    }

    #[test]
    fn test_tms_conversion() {
        let tile = TileCoordinate::new(10, 512, 384);
        let tms = tile.to_tms();
        let back = tms.to_tms();
        assert_eq!(tile, back);
    }
}
