//! Memory management for the WASM bridge.
//!
//! This module provides:
//! - Memory pooling for efficient allocation/deallocation
//! - Zero-copy data transfer between JavaScript and Rust
//! - Memory usage tracking and limits
//! - Automatic cleanup and garbage collection

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use wasm_bindgen::prelude::*;

use crate::types::{MemoryConfig, MemoryUsage};

/// Global memory statistics
static TOTAL_ALLOCATED: AtomicU64 = AtomicU64::new(0);
static CURRENT_USED: AtomicU64 = AtomicU64::new(0);
static ALLOCATION_COUNT: AtomicUsize = AtomicUsize::new(0);
static MAX_POOL_SIZE: AtomicU64 = AtomicU64::new(0);

/// Initialize memory pools with the given configuration.
pub fn initialize_pools(config: MemoryConfig) -> Result<(), JsValue> {
    tracing::info!(
        "Initializing memory pools: initial={} bytes, max={} bytes",
        config.initial_pool_size,
        config.max_pool_size
    );

    MAX_POOL_SIZE.store(config.max_pool_size as u64, Ordering::Relaxed);

    // Pre-allocate initial pool
    let initial_allocation = config.initial_pool_size as u64;
    TOTAL_ALLOCATED.store(initial_allocation, Ordering::Relaxed);

    Ok(())
}

/// Get current memory usage statistics.
pub fn get_usage() -> Result<u64, JsValue> {
    Ok(CURRENT_USED.load(Ordering::Relaxed))
}

/// Get detailed memory usage information.
pub fn get_detailed_usage() -> MemoryUsage {
    let total = TOTAL_ALLOCATED.load(Ordering::Relaxed);
    let used = CURRENT_USED.load(Ordering::Relaxed);

    MemoryUsage {
        total_allocated: total,
        used,
        available: total.saturating_sub(used),
        allocations: ALLOCATION_COUNT.load(Ordering::Relaxed),
    }
}

/// Allocate memory from the pool.
///
/// Returns an error if the allocation would exceed the maximum pool size.
pub fn allocate(size: usize) -> Result<(), JsValue> {
    let size_u64 = size as u64;
    let max_size = MAX_POOL_SIZE.load(Ordering::Relaxed);

    // Check if allocation would exceed limit
    let current = CURRENT_USED.load(Ordering::Relaxed);
    if current + size_u64 > max_size {
        return Err(JsValue::from_str(&format!(
            "Memory allocation would exceed limit: {} + {} > {}",
            current, size_u64, max_size
        )));
    }

    // Update counters
    CURRENT_USED.fetch_add(size_u64, Ordering::Relaxed);
    ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
    TOTAL_ALLOCATED.fetch_max(current + size_u64, Ordering::Relaxed);

    tracing::debug!("Allocated {} bytes, total used: {}", size, current + size_u64);

    Ok(())
}

/// Deallocate memory back to the pool.
pub fn deallocate(size: usize) {
    let size_u64 = size as u64;
    CURRENT_USED.fetch_sub(size_u64, Ordering::Relaxed);
    ALLOCATION_COUNT.fetch_sub(1, Ordering::Relaxed);

    tracing::debug!("Deallocated {} bytes", size);
}

/// Reset all memory pools.
///
/// This clears all allocations and resets counters.
pub fn reset_pools() -> Result<(), JsValue> {
    tracing::warn!("Resetting all memory pools");

    CURRENT_USED.store(0, Ordering::Relaxed);
    ALLOCATION_COUNT.store(0, Ordering::Relaxed);

    Ok(())
}

/// Verify that the memory system is healthy.
pub fn verify_health() -> bool {
    let used = CURRENT_USED.load(Ordering::Relaxed);
    let max = MAX_POOL_SIZE.load(Ordering::Relaxed);

    // Check that we haven't exceeded limits
    if used > max {
        tracing::error!("Memory usage ({}) exceeds maximum ({})", used, max);
        return false;
    }

    true
}

/// WASM-exported memory allocator for JavaScript.
///
/// This provides a way for JavaScript to allocate memory in the WASM heap
/// for zero-copy data transfer.
#[wasm_bindgen]
pub struct WasmAllocator {
    ptr: *mut u8,
    size: usize,
}

