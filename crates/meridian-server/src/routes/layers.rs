//! Layer management endpoints
//!
//! Handles CRUD operations for GIS layers

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{error::ServerResult, state::AppState, ServerError};

/// Build layer routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_layers).post(create_layer))
        .route("/:id", get(get_layer).put(update_layer).delete(delete_layer))
        .route("/:id/metadata", get(get_layer_metadata))
        .route("/:id/style", get(get_layer_style).put(update_layer_style))
}

/// Layer model
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Layer {
    /// Layer ID
    pub id: Uuid,

    /// Layer name
    pub name: String,

    /// Layer description
    pub description: Option<String>,

    /// Layer type (vector, raster)
    pub layer_type: LayerType,

    /// Geometry type (for vector layers)
    pub geometry_type: Option<GeometryType>,

    /// Coordinate reference system (EPSG code)
    pub crs: String,

    /// Bounding box [minx, miny, maxx, maxy]
    pub bbox: Option<Vec<f64>>,

    /// Layer visibility
    pub visible: bool,

    /// Layer opacity (0.0 - 1.0)
    pub opacity: f64,

    /// Layer style
    pub style: Option<serde_json::Value>,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Created by user ID
    pub created_by: Uuid,

    /// Metadata
    pub metadata: Option<serde_json::Value>,
}

/// Layer type
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum LayerType {
    Vector,
    Raster,
    Tile,
}

/// Geometry type
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum GeometryType {
    Point,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    GeometryCollection,
}

/// Create layer request
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateLayerRequest {
    /// Layer name
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Layer description
    pub description: Option<String>,

    /// Layer type
    pub layer_type: LayerType,

    /// Geometry type (required for vector layers)
    pub geometry_type: Option<GeometryType>,

    /// CRS EPSG code (defaults to EPSG:4326)
    pub crs: Option<String>,

    /// Layer visibility
    pub visible: Option<bool>,

    /// Layer opacity
    #[validate(range(min = 0.0, max = 1.0))]
    pub opacity: Option<f64>,

    /// Layer style
    pub style: Option<serde_json::Value>,

    /// Metadata
    pub metadata: Option<serde_json::Value>,
}

/// Update layer request
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateLayerRequest {
    /// Layer name
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,

    /// Layer description
    pub description: Option<String>,

    /// Layer visibility
    pub visible: Option<bool>,

    /// Layer opacity
    #[validate(range(min = 0.0, max = 1.0))]
    pub opacity: Option<f64>,

    /// Layer style
    pub style: Option<serde_json::Value>,

    /// Metadata
    pub metadata: Option<serde_json::Value>,
}

/// Layer query parameters
#[derive(Debug, Deserialize)]
pub struct LayerQuery {
    /// Filter by layer type
    pub layer_type: Option<LayerType>,

    /// Filter by visibility
    pub visible: Option<bool>,

    /// Page number
    pub page: Option<u32>,

    /// Page size
    pub limit: Option<u32>,

    /// Sort by field
    pub sort_by: Option<String>,

    /// Sort order (asc, desc)
    pub order: Option<String>,
}

/// List layers
#[utoipa::path(
    get,
    path = "/api/v1/layers",
    params(
        ("layer_type" = Option<String>, Query, description = "Filter by layer type"),
        ("visible" = Option<bool>, Query, description = "Filter by visibility"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List of layers", body = Vec<Layer>)
    ),
    tag = "Layers"
)]
pub async fn list_layers(
    State(_state): State<AppState>,
    Query(query): Query<LayerQuery>,
) -> ServerResult<Json<Vec<Layer>>> {
    tracing::info!("Listing layers with filters: {:?}", query);

    // TODO: Implement actual database query
    let layers = vec![
        Layer {
            id: Uuid::new_v4(),
            name: "Sample Layer".to_string(),
            description: Some("A sample GIS layer".to_string()),
            layer_type: LayerType::Vector,
            geometry_type: Some(GeometryType::Polygon),
            crs: "EPSG:4326".to_string(),
            bbox: Some(vec![-180.0, -90.0, 180.0, 90.0]),
            visible: true,
            opacity: 1.0,
            style: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
            metadata: None,
        },
    ];

    Ok(Json(layers))
}

