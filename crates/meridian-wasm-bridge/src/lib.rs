//! # Meridian WASM Bridge
//!
//! Enterprise-grade WebAssembly bridge connecting TypeScript to Rust for the $983M Platform.
//! Provides high-performance, type-safe bindings for CAD, compression, query optimization,
//! collaboration, and security services.
//!
//! ## Features
//!
//! - **Zero-copy data transfer** between JavaScript and Rust
//! - **Async/await support** for seamless integration
//! - **Memory pooling** for optimal performance
//! - **Type-safe bindings** with automatic serialization
//! - **Enterprise security** with built-in validation
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │           TypeScript Application                │
//! └─────────────────┬───────────────────────────────┘
//!                   │
//!                   ▼
//! ┌─────────────────────────────────────────────────┐
//! │          WASM Bridge Layer (This Crate)         │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
//! │  │   CAD    │  │Compress  │  │  Query   │      │
//! │  │ Bindings │  │ Bindings │  │ Bindings │      │
//! │  └──────────┘  └──────────┘  └──────────┘      │
//! │  ┌──────────┐  ┌──────────┐                    │
//! │  │  Collab  │  │ Security │                    │
//! │  │ Bindings │  │ Bindings │                    │
//! │  └──────────┘  └──────────┘                    │
//! └─────────────────┬───────────────────────────────┘
//!                   │
//!                   ▼
//! ┌─────────────────────────────────────────────────┐
//! │         Rust Backend Services                   │
//! └─────────────────────────────────────────────────┘
//! ```

use wasm_bindgen::prelude::*;

pub mod async_bridge;
pub mod bindings;
pub mod memory;
pub mod types;

// Re-export key types for convenience
pub use types::*;

/// Initialize the WASM module with console error panic hook for better debugging.
///
/// # Example
///
/// ```javascript
/// import init, { initialize } from '@esxi/enterprise-bridge';
///
/// await init();
/// initialize();
/// ```
#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default();
}

/// Initialize the WASM bridge with custom configuration.
///
/// This function sets up logging, panic hooks, and initializes memory pools.
///
/// # Arguments
///
/// * `config` - JSON configuration object
///
/// # Returns
///
/// Returns `true` if initialization succeeds, `false` otherwise.
#[wasm_bindgen]
pub fn initialize(config: JsValue) -> Result<bool, JsValue> {
    let config: BridgeConfig = serde_wasm_bindgen::from_value(config)
        .map_err(|e| JsValue::from_str(&format!("Invalid configuration: {}", e)))?;

    tracing::info!("Initializing WASM bridge with config: {:?}", config);

    // Initialize memory pools
    memory::initialize_pools(config.memory_config.clone())?;

    // Set up performance monitoring
    if config.enable_performance_monitoring {
        tracing::info!("Performance monitoring enabled");
    }

    Ok(true)
}

/// Get the current version of the WASM bridge.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get bridge statistics and performance metrics.
///
/// Returns a JSON object containing:
/// - Memory usage
/// - Active operations
/// - Performance counters
#[wasm_bindgen]
pub fn get_stats() -> Result<JsValue, JsValue> {
    let stats = BridgeStats {
        version: version(),
        memory_usage: memory::get_usage()?,
        active_operations: 0, // TODO: Implement operation tracking
        uptime_ms: get_uptime_ms(),
    };

    serde_wasm_bindgen::to_value(&stats)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize stats: {}", e)))
}

/// Get the uptime of the WASM module in milliseconds.
fn get_uptime_ms() -> u64 {
    // Use web_sys to get performance.now()
    let window = web_sys::window().expect("no global window");
    let performance = window.performance().expect("no performance object");
    performance.now() as u64
}

/// Health check endpoint for the WASM bridge.
///
/// Returns `true` if the bridge is healthy and operational.
#[wasm_bindgen]
pub fn health_check() -> bool {
    // Verify critical components are operational
    memory::verify_health() && true // Add more checks as needed
}

/// Reset the WASM bridge state.
///
/// This clears all caches, resets memory pools, and cancels pending operations.
/// Use with caution in production environments.
#[wasm_bindgen]
pub fn reset() -> Result<(), JsValue> {
    tracing::warn!("Resetting WASM bridge - all state will be lost");

    memory::reset_pools()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }

    #[wasm_bindgen_test]
    fn test_health_check() {
        assert!(health_check());
    }
}
