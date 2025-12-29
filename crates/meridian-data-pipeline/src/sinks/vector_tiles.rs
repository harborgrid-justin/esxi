//! Vector tile generation sink (MVT format).

use crate::error::{Result, SinkError, PipelineError};
use crate::sinks::{DataSink, SinkStatistics};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Vector tile format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorTileFormat {
    /// Mapbox Vector Tiles (MVT/PBF).
    Mvt,
}

/// Tile coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileCoordinate {
    /// Zoom level.
    pub z: u8,
    /// X coordinate.
    pub x: u32,
    /// Y coordinate.
    pub y: u32,
}

impl TileCoordinate {
    /// Create a new tile coordinate.
    pub fn new(z: u8, x: u32, y: u32) -> Self {
        Self { z, x, y }
    }

    /// Get tile bounds in EPSG:3857.
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let tile_size = 20037508.34 * 2.0 / (1_u32 << self.z) as f64;
        let min_x = -20037508.34 + self.x as f64 * tile_size;
        let max_y = 20037508.34 - self.y as f64 * tile_size;
        let max_x = min_x + tile_size;
        let min_y = max_y - tile_size;
        (min_x, min_y, max_x, max_y)
    }
}

/// Vector tile sink configuration.
#[derive(Debug, Clone)]
pub struct VectorTileSinkConfig {
    /// Output directory for tiles.
    pub output_dir: PathBuf,
    /// Tile format.
    pub format: VectorTileFormat,
    /// Layer name.
    pub layer_name: String,
    /// Minimum zoom level.
    pub min_zoom: u8,
    /// Maximum zoom level.
    pub max_zoom: u8,
    /// Tile buffer in pixels.
    pub buffer: u32,
    /// Extent (tile resolution).
    pub extent: u32,
    /// Simplification tolerance.
    pub simplify_tolerance: f64,
}

impl Default for VectorTileSinkConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("tiles"),
            format: VectorTileFormat::Mvt,
            layer_name: "default".to_string(),
            min_zoom: 0,
            max_zoom: 14,
            buffer: 64,
            extent: 4096,
            simplify_tolerance: 0.5,
        }
    }
}

/// Vector tile data sink.
pub struct VectorTileSink {
    config: VectorTileSinkConfig,
    geometry_column: String,
    stats: SinkStatistics,
}

impl VectorTileSink {
    /// Create a new vector tile sink.
    pub fn new(output_dir: impl AsRef<Path>) -> Self {
        Self {
            config: VectorTileSinkConfig {
                output_dir: output_dir.as_ref().to_path_buf(),
                ..Default::default()
            },
            geometry_column: "geometry".to_string(),
            stats: SinkStatistics::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: VectorTileSinkConfig) -> Self {
        Self {
            config,
            geometry_column: "geometry".to_string(),
            stats: SinkStatistics::new(),
        }
    }

    /// Set layer name.
    pub fn with_layer_name(mut self, name: impl Into<String>) -> Self {
        self.config.layer_name = name.into();
        self
    }

    /// Set zoom range.
    pub fn with_zoom_range(mut self, min_zoom: u8, max_zoom: u8) -> Self {
        self.config.min_zoom = min_zoom;
        self.config.max_zoom = max_zoom;
        self
    }

    /// Set buffer size.
    pub fn with_buffer(mut self, buffer: u32) -> Self {
        self.config.buffer = buffer;
        self
    }

    /// Set extent.
    pub fn with_extent(mut self, extent: u32) -> Self {
        self.config.extent = extent;
        self
    }

    /// Set geometry column.
    pub fn with_geometry_column(mut self, column: impl Into<String>) -> Self {
        self.geometry_column = column.into();
        self
    }

    /// Generate tile path.
    fn tile_path(&self, tile: &TileCoordinate) -> PathBuf {
        self.config
            .output_dir
            .join(format!("{}/{}/{}.pbf", tile.z, tile.x, tile.y))
    }

    /// Write batch to vector tiles.
    async fn write_mvt(&mut self, batch: RecordBatch) -> Result<()> {
        tracing::debug!(
            layer = %self.config.layer_name,
            records = batch.num_rows(),
            zoom_range = %format!("{}-{}", self.config.min_zoom, self.config.max_zoom),
            "Generating vector tiles"
        );

        // In a real implementation, this would:
        // 1. Extract geometries from the batch
        // 2. For each zoom level:
        //    a. Determine which tiles the geometries intersect
        //    b. Clip geometries to tile bounds
        //    c. Simplify geometries based on zoom level
        //    d. Convert to MVT format using protobuf
        //    e. Write to tile file
        // 3. Handle tile directory structure (z/x/y.pbf)

        self.stats.record_write(batch.num_rows(), 0);
        Ok(())
    }

    /// Create tile directory structure.
    fn create_tile_directories(&self, tile: &TileCoordinate) -> Result<()> {
        let tile_dir = self
            .config
            .output_dir
            .join(format!("{}/{}", tile.z, tile.x));

        std::fs::create_dir_all(&tile_dir).map_err(|e| {
            PipelineError::Sink(SinkError::FileWrite(format!(
                "Cannot create tile directory {}: {}",
                tile_dir.display(),
                e
            )))
        })?;

        Ok(())
    }
}

#[async_trait]
impl DataSink for VectorTileSink {
    async fn write(&mut self, batch: RecordBatch) -> Result<()> {
        match self.config.format {
            VectorTileFormat::Mvt => self.write_mvt(batch).await,
        }
    }

    async fn flush(&mut self) -> Result<()> {
        tracing::debug!(
            output_dir = %self.config.output_dir.display(),
            "Flushing vector tile sink"
        );
        Ok(())
    }

    fn statistics(&self) -> SinkStatistics {
        self.stats.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_coordinate() {
        let tile = TileCoordinate::new(10, 512, 384);
        assert_eq!(tile.z, 10);
        assert_eq!(tile.x, 512);
        assert_eq!(tile.y, 384);

        let (min_x, min_y, max_x, max_y) = tile.bounds();
        assert!(min_x < max_x);
        assert!(min_y < max_y);
    }

    #[test]
    fn test_vector_tile_sink_creation() {
        let sink = VectorTileSink::new("output/tiles");
        assert_eq!(sink.config.output_dir, PathBuf::from("output/tiles"));
    }

    #[test]
    fn test_vector_tile_sink_with_options() {
        let sink = VectorTileSink::new("output/tiles")
            .with_layer_name("buildings")
            .with_zoom_range(10, 16)
            .with_buffer(128)
            .with_extent(8192);

        assert_eq!(sink.config.layer_name, "buildings");
        assert_eq!(sink.config.min_zoom, 10);
        assert_eq!(sink.config.max_zoom, 16);
        assert_eq!(sink.config.buffer, 128);
        assert_eq!(sink.config.extent, 8192);
    }

    #[test]
    fn test_tile_path_generation() {
        let sink = VectorTileSink::new("output/tiles");
        let tile = TileCoordinate::new(10, 512, 384);
        let path = sink.tile_path(&tile);
        assert_eq!(path, PathBuf::from("output/tiles/10/512/384.pbf"));
    }
}
