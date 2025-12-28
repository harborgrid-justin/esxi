//! Hot-reload support for plugin development.

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;

use crate::error::{PluginError, PluginResult};
use crate::lifecycle::LifecycleManager;
use crate::loader::DynamicLoader;
use crate::traits::PluginConfig;

/// Hot-reload manager for development workflows.
pub struct HotReloadManager {
    /// File system watcher.
    watcher: Option<RecommendedWatcher>,

    /// Event receiver.
    event_rx: Arc<RwLock<Option<UnboundedReceiver<notify::Result<Event>>>>>,

    /// Event sender (kept for watcher).
    event_tx: UnboundedSender<notify::Result<Event>>,

    /// Watched plugin paths.
    watched_paths: Arc<RwLock<HashMap<String, PathBuf>>>,

    /// Lifecycle manager for reloading.
    lifecycle_manager: LifecycleManager,

    /// Plugin loader.
    loader: Arc<RwLock<DynamicLoader>>,

    /// Reload debounce duration.
    debounce_duration: Duration,

    /// Whether hot-reload is enabled.
    enabled: bool,
}

impl HotReloadManager {
    /// Create a new hot-reload manager.
    pub fn new(
        lifecycle_manager: LifecycleManager,
        loader: DynamicLoader,
    ) -> PluginResult<Self> {
        let (event_tx, event_rx) = unbounded_channel();

        Ok(Self {
            watcher: None,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            event_tx,
            watched_paths: Arc::new(RwLock::new(HashMap::new())),
            lifecycle_manager,
            loader: Arc::new(RwLock::new(loader)),
            debounce_duration: Duration::from_millis(500),
            enabled: true,
        })
    }

