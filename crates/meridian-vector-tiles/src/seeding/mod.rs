//! Tile seeding (pre-generation) module

pub mod strategy;

pub use strategy::{SeedingStrategy, BoundsSeedingStrategy, ZoomRangeSeedingStrategy};

use crate::encoding::MvtEncoder;
use crate::error::Result;
use crate::generation::TileGenerator;
use crate::source::TileSource;
use crate::storage::TileStorage;
use crate::tile::coordinate::TileCoordinate;
use rayon::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{info, warn};

/// Seeding configuration
#[derive(Debug, Clone)]
pub struct SeedingConfig {
    /// Number of parallel workers
    pub workers: usize,
    /// Skip existing tiles
    pub skip_existing: bool,
    /// Maximum retries on failure
    pub max_retries: u32,
    /// Enable progress reporting
    pub progress: bool,
}

impl Default for SeedingConfig {
    fn default() -> Self {
        Self {
            workers: num_cpus::get(),
            skip_existing: true,
            max_retries: 3,
            progress: true,
        }
    }
}

/// Tile seeder for pre-generating tiles
pub struct TileSeeder<S: TileSource, T: TileStorage> {
    source: Arc<S>,
    storage: Arc<T>,
    generator: Arc<TileGenerator>,
    encoder: Arc<MvtEncoder>,
    config: SeedingConfig,
}

impl<S: TileSource + Sync + Send, T: TileStorage + Sync + Send> TileSeeder<S, T> {
    /// Create a new tile seeder
    pub fn new(source: S, storage: T) -> Self {
        Self::with_config(source, storage, SeedingConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(source: S, storage: T, config: SeedingConfig) -> Self {
        Self {
            source: Arc::new(source),
            storage: Arc::new(storage),
            generator: Arc::new(TileGenerator::new()),
            encoder: Arc::new(MvtEncoder::new()),
            config,
        }
    }

    /// Seed tiles using a strategy
    pub async fn seed<St: SeedingStrategy>(&self, strategy: St) -> Result<SeedingStats> {
        info!("Starting tile seeding");

        // Get tiles to seed
        let tiles = strategy.get_tiles()?;
        let total = tiles.len();

        info!("Seeding {} tiles with {} workers", total, self.config.workers);

        // Statistics
        let generated = Arc::new(AtomicU64::new(0));
        let skipped = Arc::new(AtomicU64::new(0));
        let failed = Arc::new(AtomicU64::new(0));

        // Process tiles in parallel
        tiles.par_iter().for_each(|&tile| {
            let result = tokio::runtime::Handle::current().block_on(async {
                self.seed_tile(tile).await
            });

            match result {
                Ok(true) => {
                    generated.fetch_add(1, Ordering::Relaxed);
                }
                Ok(false) => {
                    skipped.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    warn!("Failed to seed tile {}: {}", tile, e);
                    failed.fetch_add(1, Ordering::Relaxed);
                }
            }

            if self.config.progress {
                let done = generated.load(Ordering::Relaxed)
                    + skipped.load(Ordering::Relaxed)
                    + failed.load(Ordering::Relaxed);

                if done % 100 == 0 {
                    info!("Progress: {}/{} tiles", done, total);
                }
            }
        });

        let stats = SeedingStats {
            total: total as u64,
            generated: generated.load(Ordering::Relaxed),
            skipped: skipped.load(Ordering::Relaxed),
            failed: failed.load(Ordering::Relaxed),
        };

        info!("Seeding complete: {:?}", stats);

        Ok(stats)
    }

    /// Seed a single tile
    async fn seed_tile(&self, tile: TileCoordinate) -> Result<bool> {
        // Check if tile exists and skip if configured
        if self.config.skip_existing {
            if self.storage.has_tile(tile).await? {
                return Ok(false); // Skipped
            }
        }

        // Generate tile
        let tile_bounds = crate::tile::bounds::MercatorBounds::from_tile(&tile);

        if let Some(mvt_tile) = self.generator.generate(&*self.source, tile).await? {
            // Encode to MVT
            let mvt_data = self.encoder.encode(&mvt_tile)?;

            // Store tile
            self.storage.put_tile(tile, mvt_data).await?;

            Ok(true) // Generated
        } else {
            Ok(false) // No data
        }
    }

    /// Seed a single zoom level
    pub async fn seed_zoom(&self, zoom: u8) -> Result<SeedingStats> {
        let strategy = ZoomRangeSeedingStrategy::new(zoom, zoom);
        self.seed(strategy).await
    }

    /// Seed multiple zoom levels
    pub async fn seed_zoom_range(&self, min_zoom: u8, max_zoom: u8) -> Result<SeedingStats> {
        let strategy = ZoomRangeSeedingStrategy::new(min_zoom, max_zoom);
        self.seed(strategy).await
    }
}

/// Seeding statistics
#[derive(Debug, Clone, Copy)]
pub struct SeedingStats {
    /// Total tiles attempted
    pub total: u64,
    /// Tiles successfully generated
    pub generated: u64,
    /// Tiles skipped (already existed)
    pub skipped: u64,
    /// Tiles that failed
    pub failed: u64,
}

impl SeedingStats {
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.generated as f64 / self.total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeding_config() {
        let config = SeedingConfig::default();
        assert!(config.workers > 0);
        assert!(config.skip_existing);
    }

    #[test]
    fn test_seeding_stats() {
        let stats = SeedingStats {
            total: 100,
            generated: 80,
            skipped: 15,
            failed: 5,
        };

        assert_eq!(stats.success_rate(), 80.0);
    }
}
