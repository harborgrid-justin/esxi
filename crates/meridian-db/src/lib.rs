//! Meridian Database Layer
//!
//! PostGIS database integration for Meridian GIS Platform with spatial indexing,
//! query optimization, and transaction management.
//!
//! # Features
//!
//! - **Connection Pool Management**: Configurable connection pooling with health checks
//! - **Spatial Queries**: PostGIS spatial operations (ST_Within, ST_Intersects, ST_Contains, etc.)
//! - **Repository Pattern**: Generic repository trait for CRUD operations
//! - **Transaction Support**: ACID transactions with savepoints and rollback
//! - **Migration System**: Schema versioning and database migrations
//! - **Query Optimization**: Spatial index hints and query planning
//!
//! # Example
//!
//! ```rust,no_run
//! use meridian_db::{Pool, PoolConfig, MigrationManager, LayerRepository, SpatialRepository};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create connection pool
//!     let config = PoolConfig::builder()
//!         .host("localhost")
//!         .database("meridian")
//!         .username("postgres")
//!         .password("password")
//!         .max_connections(20)
//!         .build();
//!
//!     let pool = Pool::new(config).await?;
//!
//!     // Run migrations
//!     let migrator = MigrationManager::new(&pool);
//!     migrator.install_postgis().await?;
//!
//!     // Use repository
//!     let repo = LayerRepository::new(&pool);
//!     let layers = repo.find_all(Default::default()).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod migrations;
pub mod models;
pub mod pool;
pub mod queries;
pub mod repository;
pub mod transaction;

// Re-export commonly used types
pub use error::{DbError, DbResult};
pub use migrations::{default_migrations, Migration, MigrationManager};
pub use models::{
    BBox, Feature, Layer, LayerStyle, Metadata, PaginatedResponse, Pagination, SpatialIndex,
};
pub use pool::{HealthStatus, Pool, PoolConfig, PoolStats};
pub use queries::{IndexHint, QueryOptimizer, SpatialAggregation, SpatialQuery};
pub use repository::{
    FeatureRepository, LayerRepository, SpatialQueryParams, SpatialRepository,
};
pub use transaction::{DbTransaction, IsolationLevel, TransactionManager};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize database with PostGIS and base schema
///
/// # Example
///
/// ```rust,no_run
/// use meridian_db::{Pool, PoolConfig, init_database};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = PoolConfig::default();
///     let pool = Pool::new(config).await?;
///     init_database(&pool).await?;
///     Ok(())
/// }
/// ```
pub async fn init_database(pool: &Pool) -> DbResult<()> {
    let migrator = MigrationManager::new(pool);

    // Install PostGIS
    migrator.install_postgis().await?;

    // Run migrations
    let migrations = default_migrations();
    migrator.migrate(&migrations).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
