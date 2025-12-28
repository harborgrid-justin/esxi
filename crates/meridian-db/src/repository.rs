//! Generic spatial repository trait with CRUD operations

use crate::error::DbResult;
use crate::models::{BBox, Feature, Layer, PaginatedResponse, Pagination};
use crate::pool::Pool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Spatial query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialQueryParams {
    /// Bounding box filter
    pub bbox: Option<BBox>,
    /// Geometry to test intersection
    pub intersects: Option<String>,
    /// Geometry to test containment
    pub contains: Option<String>,
    /// Point and distance for proximity search
    pub within_distance: Option<(String, f64)>,
    /// SRID for coordinate transformation
    pub srid: Option<i32>,
    /// Pagination
    pub pagination: Option<Pagination>,
}

impl Default for SpatialQueryParams {
    fn default() -> Self {
        Self {
            bbox: None,
            intersects: None,
            contains: None,
            within_distance: None,
            srid: None,
            pagination: Some(Pagination::default()),
        }
    }
}

/// Generic spatial repository trait
#[async_trait]
pub trait SpatialRepository<T>: Send + Sync
where
    T: Send + Sync,
{
    /// Find entity by ID
    async fn find_by_id(&self, id: Uuid) -> DbResult<Option<T>>;

    /// Find all entities with pagination
    async fn find_all(&self, pagination: Pagination) -> DbResult<PaginatedResponse<T>>;

    /// Create a new entity
    async fn create(&self, entity: &T) -> DbResult<T>;

    /// Update an existing entity
    async fn update(&self, id: Uuid, entity: &T) -> DbResult<T>;

    /// Delete an entity
    async fn delete(&self, id: Uuid) -> DbResult<bool>;

    /// Count total entities
    async fn count(&self) -> DbResult<u64>;

    /// Spatial query within bounding box
    async fn find_within(&self, bbox: &BBox, pagination: Pagination) -> DbResult<PaginatedResponse<T>>;

    /// Spatial query for intersecting geometries
    async fn find_intersecting(&self, wkt: &str, srid: i32, pagination: Pagination) -> DbResult<PaginatedResponse<T>>;

    /// Spatial query for containing geometries
    async fn find_containing(&self, wkt: &str, srid: i32, pagination: Pagination) -> DbResult<PaginatedResponse<T>>;

    /// Proximity search
    async fn find_within_distance(
        &self,
        wkt: &str,
        srid: i32,
        distance: f64,
        pagination: Pagination,
    ) -> DbResult<PaginatedResponse<T>>;
}

/// Layer repository implementation
pub struct LayerRepository {
    pool: PgPool,
}

impl LayerRepository {
    /// Create a new layer repository
    pub fn new(pool: &Pool) -> Self {
        Self {
            pool: pool.inner().clone(),
        }
    }

    /// Find layers by type
    pub async fn find_by_type(&self, layer_type: &str, pagination: Pagination) -> DbResult<PaginatedResponse<Layer>> {
        let offset = pagination.offset() as i64;
        let limit = pagination.limit() as i64;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM layers WHERE layer_type = $1"
        )
        .bind(layer_type)
        .fetch_one(&self.pool)
        .await?;

        let layers = sqlx::query_as::<_, Layer>(
            "SELECT * FROM layers WHERE layer_type = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(layer_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedResponse::new(layers, total as u64, pagination))
    }

    /// Find visible layers
    pub async fn find_visible(&self, pagination: Pagination) -> DbResult<PaginatedResponse<Layer>> {
        let offset = pagination.offset() as i64;
        let limit = pagination.limit() as i64;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM layers WHERE visible = true"
        )
        .fetch_one(&self.pool)
        .await?;

        let layers = sqlx::query_as::<_, Layer>(
            "SELECT * FROM layers WHERE visible = true ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedResponse::new(layers, total as u64, pagination))
    }
}

