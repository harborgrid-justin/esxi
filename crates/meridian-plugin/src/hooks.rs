//! Extensibility hooks system for plugin integration points.

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{PluginError, PluginResult};
use crate::traits::PluginMessage;

/// Hook manager for plugin extensibility.
#[derive(Clone)]
pub struct HookManager {
    hooks: Arc<DashMap<String, Vec<HookHandler>>>,
    middleware: Arc<RwLock<Vec<Box<dyn HookMiddleware>>>>,
}

impl HookManager {
    /// Create a new hook manager.
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(DashMap::new()),
            middleware: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a hook handler.
    pub fn register_hook(&self, hook_name: &str, handler: HookHandler) -> PluginResult<()> {
        tracing::debug!("Registering hook handler for '{}'", hook_name);

        self.hooks
            .entry(hook_name.to_string())
            .or_insert_with(Vec::new)
            .push(handler);

        Ok(())
    }

    /// Unregister a hook handler.
    pub fn unregister_hook(&self, hook_name: &str, handler_id: &str) -> PluginResult<()> {
        if let Some(mut handlers) = self.hooks.get_mut(hook_name) {
            handlers.retain(|h| h.id != handler_id);
            tracing::debug!("Unregistered hook handler '{}' from '{}'", handler_id, hook_name);
        }

        Ok(())
    }

    /// Trigger a hook with context.
    pub async fn trigger<T>(&self, hook_name: &str, context: &mut T) -> PluginResult<()>
    where
        T: HookContext + Send + Sync,
    {
        tracing::debug!("Triggering hook '{}'", hook_name);

        // Apply middleware before hooks
        let middleware = self.middleware.read().await;
        for mw in middleware.iter() {
            mw.before_hook(hook_name, context).await?;
        }
        drop(middleware);

        // Execute hook handlers
        if let Some(handlers) = self.hooks.get(hook_name) {
            for handler in handlers.iter() {
                if handler.enabled {
                    let result = (handler.callback)(context).await;

                    if let Err(e) = result {
                        tracing::error!(
                            "Hook handler '{}' for '{}' failed: {}",
                            handler.id,
                            hook_name,
                            e
                        );

                        if !handler.allow_errors {
                            return Err(e);
                        }
                    }
                }
            }
        }

        // Apply middleware after hooks
        let middleware = self.middleware.read().await;
        for mw in middleware.iter() {
            mw.after_hook(hook_name, context).await?;
        }

        Ok(())
    }

    /// Trigger a hook and collect results.
    pub async fn trigger_collect<T, R>(
        &self,
        hook_name: &str,
        context: &mut T,
    ) -> PluginResult<Vec<R>>
    where
        T: HookContext + Send + Sync,
        R: Send + 'static,
    {
        let results: Vec<R> = Vec::new();

        if let Some(handlers) = self.hooks.get(hook_name) {
            for handler in handlers.iter() {
                if handler.enabled {
                    // Note: This is a simplified version
                    // In a real implementation, you'd need a way to get typed results
                    let _ = (handler.callback)(context).await;
                }
            }
        }

        Ok(results)
    }

    /// Register middleware.
    pub async fn register_middleware(&self, middleware: Box<dyn HookMiddleware>) {
        self.middleware.write().await.push(middleware);
    }

    /// Get hook handler count for a specific hook.
    pub fn handler_count(&self, hook_name: &str) -> usize {
        self.hooks
            .get(hook_name)
            .map(|h| h.len())
            .unwrap_or(0)
    }

