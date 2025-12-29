//! Database data sources for PostGIS, Oracle Spatial, and other spatial databases.

use crate::error::{PipelineError, Result, SourceError};
use crate::sources::{DataSource, RecordBatchStream, SourceStatistics};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use async_trait::async_trait;
use futures::stream;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Pool, Postgres, Sqlite, SqlitePool};
use std::sync::Arc;

/// Database type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseType {
    /// PostgreSQL with PostGIS extension.
    PostGIS,
    /// SQLite with SpatiaLite extension.
    SpatiaLite,
    /// Oracle Spatial.
    OracleSpatial,
    /// SQL Server with spatial support.
    SqlServer,
}

/// Database connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database type.
    pub db_type: DatabaseType,
    /// Connection string.
    pub connection_string: String,
    /// Table or view name.
    pub table: String,
    /// Geometry column name.
    pub geometry_column: Option<String>,
    /// Custom SQL query (overrides table).
    pub query: Option<String>,
    /// Batch size for reading.
    pub batch_size: usize,
    /// Maximum number of connections.
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: DatabaseType::PostGIS,
            connection_string: String::new(),
            table: String::new(),
            geometry_column: Some("geom".to_string()),
            query: None,
            batch_size: 1000,
            max_connections: 10,
        }
    }
}

/// Database data source.
pub struct DatabaseSource {
    config: DatabaseConfig,
    pool: Option<DatabasePool>,
}

/// Database connection pool wrapper.
enum DatabasePool {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

impl DatabaseSource {
    /// Create a new database source.
    pub fn new(config: DatabaseConfig) -> Self {
        Self { config, pool: None }
    }

    /// Create a PostGIS source.
    pub fn postgis(
        connection_string: impl Into<String>,
        table: impl Into<String>,
    ) -> Self {
        Self::new(DatabaseConfig {
            db_type: DatabaseType::PostGIS,
            connection_string: connection_string.into(),
            table: table.into(),
            ..Default::default()
        })
    }

    /// Create a SpatiaLite source.
    pub fn spatialite(
        connection_string: impl Into<String>,
        table: impl Into<String>,
    ) -> Self {
        Self::new(DatabaseConfig {
            db_type: DatabaseType::SpatiaLite,
            connection_string: connection_string.into(),
            table: table.into(),
            ..Default::default()
        })
    }

    /// Set geometry column name.
    pub fn with_geometry_column(mut self, column: impl Into<String>) -> Self {
        self.config.geometry_column = Some(column.into());
        self
    }

    /// Set custom SQL query.
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.config.query = Some(query.into());
        self
    }

    /// Set batch size.
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.config.batch_size = batch_size;
        self
    }

    /// Initialize database connection pool.
    async fn init_pool(&mut self) -> Result<()> {
        if self.pool.is_some() {
            return Ok(());
        }

        tracing::info!(
            db_type = ?self.config.db_type,
            table = %self.config.table,
            "Initializing database connection pool"
        );

        match self.config.db_type {
            DatabaseType::PostGIS => {
                let pool = PgPool::connect(&self.config.connection_string)
                    .await
                    .map_err(|e| {
                        PipelineError::Source(SourceError::DatabaseConnection(e.to_string()))
                    })?;

                self.pool = Some(DatabasePool::Postgres(pool));
            }
            DatabaseType::SpatiaLite => {
                let pool = SqlitePool::connect(&self.config.connection_string)
                    .await
                    .map_err(|e| {
                        PipelineError::Source(SourceError::DatabaseConnection(e.to_string()))
                    })?;

                self.pool = Some(DatabasePool::Sqlite(pool));
            }
            _ => {
                return Err(PipelineError::Source(SourceError::UnsupportedFormat(
                    format!("{:?} is not yet supported", self.config.db_type),
                )));
            }
        }

        Ok(())
    }

    /// Get the SQL query to execute.
    fn get_query(&self) -> String {
        if let Some(ref query) = self.config.query {
            query.clone()
        } else {
            let geom_col = self
                .config
                .geometry_column
                .as_deref()
                .unwrap_or("geom");

            match self.config.db_type {
                DatabaseType::PostGIS => {
                    format!(
                        "SELECT *, ST_AsText({}) as geometry_wkt FROM {}",
                        geom_col, self.config.table
                    )
                }
                DatabaseType::SpatiaLite => {
                    format!(
                        "SELECT *, AsText({}) as geometry_wkt FROM {}",
                        geom_col, self.config.table
                    )
                }
                _ => {
                    format!("SELECT * FROM {}", self.config.table)
                }
            }
        }
    }

    /// Get table statistics from database.
    async fn get_table_statistics(&self) -> Result<SourceStatistics> {
        if self.pool.is_none() {
            return Ok(SourceStatistics::default());
        }

        let count_query = if self.config.query.is_some() {
            format!("SELECT COUNT(*) FROM ({})", self.get_query())
        } else {
            format!("SELECT COUNT(*) FROM {}", self.config.table)
        };

        tracing::debug!(query = %count_query, "Fetching table statistics");

        // In a real implementation, we would execute the query and get the count
        // For now, return default statistics

        Ok(SourceStatistics::default())
    }

    /// Create schema from database table.
    async fn create_schema(&self) -> Result<SchemaRef> {
        // In a real implementation, this would query the database
        // to get column types and create an Arrow schema

        let schema = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("geometry_wkt", DataType::Utf8, true),
            Field::new("properties", DataType::Utf8, true),
        ]);

        Ok(Arc::new(schema))
    }
}

