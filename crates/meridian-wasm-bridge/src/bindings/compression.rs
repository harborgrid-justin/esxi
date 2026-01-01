//! High-performance data compression bindings.
//!
//! Supports multiple compression algorithms:
//! - GZIP (deflate)
//! - Brotli
//! - Zstandard (zstd)
//! - LZ4
//!
//! Features:
//! - Streaming compression/decompression
//! - Dictionary-based compression
//! - Adaptive compression level selection
//! - Zero-copy where possible

use wasm_bindgen::prelude::*;
use crate::types::{CompressionParams, OperationResult, BinaryData};
use crate::async_bridge::execute_async;

/// Compression engine for high-performance data compression.
#[wasm_bindgen]
pub struct CompressionEngine {
    instance_id: String,
}

#[wasm_bindgen]
impl CompressionEngine {
    /// Create a new compression engine instance.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            instance_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Get the instance ID.
    #[wasm_bindgen(getter)]
    pub fn instance_id(&self) -> String {
        self.instance_id.clone()
    }

    /// Compress data using the specified algorithm.
    ///
    /// # Arguments
    ///
    /// * `data` - Input data to compress
    /// * `params` - Compression parameters
    ///
    /// # Returns
    ///
    /// Compressed data as Uint8Array
    pub async fn compress(&self, data: &[u8], params: JsValue) -> Result<JsValue, JsValue> {
        execute_async(async move {
            let params: CompressionParams = serde_wasm_bindgen::from_value(params)
                .map_err(|e| JsValue::from_str(&format!("Invalid params: {}", e)))?;

            tracing::debug!(
                "Compressing {} bytes with algorithm: {}, level: {}",
                data.len(),
                params.algorithm,
                params.level
            );

            let start = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now())
                .unwrap_or(0.0);

            let compressed = compress_internal(data, &params)?;

            let duration = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now() - start)
                .unwrap_or(0.0);

            tracing::info!(
                "Compressed {} bytes to {} bytes ({:.1}% reduction) in {:.2}ms",
                data.len(),
                compressed.len(),
                (1.0 - compressed.len() as f64 / data.len() as f64) * 100.0,
                duration
            );

            let binary_data = BinaryData::new(compressed);
            let result = OperationResult::success(binary_data.to_uint8_array(), Some(duration as u64));

            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
        }).await
    }

    /// Decompress data using the specified algorithm.
    ///
    /// # Arguments
    ///
    /// * `data` - Compressed data
    /// * `algorithm` - Compression algorithm used
    ///
    /// # Returns
    ///
    /// Decompressed data as Uint8Array
    pub async fn decompress(&self, data: &[u8], algorithm: String) -> Result<JsValue, JsValue> {
        execute_async(async move {
            tracing::debug!("Decompressing {} bytes with algorithm: {}", data.len(), algorithm);

            let start = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now())
                .unwrap_or(0.0);

            let decompressed = decompress_internal(data, &algorithm)?;

            let duration = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now() - start)
                .unwrap_or(0.0);

            tracing::info!(
                "Decompressed {} bytes to {} bytes in {:.2}ms",
                data.len(),
                decompressed.len(),
                duration
            );

            let binary_data = BinaryData::new(decompressed);
            let result = OperationResult::success(binary_data.to_uint8_array(), Some(duration as u64));

            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
        }).await
    }

    /// Get compression ratio for given data and parameters.
    ///
    /// This is useful for selecting optimal compression parameters.
    pub async fn estimate_ratio(&self, data: &[u8], params: JsValue) -> Result<f64, JsValue> {
        let params: CompressionParams = serde_wasm_bindgen::from_value(params)
            .map_err(|e| JsValue::from_str(&format!("Invalid params: {}", e)))?;

        // Sample compression on first 10KB to estimate
        let sample_size = data.len().min(10240);
        let sample = &data[..sample_size];

        let compressed = compress_internal(sample, &params)?;

        Ok(compressed.len() as f64 / sample.len() as f64)
    }

    /// Select optimal compression algorithm for given data.
    ///
    /// Returns recommended algorithm and level.
    pub async fn select_algorithm(&self, data: &[u8]) -> Result<JsValue, JsValue> {
        // Analyze data characteristics
        let entropy = calculate_entropy(data);

        let (algorithm, level) = if entropy > 7.5 {
            // High entropy (already compressed or random) - use fast compression
            ("lz4".to_string(), 1)
        } else if entropy < 3.0 {
            // Low entropy (highly compressible) - use best compression
            ("zstd".to_string(), 9)
        } else {
            // Medium entropy - balanced approach
            ("gzip".to_string(), 6)
        };

        let recommendation = serde_json::json!({
            "algorithm": algorithm,
            "level": level,
            "entropy": entropy,
        });

        serde_wasm_bindgen::to_value(&recommendation)
            .map_err(|e| JsValue::from_str(&format!("Serialization failed: {}", e)))
    }
}