/// Get layer by ID
#[utoipa::path(
    get,
    path = "/api/v1/layers/{id}",
    params(
        ("id" = Uuid, Path, description = "Layer ID")
    ),
    responses(
        (status = 200, description = "Layer found", body = Layer),
        (status = 404, description = "Layer not found")
    ),
    tag = "Layers"
)]
pub async fn get_layer(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<Layer>> {
    tracing::info!("Getting layer: {}", id);

    // TODO: Implement actual database query
    // For now, return a mock layer
    let layer = Layer {
        id,
        name: "Sample Layer".to_string(),
        description: Some("A sample GIS layer".to_string()),
        layer_type: LayerType::Vector,
        geometry_type: Some(GeometryType::Polygon),
        crs: "EPSG:4326".to_string(),
        bbox: Some(vec![-180.0, -90.0, 180.0, 90.0]),
        visible: true,
        opacity: 1.0,
        style: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: Uuid::new_v4(),
        metadata: None,
    };

    Ok(Json(layer))
}

/// Create a new layer
#[utoipa::path(
    post,
    path = "/api/v1/layers",
    request_body = CreateLayerRequest,
    responses(
        (status = 201, description = "Layer created", body = Layer),
        (status = 400, description = "Invalid request")
    ),
    tag = "Layers"
)]
pub async fn create_layer(
    State(_state): State<AppState>,
    Json(request): Json<CreateLayerRequest>,
) -> ServerResult<(StatusCode, Json<Layer>)> {
    // Validate request
    request.validate()
        .map_err(|e| ServerError::Validation(e.to_string()))?;

    tracing::info!("Creating layer: {}", request.name);

    // TODO: Implement actual database insert
    let layer = Layer {
        id: Uuid::new_v4(),
        name: request.name,
        description: request.description,
        layer_type: request.layer_type,
        geometry_type: request.geometry_type,
        crs: request.crs.unwrap_or_else(|| "EPSG:4326".to_string()),
        bbox: None,
        visible: request.visible.unwrap_or(true),
        opacity: request.opacity.unwrap_or(1.0),
        style: request.style,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: Uuid::new_v4(), // TODO: Get from auth context
        metadata: request.metadata,
    };

    Ok((StatusCode::CREATED, Json(layer)))
}

/// Update an existing layer
#[utoipa::path(
    put,
    path = "/api/v1/layers/{id}",
    params(
        ("id" = Uuid, Path, description = "Layer ID")
    ),
    request_body = UpdateLayerRequest,
    responses(
        (status = 200, description = "Layer updated", body = Layer),
        (status = 404, description = "Layer not found")
    ),
    tag = "Layers"
)]
pub async fn update_layer(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateLayerRequest>,
) -> ServerResult<Json<Layer>> {
    // Validate request
    request.validate()
        .map_err(|e| ServerError::Validation(e.to_string()))?;

    tracing::info!("Updating layer: {}", id);

    // TODO: Implement actual database update
    let layer = Layer {
        id,
        name: request.name.unwrap_or_else(|| "Updated Layer".to_string()),
        description: request.description,
        layer_type: LayerType::Vector,
        geometry_type: Some(GeometryType::Polygon),
        crs: "EPSG:4326".to_string(),
        bbox: None,
        visible: request.visible.unwrap_or(true),
        opacity: request.opacity.unwrap_or(1.0),
        style: request.style,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: Uuid::new_v4(),
        metadata: request.metadata,
    };

    Ok(Json(layer))
}

/// Delete a layer
#[utoipa::path(
    delete,
    path = "/api/v1/layers/{id}",
    params(
        ("id" = Uuid, Path, description = "Layer ID")
    ),
    responses(
        (status = 204, description = "Layer deleted"),
        (status = 404, description = "Layer not found")
    ),
    tag = "Layers"
)]
pub async fn delete_layer(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<StatusCode> {
    tracing::info!("Deleting layer: {}", id);

    // TODO: Implement actual database delete
    Ok(StatusCode::NO_CONTENT)
}

/// Get layer metadata
pub async fn get_layer_metadata(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<serde_json::Value>> {
    tracing::info!("Getting metadata for layer: {}", id);

    // TODO: Implement actual metadata retrieval
    Ok(Json(serde_json::json!({
        "layer_id": id,
        "feature_count": 0,
        "extent": null,
        "attributes": []
    })))
}

/// Get layer style
pub async fn get_layer_style(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<serde_json::Value>> {
    tracing::info!("Getting style for layer: {}", id);

    // TODO: Implement actual style retrieval
    Ok(Json(serde_json::json!({
        "layer_id": id,
        "style": {}
    })))
}

/// Update layer style
pub async fn update_layer_style(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(style): Json<serde_json::Value>,
) -> ServerResult<Json<serde_json::Value>> {
    tracing::info!("Updating style for layer: {}", id);

    // TODO: Implement actual style update
    Ok(Json(serde_json::json!({
        "layer_id": id,
        "style": style
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_serialization() {
        let layer = Layer {
            id: Uuid::new_v4(),
            name: "Test Layer".to_string(),
            description: None,
            layer_type: LayerType::Vector,
            geometry_type: Some(GeometryType::Point),
            crs: "EPSG:4326".to_string(),
            bbox: None,
            visible: true,
            opacity: 1.0,
            style: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
            metadata: None,
        };

        let json = serde_json::to_string(&layer).unwrap();
        assert!(!json.is_empty());
    }
}
