use crate::events::{EventBus, MonitorEvent};
use crate::monitor::MonitorEngine;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

/// WebSocket server state
#[derive(Clone)]
pub struct WebSocketServer {
    engine: Arc<MonitorEngine>,
    event_bus: EventBus,
    clients: Arc<DashMap<String, ClientInfo>>,
}

/// Information about connected client
#[derive(Debug, Clone)]
struct ClientInfo {
    id: String,
    connected_at: chrono::DateTime<chrono::Utc>,
    subscriptions: Vec<String>,
}

/// Client message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientMessage {
    /// Subscribe to specific event types
    Subscribe { event_types: Vec<String> },
    /// Unsubscribe from event types
    Unsubscribe { event_types: Vec<String> },
    /// Request current state
    GetState,
    /// Ping for keepalive
    Ping,
}

/// Server message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    /// Event notification
    Event { event: MonitorEvent },
    /// Current state response
    State { data: StateSnapshot },
    /// Pong response
    Pong,
    /// Error message
    Error { message: String },
}

/// Snapshot of current system state
#[derive(Debug, Serialize, Deserialize)]
struct StateSnapshot {
    active_scans: Vec<(Uuid, crate::monitor::ScanContext)>,
    metrics: crate::types::MonitorMetrics,
    health: crate::types::HealthStatus,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl WebSocketServer {
    /// Create a new WebSocket server
    pub fn new(engine: Arc<MonitorEngine>, event_bus: EventBus) -> Self {
        Self {
            engine,
            event_bus,
            clients: Arc::new(DashMap::new()),
        }
    }

    /// Get the router for WebSocket endpoints
    pub fn router(self) -> Router {
        Router::new()
            .route("/ws", get(ws_handler))
            .route("/ws/health", get(health_handler))
            .with_state(Arc::new(self))
    }

    /// Get connected client count
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Handle new WebSocket connection
    async fn handle_connection(self: Arc<Self>, socket: WebSocket) {
        let client_id = Uuid::new_v4().to_string();
        let (mut sender, mut receiver) = socket.split();

        // Register client
        let client_info = ClientInfo {
            id: client_id.clone(),
            connected_at: chrono::Utc::now(),
            subscriptions: vec![],
        };
        self.clients.insert(client_id.clone(), client_info);

        // Publish client connected event
        let _ = self.event_bus.publish(MonitorEvent::ClientConnected {
            client_id: client_id.clone(),
            timestamp: chrono::Utc::now(),
        });

        tracing::info!(client_id = %client_id, "WebSocket client connected");

        // Subscribe to event bus
        let mut event_rx = self.event_bus.subscribe();

        // Create channels for coordinating tasks
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();

        // Spawn task to forward events to client
        let client_id_clone = client_id.clone();
        let clients = self.clients.clone();
        let event_task = tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                // Check if client is still subscribed to this event type
                if let Some(client) = clients.get(&client_id_clone) {
                    if client.subscriptions.is_empty()
                        || client.subscriptions.contains(&event.event_type().to_string())
                    {
                        let msg = ServerMessage::Event { event };
                        if tx.send(msg).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        // Spawn task to send messages to client
        let send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Handle incoming client messages
        let engine = self.engine.clone();
        let clients = self.clients.clone();
        let client_id_clone = client_id.clone();
        let tx_clone = tx.clone();

        while let Some(Ok(msg)) = receiver.next().await {
            if let Err(e) = Self::handle_client_message(
                msg,
                &client_id_clone,
                &engine,
                &clients,
                &tx_clone,
            )
            .await
            {
                tracing::error!(error = ?e, "Error handling client message");
                let error_msg = ServerMessage::Error {
                    message: e.to_string(),
                };
                let _ = tx_clone.send(error_msg);
            }
        }

        // Cleanup on disconnect
        event_task.abort();
        send_task.abort();
        self.clients.remove(&client_id);

        let _ = self.event_bus.publish(MonitorEvent::ClientDisconnected {
            client_id: client_id.clone(),
            timestamp: chrono::Utc::now(),
        });

        tracing::info!(client_id = %client_id, "WebSocket client disconnected");
    }

    async fn handle_client_message(
        msg: Message,
        client_id: &str,
        engine: &MonitorEngine,
        clients: &DashMap<String, ClientInfo>,
        tx: &tokio::sync::mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<(), WebSocketError> {
        match msg {
            Message::Text(text) => {
                let client_msg: ClientMessage = serde_json::from_str(&text)
                    .map_err(|e| WebSocketError::InvalidMessage(e.to_string()))?;

                match client_msg {
                    ClientMessage::Subscribe { event_types } => {
                        if let Some(mut client) = clients.get_mut(client_id) {
                            client.subscriptions.extend(event_types);
                        }
                    }
                    ClientMessage::Unsubscribe { event_types } => {
                        if let Some(mut client) = clients.get_mut(client_id) {
                            client
                                .subscriptions
                                .retain(|s| !event_types.contains(s));
                        }
                    }
                    ClientMessage::GetState => {
                        let snapshot = StateSnapshot {
                            active_scans: engine.get_active_scans(),
                            metrics: engine.get_metrics().await,
                            health: engine.get_health().await,
                            timestamp: chrono::Utc::now(),
                        };
                        let _ = tx.send(ServerMessage::State { data: snapshot });
                    }
                    ClientMessage::Ping => {
                        let _ = tx.send(ServerMessage::Pong);
                    }
                }
            }
            Message::Close(_) => {
                return Err(WebSocketError::ConnectionClosed);
            }
            _ => {}
        }

        Ok(())
    }
}

/// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(server): State<Arc<WebSocketServer>>,
) -> Response {
    ws.on_upgrade(move |socket| async move {
        server.handle_connection(socket).await;
    })
}

/// Health check handler
async fn health_handler(State(server): State<Arc<WebSocketServer>>) -> String {
    format!(
        "{{\"status\":\"healthy\",\"clients\":{}}}",
        server.client_count()
    )
}

/// WebSocket errors
#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Send failed: {0}")]
    SendFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_server() {
        let event_bus = EventBus::new(100);
        let engine = Arc::new(MonitorEngine::new(event_bus.clone()));
        let server = WebSocketServer::new(engine, event_bus);

        assert_eq!(server.client_count(), 0);
    }
}
