//! WebSocket client implementation with reconnection logic.

use crate::error::{Result, StreamError};
use crate::messages::StreamMessage;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, sleep};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, error, info, warn};

/// WebSocket client configuration.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Server URL
    pub url: String,
    /// Reconnection enabled
    pub auto_reconnect: bool,
    /// Maximum reconnection attempts (None for infinite)
    pub max_reconnect_attempts: Option<usize>,
    /// Initial reconnection delay
    pub reconnect_delay: Duration,
    /// Maximum reconnection delay
    pub max_reconnect_delay: Duration,
    /// Ping interval for keepalive
    pub ping_interval: Duration,
    /// Connection timeout
    pub connect_timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            auto_reconnect: true,
            max_reconnect_attempts: None,
            reconnect_delay: Duration::from_secs(1),
            max_reconnect_delay: Duration::from_secs(30),
            ping_interval: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
        }
    }
}

impl ClientConfig {
    /// Create a new client configuration.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }
}

/// WebSocket client state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    /// Disconnected
    Disconnected,
    /// Connecting
    Connecting,
    /// Connected
    Connected,
    /// Reconnecting
    Reconnecting,
    /// Closed (no reconnection)
    Closed,
}

/// WebSocket client for streaming.
pub struct StreamClient {
    /// Client configuration
    config: ClientConfig,
    /// Current state
    state: Arc<Mutex<ClientState>>,
    /// Message sender
    tx: mpsc::UnboundedSender<StreamMessage>,
    /// Message receiver
    rx: Arc<Mutex<mpsc::UnboundedReceiver<StreamMessage>>>,
}

