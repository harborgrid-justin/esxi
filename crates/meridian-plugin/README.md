# Meridian Plugin System

Advanced plugin system for the Meridian GIS Platform v0.1.5, providing dynamic plugin loading, WASM runtime, lifecycle management, and comprehensive security features.

## Features

### Core Capabilities

- **Dynamic Plugin Loading**: Load native shared libraries at runtime
- **WASM Runtime**: Sandboxed WebAssembly plugin execution using Wasmtime
- **Lifecycle Management**: Complete state machine (Load → Initialize → Start → Stop → Unload)
- **Dependency Resolution**: Automatic dependency graph resolution and topological sorting
- **Version Compatibility**: Semantic versioning with compatibility checks
- **Hook System**: Extensibility points for platform integration
- **Configuration Management**: Plugin settings with validation and persistence
- **Marketplace Integration**: Plugin discovery, installation, and publishing
- **Cryptographic Signing**: Ed25519 signature verification for security
- **Hot-Reload**: Development mode with automatic plugin reloading
- **Resource Isolation**: CPU, memory, disk, and network limits
- **Inter-Plugin Communication**: Message passing, RPC, and event system

### Security Features

- **Signature Verification**: Ed25519 cryptographic signatures
- **Capability System**: Fine-grained permission control
- **Resource Limits**: Prevent resource exhaustion attacks
- **Sandbox Isolation**: WASM and filesystem sandboxing
- **Trust Chains**: Certificate-based trust verification

## Architecture

```
┌─────────────────────────────────────────────────────┐
│              Plugin Manager                         │
├─────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │   Loader     │  │   WASM       │  │ Lifecycle │ │
│  │  (Dynamic    │  │  Runtime     │  │  Manager  │ │
│  │  Libraries)  │  │ (Wasmtime)   │  │           │ │
│  └──────────────┘  └──────────────┘  └───────────┘ │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │ Dependency   │  │   Hooks      │  │    IPC    │ │
│  │  Resolver    │  │  Manager     │  │  Manager  │ │
│  └──────────────┘  └──────────────┘  └───────────┘ │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────┐ │
│  │  Resource    │  │    Config    │  │ Signing/  │ │
│  │  Monitor     │  │   Manager    │  │  Verify   │ │
│  └──────────────┘  └──────────────┘  └───────────┘ │
└─────────────────────────────────────────────────────┘
```

## Quick Start

### Basic Plugin Loading

```rust
use meridian_plugin::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create plugin manager
    let manager = PluginManager::new()?;

    // Load a native plugin
    manager.load_plugin("my-plugin", "/path/to/plugin.so").await?;

    // Initialize with configuration
    manager.initialize_plugin("my-plugin").await?;

    // Start the plugin
    manager.start_plugin("my-plugin").await?;

    // Use the plugin...

    // Stop and unload
    manager.stop_plugin("my-plugin").await?;
    manager.unload_plugin("my-plugin").await?;

    Ok(())
}
```

### WASM Plugin Loading

```rust
// Load a WASM plugin for sandboxed execution
manager.load_wasm_plugin("wasm-plugin", "/path/to/plugin.wasm").await?;

// WASM plugins have resource limits and isolation
let wasm_runtime = manager.wasm_runtime();
let result = wasm_runtime.call_function(
    "wasm-plugin",
    "process_data",
    vec![],
).await?;
```

### Plugin Implementation

```rust
use meridian_plugin::*;
use async_trait::async_trait;

pub struct MyPlugin {
    metadata: PluginMetadata,
    state: PluginState,
}

#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn initialize(&mut self, config: PluginConfig) -> PluginResult<()> {
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
```

## Module Overview

### `loader.rs` - Dynamic Library Loading

```rust
use meridian_plugin::*;

let mut loader = DynamicLoader::new();
loader.add_search_path("/usr/lib/meridian/plugins".into());

let plugin = loader.load_plugin_by_name("my-plugin")?;
```

### `wasm.rs` - WASM Runtime

