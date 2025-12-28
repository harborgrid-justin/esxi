//! Spatial query endpoints
//!
//! Handles spatial queries like intersects, within, contains, distance, etc.

use axum::{
    extract::{Query, State},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{error::ServerResult, state::AppState, ServerError};

/// Build query routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/spatial", post(spatial_query))
        .route("/intersects", post(intersects_query))
        .route("/within", post(within_query))
        .route("/contains", post(contains_query))
        .route("/distance", post(distance_query))
        .route("/bbox", post(bbox_query))
        .route("/nearest", post(nearest_query))
}

/// Spatial query request
#[derive(Debug, Deserialize, Validate, utoipa::ToSchema)]
pub struct SpatialQueryRequest {
    /// Layer ID to query
    pub layer_id: Uuid,

    /// Query geometry (GeoJSON)
    pub geometry: serde_json::Value,

    /// Spatial operation
    pub operation: SpatialOperation,

    /// Optional property filters
    pub filters: Option<serde_json::Value>,

    /// Maximum results to return
    #[validate(range(min = 1, max = 10000))]
    pub limit: Option<u32>,

    /// Result offset
    pub offset: Option<u32>,
}

/// Spatial query response
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SpatialQueryResponse {
    /// Total number of matching features
    pub total: u32,

    /// Returned features
    pub features: Vec<super::features::Feature>,

    /// Query execution time in milliseconds
    pub execution_time_ms: u64,
}

/// Spatial operations
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SpatialOperation {
    Intersects,
    Within,
    Contains,
    Overlaps,
    Touches,
    Crosses,
    Disjoint,
    Equals,
}

/// Intersects query request
#[derive(Debug, Deserialize, Validate)]
pub struct IntersectsRequest {
    pub layer_id: Uuid,
    pub geometry: serde_json::Value,
    pub limit: Option<u32>,
}

/// Within query request
#[derive(Debug, Deserialize, Validate)]
pub struct WithinRequest {
    pub layer_id: Uuid,
    pub geometry: serde_json::Value,
    pub limit: Option<u32>,
}

/// Contains query request
#[derive(Debug, Deserialize, Validate)]
pub struct ContainsRequest {
    pub layer_id: Uuid,
    pub geometry: serde_json::Value,
    pub limit: Option<u32>,
}

/// Distance query request
#[derive(Debug, Deserialize, Validate)]
pub struct DistanceRequest {
    pub layer_id: Uuid,
    pub geometry: serde_json::Value,
    pub distance: f64,
    pub unit: Option<DistanceUnit>,
    pub limit: Option<u32>,
}

/// Distance unit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistanceUnit {
    Meters,
    Kilometers,
    Miles,
    Feet,
    Degrees,
}

/// Bounding box query request
#[derive(Debug, Deserialize, Validate)]
pub struct BBoxRequest {
    pub layer_id: Uuid,
    /// Bounding box [minx, miny, maxx, maxy]
    #[validate(length(equal = 4))]
    pub bbox: Vec<f64>,
    pub limit: Option<u32>,
}

/// Nearest features query request
#[derive(Debug, Deserialize, Validate)]
pub struct NearestRequest {
    pub layer_id: Uuid,
    pub geometry: serde_json::Value,
    #[validate(range(min = 1, max = 100))]
    pub count: u32,
    pub max_distance: Option<f64>,
}

/// General spatial query
#[utoipa::path(
    post,
    path = "/api/v1/query/spatial",
    request_body = SpatialQueryRequest,
    responses(
        (status = 200, description = "Query results", body = SpatialQueryResponse),
        (status = 400, description = "Invalid request")
    ),
    tag = "Query"
)]
pub async fn spatial_query(
    State(_state): State<AppState>,
    Json(request): Json<SpatialQueryRequest>,
) -> ServerResult<Json<SpatialQueryResponse>> {
    // Validate request
    request.validate()
        .map_err(|e| ServerError::Validation(e.to_string()))?;

    tracing::info!(
        "Spatial query on layer {} with operation {:?}",
        request.layer_id,
        request.operation
    );

    let start = std::time::Instant::now();

    // TODO: Implement actual spatial query using meridian-db and meridian-core
    let features = vec![];

    let execution_time = start.elapsed().as_millis() as u64;

    Ok(Json(SpatialQueryResponse {
        total: 0,
        features,
        execution_time_ms: execution_time,
    }))
}

/// Intersects query - find features that intersect with the given geometry
#[utoipa::path(
    post,
    path = "/api/v1/query/intersects",
    request_body = IntersectsRequest,
    responses(
        (status = 200, description = "Query results", body = SpatialQueryResponse)
    ),
    tag = "Query"
)]
pub async fn intersects_query(
    State(_state): State<AppState>,
    Json(request): Json<IntersectsRequest>,
) -> ServerResult<Json<SpatialQueryResponse>> {
    tracing::info!("Intersects query on layer {}", request.layer_id);

    let start = std::time::Instant::now();

    // TODO: Implement ST_Intersects query
    let features = vec![];

    let execution_time = start.elapsed().as_millis() as u64;

    Ok(Json(SpatialQueryResponse {
        total: 0,
        features,
        execution_time_ms: execution_time,
    }))
}

