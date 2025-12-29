//! # Meridian Data Pipeline
//!
//! Enterprise ETL and data processing pipeline for the Meridian GIS Platform.
//!
//! ## Features
//!
//! - **Apache Arrow Integration**: Columnar data processing with zero-copy operations
//! - **Parallel Processing**: Multi-threaded execution with Rayon
//! - **Streaming & Batch**: Support for both streaming and batch processing modes
//! - **Multiple Data Sources**: GeoJSON, Shapefile, GeoPackage, PostGIS, Oracle Spatial, WFS, WMS, Kafka, MQTT
//! - **Rich Transformations**: Geometry operations, projections, filtering, aggregation, spatial joins
//! - **Flexible Outputs**: Files, databases, vector tiles
//! - **Monitoring**: Metrics, performance tracking, data lineage
//! - **Checkpointing**: Resume failed pipelines from last checkpoint
//! - **YAML Configuration**: Define pipelines declaratively
//!
//! ## Example
//!
//! ```rust,no_run
//! use meridian_data_pipeline::{Pipeline, PipelineBuilder};
//! use meridian_data_pipeline::sources::FileSource;
//! use meridian_data_pipeline::transforms::ProjectionTransform;
//! use meridian_data_pipeline::sinks::FileSink;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Build a pipeline using the fluent API
//!     let pipeline = PipelineBuilder::new("geo-etl-pipeline")
//!         .version("1.0.0")
//!         .source(FileSource::geojson("input.geojson"))
//!         .transform(ProjectionTransform::new("EPSG:4326", "EPSG:3857"))
//!         .sink(FileSink::geojson("output.geojson"))
//!         .with_parallelism(4)
//!         .with_checkpointing(true)
//!         .build()?;
//!
//!     // Execute the pipeline
//!     pipeline.execute().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Pipeline Configuration (YAML)
//!
//! ```yaml
//! pipeline:
//!   name: geo-etl-pipeline
//!   version: 1.0.0
//!   parallelism: 4
//!   checkpointing: true
//!
//!   source:
//!     type: file
//!     format: geojson
//!     path: input.geojson
//!
//!   transforms:
//!     - type: projection
//!       from: EPSG:4326
//!       to: EPSG:3857
//!     - type: filter
//!       expression: "population > 100000"
//!
//!   sink:
//!     type: file
//!     format: geojson
//!     path: output.geojson
//! ```

#![deny(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

pub mod config;
pub mod error;
pub mod monitoring;
pub mod pipeline;
pub mod sinks;
pub mod sources;
pub mod transforms;

// Re-export commonly used types
pub use error::{PipelineError, Result};
pub use pipeline::{Pipeline, PipelineBuilder, PipelineExecutor, PipelineScheduler};

/// Version of the Meridian Data Pipeline.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Pipeline execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Batch processing mode - process all data at once.
    Batch,
    /// Streaming mode - process data as it arrives.
    Streaming,
    /// Micro-batch mode - process small batches continuously.
    MicroBatch,
}

/// Pipeline state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineState {
    /// Pipeline is being initialized.
    Initializing,
    /// Pipeline is running.
    Running,
    /// Pipeline is paused.
    Paused,
    /// Pipeline completed successfully.
    Completed,
    /// Pipeline failed with an error.
    Failed,
    /// Pipeline was cancelled.
    Cancelled,
}

/// Data format types supported by the pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    /// GeoJSON format.
    GeoJson,
    /// Shapefile format.
    Shapefile,
    /// GeoPackage format.
    GeoPackage,
    /// CSV format.
    Csv,
    /// Parquet format.
    Parquet,
    /// Arrow IPC format.
    Arrow,
    /// WKT (Well-Known Text) format.
    Wkt,
    /// WKB (Well-Known Binary) format.
    Wkb,
}

/// Coordinate reference system specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crs {
    /// EPSG code or other CRS identifier.
    pub code: String,
}

impl Crs {
    /// Create a new CRS from an EPSG code.
    pub fn epsg(code: u32) -> Self {
        Self {
            code: format!("EPSG:{}", code),
        }
    }

    /// Create a new CRS from a string identifier.
    pub fn from_string(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }

    /// WGS84 (EPSG:4326).
    pub fn wgs84() -> Self {
        Self::epsg(4326)
    }

    /// Web Mercator (EPSG:3857).
    pub fn web_mercator() -> Self {
        Self::epsg(3857)
    }
}

/// Record batch processing statistics.
#[derive(Debug, Clone, Default)]
pub struct BatchStats {
    /// Number of records processed.
    pub records_processed: usize,
    /// Number of records filtered out.
    pub records_filtered: usize,
    /// Number of records failed validation.
    pub records_failed: usize,
    /// Total bytes processed.
    pub bytes_processed: usize,
    /// Processing duration in milliseconds.
    pub duration_ms: u64,
}

impl BatchStats {
    /// Create new empty statistics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge another batch statistics into this one.
    pub fn merge(&mut self, other: &BatchStats) {
        self.records_processed += other.records_processed;
        self.records_filtered += other.records_filtered;
        self.records_failed += other.records_failed;
        self.bytes_processed += other.bytes_processed;
        self.duration_ms += other.duration_ms;
    }

    /// Get success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        if self.records_processed == 0 {
            0.0
        } else {
            let successful = self.records_processed - self.records_failed;
            (successful as f64 / self.records_processed as f64) * 100.0
        }
    }

    /// Get throughput in records per second.
    pub fn throughput(&self) -> f64 {
        if self.duration_ms == 0 {
            0.0
        } else {
            (self.records_processed as f64 / self.duration_ms as f64) * 1000.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crs_creation() {
        let crs = Crs::epsg(4326);
        assert_eq!(crs.code, "EPSG:4326");

        let crs = Crs::wgs84();
        assert_eq!(crs.code, "EPSG:4326");

        let crs = Crs::web_mercator();
        assert_eq!(crs.code, "EPSG:3857");
    }

    #[test]
    fn test_batch_stats() {
        let mut stats = BatchStats::new();
        stats.records_processed = 100;
        stats.records_failed = 10;
        stats.duration_ms = 1000;

        assert_eq!(stats.success_rate(), 90.0);
        assert_eq!(stats.throughput(), 100.0);
    }

    #[test]
    fn test_batch_stats_merge() {
        let mut stats1 = BatchStats {
            records_processed: 100,
            records_filtered: 10,
            records_failed: 5,
            bytes_processed: 1000,
            duration_ms: 500,
        };

        let stats2 = BatchStats {
            records_processed: 50,
            records_filtered: 5,
            records_failed: 2,
            bytes_processed: 500,
            duration_ms: 250,
        };

        stats1.merge(&stats2);

        assert_eq!(stats1.records_processed, 150);
        assert_eq!(stats1.records_filtered, 15);
        assert_eq!(stats1.records_failed, 7);
        assert_eq!(stats1.bytes_processed, 1500);
        assert_eq!(stats1.duration_ms, 750);
    }
}
