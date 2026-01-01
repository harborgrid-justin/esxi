//! Shared type definitions between Rust and TypeScript.
//!
//! These types are serialized using serde and automatically converted between
//! Rust and JavaScript using wasm-bindgen.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Configuration for the WASM bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeConfig {
    /// Enable performance monitoring and metrics collection
    pub enable_performance_monitoring: bool,

    /// Memory configuration
    pub memory_config: MemoryConfig,

    /// Maximum number of concurrent operations
    #[serde(default = "default_max_concurrent_ops")]
    pub max_concurrent_operations: usize,

    /// Enable debug logging
    #[serde(default)]
    pub debug_mode: bool,
}

fn default_max_concurrent_ops() -> usize {
    100
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            enable_performance_monitoring: true,
            memory_config: MemoryConfig::default(),
            max_concurrent_operations: default_max_concurrent_ops(),
            debug_mode: false,
        }
    }
}

/// Memory configuration for the WASM bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryConfig {
    /// Initial pool size in bytes
    #[serde(default = "default_initial_pool_size")]
    pub initial_pool_size: usize,

    /// Maximum pool size in bytes
    #[serde(default = "default_max_pool_size")]
    pub max_pool_size: usize,

    /// Enable aggressive garbage collection
    #[serde(default)]
    pub aggressive_gc: bool,
}

fn default_initial_pool_size() -> usize {
    1024 * 1024 * 10 // 10 MB
}

fn default_max_pool_size() -> usize {
    1024 * 1024 * 100 // 100 MB
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            initial_pool_size: default_initial_pool_size(),
            max_pool_size: default_max_pool_size(),
            aggressive_gc: false,
        }
    }
}

/// Bridge statistics and performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeStats {
    /// Bridge version
    pub version: String,

    /// Current memory usage in bytes
    pub memory_usage: u64,

    /// Number of active operations
    pub active_operations: usize,

    /// Uptime in milliseconds
    pub uptime_ms: u64,
}

/// Memory usage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryUsage {
    /// Total allocated memory in bytes
    pub total_allocated: u64,

    /// Currently used memory in bytes
    pub used: u64,

    /// Available memory in bytes
    pub available: u64,

    /// Number of active allocations
    pub allocations: usize,
}

/// Result type for operations that can fail.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult<T> {
    /// Whether the operation succeeded
    pub success: bool,

    /// Result data (if successful)
    pub data: Option<T>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Operation duration in milliseconds
    pub duration_ms: Option<u64>,
}

impl<T> OperationResult<T> {
    pub fn success(data: T, duration_ms: Option<u64>) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            duration_ms,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            duration_ms: None,
        }
    }
}

/// CAD-specific types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CadGeometry {
    /// Geometry type (point, line, polygon, etc.)
    pub geometry_type: String,

    /// Coordinates array
    pub coordinates: Vec<Vec<f64>>,

    /// Properties/attributes
    pub properties: serde_json::Value,

    /// Bounding box [minX, minY, maxX, maxY]
    pub bbox: Option<Vec<f64>>,
}

/// Compression operation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressionParams {
    /// Compression algorithm (gzip, brotli, zstd, lz4)
    pub algorithm: String,

    /// Compression level (1-9)
    pub level: u8,

    /// Enable dictionary compression
    #[serde(default)]
    pub use_dictionary: bool,

    /// Dictionary data (if using dictionary compression)
    pub dictionary: Option<Vec<u8>>,
}

/// Query optimization parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    /// SQL query string
    pub query: String,

    /// Query parameters
    pub params: Vec<serde_json::Value>,

    /// Enable query optimization
    #[serde(default = "default_true")]
    pub optimize: bool,

    /// Maximum execution time in milliseconds
    pub timeout_ms: Option<u64>,
}

fn default_true() -> bool {
    true
}

/// Collaboration event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationEvent {
    /// Event type (insert, delete, update, cursor_move, etc.)
    pub event_type: String,

    /// User ID who triggered the event
    pub user_id: String,

    /// Timestamp in milliseconds
    pub timestamp: u64,

    /// Event payload
    pub payload: serde_json::Value,

    /// Vector clock for causality tracking
    pub vector_clock: Option<Vec<u64>>,
}

/// Security validation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityParams {
    /// Security check type (xss, sql_injection, csrf, etc.)
    pub check_type: String,

    /// Input data to validate
    pub input: String,

    /// Validation context
    pub context: Option<serde_json::Value>,
}

/// Security validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityResult {
    /// Whether the input is safe
    pub is_safe: bool,

    /// Detected threats
    pub threats: Vec<SecurityThreat>,

    /// Sanitized output (if applicable)
    pub sanitized: Option<String>,
}

/// Security threat information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityThreat {
    /// Threat type
    pub threat_type: String,

    /// Severity level (low, medium, high, critical)
    pub severity: String,

    /// Threat description
    pub description: String,

    /// Location in input where threat was detected
    pub location: Option<LocationInfo>,
}

/// Location information for errors and threats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationInfo {
    /// Line number
    pub line: usize,

    /// Column number
    pub column: usize,

    /// Length of the problematic section
    pub length: usize,
}

/// Binary data transfer wrapper
#[wasm_bindgen]
pub struct BinaryData {
    data: Vec<u8>,
}

#[wasm_bindgen]
impl BinaryData {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get the length of the binary data
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.data.len()
    }

    /// Get a pointer to the data (zero-copy transfer)
    #[wasm_bindgen(getter)]
    pub fn ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Convert to JavaScript Uint8Array
    pub fn to_uint8_array(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.data[..])
    }

    /// Create from JavaScript Uint8Array
    pub fn from_uint8_array(array: &js_sys::Uint8Array) -> Self {
        let mut data = vec![0u8; array.length() as usize];
        array.copy_to(&mut data);
        Self { data }
    }
}

impl BinaryData {
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
}
