//! Meridian Server - REST API and OGC Services
//!
//! This crate provides the HTTP server implementation for the Meridian GIS Platform,
//! including RESTful API endpoints and OGC-compliant web services (WMS, WFS, WMTS).

pub mod config;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod state;

use axum::Router;
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
use utoipa::OpenApi;

pub use config::ServerConfig;
pub use error::{ServerError, ServerResult};
pub use state::AppState;

/// Initialize the Meridian server with the given configuration
///
/// # Arguments
/// * `config` - Server configuration
///
/// # Returns
/// Returns a configured Axum application ready to be served
pub async fn init_server(config: ServerConfig) -> ServerResult<Router> {
    info!("Initializing Meridian GIS Server v{}", env!("CARGO_PKG_VERSION"));

    // Initialize application state
    let state = AppState::new(config.clone()).await?;

    // Build the application router
    let app = build_router(state, &config)?;

    info!("Meridian Server initialized successfully");
    Ok(app)
}

/// Build the main application router with all routes and middleware
fn build_router(state: AppState, config: &ServerConfig) -> ServerResult<Router> {
    // Build CORS layer
    let cors = build_cors_layer(config)?;

    let app = Router::new()
        // API routes
        .nest("/api/v1", routes::api_routes())
        // OGC services
        .nest("/ogc", routes::ogc_routes())
        // Health check
        .nest("/health", routes::health_routes())
        // OpenAPI documentation
        .merge(utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", <routes::ApiDoc as OpenApi>::openapi()))
        // Add application state
        .with_state(state)
        // Add middleware layers directly
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(config.request_timeout_secs)))
        .layer(cors);

    Ok(app)
}

/// Build CORS layer from configuration
fn build_cors_layer(config: &ServerConfig) -> ServerResult<CorsLayer> {
    use tower_http::cors::Any;

    let mut cors = CorsLayer::new();

    if config.cors.allow_any_origin {
        cors = cors.allow_origin(Any);
    } else {
        // Parse allowed origins
        let origins: Vec<http::HeaderValue> = config.cors.allowed_origins.iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        cors = cors.allow_origin(origins);
    }

    // Convert method strings to Method types
    let methods: Vec<http::Method> = config.cors.allowed_methods.iter()
        .filter_map(|m| m.parse().ok())
        .collect();
    cors = cors.allow_methods(methods);

    // Convert header strings to HeaderName types
    let headers: Vec<http::header::HeaderName> = config.cors.allowed_headers.iter()
        .filter_map(|h| h.parse().ok())
        .collect();
    cors = cors.allow_headers(headers);

    if config.cors.allow_credentials {
        cors = cors.allow_credentials(true);
    }

    Ok(cors)
}

/// Start the server and listen on the configured address
///
/// # Arguments
/// * `config` - Server configuration
///
/// # Returns
/// Returns a Result indicating success or failure
pub async fn serve(config: ServerConfig) -> ServerResult<()> {
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .map_err(|e| ServerError::Configuration(format!("Invalid host/port: {}", e)))?;

    info!("Starting Meridian Server on {}", addr);

    let app = init_server(config.clone()).await?;

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| ServerError::IoError(format!("Failed to bind to {}: {}", addr, e)))?;

    info!("Server listening on http://{}", addr);
    info!("API documentation available at http://{}/swagger-ui", addr);

    axum::serve(listener, app)
        .await
        .map_err(|e| ServerError::IoError(format!("Server error: {}", e)))?;

    warn!("Server stopped");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_initialization() {
        let config = ServerConfig::default();
        let result = init_server(config).await;
        assert!(result.is_ok());
    }
}
