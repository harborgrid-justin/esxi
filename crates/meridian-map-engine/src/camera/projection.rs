//! Map projections for converting geographic coordinates to screen space.

use glam::Vec2;
use std::f32::consts::PI;

/// Map projection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionType {
    /// Web Mercator projection (EPSG:3857).
    WebMercator,
    /// Equirectangular projection.
    Equirectangular,
    /// Albers Equal Area Conic.
    AlbersEqualArea,
}

/// Map projection trait for coordinate transformations.
pub trait Projection {
    /// Project geographic coordinates (lon, lat) to map coordinates.
    fn project(&self, lon: f32, lat: f32) -> Vec2;

    /// Unproject map coordinates to geographic coordinates (lon, lat).
    fn unproject(&self, x: f32, y: f32) -> Vec2;

    /// Get the projection type.
    fn projection_type(&self) -> ProjectionType;
}

/// Web Mercator projection (most common for web maps).
#[derive(Debug, Clone, Copy)]
pub struct WebMercatorProjection {
    /// World size in map units.
    world_size: f32,
}

impl WebMercatorProjection {
    /// Create a new Web Mercator projection.
    pub fn new(world_size: f32) -> Self {
        Self { world_size }
    }

    /// Get the default projection with standard world size.
    pub fn default_projection() -> Self {
        Self::new(512.0)
    }
}

impl Default for WebMercatorProjection {
    fn default() -> Self {
        Self::default_projection()
    }
}

impl Projection for WebMercatorProjection {
    fn project(&self, lon: f32, lat: f32) -> Vec2 {
        // Convert degrees to radians
        let lon_rad = lon.to_radians();
        let lat_rad = lat.to_radians();

        // Web Mercator formulas
        let x = (lon_rad + PI) / (2.0 * PI);
        let y = (PI - ((PI / 4.0 + lat_rad / 2.0).tan().ln())) / (2.0 * PI);

        Vec2::new(x * self.world_size, y * self.world_size)
    }

    fn unproject(&self, x: f32, y: f32) -> Vec2 {
        // Normalize coordinates
        let x_norm = x / self.world_size;
        let y_norm = y / self.world_size;

        // Web Mercator inverse formulas
        let lon = x_norm * 2.0 * PI - PI;
        let lat = 2.0 * ((PI * (0.5 - y_norm)).exp().atan()) - PI / 2.0;

        Vec2::new(lon.to_degrees(), lat.to_degrees())
    }

    fn projection_type(&self) -> ProjectionType {
        ProjectionType::WebMercator
    }
}

/// Equirectangular projection (simple lat/lon).
#[derive(Debug, Clone, Copy)]
pub struct EquirectangularProjection {
    /// World size in map units.
    world_size: f32,
}

impl EquirectangularProjection {
    /// Create a new Equirectangular projection.
    pub fn new(world_size: f32) -> Self {
        Self { world_size }
    }
}

impl Default for EquirectangularProjection {
    fn default() -> Self {
        Self::new(512.0)
    }
}

impl Projection for EquirectangularProjection {
    fn project(&self, lon: f32, lat: f32) -> Vec2 {
        let x = (lon + 180.0) / 360.0 * self.world_size;
        let y = (90.0 - lat) / 180.0 * self.world_size;
        Vec2::new(x, y)
    }

    fn unproject(&self, x: f32, y: f32) -> Vec2 {
        let lon = (x / self.world_size) * 360.0 - 180.0;
        let lat = 90.0 - (y / self.world_size) * 180.0;
        Vec2::new(lon, lat)
    }

    fn projection_type(&self) -> ProjectionType {
        ProjectionType::Equirectangular
    }
}

/// Coordinate conversion utilities.
pub struct CoordinateUtils;

impl CoordinateUtils {
    /// Clamp latitude to valid range (-85.05112878 to 85.05112878 for Web Mercator).
    pub fn clamp_lat_mercator(lat: f32) -> f32 {
        lat.clamp(-85.05112878, 85.05112878)
    }

    /// Normalize longitude to -180 to 180 range.
    pub fn normalize_lon(lon: f32) -> f32 {
        let mut normalized = lon % 360.0;
        if normalized > 180.0 {
            normalized -= 360.0;
        } else if normalized < -180.0 {
            normalized += 360.0;
        }
        normalized
    }

    /// Calculate distance between two geographic points (Haversine formula).
    pub fn haversine_distance(lon1: f32, lat1: f32, lon2: f32, lat2: f32) -> f32 {
        const EARTH_RADIUS: f32 = 6371000.0; // meters

        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lon = (lon2 - lon1).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS * c
    }

