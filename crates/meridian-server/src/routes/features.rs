//! Feature CRUD endpoints
//!
//! Handles operations on geographic features within layers

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

/// Build feature routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_features).post(create_feature))
        .route("/:id", get(get_feature).put(update_feature).delete(delete_feature))
        .route("/bulk", post(bulk_create_features).put(bulk_update_features))
        .route("/bulk/delete", post(bulk_delete_features))
}

/// Feature model
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Feature {
    /// Feature ID
    pub id: Uuid,

    /// Layer ID
    pub layer_id: Uuid,

    /// Geometry (GeoJSON format)
    pub geometry: serde_json::Value,

    /// Feature properties
    pub properties: serde_json::Value,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Created by user ID
    pub created_by: Uuid,
}

/// Create feature request
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct CreateFeatureRequest {
    /// Layer ID
    pub layer_id: Uuid,

    /// Geometry (GeoJSON format)
    pub geometry: serde_json::Value,

    /// Feature properties
    pub properties: serde_json::Value,
}

/// Update feature request
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct UpdateFeatureRequest {
    /// Geometry (GeoJSON format)
    pub geometry: Option<serde_json::Value>,

    /// Feature properties
    pub properties: Option<serde_json::Value>,
}

/// Bulk create request
#[derive(Debug, Deserialize, Validate)]
pub struct BulkCreateRequest {
    pub features: Vec<CreateFeatureRequest>,
}

/// Bulk update request
#[derive(Debug, Deserialize, Validate)]
pub struct BulkUpdateRequest {
    pub updates: Vec<BulkUpdateItem>,
}

/// Bulk update item
#[derive(Debug, Deserialize, Validate)]
pub struct BulkUpdateItem {
    pub id: Uuid,
    pub geometry: Option<serde_json::Value>,
    pub properties: Option<serde_json::Value>,
}

/// Bulk delete request
#[derive(Debug, Deserialize, Validate)]
pub struct BulkDeleteRequest {
    pub ids: Vec<Uuid>,
}

/// Feature query parameters
#[derive(Debug, Deserialize)]
pub struct FeatureQuery {
    /// Filter by layer ID
    pub layer_id: Option<Uuid>,

    /// Bounding box [minx, miny, maxx, maxy]
    pub bbox: Option<String>,

    /// Page number
    pub page: Option<u32>,

    /// Page size
    pub limit: Option<u32>,

    /// Property filter (JSON)
    pub filter: Option<String>,
}

/// List features
#[utoipa::path(
    get,
    path = "/api/v1/features",
    params(
        ("layer_id" = Option<Uuid>, Query, description = "Filter by layer ID"),
        ("bbox" = Option<String>, Query, description = "Bounding box filter"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("limit" = Option<u32>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List of features", body = Vec<Feature>)
    ),
    tag = "Features"
)]
pub async fn list_features(
    State(_state): State<AppState>,
    Query(query): Query<FeatureQuery>,
) -> ServerResult<Json<Vec<Feature>>> {
    tracing::info!("Listing features with filters: {:?}", query);

    // TODO: Implement actual database query
    let features = vec![
        Feature {
            id: Uuid::new_v4(),
            layer_id: query.layer_id.unwrap_or_else(Uuid::new_v4),
            geometry: serde_json::json!({
                "type": "Point",
                "coordinates": [0.0, 0.0]
            }),
            properties: serde_json::json!({
                "name": "Sample Feature"
            }),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        },
    ];

    Ok(Json(features))
}

/// Get feature by ID
#[utoipa::path(
    get,
    path = "/api/v1/features/{id}",
    params(
        ("id" = Uuid, Path, description = "Feature ID")
    ),
    responses(
        (status = 200, description = "Feature found", body = Feature),
        (status = 404, description = "Feature not found")
    ),
    tag = "Features"
)]
pub async fn get_feature(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<Feature>> {
    tracing::info!("Getting feature: {}", id);

    // TODO: Implement actual database query
    let feature = Feature {
        id,
        layer_id: Uuid::new_v4(),
        geometry: serde_json::json!({
            "type": "Point",
            "coordinates": [0.0, 0.0]
        }),
        properties: serde_json::json!({
            "name": "Sample Feature"
        }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: Uuid::new_v4(),
    };

    Ok(Json(feature))
}

/// Create a new feature
#[utoipa::path(
    post,
    path = "/api/v1/features",
    request_body = CreateFeatureRequest,
    responses(
        (status = 201, description = "Feature created", body = Feature),
        (status = 400, description = "Invalid request")
    ),
    tag = "Features"
)]
pub async fn create_feature(
    State(_state): State<AppState>,
    Json(request): Json<CreateFeatureRequest>,
) -> ServerResult<(StatusCode, Json<Feature>)> {
    // Validate request
    request.validate()
        .map_err(|e| ServerError::Validation(e.to_string()))?;

    // Validate geometry
    validate_geometry(&request.geometry)?;

    tracing::info!("Creating feature in layer: {}", request.layer_id);

    // TODO: Implement actual database insert
    let feature = Feature {
        id: Uuid::new_v4(),
        layer_id: request.layer_id,
        geometry: request.geometry,
        properties: request.properties,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: Uuid::new_v4(), // TODO: Get from auth context
    };

    Ok((StatusCode::CREATED, Json(feature)))
}

