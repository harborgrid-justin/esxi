//! Tile coordinate system and utilities
//!
//! Supports standard web mercator tiling schemes (XYZ and TMS)

use crate::error::{RenderError, RenderResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum supported zoom level
pub const MAX_ZOOM: u8 = 22;

/// Standard tile size in pixels
pub const TILE_SIZE: u32 = 256;

/// Web Mercator EPSG code
pub const WEB_MERCATOR_EPSG: u32 = 3857;

/// Tile coordinate in a slippy map tileset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCoord {
    /// Zoom level (0-22)
    pub z: u8,
    /// X coordinate (column)
    pub x: u32,
    /// Y coordinate (row)
    pub y: u32,
}

impl TileCoord {
    /// Create a new tile coordinate
    pub fn new(z: u8, x: u32, y: u32) -> RenderResult<Self> {
        if z > MAX_ZOOM {
            return Err(RenderError::InvalidZoomLevel(z));
        }

        let max_coord = 1 << z; // 2^z
        if x >= max_coord || y >= max_coord {
            return Err(RenderError::InvalidTileCoordinate { z, x, y });
        }

        Ok(TileCoord { z, x, y })
    }

    /// Create unchecked tile coordinate (use with caution)
    pub fn new_unchecked(z: u8, x: u32, y: u32) -> Self {
        TileCoord { z, x, y }
    }

    /// Get the bounding box for this tile in Web Mercator coordinates
    pub fn bounds(&self) -> TileBounds {
        TileBounds::from_tile(*self)
    }

    /// Convert from TMS (Tile Map Service) Y coordinate to XYZ
    pub fn from_tms(z: u8, x: u32, y: u32) -> RenderResult<Self> {
        let max_coord = (1 << z) - 1;
        let xyz_y = max_coord - y;
        Self::new(z, x, xyz_y)
    }

    /// Convert to TMS Y coordinate
    pub fn to_tms_y(&self) -> u32 {
        let max_coord = (1 << self.z) - 1;
        max_coord - self.y
    }

    /// Get the parent tile at zoom level z-1
    pub fn parent(&self) -> Option<Self> {
        if self.z == 0 {
            return None;
        }
        Some(TileCoord {
            z: self.z - 1,
            x: self.x / 2,
            y: self.y / 2,
        })
    }

    /// Get the four child tiles at zoom level z+1
    pub fn children(&self) -> RenderResult<[TileCoord; 4]> {
        if self.z >= MAX_ZOOM {
            return Err(RenderError::InvalidZoomLevel(self.z + 1));
        }

        let z = self.z + 1;
        let x = self.x * 2;
        let y = self.y * 2;

        Ok([
            TileCoord::new_unchecked(z, x, y),
            TileCoord::new_unchecked(z, x + 1, y),
            TileCoord::new_unchecked(z, x, y + 1),
            TileCoord::new_unchecked(z, x + 1, y + 1),
        ])
    }

    /// Get adjacent tiles (N, E, S, W)
    pub fn neighbors(&self) -> Vec<TileCoord> {
        let max_coord = 1 << self.z;
        let mut neighbors = Vec::with_capacity(4);

        // North
        if self.y > 0 {
            neighbors.push(TileCoord::new_unchecked(self.z, self.x, self.y - 1));
        }

        // East
        if self.x + 1 < max_coord {
            neighbors.push(TileCoord::new_unchecked(self.z, self.x + 1, self.y));
        }

        // South
        if self.y + 1 < max_coord {
            neighbors.push(TileCoord::new_unchecked(self.z, self.x, self.y + 1));
        }

        // West
        if self.x > 0 {
            neighbors.push(TileCoord::new_unchecked(self.z, self.x - 1, self.y));
        }

        neighbors
    }

    /// Create a tile coordinate from longitude, latitude, and zoom
    pub fn from_lon_lat(lon: f64, lat: f64, zoom: u8) -> RenderResult<Self> {
        if zoom > MAX_ZOOM {
            return Err(RenderError::InvalidZoomLevel(zoom));
        }

        let n = f64::from(1 << zoom);
        let x = ((lon + 180.0) / 360.0 * n).floor() as u32;
        let lat_rad = lat.to_radians();
        let y = ((1.0 - lat_rad.tan().asinh() / std::f64::consts::PI) / 2.0 * n).floor() as u32;

        Self::new(zoom, x, y)
    }
}

impl fmt::Display for TileCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.z, self.x, self.y)
    }
}

/// Bounding box for a tile in Web Mercator coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TileBounds {
    /// Minimum X (west)
    pub min_x: f64,
    /// Minimum Y (south)
    pub min_y: f64,
    /// Maximum X (east)
    pub max_x: f64,
    /// Maximum Y (north)
    pub max_y: f64,
}

impl TileBounds {
    /// Web Mercator extent
    const MAX_EXTENT: f64 = 20037508.342789244;