#[async_trait]
impl DataSource for DatabaseSource {
    async fn schema(&self) -> Result<SchemaRef> {
        self.create_schema().await
    }

    async fn read(&self) -> Result<RecordBatchStream> {
        // Ensure connection pool is initialized
        let mut source = self;
        // Note: In a real implementation, we'd need to make this mutable properly
        // For now, we'll just log and return empty stream

        let query = self.get_query();

        tracing::info!(
            db_type = ?self.config.db_type,
            table = %self.config.table,
            query = %query,
            "Reading from database"
        );

        // In a real implementation, this would:
        // 1. Execute the query
        // 2. Convert database rows to Arrow RecordBatches
        // 3. Stream the results

        let stream = stream::empty();
        Ok(Box::pin(stream))
    }

    async fn statistics(&self) -> SourceStatistics {
        self.get_table_statistics().await.unwrap_or_default()
    }

    async fn close(&self) -> Result<()> {
        if let Some(ref pool) = self.pool {
            match pool {
                DatabasePool::Postgres(pg_pool) => {
                    pg_pool.close().await;
                    tracing::info!("Closed PostgreSQL connection pool");
                }
                DatabasePool::Sqlite(sqlite_pool) => {
                    sqlite_pool.close().await;
                    tracing::info!("Closed SQLite connection pool");
                }
            }
        }

        Ok(())
    }
}

/// Builder for database sources with fluent API.
pub struct DatabaseSourceBuilder {
    config: DatabaseConfig,
}

impl DatabaseSourceBuilder {
    /// Create a new PostGIS source builder.
    pub fn postgis() -> Self {
        Self {
            config: DatabaseConfig {
                db_type: DatabaseType::PostGIS,
                ..Default::default()
            },
        }
    }

    /// Create a new SpatiaLite source builder.
    pub fn spatialite() -> Self {
        Self {
            config: DatabaseConfig {
                db_type: DatabaseType::SpatiaLite,
                ..Default::default()
            },
        }
    }

    /// Set connection string.
    pub fn connection_string(mut self, conn_str: impl Into<String>) -> Self {
        self.config.connection_string = conn_str.into();
        self
    }

    /// Set table name.
    pub fn table(mut self, table: impl Into<String>) -> Self {
        self.config.table = table.into();
        self
    }

    /// Set geometry column.
    pub fn geometry_column(mut self, column: impl Into<String>) -> Self {
        self.config.geometry_column = Some(column.into());
        self
    }

    /// Set custom query.
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.config.query = Some(query.into());
        self
    }

    /// Set batch size.
    pub fn batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    /// Set max connections.
    pub fn max_connections(mut self, max: u32) -> Self {
        self.config.max_connections = max;
        self
    }

    /// Build the database source.
    pub fn build(self) -> Result<DatabaseSource> {
        if self.config.connection_string.is_empty() {
            return Err(PipelineError::Source(SourceError::DatabaseConnection(
                "Connection string is required".into(),
            )));
        }

        if self.config.table.is_empty() && self.config.query.is_none() {
            return Err(PipelineError::Source(SourceError::DatabaseConnection(
                "Either table or query must be specified".into(),
            )));
        }

        Ok(DatabaseSource::new(self.config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_source_creation() {
        let source = DatabaseSource::postgis("postgresql://localhost/test", "cities");
        assert_eq!(source.config.db_type, DatabaseType::PostGIS);
        assert_eq!(source.config.table, "cities");
    }

    #[test]
    fn test_database_source_with_options() {
        let source = DatabaseSource::postgis("postgresql://localhost/test", "cities")
            .with_geometry_column("the_geom")
            .with_batch_size(5000);

        assert_eq!(source.config.geometry_column, Some("the_geom".to_string()));
        assert_eq!(source.config.batch_size, 5000);
    }

    #[test]
    fn test_database_source_builder() {
        let source = DatabaseSourceBuilder::postgis()
            .connection_string("postgresql://localhost/test")
            .table("cities")
            .geometry_column("geom")
            .batch_size(2000)
            .build()
            .unwrap();

        assert_eq!(source.config.db_type, DatabaseType::PostGIS);
        assert_eq!(source.config.table, "cities");
        assert_eq!(source.config.batch_size, 2000);
    }

    #[test]
    fn test_database_source_query_generation() {
        let source = DatabaseSource::postgis("postgresql://localhost/test", "cities")
            .with_geometry_column("the_geom");

        let query = source.get_query();
        assert!(query.contains("ST_AsText"));
        assert!(query.contains("the_geom"));
        assert!(query.contains("cities"));
    }
}
