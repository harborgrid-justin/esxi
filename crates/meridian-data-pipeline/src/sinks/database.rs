//! Database data sinks.

use crate::error::{Result, SinkError, PipelineError};
use crate::sinks::{DataSink, SinkStatistics};
use crate::sources::database::DatabaseType;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Database sink configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSinkConfig {
    /// Database type.
    pub db_type: DatabaseType,
    /// Connection string.
    pub connection_string: String,
    /// Target table name.
    pub table: String,
    /// Geometry column name.
    pub geometry_column: Option<String>,
    /// Write mode.
    pub write_mode: WriteMode,
    /// Batch size for bulk inserts.
    pub batch_size: usize,
    /// SRID for geometry column.
    pub srid: Option<i32>,
}

/// Write mode for database sink.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteMode {
    /// Append to existing table.
    Append,
    /// Overwrite existing table.
    Overwrite,
    /// Fail if table exists.
    ErrorIfExists,
    /// Create table if it doesn't exist.
    CreateIfNotExists,
}

impl Default for DatabaseSinkConfig {
    fn default() -> Self {
        Self {
            db_type: DatabaseType::PostGIS,
            connection_string: String::new(),
            table: String::new(),
            geometry_column: Some("geom".to_string()),
            write_mode: WriteMode::Append,
            batch_size: 1000,
            srid: Some(4326),
        }
    }
}

/// Database data sink.
pub struct DatabaseSink {
    config: DatabaseSinkConfig,
    stats: SinkStatistics,
}

impl DatabaseSink {
    /// Create a new database sink.
    pub fn new(config: DatabaseSinkConfig) -> Self {
        Self {
            config,
            stats: SinkStatistics::new(),
        }
    }

    /// Create a PostGIS sink.
    pub fn postgis(
        connection_string: impl Into<String>,
        table: impl Into<String>,
    ) -> Self {
        Self::new(DatabaseSinkConfig {
            db_type: DatabaseType::PostGIS,
            connection_string: connection_string.into(),
            table: table.into(),
            ..Default::default()
        })
    }

    /// Create a SpatiaLite sink.
    pub fn spatialite(
        connection_string: impl Into<String>,
        table: impl Into<String>,
    ) -> Self {
        Self::new(DatabaseSinkConfig {
            db_type: DatabaseType::SpatiaLite,
            connection_string: connection_string.into(),
            table: table.into(),
            ..Default::default()
        })
    }

    /// Set write mode.
    pub fn with_write_mode(mut self, mode: WriteMode) -> Self {
        self.config.write_mode = mode;
        self
    }

    /// Set geometry column.
    pub fn with_geometry_column(mut self, column: impl Into<String>) -> Self {
        self.config.geometry_column = Some(column.into());
        self
    }

    /// Set SRID.
    pub fn with_srid(mut self, srid: i32) -> Self {
        self.config.srid = Some(srid);
        self
    }

    /// Set batch size.
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    /// Write batch to PostGIS.
    async fn write_postgis(&mut self, batch: RecordBatch) -> Result<()> {
        tracing::debug!(
            table = %self.config.table,
            records = batch.num_rows(),
            "Writing to PostGIS table"
        );

        // In a real implementation, this would:
        // 1. Convert Arrow RecordBatch to SQL INSERT statements
        // 2. Use COPY for efficient bulk loading
        // 3. Convert geometries to PostGIS format using ST_GeomFromText
        // 4. Handle write mode appropriately

        self.stats.record_write(batch.num_rows(), 0);
        Ok(())
    }

    /// Write batch to SpatiaLite.
    async fn write_spatialite(&mut self, batch: RecordBatch) -> Result<()> {
        tracing::debug!(
            table = %self.config.table,
            records = batch.num_rows(),
            "Writing to SpatiaLite table"
        );

        // SpatiaLite writing would be implemented here
        self.stats.record_write(batch.num_rows(), 0);
        Ok(())
    }
}

#[async_trait]
impl DataSink for DatabaseSink {
    async fn write(&mut self, batch: RecordBatch) -> Result<()> {
        match self.config.db_type {
            DatabaseType::PostGIS => self.write_postgis(batch).await,
            DatabaseType::SpatiaLite => self.write_spatialite(batch).await,
            _ => Err(PipelineError::Sink(SinkError::DatabaseWrite(format!(
                "Unsupported database type: {:?}",
                self.config.db_type
            )))),
        }
    }

    async fn flush(&mut self) -> Result<()> {
        tracing::debug!(table = %self.config.table, "Flushing database sink");
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
    fn test_database_sink_creation() {
        let sink = DatabaseSink::postgis("postgresql://localhost/test", "cities");
        assert_eq!(sink.config.db_type, DatabaseType::PostGIS);
        assert_eq!(sink.config.table, "cities");
    }

    #[test]
    fn test_database_sink_with_options() {
        let sink = DatabaseSink::postgis("postgresql://localhost/test", "cities")
            .with_write_mode(WriteMode::Overwrite)
            .with_geometry_column("the_geom")
            .with_srid(3857)
            .with_batch_size(5000);

        assert_eq!(sink.config.write_mode, WriteMode::Overwrite);
        assert_eq!(sink.config.geometry_column, Some("the_geom".to_string()));
        assert_eq!(sink.config.srid, Some(3857));
        assert_eq!(sink.config.batch_size, 5000);
    }
}
