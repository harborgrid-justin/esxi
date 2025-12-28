//! WASM plugin runtime for sandboxed execution.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use wasmtime::*;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

use crate::error::{PluginError, PluginResult};
use crate::traits::PluginMessage;

/// WASM plugin runtime.
pub struct WasmRuntime {
    engine: Engine,
    instances: Arc<RwLock<HashMap<String, WasmInstance>>>,
    config: WasmConfig,
}

impl WasmRuntime {
    /// Create a new WASM runtime.
    pub fn new(config: WasmConfig) -> PluginResult<Self> {
        let mut engine_config = Config::new();

        // Enable WASI
        engine_config.wasm_backtrace_details(WasmBacktraceDetails::Enable);

        // Set resource limits
        engine_config.epoch_interruption(true);
        engine_config.max_wasm_stack(config.max_stack_size);

        // Enable multi-memory if needed
        engine_config.wasm_multi_memory(true);

        let engine = Engine::new(&engine_config)?;

        Ok(Self {
            engine,
            instances: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }

    /// Load a WASM plugin from a file.
    pub async fn load_plugin(&self, plugin_id: &str, wasm_path: &Path) -> PluginResult<()> {
        tracing::info!("Loading WASM plugin '{}' from {:?}", plugin_id, wasm_path);

        let wasm_bytes = tokio::fs::read(wasm_path).await?;

        self.load_plugin_bytes(plugin_id, &wasm_bytes).await
    }

    /// Load a WASM plugin from bytes.
    pub async fn load_plugin_bytes(&self, plugin_id: &str, wasm_bytes: &[u8]) -> PluginResult<()> {
        // Create a new store with WASI
        let mut store = self.create_store()?;

        // Compile the WASM module
        let module = Module::new(&self.engine, wasm_bytes)?;

        // Create WASI linker
        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s: &mut StoreData| &mut s.wasi)?;

        // Add host functions
        self.add_host_functions(&mut linker)?;

        // Instantiate the module
        let instance = linker.instantiate(&mut store, &module)?;

        // Initialize the plugin if it has an init function
        if let Ok(init) = instance.get_typed_func::<(), ()>(&mut store, "_plugin_init") {
            init.call(&mut store, ()).map_err(|e| {
                PluginError::InitializationFailed {
                    id: plugin_id.to_string(),
                    reason: e.to_string(),
                }
            })?;
        }

        // Store the instance
        let wasm_instance = WasmInstance {
            module,
            store,
            instance,
            state: WasmPluginState::Initialized,
        };

        self.instances
            .write()
            .insert(plugin_id.to_string(), wasm_instance);

        tracing::info!("Successfully loaded WASM plugin '{}'", plugin_id);

        Ok(())
    }

    /// Call a function in a WASM plugin.
    pub async fn call_function(
        &self,
        plugin_id: &str,
        function_name: &str,
        params: Vec<Val>,
    ) -> PluginResult<Vec<Val>> {
        let mut instances = self.instances.write();

        let wasm_instance = instances.get_mut(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        // Get the function
        let func = wasm_instance
            .instance
            .get_func(&mut wasm_instance.store, function_name)
            .ok_or_else(|| PluginError::Generic(format!("Function '{}' not found", function_name)))?;

        // Prepare results buffer
        let func_ty = func.ty(&wasm_instance.store);
        let mut results = vec![Val::I32(0); func_ty.results().len()];

        // Call the function
        func.call(&mut wasm_instance.store, &params, &mut results)?;

        Ok(results)
    }

    /// Send a message to a WASM plugin.
    pub async fn send_message(
        &self,
        plugin_id: &str,
        message: &PluginMessage,
    ) -> PluginResult<PluginMessage> {
        // Serialize message to JSON
        let message_json = serde_json::to_string(message)?;

        // Call the message handler function
        let result = self.call_string_function(plugin_id, "handle_message", &message_json).await?;

        // Deserialize response
        let response: PluginMessage = serde_json::from_str(&result)?;

        Ok(response)
    }

    /// Call a string-based function (helper method).
    async fn call_string_function(
        &self,
        plugin_id: &str,
        function_name: &str,
        input: &str,
    ) -> PluginResult<String> {
        let mut instances = self.instances.write();

        let wasm_instance = instances.get_mut(plugin_id).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_id.to_string(),
            }
        })?;

