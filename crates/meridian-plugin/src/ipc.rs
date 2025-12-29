//! Inter-plugin communication system.

use crossbeam_channel::{bounded, Receiver, Sender};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{PluginError, PluginResult};
use crate::traits::PluginMessage;

/// Inter-plugin communication manager.
#[derive(Clone)]
pub struct IpcManager {
    /// Message channels for each plugin.
    channels: Arc<DashMap<String, PluginChannel>>,

    /// Message bus for broadcast.
    message_bus: Arc<RwLock<MessageBus>>,

    /// RPC handlers.
    rpc_handlers: Arc<DashMap<String, RpcHandler>>,

    /// Event subscribers.
    event_subscribers: Arc<DashMap<String, Vec<String>>>,
}

impl IpcManager {
    /// Create a new IPC manager.
    pub fn new() -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
            message_bus: Arc::new(RwLock::new(MessageBus::new())),
            rpc_handlers: Arc::new(DashMap::new()),
            event_subscribers: Arc::new(DashMap::new()),
        }
    }

    /// Register a plugin for IPC.
    pub fn register_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        let (tx, rx) = bounded(100);

        let channel = PluginChannel {
            plugin_id: plugin_id.to_string(),
            sender: tx,
            receiver: Arc::new(RwLock::new(rx)),
        };

        self.channels.insert(plugin_id.to_string(), channel);

        tracing::debug!("Plugin '{}' registered for IPC", plugin_id);

        Ok(())
    }

    /// Unregister a plugin from IPC.
    pub fn unregister_plugin(&self, plugin_id: &str) {
        self.channels.remove(plugin_id);
        self.rpc_handlers.retain(|k, _| !k.starts_with(plugin_id));
        self.event_subscribers.retain(|_, v| {
            v.retain(|subscriber| subscriber != plugin_id);
            !v.is_empty()
        });

        tracing::debug!("Plugin '{}' unregistered from IPC", plugin_id);
    }

    /// Send a message to a specific plugin.
    pub async fn send_message(
        &self,
        from: &str,
        to: &str,
        message: PluginMessage,
    ) -> PluginResult<()> {
        let channel = self.channels.get(to).ok_or_else(|| {
            PluginError::IpcError(format!("Plugin '{}' not found", to))
        })?;

        let mut msg = message;
        msg.sender = Some(from.to_string());

        channel
            .sender
            .send(msg)
            .map_err(|e| PluginError::IpcError(format!("Failed to send message: {}", e)))?;

        Ok(())
    }

    /// Receive a message for a plugin (non-blocking).
    pub async fn receive_message(&self, plugin_id: &str) -> PluginResult<Option<PluginMessage>> {
        let channel = self.channels.get(plugin_id).ok_or_else(|| {
            PluginError::IpcError(format!("Plugin '{}' not found", plugin_id))
        })?;

        let receiver = channel.receiver.read().await;

        match receiver.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(crossbeam_channel::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(PluginError::IpcError(format!(
                "Failed to receive message: {}",
                e
            ))),
        }
    }

    /// Broadcast a message to all plugins.
    pub async fn broadcast(&self, from: &str, message: PluginMessage) -> PluginResult<()> {
        let mut msg = message;
        msg.sender = Some(from.to_string());

        for entry in self.channels.iter() {
            let plugin_id = entry.key();

            // Don't send to sender
            if plugin_id == from {
                continue;
            }

            let _ = entry.sender.send(msg.clone());
        }

        // Also add to message bus
        self.message_bus.write().await.publish(msg);

        Ok(())
    }

    /// Register an RPC handler.
    pub fn register_rpc_handler(
        &self,
        plugin_id: &str,
        method: &str,
        handler: RpcHandler,
    ) -> PluginResult<()> {
        let key = format!("{}::{}", plugin_id, method);
        self.rpc_handlers.insert(key, handler);

        tracing::debug!(
            "RPC handler registered: {}::{}",
            plugin_id,
            method
        );

        Ok(())
    }

    /// Call an RPC method on another plugin.
    pub async fn call_rpc(
        &self,
        from: &str,
        to: &str,
        method: &str,
        params: serde_json::Value,
    ) -> PluginResult<serde_json::Value> {
        let key = format!("{}::{}", to, method);

        let handler = self.rpc_handlers.get(&key).ok_or_else(|| {
            PluginError::IpcError(format!(
                "RPC method '{}' not found on plugin '{}'",
                method, to
            ))
        })?;

        let request = RpcRequest {
            id: Uuid::new_v4().to_string(),
            method: method.to_string(),
            params,
            caller: from.to_string(),
        };

        let result = (handler.callback)(request).await?;

        Ok(result)
    }

    /// Subscribe to an event.
    pub fn subscribe_event(&self, event_name: &str, subscriber: &str) -> PluginResult<()> {
        self.event_subscribers
            .entry(event_name.to_string())
            .or_insert_with(Vec::new)
            .push(subscriber.to_string());

        tracing::debug!(
            "Plugin '{}' subscribed to event '{}'",
            subscriber,
            event_name
        );

        Ok(())
    }

    /// Unsubscribe from an event.
    pub fn unsubscribe_event(&self, event_name: &str, subscriber: &str) {
        if let Some(mut subscribers) = self.event_subscribers.get_mut(event_name) {
            subscribers.retain(|s| s != subscriber);
        }
    }

    /// Publish an event.
    pub async fn publish_event(
        &self,
        publisher: &str,
        event_name: &str,
        data: serde_json::Value,
    ) -> PluginResult<()> {
        let subscribers = self
            .event_subscribers
            .get(event_name)
            .map(|s| s.clone())
            .unwrap_or_default();

        let message = PluginMessage::new(
            format!("event::{}", event_name),
            data,
        );

        for subscriber in subscribers {
            self.send_message(publisher, &subscriber, message.clone())
                .await?;
        }

        Ok(())
    }

    /// Get channel count.
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Get RPC handler count.
    pub fn rpc_handler_count(&self) -> usize {
        self.rpc_handlers.len()
    }
}

