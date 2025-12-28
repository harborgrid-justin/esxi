//! # Meridian Plugin System
//!
//! Advanced plugin system for the Meridian GIS Platform providing:
//!
//! - Dynamic plugin loading from shared libraries
//! - WASM runtime for sandboxed plugin execution
//! - Complete lifecycle management (load, init, start, stop, unload)
//! - Automatic dependency resolution
//! - Semantic versioning and compatibility checks
//! - Extensibility hooks for integration points
//! - Plugin configuration and settings management
//! - Marketplace integration for plugin discovery and installation
//! - Cryptographic signing and verification
//! - Hot-reload support for development workflows
//! - Resource isolation and limits
//! - Inter-plugin communication (IPC)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use meridian_plugin::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a plugin manager
//!     let manager = PluginManager::new()?;
//!
//!     // Load a plugin
//!     manager.load_plugin("my-plugin", "/path/to/plugin.so").await?;
//!
//!     // Initialize and start the plugin
//!     manager.initialize_plugin("my-plugin").await?;
//!     manager.start_plugin("my-plugin").await?;
//!
//!     // Use the plugin...
//!
//!     // Stop and unload
//!     manager.stop_plugin("my-plugin").await?;
//!     manager.unload_plugin("my-plugin").await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The plugin system is built around several core components:
//!
//! - **Loader**: Handles dynamic library loading and WASM module instantiation
//! - **Lifecycle Manager**: Controls plugin state transitions
//! - **Dependency Resolver**: Manages plugin dependencies and load order
//! - **Hook Manager**: Provides extensibility points
//! - **IPC Manager**: Enables inter-plugin communication
//! - **Resource Monitor**: Enforces resource limits and isolation
//! - **Marketplace Client**: Integrates with plugin marketplace
//!
//! ## Security
//!
//! The plugin system provides multiple layers of security:
//!
//! - Cryptographic signature verification
//! - Capability-based permissions
//! - Resource limits (CPU, memory, disk, network)
//! - File system sandboxing
//! - WASM sandboxing for untrusted code
//!

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod dependency;
pub mod error;
pub mod hooks;
pub mod hotreload;
pub mod ipc;
pub mod isolation;
pub mod lifecycle;
pub mod loader;
pub mod marketplace;
pub mod signing;
pub mod traits;
pub mod versioning;
pub mod wasm;

// Re-export commonly used types
pub use config::{ConfigManager, PluginConfigData, ResourceLimits};
pub use dependency::{DependencyResolver, DependencyTree};
pub use error::{PluginError, PluginResult};
pub use hooks::{HookContext, HookHandler, HookManager};
pub use hotreload::{HotReloadConfig, HotReloadManager};
pub use ipc::{IpcManager, RpcHandler, SharedDataStore};
pub use isolation::{Capability, ResourceMonitor, Sandbox};
pub use lifecycle::LifecycleManager;
pub use loader::{DynamicLoader, PluginDiscovery, SafePluginLoader};
pub use marketplace::{MarketplaceClient, MarketplacePlugin};
pub use signing::{PluginSigner, SignatureManager};
pub use traits::{
    LogLevel, Plugin, PluginConfig, PluginContext, PluginDependency, PluginFactory,
    PluginMessage, PluginMetadata, PluginState,
};
pub use versioning::{VersionChecker, VersionManifest};
pub use wasm::{WasmConfig, WasmRuntime};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// Comprehensive plugin manager that integrates all subsystems.
pub struct PluginManager {
    /// Plugin lifecycle manager.
    lifecycle: LifecycleManager,

    /// Dynamic library loader.
    loader: Arc<RwLock<DynamicLoader>>,

    /// WASM runtime.
    wasm_runtime: Arc<RwLock<WasmRuntime>>,

    /// Dependency resolver.
    dependency_resolver: Arc<RwLock<DependencyResolver>>,

    /// Version checker.
    version_checker: VersionChecker,

    /// Hook manager.
    hook_manager: HookManager,

    /// IPC manager.
    ipc_manager: IpcManager,

    /// Resource monitor.
    resource_monitor: ResourceMonitor,

    /// Configuration manager.
    config_manager: Arc<RwLock<ConfigManager>>,

    /// Hot-reload manager (optional).
    hot_reload: Option<Arc<RwLock<HotReloadManager>>>,

    /// Platform version.
    platform_version: semver::Version,
}

impl PluginManager {
    /// Create a new plugin manager with default configuration.
    pub fn new() -> PluginResult<Self> {
        Self::with_config(PluginManagerConfig::default())
    }

