//! Tile server module

pub mod cache;
pub mod etag;
pub mod handler;

pub use cache::TileCache;
pub use etag::ETagGenerator;
pub use handler::TileHandler;

use crate::encoding::CompressionFormat;
use crate::error::Result;
use crate::generation::TileGenerator;
use crate::source::TileSource;
use crate::storage::TileStorage;
use axum::{
    Router,
    routing::get,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Tile server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_addr: SocketAddr,
    /// Enable CORS
    pub cors: bool,
    /// Default compression format
    pub compression: CompressionFormat,
    /// Enable tile caching
    pub cache_enabled: bool,
    /// Cache size (number of tiles)
    pub cache_size: u64,
    /// Enable ETag support
    pub etag_enabled: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:8080".parse().unwrap(),
            cors: true,
            compression: CompressionFormat::Gzip,
            cache_enabled: true,
            cache_size: 1000,
            etag_enabled: true,
        }
    }
}

/// Tile server
pub struct TileServer<S: TileSource, T: TileStorage> {
    config: ServerConfig,
    source: Arc<S>,
    storage: Arc<T>,
    generator: Arc<TileGenerator>,
    cache: Option<Arc<TileCache>>,
}

impl<S: TileSource + 'static, T: TileStorage + 'static> TileServer<S, T> {
    /// Create a new tile server
    pub fn new(source: S, storage: T) -> Self {
        let config = ServerConfig::default();
        let cache = if config.cache_enabled {
            Some(Arc::new(TileCache::new(config.cache_size)))
        } else {
            None
        };

        Self {
            config,
            source: Arc::new(source),
            storage: Arc::new(storage),
            generator: Arc::new(TileGenerator::new()),
            cache,
        }
    }

    /// Create with custom configuration
    pub fn with_config(source: S, storage: T, config: ServerConfig) -> Self {
        let cache = if config.cache_enabled {
            Some(Arc::new(TileCache::new(config.cache_size)))
        } else {
            None
        };

        Self {
            config,
            source: Arc::new(source),
            storage: Arc::new(storage),
            generator: Arc::new(TileGenerator::new()),
            cache,
        }
    }

    /// Create the router
    pub fn router(&self) -> Router {
        let handler = TileHandler::new(
            self.source.clone(),
            self.storage.clone(),
            self.generator.clone(),
            self.cache.clone(),
            self.config.clone(),
        );

        let mut router = Router::new()
            .route("/tiles/:z/:x/:y.mvt", get(handler::handle_tile::<S, T>))
            .route("/tiles/:z/:x/:y.pbf", get(handler::handle_tile::<S, T>))
            .route("/tilejson.json", get(handler::handle_tilejson::<S, T>))
            .route("/health", get(handler::handle_health))
            .with_state(Arc::new(handler));

        if self.config.cors {
            router = router.layer(CorsLayer::permissive());
        }

        router.layer(TraceLayer::new_for_http())
    }

    /// Run the server
    pub async fn run(self) -> Result<()> {
        let router = self.router();
        let listener = tokio::net::TcpListener::bind(self.config.bind_addr).await?;

        tracing::info!("Tile server listening on {}", self.config.bind_addr);

        axum::serve(listener, router)
            .await
            .map_err(|e| crate::error::Error::Http(format!("Server error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let config = ServerConfig::default();
        assert!(config.cors);
        assert!(config.cache_enabled);
    }
}
