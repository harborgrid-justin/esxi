//! HTTP request handlers for tile server

use crate::encoding::{compress, CompressionFormat, MvtEncoder};
use crate::error::{Error, Result};
use crate::generation::TileGenerator;
use crate::server::{ServerConfig, TileCache};
use crate::source::TileSource;
use crate::storage::TileStorage;
use crate::tile::coordinate::TileCoordinate;
use crate::tilejson::TileJSON;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

/// Tile handler state
pub struct TileHandler<S: TileSource, T: TileStorage> {
    source: Arc<S>,
    storage: Arc<T>,
    generator: Arc<TileGenerator>,
    cache: Option<Arc<TileCache>>,
    config: ServerConfig,
}

impl<S: TileSource, T: TileStorage> TileHandler<S, T> {
    /// Create a new tile handler
    pub fn new(
        source: Arc<S>,
        storage: Arc<T>,
        generator: Arc<TileGenerator>,
        cache: Option<Arc<TileCache>>,
        config: ServerConfig,
    ) -> Self {
        Self {
            source,
            storage,
            generator,
            cache,
            config,
        }
    }
}

/// Handle tile requests
pub async fn handle_tile<S: TileSource, T: TileStorage>(
    State(handler): State<Arc<TileHandler<S, T>>>,
    Path((z, x, y)): Path<(u8, u32, u32)>,
) -> Result<Response> {
    // Parse tile coordinate
    let tile = TileCoordinate::new(z, x, y);
    tile.validate()?;

    // Check cache first
    if let Some(ref cache) = handler.cache {
        if let Some(cached) = cache.get(&tile).await {
            return Ok(create_tile_response(
                cached,
                handler.config.compression,
                true,
            ));
        }
    }

    // Try storage
    if let Some(stored) = handler.storage.get_tile(tile).await? {
        // Cache the result
        if let Some(ref cache) = handler.cache {
            cache.put(tile, stored.clone()).await;
        }

        return Ok(create_tile_response(
            stored,
            handler.config.compression,
            false,
        ));
    }

    // Generate tile
    let tile_bounds = crate::tile::bounds::MercatorBounds::from_tile(&tile);

    if let Some(mvt_tile) = handler.generator.generate(&*handler.source, tile).await? {
        // Encode to MVT
        let encoder = MvtEncoder::new();
        let mvt_data = encoder.encode(&mvt_tile)?;

        // Compress if needed
        let data = compress(&mvt_data, handler.config.compression)?;

        // Store for future requests
        let _ = handler.storage.put_tile(tile, data.clone()).await;

        // Cache the result
        if let Some(ref cache) = handler.cache {
            cache.put(tile, data.clone()).await;
        }

        Ok(create_tile_response(
            data,
            handler.config.compression,
            false,
        ))
    } else {
        // No data for this tile
        Ok((StatusCode::NO_CONTENT, "").into_response())
    }
}

/// Handle TileJSON requests
pub async fn handle_tilejson<S: TileSource, T: TileStorage>(
    State(handler): State<Arc<TileHandler<S, T>>>,
) -> Result<Json<TileJSON>> {
    let metadata = handler.source.metadata().await?;

    let tilejson = TileJSON {
        tilejson: "3.0.0".to_string(),
        name: Some(metadata.name),
        description: metadata.description,
        version: Some("1.0.0".to_string()),
        attribution: metadata.attribution,
        scheme: Some("xyz".to_string()),
        tiles: vec!["http://localhost:8080/tiles/{z}/{x}/{y}.mvt".to_string()],
        minzoom: Some(metadata.min_zoom),
        maxzoom: Some(metadata.max_zoom),
        bounds: metadata.bounds,
        center: metadata.center,
        vector_layers: Some(
            metadata
                .layers
                .into_iter()
                .map(|l| crate::tilejson::VectorLayer {
                    id: l.name.clone(),
                    description: l.description,
                    minzoom: Some(l.min_zoom),
                    maxzoom: Some(l.max_zoom),
                    fields: l
                        .fields
                        .into_iter()
                        .map(|f| (f.name, f.field_type.as_str().to_string()))
                        .collect(),
                })
                .collect(),
        ),
    };

    Ok(Json(tilejson))
}

/// Handle health check
pub async fn handle_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": crate::VERSION,
    }))
}

/// Create tile response with appropriate headers
fn create_tile_response(
    data: Vec<u8>,
    compression: CompressionFormat,
    from_cache: bool,
) -> Response {
    let mut headers = HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE,
        "application/vnd.mapbox-vector-tile"
            .parse()
            .unwrap(),
    );

    if let Some(encoding) = compression.content_encoding() {
        headers.insert(header::CONTENT_ENCODING, encoding.parse().unwrap());
    }

    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=86400".parse().unwrap(),
    );

    if from_cache {
        headers.insert("X-Cache", "HIT".parse().unwrap());
    } else {
        headers.insert("X-Cache", "MISS".parse().unwrap());
    }

    (StatusCode::OK, headers, data).into_response()
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Error::TileNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Error::InvalidCoordinate(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::InvalidZoom { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        (
            status,
            Json(serde_json::json!({
                "error": message,
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_response() {
        let data = vec![1, 2, 3];
        let response = create_tile_response(data, CompressionFormat::Gzip, false);
        // Response testing would require axum test utilities
    }
}
