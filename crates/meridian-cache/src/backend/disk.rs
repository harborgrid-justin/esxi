//! Disk-based cache backend for persistent caching

use async_trait::async_trait;
use bytes::Bytes;
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::{CacheBackend, CacheEntry, CacheMetadata, CacheOptions};
use crate::error::{CacheError, CacheResult};
use crate::stats::CacheStats;

/// Disk cache configuration
#[derive(Debug, Clone)]
pub struct DiskCacheConfig {
    /// Root directory for cache storage
    pub cache_dir: PathBuf,
    /// Maximum cache size in bytes
    pub max_size_bytes: usize,
    /// Maximum number of entries
    pub max_entries: usize,
    /// Default TTL
    pub default_ttl: Option<Duration>,
    /// Sync writes to disk immediately
    pub sync_writes: bool,
    /// Number of subdirectories for sharding (0-255)
    pub shard_count: u8,
}

impl Default for DiskCacheConfig {
    fn default() -> Self {
        Self {
            cache_dir: PathBuf::from("/tmp/meridian-cache"),
            max_size_bytes: 1024 * 1024 * 1024, // 1 GB
            max_entries: 100_000,
            default_ttl: Some(Duration::from_secs(86400)), // 24 hours
            sync_writes: false,
            shard_count: 16,
        }
    }
}

/// Index entry for tracking cached files
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct IndexEntry {
    /// File path relative to cache_dir
    file_path: PathBuf,
    /// Metadata
    metadata: CacheMetadata,
}

/// Disk cache backend
pub struct DiskCache {
    config: DiskCacheConfig,
    index: Arc<RwLock<HashMap<String, IndexEntry>>>,
    stats: Arc<RwLock<CacheStats>>,
    current_size: Arc<parking_lot::Mutex<usize>>,
}

impl DiskCache {
    /// Create a new disk cache
    pub async fn new(config: DiskCacheConfig) -> CacheResult<Self> {
        // Create cache directory if it doesn't exist
        fs::create_dir_all(&config.cache_dir).await?;

        // Create shard directories
        for i in 0..config.shard_count {
            let shard_dir = config.cache_dir.join(format!("{:02x}", i));
            fs::create_dir_all(&shard_dir).await?;
        }

        let cache = Self {
            config,
            index: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            current_size: Arc::new(parking_lot::Mutex::new(0)),
        };

        // Load existing index
        cache.load_index().await?;

        Ok(cache)
    }

    /// Create a disk cache with default configuration in the specified directory
    pub async fn with_dir<P: AsRef<Path>>(dir: P) -> CacheResult<Self> {
        Self::new(DiskCacheConfig {
            cache_dir: dir.as_ref().to_path_buf(),
            ..Default::default()
        })
        .await
    }

    /// Get the shard number for a key
    fn get_shard(&self, key: &str) -> u8 {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        hash[0] % self.config.shard_count
    }

    /// Get the file path for a key
    fn get_file_path(&self, key: &str) -> PathBuf {
        let shard = self.get_shard(key);
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hex::encode(hasher.finalize());

        self.config
            .cache_dir
            .join(format!("{:02x}", shard))
            .join(format!("{}.cache", hash))
    }

    /// Get the metadata file path for a key
    fn get_metadata_path(&self, key: &str) -> PathBuf {
        let shard = self.get_shard(key);
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hex::encode(hasher.finalize());

        self.config
            .cache_dir
            .join(format!("{:02x}", shard))
            .join(format!("{}.meta", hash))
    }

    /// Get the index file path
    fn get_index_path(&self) -> PathBuf {
        self.config.cache_dir.join("index.json")
    }

    /// Load the index from disk
    async fn load_index(&self) -> CacheResult<()> {
        let index_path = self.get_index_path();

        if !index_path.exists() {
            return Ok(());
        }

        let data = fs::read(&index_path).await?;
        let loaded_index: HashMap<String, IndexEntry> = serde_json::from_slice(&data)
            .map_err(|e| CacheError::Backend(format!("Failed to parse index: {}", e)))?;

        let mut total_size = 0;
        for entry in loaded_index.values() {
            total_size += entry.metadata.size;
        }

        *self.index.write() = loaded_index;
        *self.current_size.lock() = total_size;

        Ok(())
    }

