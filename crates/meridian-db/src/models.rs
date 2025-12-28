//! Database models for Meridian GIS Platform

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Layer model representing a GIS layer
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Layer {
    /// Unique identifier
    pub id: Uuid,
    /// Layer name
    pub name: String,
    /// Layer description
    pub description: Option<String>,
    /// Layer type (vector, raster, etc.)
    pub layer_type: String,
    /// Geometry type (point, linestring, polygon, etc.)
    pub geometry_type: Option<String>,
    /// SRID (Spatial Reference System Identifier)
    pub srid: i32,
    /// Layer visibility
    pub visible: bool,
    /// Layer opacity (0.0 to 1.0)
    pub opacity: f64,
    /// Layer metadata as JSON
    pub metadata: serde_json::Value,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Created by user ID
    pub created_by: Option<Uuid>,
}

impl Layer {
    /// Create a new layer
    pub fn new(name: String, layer_type: String, srid: i32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            layer_type,
            geometry_type: None,
            srid,
            visible: true,
            opacity: 1.0,
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
            created_by: None,
        }
    }
}

/// Feature model with geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// Unique identifier
    pub id: Uuid,
    /// Layer ID
    pub layer_id: Uuid,
    /// Geometry in WKB format
    #[serde(skip)]
    pub geometry: Option<Vec<u8>>,
    /// Geometry as GeoJSON
    pub geometry_json: Option<serde_json::Value>,
    /// Feature properties
    pub properties: serde_json::Value,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Feature {
    /// Create a new feature
    pub fn new(layer_id: Uuid, properties: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            layer_id,
            geometry: None,
            geometry_json: None,
            properties,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Spatial index metadata
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SpatialIndex {
    /// Index name
    pub name: String,
    /// Table name
    pub table_name: String,
    /// Column name
    pub column_name: String,
    /// Index type (gist, brin, etc.)
    pub index_type: String,
    /// Whether index is valid
    pub is_valid: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Layer style configuration
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LayerStyle {
    /// Unique identifier
    pub id: Uuid,
    /// Layer ID
    pub layer_id: Uuid,
    /// Style name
    pub name: String,
    /// Style definition (JSON)
    pub style: serde_json::Value,
    /// Whether this is the default style
    pub is_default: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Metadata for layers and features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Key-value pairs
    pub properties: serde_json::Value,
    /// Tags
    pub tags: Vec<String>,
    /// Custom attributes
    pub attributes: Option<serde_json::Value>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            properties: serde_json::json!({}),
            tags: Vec::new(),
            attributes: None,
        }
    }
}

/// Bounding box for spatial queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BBox {
    /// Minimum X coordinate
    pub min_x: f64,
    /// Minimum Y coordinate
    pub min_y: f64,
    /// Maximum X coordinate
    pub max_x: f64,
    /// Maximum Y coordinate
    pub max_y: f64,
    /// SRID
    pub srid: i32,
}

impl BBox {
    /// Create a new bounding box
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64, srid: i32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
            srid,
        }
    }

    /// Convert to WKT polygon
    pub fn to_wkt(&self) -> String {
        format!(
            "POLYGON(({} {},{} {},{} {},{} {},{} {}))",
            self.min_x,
            self.min_y,
            self.max_x,
            self.min_y,
            self.max_x,
            self.max_y,
            self.min_x,
            self.max_y,
            self.min_x,
            self.min_y
        )
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Page number (0-indexed)
    pub page: u32,
    /// Items per page
    pub page_size: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: 100,
        }
    }
}

impl Pagination {
    /// Create new pagination
    pub fn new(page: u32, page_size: u32) -> Self {
        Self { page, page_size }
    }

    /// Get offset
    pub fn offset(&self) -> u32 {
        self.page * self.page_size
    }

    /// Get limit
    pub fn limit(&self) -> u32 {
        self.page_size
    }
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Items in current page
    pub items: Vec<T>,
    /// Total count
    pub total: u64,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
    /// Total pages
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(items: Vec<T>, total: u64, pagination: Pagination) -> Self {
        let total_pages = ((total as f64) / (pagination.page_size as f64)).ceil() as u32;
        Self {
            items,
            total,
            page: pagination.page,
            page_size: pagination.page_size,
            total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = Layer::new("Test Layer".to_string(), "vector".to_string(), 4326);
        assert_eq!(layer.name, "Test Layer");
        assert_eq!(layer.srid, 4326);
        assert!(layer.visible);
    }

    #[test]
    fn test_bbox_wkt() {
        let bbox = BBox::new(-180.0, -90.0, 180.0, 90.0, 4326);
        let wkt = bbox.to_wkt();
        assert!(wkt.contains("POLYGON"));
        assert!(wkt.contains("-180"));
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(2, 50);
        assert_eq!(pagination.offset(), 100);
        assert_eq!(pagination.limit(), 50);
    }
}