    /// Start the hot-reload system.
    pub fn start(&mut self) -> PluginResult<()> {
        if !self.enabled {
            return Ok(());
        }

        tracing::info!("Starting hot-reload manager");

        let event_tx = self.event_tx.clone();

        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = event_tx.send(res);
        })?;

        self.watcher = Some(watcher);

        Ok(())
    }

    /// Watch a plugin for changes.
    pub async fn watch_plugin(&mut self, plugin_id: &str, plugin_path: PathBuf) -> PluginResult<()> {
        if !self.enabled {
            return Ok(());
        }

        tracing::info!("Watching plugin '{}' at {:?}", plugin_id, plugin_path);

        // Add to watched paths
        self.watched_paths
            .write()
            .await
            .insert(plugin_id.to_string(), plugin_path.clone());

        // Start watching the file
        if let Some(watcher) = &mut self.watcher {
            watcher.watch(&plugin_path, RecursiveMode::NonRecursive)?;
        }

        Ok(())
    }

    /// Stop watching a plugin.
    pub async fn unwatch_plugin(&mut self, plugin_id: &str) -> PluginResult<()> {
        let path = {
            let mut paths = self.watched_paths.write().await;
            paths.remove(plugin_id)
        };

        if let Some(path) = path {
            if let Some(watcher) = &mut self.watcher {
                watcher.unwatch(&path)?;
            }
        }

        Ok(())
    }

    /// Start the event processing loop.
    pub async fn process_events(&self) -> PluginResult<()> {
        let mut event_rx = self.event_rx.write().await.take().ok_or_else(|| {
            PluginError::HotReloadError {
                id: "system".to_string(),
                reason: "Event receiver already taken".to_string(),
            }
        })?;

        drop(std::mem::replace(&mut *self.event_rx.write().await, None));

        let watched_paths = Arc::clone(&self.watched_paths);
        let lifecycle_manager = self.lifecycle_manager.clone();
        let loader = Arc::clone(&self.loader);
        let debounce = self.debounce_duration;

        tokio::spawn(async move {
            let mut last_reload: HashMap<String, std::time::Instant> = HashMap::new();

            while let Some(event_result) = event_rx.recv().await {
                if let Ok(event) = event_result {
                    if let EventKind::Modify(_) = event.kind {
                        // Find which plugin was modified
                        let paths = watched_paths.read().await;

                        for (plugin_id, watched_path) in paths.iter() {
                            if event.paths.contains(watched_path) {
                                // Check debounce
                                let now = std::time::Instant::now();
                                let should_reload = last_reload
                                    .get(plugin_id)
                                    .map(|last| now.duration_since(*last) > debounce)
                                    .unwrap_or(true);

                                if should_reload {
                                    tracing::info!("Reloading plugin '{}' due to file change", plugin_id);

                                    // Perform reload
                                    if let Err(e) = Self::reload_plugin_internal(
                                        plugin_id,
                                        watched_path,
                                        &lifecycle_manager,
                                        &loader,
                                    )
                                    .await
                                    {
                                        tracing::error!("Failed to reload plugin '{}': {}", plugin_id, e);
                                    } else {
                                        last_reload.insert(plugin_id.clone(), now);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Reload a specific plugin.
    pub async fn reload_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        let paths = self.watched_paths.read().await;
        let plugin_path = paths.get(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        Self::reload_plugin_internal(
            plugin_id,
            plugin_path,
            &self.lifecycle_manager,
            &self.loader,
        )
        .await
    }

    /// Internal reload implementation.
    async fn reload_plugin_internal(
        plugin_id: &str,
        plugin_path: &Path,
        lifecycle_manager: &LifecycleManager,
        loader: &RwLock<DynamicLoader>,
    ) -> PluginResult<()> {
        // Stop the current plugin
        if let Err(e) = lifecycle_manager.stop(plugin_id).await {
            tracing::warn!("Failed to stop plugin during reload: {}", e);
        }

        // Cleanup
        if let Err(e) = lifecycle_manager.cleanup(plugin_id).await {
            tracing::warn!("Failed to cleanup plugin during reload: {}", e);
        }

        // Unregister
        if let Err(e) = lifecycle_manager.unregister(plugin_id).await {
            tracing::warn!("Failed to unregister plugin during reload: {}", e);
        }

        // Small delay to ensure file handles are released
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Reload the plugin
        let mut loader_guard = loader.write().await;
        let new_plugin = loader_guard.load_plugin(plugin_path)?;
        drop(loader_guard);

        // Register with lifecycle manager
        lifecycle_manager
            .register(plugin_id.to_string(), new_plugin)
            .await;

        // Initialize and start
        lifecycle_manager
            .initialize(plugin_id, PluginConfig::default())
            .await?;

        lifecycle_manager.start(plugin_id).await?;

        tracing::info!("Plugin '{}' reloaded successfully", plugin_id);

        Ok(())
    }

    /// Set debounce duration.
    pub fn set_debounce(&mut self, duration: Duration) {
        self.debounce_duration = duration;
    }

    /// Enable or disable hot-reload.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if hot-reload is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get watched plugin count.
    pub async fn watched_count(&self) -> usize {
        self.watched_paths.read().await.len()
    }
}

/// Hot-reload event handler trait.
#[async_trait::async_trait]
pub trait ReloadHandler: Send + Sync {
    /// Called before a plugin is reloaded.
    async fn before_reload(&self, plugin_id: &str) -> PluginResult<()>;

    /// Called after a plugin is reloaded.
    async fn after_reload(&self, plugin_id: &str, success: bool) -> PluginResult<()>;
}

/// Hot-reload statistics.
#[derive(Debug, Clone, Default)]
pub struct ReloadStats {
    /// Total reload attempts.
    pub total_attempts: u64,

    /// Successful reloads.
    pub successful: u64,

    /// Failed reloads.
    pub failed: u64,

    /// Last reload time.
    pub last_reload: Option<chrono::DateTime<chrono::Utc>>,

    /// Average reload duration.
    pub avg_duration_ms: f64,
}

impl ReloadStats {
    /// Record a reload attempt.
    pub fn record_attempt(&mut self, success: bool, duration: Duration) {
        self.total_attempts += 1;

        if success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }

        self.last_reload = Some(chrono::Utc::now());

        // Update average duration
        let total_duration = self.avg_duration_ms * (self.total_attempts - 1) as f64;
        let new_total = total_duration + duration.as_millis() as f64;
        self.avg_duration_ms = new_total / self.total_attempts as f64;
    }

    /// Get success rate.
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            (self.successful as f64 / self.total_attempts as f64) * 100.0
        }
    }
}

/// Hot-reload configuration.
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Whether to enable hot-reload.
    pub enabled: bool,

    /// Debounce duration for file changes.
    pub debounce_ms: u64,

    /// Whether to automatically reload on changes.
    pub auto_reload: bool,

    /// Whether to preserve plugin state on reload.
    pub preserve_state: bool,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            debounce_ms: 500,
            auto_reload: true,
            preserve_state: false,
        }
    }
}

/// Hot-reload builder for configuration.
pub struct HotReloadBuilder {
    config: HotReloadConfig,
}

impl HotReloadBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            config: HotReloadConfig::default(),
        }
    }

    /// Set enabled state.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set debounce duration.
    pub fn debounce_ms(mut self, ms: u64) -> Self {
        self.config.debounce_ms = ms;
        self
    }

    /// Set auto-reload.
    pub fn auto_reload(mut self, auto: bool) -> Self {
        self.config.auto_reload = auto;
        self
    }

    /// Set preserve state.
    pub fn preserve_state(mut self, preserve: bool) -> Self {
        self.config.preserve_state = preserve;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> HotReloadConfig {
        self.config
    }
}

impl Default for HotReloadBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reload_stats() {
        let mut stats = ReloadStats::default();

        stats.record_attempt(true, Duration::from_millis(100));
        stats.record_attempt(true, Duration::from_millis(200));
        stats.record_attempt(false, Duration::from_millis(150));

        assert_eq!(stats.total_attempts, 3);
        assert_eq!(stats.successful, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.success_rate(), 66.66666666666666);
    }

    #[test]
    fn test_hot_reload_config() {
        let config = HotReloadBuilder::new()
            .enabled(true)
            .debounce_ms(1000)
            .auto_reload(false)
            .build();

        assert!(config.enabled);
        assert_eq!(config.debounce_ms, 1000);
        assert!(!config.auto_reload);
    }
}
