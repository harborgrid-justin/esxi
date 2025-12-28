//! Rendering pipeline for parallel tile generation

use crate::cache::{CachedTile, TileCache};
use crate::error::{RenderError, RenderResult};
use crate::mvt::{Layer as MvtLayer, MvtEncoder};
use crate::raster::{RasterRenderer, TileData, TileFormat};
use crate::style::Style;
use crate::tile::TileCoord;
use image::DynamicImage;
use rayon::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Rendering statistics
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    /// Number of tiles rendered
    pub tiles_rendered: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
    /// Total rendering time
    pub render_time: Duration,
    /// Average time per tile
    pub avg_tile_time: Duration,
}

impl RenderStats {
    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

/// Rendering pipeline configuration
pub struct PipelineConfig {
    /// Enable parallel rendering
    pub parallel: bool,
    /// Maximum number of features per tile
    pub max_features: usize,
    /// Rendering timeout per tile
    pub timeout: Duration,
    /// Enable caching
    pub cache_enabled: bool,
    /// Tile format for raster tiles
    pub tile_format: TileFormat,
    /// Compression quality (1-100)
    pub quality: u8,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        PipelineConfig {
            parallel: true,
            max_features: 10000,
            timeout: Duration::from_secs(30),
            cache_enabled: true,
            tile_format: TileFormat::Png,
            quality: 90,
        }
    }
}

/// Tile rendering pipeline
pub struct RenderPipeline {
    config: PipelineConfig,
    cache: Option<Arc<TileCache>>,
    renderer: Arc<RasterRenderer>,
}

impl RenderPipeline {
    /// Create a new rendering pipeline
    pub fn new(config: PipelineConfig) -> Self {
        RenderPipeline {
            config,
            cache: None,
            renderer: Arc::new(RasterRenderer::new()),
        }
    }

    /// Set tile cache
    pub fn with_cache(mut self, cache: TileCache) -> Self {
        self.cache = Some(Arc::new(cache));
        self
    }

    /// Set raster renderer
    pub fn with_renderer(mut self, renderer: RasterRenderer) -> Self {
        self.renderer = Arc::new(renderer);
        self
    }

    /// Render a single raster tile
    pub fn render_raster_tile(
        &self,
        coord: TileCoord,
        style: &Style,
        data: &TileData,
    ) -> RenderResult<Vec<u8>> {
        // Check cache first
        if self.config.cache_enabled {
            if let Some(cache) = &self.cache {
                if let Ok(Some(cached)) = cache.get(&coord) {
                    return Ok(cached.data);
                }
            }
        }

        // Render the tile
        let start = Instant::now();
        let image = self.renderer.render_tile(coord, style, data)?;
        let encoded = self.renderer.encode(&image, self.config.tile_format)?;

        // Check timeout
        if start.elapsed() > self.config.timeout {
            return Err(RenderError::Timeout(start.elapsed().as_millis() as u64));
        }

        // Cache the result
        if self.config.cache_enabled {
            if let Some(cache) = &self.cache {
                let cached = CachedTile::new(
                    coord,
                    encoded.clone(),
                    self.config.tile_format.mime_type().to_string(),
                );
                let _ = cache.put(cached);
            }
        }

        Ok(encoded)
    }

    /// Render a single vector tile
    pub fn render_vector_tile(
        &self,
        coord: TileCoord,
        layers: Vec<MvtLayer>,
    ) -> RenderResult<Vec<u8>> {
        // Check cache first
        if self.config.cache_enabled {
            if let Some(cache) = &self.cache {
                if let Ok(Some(cached)) = cache.get(&coord) {
                    return Ok(cached.data);
                }
            }
        }

        // Build the vector tile
        let start = Instant::now();
        let mut encoder = MvtEncoder::new(coord);

        for layer in layers {
            if layer.feature_count() > self.config.max_features {
                return Err(RenderError::FeatureLimitExceeded {
                    actual: layer.feature_count(),
                    limit: self.config.max_features,
                });
            }
            encoder.add_layer(layer)?;
        }

        let encoded = encoder.encode_compressed()?;

        // Check timeout
        if start.elapsed() > self.config.timeout {
            return Err(RenderError::Timeout(start.elapsed().as_millis() as u64));
        }

        // Cache the result
        if self.config.cache_enabled {
            if let Some(cache) = &self.cache {
                let cached = CachedTile::new(
                    coord,
                    encoded.clone(),
                    "application/x-protobuf".to_string(),
                );
                let _ = cache.put(cached);
            }
        }

        Ok(encoded)
    }

