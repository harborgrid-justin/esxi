//! Database migration support with schema versioning and PostGIS setup

use crate::error::{DbError, DbResult};
use crate::pool::Pool;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};

/// Migration metadata
#[derive(Debug, Clone)]
pub struct Migration {
    /// Migration version
    pub version: i32,
    /// Migration name
    pub name: String,
    /// SQL to run for upgrade
    pub up: String,
    /// SQL to run for downgrade
    pub down: String,
    /// Applied timestamp
    pub applied_at: Option<DateTime<Utc>>,
}

impl Migration {
    /// Create a new migration
    pub fn new(version: i32, name: impl Into<String>, up: impl Into<String>, down: impl Into<String>) -> Self {
        Self {
            version,
            name: name.into(),
            up: up.into(),
            down: down.into(),
            applied_at: None,
        }
    }
}

/// Migration manager
pub struct MigrationManager {
    pool: PgPool,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(pool: &Pool) -> Self {
        Self {
            pool: pool.inner().clone(),
        }
    }

    /// Initialize migration tracking table
    pub async fn init(&self) -> DbResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::MigrationError(format!("Failed to create migrations table: {}", e)))?;

        Ok(())
    }

    /// Get current schema version
    pub async fn current_version(&self) -> DbResult<Option<i32>> {
        let version: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(version) FROM _migrations"
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten();

        Ok(version)
    }

    /// Get all applied migrations
    pub async fn applied_migrations(&self) -> DbResult<Vec<Migration>> {
        let rows = sqlx::query(
            "SELECT version, name, applied_at FROM _migrations ORDER BY version"
        )
        .fetch_all(&self.pool)
        .await?;

        let migrations = rows
            .iter()
            .map(|row| Migration {
                version: row.get("version"),
                name: row.get("name"),
                up: String::new(),
                down: String::new(),
                applied_at: Some(row.get("applied_at")),
            })
            .collect();

        Ok(migrations)
    }

    /// Apply a migration
    pub async fn apply(&self, migration: &Migration) -> DbResult<()> {
        let mut tx = self.pool.begin().await
            .map_err(|e| DbError::MigrationError(format!("Failed to start transaction: {}", e)))?;

        // Execute migration
        sqlx::query(&migration.up)
            .execute(&mut *tx)
            .await
            .map_err(|e| DbError::MigrationError(format!("Failed to apply migration {}: {}", migration.version, e)))?;

        // Record migration
        sqlx::query(
            "INSERT INTO _migrations (version, name) VALUES ($1, $2)"
        )
        .bind(migration.version)
        .bind(&migration.name)
        .execute(&mut *tx)
        .await
        .map_err(|e| DbError::MigrationError(format!("Failed to record migration: {}", e)))?;

        tx.commit().await
            .map_err(|e| DbError::MigrationError(format!("Failed to commit migration: {}", e)))?;

        Ok(())
    }

    /// Rollback a migration
    pub async fn rollback(&self, migration: &Migration) -> DbResult<()> {
        let mut tx = self.pool.begin().await
            .map_err(|e| DbError::MigrationError(format!("Failed to start transaction: {}", e)))?;

        // Execute rollback
        sqlx::query(&migration.down)
            .execute(&mut *tx)
            .await
            .map_err(|e| DbError::MigrationError(format!("Failed to rollback migration {}: {}", migration.version, e)))?;

        // Remove migration record
        sqlx::query(
            "DELETE FROM _migrations WHERE version = $1"
        )
        .bind(migration.version)
        .execute(&mut *tx)
        .await
        .map_err(|e| DbError::MigrationError(format!("Failed to remove migration record: {}", e)))?;

        tx.commit().await
            .map_err(|e| DbError::MigrationError(format!("Failed to commit rollback: {}", e)))?;

        Ok(())
    }

    /// Run all pending migrations
    pub async fn migrate(&self, migrations: &[Migration]) -> DbResult<()> {
        self.init().await?;

        let current = self.current_version().await?.unwrap_or(0);

        for migration in migrations {
            if migration.version > current {
                println!("Applying migration {}: {}", migration.version, migration.name);
                self.apply(migration).await?;
            }
        }

        Ok(())
    }

    /// Check if PostGIS extension is installed
    pub async fn postgis_installed(&self) -> DbResult<bool> {
        let result: Option<String> = sqlx::query_scalar(
            "SELECT extname FROM pg_extension WHERE extname = 'postgis'"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }

    /// Install PostGIS extension
    pub async fn install_postgis(&self) -> DbResult<()> {
        if self.postgis_installed().await? {
            return Ok(());
        }

        sqlx::query("CREATE EXTENSION IF NOT EXISTS postgis")
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::PostGisError(format!("Failed to install PostGIS: {}", e)))?;

        // Also install topology extension
        sqlx::query("CREATE EXTENSION IF NOT EXISTS postgis_topology")
            .execute(&self.pool)
            .await
            .ok(); // Topology is optional

        Ok(())
    }

    /// Get PostGIS version
    pub async fn postgis_version(&self) -> DbResult<String> {
        let version: String = sqlx::query_scalar("SELECT PostGIS_Version()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::PostGisError(format!("Failed to get PostGIS version: {}", e)))?;

        Ok(version)
    }
}

