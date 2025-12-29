//! Tile management system for loading and caching map tiles.

pub mod cache;
pub mod loader;

use std::hash::{Hash, Hasher};

/// Tile coordinate in the standard XYZ tile scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileCoord {
    /// X coordinate (column).
    pub x: u32,
    /// Y coordinate (row).
    pub y: u32,
    /// Zoom level.
    pub z: u32,
}

impl TileCoord {
    /// Create a new tile coordinate.
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    /// Get the parent tile at zoom level z-1.
    pub fn parent(&self) -> Option<TileCoord> {
        if self.z == 0 {
            None
        } else {
            Some(TileCoord {
                x: self.x / 2,
                y: self.y / 2,
                z: self.z - 1,
            })
        }
    }

    /// Get the four child tiles at zoom level z+1.
    pub fn children(&self) -> [TileCoord; 4] {
        let x = self.x * 2;
        let y = self.y * 2;
        let z = self.z + 1;

        [
            TileCoord::new(x, y, z),
            TileCoord::new(x + 1, y, z),
            TileCoord::new(x, y + 1, z),
            TileCoord::new(x + 1, y + 1, z),
        ]
    }

    /// Get neighboring tiles (8 surrounding tiles).
    pub fn neighbors(&self) -> Vec<TileCoord> {
        let mut neighbors = Vec::new();
        let max_tile = 2_u32.pow(self.z);

        for dx in -1..=1_i32 {
            for dy in -1..=1_i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let x = (self.x as i32 + dx) as u32;
                let y = (self.y as i32 + dy) as u32;

                if x < max_tile && y < max_tile {
                    neighbors.push(TileCoord::new(x, y, self.z));
                }
            }
        }

        neighbors
    }

    /// Convert to quadkey string (for tile URLs).
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

    /// Check if this tile is valid at its zoom level.
    pub fn is_valid(&self) -> bool {
        let max_tile = 2_u32.pow(self.z);
        self.x < max_tile && self.y < max_tile
    }
}

impl Hash for TileCoord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
        self.z.hash(state);
    }
}

impl std::fmt::Display for TileCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}", self.z, self.x, self.y)
    }
}

/// Tile data containing the raw image bytes.
#[derive(Debug, Clone)]
pub struct TileData {
    /// Tile coordinate.
    pub coord: TileCoord,
    /// Raw image data (PNG, JPEG, etc.).
    pub data: Vec<u8>,
    /// Timestamp when tile was loaded.
    pub loaded_at: std::time::Instant,
}

impl TileData {
    /// Create new tile data.
    pub fn new(coord: TileCoord, data: Vec<u8>) -> Self {
        Self {
            coord,
            data,
            loaded_at: std::time::Instant::now(),
        }
    }

    /// Get the age of this tile data.
    pub fn age(&self) -> std::time::Duration {
        self.loaded_at.elapsed()
    }

    /// Get the size of the tile data in bytes.
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Tile bounds in geographic coordinates.
#[derive(Debug, Clone, Copy)]
pub struct TileBounds {
    /// West longitude.
    pub west: f64,
    /// South latitude.
    pub south: f64,
    /// East longitude.
    pub east: f64,
    /// North latitude.
    pub north: f64,
}

impl TileBounds {
    /// Calculate bounds for a tile coordinate (Web Mercator).
    pub fn from_tile_coord(coord: &TileCoord) -> Self {
        let n = 2_f64.powi(coord.z as i32);
        let west = coord.x as f64 / n * 360.0 - 180.0;
        let east = (coord.x + 1) as f64 / n * 360.0 - 180.0;

        let north_rad = ((1.0 - coord.y as f64 / n) * std::f64::consts::PI).sinh().atan();
        let south_rad = ((1.0 - (coord.y + 1) as f64 / n) * std::f64::consts::PI)
            .sinh()
            .atan();

        Self {
            west,
            south: south_rad.to_degrees(),
            east,
            north: north_rad.to_degrees(),
        }
    }

    /// Check if a point (lon, lat) is within these bounds.
    pub fn contains(&self, lon: f64, lat: f64) -> bool {
        lon >= self.west && lon <= self.east && lat >= self.south && lat <= self.north
    }
}

/// Tile utilities.
pub struct TileUtils;

impl TileUtils {
    /// Convert geographic coordinates to tile coordinates at a given zoom.
    pub fn coords_to_tile(lon: f64, lat: f64, zoom: u32) -> TileCoord {
        let lat_rad = lat.to_radians();
        let n = 2_f64.powi(zoom as i32);

        let x = ((lon + 180.0) / 360.0 * n).floor() as u32;
        let y = ((1.0 - lat_rad.tan().asinh() / std::f64::consts::PI) / 2.0 * n).floor() as u32;

        TileCoord::new(x, y, zoom)
    }

    /// Get tiles covering a bounding box at a given zoom.
    pub fn tiles_in_bbox(
        west: f64,
        south: f64,
        east: f64,
        north: f64,
        zoom: u32,
    ) -> Vec<TileCoord> {
        let min_tile = Self::coords_to_tile(west, north, zoom);
        let max_tile = Self::coords_to_tile(east, south, zoom);

        let mut tiles = Vec::new();
        for x in min_tile.x..=max_tile.x {
            for y in min_tile.y..=max_tile.y {
                tiles.push(TileCoord::new(x, y, zoom));
            }
        }
        tiles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_coord_creation() {
        let coord = TileCoord::new(1, 2, 3);
        assert_eq!(coord.x, 1);
        assert_eq!(coord.y, 2);
        assert_eq!(coord.z, 3);
    }

    #[test]
    fn test_tile_parent() {
        let coord = TileCoord::new(4, 6, 3);
        let parent = coord.parent().unwrap();
        assert_eq!(parent.x, 2);
        assert_eq!(parent.y, 3);
        assert_eq!(parent.z, 2);

        let root = TileCoord::new(0, 0, 0);
        assert!(root.parent().is_none());
    }

    #[test]
    fn test_tile_children() {
        let coord = TileCoord::new(1, 1, 1);
        let children = coord.children();
        assert_eq!(children.len(), 4);
        assert_eq!(children[0], TileCoord::new(2, 2, 2));
        assert_eq!(children[1], TileCoord::new(3, 2, 2));
        assert_eq!(children[2], TileCoord::new(2, 3, 2));
        assert_eq!(children[3], TileCoord::new(3, 3, 2));
    }

    #[test]
    fn test_tile_quadkey() {
        let coord = TileCoord::new(3, 5, 3);
        let quadkey = coord.to_quadkey();
        assert!(!quadkey.is_empty());
    }

    #[test]
    fn test_tile_validity() {
        let valid = TileCoord::new(0, 0, 0);
        assert!(valid.is_valid());

        let valid = TileCoord::new(3, 3, 2);
        assert!(valid.is_valid());

        let invalid = TileCoord::new(5, 5, 2);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_coords_to_tile() {
        let coord = TileUtils::coords_to_tile(0.0, 0.0, 0);
        assert_eq!(coord.x, 0);
        assert_eq!(coord.y, 0);
        assert_eq!(coord.z, 0);
    }

    #[test]
    fn test_tile_bounds() {
        let coord = TileCoord::new(0, 0, 0);
        let bounds = TileBounds::from_tile_coord(&coord);
        assert!(bounds.contains(0.0, 0.0));
    }
}
