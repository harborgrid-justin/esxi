//! Overzoom handling for tiles

use crate::error::{Error, Result};
use crate::tile::coordinate::TileCoordinate;

/// Overzoom handler
pub struct OverzoomHandler {
    max_overzoom: u8,
}

impl OverzoomHandler {
    /// Create a new overzoom handler
    pub fn new(max_overzoom: u8) -> Self {
        Self { max_overzoom }
    }

    /// Check if a tile coordinate requires overzooming
    pub fn requires_overzoom(&self, tile: TileCoordinate, max_source_zoom: u8) -> bool {
        tile.z > max_source_zoom
    }

    /// Get the parent tile to use for overzooming
    pub fn get_source_tile(
        &self,
        tile: TileCoordinate,
        max_source_zoom: u8,
    ) -> Result<TileCoordinate> {
        if tile.z <= max_source_zoom {
            return Ok(tile);
        }

        let overzoom_levels = tile.z - max_source_zoom;
        if overzoom_levels > self.max_overzoom {
            return Err(Error::InvalidZoom {
                zoom: tile.z,
                min: 0,
                max: max_source_zoom + self.max_overzoom,
            });
        }

        // Walk up the tile tree to max_source_zoom
        let mut current = tile;
        for _ in 0..overzoom_levels {
            current = current
                .parent()
                .ok_or_else(|| Error::InvalidCoordinate("Cannot find parent tile".to_string()))?;
        }

        Ok(current)
    }

    /// Calculate the scaling factor for overzoomed tiles
    pub fn scale_factor(&self, tile: TileCoordinate, source_tile: TileCoordinate) -> f64 {
        let zoom_diff = tile.z - source_tile.z;
        2.0_f64.powi(zoom_diff as i32)
    }

    /// Get the sub-tile offset within the source tile
    pub fn get_offset(&self, tile: TileCoordinate, source_tile: TileCoordinate) -> (u32, u32) {
        let zoom_diff = tile.z - source_tile.z;
        let scale = 1u32 << zoom_diff;

        let offset_x = tile.x - (source_tile.x * scale);
        let offset_y = tile.y - (source_tile.y * scale);

        (offset_x, offset_y)
    }

    /// Check if overzoom is allowed
    pub fn is_allowed(&self, tile: TileCoordinate, max_source_zoom: u8) -> bool {
        if tile.z <= max_source_zoom {
            return true;
        }

        let overzoom_levels = tile.z - max_source_zoom;
        overzoom_levels <= self.max_overzoom
    }
}

impl Default for OverzoomHandler {
    fn default() -> Self {
        Self::new(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_overzoom() {
        let handler = OverzoomHandler::new(5);
        let tile = TileCoordinate::new(15, 100, 100);

        assert!(!handler.requires_overzoom(tile, 15));
        assert!(handler.requires_overzoom(tile, 14));
    }

    #[test]
    fn test_get_source_tile() {
        let handler = OverzoomHandler::new(5);
        let tile = TileCoordinate::new(15, 100, 100);

        let source = handler.get_source_tile(tile, 14).unwrap();
        assert_eq!(source.z, 14);
        assert_eq!(source.x, 50);
        assert_eq!(source.y, 50);
    }

    #[test]
    fn test_scale_factor() {
        let handler = OverzoomHandler::new(5);
        let tile = TileCoordinate::new(15, 100, 100);
        let source = TileCoordinate::new(14, 50, 50);

        let scale = handler.scale_factor(tile, source);
        assert_eq!(scale, 2.0);
    }

    #[test]
    fn test_get_offset() {
        let handler = OverzoomHandler::new(5);
        let tile = TileCoordinate::new(15, 101, 101);
        let source = TileCoordinate::new(14, 50, 50);

        let (offset_x, offset_y) = handler.get_offset(tile, source);
        assert_eq!(offset_x, 1);
        assert_eq!(offset_y, 1);
    }

    #[test]
    fn test_max_overzoom_limit() {
        let handler = OverzoomHandler::new(2);
        let tile = TileCoordinate::new(15, 100, 100);

        // 3 levels of overzoom should fail
        assert!(handler.get_source_tile(tile, 12).is_err());

        // 2 levels should succeed
        assert!(handler.get_source_tile(tile, 13).is_ok());
    }
}