    /// Save the index to disk
    async fn save_index(&self) -> CacheResult<()> {
        let index_path = self.get_index_path();
        let index = self.index.read().clone();

        let data = serde_json::to_vec(&index)
            .map_err(|e| CacheError::Backend(format!("Failed to serialize index: {}", e)))?;

        fs::write(&index_path, &data).await?;

        Ok(())
    }

    /// Check if an entry is expired
    fn is_expired(&self, entry: &IndexEntry) -> bool {
        if let Some(ttl) = entry.metadata.ttl {
            let age = chrono::Utc::now()
                .signed_duration_since(entry.metadata.created_at)
                .to_std()
                .unwrap_or(Duration::from_secs(0));
            age > ttl
        } else {
            false
        }
    }

    /// Evict entries to make space
    async fn evict_if_needed(&self, new_entry_size: usize) -> CacheResult<()> {
        // Evict LRU entries until we have space
        loop {
            let needs_eviction = {
                let size = self.current_size.lock();
                *size + new_entry_size > self.config.max_size_bytes
            };

            if !needs_eviction {
                break;
            }

            let key_to_evict = {
                let index = self.index.read();
                if index.is_empty() {
                    break;
                }

                // Find LRU entry
                index
                    .iter()
                    .min_by_key(|(_, entry)| entry.metadata.last_accessed)
                    .map(|(k, _)| k.clone())
            };

            if let Some(key) = key_to_evict {
                self.delete_internal(&key, false).await?;
                self.stats.write().evictions += 1;
            } else {
                break;
            }
        }

        let still_over_capacity = {
            let size = self.current_size.lock();
            *size + new_entry_size > self.config.max_size_bytes
        };

        if still_over_capacity {
            return Err(CacheError::CapacityExceeded);
        }

        Ok(())
    }

