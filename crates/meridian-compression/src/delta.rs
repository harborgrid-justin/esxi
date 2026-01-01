//! Delta compression for versioned data
//!
//! Efficient storage of incremental changes between versions.

use crate::error::{CompressionError, Result};
use crate::Compressor;
use crate::zstd::ZstdCompressor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Delta compression operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeltaOp {
    /// Copy bytes from source at offset
    Copy { offset: usize, length: usize },
    /// Insert new bytes
    Insert { data: Vec<u8> },
    /// Skip bytes in source
    Skip { length: usize },
}

/// Delta patch containing operations to transform source to target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaPatch {
    /// Source data hash for validation
    pub source_hash: u64,
    /// Target data hash
    pub target_hash: u64,
    /// Delta operations
    pub operations: Vec<DeltaOp>,
    /// Original source size
    pub source_size: usize,
    /// Expected target size
    pub target_size: usize,
}

impl DeltaPatch {
    /// Create a new delta patch
    pub fn new(
        source_hash: u64,
        target_hash: u64,
        operations: Vec<DeltaOp>,
        source_size: usize,
        target_size: usize,
    ) -> Self {
        Self {
            source_hash,
            target_hash,
            operations,
            source_size,
            target_size,
        }
    }

    /// Serialize patch to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(CompressionError::from)
    }

    /// Deserialize patch from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data).map_err(CompressionError::from)
    }

    /// Get estimated patch size
    pub fn estimated_size(&self) -> usize {
        self.operations.iter().map(|op| match op {
            DeltaOp::Copy { .. } => 16, // offset + length
            DeltaOp::Insert { data } => 8 + data.len(),
            DeltaOp::Skip { .. } => 8,
        }).sum()
    }
}

/// Delta compressor using simple diff algorithm
pub struct DeltaCompressor {
    /// Minimum match length for copy operations
    min_match_len: usize,
    /// Maximum search distance
    max_search_distance: usize,
    /// Enable additional compression of patch
    compress_patch: bool,
}

impl Default for DeltaCompressor {
    fn default() -> Self {
        Self {
            min_match_len: 8,
            max_search_distance: 64 * 1024, // 64 KB
            compress_patch: true,
        }
    }
}

impl DeltaCompressor {
    /// Create a new delta compressor
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom configuration
    pub fn with_config(
        min_match_len: usize,
        max_search_distance: usize,
        compress_patch: bool,
    ) -> Self {
        Self {
            min_match_len,
            max_search_distance,
            compress_patch,
        }
    }

    /// Create delta patch from source to target
    pub fn create_patch(&self, source: &[u8], target: &[u8]) -> Result<DeltaPatch> {
        let source_hash = Self::hash_data(source);
        let target_hash = Self::hash_data(target);

        let operations = self.compute_delta(source, target);

        let patch = DeltaPatch::new(
            source_hash,
            target_hash,
            operations,
            source.len(),
            target.len(),
        );

        Ok(patch)
    }

    /// Apply delta patch to source data
    pub fn apply_patch(&self, source: &[u8], patch: &DeltaPatch) -> Result<Vec<u8>> {
        // Validate source
        let source_hash = Self::hash_data(source);
        if source_hash != patch.source_hash {
            return Err(CompressionError::Delta(
                "Source hash mismatch".to_string(),
            ));
        }

        if source.len() != patch.source_size {
            return Err(CompressionError::Delta(
                format!(
                    "Source size mismatch: expected {}, got {}",
                    patch.source_size,
                    source.len()
                ),
            ));
        }

        let mut result = Vec::with_capacity(patch.target_size);

        for op in &patch.operations {
            match op {
                DeltaOp::Copy { offset, length } => {
                    if offset + length > source.len() {
                        return Err(CompressionError::Delta(
                            "Copy operation out of bounds".to_string(),
                        ));
                    }
                    result.extend_from_slice(&source[*offset..*offset + *length]);
                }
                DeltaOp::Insert { data } => {
                    result.extend_from_slice(data);
                }
                DeltaOp::Skip { .. } => {
                    // Skip is used during creation, not application
                }
            }
        }

        // Validate result
        let result_hash = Self::hash_data(&result);
        if result_hash != patch.target_hash {
            return Err(CompressionError::Delta(
                "Result hash mismatch".to_string(),
            ));
        }

        Ok(result)
    }

