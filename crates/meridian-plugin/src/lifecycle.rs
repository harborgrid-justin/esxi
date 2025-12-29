//! Plugin lifecycle management.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

use crate::error::{PluginError, PluginResult};
use crate::traits::{Plugin, PluginConfig, PluginState};

/// Plugin lifecycle manager.
#[derive(Clone)]
pub struct LifecycleManager {
    plugins: Arc<RwLock<HashMap<String, PluginLifecycle>>>,
    timeout_duration: Duration,
}

impl LifecycleManager {
    /// Create a new lifecycle manager.
    pub fn new(timeout_duration: Duration) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            timeout_duration,
        }
    }

    /// Register a plugin for lifecycle management.
    pub async fn register(&self, plugin_id: String, plugin: Box<dyn Plugin>) {
        let lifecycle = PluginLifecycle {
            plugin,
            state: PluginState::Loaded,
        };

        self.plugins.write().await.insert(plugin_id, lifecycle);
    }

    /// Unregister a plugin.
    pub async fn unregister(&self, plugin_id: &str) -> PluginResult<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(lifecycle) = plugins.get(plugin_id) {
            if lifecycle.state != PluginState::Stopped
                && lifecycle.state != PluginState::Unloaded
            {
                return Err(PluginError::InvalidStateTransition {
                    id: plugin_id.to_string(),
                    from: lifecycle.state.to_string(),
                    to: "Unloaded".to_string(),
                });
            }
        }

        plugins.remove(plugin_id);
        Ok(())
    }

    /// Initialize a plugin.
    pub async fn initialize(
        &self,
        plugin_id: &str,
        config: PluginConfig,
    ) -> PluginResult<()> {
        let mut plugins = self.plugins.write().await;

        let lifecycle = plugins.get_mut(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        // Check state transition
        if lifecycle.state != PluginState::Loaded {
            return Err(PluginError::InvalidStateTransition {
                id: plugin_id.to_string(),
                from: lifecycle.state.to_string(),
                to: "Initialized".to_string(),
            });
        }

        // Initialize with timeout
        let result = timeout(
            self.timeout_duration,
            lifecycle.plugin.initialize(config),
        )
        .await;

        match result {
            Ok(Ok(())) => {
                lifecycle.state = PluginState::Initialized;
                Ok(())
            }
            Ok(Err(e)) => {
                lifecycle.state = PluginState::Error;
                Err(PluginError::InitializationFailed {
                    id: plugin_id.to_string(),
                    reason: e.to_string(),
                })
            }
            Err(_) => {
                lifecycle.state = PluginState::Error;
                Err(PluginError::Timeout {
                    id: plugin_id.to_string(),
                })
            }
        }
    }

    /// Start a plugin.
    pub async fn start(&self, plugin_id: &str) -> PluginResult<()> {
        let mut plugins = self.plugins.write().await;

        let lifecycle = plugins.get_mut(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        // Check state transition
        if lifecycle.state != PluginState::Initialized
            && lifecycle.state != PluginState::Stopped
            && lifecycle.state != PluginState::Paused
        {
            return Err(PluginError::InvalidStateTransition {
                id: plugin_id.to_string(),
                from: lifecycle.state.to_string(),
                to: "Running".to_string(),
            });
        }

        // Start with timeout
        let result = timeout(self.timeout_duration, lifecycle.plugin.start()).await;

        match result {
            Ok(Ok(())) => {
                lifecycle.state = PluginState::Running;
                Ok(())
            }
            Ok(Err(e)) => {
                lifecycle.state = PluginState::Error;
                Err(e)
            }
            Err(_) => {
                lifecycle.state = PluginState::Error;
                Err(PluginError::Timeout {
                    id: plugin_id.to_string(),
                })
            }
        }
    }

    /// Stop a plugin.
    pub async fn stop(&self, plugin_id: &str) -> PluginResult<()> {
        let mut plugins = self.plugins.write().await;

        let lifecycle = plugins.get_mut(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        // Check state transition
        if lifecycle.state != PluginState::Running
            && lifecycle.state != PluginState::Paused
        {
            return Err(PluginError::InvalidStateTransition {
                id: plugin_id.to_string(),
                from: lifecycle.state.to_string(),
                to: "Stopped".to_string(),
            });
        }

        // Stop with timeout
        let result = timeout(self.timeout_duration, lifecycle.plugin.stop()).await;

        match result {
            Ok(Ok(())) => {
                lifecycle.state = PluginState::Stopped;
                Ok(())
            }
            Ok(Err(e)) => {
                lifecycle.state = PluginState::Error;
                Err(e)
            }
            Err(_) => {
                lifecycle.state = PluginState::Error;
                Err(PluginError::Timeout {
                    id: plugin_id.to_string(),
                })
            }
        }
    }

    /// Pause a plugin.
    pub async fn pause(&self, plugin_id: &str) -> PluginResult<()> {
        let mut plugins = self.plugins.write().await;

        let lifecycle = plugins.get_mut(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        // Check state transition
        if lifecycle.state != PluginState::Running {
            return Err(PluginError::InvalidStateTransition {
                id: plugin_id.to_string(),
                from: lifecycle.state.to_string(),
                to: "Paused".to_string(),
            });
        }

        // Pause with timeout
        let result = timeout(self.timeout_duration, lifecycle.plugin.pause()).await;

        match result {
            Ok(Ok(())) => {
                lifecycle.state = PluginState::Paused;
                Ok(())
            }
            Ok(Err(e)) => {
                lifecycle.state = PluginState::Error;
                Err(e)
            }
            Err(_) => {
                lifecycle.state = PluginState::Error;
                Err(PluginError::Timeout {
                    id: plugin_id.to_string(),
                })
            }
        }
    }

    /// Resume a paused plugin.
    pub async fn resume(&self, plugin_id: &str) -> PluginResult<()> {
        let mut plugins = self.plugins.write().await;

        let lifecycle = plugins.get_mut(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        // Check state transition
        if lifecycle.state != PluginState::Paused {
            return Err(PluginError::InvalidStateTransition {
                id: plugin_id.to_string(),
                from: lifecycle.state.to_string(),
                to: "Running".to_string(),
            });
        }

        // Resume with timeout
        let result = timeout(self.timeout_duration, lifecycle.plugin.resume()).await;

        match result {
            Ok(Ok(())) => {
                lifecycle.state = PluginState::Running;
                Ok(())
            }
            Ok(Err(e)) => {
                lifecycle.state = PluginState::Error;
                Err(e)
            }
            Err(_) => {
                lifecycle.state = PluginState::Error;
                Err(PluginError::Timeout {
                    id: plugin_id.to_string(),
                })
            }
        }
    }

    /// Cleanup and unload a plugin.
    pub async fn cleanup(&self, plugin_id: &str) -> PluginResult<()> {
        let mut plugins = self.plugins.write().await;

        let lifecycle = plugins.get_mut(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        // Stop first if running
        if lifecycle.state == PluginState::Running
            || lifecycle.state == PluginState::Paused
        {
            let result = timeout(self.timeout_duration, lifecycle.plugin.stop()).await;

            if let Err(_) = result {
                tracing::warn!("Plugin '{}' stop timed out during cleanup", plugin_id);
            }
        }

        // Cleanup with timeout
        let result = timeout(self.timeout_duration, lifecycle.plugin.cleanup()).await;

        match result {
            Ok(Ok(())) => {
                lifecycle.state = PluginState::Unloaded;
                Ok(())
            }
            Ok(Err(e)) => {
                lifecycle.state = PluginState::Error;
                Err(e)
            }
            Err(_) => {
                lifecycle.state = PluginState::Error;
                Err(PluginError::Timeout {
                    id: plugin_id.to_string(),
                })
            }
        }
    }

    /// Get the current state of a plugin.
    pub async fn get_state(&self, plugin_id: &str) -> Option<PluginState> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).map(|lifecycle| lifecycle.state)
    }

    /// Get all plugin states.
    pub async fn get_all_states(&self) -> HashMap<String, PluginState> {
        let plugins = self.plugins.read().await;
        plugins
            .iter()
            .map(|(id, lifecycle)| (id.clone(), lifecycle.state))
            .collect()
    }

    /// Get a reference to a plugin.
    pub async fn get_plugin(&self, plugin_id: &str) -> Option<PluginRef> {
        let plugins = self.plugins.read().await;
        if plugins.contains_key(plugin_id) {
            Some(PluginRef {
                plugin_id: plugin_id.to_string(),
                plugins: Arc::clone(&self.plugins),
            })
        } else {
            None
        }
    }

    /// Execute full lifecycle: initialize -> start.
    pub async fn full_start(
        &self,
        plugin_id: &str,
        config: PluginConfig,
    ) -> PluginResult<()> {
        self.initialize(plugin_id, config).await?;
        self.start(plugin_id).await?;
        Ok(())
    }

    /// Execute full shutdown: stop -> cleanup.
    pub async fn full_shutdown(&self, plugin_id: &str) -> PluginResult<()> {
        // Stop first (ignore errors if already stopped)
        let _ = self.stop(plugin_id).await;

        // Cleanup
        self.cleanup(plugin_id).await?;
        Ok(())
    }
}

/// Internal plugin lifecycle state.
struct PluginLifecycle {
    plugin: Box<dyn Plugin>,
    state: PluginState,
}

/// Reference to a plugin for safe concurrent access.
pub struct PluginRef {
    plugin_id: String,
    plugins: Arc<RwLock<HashMap<String, PluginLifecycle>>>,
}

impl PluginRef {
    /// Get plugin state.
    pub async fn state(&self) -> Option<PluginState> {
        let plugins = self.plugins.read().await;
        plugins.get(&self.plugin_id).map(|lc| lc.state)
    }

    /// Check if plugin is in a specific state.
    pub async fn is_state(&self, state: PluginState) -> bool {
        self.state().await == Some(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::PluginMetadata;
    use async_trait::async_trait;
    use semver::Version;
    use std::any::Any;

    struct TestPlugin {
        metadata: PluginMetadata,
        state: PluginState,
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        async fn initialize(&mut self, _config: PluginConfig) -> PluginResult<()> {
            self.state = PluginState::Initialized;
            Ok(())
        }

        async fn start(&mut self) -> PluginResult<()> {
            self.state = PluginState::Running;
            Ok(())
        }

        async fn stop(&mut self) -> PluginResult<()> {
            self.state = PluginState::Stopped;
            Ok(())
        }

        async fn cleanup(&mut self) -> PluginResult<()> {
            self.state = PluginState::Unloaded;
            Ok(())
        }

        fn state(&self) -> PluginState {
            self.state
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    fn create_test_plugin() -> TestPlugin {
        TestPlugin {
            metadata: PluginMetadata {
                id: "test".to_string(),
                name: "Test Plugin".to_string(),
                version: Version::new(1, 0, 0),
                description: String::new(),
                authors: vec![],
                license: None,
                homepage: None,
                dependencies: vec![],
                min_platform_version: Version::new(0, 1, 0),
                max_platform_version: None,
                capabilities: vec![],
                tags: vec![],
            },
            state: PluginState::Loaded,
        }
    }

    #[tokio::test]
    async fn test_lifecycle_transitions() {
        let manager = LifecycleManager::new(Duration::from_secs(5));
        let plugin = create_test_plugin();

        manager
            .register("test".to_string(), Box::new(plugin))
            .await;

        // Initialize
        manager
            .initialize("test", PluginConfig::default())
            .await
            .unwrap();
        assert_eq!(
            manager.get_state("test").await,
            Some(PluginState::Initialized)
        );

        // Start
        manager.start("test").await.unwrap();
        assert_eq!(
            manager.get_state("test").await,
            Some(PluginState::Running)
        );

        // Stop
        manager.stop("test").await.unwrap();
        assert_eq!(
            manager.get_state("test").await,
            Some(PluginState::Stopped)
        );

        // Cleanup
        manager.cleanup("test").await.unwrap();
        assert_eq!(
            manager.get_state("test").await,
            Some(PluginState::Unloaded)
        );
    }

    #[tokio::test]
    async fn test_invalid_state_transition() {
        let manager = LifecycleManager::new(Duration::from_secs(5));
        let plugin = create_test_plugin();

        manager
            .register("test".to_string(), Box::new(plugin))
            .await;

        // Try to start without initializing
        let result = manager.start("test").await;
        assert!(result.is_err());
    }
}