    /// Create a new plugin manager with custom configuration.
    pub fn with_config(config: PluginManagerConfig) -> PluginResult<Self> {
        let lifecycle = LifecycleManager::new(config.lifecycle_timeout);
        let loader = Arc::new(RwLock::new(DynamicLoader::new()));
        let wasm_runtime = Arc::new(RwLock::new(WasmRuntime::new(config.wasm_config)?));
        let dependency_resolver = Arc::new(RwLock::new(DependencyResolver::new()));
        let version_checker = VersionChecker::new(config.platform_version.clone());
        let hook_manager = HookManager::new();
        let ipc_manager = IpcManager::new();
        let resource_monitor = ResourceMonitor::new(config.global_resource_limits);
        let config_manager = Arc::new(RwLock::new(ConfigManager::new(config.config_dir)));

        let hot_reload = if config.enable_hot_reload {
            let hr_manager = HotReloadManager::new(
                lifecycle.clone(),
                DynamicLoader::new(),
            )?;
            Some(Arc::new(RwLock::new(hr_manager)))
        } else {
            None
        };

        Ok(Self {
            lifecycle,
            loader,
            wasm_runtime,
            dependency_resolver,
            version_checker,
            hook_manager,
            ipc_manager,
            resource_monitor,
            config_manager,
            hot_reload,
            platform_version: config.platform_version,
        })
    }

    /// Load a plugin from a dynamic library.
    pub async fn load_plugin(&self, plugin_id: &str, plugin_path: &Path) -> PluginResult<()> {
        tracing::info!("Loading plugin '{}' from {:?}", plugin_id, plugin_path);

        // Load the plugin
        let mut loader = self.loader.write().await;
        let plugin = loader.load_plugin(plugin_path)?;
        drop(loader);

        // Get metadata
        let metadata = plugin.metadata().clone();

        // Check version compatibility
        self.version_checker.is_compatible(&metadata)?;

        // Register with dependency resolver
        self.dependency_resolver
            .write()
            .await
            .register(metadata.clone());

        // Register with lifecycle manager
        self.lifecycle
            .register(plugin_id.to_string(), plugin)
            .await;

        // Register with IPC manager
        self.ipc_manager.register_plugin(plugin_id)?;

        // Start resource tracking
        self.resource_monitor.start_tracking(plugin_id);

        // Trigger hook
        let mut ctx = hooks::PluginLifecycleContext {
            plugin_id: plugin_id.to_string(),
            hook_name: hooks::standard_hooks::PLUGIN_LOADED.to_string(),
            data: Default::default(),
        };
        let _ = self.hook_manager
            .trigger(hooks::standard_hooks::PLUGIN_LOADED, &mut ctx)
            .await;

        tracing::info!("Plugin '{}' loaded successfully", plugin_id);

        Ok(())
    }

    /// Load a WASM plugin.
    pub async fn load_wasm_plugin(&self, plugin_id: &str, wasm_path: &Path) -> PluginResult<()> {
        tracing::info!("Loading WASM plugin '{}' from {:?}", plugin_id, wasm_path);

        let wasm = self.wasm_runtime.read().await;
        wasm.load_plugin(plugin_id, wasm_path).await?;

        // Register with IPC
        self.ipc_manager.register_plugin(plugin_id)?;

        // Start resource tracking
        self.resource_monitor.start_tracking(plugin_id);

        tracing::info!("WASM plugin '{}' loaded successfully", plugin_id);

        Ok(())
    }

    /// Initialize a plugin.
    pub async fn initialize_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        // Load configuration
        let config_data = self
            .config_manager
            .write()
            .await
            .load(plugin_id)
            .await?;

        let config = PluginConfig {
            settings: config_data.settings,
            data_dir: PathBuf::from(format!("/var/lib/meridian/plugins/{}", plugin_id)),
            cache_dir: PathBuf::from(format!("/var/cache/meridian/plugins/{}", plugin_id)),
            instance_id: uuid::Uuid::new_v4(),
        };

        // Set resource limits
        self.resource_monitor
            .set_limits(plugin_id, config_data.limits);

        // Initialize
        self.lifecycle.initialize(plugin_id, config).await?;

        // Trigger hook
        let mut ctx = hooks::PluginLifecycleContext {
            plugin_id: plugin_id.to_string(),
            hook_name: hooks::standard_hooks::PLUGIN_INITIALIZED.to_string(),
            data: Default::default(),
        };
        let _ = self.hook_manager
            .trigger(hooks::standard_hooks::PLUGIN_INITIALIZED, &mut ctx)
            .await;

