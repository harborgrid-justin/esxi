//! WebSocket connection management

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ws::{Message as WsMessage, WebSocket};
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{Error, Result};
use crate::protocol::{Message, MessageType};
use crate::server::ServerState;

/// Connection ID type
pub type ConnectionId = Uuid;

/// Connection state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connecting
    Connecting,
    /// Connected and authenticated
    Connected,
    /// Disconnecting
    Disconnecting,
    /// Disconnected
    Disconnected,
}

/// User information
#[derive(Debug, Clone)]
pub struct UserInfo {
    /// User ID
    pub user_id: String,
    /// User name
    pub name: String,
    /// User metadata
    pub metadata: serde_json::Value,
}

/// WebSocket connection
pub struct Connection {
    /// Connection ID
    id: ConnectionId,

    /// WebSocket
    socket: Arc<RwLock<WebSocket>>,

    /// Server state
    state: ServerState,

    /// Connection state
    conn_state: Arc<RwLock<ConnectionState>>,

    /// User information (after authentication)
    user_info: Arc<RwLock<Option<UserInfo>>>,

    /// Last activity timestamp
    last_activity: Arc<RwLock<Instant>>,

    /// Message sender
    tx: mpsc::UnboundedSender<Message>,

    /// Message receiver
    rx: Arc<RwLock<mpsc::UnboundedReceiver<Message>>>,
}

impl Connection {
    /// Create new connection
    pub async fn new(socket: WebSocket, state: ServerState) -> Result<Self> {
        let id = Uuid::new_v4();
        let (tx, rx) = mpsc::unbounded_channel();

        info!("New connection: {}", id);

        Ok(Self {
            id,
            socket: Arc::new(RwLock::new(socket)),
            state,
            conn_state: Arc::new(RwLock::new(ConnectionState::Connecting)),
            user_info: Arc::new(RwLock::new(None)),
            last_activity: Arc::new(RwLock::new(Instant::now())),
            tx,
            rx: Arc::new(RwLock::new(rx)),
        })
    }

    /// Get connection ID
    pub fn id(&self) -> ConnectionId {
        self.id
    }

    /// Get connection state
    pub async fn state(&self) -> ConnectionState {
        self.conn_state.read().await.clone()
    }

    /// Get user info
    pub async fn user_info(&self) -> Option<UserInfo> {
        self.user_info.read().await.clone()
    }

    /// Set user info (after authentication)
    pub async fn set_user_info(&self, info: UserInfo) {
        *self.user_info.write().await = Some(info);
        *self.conn_state.write().await = ConnectionState::Connected;
    }

    /// Send message
    pub async fn send(&self, message: Message) -> Result<()> {
        self.tx
            .send(message)
            .map_err(|_| Error::ChannelClosed)?;
        Ok(())
    }

    /// Run connection event loop
    pub async fn run(self) -> Result<()> {
        let mut shutdown_rx = self.state.subscribe_shutdown();
        let mut heartbeat = interval(self.state.config().heartbeat_interval);

        // Split socket
        let socket = self.socket.clone();
        let (mut ws_tx, mut ws_rx) = {
            let mut socket = socket.write().await;
            let (tx, rx) = socket.split();
            (tx, rx)
        };

        let mut rx = self.rx.write().await;

        loop {
            tokio::select! {
                // Receive from WebSocket
                ws_msg = ws_rx.next() => {
                    match ws_msg {
                        Some(Ok(msg)) => {
                            if let Err(e) = self.handle_ws_message(msg).await {
                                error!("Error handling WebSocket message: {}", e);
                                break;
                            }
                            *self.last_activity.write().await = Instant::now();
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            debug!("WebSocket closed");
                            break;
                        }
                    }
                }

                // Send to WebSocket
                Some(msg) = rx.recv() => {
                    if let Err(e) = self.send_ws_message(&mut ws_tx, msg).await {
                        error!("Error sending WebSocket message: {}", e);
                        break;
                    }
                }

                // Heartbeat
                _ = heartbeat.tick() => {
                    if let Err(e) = self.check_timeout().await {
                        warn!("Connection timeout: {}", e);
                        break;
                    }

                    // Send ping
                    if let Err(e) = ws_tx.send(WsMessage::Ping(vec![])).await {
                        error!("Error sending ping: {}", e);
                        break;
                    }
                }

                // Shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received");
                    break;
                }
            }
        }

        // Cleanup
        *self.conn_state.write().await = ConnectionState::Disconnected;
        info!("Connection closed: {}", self.id);