    /// Encode patch to compressed bytes
    pub fn encode_patch(&self, patch: &DeltaPatch) -> Result<Vec<u8>> {
        let patch_bytes = patch.to_bytes()?;

        if self.compress_patch {
            let compressor = ZstdCompressor::with_level(3);
            compressor.compress(&patch_bytes)
        } else {
            Ok(patch_bytes)
        }
    }

    /// Decode patch from compressed bytes
    pub fn decode_patch(&self, data: &[u8]) -> Result<DeltaPatch> {
        let patch_bytes = if self.compress_patch {
            let compressor = ZstdCompressor::with_level(3);
            compressor.decompress(data)?
        } else {
            data.to_vec()
        };

        DeltaPatch::from_bytes(&patch_bytes)
    }

    /// Compute delta operations
    fn compute_delta(&self, source: &[u8], target: &[u8]) -> Vec<DeltaOp> {
        let mut operations = Vec::new();
        let mut target_pos = 0;
        let mut pending_insert = Vec::new();

        while target_pos < target.len() {
            // Try to find a match in source
            if let Some((match_offset, match_len)) = self.find_match(source, target, target_pos) {
                // Flush pending inserts
                if !pending_insert.is_empty() {
                    operations.push(DeltaOp::Insert {
                        data: pending_insert.clone(),
                    });
                    pending_insert.clear();
                }

                // Add copy operation
                operations.push(DeltaOp::Copy {
                    offset: match_offset,
                    length: match_len,
                });

                target_pos += match_len;
            } else {
                // No match found, accumulate for insert
                pending_insert.push(target[target_pos]);
                target_pos += 1;
            }
        }

        // Flush remaining inserts
        if !pending_insert.is_empty() {
            operations.push(DeltaOp::Insert {
                data: pending_insert,
            });
        }

        operations
    }

    /// Find best match in source for target at position
    fn find_match(&self, source: &[u8], target: &[u8], target_pos: usize) -> Option<(usize, usize)> {
        let remaining = target.len() - target_pos;
        if remaining < self.min_match_len {
            return None;
        }

        let mut best_match: Option<(usize, usize)> = None;
        let mut best_len = 0;

        // Simple brute force search (could be optimized with rolling hash)
        for source_pos in 0..source.len() {
            let max_len = (source.len() - source_pos).min(remaining);
            if max_len < self.min_match_len || max_len <= best_len {
                continue;
            }

            let match_len = self.match_length(source, source_pos, target, target_pos, max_len);

            if match_len >= self.min_match_len && match_len > best_len {
                best_len = match_len;
                best_match = Some((source_pos, match_len));
            }
        }

        best_match
    }

    /// Calculate match length between source and target
    fn match_length(
        &self,
        source: &[u8],
        source_pos: usize,
        target: &[u8],
        target_pos: usize,
        max_len: usize,
    ) -> usize {
        let mut len = 0;
        while len < max_len && source[source_pos + len] == target[target_pos + len] {
            len += 1;
        }
        len
    }

    /// Hash data using simple FNV-1a hash
    fn hash_data(data: &[u8]) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
}

/// Version chain manager for delta compression
pub struct VersionChain {
    /// Base version data
    base: Vec<u8>,
    /// Delta patches for each version
    deltas: Vec<DeltaPatch>,
    /// Compressor instance
    compressor: DeltaCompressor,
}

impl VersionChain {
    /// Create a new version chain with base version
    pub fn new(base: Vec<u8>) -> Self {
        Self {
            base,
            deltas: Vec::new(),
            compressor: DeltaCompressor::new(),
        }
    }

    /// Add a new version to the chain
    pub fn add_version(&mut self, new_version: &[u8]) -> Result<usize> {
        let previous = if self.deltas.is_empty() {
            &self.base
        } else {
            // Reconstruct previous version
            &self.get_version(self.deltas.len() - 1)?
        };

        let patch = self.compressor.create_patch(previous, new_version)?;
        self.deltas.push(patch);

        Ok(self.deltas.len())
    }