        Ok(())
    }

    /// Start a plugin.
    pub async fn start_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        self.lifecycle.start(plugin_id).await?;

        // Trigger hook
        let mut ctx = hooks::PluginLifecycleContext {
            plugin_id: plugin_id.to_string(),
            hook_name: hooks::standard_hooks::PLUGIN_STARTED.to_string(),
            data: Default::default(),
        };
        let _ = self.hook_manager
            .trigger(hooks::standard_hooks::PLUGIN_STARTED, &mut ctx)
            .await;

        Ok(())
    }

    /// Stop a plugin.
    pub async fn stop_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        self.lifecycle.stop(plugin_id).await?;

        // Trigger hook
        let mut ctx = hooks::PluginLifecycleContext {
            plugin_id: plugin_id.to_string(),
            hook_name: hooks::standard_hooks::PLUGIN_STOPPED.to_string(),
            data: Default::default(),
        };
        let _ = self.hook_manager
            .trigger(hooks::standard_hooks::PLUGIN_STOPPED, &mut ctx)
            .await;

        Ok(())
    }

    /// Unload a plugin.
    pub async fn unload_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        // Cleanup
        self.lifecycle.cleanup(plugin_id).await?;

        // Unregister
        self.lifecycle.unregister(plugin_id).await?;

        // Unregister from IPC
        self.ipc_manager.unregister_plugin(plugin_id);

        // Stop resource tracking
        self.resource_monitor.stop_tracking(plugin_id);

        // Unregister from dependency resolver
        self.dependency_resolver.write().await.unregister(plugin_id);

        // Trigger hook
        let mut ctx = hooks::PluginLifecycleContext {
            plugin_id: plugin_id.to_string(),
            hook_name: hooks::standard_hooks::PLUGIN_UNLOADED.to_string(),
            data: Default::default(),
        };
        let _ = self.hook_manager
            .trigger(hooks::standard_hooks::PLUGIN_UNLOADED, &mut ctx)
            .await;

        tracing::info!("Plugin '{}' unloaded", plugin_id);

        Ok(())
    }

    /// Get plugin state.
    pub async fn get_plugin_state(&self, plugin_id: &str) -> Option<PluginState> {
        self.lifecycle.get_state(plugin_id).await
    }

    /// Get all plugin states.
    pub async fn get_all_plugin_states(&self) -> std::collections::HashMap<String, PluginState> {
        self.lifecycle.get_all_states().await
    }

    /// Get the hook manager.
    pub fn hooks(&self) -> &HookManager {
        &self.hook_manager
    }

    /// Get the IPC manager.
    pub fn ipc(&self) -> &IpcManager {
        &self.ipc_manager
    }

    /// Get the resource monitor.
    pub fn resources(&self) -> &ResourceMonitor {
        &self.resource_monitor
    }

    /// Enable hot-reload for a plugin.
    pub async fn enable_hot_reload(&self, plugin_id: &str, plugin_path: PathBuf) -> PluginResult<()> {
        if let Some(hot_reload) = &self.hot_reload {
            hot_reload
                .write()
                .await
                .watch_plugin(plugin_id, plugin_path)
                .await?;
        }

        Ok(())
    }

    /// Get platform version.
    pub fn platform_version(&self) -> &semver::Version {
        &self.platform_version
    }
}

/// Configuration for the plugin manager.
#[derive(Clone)]
pub struct PluginManagerConfig {
    /// Platform version.
    pub platform_version: semver::Version,

    /// Configuration directory.
    pub config_dir: PathBuf,

    /// Lifecycle operation timeout.
    pub lifecycle_timeout: Duration,

    /// WASM runtime configuration.
    pub wasm_config: WasmConfig,

    /// Global resource limits.
    pub global_resource_limits: isolation::GlobalResourceLimits,

    /// Enable hot-reload.
    pub enable_hot_reload: bool,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            platform_version: semver::Version::new(0, 1, 5),
            config_dir: PathBuf::from("/etc/meridian/plugins"),
            lifecycle_timeout: Duration::from_secs(30),
            wasm_config: WasmConfig::default(),
            global_resource_limits: isolation::GlobalResourceLimits::default(),
            enable_hot_reload: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_plugin_manager_config() {
        let config = PluginManagerConfig {
            platform_version: semver::Version::new(1, 0, 0),
            enable_hot_reload: true,
            ..Default::default()
        };

        let manager = PluginManager::with_config(config);
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert_eq!(manager.platform_version(), &semver::Version::new(1, 0, 0));
    }
}