```rust
let wasm_config = WasmConfig {
    max_stack_size: 1024 * 1024,
    epoch_timeout: 10000,
    max_memory_pages: 256,
};

let runtime = WasmRuntime::new(wasm_config)?;
runtime.load_plugin("wasm-plugin", wasm_path).await?;
```

### `lifecycle.rs` - State Management

```rust
let lifecycle = LifecycleManager::new(Duration::from_secs(30));

lifecycle.register("plugin-id", plugin).await;
lifecycle.initialize("plugin-id", config).await?;
lifecycle.start("plugin-id").await?;
lifecycle.stop("plugin-id").await?;
lifecycle.cleanup("plugin-id").await?;
```

### `dependency.rs` - Dependency Resolution

```rust
let mut resolver = DependencyResolver::new();

resolver.register(plugin_a_metadata);
resolver.register(plugin_b_metadata);

// Resolve dependencies for a plugin
let deps = resolver.resolve("plugin-b")?;

// Get load order for multiple plugins
let load_order = resolver.get_load_order(&["plugin-a", "plugin-b"])?;
```

### `hooks.rs` - Extensibility Hooks

```rust
let hook_manager = HookManager::new();

// Register a hook handler
let handler = HookHandler::new(
    "my-handler",
    "my-plugin",
    |ctx| async move {
        println!("Hook triggered!");
        Ok(())
    },
);

hook_manager.register_hook("plugin.loaded", handler)?;

// Trigger hooks
hook_manager.trigger("plugin.loaded", &mut context).await?;
```

### `ipc.rs` - Inter-Plugin Communication

```rust
let ipc = IpcManager::new();

ipc.register_plugin("plugin-a")?;
ipc.register_plugin("plugin-b")?;

// Send message
let message = PluginMessage::new("greeting", json!({"text": "Hello!"}));
ipc.send_message("plugin-a", "plugin-b", message).await?;

// RPC call
let handler = RpcHandler::new(|req| async move {
    Ok(json!({"result": "success"}))
});

ipc.register_rpc_handler("plugin-b", "echo", handler)?;

let result = ipc.call_rpc(
    "plugin-a",
    "plugin-b",
    "echo",
    json!({"data": "test"}),
).await?;
```

### `signing.rs` - Cryptographic Verification

```rust
// Sign a plugin
let signer = PluginSigner::generate();
let signature = signer.sign_plugin(&plugin_path).await?;

// Verify a plugin
let mut verifier = SignatureManager::new(true);
verifier.add_trusted_key(signer.public_key());
verifier.verify_plugin(&plugin_path).await?;
```

### `marketplace.rs` - Plugin Marketplace

```rust
let marketplace = MarketplaceClient::new(
    Url::parse("https://marketplace.meridian.dev")?,
    cache_dir,
)?;

// Search plugins
let results = marketplace.search("gis tools").await?;

// Install plugin
marketplace.install(
    "awesome-plugin",
    Some("1.0.0"),
    &install_dir,
).await?;

// Publish plugin
let metadata = PublishMetadata {
    name: "My Plugin".to_string(),
    version: "1.0.0".to_string(),
    // ...
};
marketplace.publish(&plugin_path, metadata).await?;
```

### `hotreload.rs` - Development Hot-Reload

```rust
let mut hot_reload = HotReloadManager::new(lifecycle, loader)?;

hot_reload.start()?;
hot_reload.watch_plugin("dev-plugin", plugin_path).await?;

// Process events (automatic reload on file changes)
hot_reload.process_events().await?;
```

### `isolation.rs` - Resource Limits

```rust
let monitor = ResourceMonitor::new(GlobalResourceLimits::default());

// Set plugin-specific limits
let limits = ResourceLimits {
    max_memory_bytes: 512 * 1024 * 1024, // 512 MB
    max_cpu_time_ms: 10000,              // 10 seconds
    max_threads: 4,
    max_disk_bytes: 100 * 1024 * 1024,   // 100 MB
    max_network_bytes_per_sec: 1024 * 1024, // 1 MB/s
};

monitor.set_limits("plugin-id", limits);

// Track resource usage
monitor.record_memory_alloc("plugin-id", 1024 * 1024)?;
monitor.record_cpu_time("plugin-id", duration)?;

// Check limits
monitor.check_limits("plugin-id")?;
```