    /// Render multiple tiles in parallel
    pub fn render_tiles_parallel<F>(
        &self,
        coords: Vec<TileCoord>,
        render_fn: F,
    ) -> RenderResult<Vec<(TileCoord, Vec<u8>)>>
    where
        F: Fn(TileCoord) -> RenderResult<Vec<u8>> + Send + Sync,
    {
        let start = Instant::now();

        let results: Vec<_> = if self.config.parallel {
            coords
                .par_iter()
                .map(|&coord| {
                    let data = render_fn(coord)?;
                    Ok((coord, data))
                })
                .collect()
        } else {
            coords
                .iter()
                .map(|&coord| {
                    let data = render_fn(coord)?;
                    Ok((coord, data))
                })
                .collect()
        };

        // Check if any failed
        results
            .into_iter()
            .collect::<RenderResult<Vec<_>>>()
            .map_err(|e| {
                RenderError::PipelineError(format!(
                    "Failed to render {} tiles in {:?}: {}",
                    coords.len(),
                    start.elapsed(),
                    e
                ))
            })
    }

    /// Batch render raster tiles
    pub fn batch_render_raster(
        &self,
        coords: Vec<TileCoord>,
        style: Arc<Style>,
        data_fn: Arc<dyn Fn(TileCoord) -> RenderResult<TileData> + Send + Sync>,
    ) -> RenderResult<RenderStats> {
        let start = Instant::now();
        let mut stats = RenderStats::default();

        let pipeline = self;
        let results: Vec<_> = if self.config.parallel {
            coords
                .par_iter()
                .map(|&coord| {
                    // Check cache
                    if pipeline.config.cache_enabled {
                        if let Some(cache) = &pipeline.cache {
                            if let Ok(Some(_)) = cache.get(&coord) {
                                return Ok(true); // Cache hit
                            }
                        }
                    }

                    // Render tile
                    let tile_data = data_fn(coord)?;
                    pipeline.render_raster_tile(coord, &style, &tile_data)?;
                    Ok(false) // Cache miss
                })
                .collect()
        } else {
            coords
                .iter()
                .map(|&coord| {
                    // Check cache
                    if pipeline.config.cache_enabled {
                        if let Some(cache) = &pipeline.cache {
                            if let Ok(Some(_)) = cache.get(&coord) {
                                return Ok(true); // Cache hit
                            }
                        }
                    }

                    // Render tile
                    let tile_data = data_fn(coord)?;
                    pipeline.render_raster_tile(coord, &style, &tile_data)?;
                    Ok(false) // Cache miss
                })
                .collect()
        };

        // Aggregate stats
        for result in results {
            match result {
                Ok(true) => stats.cache_hits += 1,
                Ok(false) => stats.cache_misses += 1,
                Err(e) => return Err(e),
            }
        }

        stats.tiles_rendered = coords.len();
        stats.render_time = start.elapsed();
        stats.avg_tile_time = if stats.tiles_rendered > 0 {
            stats.render_time / stats.tiles_rendered as u32
        } else {
            Duration::from_secs(0)
        };

        Ok(stats)
    }

    /// Invalidate cache for specific tiles
    pub fn invalidate_cache(&self, _coords: &[TileCoord]) -> RenderResult<()> {
        if let Some(cache) = &self.cache {
            // For now, just clear the entire cache
            // In a real implementation, we'd remove specific tiles
            cache.clear()?;
        }
        Ok(())
    }

    /// Get pipeline statistics
    pub fn get_stats(&self) -> Option<crate::cache::CacheStats> {
        self.cache.as_ref().map(|cache| cache.stats())
    }

