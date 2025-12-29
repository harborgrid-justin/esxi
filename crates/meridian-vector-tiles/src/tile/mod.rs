//! Tile coordinate systems and bounds management

pub mod bounds;
pub mod coordinate;
pub mod extent;

pub use bounds::TileBounds;
pub use coordinate::TileCoordinate;
pub use extent::TileExtent;

use crate::error::{Error, Result};

/// Web Mercator projection constants
pub const EARTH_RADIUS: f64 = 6378137.0;
pub const EARTH_CIRCUMFERENCE: f64 = 2.0 * std::f64::consts::PI * EARTH_RADIUS;
pub const MAX_LATITUDE: f64 = 85.051129;

/// Convert longitude to Web Mercator X coordinate
pub fn lon_to_mercator_x(lon: f64) -> f64 {
    EARTH_RADIUS * lon.to_radians()
}

/// Convert latitude to Web Mercator Y coordinate
pub fn lat_to_mercator_y(lat: f64) -> f64 {
    let lat = lat.clamp(-MAX_LATITUDE, MAX_LATITUDE);
    let y = (std::f64::consts::PI / 4.0 + lat.to_radians() / 2.0).tan().ln();
    EARTH_RADIUS * y
}

/// Convert Web Mercator X to longitude
pub fn mercator_x_to_lon(x: f64) -> f64 {
    (x / EARTH_RADIUS).to_degrees()
}

/// Convert Web Mercator Y to latitude
pub fn mercator_y_to_lat(y: f64) -> f64 {
    let lat = 2.0 * ((y / EARTH_RADIUS).exp().atan()) - std::f64::consts::PI / 2.0;
    lat.to_degrees()
}

/// Get the resolution (meters per pixel) for a given zoom level
pub fn resolution_at_zoom(zoom: u8) -> f64 {
    EARTH_CIRCUMFERENCE / (256.0 * (1 << zoom) as f64)
}

/// Get tile size in meters for a given zoom level
pub fn tile_size_at_zoom(zoom: u8) -> f64 {
    EARTH_CIRCUMFERENCE / (1 << zoom) as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mercator_conversion() {
        let lon = -122.4194;
        let lat = 37.7749;

        let x = lon_to_mercator_x(lon);
        let y = lat_to_mercator_y(lat);

        let lon2 = mercator_x_to_lon(x);
        let lat2 = mercator_y_to_lat(y);

        assert!((lon - lon2).abs() < 1e-6);
        assert!((lat - lat2).abs() < 1e-6);
    }

    #[test]
    fn test_resolution() {
        let res0 = resolution_at_zoom(0);
        let res1 = resolution_at_zoom(1);

        assert!((res0 / res1 - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_tile_size() {
        let size0 = tile_size_at_zoom(0);
        assert!((size0 - EARTH_CIRCUMFERENCE).abs() < 1e-6);

        let size1 = tile_size_at_zoom(1);
        assert!((size1 - EARTH_CIRCUMFERENCE / 2.0).abs() < 1e-6);
    }
}