        Ok(())
    }

    /// Handle WebSocket message
    async fn handle_ws_message(&self, ws_msg: WsMessage) -> Result<()> {
        match ws_msg {
            WsMessage::Text(text) => {
                let message: Message = serde_json::from_str(&text)?;
                self.handle_message(message).await
            }
            WsMessage::Binary(data) => {
                let message: Message = rmp_serde::from_slice(&data)?;
                self.handle_message(message).await
            }
            WsMessage::Ping(data) => {
                // Echo pong
                Ok(())
            }
            WsMessage::Pong(_) => {
                // Update activity
                Ok(())
            }
            WsMessage::Close(_) => {
                *self.conn_state.write().await = ConnectionState::Disconnecting;
                Ok(())
            }
        }
    }

    /// Handle protocol message
    async fn handle_message(&self, message: Message) -> Result<()> {
        debug!("Received message: {:?}", message.msg_type);

        match message.msg_type {
            MessageType::Auth => self.handle_auth(message).await,
            MessageType::JoinRoom => self.handle_join_room(message).await,
            MessageType::LeaveRoom => self.handle_leave_room(message).await,
            MessageType::Data => self.handle_data(message).await,
            MessageType::Sync => self.handle_sync(message).await,
            MessageType::Presence => self.handle_presence(message).await,
            _ => Ok(()),
        }
    }

    /// Handle authentication
    async fn handle_auth(&self, message: Message) -> Result<()> {
        // Extract auth info from payload
        let auth_data: serde_json::Value = serde_json::from_slice(&message.payload)?;

        let user_id = auth_data["user_id"]
            .as_str()
            .ok_or_else(|| Error::InvalidMessage("Missing user_id".to_string()))?;

        let name = auth_data["name"]
            .as_str()
            .unwrap_or("Anonymous")
            .to_string();

        let user_info = UserInfo {
            user_id: user_id.to_string(),
            name,
            metadata: auth_data["metadata"].clone(),
        };

        self.set_user_info(user_info).await;

        // Send auth success
        let response = Message::new(MessageType::AuthSuccess, vec![]);
        self.send(response).await?;

        info!("User authenticated: {}", user_id);
        Ok(())
    }

    /// Handle join room
    async fn handle_join_room(&self, message: Message) -> Result<()> {
        let room_id = String::from_utf8_lossy(&message.payload).to_string();

        // Get or create room
        let room = self.state.room_manager().get_or_create(&room_id)?;

        // Add connection to room
        room.add_connection(self.id, self.tx.clone()).await?;

        info!("Connection {} joined room {}", self.id, room_id);
        Ok(())
    }

    /// Handle leave room
    async fn handle_leave_room(&self, message: Message) -> Result<()> {
        let room_id = String::from_utf8_lossy(&message.payload).to_string();

        if let Some(room) = self.state.room_manager().get(&room_id) {
            room.remove_connection(self.id).await;
        }

        info!("Connection {} left room {}", self.id, room_id);
        Ok(())
    }

    /// Handle data message
    async fn handle_data(&self, message: Message) -> Result<()> {
        // Forward to room for broadcasting
        // This would be implemented based on the message routing
        Ok(())
    }

    /// Handle sync message
    async fn handle_sync(&self, message: Message) -> Result<()> {
        // Handle CRDT synchronization
        Ok(())
    }

    /// Handle presence message
    async fn handle_presence(&self, message: Message) -> Result<()> {
        // Update presence information
        Ok(())
    }

    /// Send WebSocket message
    async fn send_ws_message(
        &self,
        ws_tx: &mut futures::stream::SplitSink<WebSocket, WsMessage>,
        message: Message,
    ) -> Result<()> {
        // Serialize to binary (MessagePack) for efficiency
        let data = rmp_serde::to_vec(&message)?;
        ws_tx.send(WsMessage::Binary(data)).await?;
        Ok(())
    }

    /// Check for timeout
    async fn check_timeout(&self) -> Result<()> {
        let last_activity = *self.last_activity.read().await;
        let timeout = self.state.config().client_timeout;

        if last_activity.elapsed() > timeout {
            return Err(Error::Timeout);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_id() {
        let id = Uuid::new_v4();
        assert!(!id.to_string().is_empty());
    }

    #[test]
    fn test_connection_state() {
        assert_eq!(ConnectionState::Connecting, ConnectionState::Connecting);
        assert_ne!(ConnectionState::Connecting, ConnectionState::Connected);
    }
}