impl StreamClient {
    /// Create a new streaming client.
    pub fn new(config: ClientConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            config,
            state: Arc::new(Mutex::new(ClientState::Disconnected)),
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    /// Connect to the server.
    pub async fn connect(&self) -> Result<()> {
        *self.state.lock().await = ClientState::Connecting;
        info!("Connecting to {}", self.config.url);

        let ws_stream = tokio::time::timeout(
            self.config.connect_timeout,
            connect_async(&self.config.url),
        )
        .await
        .map_err(|_| StreamError::Timeout)?
        .map_err(StreamError::WebSocket)?
        .0;

        *self.state.lock().await = ClientState::Connected;
        info!("Connected to {}", self.config.url);

        // Start message handling task
        self.start_message_handler(ws_stream).await;

        Ok(())
    }

    /// Send a message to the server.
    pub async fn send(&self, message: StreamMessage) -> Result<()> {
        let state = *self.state.lock().await;
        if state != ClientState::Connected {
            return Err(StreamError::ConnectionClosed);
        }

        self.tx
            .send(message)
            .map_err(|_| StreamError::ConnectionClosed)
    }

    /// Receive the next message from the server.
    pub async fn recv(&self) -> Result<StreamMessage> {
        let mut rx = self.rx.lock().await;
        rx.recv()
            .await
            .ok_or(StreamError::ConnectionClosed)
    }

    /// Get the current client state.
    pub async fn state(&self) -> ClientState {
        *self.state.lock().await
    }

    /// Close the connection.
    pub async fn close(&self) -> Result<()> {
        *self.state.lock().await = ClientState::Closed;
        info!("Connection closed");
        Ok(())
    }

    /// Start the message handler task.
    async fn start_message_handler(
        &self,
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    ) {
        let (mut ws_tx, mut ws_rx) = ws_stream.split();
        let rx = self.rx.clone();
        let tx = self.tx.clone();
        let state = self.state.clone();
        let _config = self.config.clone();

        // Spawn send task
        tokio::spawn(async move {
            let mut rx = rx.lock().await;
            while let Some(msg) = rx.recv().await {
                if let Ok(json) = msg.to_json() {
                    if ws_tx.send(Message::Text(json)).await.is_err() {
                        error!("Failed to send message");
                        break;
                    }
                }
            }
        });

        // Spawn receive task
        tokio::spawn(async move {
            while let Some(result) = ws_rx.next().await {
                match result {
                    Ok(Message::Text(text)) => {
                        if let Ok(msg) = StreamMessage::from_json(&text) {
                            if tx.send(msg).is_err() {
                                error!("Failed to forward received message");
                                break;
                            }
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        if let Ok(text) = String::from_utf8(data) {
                            if let Ok(msg) = StreamMessage::from_json(&text) {
                                let _ = tx.send(msg);
                            }
                        }
                    }
                    Ok(Message::Ping(_data)) => {
                        debug!("Received ping");
                        // Pong is handled automatically by tungstenite
                    }
                    Ok(Message::Pong(_)) => {
                        debug!("Received pong");
                    }
                    Ok(Message::Close(_)) => {
                        info!("Server closed connection");
                        *state.lock().await = ClientState::Disconnected;
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        *state.lock().await = ClientState::Disconnected;
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Spawn ping task
        let state = self.state.clone();
        let tx = self.tx.clone();
        let ping_interval = self.config.ping_interval;
        tokio::spawn(async move {
            let mut interval = interval(ping_interval);
            loop {
                interval.tick().await;
                let current_state = *state.lock().await;
                if current_state != ClientState::Connected {
                    break;
                }
                if tx.send(StreamMessage::ping()).is_err() {
                    break;
                }
            }
        });
    }

    /// Connect with automatic reconnection.
    pub async fn connect_with_reconnect(&self) -> Result<()> {
        let mut attempts = 0;
        let mut delay = self.config.reconnect_delay;

        loop {
            match self.connect().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if !self.config.auto_reconnect {
                        return Err(e);
                    }

                    if let Some(max_attempts) = self.config.max_reconnect_attempts {
                        if attempts >= max_attempts {
                            error!("Max reconnection attempts reached");
                            return Err(StreamError::generic("Max reconnection attempts reached"));
                        }
                    }

                    attempts += 1;
                    warn!("Connection failed (attempt {}): {}. Retrying in {:?}", attempts, e, delay);

                    *self.state.lock().await = ClientState::Reconnecting;
                    sleep(delay).await;

                    // Exponential backoff
                    delay = std::cmp::min(delay * 2, self.config.max_reconnect_delay);
                }
            }
        }
    }
}

/// Builder for creating a streaming client.
pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    /// Create a new client builder.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            config: ClientConfig::new(url),
        }
    }

    /// Enable or disable auto-reconnect.
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.config.auto_reconnect = enabled;
        self
    }

    /// Set maximum reconnection attempts.
    pub fn max_reconnect_attempts(mut self, attempts: usize) -> Self {
        self.config.max_reconnect_attempts = Some(attempts);
        self
    }

    /// Set reconnection delay.
    pub fn reconnect_delay(mut self, delay: Duration) -> Self {
        self.config.reconnect_delay = delay;
        self
    }

    /// Set ping interval.
    pub fn ping_interval(mut self, interval: Duration) -> Self {
        self.config.ping_interval = interval;
        self
    }

    /// Set connection timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Build the client.
    pub fn build(self) -> StreamClient {
        StreamClient::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config() {
        let config = ClientConfig::new("ws://localhost:8080");
        assert_eq!(config.url, "ws://localhost:8080");
        assert!(config.auto_reconnect);
    }

    #[test]
    fn test_client_builder() {
        let client = ClientBuilder::new("ws://localhost:8080")
            .auto_reconnect(false)
            .max_reconnect_attempts(5)
            .reconnect_delay(Duration::from_secs(2))
            .build();

        assert!(!client.config.auto_reconnect);
        assert_eq!(client.config.max_reconnect_attempts, Some(5));
    }

    #[tokio::test]
    async fn test_client_state() {
        let client = ClientBuilder::new("ws://localhost:8080").build();
        assert_eq!(client.state().await, ClientState::Disconnected);
    }
}
