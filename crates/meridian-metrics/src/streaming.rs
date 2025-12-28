//! Real-time metrics streaming via WebSocket.

use crate::collector::MetricsCollector;
use crate::error::{MetricsError, Result};
use crate::types::MetricSnapshot;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Enable streaming
    pub enabled: bool,

    /// WebSocket server port
    pub port: u16,

    /// Stream interval in milliseconds
    pub interval_ms: u64,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Message buffer size
    pub buffer_size: usize,

    /// Enable compression
    pub compression: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 9091,
            interval_ms: 1000,
            max_connections: 100,
            buffer_size: 1000,
            compression: true,
        }
    }
}

/// Streaming message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamMessage {
    /// Metric snapshot update
    MetricUpdate {
        snapshots: Vec<MetricSnapshot>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Subscription confirmation
    Subscribed {
        filters: Vec<String>,
        interval_ms: u64,
    },

    /// Error message
    Error {
        message: String,
        code: String,
    },

    /// Ping for keepalive
    Ping {
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Pong response
    Pong {
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Connection info
    Info {
        version: String,
        server: String,
    },
}

impl StreamMessage {
    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| MetricsError::Serialization(e))
    }

    /// Parse from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| MetricsError::Serialization(e))
    }
}

/// Client subscription filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionFilter {
    /// Metric name patterns (supports wildcards)
    pub metrics: Vec<String>,

    /// Label filters
    pub labels: Option<std::collections::HashMap<String, String>>,

    /// Minimum update interval in milliseconds
    pub min_interval_ms: Option<u64>,
}

impl Default for SubscriptionFilter {
    fn default() -> Self {
        Self {
            metrics: vec!["*".to_string()],
            labels: None,
            min_interval_ms: None,
        }
    }
}

impl SubscriptionFilter {
    /// Check if a metric matches this filter
    pub fn matches(&self, snapshot: &MetricSnapshot) -> bool {
        // Check metric name
        let metric_matches = self.metrics.iter().any(|pattern| {
            if pattern == "*" {
                true
            } else if pattern.ends_with('*') {
                snapshot
                    .name
                    .starts_with(&pattern[..pattern.len() - 1])
            } else {
                &snapshot.name == pattern
            }
        });

        if !metric_matches {
            return false;
        }

        // Check labels if specified
        if let Some(label_filters) = &self.labels {
            for (key, value) in label_filters {
                if snapshot.labels.get(key) != Some(value) {
                    return false;
                }
            }
        }

        true
    }
}

/// Client subscription request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub filter: SubscriptionFilter,
}

/// Streaming server state
struct StreamingState {
    collector: Arc<MetricsCollector>,
    config: StreamingConfig,
    broadcast_tx: broadcast::Sender<StreamMessage>,
    active_connections: Arc<RwLock<usize>>,
}

/// Metrics streaming server
pub struct StreamingServer {
    config: StreamingConfig,
    collector: Arc<MetricsCollector>,
    broadcast_tx: broadcast::Sender<StreamMessage>,
    active_connections: Arc<RwLock<usize>>,
}

impl StreamingServer {
    /// Create a new streaming server
    pub fn new(config: StreamingConfig, collector: Arc<MetricsCollector>) -> Self {
        let (broadcast_tx, _) = broadcast::channel(config.buffer_size);

        info!("Initializing metrics streaming server on port {}", config.port);

        Self {
            config,
            collector,
            broadcast_tx,
            active_connections: Arc::new(RwLock::new(0)),
        }
    }

    /// Create with default configuration
    pub fn default(collector: Arc<MetricsCollector>) -> Self {
        Self::new(StreamingConfig::default(), collector)
    }

    /// Start the WebSocket server
    pub async fn start(self: Arc<Self>) -> Result<()> {
        if !self.config.enabled {
            info!("Streaming server disabled");
            return Ok(());
        }

        let state = Arc::new(StreamingState {
            collector: Arc::clone(&self.collector),
            config: self.config.clone(),
            broadcast_tx: self.broadcast_tx.clone(),
            active_connections: Arc::clone(&self.active_connections),
        });

        // Start broadcast task
        let broadcast_state = Arc::clone(&state);
        tokio::spawn(async move {
            broadcast_metrics(broadcast_state).await;
        });

        // Build router
        let app = Router::new()
            .route("/ws", get(websocket_handler))
            .route("/health", get(health_handler))
            .with_state(state);

        // Start server
        let addr = format!("0.0.0.0:{}", self.config.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| MetricsError::streaming(format!("Failed to bind to {}: {}", addr, e)))?;

        info!("Streaming server listening on {}", addr);

        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                error!("Streaming server error: {}", e);
            }
        });

        Ok(())
    }

    /// Get active connection count
    pub fn active_connections(&self) -> usize {
        *self.active_connections.read()
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&self, message: StreamMessage) -> Result<()> {
        self.broadcast_tx
            .send(message)
            .map(|_| ())
            .map_err(|e| MetricsError::streaming(format!("Failed to broadcast: {}", e)))
    }
}