    /// Get specific version by index
    pub fn get_version(&self, version: usize) -> Result<Vec<u8>> {
        if version == 0 {
            return Ok(self.base.clone());
        }

        if version > self.deltas.len() {
            return Err(CompressionError::Delta(
                format!("Version {} does not exist", version),
            ));
        }

        let mut current = self.base.clone();

        for i in 0..version {
            current = self.compressor.apply_patch(&current, &self.deltas[i])?;
        }

        Ok(current)
    }

    /// Get latest version
    pub fn get_latest(&self) -> Result<Vec<u8>> {
        self.get_version(self.deltas.len())
    }

    /// Get version count
    pub fn version_count(&self) -> usize {
        self.deltas.len() + 1 // +1 for base version
    }

    /// Get total storage size
    pub fn total_size(&self) -> usize {
        let base_size = self.base.len();
        let deltas_size: usize = self.deltas.iter()
            .map(|d| d.estimated_size())
            .sum();
        base_size + deltas_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_patch() {
        let compressor = DeltaCompressor::new();

        let source = b"Hello, World!";
        let target = b"Hello, Rust World!";

        let patch = compressor.create_patch(source, target).unwrap();
        let result = compressor.apply_patch(source, &patch).unwrap();

        assert_eq!(target.as_slice(), result.as_slice());
    }

    #[test]
    fn test_delta_identical() {
        let compressor = DeltaCompressor::new();

        let source = b"Identical data";
        let target = b"Identical data";

        let patch = compressor.create_patch(source, target).unwrap();
        assert_eq!(patch.operations.len(), 1); // Should be one copy operation
    }

    #[test]
    fn test_delta_completely_different() {
        let compressor = DeltaCompressor::new();

        let source = b"AAAAAAAAAA";
        let target = b"BBBBBBBBBB";

        let patch = compressor.create_patch(source, target).unwrap();
        let result = compressor.apply_patch(source, &patch).unwrap();

        assert_eq!(target.as_slice(), result.as_slice());
    }

    #[test]
    fn test_patch_serialization() {
        let compressor = DeltaCompressor::new();

        let source = b"Source data";
        let target = b"Target data";

        let patch = compressor.create_patch(source, target).unwrap();
        let encoded = compressor.encode_patch(&patch).unwrap();
        let decoded = compressor.decode_patch(&encoded).unwrap();

        let result = compressor.apply_patch(source, &decoded).unwrap();
        assert_eq!(target.as_slice(), result.as_slice());
    }

    #[test]
    fn test_version_chain() {
        let mut chain = VersionChain::new(b"Version 1".to_vec());

        chain.add_version(b"Version 2").unwrap();
        chain.add_version(b"Version 3").unwrap();

        assert_eq!(chain.version_count(), 3);

        let v1 = chain.get_version(0).unwrap();
        let v2 = chain.get_version(1).unwrap();
        let v3 = chain.get_version(2).unwrap();

        assert_eq!(v1, b"Version 1");
        assert_eq!(v2, b"Version 2");
        assert_eq!(v3, b"Version 3");
    }

    #[test]
    fn test_large_delta() {
        let compressor = DeltaCompressor::new();

        let source = b"x".repeat(10000);
        let mut target = source.clone();
        target.extend_from_slice(b"ADDED");

        let patch = compressor.create_patch(&source, &target).unwrap();
        let result = compressor.apply_patch(&source, &patch).unwrap();

        assert_eq!(target, result);
        // Patch should be much smaller than full data
        assert!(patch.estimated_size() < target.len() / 10);
    }

    #[test]
    fn test_invalid_source() {
        let compressor = DeltaCompressor::new();

        let source = b"Original";
        let target = b"Modified";

        let patch = compressor.create_patch(source, target).unwrap();

        let wrong_source = b"WrongData";
        let result = compressor.apply_patch(wrong_source, &patch);

        assert!(result.is_err());
    }
}
