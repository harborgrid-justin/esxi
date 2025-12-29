//! Asynchronous tile loader for fetching map tiles from remote sources.

use super::{TileCoord, TileData};
use crate::error::{MapEngineError, Result};
use futures::stream::{self, StreamExt};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Tile load request.
#[derive(Debug, Clone)]
pub struct TileRequest {
    /// Tile coordinate to load.
    pub coord: TileCoord,
    /// Priority (higher = more important).
    pub priority: i32,
}

impl TileRequest {
    /// Create a new tile request.
    pub fn new(coord: TileCoord, priority: i32) -> Self {
        Self { coord, priority }
    }
}

/// Tile load result.
#[derive(Debug)]
pub enum TileLoadResult {
    /// Tile loaded successfully.
    Success(TileData),
    /// Tile failed to load.
    Error { coord: TileCoord, error: String },
}

/// Configuration for tile loader.
#[derive(Debug, Clone)]
pub struct TileLoaderConfig {
    /// Maximum number of concurrent tile loads.
    pub max_concurrent_loads: usize,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Maximum retries per tile.
    pub max_retries: u32,
    /// User agent string for HTTP requests.
    pub user_agent: String,
}

impl Default for TileLoaderConfig {
    fn default() -> Self {
        Self {
            max_concurrent_loads: 6,
            timeout_secs: 30,
            max_retries: 3,
            user_agent: "MeridianMapEngine/0.2.5".to_string(),
        }
    }
}

/// Asynchronous tile loader.
pub struct TileLoader {
    /// Loader configuration.
    config: TileLoaderConfig,
    /// Tile URL template.
    url_template: String,
    /// Request queue.
    request_queue: Arc<RwLock<Vec<TileRequest>>>,
    /// Result sender.
    result_sender: Option<mpsc::UnboundedSender<TileLoadResult>>,
    /// Active downloads.
    active_downloads: Arc<RwLock<HashMap<TileCoord, u32>>>,
}

impl TileLoader {
    /// Create a new tile loader.
    pub fn new(url_template: impl Into<String>, config: TileLoaderConfig) -> Self {
        Self {
            config,
            url_template: url_template.into(),
            request_queue: Arc::new(RwLock::new(Vec::new())),
            result_sender: None,
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set the tile URL template.
    pub fn set_url_template(&mut self, template: impl Into<String>) {
        self.url_template = template.into();
    }

    /// Request a tile to be loaded.
    pub fn request_tile(&mut self, coord: TileCoord, priority: i32) {
        let mut queue = self.request_queue.write();

        // Check if tile is already requested or downloading
        if queue.iter().any(|r| r.coord == coord) {
            return;
        }

        if self.active_downloads.read().contains_key(&coord) {
            return;
        }

        queue.push(TileRequest::new(coord, priority));

        // Sort by priority (higher priority first)
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Start the tile loader (spawn background tasks).
    pub fn start(&mut self) -> mpsc::UnboundedReceiver<TileLoadResult> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.result_sender = Some(tx.clone());

        let request_queue = self.request_queue.clone();
        let active_downloads = self.active_downloads.clone();
        let url_template = self.url_template.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                // Get next batch of requests
                let requests: Vec<TileRequest> = {
                    let mut queue = request_queue.write();
                    let active_count = active_downloads.read().len();
                    let available_slots = config.max_concurrent_loads.saturating_sub(active_count);

                    queue.drain(..available_slots.min(queue.len())).collect()
                };

                if requests.is_empty() {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }

                // Mark tiles as downloading
                {
                    let mut active = active_downloads.write();
                    for req in &requests {
                        active.insert(req.coord, 0);
                    }
                }

                // Process requests concurrently
                let results = stream::iter(requests)
                    .map(|req| {
                        let url_template = url_template.clone();
                        let tx = tx.clone();
                        let active_downloads = active_downloads.clone();
                        let config = config.clone();

                        async move {
                            let result = Self::load_tile_async(&url_template, req.coord, &config).await;

                            // Remove from active downloads
                            active_downloads.write().remove(&req.coord);

                            // Send result
                            let _ = tx.send(result);
                        }
                    })
                    .buffer_unordered(config.max_concurrent_loads)
                    .collect::<Vec<_>>();

                results.await;
            }
        });

        rx
    }

    /// Load a single tile asynchronously.
    async fn load_tile_async(
        url_template: &str,
        coord: TileCoord,
        config: &TileLoaderConfig,
    ) -> TileLoadResult {
        let url = url_template
            .replace("{z}", &coord.z.to_string())
            .replace("{x}", &coord.x.to_string())
            .replace("{y}", &coord.y.to_string());

        for attempt in 0..=config.max_retries {
            match Self::fetch_tile(&url, config).await {
                Ok(data) => {
                    return TileLoadResult::Success(TileData::new(coord, data));
                }
                Err(e) => {
                    if attempt == config.max_retries {
                        return TileLoadResult::Error {
                            coord,
                            error: e,
                        };
                    }
                    // Exponential backoff
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        100 * 2_u64.pow(attempt),
                    ))
                    .await;
                }
            }
        }

        TileLoadResult::Error {
            coord,
            error: "Max retries exceeded".to_string(),
        }
    }

    /// Fetch tile data from URL.
    async fn fetch_tile(url: &str, _config: &TileLoaderConfig) -> std::result::Result<Vec<u8>, String> {
        // Note: This is a placeholder implementation
        // In production, use a proper HTTP client like reqwest

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Native implementation would use reqwest or similar
            // For now, return empty data
            Ok(Vec::new())
        }

        #[cfg(target_arch = "wasm32")]
        {
            // WASM implementation would use web-sys fetch API
            Ok(Vec::new())
        }
    }

    /// Get the number of active downloads.
    pub fn active_download_count(&self) -> usize {
        self.active_downloads.read().len()
    }

    /// Get the number of queued requests.
    pub fn queued_request_count(&self) -> usize {
        self.request_queue.read().len()
    }

    /// Clear all pending requests.
    pub fn clear_queue(&mut self) {
        self.request_queue.write().clear();
    }

    /// Cancel loading of a specific tile.
    pub fn cancel_tile(&mut self, coord: TileCoord) {
        let mut queue = self.request_queue.write();
        queue.retain(|r| r.coord != coord);
    }
}

/// Statistics about tile loading.
#[derive(Debug, Clone, Default)]
pub struct TileLoaderStats {
    /// Total tiles loaded.
    pub total_loaded: u64,
    /// Total tiles failed.
    pub total_failed: u64,
    /// Total bytes loaded.
    pub total_bytes: u64,
    /// Average load time in milliseconds.
    pub avg_load_time_ms: f64,
}

impl TileLoaderStats {
    /// Get success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        let total = self.total_loaded + self.total_failed;
        if total == 0 {
            0.0
        } else {
            (self.total_loaded as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_request() {
        let req = TileRequest::new(TileCoord::new(1, 2, 3), 100);
        assert_eq!(req.coord.x, 1);
        assert_eq!(req.priority, 100);
    }

    #[test]
    fn test_loader_config() {
        let config = TileLoaderConfig::default();
        assert_eq!(config.max_concurrent_loads, 6);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_loader_stats() {
        let mut stats = TileLoaderStats::default();
        stats.total_loaded = 90;
        stats.total_failed = 10;
        assert_eq!(stats.success_rate(), 90.0);
    }
}
