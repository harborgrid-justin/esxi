//! # Meridian Cache
//!
//! Enterprise distributed caching system for the Meridian GIS Platform.
//!
//! ## Features
//!
//! - **Multi-tier caching**: Combine L1 in-memory, L2 Redis, and L3 disk caches
//! - **Multiple eviction strategies**: LRU, LFU, and ARC algorithms
//! - **Redis Cluster support**: Automatic failover and sharding
//! - **Write policies**: Write-through, write-back, and write-around
//! - **Cache invalidation**: TTL, dependency-based, event-based, and manual
//! - **Compression**: LZ4 and Zstd support for cached data
//! - **Cache warming**: Pre-load frequently accessed data
//! - **GIS optimization**: Specialized tile caching for map data
//! - **Metrics**: Comprehensive statistics and performance tracking
//!
//! ## Quick Start
//!
//! ### Simple In-Memory Cache
//!
//! ```rust,no_run
//! use meridian_cache::prelude::*;
//! use bytes::Bytes;
//!
//! #[tokio::main]
//! async fn main() -> CacheResult<()> {
//!     // Create an LRU cache with 1000 entries
//!     let cache = MemoryCache::lru(1000);
//!
//!     // Store a value
//!     cache.set(
//!         "user:123",
//!         Bytes::from("user data"),
//!         CacheOptions::default(),
//!     ).await?;
//!
//!     // Retrieve the value
//!     if let Some(entry) = cache.get("user:123").await? {
//!         println!("Found: {:?}", entry.value);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Multi-Tier Cache
//!
//! ```rust,no_run
//! use meridian_cache::prelude::*;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> CacheResult<()> {
//!     // Create L1 in-memory cache
//!     let l1 = Arc::new(MemoryCache::lru(1000));
//!
//!     // Create L2 Redis cache
//!     let l2 = Arc::new(RedisCache::connect("redis://localhost:6379").await?);
//!
//!     // Build tiered cache
//!     let cache = TieredCacheBuilder::new(WritePolicy::WriteThrough)
//!         .add_tier(
//!             TierConfig {
//!                 name: "L1".to_string(),
//!                 priority: 0,
//!                 promote_on_hit: true,
//!                 write_policy: WritePolicy::WriteThrough,
//!             },
//!             l1,
//!         )
//!         .add_tier(
//!             TierConfig {
//!                 name: "L2".to_string(),
//!                 priority: 1,
//!                 promote_on_hit: false,
//!                 write_policy: WritePolicy::WriteThrough,
//!             },
//!             l2,
//!         )
//!         .build();
//!
//!     // Use the cache - automatically manages all tiers
//!     cache.set("key", bytes::Bytes::from("value"), CacheOptions::default()).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Tile Cache for GIS
//!
//! ```rust,no_run
//! use meridian_cache::prelude::*;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> CacheResult<()> {
//!     let backend = Arc::new(MemoryCache::lru(10000));
//!     let tile_cache = TileCache::new(backend, TileCacheConfig::default());
//!
//!     let coord = TileCoord::new(10, 512, 256);
//!     let tile_data = bytes::Bytes::from("tile image data");
//!
//!     // Cache a tile
//!     tile_cache.set_tile("osm", coord, tile_data, TileFormat::PNG).await?;
//!
//!     // Retrieve a tile
//!     if let Some(data) = tile_cache.get_tile("osm", coord).await? {
//!         println!("Tile size: {} bytes", data.len());
//!     }
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod backend;
pub mod compression;
pub mod error;
pub mod invalidation;
pub mod stats;
pub mod tile;
pub mod tiered;
pub mod warmup;

// Re-exports for convenience
pub use backend::{
    CacheBackend, CacheEntry, CacheMetadata, CacheOptions, WritePolicy,
};
pub use error::{CacheError, CacheResult};
pub use stats::{CacheStats, CacheMetrics, MetricsCollector, MetricsSnapshot};
pub use tiered::{TieredCache, TieredCacheBuilder, TierConfig};

/// Prelude module for common imports
pub mod prelude {
    pub use crate::backend::{
        memory::{EvictionStrategy, MemoryCache, MemoryCacheConfig},
        redis::{RedisCache, RedisCacheConfig},
        disk::{DiskCache, DiskCacheConfig},
        CacheBackend, CacheEntry, CacheMetadata, CacheOptions, WritePolicy,
    };
    pub use crate::compression::{
        AdaptiveCompressor, CompressionAlgorithm, CompressionConfig, Compressor,
    };
    pub use crate::error::{CacheError, CacheResult};
    pub use crate::invalidation::{
        InvalidationEvent, InvalidationManager, InvalidationPolicy, InvalidationStrategy,
        QueryInvalidationTracker,
    };
    pub use crate::stats::{
        CacheMetrics, CacheStats, MetricsCollector, MetricsSnapshot, PerformanceAnalyzer,
    };
    pub use crate::tile::{
        LayerStats, TileCache, TileCacheConfig, TileCoord, TileFormat, TileMetadata,
        TilePyramidGenerator,
    };
    pub use crate::tiered::{TieredCache, TieredCacheBuilder, TierConfig};
    pub use crate::warmup::{
        CachePreloader, CacheWarmer, PreloadPattern, WarmupConfig, WarmupSource, WarmupStats,
        WarmupStrategy,
    };
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the library version
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.1.5");
    }
}
