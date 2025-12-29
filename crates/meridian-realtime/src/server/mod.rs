//! WebSocket server implementation

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use dashmap::DashMap;
use tokio::sync::broadcast;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use crate::error::{Error, Result};
use crate::pubsub::Channel;

pub mod connection;
pub mod room;
pub mod broadcast;

pub use connection::{Connection, ConnectionId, ConnectionState};
pub use room::{Room, RoomId, RoomManager};
pub use broadcast::Broadcaster;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// WebSocket listening address
    pub addr: SocketAddr,

    /// Redis connection URL
    pub redis_url: String,

    /// Maximum connections per room
    pub max_connections_per_room: usize,

    /// Maximum message size
    pub max_message_size: usize,

    /// Heartbeat interval
    pub heartbeat_interval: Duration,

    /// Client timeout
    pub client_timeout: Duration,

    /// Enable compression
    pub enable_compression: bool,

    /// Enable end-to-end encryption
    pub enable_e2ee: bool,

    /// Maximum rooms
    pub max_rooms: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: ([0, 0, 0, 0], crate::DEFAULT_WS_PORT).into(),
            redis_url: crate::DEFAULT_REDIS_URL.to_string(),
            max_connections_per_room: 100,
            max_message_size: crate::MAX_MESSAGE_SIZE,
            heartbeat_interval: Duration::from_secs(crate::HEARTBEAT_INTERVAL_SECS),
            client_timeout: Duration::from_secs(crate::CLIENT_TIMEOUT_SECS),
            enable_compression: true,
            enable_e2ee: false,
            max_rooms: 10000,
        }
    }
}

/// Shared server state
#[derive(Clone)]
pub struct ServerState {
    config: Arc<ServerConfig>,
    room_manager: Arc<RoomManager>,
    redis_pool: deadpool_redis::Pool,
    broadcaster: Arc<Broadcaster>,
    shutdown_tx: broadcast::Sender<()>,
}

impl ServerState {
    /// Create new server state
    pub async fn new(config: ServerConfig) -> Result<Self> {
        // Create Redis pool
        let redis_config = deadpool_redis::Config::from_url(&config.redis_url);
        let redis_pool = redis_config
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .map_err(|e| Error::Redis(redis::RedisError::from((
                redis::ErrorKind::InvalidClientConfig,
                "Failed to create Redis pool",
                e.to_string(),
            ))))?;

        let (shutdown_tx, _) = broadcast::channel(1);
        let room_manager = Arc::new(RoomManager::new(config.max_rooms));
        let broadcaster = Arc::new(Broadcaster::new(redis_pool.clone()));

        Ok(Self {
            config: Arc::new(config),
            room_manager,
            redis_pool,
            broadcaster,
            shutdown_tx,
        })
    }

    /// Get configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get room manager
    pub fn room_manager(&self) -> &RoomManager {
        &self.room_manager
    }

    /// Get Redis pool
    pub fn redis_pool(&self) -> &deadpool_redis::Pool {
        &self.redis_pool
    }

    /// Get broadcaster
    pub fn broadcaster(&self) -> &Broadcaster {
        &self.broadcaster
    }

    /// Subscribe to shutdown signal
    pub fn subscribe_shutdown(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Trigger shutdown
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

/// Main WebSocket server
pub struct Server {
    state: ServerState,
}

impl Server {
    /// Create new server
    pub async fn new(config: ServerConfig) -> Result<Self> {
        let state = ServerState::new(config).await?;
        Ok(Self { state })
    }

    /// Run the server
    pub async fn run(self) -> Result<()> {
        let addr = self.state.config.addr;

        // Build router
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .route("/health", get(health_handler))
            .route("/metrics", get(metrics_handler))
            .layer(TraceLayer::new_for_http())
            .with_state(self.state.clone());

        info!("Starting WebSocket server on {}", addr);

        // Start server
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;

        Ok(())
    }

    /// Get server state
    pub fn state(&self) -> &ServerState {
        &self.state
    }
}

/// WebSocket handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ServerState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: ServerState) {
    let connection = match Connection::new(socket, state.clone()).await {
        Ok(conn) => conn,
        Err(e) => {
            warn!("Failed to create connection: {}", e);
            return;
        }
    };

    if let Err(e) = connection.run().await {
        warn!("Connection error: {}", e);
    }
}

/// Health check handler
async fn health_handler(State(state): State<ServerState>) -> &'static str {
    // Check Redis connection
    match state.redis_pool.get().await {
        Ok(_) => "OK",
        Err(_) => "DEGRADED",
    }
}

/// Metrics handler
async fn metrics_handler(State(state): State<ServerState>) -> String {
    let room_count = state.room_manager.room_count();
    let total_connections = state.room_manager.total_connections();

    format!(
        "# HELP meridian_rooms_total Total number of active rooms\n\
         # TYPE meridian_rooms_total gauge\n\
         meridian_rooms_total {}\n\
         # HELP meridian_connections_total Total number of active connections\n\
         # TYPE meridian_connections_total gauge\n\
         meridian_connections_total {}\n",
        room_count, total_connections
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.max_connections_per_room, 100);
        assert!(config.enable_compression);
    }

    #[tokio::test]
    async fn test_server_state_creation() {
        // This would require a Redis instance, so we'll skip actual creation
        // Just test that the types are correct
        let config = ServerConfig::default();
        assert!(config.redis_url.starts_with("redis://"));
    }
}