/// Update an existing feature
#[utoipa::path(
    put,
    path = "/api/v1/features/{id}",
    params(
        ("id" = Uuid, Path, description = "Feature ID")
    ),
    request_body = UpdateFeatureRequest,
    responses(
        (status = 200, description = "Feature updated", body = Feature),
        (status = 404, description = "Feature not found")
    ),
    tag = "Features"
)]
pub async fn update_feature(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateFeatureRequest>,
) -> ServerResult<Json<Feature>> {
    // Validate request
    request.validate()
        .map_err(|e| ServerError::Validation(e.to_string()))?;

    // Validate geometry if provided
    if let Some(ref geometry) = request.geometry {
        validate_geometry(geometry)?;
    }

    tracing::info!("Updating feature: {}", id);

    // TODO: Implement actual database update
    let feature = Feature {
        id,
        layer_id: Uuid::new_v4(),
        geometry: request.geometry.unwrap_or_else(|| serde_json::json!({
            "type": "Point",
            "coordinates": [0.0, 0.0]
        })),
        properties: request.properties.unwrap_or_else(|| serde_json::json!({})),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: Uuid::new_v4(),
    };

    Ok(Json(feature))
}

/// Delete a feature
#[utoipa::path(
    delete,
    path = "/api/v1/features/{id}",
    params(
        ("id" = Uuid, Path, description = "Feature ID")
    ),
    responses(
        (status = 204, description = "Feature deleted"),
        (status = 404, description = "Feature not found")
    ),
    tag = "Features"
)]
pub async fn delete_feature(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<StatusCode> {
    tracing::info!("Deleting feature: {}", id);

    // TODO: Implement actual database delete
    Ok(StatusCode::NO_CONTENT)
}

/// Bulk create features
pub async fn bulk_create_features(
    State(_state): State<AppState>,
    Json(request): Json<BulkCreateRequest>,
) -> ServerResult<(StatusCode, Json<Vec<Feature>>)> {
    tracing::info!("Bulk creating {} features", request.features.len());

    if request.features.is_empty() {
        return Err(ServerError::BadRequest("No features provided".to_string()));
    }

    if request.features.len() > 1000 {
        return Err(ServerError::BadRequest(
            "Maximum 1000 features per bulk operation".to_string(),
        ));
    }

    // Validate all geometries
    for feature_req in &request.features {
        validate_geometry(&feature_req.geometry)?;
    }

    // TODO: Implement actual bulk insert
    let features: Vec<Feature> = request.features
        .into_iter()
        .map(|req| Feature {
            id: Uuid::new_v4(),
            layer_id: req.layer_id,
            geometry: req.geometry,
            properties: req.properties,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        })
        .collect();

    Ok((StatusCode::CREATED, Json(features)))
}

/// Bulk update features
pub async fn bulk_update_features(
    State(_state): State<AppState>,
    Json(request): Json<BulkUpdateRequest>,
) -> ServerResult<Json<Vec<Feature>>> {
    tracing::info!("Bulk updating {} features", request.updates.len());

    if request.updates.is_empty() {
        return Err(ServerError::BadRequest("No updates provided".to_string()));
    }

    if request.updates.len() > 1000 {
        return Err(ServerError::BadRequest(
            "Maximum 1000 features per bulk operation".to_string(),
        ));
    }

    // TODO: Implement actual bulk update
    let features: Vec<Feature> = request.updates
        .into_iter()
        .map(|update| Feature {
            id: update.id,
            layer_id: Uuid::new_v4(),
            geometry: update.geometry.unwrap_or_else(|| serde_json::json!({
                "type": "Point",
                "coordinates": [0.0, 0.0]
            })),
            properties: update.properties.unwrap_or_else(|| serde_json::json!({})),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        })
        .collect();

    Ok(Json(features))
}

/// Bulk delete features
pub async fn bulk_delete_features(
    State(_state): State<AppState>,
    Json(request): Json<BulkDeleteRequest>,
) -> ServerResult<StatusCode> {
    tracing::info!("Bulk deleting {} features", request.ids.len());

    if request.ids.is_empty() {
        return Err(ServerError::BadRequest("No IDs provided".to_string()));
    }

    if request.ids.len() > 1000 {
        return Err(ServerError::BadRequest(
            "Maximum 1000 features per bulk operation".to_string(),
        ));
    }

    // TODO: Implement actual bulk delete
    Ok(StatusCode::NO_CONTENT)
}

/// Validate GeoJSON geometry
fn validate_geometry(geometry: &serde_json::Value) -> ServerResult<()> {
    // Basic validation - check for required fields
    let obj = geometry.as_object()
        .ok_or_else(|| ServerError::GeometryError("Geometry must be an object".to_string()))?;

    if !obj.contains_key("type") {
        return Err(ServerError::GeometryError(
            "Geometry must have a 'type' field".to_string(),
        ));
    }

    if !obj.contains_key("coordinates") {
        return Err(ServerError::GeometryError(
            "Geometry must have a 'coordinates' field".to_string(),
        ));
    }

    // TODO: Implement more thorough geometry validation using meridian-core

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_serialization() {
        let feature = Feature {
            id: Uuid::new_v4(),
            layer_id: Uuid::new_v4(),
            geometry: serde_json::json!({
                "type": "Point",
                "coordinates": [0.0, 0.0]
            }),
            properties: serde_json::json!({
                "name": "Test"
            }),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        };

        let json = serde_json::to_string(&feature).unwrap();
        assert!(!json.is_empty());
    }

    #[test]
    fn test_geometry_validation() {
        let valid = serde_json::json!({
            "type": "Point",
            "coordinates": [0.0, 0.0]
        });
        assert!(validate_geometry(&valid).is_ok());

        let invalid = serde_json::json!({
            "coordinates": [0.0, 0.0]
        });
        assert!(validate_geometry(&invalid).is_err());
    }
}
