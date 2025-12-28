//! API route definitions
//!
//! This module contains all HTTP route handlers for the Meridian API

pub mod features;
pub mod layers;
pub mod ogc;
pub mod query;

use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::OpenApi;

use crate::state::AppState;

/// Build API v1 routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .nest("/layers", layers::routes())
        .nest("/features", features::routes())
        .nest("/query", query::routes())
        .route("/", get(api_info))
}

/// Build OGC service routes
pub fn ogc_routes() -> Router<AppState> {
    Router::new()
        .nest("/wms", ogc::wms_routes())
        .nest("/wfs", ogc::wfs_routes())
        .nest("/wmts", ogc::wmts_routes())
        .route("/", get(ogc_info))
}

/// Build health check routes
pub fn health_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/live", get(liveness_check))
}

/// API information response
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ApiInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub endpoints: Vec<EndpointInfo>,
}

/// Endpoint information
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EndpointInfo {
    pub path: String,
    pub description: String,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// API info handler
#[utoipa::path(
    get,
    path = "/api/v1",
    responses(
        (status = 200, description = "API information", body = ApiInfo)
    ),
    tag = "Info"
)]
async fn api_info() -> Json<ApiInfo> {
    Json(ApiInfo {
        name: "Meridian GIS API".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Enterprise Geographic Information System API".to_string(),
        endpoints: vec![
            EndpointInfo {
                path: "/api/v1/layers".to_string(),
                description: "Layer management endpoints".to_string(),
            },
            EndpointInfo {
                path: "/api/v1/features".to_string(),
                description: "Feature CRUD endpoints".to_string(),
            },
            EndpointInfo {
                path: "/api/v1/query".to_string(),
                description: "Spatial query endpoints".to_string(),
            },
            EndpointInfo {
                path: "/ogc".to_string(),
                description: "OGC web services".to_string(),
            },
        ],
    })
}

/// OGC services info handler
async fn ogc_info() -> Json<ApiInfo> {
    Json(ApiInfo {
        name: "Meridian OGC Services".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "OGC-compliant web services".to_string(),
        endpoints: vec![
            EndpointInfo {
                path: "/ogc/wms".to_string(),
                description: "Web Map Service (WMS)".to_string(),
            },
            EndpointInfo {
                path: "/ogc/wfs".to_string(),
                description: "Web Feature Service (WFS)".to_string(),
            },
            EndpointInfo {
                path: "/ogc/wmts".to_string(),
                description: "Web Map Tile Service (WMTS)".to_string(),
            },
        ],
    })
}

/// Health check handler
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    ),
    tag = "Health"
)]
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
    })
}

/// Readiness check handler
async fn readiness_check() -> Json<HealthResponse> {
    // TODO: Check database connectivity, cache availability, etc.
    Json(HealthResponse {
        status: "ready".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
    })
}

/// Liveness check handler
async fn liveness_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "alive".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
    })
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        api_info,
        health_check,
        layers::list_layers,
        layers::get_layer,
        layers::create_layer,
        layers::update_layer,
        layers::delete_layer,
        features::list_features,
        features::get_feature,
        features::create_feature,
        features::update_feature,
        features::delete_feature,
        query::spatial_query,
        query::intersects_query,
        query::within_query,
        query::bbox_query,
    ),
    components(
        schemas(
            ApiInfo,
            EndpointInfo,
            HealthResponse,
            layers::Layer,
            layers::CreateLayerRequest,
            layers::UpdateLayerRequest,
            features::Feature,
            features::CreateFeatureRequest,
            features::UpdateFeatureRequest,
            query::SpatialQueryRequest,
            query::SpatialQueryResponse,
        )
    ),
    tags(
        (name = "Info", description = "API information endpoints"),
        (name = "Health", description = "Health check endpoints"),
        (name = "Layers", description = "Layer management endpoints"),
        (name = "Features", description = "Feature CRUD endpoints"),
        (name = "Query", description = "Spatial query endpoints"),
        (name = "OGC", description = "OGC web services"),
    )
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_routes_creation() {
        // Just verify routes can be created
        let _routes = api_routes();
    }
}