impl Default for IpcManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin message channel.
struct PluginChannel {
    plugin_id: String,
    sender: Sender<PluginMessage>,
    receiver: Arc<RwLock<Receiver<PluginMessage>>>,
}

/// Message bus for event publishing.
struct MessageBus {
    messages: Vec<PluginMessage>,
    max_messages: usize,
}

impl MessageBus {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 1000,
        }
    }

    fn publish(&mut self, message: PluginMessage) {
        self.messages.push(message);

        // Keep only recent messages
        if self.messages.len() > self.max_messages {
            self.messages.drain(0..self.messages.len() - self.max_messages);
        }
    }

    fn get_recent(&self, count: usize) -> Vec<PluginMessage> {
        let start = self.messages.len().saturating_sub(count);
        self.messages[start..].to_vec()
    }
}

/// RPC handler function.
pub struct RpcHandler {
    pub callback: Arc<
        dyn Fn(RpcRequest) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = PluginResult<serde_json::Value>> + Send>,
            > + Send
            + Sync,
    >,
}

impl RpcHandler {
    /// Create a new RPC handler.
    pub fn new<F, Fut>(callback: F) -> Self
    where
        F: Fn(RpcRequest) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = PluginResult<serde_json::Value>> + Send + 'static,
    {
        Self {
            callback: Arc::new(move |req| Box::pin(callback(req))),
        }
    }
}

/// RPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    /// Request ID.
    pub id: String,

    /// Method name.
    pub method: String,

    /// Parameters.
    pub params: serde_json::Value,

    /// Caller plugin ID.
    pub caller: String,
}

/// RPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    /// Request ID.
    pub id: String,

    /// Result (if successful).
    pub result: Option<serde_json::Value>,

    /// Error (if failed).
    pub error: Option<String>,
}