    /// Create bounds from a tile coordinate
    pub fn from_tile(tile: TileCoord) -> Self {
        let n = f64::from(1 << tile.z);
        let tile_size = (Self::MAX_EXTENT * 2.0) / n;

        let min_x = -Self::MAX_EXTENT + f64::from(tile.x) * tile_size;
        let max_y = Self::MAX_EXTENT - f64::from(tile.y) * tile_size;
        let max_x = min_x + tile_size;
        let min_y = max_y - tile_size;

        TileBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    /// Get the width of the bounds
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Get the height of the bounds
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Get the center point
    pub fn center(&self) -> (f64, f64) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    /// Check if a point is within the bounds
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    /// Check if bounds intersect
    pub fn intersects(&self, other: &TileBounds) -> bool {
        self.min_x <= other.max_x
            && self.max_x >= other.min_x
            && self.min_y <= other.max_y
            && self.max_y >= other.min_y
    }

    /// Expand bounds by a buffer distance
    pub fn buffer(&self, distance: f64) -> Self {
        TileBounds {
            min_x: self.min_x - distance,
            min_y: self.min_y - distance,
            max_x: self.max_x + distance,
            max_y: self.max_y + distance,
        }
    }
}

/// Grid of tiles covering a bounding box
pub struct TileGrid {
    zoom: u8,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
}

impl TileGrid {
    /// Create a tile grid covering a geographic bounding box
    pub fn from_bounds(bounds: &TileBounds, zoom: u8) -> RenderResult<Self> {
        if zoom > MAX_ZOOM {
            return Err(RenderError::InvalidZoomLevel(zoom));
        }

        let n = 1 << zoom;
        let extent = TileBounds::MAX_EXTENT * 2.0;
        let tile_size = extent / f64::from(n);

        let min_x = ((bounds.min_x + TileBounds::MAX_EXTENT) / tile_size)
            .floor()
            .max(0.0) as u32;
        let max_x = ((bounds.max_x + TileBounds::MAX_EXTENT) / tile_size)
            .ceil()
            .min(f64::from(n)) as u32;
        let min_y = ((TileBounds::MAX_EXTENT - bounds.max_y) / tile_size)
            .floor()
            .max(0.0) as u32;
        let max_y = ((TileBounds::MAX_EXTENT - bounds.min_y) / tile_size)
            .ceil()
            .min(f64::from(n)) as u32;

        Ok(TileGrid {
            zoom,
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    /// Get all tiles in the grid
    pub fn tiles(&self) -> Vec<TileCoord> {
        let mut tiles = Vec::new();
        for y in self.min_y..self.max_y {
            for x in self.min_x..self.max_x {
                tiles.push(TileCoord::new_unchecked(self.zoom, x, y));
            }
        }
        tiles
    }

    /// Get the number of tiles in the grid
    pub fn count(&self) -> usize {
        ((self.max_x - self.min_x) * (self.max_y - self.min_y)) as usize
    }
}

impl Iterator for TileGrid {
    type Item = TileCoord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.min_y >= self.max_y {
            return None;
        }

        let tile = TileCoord::new_unchecked(self.zoom, self.min_x, self.min_y);

        self.min_x += 1;
        if self.min_x >= self.max_x {
            self.min_x = 0;
            self.min_y += 1;
        }

        Some(tile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_coord_creation() {
        let tile = TileCoord::new(10, 512, 384).unwrap();
        assert_eq!(tile.z, 10);
        assert_eq!(tile.x, 512);
        assert_eq!(tile.y, 384);
    }

    #[test]
    fn test_invalid_zoom() {
        assert!(TileCoord::new(23, 0, 0).is_err());
    }

    #[test]
    fn test_invalid_coords() {
        assert!(TileCoord::new(10, 1024, 0).is_err());
        assert!(TileCoord::new(10, 0, 1024).is_err());
    }

    #[test]
    fn test_tms_conversion() {
        let tile = TileCoord::from_tms(10, 512, 639).unwrap();
        assert_eq!(tile.y, 384); // 1023 - 639 = 384
        assert_eq!(tile.to_tms_y(), 639);
    }

    #[test]
    fn test_parent() {
        let tile = TileCoord::new(10, 512, 384).unwrap();
        let parent = tile.parent().unwrap();
        assert_eq!(parent.z, 9);
        assert_eq!(parent.x, 256);
        assert_eq!(parent.y, 192);
    }

    #[test]
    fn test_children() {
        let tile = TileCoord::new(10, 512, 384).unwrap();
        let children = tile.children().unwrap();
        assert_eq!(children.len(), 4);
        assert_eq!(children[0], TileCoord::new_unchecked(11, 1024, 768));
        assert_eq!(children[1], TileCoord::new_unchecked(11, 1025, 768));
        assert_eq!(children[2], TileCoord::new_unchecked(11, 1024, 769));
        assert_eq!(children[3], TileCoord::new_unchecked(11, 1025, 769));
    }

    #[test]
    fn test_tile_bounds() {
        let tile = TileCoord::new(0, 0, 0).unwrap();
        let bounds = tile.bounds();
        assert_eq!(bounds.min_x, -TileBounds::MAX_EXTENT);
        assert_eq!(bounds.max_x, TileBounds::MAX_EXTENT);
        assert_eq!(bounds.min_y, -TileBounds::MAX_EXTENT);
        assert_eq!(bounds.max_y, TileBounds::MAX_EXTENT);
    }

    #[test]
    fn test_from_lon_lat() {
        let tile = TileCoord::from_lon_lat(0.0, 0.0, 0).unwrap();
        assert_eq!(tile.z, 0);
        assert_eq!(tile.x, 0);
        assert_eq!(tile.y, 0);
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = TileBounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 100.0,
            max_y: 100.0,
        };
        assert!(bounds.contains(50.0, 50.0));
        assert!(!bounds.contains(150.0, 50.0));
    }

    #[test]
    fn test_bounds_intersects() {
        let bounds1 = TileBounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 100.0,
            max_y: 100.0,
        };
        let bounds2 = TileBounds {
            min_x: 50.0,
            min_y: 50.0,
            max_x: 150.0,
            max_y: 150.0,
        };
        assert!(bounds1.intersects(&bounds2));
    }
}
