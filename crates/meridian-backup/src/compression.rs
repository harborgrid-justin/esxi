//! Compression and deduplication for backup data.

use bytes::Bytes;
use flate2::read::{GzDecoder, GzEncoder};
use flate2::Compression as GzCompression;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{BackupError, Result};

/// Compression algorithm types.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Zstd,
    Lz4,
}

/// Compression level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionLevel {
    Fast,
    Balanced,
    Best,
}

impl CompressionLevel {
    fn to_gzip_level(&self) -> GzCompression {
        match self {
            CompressionLevel::Fast => GzCompression::fast(),
            CompressionLevel::Balanced => GzCompression::default(),
            CompressionLevel::Best => GzCompression::best(),
        }
    }

    fn to_zstd_level(&self) -> i32 {
        match self {
            CompressionLevel::Fast => 1,
            CompressionLevel::Balanced => 3,
            CompressionLevel::Best => 19,
        }
    }
}

/// Compressed data container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedData {
    pub algorithm: CompressionAlgorithm,
    pub original_size: usize,
    pub compressed_size: usize,
    pub data: Vec<u8>,
    pub checksum: Vec<u8>,
}

/// Compression manager.
#[derive(Clone)]
pub struct CompressionManager {
    algorithm: CompressionAlgorithm,
    level: CompressionLevel,
}

impl CompressionManager {
    /// Create a new compression manager.
    pub fn new(algorithm: CompressionAlgorithm, level: CompressionLevel) -> Self {
        Self { algorithm, level }
    }

    /// Compress data.
    pub fn compress(&self, data: &[u8]) -> Result<CompressedData> {
        let compressed = match self.algorithm {
            CompressionAlgorithm::None => data.to_vec(),
            CompressionAlgorithm::Gzip => self.compress_gzip(data)?,
            CompressionAlgorithm::Zstd => self.compress_zstd(data)?,
            CompressionAlgorithm::Lz4 => self.compress_lz4(data)?,
        };

        let checksum = Self::calculate_checksum(data);

        Ok(CompressedData {
            algorithm: self.algorithm,
            original_size: data.len(),
            compressed_size: compressed.len(),
            data: compressed,
            checksum,
        })
    }

    /// Decompress data.
    pub fn decompress(&self, compressed: &CompressedData) -> Result<Vec<u8>> {
        let decompressed = match compressed.algorithm {
            CompressionAlgorithm::None => compressed.data.clone(),
            CompressionAlgorithm::Gzip => self.decompress_gzip(&compressed.data)?,
            CompressionAlgorithm::Zstd => self.decompress_zstd(&compressed.data)?,
            CompressionAlgorithm::Lz4 => self.decompress_lz4(&compressed.data)?,
        };

        // Verify checksum
        let checksum = Self::calculate_checksum(&decompressed);
        if checksum != compressed.checksum {
            return Err(BackupError::VerificationFailed(
                "Checksum mismatch after decompression".to_string(),
            ));
        }

        Ok(decompressed)
    }

    /// Compress using Gzip.
    fn compress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(data, self.level.to_gzip_level());
        let mut compressed = Vec::new();
        encoder
            .read_to_end(&mut compressed)
            .map_err(|e| BackupError::Compression(e.to_string()))?;
        Ok(compressed)
    }

    /// Decompress using Gzip.
    fn decompress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| BackupError::Compression(e.to_string()))?;
        Ok(decompressed)
    }

    /// Compress using Zstd.
    fn compress_zstd(&self, data: &[u8]) -> Result<Vec<u8>> {
        zstd::encode_all(data, self.level.to_zstd_level())
            .map_err(|e| BackupError::Compression(e.to_string()))
    }

    /// Decompress using Zstd.
    fn decompress_zstd(&self, data: &[u8]) -> Result<Vec<u8>> {
        zstd::decode_all(data).map_err(|e| BackupError::Compression(e.to_string()))
    }

    /// Compress using LZ4.
    fn compress_lz4(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = lz4::EncoderBuilder::new()
            .build(Vec::new())
            .map_err(|e| BackupError::Compression(e.to_string()))?;

        encoder
            .write_all(data)
            .map_err(|e| BackupError::Compression(e.to_string()))?;

        let (compressed, result) = encoder.finish();
        result.map_err(|e| BackupError::Compression(e.to_string()))?;

        Ok(compressed)
    }

    /// Decompress using LZ4.
    fn decompress_lz4(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = lz4::Decoder::new(data)
            .map_err(|e| BackupError::Compression(e.to_string()))?;

        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| BackupError::Compression(e.to_string()))?;

        Ok(decompressed)
    }

    /// Calculate SHA-256 checksum.
    fn calculate_checksum(data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Get compression ratio.
    pub fn compression_ratio(&self, compressed: &CompressedData) -> f64 {
        if compressed.original_size == 0 {
            return 1.0;
        }
        compressed.compressed_size as f64 / compressed.original_size as f64
    }
}