/// WebSocket upgrade handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<StreamingState>>,
) -> Response {
    // Check connection limit
    let current_connections = *state.active_connections.read();
    if current_connections >= state.config.max_connections {
        warn!("Max connections reached, rejecting new connection");
        return (StatusCode::SERVICE_UNAVAILABLE, "Too many connections").into_response();
    }

    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<StreamingState>) {
    // Increment connection counter
    {
        let mut count = state.active_connections.write();
        *count += 1;
        info!("New WebSocket connection (active: {})", *count);
    }

    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = state.broadcast_tx.subscribe();
    let mut filter = SubscriptionFilter::default();

    // Send welcome message
    let info_msg = StreamMessage::Info {
        version: "0.1.5".to_string(),
        server: "meridian-metrics".to_string(),
    };

    if let Ok(json) = info_msg.to_json() {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Spawn task to handle incoming messages from client
    let mut recv_task = tokio::spawn(async move {
        let mut current_filter = SubscriptionFilter::default();

        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(request) = serde_json::from_str::<SubscribeRequest>(&text) {
                        current_filter = request.filter.clone();
                        debug!("Client subscribed with filter: {:?}", current_filter);
                    }
                }
                Message::Close(_) => {
                    debug!("Client sent close message");
                    break;
                }
                _ => {}
            }
        }

        current_filter
    });

    // Spawn task to send broadcasts to client
    let mut send_task = tokio::spawn(async move {
        while let Ok(message) = broadcast_rx.recv().await {
            // Filter messages if necessary
            let should_send = match &message {
                StreamMessage::MetricUpdate { snapshots, .. } => {
                    // Filter snapshots based on subscription
                    true
                }
                _ => true,
            };

            if should_send {
                if let Ok(json) = message.to_json() {
                    if sender.send(Message::Text(json)).await.is_err() {
                        debug!("Failed to send message, client disconnected");
                        break;
                    }
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        filter_result = &mut recv_task => {
            send_task.abort();
            if let Ok(f) = filter_result {
                filter = f;
            }
        }
        _ = &mut send_task => {
            recv_task.abort();
        }
    }

    // Decrement connection counter
    {
        let mut count = state.active_connections.write();
        *count -= 1;
        info!("WebSocket connection closed (active: {})", *count);
    }
}

/// Background task to broadcast metrics
async fn broadcast_metrics(state: Arc<StreamingState>) {
    let mut ticker = interval(Duration::from_millis(state.config.interval_ms));

    loop {
        ticker.tick().await;

        // Collect metrics
        let snapshots = state.collector.collect_snapshots();

        if !snapshots.is_empty() {
            let message = StreamMessage::MetricUpdate {
                snapshots,
                timestamp: chrono::Utc::now(),
            };

            if let Err(e) = state.broadcast_tx.send(message) {
                // No receivers, that's okay
                debug!("No active receivers: {}", e);
            }
        }
    }
}

/// Health check handler
async fn health_handler(State(state): State<Arc<StreamingState>>) -> impl IntoResponse {
    let connections = *state.active_connections.read();

    let response = serde_json::json!({
        "status": "healthy",
        "active_connections": connections,
        "max_connections": state.config.max_connections,
    });

    axum::Json(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_filter() {
        let filter = SubscriptionFilter {
            metrics: vec!["query_*".to_string()],
            labels: None,
            min_interval_ms: None,
        };

        let snapshot = MetricSnapshot {
            name: "query_latency".to_string(),
            help: "Query latency".to_string(),
            labels: std::collections::HashMap::new(),
            value: crate::types::MetricValue::Gauge { value: 100.0 },
            timestamp: chrono::Utc::now(),
        };

        assert!(filter.matches(&snapshot));

        let snapshot2 = MetricSnapshot {
            name: "other_metric".to_string(),
            help: "Other".to_string(),
            labels: std::collections::HashMap::new(),
            value: crate::types::MetricValue::Gauge { value: 100.0 },
            timestamp: chrono::Utc::now(),
        };

        assert!(!filter.matches(&snapshot2));
    }

    #[test]
    fn test_stream_message_serialization() {
        let msg = StreamMessage::Ping {
            timestamp: chrono::Utc::now(),
        };

        let json = msg.to_json().unwrap();
        let parsed = StreamMessage::from_json(&json).unwrap();

        match parsed {
            StreamMessage::Ping { .. } => {}
            _ => panic!("Wrong message type"),
        }
    }

    #[tokio::test]
    async fn test_streaming_server_creation() {
        let collector = Arc::new(MetricsCollector::default());
        let config = StreamingConfig {
            enabled: false,
            ..Default::default()
        };

        let server = StreamingServer::new(config, collector);
        assert_eq!(server.active_connections(), 0);
    }
}