    /// Calculate bearing between two geographic points.
    pub fn bearing(lon1: f32, lat1: f32, lon2: f32, lat2: f32) -> f32 {
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lon = (lon2 - lon1).to_radians();

        let y = delta_lon.sin() * lat2_rad.cos();
        let x = lat1_rad.cos() * lat2_rad.sin()
            - lat1_rad.sin() * lat2_rad.cos() * delta_lon.cos();

        let bearing_rad = y.atan2(x);
        (bearing_rad.to_degrees() + 360.0) % 360.0
    }

    /// Calculate destination point given start point, bearing, and distance.
    pub fn destination_point(lon: f32, lat: f32, bearing: f32, distance: f32) -> Vec2 {
        const EARTH_RADIUS: f32 = 6371000.0; // meters

        let lat_rad = lat.to_radians();
        let lon_rad = lon.to_radians();
        let bearing_rad = bearing.to_radians();
        let angular_distance = distance / EARTH_RADIUS;

        let lat2 = (lat_rad.sin() * angular_distance.cos()
            + lat_rad.cos() * angular_distance.sin() * bearing_rad.cos())
        .asin();

        let lon2 = lon_rad
            + (bearing_rad.sin() * angular_distance.sin() * lat_rad.cos())
                .atan2(angular_distance.cos() - lat_rad.sin() * lat2.sin());

        Vec2::new(lon2.to_degrees(), lat2.to_degrees())
    }

    /// Convert zoom level to meters per pixel at equator.
    pub fn meters_per_pixel_at_zoom(zoom: f32) -> f32 {
        const EARTH_CIRCUMFERENCE: f32 = 40075016.686; // meters at equator
        const TILE_SIZE: f32 = 256.0;

        EARTH_CIRCUMFERENCE / (TILE_SIZE * 2_f32.powf(zoom))
    }

    /// Convert meters per pixel to zoom level.
    pub fn zoom_from_meters_per_pixel(meters_per_pixel: f32) -> f32 {
        const EARTH_CIRCUMFERENCE: f32 = 40075016.686;
        const TILE_SIZE: f32 = 256.0;

        (EARTH_CIRCUMFERENCE / (meters_per_pixel * TILE_SIZE)).log2()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_mercator_projection() {
        let proj = WebMercatorProjection::new(512.0);

        // Test null island (0, 0)
        let result = proj.project(0.0, 0.0);
        assert!((result.x - 256.0).abs() < 0.1);
        assert!((result.y - 256.0).abs() < 0.1);

        // Test roundtrip
        let original = Vec2::new(-122.4194, 37.7749); // San Francisco
        let projected = proj.project(original.x, original.y);
        let unprojected = proj.unproject(projected.x, projected.y);
        assert!((original.x - unprojected.x).abs() < 0.001);
        assert!((original.y - unprojected.y).abs() < 0.001);
    }

    #[test]
    fn test_equirectangular_projection() {
        let proj = EquirectangularProjection::new(512.0);

        // Test null island (0, 0)
        let result = proj.project(0.0, 0.0);
        assert!((result.x - 256.0).abs() < 0.1);
        assert!((result.y - 256.0).abs() < 0.1);

        // Test corners
        let top_left = proj.project(-180.0, 90.0);
        assert!(top_left.x < 1.0);
        assert!(top_left.y < 1.0);
    }

    #[test]
    fn test_coordinate_normalization() {
        assert_eq!(CoordinateUtils::normalize_lon(190.0), -170.0);
        assert_eq!(CoordinateUtils::normalize_lon(-190.0), 170.0);
        assert_eq!(CoordinateUtils::normalize_lon(0.0), 0.0);
    }

    #[test]
    fn test_haversine_distance() {
        // San Francisco to Los Angeles
        let distance = CoordinateUtils::haversine_distance(
            -122.4194, 37.7749, // SF
            -118.2437, 34.0522, // LA
        );
        // Approximate distance is ~559 km
        assert!((distance - 559000.0).abs() < 10000.0);
    }

    #[test]
    fn test_meters_per_pixel() {
        let mpp_z0 = CoordinateUtils::meters_per_pixel_at_zoom(0.0);
        let mpp_z1 = CoordinateUtils::meters_per_pixel_at_zoom(1.0);

        // Each zoom level should halve the meters per pixel
        assert!((mpp_z0 / 2.0 - mpp_z1).abs() < 1.0);
    }
}