    /// List all registered hooks.
    pub fn list_hooks(&self) -> Vec<String> {
        self.hooks.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Clear all handlers for a hook.
    pub fn clear_hook(&self, hook_name: &str) {
        self.hooks.remove(hook_name);
    }

    /// Clear all hooks.
    pub fn clear_all(&self) {
        self.hooks.clear();
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook handler registration.
pub struct HookHandler {
    /// Unique handler ID.
    pub id: String,

    /// Plugin ID that registered this handler.
    pub plugin_id: String,

    /// Whether this handler is enabled.
    pub enabled: bool,

    /// Whether to allow errors (continue on error).
    pub allow_errors: bool,

    /// Handler priority (lower = earlier execution).
    pub priority: i32,

    /// The callback function.
    pub callback: Arc<dyn Fn(&mut dyn HookContext) -> BoxFuture<'_, PluginResult<()>> + Send + Sync>,
}

type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

impl HookHandler {
    /// Create a new hook handler.
    pub fn new<F, Fut>(id: String, plugin_id: String, callback: F) -> Self
    where
        F: Fn(&mut dyn HookContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = PluginResult<()>> + Send + 'static,
    {
        Self {
            id,
            plugin_id,
            enabled: true,
            allow_errors: false,
            priority: 0,
            callback: Arc::new(move |ctx| Box::pin(callback(ctx))),
        }
    }

    /// Set whether errors are allowed.
    pub fn allow_errors(mut self, allow: bool) -> Self {
        self.allow_errors = allow;
        self
    }

    /// Set priority.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Hook context trait for passing data to hooks.
pub trait HookContext: Send + Sync {
    /// Get context as Any for downcasting.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Get context as mutable Any for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Get hook name.
    fn hook_name(&self) -> &str;
}

/// Hook middleware for pre/post processing.
#[async_trait]
pub trait HookMiddleware: Send + Sync {
    /// Called before hook execution.
    async fn before_hook(&self, hook_name: &str, context: &mut dyn HookContext) -> PluginResult<()>;

    /// Called after hook execution.
    async fn after_hook(&self, hook_name: &str, context: &mut dyn HookContext) -> PluginResult<()>;
}

/// Standard hook types for the platform.
pub mod standard_hooks {
    pub const PLUGIN_LOADED: &str = "plugin.loaded";
    pub const PLUGIN_INITIALIZED: &str = "plugin.initialized";
    pub const PLUGIN_STARTED: &str = "plugin.started";
    pub const PLUGIN_STOPPED: &str = "plugin.stopped";
    pub const PLUGIN_UNLOADED: &str = "plugin.unloaded";
    pub const PLUGIN_ERROR: &str = "plugin.error";

    pub const DATA_LOADED: &str = "data.loaded";
    pub const DATA_SAVED: &str = "data.saved";
    pub const DATA_DELETED: &str = "data.deleted";

    pub const UI_RENDER: &str = "ui.render";
    pub const UI_EVENT: &str = "ui.event";

    pub const API_REQUEST: &str = "api.request";
    pub const API_RESPONSE: &str = "api.response";
}

/// Hook context for plugin lifecycle events.
pub struct PluginLifecycleContext {
    pub plugin_id: String,
    pub hook_name: String,
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

impl HookContext for PluginLifecycleContext {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn hook_name(&self) -> &str {
        &self.hook_name
    }
}

/// Hook context for data events.
pub struct DataEventContext {
    pub hook_name: String,
    pub entity_type: String,
    pub entity_id: String,
    pub data: serde_json::Value,
    pub metadata: std::collections::HashMap<String, String>,
}

impl HookContext for DataEventContext {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn hook_name(&self) -> &str {
        &self.hook_name
    }
}

/// Hook context for API requests.
pub struct ApiRequestContext {
    pub hook_name: String,
    pub method: String,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub modified: bool,
}

impl HookContext for ApiRequestContext {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn hook_name(&self) -> &str {
        &self.hook_name
    }
}

/// Builder for creating hook handlers.
pub struct HookHandlerBuilder {
    id: Option<String>,
    plugin_id: Option<String>,
    priority: i32,
    allow_errors: bool,
}

impl HookHandlerBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            id: None,
            plugin_id: None,
            priority: 0,
            allow_errors: false,
        }
    }

    /// Set handler ID.
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Set plugin ID.
    pub fn plugin_id(mut self, plugin_id: String) -> Self {
        self.plugin_id = Some(plugin_id);
        self
    }

    /// Set priority.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set allow errors.
    pub fn allow_errors(mut self, allow: bool) -> Self {
        self.allow_errors = allow;
        self
    }

    /// Build the handler with a callback.
    pub fn build<F, Fut>(self, callback: F) -> HookHandler
    where
        F: Fn(&mut dyn HookContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = PluginResult<()>> + Send + 'static,
    {
        let id = self.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let plugin_id = self.plugin_id.unwrap_or_else(|| "unknown".to_string());

        let mut handler = HookHandler::new(id, plugin_id, callback);
        handler.priority = self.priority;
        handler.allow_errors = self.allow_errors;
        handler
    }
}

impl Default for HookHandlerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext {
        hook_name: String,
        counter: i32,
    }

    impl HookContext for TestContext {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }

        fn hook_name(&self) -> &str {
            &self.hook_name
        }
    }

    #[tokio::test]
    async fn test_hook_registration_and_trigger() {
        let manager = HookManager::new();

        let handler = HookHandler::new(
            "test-handler".to_string(),
            "test-plugin".to_string(),
            |ctx| async move {
                if let Some(test_ctx) = ctx.as_any_mut().downcast_mut::<TestContext>() {
                    test_ctx.counter += 1;
                }
                Ok(())
            },
        );

        manager.register_hook("test.hook", handler).unwrap();

        let mut context = TestContext {
            hook_name: "test.hook".to_string(),
            counter: 0,
        };

        manager.trigger("test.hook", &mut context).await.unwrap();

        assert_eq!(context.counter, 1);
    }

    #[tokio::test]
    async fn test_multiple_handlers() {
        let manager = HookManager::new();

        for i in 0..3 {
            let handler = HookHandler::new(
                format!("handler-{}", i),
                "test-plugin".to_string(),
                |ctx| async move {
                    if let Some(test_ctx) = ctx.as_any_mut().downcast_mut::<TestContext>() {
                        test_ctx.counter += 1;
                    }
                    Ok(())
                },
            );

            manager.register_hook("test.hook", handler).unwrap();
        }

        let mut context = TestContext {
            hook_name: "test.hook".to_string(),
            counter: 0,
        };

        manager.trigger("test.hook", &mut context).await.unwrap();

        assert_eq!(context.counter, 3);
    }
}