        // Allocate memory for input string
        let input_bytes = input.as_bytes();
        let input_len = input_bytes.len() as i32;

        // Call allocate function to get memory pointer
        let alloc_func = wasm_instance
            .instance
            .get_typed_func::<i32, i32>(&mut wasm_instance.store, "allocate")
            .map_err(|_| PluginError::Generic("allocate function not found".to_string()))?;

        let input_ptr = alloc_func.call(&mut wasm_instance.store, input_len)?;

        // Write input to memory
        let memory = wasm_instance
            .instance
            .get_memory(&mut wasm_instance.store, "memory")
            .ok_or_else(|| PluginError::Generic("Memory not found".to_string()))?;

        memory.write(&mut wasm_instance.store, input_ptr as usize, input_bytes)?;

        // Call the actual function
        let func = wasm_instance
            .instance
            .get_typed_func::<(i32, i32), i32>(&mut wasm_instance.store, function_name)
            .map_err(|_| PluginError::Generic(format!("Function '{}' not found", function_name)))?;

        let result_ptr = func.call(&mut wasm_instance.store, (input_ptr, input_len))?;

        // Read result from memory
        let result_len_bytes = &mut [0u8; 4];
        memory.read(&wasm_instance.store, result_ptr as usize, result_len_bytes)?;
        let result_len = u32::from_le_bytes(*result_len_bytes) as usize;

        let mut result_bytes = vec![0u8; result_len];
        memory.read(
            &wasm_instance.store,
            (result_ptr + 4) as usize,
            &mut result_bytes,
        )?;

        // Deallocate input memory
        if let Ok(dealloc_func) =
            wasm_instance
                .instance
                .get_typed_func::<(i32, i32), ()>(&mut wasm_instance.store, "deallocate")
        {
            let _ = dealloc_func.call(&mut wasm_instance.store, (input_ptr, input_len));
        }

        let result = String::from_utf8(result_bytes)
            .map_err(|e| PluginError::Generic(format!("Invalid UTF-8: {}", e)))?;

        Ok(result)
    }

    /// Unload a WASM plugin.
    pub async fn unload_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        let mut instances = self.instances.write();

        if let Some(mut wasm_instance) = instances.remove(plugin_id) {
            // Call cleanup if available
            if let Ok(cleanup) =
                wasm_instance
                    .instance
                    .get_typed_func::<(), ()>(&mut wasm_instance.store, "_plugin_cleanup")
            {
                let _ = cleanup.call(&mut wasm_instance.store, ());
            }
        }

        Ok(())
    }

    /// Create a new store with WASI context.
    fn create_store(&self) -> PluginResult<Store<StoreData>> {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()
            .map_err(|e| PluginError::Generic(format!("Failed to build WASI context: {}", e)))?
            .build();

        let data = StoreData { wasi };

        let mut store = Store::new(&self.engine, data);

        // Set resource limits
        store.limiter(|data| data as &mut dyn ResourceLimiter);

        // Set epoch deadline for timeout
        store.set_epoch_deadline(self.config.epoch_timeout);

        Ok(store)
    }

    /// Add host functions to the linker.
    fn add_host_functions(&self, linker: &mut Linker<StoreData>) -> PluginResult<()> {
        // Add logging function
        linker
            .func_wrap("env", "host_log", |mut _caller: Caller<'_, StoreData>, level: i32, ptr: i32, len: i32| {
                // Implementation would read string from memory and log it
                tracing::info!("Plugin log [{}]: ptr={}, len={}", level, ptr, len);
            })?;

        Ok(())
    }

    /// Get plugin state.
    pub fn get_state(&self, plugin_id: &str) -> Option<WasmPluginState> {
        self.instances
            .read()
            .get(plugin_id)
            .map(|instance| instance.state)
    }
}