#[async_trait]
impl SpatialRepository<Layer> for LayerRepository {
    async fn find_by_id(&self, id: Uuid) -> DbResult<Option<Layer>> {
        let layer = sqlx::query_as::<_, Layer>(
            "SELECT * FROM layers WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(layer)
    }

    async fn find_all(&self, pagination: Pagination) -> DbResult<PaginatedResponse<Layer>> {
        let offset = pagination.offset() as i64;
        let limit = pagination.limit() as i64;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM layers")
            .fetch_one(&self.pool)
            .await?;

        let layers = sqlx::query_as::<_, Layer>(
            "SELECT * FROM layers ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(PaginatedResponse::new(layers, total as u64, pagination))
    }

    async fn create(&self, layer: &Layer) -> DbResult<Layer> {
        let created = sqlx::query_as::<_, Layer>(
            r#"
            INSERT INTO layers (id, name, description, layer_type, geometry_type, srid, visible, opacity, metadata, created_at, updated_at, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#
        )
        .bind(&layer.id)
        .bind(&layer.name)
        .bind(&layer.description)
        .bind(&layer.layer_type)
        .bind(&layer.geometry_type)
        .bind(layer.srid)
        .bind(layer.visible)
        .bind(layer.opacity)
        .bind(&layer.metadata)
        .bind(layer.created_at)
        .bind(layer.updated_at)
        .bind(layer.created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(created)
    }

    async fn update(&self, id: Uuid, layer: &Layer) -> DbResult<Layer> {
        let updated = sqlx::query_as::<_, Layer>(
            r#"
            UPDATE layers
            SET name = $2, description = $3, layer_type = $4, geometry_type = $5,
                srid = $6, visible = $7, opacity = $8, metadata = $9, updated_at = $10
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(&layer.name)
        .bind(&layer.description)
        .bind(&layer.layer_type)
        .bind(&layer.geometry_type)
        .bind(layer.srid)
        .bind(layer.visible)
        .bind(layer.opacity)
        .bind(&layer.metadata)
        .bind(chrono::Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(updated)
    }

    async fn delete(&self, id: Uuid) -> DbResult<bool> {
        let result = sqlx::query("DELETE FROM layers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn count(&self) -> DbResult<u64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM layers")
            .fetch_one(&self.pool)
            .await?;

        Ok(count as u64)
    }

    async fn find_within(&self, _bbox: &BBox, pagination: Pagination) -> DbResult<PaginatedResponse<Layer>> {
        // For layers, bbox filtering might not apply - return all
        self.find_all(pagination).await
    }

    async fn find_intersecting(&self, _wkt: &str, _srid: i32, pagination: Pagination) -> DbResult<PaginatedResponse<Layer>> {
        // For layers, spatial queries might not apply - return all
        self.find_all(pagination).await
    }

    async fn find_containing(&self, _wkt: &str, _srid: i32, pagination: Pagination) -> DbResult<PaginatedResponse<Layer>> {
        // For layers, spatial queries might not apply - return all
        self.find_all(pagination).await
    }

    async fn find_within_distance(
        &self,
        _wkt: &str,
        _srid: i32,
        _distance: f64,
        pagination: Pagination,
    ) -> DbResult<PaginatedResponse<Layer>> {
        // For layers, distance queries might not apply - return all
        self.find_all(pagination).await
    }
}

/// Feature repository implementation
pub struct FeatureRepository {
    pool: PgPool,
}

impl FeatureRepository {
    /// Create a new feature repository
    pub fn new(pool: &Pool) -> Self {
        Self {
            pool: pool.inner().clone(),
        }
    }

    /// Find features by layer ID
    pub async fn find_by_layer_id(&self, layer_id: Uuid, pagination: Pagination) -> DbResult<PaginatedResponse<Feature>> {
        let offset = pagination.offset() as i64;
        let limit = pagination.limit() as i64;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM features WHERE layer_id = $1"
        )
        .bind(layer_id)
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query(
            r#"
            SELECT id, layer_id, ST_AsBinary(geometry) as geometry,
                   ST_AsGeoJSON(geometry) as geometry_json, properties, created_at, updated_at
            FROM features
            WHERE layer_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(layer_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let features: Vec<Feature> = rows
            .iter()
            .map(|row| Feature {
                id: row.get("id"),
                layer_id: row.get("layer_id"),
                geometry: row.get("geometry"),
                geometry_json: row.get::<Option<serde_json::Value>, _>("geometry_json"),
                properties: row.get("properties"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(PaginatedResponse::new(features, total as u64, pagination))
    }

    /// Batch insert features
    pub async fn batch_create(&self, features: &[Feature]) -> DbResult<usize> {
        let mut tx = self.pool.begin().await?;
        let mut count = 0;

        for feature in features {
            sqlx::query(
                r#"
                INSERT INTO features (id, layer_id, geometry, properties, created_at, updated_at)
                VALUES ($1, $2, ST_GeomFromGeoJSON($3), $4, $5, $6)
                "#
            )
            .bind(&feature.id)
            .bind(&feature.layer_id)
            .bind(&feature.geometry_json)
            .bind(&feature.properties)
            .bind(feature.created_at)
            .bind(feature.updated_at)
            .execute(&mut *tx)
            .await?;

            count += 1;
        }

        tx.commit().await?;
        Ok(count)
    }
}

#[async_trait]
impl SpatialRepository<Feature> for FeatureRepository {
    async fn find_by_id(&self, id: Uuid) -> DbResult<Option<Feature>> {
        let row = sqlx::query(
            r#"
            SELECT id, layer_id, ST_AsBinary(geometry) as geometry,
                   ST_AsGeoJSON(geometry) as geometry_json, properties, created_at, updated_at
            FROM features
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Feature {
            id: row.get("id"),
            layer_id: row.get("layer_id"),
            geometry: row.get("geometry"),
            geometry_json: row.get("geometry_json"),
            properties: row.get("properties"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    async fn find_all(&self, pagination: Pagination) -> DbResult<PaginatedResponse<Feature>> {
        let offset = pagination.offset() as i64;
        let limit = pagination.limit() as i64;

        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM features")
            .fetch_one(&self.pool)
            .await?;

        let rows = sqlx::query(
            r#"
            SELECT id, layer_id, ST_AsBinary(geometry) as geometry,
                   ST_AsGeoJSON(geometry) as geometry_json, properties, created_at, updated_at
            FROM features
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let features: Vec<Feature> = rows
            .iter()
            .map(|row| Feature {
                id: row.get("id"),
                layer_id: row.get("layer_id"),
                geometry: row.get("geometry"),
                geometry_json: row.get("geometry_json"),
                properties: row.get("properties"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(PaginatedResponse::new(features, total as u64, pagination))
    }

    async fn create(&self, feature: &Feature) -> DbResult<Feature> {
        let row = sqlx::query(
            r#"
            INSERT INTO features (id, layer_id, geometry, properties, created_at, updated_at)
            VALUES ($1, $2, ST_GeomFromGeoJSON($3), $4, $5, $6)
            RETURNING id, layer_id, ST_AsBinary(geometry) as geometry,
                      ST_AsGeoJSON(geometry) as geometry_json, properties, created_at, updated_at
            "#
        )
        .bind(&feature.id)
        .bind(&feature.layer_id)
        .bind(&feature.geometry_json)
        .bind(&feature.properties)
        .bind(feature.created_at)
        .bind(feature.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(Feature {
            id: row.get("id"),
            layer_id: row.get("layer_id"),
            geometry: row.get("geometry"),
            geometry_json: row.get("geometry_json"),
            properties: row.get("properties"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn update(&self, id: Uuid, feature: &Feature) -> DbResult<Feature> {
        let row = sqlx::query(
            r#"
            UPDATE features
            SET layer_id = $2, geometry = ST_GeomFromGeoJSON($3), properties = $4, updated_at = $5
            WHERE id = $1
            RETURNING id, layer_id, ST_AsBinary(geometry) as geometry,
                      ST_AsGeoJSON(geometry) as geometry_json, properties, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&feature.layer_id)
        .bind(&feature.geometry_json)
        .bind(&feature.properties)
        .bind(chrono::Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(Feature {
            id: row.get("id"),
            layer_id: row.get("layer_id"),
            geometry: row.get("geometry"),
            geometry_json: row.get("geometry_json"),
            properties: row.get("properties"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn delete(&self, id: Uuid) -> DbResult<bool> {
        let result = sqlx::query("DELETE FROM features WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn count(&self) -> DbResult<u64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM features")
            .fetch_one(&self.pool)
            .await?;

        Ok(count as u64)
    }

    async fn find_within(&self, bbox: &BBox, pagination: Pagination) -> DbResult<PaginatedResponse<Feature>> {
        let offset = pagination.offset() as i64;
        let limit = pagination.limit() as i64;

        let envelope = format!(
            "ST_MakeEnvelope({}, {}, {}, {}, {})",
            bbox.min_x, bbox.min_y, bbox.max_x, bbox.max_y, bbox.srid
        );

        let total: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM features WHERE ST_Within(geometry, {})",
            envelope
        ))
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query(&format!(
            r#"
            SELECT id, layer_id, ST_AsBinary(geometry) as geometry,
                   ST_AsGeoJSON(geometry) as geometry_json, properties, created_at, updated_at
            FROM features
            WHERE ST_Within(geometry, {})
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            envelope
        ))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let features: Vec<Feature> = rows
            .iter()
            .map(|row| Feature {
                id: row.get("id"),
                layer_id: row.get("layer_id"),
                geometry: row.get("geometry"),
                geometry_json: row.get("geometry_json"),
                properties: row.get("properties"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(PaginatedResponse::new(features, total as u64, pagination))
    }

    async fn find_intersecting(&self, wkt: &str, srid: i32, pagination: Pagination) -> DbResult<PaginatedResponse<Feature>> {
        let offset = pagination.offset() as i64;
        let limit = pagination.limit() as i64;

        let geom = format!("ST_GeomFromText('{}', {})", wkt, srid);

        let total: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM features WHERE ST_Intersects(geometry, {})",
            geom
        ))
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query(&format!(
            r#"
            SELECT id, layer_id, ST_AsBinary(geometry) as geometry,
                   ST_AsGeoJSON(geometry) as geometry_json, properties, created_at, updated_at
            FROM features
            WHERE ST_Intersects(geometry, {})
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            geom
        ))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let features: Vec<Feature> = rows
            .iter()
            .map(|row| Feature {
                id: row.get("id"),
                layer_id: row.get("layer_id"),
                geometry: row.get("geometry"),
                geometry_json: row.get("geometry_json"),
                properties: row.get("properties"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(PaginatedResponse::new(features, total as u64, pagination))
    }

    async fn find_containing(&self, wkt: &str, srid: i32, pagination: Pagination) -> DbResult<PaginatedResponse<Feature>> {
        let offset = pagination.offset() as i64;
        let limit = pagination.limit() as i64;

        let geom = format!("ST_GeomFromText('{}', {})", wkt, srid);

        let total: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM features WHERE ST_Contains(geometry, {})",
            geom
        ))
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query(&format!(
            r#"
            SELECT id, layer_id, ST_AsBinary(geometry) as geometry,
                   ST_AsGeoJSON(geometry) as geometry_json, properties, created_at, updated_at
            FROM features
            WHERE ST_Contains(geometry, {})
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            geom
        ))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let features: Vec<Feature> = rows
            .iter()
            .map(|row| Feature {
                id: row.get("id"),
                layer_id: row.get("layer_id"),
                geometry: row.get("geometry"),
                geometry_json: row.get("geometry_json"),
                properties: row.get("properties"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(PaginatedResponse::new(features, total as u64, pagination))
    }

    async fn find_within_distance(
        &self,
        wkt: &str,
        srid: i32,
        distance: f64,
        pagination: Pagination,
    ) -> DbResult<PaginatedResponse<Feature>> {
        let offset = pagination.offset() as i64;
        let limit = pagination.limit() as i64;

        let geom = format!("ST_GeomFromText('{}', {})", wkt, srid);

        let total: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM features WHERE ST_DWithin(geometry, {}, {})",
            geom, distance
        ))
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query(&format!(
            r#"
            SELECT id, layer_id, ST_AsBinary(geometry) as geometry,
                   ST_AsGeoJSON(geometry) as geometry_json, properties, created_at, updated_at
            FROM features
            WHERE ST_DWithin(geometry, {}, {})
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            geom, distance
        ))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let features: Vec<Feature> = rows
            .iter()
            .map(|row| Feature {
                id: row.get("id"),
                layer_id: row.get("layer_id"),
                geometry: row.get("geometry"),
                geometry_json: row.get("geometry_json"),
                properties: row.get("properties"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(PaginatedResponse::new(features, total as u64, pagination))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_query_params() {
        let params = SpatialQueryParams::default();
        assert!(params.bbox.is_none());
        assert!(params.pagination.is_some());
    }
}
