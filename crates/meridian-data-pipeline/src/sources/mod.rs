//! Data source module for reading from various sources.
//!
//! Supports reading from:
//! - Files (GeoJSON, Shapefile, GeoPackage, CSV, Parquet)
//! - Databases (PostGIS, Oracle Spatial, SQLite)
//! - APIs (REST, WFS, WMS)
//! - Streaming platforms (Kafka, MQTT)

pub mod api;
pub mod database;
pub mod file;
pub mod stream;

use crate::error::Result;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

/// A stream of record batches.
pub type RecordBatchStream = Pin<Box<dyn Stream<Item = Result<RecordBatch>> + Send>>;

/// Trait for data sources that can provide record batches.
#[async_trait]
pub trait DataSource: Send + Sync {
    /// Get the schema of the data source.
    async fn schema(&self) -> Result<arrow::datatypes::SchemaRef>;

    /// Read data as a stream of record batches.
    async fn read(&self) -> Result<RecordBatchStream>;

    /// Get the estimated number of records (if known).
    async fn estimated_records(&self) -> Option<usize> {
        None
    }

    /// Get source statistics.
    async fn statistics(&self) -> SourceStatistics {
        SourceStatistics::default()
    }

    /// Close the data source and release resources.
    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

/// Statistics about a data source.
#[derive(Debug, Clone, Default)]
pub struct SourceStatistics {
    /// Total number of records.
    pub total_records: Option<usize>,
    /// Total size in bytes.
    pub total_bytes: Option<usize>,
    /// Number of partitions.
    pub num_partitions: Option<usize>,
    /// Average record size in bytes.
    pub avg_record_size: Option<usize>,
}

impl SourceStatistics {
    /// Create new source statistics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set total records.
    pub fn with_total_records(mut self, total: usize) -> Self {
        self.total_records = Some(total);
        self
    }

    /// Set total bytes.
    pub fn with_total_bytes(mut self, bytes: usize) -> Self {
        self.total_bytes = Some(bytes);
        self
    }

    /// Set number of partitions.
    pub fn with_num_partitions(mut self, partitions: usize) -> Self {
        self.num_partitions = Some(partitions);
        self
    }

    /// Calculate average record size.
    pub fn calculate_avg_record_size(&mut self) {
        if let (Some(total_records), Some(total_bytes)) = (self.total_records, self.total_bytes) {
            if total_records > 0 {
                self.avg_record_size = Some(total_bytes / total_records);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_statistics() {
        let mut stats = SourceStatistics::new()
            .with_total_records(1000)
            .with_total_bytes(50000)
            .with_num_partitions(4);

        stats.calculate_avg_record_size();

        assert_eq!(stats.total_records, Some(1000));
        assert_eq!(stats.total_bytes, Some(50000));
        assert_eq!(stats.num_partitions, Some(4));
        assert_eq!(stats.avg_record_size, Some(50));
    }
}