    /// Clear all caches
    pub fn clear_cache(&self) -> RenderResult<()> {
        if let Some(cache) = &self.cache {
            cache.clear()?;
        }
        Ok(())
    }
}

/// Layer compositor for combining multiple tile layers
pub struct LayerCompositor {
    layers: Vec<CompositeLayer>,
}

impl LayerCompositor {
    /// Create a new layer compositor
    pub fn new() -> Self {
        LayerCompositor { layers: Vec::new() }
    }

    /// Add a layer to composite
    pub fn add_layer(&mut self, layer: CompositeLayer) {
        self.layers.push(layer);
    }

    /// Composite all layers into a single image
    pub fn composite(&self, tile_size: u32) -> RenderResult<DynamicImage> {
        let mut result = DynamicImage::new_rgba8(tile_size, tile_size);

        for layer in &self.layers {
            if let Some(image) = &layer.image {
                // Overlay with opacity
                overlay_with_opacity(&mut result, image, layer.opacity)?;
            }
        }

        Ok(result)
    }
}

impl Default for LayerCompositor {
    fn default() -> Self {
        Self::new()
    }
}

/// Layer for compositing
pub struct CompositeLayer {
    /// Layer image
    pub image: Option<DynamicImage>,
    /// Layer opacity (0.0-1.0)
    pub opacity: f32,
    /// Blend mode
    pub blend_mode: BlendMode,
}

impl CompositeLayer {
    /// Create a new composite layer
    pub fn new(image: DynamicImage) -> Self {
        CompositeLayer {
            image: Some(image),
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
        }
    }

    /// Set opacity
    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set blend mode
    pub fn with_blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }
}

/// Blend mode for compositing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
}

/// Overlay an image with opacity
fn overlay_with_opacity(
    base: &mut DynamicImage,
    overlay: &DynamicImage,
    opacity: f32,
) -> RenderResult<()> {
    let base_rgba = base.as_mut_rgba8().ok_or_else(|| {
        RenderError::Other("Base image must be RGBA8".to_string())
    })?;

    let overlay_rgba = overlay.as_rgba8().ok_or_else(|| {
        RenderError::Other("Overlay image must be RGBA8".to_string())
    })?;

    for (x, y, pixel) in overlay_rgba.enumerate_pixels() {
        if x < base_rgba.width() && y < base_rgba.height() {
            let base_pixel = base_rgba.get_pixel_mut(x, y);

            // Alpha blend with opacity
            let alpha = (pixel[3] as f32 / 255.0) * opacity;
            let inv_alpha = 1.0 - alpha;

            base_pixel[0] = ((base_pixel[0] as f32 * inv_alpha) + (pixel[0] as f32 * alpha)) as u8;
            base_pixel[1] = ((base_pixel[1] as f32 * inv_alpha) + (pixel[1] as f32 * alpha)) as u8;
            base_pixel[2] = ((base_pixel[2] as f32 * inv_alpha) + (pixel[2] as f32 * alpha)) as u8;
            base_pixel[3] = ((base_pixel[3] as f32).max(pixel[3] as f32 * opacity)) as u8;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_config() {
        let config = PipelineConfig::default();
        assert!(config.parallel);
        assert_eq!(config.max_features, 10000);
    }

    #[test]
    fn test_render_stats() {
        let stats = RenderStats {
            tiles_rendered: 100,
            cache_hits: 75,
            cache_misses: 25,
            render_time: Duration::from_secs(10),
            avg_tile_time: Duration::from_millis(100),
        };

        assert_eq!(stats.cache_hit_rate(), 0.75);
    }

    #[test]
    fn test_layer_compositor() {
        let mut compositor = LayerCompositor::new();
        let image = DynamicImage::new_rgba8(256, 256);
        let layer = CompositeLayer::new(image).with_opacity(0.5);

        compositor.add_layer(layer);
        assert_eq!(compositor.layers.len(), 1);
    }

    #[test]
    fn test_composite_layer_opacity() {
        let image = DynamicImage::new_rgba8(256, 256);
        let layer = CompositeLayer::new(image).with_opacity(1.5);

        // Opacity should be clamped to 1.0
        assert_eq!(layer.opacity, 1.0);
    }
}