/// Create base schema migration
pub fn create_base_schema() -> Migration {
    Migration::new(
        1,
        "create_base_schema",
        r#"
        -- Enable PostGIS extension
        CREATE EXTENSION IF NOT EXISTS postgis;

        -- Create layers table
        CREATE TABLE layers (
            id UUID PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            layer_type TEXT NOT NULL,
            geometry_type TEXT,
            srid INTEGER NOT NULL DEFAULT 4326,
            visible BOOLEAN NOT NULL DEFAULT true,
            opacity DOUBLE PRECISION NOT NULL DEFAULT 1.0,
            metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            created_by UUID
        );

        -- Create features table
        CREATE TABLE features (
            id UUID PRIMARY KEY,
            layer_id UUID NOT NULL REFERENCES layers(id) ON DELETE CASCADE,
            geometry GEOMETRY(Geometry, 4326),
            properties JSONB NOT NULL DEFAULT '{}'::jsonb,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        -- Create layer styles table
        CREATE TABLE layer_styles (
            id UUID PRIMARY KEY,
            layer_id UUID NOT NULL REFERENCES layers(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            style JSONB NOT NULL,
            is_default BOOLEAN NOT NULL DEFAULT false,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        -- Create indexes
        CREATE INDEX idx_layers_type ON layers(layer_type);
        CREATE INDEX idx_layers_visible ON layers(visible);
        CREATE INDEX idx_features_layer_id ON features(layer_id);
        CREATE INDEX idx_layer_styles_layer_id ON layer_styles(layer_id);
        "#,
        r#"
        DROP TABLE IF EXISTS layer_styles;
        DROP TABLE IF EXISTS features;
        DROP TABLE IF EXISTS layers;
        "#,
    )
}

/// Create spatial indexes migration
pub fn create_spatial_indexes() -> Migration {
    Migration::new(
        2,
        "create_spatial_indexes",
        r#"
        -- Create spatial index on features geometry
        CREATE INDEX idx_features_geometry ON features USING GIST(geometry);

        -- Create additional metadata indexes
        CREATE INDEX idx_layers_metadata ON layers USING GIN(metadata);
        CREATE INDEX idx_features_properties ON features USING GIN(properties);
        "#,
        r#"
        DROP INDEX IF EXISTS idx_features_geometry;
        DROP INDEX IF EXISTS idx_layers_metadata;
        DROP INDEX IF EXISTS idx_features_properties;
        "#,
    )
}

/// Create updated_at triggers migration
pub fn create_triggers() -> Migration {
    Migration::new(
        3,
        "create_updated_at_triggers",
        r#"
        -- Function to update updated_at timestamp
        CREATE OR REPLACE FUNCTION update_updated_at_column()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at = NOW();
            RETURN NEW;
        END;
        $$ language 'plpgsql';

        -- Trigger for layers table
        CREATE TRIGGER update_layers_updated_at
            BEFORE UPDATE ON layers
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();

        -- Trigger for features table
        CREATE TRIGGER update_features_updated_at
            BEFORE UPDATE ON features
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();

        -- Trigger for layer_styles table
        CREATE TRIGGER update_layer_styles_updated_at
            BEFORE UPDATE ON layer_styles
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();
        "#,
        r#"
        DROP TRIGGER IF EXISTS update_layer_styles_updated_at ON layer_styles;
        DROP TRIGGER IF EXISTS update_features_updated_at ON features;
        DROP TRIGGER IF EXISTS update_layers_updated_at ON layers;
        DROP FUNCTION IF EXISTS update_updated_at_column();
        "#,
    )
}

/// Get all default migrations
pub fn default_migrations() -> Vec<Migration> {
    vec![
        create_base_schema(),
        create_spatial_indexes(),
        create_triggers(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_creation() {
        let migration = Migration::new(1, "test", "CREATE TABLE test (id INT)", "DROP TABLE test");
        assert_eq!(migration.version, 1);
        assert_eq!(migration.name, "test");
        assert!(migration.applied_at.is_none());
    }

    #[test]
    fn test_default_migrations() {
        let migrations = default_migrations();
        assert_eq!(migrations.len(), 3);
        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[1].version, 2);
        assert_eq!(migrations[2].version, 3);
    }
}