impl RpcResponse {
    /// Create a success response.
    pub fn success(id: String, result: serde_json::Value) -> Self {
        Self {
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: String, error: String) -> Self {
        Self {
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// Shared data store for inter-plugin data sharing.
#[derive(Clone)]
pub struct SharedDataStore {
    data: Arc<DashMap<String, serde_json::Value>>,
}

impl SharedDataStore {
    /// Create a new shared data store.
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    /// Set a value.
    pub fn set(&self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }

    /// Get a value.
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.data.get(key).map(|v| v.clone())
    }

    /// Delete a value.
    pub fn delete(&self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key).map(|(_, v)| v)
    }

    /// Check if a key exists.
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// List all keys.
    pub fn keys(&self) -> Vec<String> {
        self.data.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Clear all data.
    pub fn clear(&self) {
        self.data.clear();
    }

    /// Get entry count.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if store is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for SharedDataStore {
    fn default() -> Self {
        Self::new()
    }
}

/// IPC statistics.
#[derive(Debug, Clone, Default)]
pub struct IpcStats {
    /// Total messages sent.
    pub messages_sent: u64,

    /// Total messages received.
    pub messages_received: u64,

    /// Total RPC calls.
    pub rpc_calls: u64,

    /// Total events published.
    pub events_published: u64,

    /// Failed message count.
    pub failed_messages: u64,
}

impl IpcStats {
    /// Record a sent message.
    pub fn record_sent(&mut self) {
        self.messages_sent += 1;
    }

    /// Record a received message.
    pub fn record_received(&mut self) {
        self.messages_received += 1;
    }

    /// Record an RPC call.
    pub fn record_rpc(&mut self) {
        self.rpc_calls += 1;
    }

    /// Record an event publication.
    pub fn record_event(&mut self) {
        self.events_published += 1;
    }

    /// Record a failed message.
    pub fn record_failure(&mut self) {
        self.failed_messages += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ipc_messaging() {
        let ipc = IpcManager::new();

        ipc.register_plugin("plugin-a").unwrap();
        ipc.register_plugin("plugin-b").unwrap();

        let message = PluginMessage::new("test", serde_json::json!({"data": "hello"}));

        ipc.send_message("plugin-a", "plugin-b", message)
            .await
            .unwrap();

        let received = ipc.receive_message("plugin-b").await.unwrap();
        assert!(received.is_some());

        let msg = received.unwrap();
        assert_eq!(msg.message_type, "test");
        assert_eq!(msg.sender.as_deref(), Some("plugin-a"));
    }

    #[tokio::test]
    async fn test_rpc_call() {
        let ipc = IpcManager::new();

        ipc.register_plugin("plugin-a").unwrap();
        ipc.register_plugin("plugin-b").unwrap();

        // Register RPC handler
        let handler = RpcHandler::new(|req| async move {
            Ok(serde_json::json!({
                "echo": req.params,
                "caller": req.caller
            }))
        });

        ipc.register_rpc_handler("plugin-b", "echo", handler)
            .unwrap();

        // Call RPC
        let result = ipc
            .call_rpc(
                "plugin-a",
                "plugin-b",
                "echo",
                serde_json::json!({"message": "hello"}),
            )
            .await
            .unwrap();

        assert_eq!(result["caller"], "plugin-a");
        assert_eq!(result["echo"]["message"], "hello");
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let ipc = IpcManager::new();

        ipc.register_plugin("plugin-a").unwrap();
        ipc.register_plugin("plugin-b").unwrap();

        ipc.subscribe_event("test.event", "plugin-b").unwrap();

        ipc.publish_event(
            "plugin-a",
            "test.event",
            serde_json::json!({"data": "event data"}),
        )
        .await
        .unwrap();

        let received = ipc.receive_message("plugin-b").await.unwrap();
        assert!(received.is_some());

        let msg = received.unwrap();
        assert_eq!(msg.message_type, "event::test.event");
    }

    #[test]
    fn test_shared_data_store() {
        let store = SharedDataStore::new();

        store.set("key1".to_string(), serde_json::json!({"value": 42}));

        let value = store.get("key1").unwrap();
        assert_eq!(value["value"], 42);

        assert!(store.contains("key1"));
        assert_eq!(store.len(), 1);

        store.delete("key1");
        assert!(!store.contains("key1"));
    }
}