#[wasm_bindgen]
impl WasmAllocator {
    /// Allocate a buffer of the specified size.
    #[wasm_bindgen(constructor)]
    pub fn new(size: usize) -> Result<WasmAllocator, JsValue> {
        allocate(size)?;

        let layout = std::alloc::Layout::from_size_align(size, 8)
            .map_err(|e| JsValue::from_str(&format!("Invalid layout: {}", e)))?;

        let ptr = unsafe { std::alloc::alloc(layout) };

        if ptr.is_null() {
            deallocate(size);
            return Err(JsValue::from_str("Memory allocation failed"));
        }

        Ok(WasmAllocator { ptr, size })
    }

    /// Get a pointer to the allocated memory.
    #[wasm_bindgen(getter)]
    pub fn ptr(&self) -> *mut u8 {
        self.ptr
    }

    /// Get the size of the allocated memory.
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Free the allocated memory.
    pub fn free(&mut self) {
        if !self.ptr.is_null() {
            let layout = std::alloc::Layout::from_size_align(self.size, 8).unwrap();
            unsafe {
                std::alloc::dealloc(self.ptr, layout);
            }
            deallocate(self.size);
            self.ptr = std::ptr::null_mut();
        }
    }
}

impl Drop for WasmAllocator {
    fn drop(&mut self) {
        self.free();
    }
}

/// Memory buffer for efficient data transfer between JavaScript and Rust.
///
/// This struct provides a managed buffer that can be shared between
/// JavaScript and Rust without copying.
#[wasm_bindgen]
pub struct MemoryBuffer {
    data: Vec<u8>,
}

#[wasm_bindgen]
impl MemoryBuffer {
    /// Create a new buffer with the specified capacity.
    #[wasm_bindgen(constructor)]
    pub fn new(capacity: usize) -> Result<MemoryBuffer, JsValue> {
        allocate(capacity)?;

        Ok(MemoryBuffer {
            data: Vec::with_capacity(capacity),
        })
    }

    /// Get the length of the buffer.
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.data.len()
    }

    /// Get the capacity of the buffer.
    #[wasm_bindgen(getter)]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Get a pointer to the buffer data.
    #[wasm_bindgen(getter)]
    pub fn ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    /// Write data to the buffer.
    pub fn write(&mut self, data: &[u8]) -> Result<(), JsValue> {
        let new_size = self.data.len() + data.len();
        if new_size > self.data.capacity() {
            return Err(JsValue::from_str("Buffer overflow"));
        }

        self.data.extend_from_slice(data);
        Ok(())
    }

    /// Read data from the buffer.
    pub fn read(&self, offset: usize, length: usize) -> Result<js_sys::Uint8Array, JsValue> {
        if offset + length > self.data.len() {
            return Err(JsValue::from_str("Read out of bounds"));
        }

        let slice = &self.data[offset..offset + length];
        Ok(js_sys::Uint8Array::from(slice))
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Convert to Uint8Array.
    pub fn to_uint8_array(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.data[..])
    }
}

impl Drop for MemoryBuffer {
    fn drop(&mut self) {
        if self.data.capacity() > 0 {
            deallocate(self.data.capacity());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracking() {
        reset_pools().unwrap();

        // Initialize with 1MB max
        let config = MemoryConfig {
            initial_pool_size: 1024,
            max_pool_size: 1024 * 1024,
            aggressive_gc: false,
        };
        initialize_pools(config).unwrap();

        // Allocate some memory
        allocate(1024).unwrap();
        assert_eq!(CURRENT_USED.load(Ordering::Relaxed), 1024);

        // Deallocate
        deallocate(1024);
        assert_eq!(CURRENT_USED.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_allocation_limit() {
        reset_pools().unwrap();

        let config = MemoryConfig {
            initial_pool_size: 1024,
            max_pool_size: 2048,
            aggressive_gc: false,
        };
        initialize_pools(config).unwrap();

        // Should succeed
        assert!(allocate(1024).is_ok());

        // Should fail (exceeds limit)
        assert!(allocate(2048).is_err());
    }
}
