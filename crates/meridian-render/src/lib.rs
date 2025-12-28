//! # Meridian Render
//!
//! Map tile generation and rendering engine for the Meridian GIS Platform.
//!
//! ## Features
//!
//! - **Tile System**: Web Mercator tile coordinate system with TMS/XYZ support
//! - **Vector Tiles**: Mapbox Vector Tile (MVT) encoding with geometry simplification
//! - **Raster Rendering**: High-quality raster tile generation with anti-aliasing
//! - **Styling**: Mapbox GL style specification support
//! - **Caching**: Multi-tier caching with LRU memory cache and disk persistence
//! - **Parallel Processing**: Efficient parallel tile generation with Rayon
//! - **Symbols**: Symbol and icon management with sprite sheet support
//!
//! ## Example
//!
//! ```no_run
//! use meridian_render::{
//!     tile::TileCoord,
//!     raster::{RasterRenderer, TileData, TileFormat},
//!     style::Style,
//!     cache::TileCache,
//!     pipeline::{RenderPipeline, PipelineConfig},
//! };
//!
//! // Create a tile coordinate
//! let coord = TileCoord::new(10, 512, 384).unwrap();
//!
//! // Create a renderer
//! let renderer = RasterRenderer::new();
//!
//! // Create a style
//! let style = Style::new("default".to_string());
//!
//! // Create tile data
//! let data = TileData::new();
//!
//! // Render the tile
//! let image = renderer.render_tile(coord, &style, &data).unwrap();
//! ```
//!
//! ## Tile Coordinate System
//!
//! The library uses the standard Web Mercator (EPSG:3857) tile coordinate system:
//!
//! - Zoom levels: 0-22
//! - Tile size: 256x256 pixels (configurable)
//! - Origin: Top-left corner
//! - Both XYZ and TMS coordinate systems supported
//!
//! ## Rendering Pipeline
//!
//! The rendering pipeline supports both vector and raster tile generation:
//!
//! 1. **Tile Request**: Client requests a tile at specific coordinates
//! 2. **Cache Check**: Check if tile exists in cache
//! 3. **Data Fetch**: Fetch geometry data from data source
//! 4. **Styling**: Apply style rules based on zoom level and feature properties
//! 5. **Rendering**: Render to raster image or encode as vector tile
//! 6. **Compression**: Compress output (PNG, JPEG, WebP, or gzipped MVT)
//! 7. **Caching**: Store result in cache for future requests
//!
//! ## Performance
//!
//! - Parallel tile generation with configurable worker threads
//! - Memory-efficient LRU caching
//! - Geometry simplification for reduced tile sizes
//! - Progressive rendering for large datasets

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod cache;
pub mod error;
pub mod mvt;
pub mod pipeline;
pub mod raster;
pub mod style;
pub mod symbols;
pub mod tile;

// Re-export commonly used types
pub use error::{RenderError, RenderResult};
pub use tile::{TileBounds, TileCoord, TILE_SIZE};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Maximum supported zoom level
pub const MAX_ZOOM: u8 = tile::MAX_ZOOM;

/// Web Mercator EPSG code
pub const WEB_MERCATOR_EPSG: u32 = tile::WEB_MERCATOR_EPSG;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_ZOOM, 22);
        assert_eq!(WEB_MERCATOR_EPSG, 3857);
        assert_eq!(TILE_SIZE, 256);
    }
}
