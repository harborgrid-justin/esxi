//! WebSocket server implementation for real-time streaming.

use crate::channel::ChannelManager;
use crate::error::{Result, StreamError};
use crate::messages::{ClientId, StreamMessage};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::{interval, Instant};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Maximum number of concurrent connections.
const MAX_CONNECTIONS: usize = 10000;

/// Ping interval for keepalive.
const PING_INTERVAL: Duration = Duration::from_secs(30);

/// Pong timeout - if no pong received, connection is considered dead.
const PONG_TIMEOUT: Duration = Duration::from_secs(60);

/// Server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address
    pub addr: SocketAddr,
    /// Maximum connections
    pub max_connections: usize,
    /// Ping interval
    pub ping_interval: Duration,
    /// Pong timeout
    pub pong_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:8080".parse().unwrap(),
            max_connections: MAX_CONNECTIONS,
            ping_interval: PING_INTERVAL,
            pong_timeout: PONG_TIMEOUT,
        }
    }
}

impl ServerConfig {
    /// Create a new server configuration.
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            ..Default::default()
        }
    }
}

/// Information about a connected client.
struct ClientInfo {
    /// Client identifier
    id: ClientId,
    /// Message sender
    tx: mpsc::UnboundedSender<Message>,
    /// Last pong received
    last_pong: Instant,
    /// Connection metadata
    metadata: serde_json::Value,
}

/// WebSocket streaming server.
pub struct StreamServer {
    /// Server configuration
    config: ServerConfig,
    /// Channel manager
    channels: Arc<ChannelManager>,
    /// Connected clients
    clients: Arc<DashMap<ClientId, ClientInfo>>,
}

impl StreamServer {
    /// Create a new streaming server.
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            channels: Arc::new(ChannelManager::new()),
            clients: Arc::new(DashMap::new()),
        }
    }

    /// Create a new streaming server with custom channel manager.
    pub fn with_channel_manager(config: ServerConfig, channels: Arc<ChannelManager>) -> Self {
        Self {
            config,
            channels,
            clients: Arc::new(DashMap::new()),
        }
    }

    /// Get the channel manager.
    pub fn channels(&self) -> Arc<ChannelManager> {
        self.channels.clone()
    }

    /// Get the number of connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Run the server.
    pub async fn run(self: Arc<Self>) -> Result<()> {
        let listener = TcpListener::bind(&self.config.addr).await?;
        info!("WebSocket server listening on {}", self.config.addr);

        // Start cleanup task
        let server = self.clone();
        tokio::spawn(async move {
            server.cleanup_task().await;
        });

        // Accept connections
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    if self.clients.len() >= self.config.max_connections {
                        warn!("Max connections reached, rejecting connection from {}", addr);
                        continue;
                    }

                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream, addr).await {
                            error!("Connection error from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a WebSocket connection.
    async fn handle_connection(&self, stream: TcpStream, addr: SocketAddr) -> Result<()> {
        info!("New connection from {}", addr);

        let ws_stream = accept_async(stream).await?;
        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        let client_id = Uuid::new_v4();
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Register client
        let client_info = ClientInfo {
            id: client_id,
            tx: tx.clone(),
            last_pong: Instant::now(),
            metadata: serde_json::Value::Null,
        };
        self.clients.insert(client_id, client_info);

        info!("Client {} connected from {}", client_id, addr);

        // Spawn send task
        let send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if ws_tx.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Spawn ping task
        let clients = self.clients.clone();
        let client_tx = tx.clone();
        let ping_interval = self.config.ping_interval;
        let pong_timeout = self.config.pong_timeout;
        let ping_task = tokio::spawn(async move {
            let mut interval = interval(ping_interval);
            loop {
                interval.tick().await;

                // Check if client is still alive
                if let Some(client) = clients.get(&client_id) {
                    if client.last_pong.elapsed() > pong_timeout {
                        warn!("Client {} ping timeout", client_id);
                        break;
                    }
                } else {
                    break;
                }

                // Send ping
                if client_tx.send(Message::Ping(vec![])).is_err() {
                    break;
                }
            }
        });

        // Handle incoming messages
        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.handle_message(client_id, &text).await {
                        error!("Error handling message from {}: {}", client_id, e);
                    }
                }
                Ok(Message::Binary(data)) => {
                    if let Ok(text) = String::from_utf8(data) {
                        let _ = self.handle_message(client_id, &text).await;
                    }
                }
                Ok(Message::Ping(data)) => {
                    if tx.send(Message::Pong(data)).is_err() {
                        break;
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Update last pong time
                    if let Some(mut client) = self.clients.get_mut(&client_id) {
                        client.last_pong = Instant::now();
                    }
                    debug!("Received pong from {}", client_id);
                }
                Ok(Message::Close(_)) => {
                    info!("Client {} closed connection", client_id);
                    break;
                }
                Err(e) => {
                    error!("WebSocket error from {}: {}", client_id, e);
                    break;
                }
                _ => {}
            }
        }

        // Cleanup
        send_task.abort();
        ping_task.abort();
        self.cleanup_client(client_id).await;

        Ok(())
    }

    /// Handle a message from a client.
    async fn handle_message(&self, client_id: ClientId, text: &str) -> Result<()> {
        let message = StreamMessage::from_json(text)?;
        debug!("Received message from {}: {:?}", client_id, message);

        match message {
            StreamMessage::Ping { timestamp } => {
                self.send_to_client(client_id, StreamMessage::Pong { timestamp })
                    .await?;
            }
            StreamMessage::Subscribe(sub) => {
                // Subscribe client to channel
                let _ = self.channels.subscribe(client_id, sub.channel);
            }
            StreamMessage::Unsubscribe(unsub) => {
                // Unsubscribe client from channel
                let _ = self.channels.unsubscribe(client_id, &unsub.channel);
            }
            _msg => {
                // For other messages, you would typically route them through handlers
                // For now, we'll just log them
                debug!("Unhandled message type from {}", client_id);
            }
        }

        Ok(())
    }

    /// Send a message to a specific client.
    pub async fn send_to_client(&self, client_id: ClientId, message: StreamMessage) -> Result<()> {
        let client = self
            .clients
            .get(&client_id)
            .ok_or_else(|| StreamError::ClientNotFound(client_id.to_string()))?;

        let json = message.to_json()?;
        client
            .tx
            .send(Message::Text(json))
            .map_err(|_| StreamError::ConnectionClosed)?;

        Ok(())
    }

    /// Broadcast a message to all connected clients.
    pub async fn broadcast(&self, message: StreamMessage) -> Result<usize> {
        let json = message.to_json()?;
        let mut count = 0;

        for client in self.clients.iter() {
            if client.tx.send(Message::Text(json.clone())).is_ok() {
                count += 1;
            }
        }

        Ok(count)
    }

    /// Cleanup a disconnected client.
    async fn cleanup_client(&self, client_id: ClientId) {
        // Unsubscribe from all channels
        let _ = self.channels.unsubscribe_all(client_id);

        // Remove from clients
        self.clients.remove(&client_id);

        info!("Client {} cleaned up", client_id);
    }

    /// Periodic cleanup task.
    async fn cleanup_task(&self) {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;

            // Clean up empty channels
            let removed = self.channels.cleanup_empty_channels();
            if removed > 0 {
                info!("Cleaned up {} empty channels", removed);
            }

            // Log stats
            info!(
                "Server stats - Clients: {}, Channels: {}",
                self.client_count(),
                self.channels.channel_count()
            );
        }
    }

    /// Get client metadata.
    pub fn get_client_metadata(&self, client_id: &ClientId) -> Option<serde_json::Value> {
        self.clients
            .get(client_id)
            .map(|c| c.metadata.clone())
    }

    /// Update client metadata.
    pub fn update_client_metadata(
        &self,
        client_id: &ClientId,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let mut client = self
            .clients
            .get_mut(client_id)
            .ok_or_else(|| StreamError::ClientNotFound(client_id.to_string()))?;
        client.metadata = metadata;
        Ok(())
    }
}

