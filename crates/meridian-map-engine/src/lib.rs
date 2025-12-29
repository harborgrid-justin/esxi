//! # Meridian Map Engine
//!
//! High-performance WebGPU-based map rendering engine for the Meridian GIS Platform.
//!
//! This crate provides a complete map rendering solution with support for:
//! - WebGPU-accelerated rendering
//! - Vector and raster tile layers
//! - 2D and 2.5D camera views
//! - Interactive feature picking
//! - Style-based rendering
//! - Tile caching and management
//!
//! ## Features
//!
//! - `tile-cache` - Enable tile caching system (default)
//! - `gpu-picking` - Enable GPU-based feature picking (default)
//! - `text-rendering` - Enable text/label rendering (default)
//! - `profiling` - Enable detailed performance profiling
//! - `debug-shaders` - Enable shader debugging features
//!
//! ## Example
//!
//! ```rust,no_run
//! use meridian_map_engine::{Camera, Renderer, RendererConfig};
//!
//! async fn setup_renderer() {
//!     // Initialize renderer with default configuration
//!     let config = RendererConfig::default();
//!
//!     // Create camera at a specific location
//!     let camera = Camera::at_position(0.0, 0.0, 10.0);
//!
//!     // Renderer setup would go here...
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![allow(clippy::too_many_arguments)]

// Core modules
pub mod camera;
pub mod error;
pub mod interaction;
pub mod layers;
pub mod renderer;
pub mod style;
pub mod tile;

// Re-export commonly used types
pub use camera::{Camera, CameraUniform};
pub use error::{MapEngineError, Result};
pub use renderer::{FrameContext, InstanceData, Renderer, RendererConfig, Vertex};
pub use tile::{TileBounds, TileCoord, TileData};

/// Current version of the Meridian Map Engine.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Map engine initialization and configuration.
pub mod prelude {
    //! Prelude module for convenient imports.
    //!
    //! This module re-exports the most commonly used types and traits.

    pub use crate::camera::{Camera, CameraUniform};
    pub use crate::error::{MapEngineError, Result};
    pub use crate::renderer::{FrameContext, Renderer, RendererConfig, Vertex};
    pub use crate::tile::{TileCoord, TileData};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(VERSION, "0.2.5");
    }

    #[test]
    fn test_prelude_imports() {
        use crate::prelude::*;

        // Test that we can create basic types from prelude
        let _camera = Camera::new();
    }
}