## Configuration

### Plugin Manager Configuration

```rust
let config = PluginManagerConfig {
    platform_version: semver::Version::new(0, 1, 5),
    config_dir: PathBuf::from("/etc/meridian/plugins"),
    lifecycle_timeout: Duration::from_secs(30),
    wasm_config: WasmConfig::default(),
    global_resource_limits: GlobalResourceLimits::default(),
    enable_hot_reload: true,
};

let manager = PluginManager::with_config(config)?;
```

### Plugin Configuration File

```toml
# /etc/meridian/plugins/my-plugin.toml
version = "1.0"
enabled = true

[settings]
api_key = "secret-key"
endpoint = "https://api.example.com"
batch_size = 100

[limits]
max_memory_bytes = 536870912  # 512 MB
max_cpu_time_ms = 10000       # 10 seconds
max_threads = 4
max_disk_bytes = 104857600    # 100 MB
max_network_bytes_per_sec = 1048576  # 1 MB/s
```

## Security Best Practices

### 1. Signature Verification

Always verify plugin signatures in production:

```rust
let mut manager = SignatureManager::new(true); // Require signatures
manager.add_trusted_key_bytes(&trusted_key)?;
manager.verify_plugin(&plugin_path).await?;
```

### 2. Capability Restrictions

Limit plugin capabilities:

```rust
let sandbox = Sandbox::new(
    "plugin-id".to_string(),
    monitor,
    vec![
        Capability::FileSystemRead,
        Capability::Network,
        // Don't grant FileSystemWrite or ProcessSpawn
    ],
);
```

### 3. Resource Limits

Enforce strict resource limits:

```rust
let limits = ResourceLimits {
    max_memory_bytes: 256 * 1024 * 1024, // Conservative 256 MB
    max_cpu_time_ms: 5000,                // 5 seconds
    max_threads: 2,
    max_disk_bytes: 50 * 1024 * 1024,    // 50 MB
    max_network_bytes_per_sec: 512 * 1024, // 512 KB/s
};
```

### 4. WASM for Untrusted Code

Use WASM for untrusted plugins:

```rust
// WASM provides strong sandboxing
manager.load_wasm_plugin("untrusted-plugin", wasm_path).await?;
```

## Testing

Run tests:

```bash
cargo test -p meridian-plugin
```

Run with logging:

```bash
RUST_LOG=meridian_plugin=debug cargo test
```

## Performance

The plugin system is designed for high performance:

- **Lock-free data structures**: Uses DashMap for concurrent access
- **Zero-copy message passing**: Efficient IPC with crossbeam channels
- **Lazy loading**: Plugins loaded on-demand
- **Hot-reload optimization**: Debounced file watching
- **Resource monitoring**: Low-overhead tracking

## Examples

See the `examples/` directory for complete examples:

- `basic_plugin.rs` - Simple plugin implementation
- `wasm_plugin.rs` - WASM plugin example
- `plugin_communication.rs` - IPC and messaging
- `marketplace_client.rs` - Marketplace integration
- `hot_reload_dev.rs` - Development workflow

## Dependencies

Core dependencies:

- `libloading` - Dynamic library loading
- `wasmtime` - WASM runtime
- `tokio` - Async runtime
- `ed25519-dalek` - Cryptographic signing
- `semver` - Version management
- `serde` - Serialization

## License

Dual licensed under MIT OR Apache-2.0.

## Contributing

Contributions welcome! Please ensure:

1. All tests pass
2. Code is formatted with `cargo fmt`
3. No clippy warnings
4. Documentation is updated

## Roadmap

- [ ] Plugin marketplace web interface
- [ ] Python plugin bindings
- [ ] JavaScript/Node.js plugin support
- [ ] Plugin analytics and telemetry
- [ ] GUI plugin development tools
- [ ] Docker container plugins
- [ ] gRPC plugin protocol

## Support

For issues and questions:

- GitHub Issues: https://github.com/meridian/meridian-gis/issues
- Documentation: https://docs.meridian.dev/plugins
- Discord: https://discord.gg/meridian
