//! Zstandard compression with dictionary training
//!
//! High-performance compression with excellent ratio and dictionary support.

use crate::error::{CompressionError, Result};
use crate::stats::{CompressionStats, Timer};
use crate::Compressor;
use async_trait::async_trait;
use bytes::Bytes;
use std::io::{Read, Write};
use std::sync::Arc;
use parking_lot::RwLock;

/// Zstandard compression configuration
#[derive(Debug, Clone)]
pub struct ZstdConfig {
    /// Compression level (1-22)
    pub level: i32,
    /// Enable multithreading
    pub workers: u32,
    /// Window log size
    pub window_log: Option<u32>,
    /// Enable long-range mode
    pub long_range_mode: bool,
    /// Enable checksum
    pub enable_checksum: bool,
}

impl Default for ZstdConfig {
    fn default() -> Self {
        Self {
            level: 3,
            workers: 0, // 0 = auto-detect
            window_log: None,
            long_range_mode: false,
            enable_checksum: true,
        }
    }
}

/// Zstandard compressor with dictionary support
pub struct ZstdCompressor {
    config: ZstdConfig,
    dictionary: Option<Arc<Vec<u8>>>,
}

impl Default for ZstdCompressor {
    fn default() -> Self {
        Self {
            config: ZstdConfig::default(),
            dictionary: None,
        }
    }
}

impl ZstdCompressor {
    /// Create a new Zstandard compressor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom configuration
    pub fn with_config(config: ZstdConfig) -> Self {
        Self {
            config,
            dictionary: None,
        }
    }

    /// Create with specific compression level
    pub fn with_level(level: i32) -> Self {
        Self {
            config: ZstdConfig {
                level,
                ..Default::default()
            },
            dictionary: None,
        }
    }

    /// Set dictionary for compression
    pub fn with_dictionary(mut self, dictionary: Vec<u8>) -> Self {
        self.dictionary = Some(Arc::new(dictionary));
        self
    }

    /// Train a dictionary from sample data
    pub fn train_dictionary(samples: &[Vec<u8>], dict_size: usize) -> Result<Vec<u8>> {
        if samples.is_empty() {
            return Err(CompressionError::DictionaryTraining(
                "No samples provided for training".to_string(),
            ));
        }

        // Collect all samples into a single buffer
        let total_size: usize = samples.iter().map(|s| s.len()).sum();
        let mut all_samples = Vec::with_capacity(total_size);

        for sample in samples {
            all_samples.extend_from_slice(sample);
        }

        // Train dictionary using zstd
        let dictionary = zstd::dict::from_continuous(&all_samples, &[], dict_size)
            .map_err(|e| {
                CompressionError::DictionaryTraining(format!(
                    "Dictionary training failed: {}",
                    e
                ))
            })?;

        Ok(dictionary)
    }

    /// Compress with statistics
    pub fn compress_with_stats(&self, data: &[u8]) -> Result<(Vec<u8>, CompressionStats)> {
        let timer = Timer::start("zstd_compress");
        let compressed = self.compress(data)?;
        let elapsed = timer.stop();

        let stats = CompressionStats::new(
            data.len(),
            compressed.len(),
            elapsed,
            "zstd",
        )
        .with_level(self.config.level);

        Ok((compressed, stats))
    }

    /// Stream compress with Zstandard
    pub fn compress_stream<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<usize> {
        let mut encoder = zstd::Encoder::new(writer, self.config.level)
            .map_err(|e| CompressionError::zstd(e.to_string()))?;

        if let Some(dict) = &self.dictionary {
            encoder
                .set_dictionary(dict.as_ref())
                .map_err(|e| CompressionError::zstd(e.to_string()))?;
        }

        if self.config.workers > 0 {
            encoder
                .multithread(self.config.workers)
                .map_err(|e| CompressionError::zstd(e.to_string()))?;
        }

        if self.config.long_range_mode {
            encoder
                .long_distance_matching(true)
                .map_err(|e| CompressionError::zstd(e.to_string()))?;
        }

        let bytes_written = std::io::copy(reader, &mut encoder)
            .map_err(|e| CompressionError::zstd(e.to_string()))?;

        encoder
            .finish()
            .map_err(|e| CompressionError::zstd(e.to_string()))?;

        Ok(bytes_written as usize)
    }

    /// Decompress stream
    pub fn decompress_stream<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<usize> {
        let mut decoder = if let Some(dict) = &self.dictionary {
            zstd::Decoder::with_dictionary(reader, dict.as_ref())
                .map_err(|e| CompressionError::zstd(e.to_string()))?
        } else {
            zstd::Decoder::new(reader).map_err(|e| CompressionError::zstd(e.to_string()))?
        };

        let bytes_written = std::io::copy(&mut decoder, writer)
            .map_err(|e| CompressionError::zstd(e.to_string()))?;

        Ok(bytes_written as usize)
    }
}

#[async_trait]
impl Compressor for ZstdCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let compressed = if let Some(dict) = &self.dictionary {
            zstd::bulk::compress_using_dict(data, dict.as_ref(), self.config.level)
                .map_err(|e| CompressionError::zstd(e.to_string()))?
        } else {
            zstd::bulk::compress(data, self.config.level)
                .map_err(|e| CompressionError::zstd(e.to_string()))?
        };

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let decompressed = if let Some(dict) = &self.dictionary {
            zstd::bulk::decompress_using_dict(data, dict.as_ref())
                .map_err(|e| CompressionError::zstd(e.to_string()))?
        } else {
            zstd::bulk::decompress(data, 128 * 1024 * 1024) // Max 128 MB
                .map_err(|e| CompressionError::zstd(e.to_string()))?
        };

        Ok(decompressed)
    }

    fn algorithm(&self) -> &str {
        "zstd"
    }

    fn level(&self) -> Option<i32> {
        Some(self.config.level)
    }
}