/// WASM runtime configuration.
#[derive(Debug, Clone)]
pub struct WasmConfig {
    /// Maximum stack size in bytes.
    pub max_stack_size: usize,

    /// Epoch timeout for preventing infinite loops.
    pub epoch_timeout: u64,

    /// Maximum memory size in pages (64KB per page).
    pub max_memory_pages: u64,
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            max_stack_size: 1024 * 1024, // 1 MB
            epoch_timeout: 10000,         // 10 seconds worth of epochs
            max_memory_pages: 256,        // 16 MB
        }
    }
}

/// Store data for WASM instances.
struct StoreData {
    wasi: WasiCtx,
}

impl ResourceLimiter for StoreData {
    fn memory_growing(&mut self, _current: usize, desired: usize, maximum: Option<usize>) -> Result<bool, wasmtime::Error> {
        const MAX_MEMORY: usize = 16 * 1024 * 1024; // 16 MB

        let result = if let Some(max) = maximum {
            desired <= max
        } else {
            desired <= MAX_MEMORY
        };
        Ok(result)
    }

    fn table_growing(&mut self, _current: u32, desired: u32, maximum: Option<u32>) -> Result<bool, wasmtime::Error> {
        const MAX_TABLE_SIZE: u32 = 10000;

        let result = if let Some(max) = maximum {
            desired <= max
        } else {
            desired <= MAX_TABLE_SIZE
        };
        Ok(result)
    }
}

/// WASM plugin instance.
struct WasmInstance {
    module: Module,
    store: Store<StoreData>,
    instance: Instance,
    state: WasmPluginState,
}

/// WASM plugin state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmPluginState {
    Loading,
    Initialized,
    Running,
    Stopped,
    Error,
}

/// WASM plugin metadata (parsed from plugin).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmPluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

impl WasmPluginMetadata {
    /// Parse metadata from WASM custom section.
    pub fn from_wasm(wasm_bytes: &[u8]) -> PluginResult<Self> {
        // Parse WASM module to extract custom section with metadata
        // This is a simplified version - a real implementation would parse the WASM binary
        Ok(Self {
            id: "unknown".to_string(),
            name: "Unknown Plugin".to_string(),
            version: "0.0.0".to_string(),
            description: String::new(),
            author: String::new(),
        })
    }
}

/// WASM plugin builder for creating plugin modules.
pub struct WasmPluginBuilder {
    metadata: WasmPluginMetadata,
    functions: Vec<String>,
}

impl WasmPluginBuilder {
    /// Create a new WASM plugin builder.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            metadata: WasmPluginMetadata {
                id: id.into(),
                name: name.into(),
                version: "0.1.0".to_string(),
                description: String::new(),
                author: String::new(),
            },
            functions: vec![],
        }
    }

    /// Set version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.metadata.version = version.into();
        self
    }

    /// Set description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = description.into();
        self
    }

    /// Add a function.
    pub fn function(mut self, name: impl Into<String>) -> Self {
        self.functions.push(name.into());
        self
    }

    /// Build the metadata.
    pub fn build(self) -> WasmPluginMetadata {
        self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_config_default() {
        let config = WasmConfig::default();
        assert_eq!(config.max_stack_size, 1024 * 1024);
        assert_eq!(config.epoch_timeout, 10000);
    }

    #[test]
    fn test_wasm_plugin_builder() {
        let metadata = WasmPluginBuilder::new("test", "Test Plugin")
            .version("1.0.0")
            .description("A test plugin")
            .function("hello")
            .build();

        assert_eq!(metadata.id, "test");
        assert_eq!(metadata.name, "Test Plugin");
        assert_eq!(metadata.version, "1.0.0");
    }
}