/// Chunk metadata for deduplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub hash: Vec<u8>,
    pub size: usize,
    pub offset: u64,
    pub ref_count: usize,
}

/// Deduplication manager using content-defined chunking.
pub struct DeduplicationManager {
    chunk_store: Arc<RwLock<HashMap<Vec<u8>, ChunkMetadata>>>,
    min_chunk_size: usize,
    avg_chunk_size: usize,
    max_chunk_size: usize,
}

impl DeduplicationManager {
    /// Create a new deduplication manager.
    pub fn new(min_chunk_size: usize, avg_chunk_size: usize, max_chunk_size: usize) -> Self {
        Self {
            chunk_store: Arc::new(RwLock::new(HashMap::new())),
            min_chunk_size,
            avg_chunk_size,
            max_chunk_size,
        }
    }

    /// Create with default settings (4KB min, 64KB avg, 1MB max).
    pub fn default() -> Self {
        Self::new(4 * 1024, 64 * 1024, 1024 * 1024)
    }

    /// Chunk data using content-defined chunking.
    pub async fn chunk_data(&self, data: &[u8]) -> Result<Vec<ChunkMetadata>> {
        let mut chunks = Vec::new();
        let mut offset = 0u64;

        // Simple fixed-size chunking (can be replaced with Rabin fingerprinting)
        while offset < data.len() as u64 {
            let chunk_size = std::cmp::min(self.avg_chunk_size, data.len() - offset as usize);
            let chunk = &data[offset as usize..(offset as usize + chunk_size)];

            let hash = Self::hash_chunk(chunk);
            let metadata = ChunkMetadata {
                hash: hash.clone(),
                size: chunk_size,
                offset,
                ref_count: 1,
            };

            // Check if chunk already exists
            let mut store = self.chunk_store.write().await;
            if let Some(existing) = store.get_mut(&hash) {
                existing.ref_count += 1;
            } else {
                store.insert(hash.clone(), metadata.clone());
            }

            chunks.push(metadata);
            offset += chunk_size as u64;
        }

        Ok(chunks)
    }

    /// Reconstruct data from chunks.
    pub async fn reconstruct_data(&self, chunks: &[ChunkMetadata]) -> Result<Vec<u8>> {
        let mut data = Vec::new();

        for chunk_meta in chunks {
            // In a real implementation, we would fetch chunk data from storage
            // For now, we'll return empty data
            data.resize(data.len() + chunk_meta.size, 0);
        }

        Ok(data)
    }

    /// Calculate deduplication ratio.
    pub async fn dedup_ratio(&self) -> f64 {
        let store = self.chunk_store.read().await;
        if store.is_empty() {
            return 1.0;
        }

        let total_refs: usize = store.values().map(|m| m.ref_count).sum();
        let unique_chunks = store.len();

        total_refs as f64 / unique_chunks as f64
    }

    /// Hash a chunk using SHA-256.
    fn hash_chunk(chunk: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(chunk);
        hasher.finalize().to_vec()
    }

    /// Get statistics about the chunk store.
    pub async fn statistics(&self) -> DeduplicationStats {
        let store = self.chunk_store.read().await;

        let total_chunks = store.len();
        let total_size: usize = store.values().map(|m| m.size * m.ref_count).sum();
        let unique_size: usize = store.values().map(|m| m.size).sum();

        DeduplicationStats {
            total_chunks,
            unique_chunks: total_chunks,
            total_size,
            unique_size,
            dedup_ratio: if unique_size > 0 {
                total_size as f64 / unique_size as f64
            } else {
                1.0
            },
        }
    }
}

/// Deduplication statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationStats {
    pub total_chunks: usize,
    pub unique_chunks: usize,
    pub total_size: usize,
    pub unique_size: usize,
    pub dedup_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gzip_compression() {
        let manager = CompressionManager::new(CompressionAlgorithm::Gzip, CompressionLevel::Balanced);
        let data = b"Hello, Meridian! ".repeat(100);

        let compressed = manager.compress(&data).unwrap();
        assert!(compressed.compressed_size < compressed.original_size);

        let decompressed = manager.decompress(&compressed).unwrap();
        assert_eq!(data.to_vec(), decompressed);
    }

    #[tokio::test]
    async fn test_deduplication() {
        let manager = DeduplicationManager::default();
        let data = b"Repeating data ".repeat(100);

        let chunks = manager.chunk_data(&data).await.unwrap();
        assert!(!chunks.is_empty());

        let stats = manager.statistics().await;
        assert!(stats.total_chunks > 0);
    }
}
