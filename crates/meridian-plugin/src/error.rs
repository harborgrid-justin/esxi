//! Error types for the Meridian plugin system.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

/// Comprehensive error types for the plugin system.
#[derive(Debug, Error)]
pub enum PluginError {
    /// Error loading a dynamic library.
    #[error("Failed to load library from {path}: {source}")]
    LibraryLoadError {
        path: PathBuf,
        source: libloading::Error,
    },

    /// Error loading a WASM module.
    #[error("Failed to load WASM module: {0}")]
    WasmLoadError(#[from] wasmtime::Error),

    /// Plugin not found.
    #[error("Plugin '{id}' not found")]
    PluginNotFound { id: String },

    /// Plugin already loaded.
    #[error("Plugin '{id}' is already loaded")]
    PluginAlreadyLoaded { id: String },

    /// Plugin initialization failed.
    #[error("Failed to initialize plugin '{id}': {reason}")]
    InitializationFailed { id: String, reason: String },

    /// Plugin lifecycle state error.
    #[error("Invalid lifecycle state transition from {from} to {to} for plugin '{id}'")]
    InvalidStateTransition {
        id: String,
        from: String,
        to: String,
    },

    /// Dependency resolution error.
    #[error("Dependency resolution failed for plugin '{id}': {reason}")]
    DependencyError { id: String, reason: String },

    /// Version incompatibility.
    #[error("Version incompatibility: {0}")]
    VersionIncompatible(String),

    /// Configuration error.
    #[error("Configuration error for plugin '{id}': {reason}")]
    ConfigError { id: String, reason: String },

    /// Marketplace error.
    #[error("Marketplace error: {0}")]
    MarketplaceError(String),

    /// Signing/verification error.
    #[error("Signature verification failed: {0}")]
    SignatureError(String),

    /// Hot-reload error.
    #[error("Hot-reload failed for plugin '{id}': {reason}")]
    HotReloadError { id: String, reason: String },

    /// Resource limit exceeded.
    #[error("Resource limit exceeded for plugin '{id}': {resource}")]
    ResourceLimitExceeded { id: String, resource: String },

    /// Inter-plugin communication error.
    #[error("IPC error: {0}")]
    IpcError(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Hook registration error.
    #[error("Failed to register hook '{hook}': {reason}")]
    HookError { hook: String, reason: String },

    /// Permission denied.
    #[error("Permission denied for plugin '{id}': {action}")]
    PermissionDenied { id: String, action: String },

    /// Timeout error.
    #[error("Operation timed out for plugin '{id}'")]
    Timeout { id: String },

    /// Generic error.
    #[error("Plugin error: {0}")]
    Generic(String),
}

impl From<serde_json::Error> for PluginError {
    fn from(err: serde_json::Error) -> Self {
        PluginError::SerializationError(err.to_string())
    }
}

impl From<toml::de::Error> for PluginError {
    fn from(err: toml::de::Error) -> Self {
        PluginError::SerializationError(err.to_string())
    }
}

impl From<reqwest::Error> for PluginError {
    fn from(err: reqwest::Error) -> Self {
        PluginError::MarketplaceError(err.to_string())
    }
}

impl From<notify::Error> for PluginError {
    fn from(err: notify::Error) -> Self {
        PluginError::HotReloadError {
            id: "unknown".to_string(),
            reason: err.to_string(),
        }
    }
}

impl From<url::ParseError> for PluginError {
    fn from(err: url::ParseError) -> Self {
        PluginError::MarketplaceError(err.to_string())
    }
}

impl From<wasmtime::MemoryAccessError> for PluginError {
    fn from(err: wasmtime::MemoryAccessError) -> Self {
        PluginError::WasmLoadError(err.into())
    }
}
