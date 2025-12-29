//! PostGIS tile source

use crate::encoding::mvt::MvtValue;
use crate::error::{Error, Result};
use crate::generation::SourceFeature;
use crate::source::{TileSource, SourceMetadata, LayerMetadata};
use crate::tile::bounds::MercatorBounds;
use crate::tile::coordinate::TileCoordinate;
use async_trait::async_trait;
use geozero::wkb;
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;

/// PostGIS tile source configuration
#[derive(Debug, Clone)]
pub struct PostGISConfig {
    /// Database connection string
    pub connection_string: String,
    /// Layer configurations
    pub layers: Vec<LayerConfig>,
    /// SRID for queries (default: 3857 for Web Mercator)
    pub srid: i32,
    /// Maximum features per tile
    pub max_features: Option<usize>,
}

/// Layer configuration for PostGIS
#[derive(Debug, Clone)]
pub struct LayerConfig {
    /// Layer name in the tile
    pub name: String,
    /// Table or view name
    pub table: String,
    /// Schema name (default: "public")
    pub schema: String,
    /// Geometry column name
    pub geometry_column: String,
    /// ID column name (optional)
    pub id_column: Option<String>,
    /// Property columns to include
    pub properties: Vec<String>,
    /// Minimum zoom level
    pub min_zoom: u8,
    /// Maximum zoom level
    pub max_zoom: u8,
    /// WHERE clause filter (optional)
    pub filter: Option<String>,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            table: String::new(),
            schema: "public".to_string(),
            geometry_column: "geom".to_string(),
            id_column: None,
            properties: Vec::new(),
            min_zoom: 0,
            max_zoom: 24,
            filter: None,
        }
    }
}

/// PostGIS tile source
pub struct PostGISSource {
    pool: PgPool,
    config: PostGISConfig,
}

impl PostGISSource {
    /// Create a new PostGIS source
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = PgPool::connect(connection_string).await?;
        let config = PostGISConfig {
            connection_string: connection_string.to_string(),
            layers: Vec::new(),
            srid: 3857,
            max_features: Some(10000),
        };