/// Dictionary manager for Zstandard compression
pub struct ZstdDictionaryManager {
    dictionaries: Arc<RwLock<std::collections::HashMap<String, Arc<Vec<u8>>>>>,
}

impl Default for ZstdDictionaryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ZstdDictionaryManager {
    /// Create a new dictionary manager
    pub fn new() -> Self {
        Self {
            dictionaries: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Store a trained dictionary
    pub fn store_dictionary(&self, id: String, dictionary: Vec<u8>) {
        let mut dicts = self.dictionaries.write();
        dicts.insert(id, Arc::new(dictionary));
    }

    /// Get a dictionary by ID
    pub fn get_dictionary(&self, id: &str) -> Option<Arc<Vec<u8>>> {
        let dicts = self.dictionaries.read();
        dicts.get(id).cloned()
    }

    /// Remove a dictionary
    pub fn remove_dictionary(&self, id: &str) -> bool {
        let mut dicts = self.dictionaries.write();
        dicts.remove(id).is_some()
    }

    /// List all dictionary IDs
    pub fn list_dictionaries(&self) -> Vec<String> {
        let dicts = self.dictionaries.read();
        dicts.keys().cloned().collect()
    }

    /// Get dictionary count
    pub fn count(&self) -> usize {
        let dicts = self.dictionaries.read();
        dicts.len()
    }

    /// Clear all dictionaries
    pub fn clear(&self) {
        let mut dicts = self.dictionaries.write();
        dicts.clear();
    }

    /// Train and store a new dictionary
    pub fn train_and_store(
        &self,
        id: String,
        samples: &[Vec<u8>],
        dict_size: usize,
    ) -> Result<()> {
        let dictionary = ZstdCompressor::train_dictionary(samples, dict_size)?;
        self.store_dictionary(id, dictionary);
        Ok(())
    }
}

/// Parallel Zstandard compressor using multiple threads
pub struct ParallelZstdCompressor {
    level: i32,
    workers: u32,
}

impl ParallelZstdCompressor {
    /// Create a new parallel compressor
    pub fn new(workers: u32) -> Self {
        Self {
            level: 3,
            workers,
        }
    }

    /// Create with specific level and workers
    pub fn with_level(level: i32, workers: u32) -> Self {
        Self { level, workers }
    }
}

#[async_trait]
impl Compressor for ParallelZstdCompressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut compressed = Vec::new();

        {
            let mut encoder = zstd::Encoder::new(&mut compressed, self.level)
                .map_err(|e| CompressionError::zstd(e.to_string()))?;

            encoder
                .multithread(self.workers)
                .map_err(|e| CompressionError::zstd(e.to_string()))?;

            encoder
                .write_all(data)
                .map_err(|e| CompressionError::zstd(e.to_string()))?;

            encoder
                .finish()
                .map_err(|e| CompressionError::zstd(e.to_string()))?;
        }

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        zstd::bulk::decompress(data, 128 * 1024 * 1024)
            .map_err(|e| CompressionError::zstd(e.to_string()))
    }

    fn algorithm(&self) -> &str {
        "zstd_parallel"
    }

    fn level(&self) -> Option<i32> {
        Some(self.level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zstd_compress_decompress() {
        let compressor = ZstdCompressor::new();
        let data = b"Hello, Zstandard compression!";

        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_zstd_levels() {
        let data = b"Test data" as &[u8];
        let data = data.repeat(1000);

        let low = ZstdCompressor::with_level(1);
        let high = ZstdCompressor::with_level(19);

        let compressed_low = low.compress(&data).unwrap();
        let compressed_high = high.compress(&data).unwrap();

        assert!(compressed_high.len() <= compressed_low.len());
    }

    #[test]
    fn test_dictionary_training() {
        let samples = vec![
            b"Hello, World!".to_vec(),
            b"Hello, Zstandard!".to_vec(),
            b"Hello, Compression!".to_vec(),
        ];

        let dictionary = ZstdCompressor::train_dictionary(&samples, 1024).unwrap();
        assert!(!dictionary.is_empty());
    }

    #[test]
    fn test_compression_with_dictionary() {
        let samples = vec![
            b"Enterprise SaaS Platform v0.5".to_vec(),
            b"Enterprise SaaS Platform v0.4".to_vec(),
            b"Enterprise SaaS Platform v0.3".to_vec(),
        ];

        let dictionary = ZstdCompressor::train_dictionary(&samples, 1024).unwrap();
        let compressor = ZstdCompressor::new().with_dictionary(dictionary);

        let data = b"Enterprise SaaS Platform v0.6";
        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data.as_slice(), decompressed.as_slice());
    }

    #[test]
    fn test_dictionary_manager() {
        let manager = ZstdDictionaryManager::new();

        let samples = vec![b"test data".to_vec()];
        manager
            .train_and_store("test_dict".to_string(), &samples, 512)
            .unwrap();

        assert_eq!(manager.count(), 1);
        assert!(manager.get_dictionary("test_dict").is_some());

        let dicts = manager.list_dictionaries();
        assert_eq!(dicts.len(), 1);
    }

    #[tokio::test]
    async fn test_parallel_compression() {
        let compressor = ParallelZstdCompressor::new(4);
        let data = b"Parallel compression test" as &[u8];
        let data = data.repeat(100);

        let compressed = compressor.compress(&data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(data, decompressed);
    }
}