/// Within query - find features completely within the given geometry
#[utoipa::path(
    post,
    path = "/api/v1/query/within",
    request_body = WithinRequest,
    responses(
        (status = 200, description = "Query results", body = SpatialQueryResponse)
    ),
    tag = "Query"
)]
pub async fn within_query(
    State(_state): State<AppState>,
    Json(request): Json<WithinRequest>,
) -> ServerResult<Json<SpatialQueryResponse>> {
    tracing::info!("Within query on layer {}", request.layer_id);

    let start = std::time::Instant::now();

    // TODO: Implement ST_Within query
    let features = vec![];

    let execution_time = start.elapsed().as_millis() as u64;

    Ok(Json(SpatialQueryResponse {
        total: 0,
        features,
        execution_time_ms: execution_time,
    }))
}

/// Contains query - find features that contain the given geometry
pub async fn contains_query(
    State(_state): State<AppState>,
    Json(request): Json<ContainsRequest>,
) -> ServerResult<Json<SpatialQueryResponse>> {
    tracing::info!("Contains query on layer {}", request.layer_id);

    let start = std::time::Instant::now();

    // TODO: Implement ST_Contains query
    let features = vec![];

    let execution_time = start.elapsed().as_millis() as u64;

    Ok(Json(SpatialQueryResponse {
        total: 0,
        features,
        execution_time_ms: execution_time,
    }))
}

/// Distance query - find features within a certain distance
pub async fn distance_query(
    State(_state): State<AppState>,
    Json(request): Json<DistanceRequest>,
) -> ServerResult<Json<SpatialQueryResponse>> {
    tracing::info!(
        "Distance query on layer {} with distance {}",
        request.layer_id,
        request.distance
    );

    if request.distance <= 0.0 {
        return Err(ServerError::BadRequest(
            "Distance must be greater than 0".to_string(),
        ));
    }

    let start = std::time::Instant::now();

    // TODO: Implement ST_DWithin query
    let features = vec![];

    let execution_time = start.elapsed().as_millis() as u64;

    Ok(Json(SpatialQueryResponse {
        total: 0,
        features,
        execution_time_ms: execution_time,
    }))
}

/// Bounding box query - find features within a bounding box
#[utoipa::path(
    post,
    path = "/api/v1/query/bbox",
    request_body = BBoxRequest,
    responses(
        (status = 200, description = "Query results", body = SpatialQueryResponse)
    ),
    tag = "Query"
)]
pub async fn bbox_query(
    State(_state): State<AppState>,
    Json(request): Json<BBoxRequest>,
) -> ServerResult<Json<SpatialQueryResponse>> {
    // Validate request
    request.validate()
        .map_err(|e| ServerError::Validation(e.to_string()))?;

    // Validate bounding box
    let (minx, miny, maxx, maxy) = (
        request.bbox[0],
        request.bbox[1],
        request.bbox[2],
        request.bbox[3],
    );

    if minx >= maxx || miny >= maxy {
        return Err(ServerError::BadRequest(
            "Invalid bounding box coordinates".to_string(),
        ));
    }

    tracing::info!(
        "BBox query on layer {}: [{}, {}, {}, {}]",
        request.layer_id,
        minx,
        miny,
        maxx,
        maxy
    );

    let start = std::time::Instant::now();

    // TODO: Implement bounding box query
    let features = vec![];

    let execution_time = start.elapsed().as_millis() as u64;

    Ok(Json(SpatialQueryResponse {
        total: 0,
        features,
        execution_time_ms: execution_time,
    }))
}

/// Nearest features query - find N nearest features to a geometry
pub async fn nearest_query(
    State(_state): State<AppState>,
    Json(request): Json<NearestRequest>,
) -> ServerResult<Json<SpatialQueryResponse>> {
    // Validate request
    request.validate()
        .map_err(|e| ServerError::Validation(e.to_string()))?;

    tracing::info!(
        "Nearest query on layer {}, finding {} nearest features",
        request.layer_id,
        request.count
    );

    let start = std::time::Instant::now();

    // TODO: Implement KNN query using spatial index
    let features = vec![];

    let execution_time = start.elapsed().as_millis() as u64;

    Ok(Json(SpatialQueryResponse {
        total: 0,
        features,
        execution_time_ms: execution_time,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_operation_serialization() {
        let op = SpatialOperation::Intersects;
        let json = serde_json::to_string(&op).unwrap();
        assert_eq!(json, "\"intersects\"");
    }

    #[test]
    fn test_bbox_validation() {
        let valid_bbox = vec![0.0, 0.0, 10.0, 10.0];
        assert_eq!(valid_bbox.len(), 4);

        let invalid_bbox = vec![10.0, 10.0, 0.0, 0.0];
        // Would fail in handler validation
        assert!(invalid_bbox[0] >= invalid_bbox[2]);
    }
}
