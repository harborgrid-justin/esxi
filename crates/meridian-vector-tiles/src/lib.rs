//! # Meridian Vector Tiles
//!
//! Enterprise-grade vector tile generation and serving library for the Meridian GIS Platform.
//!
//! ## Features
//!
//! - **MVT 2.0 Compliance**: Full Mapbox Vector Tile specification support
//! - **High Performance**: Parallel tile generation with efficient geometry processing
//! - **Multiple Formats**: MVT, PMTiles, MBTiles support
//! - **Dynamic Simplification**: Automatic geometry simplification per zoom level
//! - **Flexible Sources**: PostGIS, files, PMTiles readers
//! - **Multiple Storage**: MBTiles, PMTiles, directory, S3
//! - **Built-in Server**: Production-ready tile server with caching
//! - **Style Support**: Mapbox Style Spec compatible
//!
//! ## Example
//!
//! ```no_run
//! use meridian_vector_tiles::{
//!     generation::TileGenerator,
//!     source::postgis::PostGISSource,
//!     encoding::mvt::MvtEncoder,
//!     tile::coordinate::TileCoordinate,
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a PostGIS source
//! let source = PostGISSource::new("postgresql://user:pass@localhost/db").await?;
//!
//! // Generate a tile
//! let generator = TileGenerator::new();
//! let tile_data = generator.generate(
//!     &source,
//!     TileCoordinate::new(10, 512, 384)
//! ).await?;
//!
//! // Encode to MVT
//! let encoder = MvtEncoder::new();
//! let mvt_bytes = encoder.encode(&tile_data)?;
//! # Ok(())
//! # }
//! ```

pub mod encoding;
pub mod error;
pub mod generation;
pub mod seeding;
pub mod server;
pub mod source;
pub mod storage;
pub mod style;
pub mod tile;
pub mod tilejson;

// Re-export commonly used types
pub use error::{Error, Result};
pub use tile::coordinate::TileCoordinate;
pub use tile::bounds::TileBounds;
pub use tile::extent::TileExtent;

/// Current version of the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default tile size in pixels
pub const DEFAULT_TILE_SIZE: u32 = 512;

/// Default tile extent (MVT specification)
pub const DEFAULT_EXTENT: u32 = 4096;

/// Maximum zoom level supported
pub const MAX_ZOOM_LEVEL: u8 = 24;

/// Minimum zoom level
pub const MIN_ZOOM_LEVEL: u8 = 0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_TILE_SIZE, 512);
        assert_eq!(DEFAULT_EXTENT, 4096);
        assert_eq!(MAX_ZOOM_LEVEL, 24);
        assert_eq!(MIN_ZOOM_LEVEL, 0);
    }
}
