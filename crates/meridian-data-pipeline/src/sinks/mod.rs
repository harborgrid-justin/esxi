//! Data sink module for writing to various destinations.
//!
//! Supports writing to:
//! - Files (GeoJSON, Shapefile, GeoPackage, Parquet)
//! - Databases (PostGIS, Oracle Spatial, SQLite)
//! - Vector tiles (MVT format)

pub mod database;
pub mod file;
pub mod vector_tiles;

use crate::error::Result;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;

/// Trait for data sinks that can write record batches.
#[async_trait]
pub trait DataSink: Send + Sync {
    /// Write a single record batch.
    async fn write(&mut self, batch: RecordBatch) -> Result<()>;

    /// Write multiple record batches.
    async fn write_batches(&mut self, batches: Vec<RecordBatch>) -> Result<()> {
        for batch in batches {
            self.write(batch).await?;
        }
        Ok(())
    }

    /// Flush any buffered data.
    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    /// Close the sink and release resources.
    async fn close(&mut self) -> Result<()> {
        self.flush().await
    }

    /// Get sink statistics.
    fn statistics(&self) -> SinkStatistics {
        SinkStatistics::default()
    }
}

/// Statistics about a data sink.
#[derive(Debug, Clone, Default)]
pub struct SinkStatistics {
    /// Total records written.
    pub records_written: usize,
    /// Total bytes written.
    pub bytes_written: usize,
    /// Number of batches written.
    pub batches_written: usize,
    /// Number of write errors.
    pub write_errors: usize,
}

impl SinkStatistics {
    /// Create new sink statistics.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful write.
    pub fn record_write(&mut self, records: usize, bytes: usize) {
        self.records_written += records;
        self.bytes_written += bytes;
        self.batches_written += 1;
    }

    /// Record a write error.
    pub fn record_error(&mut self) {
        self.write_errors += 1;
    }

    /// Get write success rate.
    pub fn success_rate(&self) -> f64 {
        let total = self.batches_written + self.write_errors;
        if total == 0 {
            0.0
        } else {
            (self.batches_written as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sink_statistics() {
        let mut stats = SinkStatistics::new();
        stats.record_write(100, 5000);
        stats.record_write(50, 2500);

        assert_eq!(stats.records_written, 150);
        assert_eq!(stats.bytes_written, 7500);
        assert_eq!(stats.batches_written, 2);
    }

    #[test]
    fn test_sink_success_rate() {
        let mut stats = SinkStatistics::new();
        stats.record_write(100, 5000);
        stats.record_write(100, 5000);
        stats.record_error();

        assert_eq!(stats.success_rate(), 200.0 / 3.0 * 100.0);
    }
}