    /// Internal delete without stats update
    async fn delete_internal(&self, key: &str, update_stats: bool) -> CacheResult<bool> {
        let entry = {
            let mut index = self.index.write();
            index.remove(key)
        };

        if let Some(entry) = entry {
            let file_path = self.get_file_path(key);
            let meta_path = self.get_metadata_path(key);

            // Delete files
            let _ = fs::remove_file(&file_path).await;
            let _ = fs::remove_file(&meta_path).await;

            let mut size = self.current_size.lock();
            *size = size.saturating_sub(entry.metadata.size);

            if update_stats {
                self.stats.write().deletes += 1;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
impl CacheBackend for DiskCache {
    async fn get(&self, key: &str) -> CacheResult<Option<CacheEntry>> {
        let entry = {
            let index = self.index.read();
            index.get(key).cloned()
        };

        if let Some(mut idx_entry) = entry {
            // Check expiration
            if self.is_expired(&idx_entry) {
                self.delete(key).await?;
                self.stats.write().misses += 1;
                return Ok(None);
            }

            let file_path = self.get_file_path(key);

            // Read from disk
            let mut file = fs::File::open(&file_path).await?;
            let mut data = Vec::new();
            file.read_to_end(&mut data).await?;

            // Update access time
            idx_entry.metadata.last_accessed = chrono::Utc::now();
            idx_entry.metadata.access_count += 1;

            {
                let mut index = self.index.write();
                index.insert(key.to_string(), idx_entry.clone());
            }

            self.stats.write().hits += 1;

            Ok(Some(CacheEntry {
                value: Bytes::from(data),
                metadata: idx_entry.metadata,
            }))
        } else {
            self.stats.write().misses += 1;
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: Bytes, options: CacheOptions) -> CacheResult<()> {
        // Evict if needed
        self.evict_if_needed(value.len()).await?;

        let file_path = self.get_file_path(key);
        let meta_path = self.get_metadata_path(key);

        // Write data to disk
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(&value).await?;

        if self.config.sync_writes {
            file.sync_all().await?;
        }

        let metadata = CacheMetadata {
            ttl: options.ttl.or(self.config.default_ttl),
            size: value.len(),
            tags: options.tags,
            ..Default::default()
        };

        // Write metadata
        let meta_data = serde_json::to_vec(&metadata)
            .map_err(|e| CacheError::Backend(format!("Failed to serialize metadata: {}", e)))?;
        fs::write(&meta_path, &meta_data).await?;

        // Update index
        let idx_entry = IndexEntry {
            file_path: file_path.clone(),
            metadata,
        };

        {
            let mut index = self.index.write();
            if let Some(old_entry) = index.insert(key.to_string(), idx_entry.clone()) {
                let mut size = self.current_size.lock();
                *size = size.saturating_sub(old_entry.metadata.size);
            }
        }

        {
            let mut size = self.current_size.lock();
            *size += idx_entry.metadata.size;
        }

        let should_save = {
            self.stats.write().sets += 1;
            self.stats.read().sets.is_multiple_of(100)
        };

        // Periodically save index
        if should_save {
            self.save_index().await?;
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> CacheResult<bool> {
        self.delete_internal(key, true).await
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let index = self.index.read();
        Ok(index.contains_key(key))
    }

    async fn clear(&self) -> CacheResult<()> {
        // Remove all files
        for shard in 0..self.config.shard_count {
            let shard_dir = self.config.cache_dir.join(format!("{:02x}", shard));
            if shard_dir.exists() {
                fs::remove_dir_all(&shard_dir).await?;
                fs::create_dir_all(&shard_dir).await?;
            }
        }

        // Clear index
        self.index.write().clear();
        *self.current_size.lock() = 0;

        // Save empty index
        self.save_index().await?;

        Ok(())
    }

    async fn stats(&self) -> CacheResult<CacheStats> {
        Ok(*self.stats.read())
    }

    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>> {
        let regex_pattern = pattern
            .replace(".", "\\.")
            .replace("*", ".*")
            .replace("?", ".");
        let regex = regex::Regex::new(&format!("^{}$", regex_pattern))
            .map_err(|e| CacheError::InvalidKey(e.to_string()))?;

        let index = self.index.read();
        let keys: Vec<String> = index
            .keys()
            .filter(|k| regex.is_match(k))
            .cloned()
            .collect();

        Ok(keys)
    }

    async fn delete_pattern(&self, pattern: &str) -> CacheResult<usize> {
        let keys = self.keys(pattern).await?;
        let mut count = 0;
        for key in keys {
            if self.delete(&key).await? {
                count += 1;
            }
        }
        Ok(count)
    }

    async fn delete_by_tags(&self, tags: &[String]) -> CacheResult<usize> {
        let keys_to_delete: Vec<String> = {
            let index = self.index.read();
            index
                .iter()
                .filter(|(_, entry)| {
                    tags.iter()
                        .any(|tag| entry.metadata.tags.contains(tag))
                })
                .map(|(k, _)| k.clone())
                .collect()
        };

        let mut count = 0;
        for key in keys_to_delete {
            if self.delete(&key).await? {
                count += 1;
            }
        }

        Ok(count)
    }

    async fn size(&self) -> CacheResult<usize> {
        Ok(*self.current_size.lock())
    }

    async fn len(&self) -> CacheResult<usize> {
        Ok(self.index.read().len())
    }

    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
        let mut index = self.index.write();
        if let Some(entry) = index.get_mut(key) {
            entry.metadata.ttl = Some(ttl);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        let index = self.index.read();
        if let Some(entry) = index.get(key) {
            Ok(entry.metadata.ttl)
        } else {
            Ok(None)
        }
    }

    async fn mget(&self, keys: &[String]) -> CacheResult<Vec<Option<CacheEntry>>> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.get(key).await?);
        }
        Ok(results)
    }

    async fn mset(&self, entries: Vec<(String, Bytes, CacheOptions)>) -> CacheResult<()> {
        for (key, value, options) in entries {
            self.set(&key, value, options).await?;
        }
        Ok(())
    }

    async fn flush(&self) -> CacheResult<()> {
        self.save_index().await
    }

    async fn close(&self) -> CacheResult<()> {
        self.save_index().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_disk_cache_basic() -> CacheResult<()> {
        let temp_dir = TempDir::new().unwrap();
        let cache = DiskCache::with_dir(temp_dir.path()).await?;

        let key = "test_key";
        let value = Bytes::from("test_value");

        cache.set(key, value.clone(), CacheOptions::default()).await?;
        let result = cache.get(key).await?;

        assert!(result.is_some());
        assert_eq!(result.unwrap().value, value);

        cache.delete(key).await?;
        Ok(())
    }
}