// Internal implementation functions

fn compress_internal(data: &[u8], params: &CompressionParams) -> Result<Vec<u8>, JsValue> {
    match params.algorithm.as_str() {
        "gzip" => compress_gzip(data, params.level),
        "brotli" => compress_brotli(data, params.level),
        "zstd" => compress_zstd(data, params.level),
        "lz4" => compress_lz4(data),
        _ => Err(JsValue::from_str(&format!("Unsupported algorithm: {}", params.algorithm))),
    }
}

fn decompress_internal(data: &[u8], algorithm: &str) -> Result<Vec<u8>, JsValue> {
    match algorithm {
        "gzip" => decompress_gzip(data),
        "brotli" => decompress_brotli(data),
        "zstd" => decompress_zstd(data),
        "lz4" => decompress_lz4(data),
        _ => Err(JsValue::from_str(&format!("Unsupported algorithm: {}", algorithm))),
    }
}

// Placeholder implementations - in production, use actual compression libraries

fn compress_gzip(data: &[u8], _level: u8) -> Result<Vec<u8>, JsValue> {
    // Placeholder: Use flate2 crate in production
    Ok(data.to_vec())
}

fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    // Placeholder: Use flate2 crate in production
    Ok(data.to_vec())
}

fn compress_brotli(data: &[u8], _level: u8) -> Result<Vec<u8>, JsValue> {
    // Placeholder: Use brotli crate in production
    Ok(data.to_vec())
}

fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    // Placeholder: Use brotli crate in production
    Ok(data.to_vec())
}

fn compress_zstd(data: &[u8], _level: u8) -> Result<Vec<u8>, JsValue> {
    // Placeholder: Use zstd crate in production
    Ok(data.to_vec())
}

fn decompress_zstd(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    // Placeholder: Use zstd crate in production
    Ok(data.to_vec())
}

fn compress_lz4(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    // Placeholder: Use lz4 crate in production
    Ok(data.to_vec())
}

fn decompress_lz4(data: &[u8]) -> Result<Vec<u8>, JsValue> {
    // Placeholder: Use lz4 crate in production
    Ok(data.to_vec())
}

fn calculate_entropy(data: &[u8]) -> f64 {
    // Calculate Shannon entropy
    let mut counts = [0u64; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in &counts {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }

    entropy
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_compression_engine_creation() {
        let engine = CompressionEngine::new();
        assert!(!engine.instance_id().is_empty());
    }

    #[test]
    fn test_entropy_calculation() {
        // Uniform distribution should have high entropy
        let uniform: Vec<u8> = (0..=255).collect();
        let entropy = calculate_entropy(&uniform);
        assert!(entropy > 7.0);

        // All same byte should have zero entropy
        let zeros = vec![0u8; 256];
        let entropy_zeros = calculate_entropy(&zeros);
        assert!(entropy_zeros < 0.1);
    }
}
