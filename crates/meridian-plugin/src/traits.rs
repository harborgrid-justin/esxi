//! Core trait definitions for the plugin system.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::PluginResult;

/// Plugin metadata information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier.
    pub id: String,

    /// Human-readable plugin name.
    pub name: String,

    /// Plugin version.
    pub version: semver::Version,

    /// Plugin description.
    pub description: String,

    /// Plugin author(s).
    pub authors: Vec<String>,

    /// Plugin license.
    pub license: Option<String>,

    /// Plugin homepage URL.
    pub homepage: Option<String>,

    /// Plugin dependencies.
    pub dependencies: Vec<PluginDependency>,

    /// Minimum platform version required.
    pub min_platform_version: semver::Version,

    /// Maximum platform version supported.
    pub max_platform_version: Option<semver::Version>,

    /// Plugin capabilities/permissions.
    pub capabilities: Vec<String>,

    /// Plugin tags/categories.
    pub tags: Vec<String>,
}

/// Plugin dependency specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Dependency plugin ID.
    pub id: String,

    /// Version requirement.
    pub version: semver::VersionReq,

    /// Whether this dependency is optional.
    pub optional: bool,
}

/// Plugin lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginState {
    /// Plugin is loaded but not initialized.
    Loaded,

    /// Plugin is initialized.
    Initialized,

    /// Plugin is running.
    Running,

    /// Plugin is paused.
    Paused,

    /// Plugin is stopped.
    Stopped,

    /// Plugin is unloaded.
    Unloaded,

    /// Plugin encountered an error.
    Error,
}

impl std::fmt::Display for PluginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginState::Loaded => write!(f, "Loaded"),
            PluginState::Initialized => write!(f, "Initialized"),
            PluginState::Running => write!(f, "Running"),
            PluginState::Paused => write!(f, "Paused"),
            PluginState::Stopped => write!(f, "Stopped"),
            PluginState::Unloaded => write!(f, "Unloaded"),
            PluginState::Error => write!(f, "Error"),
        }
    }
}

/// Core plugin trait that all plugins must implement.
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin metadata.
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin with configuration.
    async fn initialize(&mut self, config: PluginConfig) -> PluginResult<()>;

    /// Start the plugin.
    async fn start(&mut self) -> PluginResult<()>;

    /// Stop the plugin.
    async fn stop(&mut self) -> PluginResult<()>;

    /// Pause the plugin.
    async fn pause(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// Resume the plugin.
    async fn resume(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// Cleanup and unload the plugin.
    async fn cleanup(&mut self) -> PluginResult<()>;

    /// Get plugin state.
    fn state(&self) -> PluginState;

    /// Handle a message sent to this plugin.
    async fn handle_message(&mut self, message: PluginMessage) -> PluginResult<PluginMessage> {
        Ok(PluginMessage::error("Not implemented"))
    }

    /// Get plugin as Any for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Get plugin as mutable Any for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Configuration key-value pairs.
    pub settings: HashMap<String, serde_json::Value>,

    /// Plugin data directory.
    pub data_dir: std::path::PathBuf,

    /// Plugin cache directory.
    pub cache_dir: std::path::PathBuf,

    /// Plugin runtime instance ID.
    pub instance_id: Uuid,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            settings: HashMap::new(),
            data_dir: std::path::PathBuf::from("/tmp/plugins/data"),
            cache_dir: std::path::PathBuf::from("/tmp/plugins/cache"),
            instance_id: Uuid::new_v4(),
        }
    }
}

/// Message passed between plugins or from platform to plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMessage {
    /// Unique message ID.
    pub id: Uuid,

    /// Message type/topic.
    pub message_type: String,

    /// Message payload.
    pub payload: serde_json::Value,

    /// Message sender ID.
    pub sender: Option<String>,

    /// Message timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PluginMessage {
    /// Create a new message.
    pub fn new(message_type: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type: message_type.into(),
            payload,
            sender: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Create an error message.
    pub fn error(error: impl Into<String>) -> Self {
        Self::new(
            "error",
            serde_json::json!({ "error": error.into() }),
        )
    }

    /// Create a success message.
    pub fn success(data: serde_json::Value) -> Self {
        Self::new("success", data)
    }
}

/// Plugin context provided to plugins for platform interaction.
#[async_trait]
pub trait PluginContext: Send + Sync {
    /// Get the platform version.
    fn platform_version(&self) -> &semver::Version;

    /// Send a message to another plugin.
    async fn send_message(&self, target: &str, message: PluginMessage) -> PluginResult<()>;

    /// Register a hook handler.
    async fn register_hook(&self, hook: &str, handler_id: &str) -> PluginResult<()>;

    /// Unregister a hook handler.
    async fn unregister_hook(&self, hook: &str, handler_id: &str) -> PluginResult<()>;

    /// Get a configuration value.
    fn get_config(&self, key: &str) -> Option<serde_json::Value>;

    /// Set a configuration value.
    async fn set_config(&self, key: &str, value: serde_json::Value) -> PluginResult<()>;

    /// Log a message.
    fn log(&self, level: LogLevel, message: &str);
}

/// Log levels for plugin logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Plugin factory for creating plugin instances.
pub trait PluginFactory: Send + Sync {
    /// Create a new plugin instance.
    fn create(&self) -> PluginResult<Box<dyn Plugin>>;

    /// Get the plugin metadata without creating an instance.
    fn metadata(&self) -> &PluginMetadata;
}