        Ok(Self { pool, config })
    }

    /// Create with configuration
    pub async fn with_config(config: PostGISConfig) -> Result<Self> {
        let pool = PgPool::connect(&config.connection_string).await?;
        Ok(Self { pool, config })
    }

    /// Add a layer configuration
    pub fn add_layer(mut self, layer: LayerConfig) -> Self {
        self.config.layers.push(layer);
        self
    }

    /// Build SQL query for a layer
    fn build_query(&self, layer: &LayerConfig, bounds: &MercatorBounds) -> String {
        let mut columns = vec![format!(
            "ST_AsBinary(ST_Transform({}, {})) as geom",
            layer.geometry_column, self.config.srid
        )];

        if let Some(ref id_col) = layer.id_column {
            columns.push(id_col.clone());
        }

        columns.extend(layer.properties.clone());

        let mut query = format!(
            "SELECT {} FROM {}.{}",
            columns.join(", "),
            layer.schema,
            layer.table
        );

        // Add spatial filter
        query.push_str(&format!(
            " WHERE ST_Intersects({}, ST_MakeEnvelope({}, {}, {}, {}, {}))",
            layer.geometry_column,
            bounds.min_x,
            bounds.min_y,
            bounds.max_x,
            bounds.max_y,
            self.config.srid
        ));

        // Add custom filter if present
        if let Some(ref filter) = layer.filter {
            query.push_str(&format!(" AND ({})", filter));
        }

        // Add limit
        if let Some(max) = self.config.max_features {
            query.push_str(&format!(" LIMIT {}", max));
        }

        query
    }

    /// Fetch features for a layer
    async fn fetch_layer_features(
        &self,
        layer: &LayerConfig,
        tile: TileCoordinate,
        bounds: &MercatorBounds,
    ) -> Result<Vec<SourceFeature>> {
        // Check zoom range
        if tile.z < layer.min_zoom || tile.z > layer.max_zoom {
            return Ok(Vec::new());
        }

        let query = self.build_query(layer, bounds);
        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        let mut features = Vec::new();

        for row in rows {
            // Parse geometry from WKB
            let wkb_data: Vec<u8> = row.try_get("geom")?;
            let geometry = wkb::wkb_to_geom(&wkb_data)
                .map_err(|e| Error::geometry(format!("Failed to parse WKB: {}", e)))?;

            // Get ID if present
            let id = if let Some(ref id_col) = layer.id_column {
                row.try_get::<i64, _>(id_col.as_str())
                    .ok()
                    .map(|v| v as u64)
            } else {
                None
            };

            // Get properties
            let mut properties = HashMap::new();
            for prop in &layer.properties {
                if let Some(value) = self.get_column_value(&row, prop)? {
                    properties.insert(prop.clone(), value);
                }
            }

            features.push(SourceFeature {
                id,
                layer: layer.name.clone(),
                geometry,
                properties,
            });
        }

        Ok(features)
    }

    /// Get a column value as MvtValue
    fn get_column_value(&self, row: &sqlx::postgres::PgRow, column: &str) -> Result<Option<MvtValue>> {
        // Try different types
        if let Ok(v) = row.try_get::<String, _>(column) {
            return Ok(Some(MvtValue::String(v)));
        }
        if let Ok(v) = row.try_get::<i64, _>(column) {
            return Ok(Some(MvtValue::Int(v)));
        }
        if let Ok(v) = row.try_get::<i32, _>(column) {
            return Ok(Some(MvtValue::Int(v as i64)));
        }
        if let Ok(v) = row.try_get::<f64, _>(column) {
            return Ok(Some(MvtValue::Double(v)));
        }
        if let Ok(v) = row.try_get::<f32, _>(column) {
            return Ok(Some(MvtValue::Float(v)));
        }
        if let Ok(v) = row.try_get::<bool, _>(column) {
            return Ok(Some(MvtValue::Bool(v)));
        }

        Ok(None)
    }
}

#[async_trait]
impl TileSource for PostGISSource {
    async fn get_features(
        &self,
        tile: TileCoordinate,
        bounds: &MercatorBounds,
    ) -> Result<Vec<SourceFeature>> {
        let mut all_features = Vec::new();

        for layer in &self.config.layers {
            let features = self.fetch_layer_features(layer, tile, bounds).await?;
            all_features.extend(features);
        }

        Ok(all_features)
    }

    fn max_zoom(&self) -> u8 {
        self.config
            .layers
            .iter()
            .map(|l| l.max_zoom)
            .max()
            .unwrap_or(crate::MAX_ZOOM_LEVEL)
    }

    fn min_zoom(&self) -> u8 {
        self.config
            .layers
            .iter()
            .map(|l| l.min_zoom)
            .min()
            .unwrap_or(crate::MIN_ZOOM_LEVEL)
    }

    async fn layers(&self) -> Result<Vec<String>> {
        Ok(self.config.layers.iter().map(|l| l.name.clone()).collect())
    }

    async fn metadata(&self) -> Result<SourceMetadata> {
        Ok(SourceMetadata {
            name: "PostGIS Source".to_string(),
            min_zoom: self.min_zoom(),
            max_zoom: self.max_zoom(),
            layers: self
                .config
                .layers
                .iter()
                .map(|l| LayerMetadata {
                    name: l.name.clone(),
                    description: None,
                    min_zoom: l.min_zoom,
                    max_zoom: l.max_zoom,
                    geometry_type: None,
                    fields: Vec::new(),
                })
                .collect(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_config() {
        let config = LayerConfig {
            name: "roads".to_string(),
            table: "osm_roads".to_string(),
            ..Default::default()
        };

        assert_eq!(config.name, "roads");
        assert_eq!(config.schema, "public");
    }
}
