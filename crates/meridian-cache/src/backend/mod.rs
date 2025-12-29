//! Cache backend implementations

use async_trait::async_trait;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::CacheResult;
use crate::stats::CacheStats;

pub mod disk;
pub mod memory;
pub mod redis;

/// Metadata associated with cached entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Time-to-live for the entry
    pub ttl: Option<Duration>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last access timestamp
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Access count
    pub access_count: u64,
    /// Size in bytes
    pub size: usize,
    /// Custom tags for grouping
    pub tags: Vec<String>,
}

impl Default for CacheMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            ttl: None,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            size: 0,
            tags: Vec::new(),
        }
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// The cached value
    #[serde(with = "serde_bytes")]
    pub value: Bytes,
    /// Entry metadata
    pub metadata: CacheMetadata,
}

mod serde_bytes {
    use bytes::Bytes;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        bytes.as_ref().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<u8> = Vec::deserialize(deserializer)?;
        Ok(Bytes::from(vec))
    }
}

/// Options for cache operations
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct CacheOptions {
    /// Time-to-live
    pub ttl: Option<Duration>,
    /// Tags for grouping
    pub tags: Vec<String>,
    /// Compression enabled
    pub compress: bool,
}


/// Write policy for cache backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritePolicy {
    /// Write through - write to cache and backing store synchronously
    WriteThrough,
    /// Write back - write to cache, async write to backing store
    WriteBack,
    /// Write around - skip cache, write directly to backing store
    WriteAround,
}

/// Trait defining cache backend operations
#[async_trait]
pub trait CacheBackend: Send + Sync {
    /// Get a value from the cache
    async fn get(&self, key: &str) -> CacheResult<Option<CacheEntry>>;

    /// Set a value in the cache
    async fn set(&self, key: &str, value: Bytes, options: CacheOptions) -> CacheResult<()>;

    /// Delete a value from the cache
    async fn delete(&self, key: &str) -> CacheResult<bool>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> CacheResult<bool>;

    /// Clear all entries
    async fn clear(&self) -> CacheResult<()>;

    /// Get cache statistics
    async fn stats(&self) -> CacheResult<CacheStats>;

    /// Get all keys matching a pattern
    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>>;

    /// Delete keys matching a pattern
    async fn delete_pattern(&self, pattern: &str) -> CacheResult<usize>;

    /// Delete keys with specific tags
    async fn delete_by_tags(&self, tags: &[String]) -> CacheResult<usize>;

    /// Get the size of the cache in bytes
    async fn size(&self) -> CacheResult<usize>;

    /// Get the number of entries in the cache
    async fn len(&self) -> CacheResult<usize>;

    /// Check if the cache is empty
    async fn is_empty(&self) -> CacheResult<bool> {
        Ok(self.len().await? == 0)
    }

    /// Update TTL for a key
    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<bool>;

    /// Get TTL for a key
    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>>;

    /// Batch get operation
    async fn mget(&self, keys: &[String]) -> CacheResult<Vec<Option<CacheEntry>>>;

    /// Batch set operation
    async fn mset(&self, entries: Vec<(String, Bytes, CacheOptions)>) -> CacheResult<()>;

    /// Flush any pending writes (for write-back caches)
    async fn flush(&self) -> CacheResult<()>;

    /// Close the backend and cleanup resources
    async fn close(&self) -> CacheResult<()>;
}
