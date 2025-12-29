//! Seeding strategies for tile generation

use crate::error::{Error, Result};
use crate::tile::bounds::TileBounds;
use crate::tile::coordinate::TileCoordinate;

/// Trait for seeding strategies
pub trait SeedingStrategy {
    /// Get the list of tiles to seed
    fn get_tiles(&self) -> Result<Vec<TileCoordinate>>;
}

/// Seed all tiles within a zoom range
pub struct ZoomRangeSeedingStrategy {
    min_zoom: u8,
    max_zoom: u8,
}

impl ZoomRangeSeedingStrategy {
    /// Create a new zoom range strategy
    pub fn new(min_zoom: u8, max_zoom: u8) -> Self {
        Self { min_zoom, max_zoom }
    }
}

impl SeedingStrategy for ZoomRangeSeedingStrategy {
    fn get_tiles(&self) -> Result<Vec<TileCoordinate>> {
        let mut tiles = Vec::new();

        for z in self.min_zoom..=self.max_zoom {
            let max_coord = 1u32 << z;
            for x in 0..max_coord {
                for y in 0..max_coord {
                    tiles.push(TileCoordinate::new(z, x, y));
                }
            }
        }

        Ok(tiles)
    }
}

/// Seed tiles within geographic bounds
pub struct BoundsSeedingStrategy {
    bounds: TileBounds,
    min_zoom: u8,
    max_zoom: u8,
}

impl BoundsSeedingStrategy {
    /// Create a new bounds-based strategy
    pub fn new(bounds: TileBounds, min_zoom: u8, max_zoom: u8) -> Self {
        Self {
            bounds,
            min_zoom,
            max_zoom,
        }
    }
}

impl SeedingStrategy for BoundsSeedingStrategy {
    fn get_tiles(&self) -> Result<Vec<TileCoordinate>> {
        let mut tiles = Vec::new();

        for z in self.min_zoom..=self.max_zoom {
            // Calculate tile coordinates for bounds
            let min_tile = TileCoordinate::from_lon_lat(
                self.bounds.west,
                self.bounds.north,
                z,
            )?;

            let max_tile = TileCoordinate::from_lon_lat(
                self.bounds.east,
                self.bounds.south,
                z,
            )?;

            // Iterate over tile range
            for x in min_tile.x..=max_tile.x {
                for y in min_tile.y..=max_tile.y {
                    tiles.push(TileCoordinate::new(z, x, y));
                }
            }
        }

        Ok(tiles)
    }
}

/// Seed specific tiles from a list
pub struct ListSeedingStrategy {
    tiles: Vec<TileCoordinate>,
}

impl ListSeedingStrategy {
    /// Create from a list of tiles
    pub fn new(tiles: Vec<TileCoordinate>) -> Self {
        Self { tiles }
    }
}

impl SeedingStrategy for ListSeedingStrategy {
    fn get_tiles(&self) -> Result<Vec<TileCoordinate>> {
        Ok(self.tiles.clone())
    }
}

/// Seed tiles in a pyramid (parent tiles at lower zooms)
pub struct PyramidSeedingStrategy {
    base_tiles: Vec<TileCoordinate>,
    min_zoom: u8,
}

impl PyramidSeedingStrategy {
    /// Create a new pyramid strategy
    pub fn new(base_tiles: Vec<TileCoordinate>, min_zoom: u8) -> Self {
        Self {
            base_tiles,
            min_zoom,
        }
    }
}

impl SeedingStrategy for PyramidSeedingStrategy {
    fn get_tiles(&self) -> Result<Vec<TileCoordinate>> {
        let mut tiles = Vec::new();
        let mut current_level = self.base_tiles.clone();

        // Add base tiles
        tiles.extend(current_level.clone());

        // Walk up the pyramid
        while !current_level.is_empty() && current_level[0].z > self.min_zoom {
            let mut next_level = Vec::new();

            for tile in &current_level {
                if let Some(parent) = tile.parent() {
                    // Avoid duplicates
                    if !next_level.contains(&parent) {
                        next_level.push(parent);
                    }
                }
            }

            tiles.extend(next_level.clone());
            current_level = next_level;
        }

        Ok(tiles)
    }
}

/// Seed tiles along a route/path
pub struct RouteSeedingStrategy {
    points: Vec<(f64, f64)>, // lon, lat pairs
    zoom_levels: Vec<u8>,
    buffer: u32, // Buffer in tiles
}

impl RouteSeedingStrategy {
    /// Create a new route strategy
    pub fn new(points: Vec<(f64, f64)>, zoom_levels: Vec<u8>, buffer: u32) -> Self {
        Self {
            points,
            zoom_levels,
            buffer,
        }
    }
}

impl SeedingStrategy for RouteSeedingStrategy {
    fn get_tiles(&self) -> Result<Vec<TileCoordinate>> {
        let mut tiles = Vec::new();

        for &zoom in &self.zoom_levels {
            for &(lon, lat) in &self.points {
                let center = TileCoordinate::from_lon_lat(lon, lat, zoom)?;

                // Add tiles in buffer radius
                for dx in -(self.buffer as i32)..=(self.buffer as i32) {
                    for dy in -(self.buffer as i32)..=(self.buffer as i32) {
                        let x = (center.x as i32 + dx).max(0) as u32;
                        let y = (center.y as i32 + dy).max(0) as u32;

                        let tile = TileCoordinate::new(zoom, x, y);
                        if tile.validate().is_ok() && !tiles.contains(&tile) {
                            tiles.push(tile);
                        }
                    }
                }
            }
        }

        Ok(tiles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_range_strategy() {
        let strategy = ZoomRangeSeedingStrategy::new(0, 2);
        let tiles = strategy.get_tiles().unwrap();

        // z=0: 1 tile, z=1: 4 tiles, z=2: 16 tiles = 21 total
        assert_eq!(tiles.len(), 21);
    }

    #[test]
    fn test_bounds_strategy() {
        let bounds = TileBounds::new(-10.0, -10.0, 10.0, 10.0);
        let strategy = BoundsSeedingStrategy::new(bounds, 1, 1);
        let tiles = strategy.get_tiles().unwrap();

        assert!(!tiles.is_empty());
        assert!(tiles.len() <= 4); // At most 4 tiles at zoom 1
    }

    #[test]
    fn test_list_strategy() {
        let list = vec![
            TileCoordinate::new(10, 512, 384),
            TileCoordinate::new(10, 513, 384),
        ];

        let strategy = ListSeedingStrategy::new(list.clone());
        let tiles = strategy.get_tiles().unwrap();

        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles, list);
    }

    #[test]
    fn test_pyramid_strategy() {
        let base = vec![TileCoordinate::new(5, 16, 16)];
        let strategy = PyramidSeedingStrategy::new(base, 0);
        let tiles = strategy.get_tiles().unwrap();

        // Should include tiles at z=5, 4, 3, 2, 1, 0
        assert!(tiles.len() >= 6);
        assert!(tiles.iter().any(|t| t.z == 0));
    }

    #[test]
    fn test_route_strategy() {
        let points = vec![(-122.4194, 37.7749), (-122.4, 37.78)];
        let zooms = vec![10];
        let strategy = RouteSeedingStrategy::new(points, zooms, 1);
        let tiles = strategy.get_tiles().unwrap();

        assert!(!tiles.is_empty());
    }
}