/// Builder for creating a streaming server.
pub struct ServerBuilder {
    config: ServerConfig,
    channel_manager: Option<Arc<ChannelManager>>,
}

impl ServerBuilder {
    /// Create a new server builder.
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            config: ServerConfig::new(addr),
            channel_manager: None,
        }
    }

    /// Set maximum connections.
    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = max;
        self
    }

    /// Set ping interval.
    pub fn ping_interval(mut self, interval: Duration) -> Self {
        self.config.ping_interval = interval;
        self
    }

    /// Set pong timeout.
    pub fn pong_timeout(mut self, timeout: Duration) -> Self {
        self.config.pong_timeout = timeout;
        self
    }

    /// Set a custom channel manager.
    pub fn channel_manager(mut self, manager: Arc<ChannelManager>) -> Self {
        self.channel_manager = Some(manager);
        self
    }

    /// Build the server.
    pub fn build(self) -> StreamServer {
        if let Some(manager) = self.channel_manager {
            StreamServer::with_channel_manager(self.config, manager)
        } else {
            StreamServer::new(self.config)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let config = ServerConfig::new(addr);
        assert_eq!(config.addr, addr);
        assert_eq!(config.max_connections, MAX_CONNECTIONS);
    }

    #[test]
    fn test_server_builder() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let server = ServerBuilder::new(addr)
            .max_connections(5000)
            .ping_interval(Duration::from_secs(15))
            .build();

        assert_eq!(server.config.max_connections, 5000);
        assert_eq!(server.config.ping_interval, Duration::from_secs(15));
    }

    #[test]
    fn test_server_client_count() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let server = StreamServer::new(ServerConfig::new(addr));
        assert_eq!(server.client_count(), 0);
    }
}
