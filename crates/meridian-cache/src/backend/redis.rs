//! Redis cache backend with cluster support

use async_trait::async_trait;
use bytes::Bytes;
use redis::{aio::ConnectionManager, Client, Cmd, FromRedisValue, RedisResult, Value};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

use super::{CacheBackend, CacheEntry, CacheMetadata, CacheOptions};
use crate::error::{CacheError, CacheResult};
use crate::stats::CacheStats;

/// Redis cache configuration
#[derive(Debug, Clone)]
pub struct RedisCacheConfig {
    /// Redis connection URLs (supports cluster)
    pub urls: Vec<String>,
    /// Connection pool size
    pub pool_size: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Operation timeout
    pub operation_timeout: Duration,
    /// Default TTL
    pub default_ttl: Option<Duration>,
    /// Key prefix for namespacing
    pub key_prefix: String,
    /// Enable cluster mode
    pub cluster_mode: bool,
    /// Max retries for failed operations
    pub max_retries: usize,
}

impl Default for RedisCacheConfig {
    fn default() -> Self {
        Self {
            urls: vec!["redis://127.0.0.1:6379".to_string()],
            pool_size: 10,
            connection_timeout: Duration::from_secs(5),
            operation_timeout: Duration::from_secs(3),
            default_ttl: Some(Duration::from_secs(3600)),
            key_prefix: "meridian:cache:".to_string(),
            cluster_mode: false,
            max_retries: 3,
        }
    }
}

/// Redis cache backend
pub struct RedisCache {
    client: Client,
    connection: ConnectionManager,
    config: RedisCacheConfig,
    stats: Arc<parking_lot::RwLock<CacheStats>>,
}

impl RedisCache {
    /// Create a new Redis cache
    pub async fn new(config: RedisCacheConfig) -> CacheResult<Self> {
        // TODO: Implement proper cluster mode support using redis::cluster::ClusterClient
        // For now, use the first URL regardless of cluster mode
        let url = config.urls.first()
            .ok_or_else(|| CacheError::Configuration("No Redis URL provided".to_string()))?;

        let client = Client::open(url.as_str())
            .map_err(|e| CacheError::Configuration(format!("Failed to create Redis client: {}", e)))?;

        let connection = ConnectionManager::new(client.clone())
            .await
            .map_err(|e| CacheError::Redis(e))?;

        Ok(Self {
            client,
            connection,
            config,
            stats: Arc::new(parking_lot::RwLock::new(CacheStats::default())),
        })
    }

    /// Create a connection to a single Redis instance
    pub async fn connect(url: &str) -> CacheResult<Self> {
        Self::new(RedisCacheConfig {
            urls: vec![url.to_string()],
            ..Default::default()
        })
        .await
    }

    /// Create a connection to a Redis cluster
    pub async fn connect_cluster(urls: Vec<String>) -> CacheResult<Self> {
        Self::new(RedisCacheConfig {
            urls,
            cluster_mode: true,
            ..Default::default()
        })
        .await
    }

    /// Get the full key with prefix
    fn prefixed_key(&self, key: &str) -> String {
        format!("{}{}", self.config.key_prefix, key)
    }

    /// Strip prefix from key
    fn strip_prefix(&self, key: &str) -> String {
        key.trim_start_matches(&self.config.key_prefix).to_string()
    }

    /// Serialize cache entry
    fn serialize_entry(&self, entry: &CacheEntry) -> CacheResult<Vec<u8>> {
        bincode::serialize(entry).map_err(|e| CacheError::Serialization(e))
    }

    /// Deserialize cache entry
    fn deserialize_entry(&self, data: &[u8]) -> CacheResult<CacheEntry> {
        bincode::deserialize(data).map_err(|e| CacheError::Serialization(e))
    }

    /// Execute with retry logic
    async fn execute_with_retry<T, F>(&self, mut f: F) -> CacheResult<T>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = RedisResult<T>> + Send>>,
    {
        let mut attempts = 0;
        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    if attempts >= self.config.max_retries {
                        return Err(CacheError::Redis(e));
                    }
                    tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                }
            }
        }
    }
}

#[async_trait]
impl CacheBackend for RedisCache {
    async fn get(&self, key: &str) -> CacheResult<Option<CacheEntry>> {
        let full_key = self.prefixed_key(key);

        let mut conn = self.connection.clone();
        let result: Option<Vec<u8>> = redis::cmd("GET")
            .arg(&full_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::Redis(e))?;

        if let Some(data) = result {
            let entry = self.deserialize_entry(&data)?;
            self.stats.write().hits += 1;
            Ok(Some(entry))
        } else {
            self.stats.write().misses += 1;
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: Bytes, options: CacheOptions) -> CacheResult<()> {
        let full_key = self.prefixed_key(key);

        let metadata = CacheMetadata {
            ttl: options.ttl.or(self.config.default_ttl),
            size: value.len(),
            tags: options.tags.clone(),
            ..Default::default()
        };

        let entry = CacheEntry { value, metadata };
        let data = self.serialize_entry(&entry)?;

        let mut conn = self.connection.clone();

        if let Some(ttl) = entry.metadata.ttl {
            redis::cmd("SETEX")
                .arg(&full_key)
                .arg(ttl.as_secs())
                .arg(&data)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::Redis(e))?;
        } else {
            redis::cmd("SET")
                .arg(&full_key)
                .arg(&data)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::Redis(e))?;
        }

        // Store tags in a separate set for tag-based invalidation
        if !options.tags.is_empty() {
            for tag in &options.tags {
                let tag_key = format!("{}tag:{}", self.config.key_prefix, tag);
                redis::cmd("SADD")
                    .arg(&tag_key)
                    .arg(&full_key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| CacheError::Redis(e))?;
            }
        }

        self.stats.write().sets += 1;
        Ok(())
    }

    async fn delete(&self, key: &str) -> CacheResult<bool> {
        let full_key = self.prefixed_key(key);
        let mut conn = self.connection.clone();

        let result: i32 = redis::cmd("DEL")
            .arg(&full_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::Redis(e))?;

        if result > 0 {
            self.stats.write().deletes += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let full_key = self.prefixed_key(key);
        let mut conn = self.connection.clone();

        let result: bool = redis::cmd("EXISTS")
            .arg(&full_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::Redis(e))?;

        Ok(result)
    }

    async fn clear(&self) -> CacheResult<()> {
        let mut conn = self.connection.clone();

        // Use SCAN to find all keys with our prefix
        let pattern = format!("{}*", self.config.key_prefix);
        let mut cursor = 0u64;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(1000)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::Redis(e))?;

            if !keys.is_empty() {
                redis::cmd("DEL")
                    .arg(&keys)
                    .query_async::<_, ()>(&mut conn)
                    .await
                    .map_err(|e| CacheError::Redis(e))?;
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(())
    }

    async fn stats(&self) -> CacheResult<CacheStats> {
        Ok(*self.stats.read())
    }

    async fn keys(&self, pattern: &str) -> CacheResult<Vec<String>> {
        let mut conn = self.connection.clone();
        let full_pattern = self.prefixed_key(pattern);

        let mut all_keys = Vec::new();
        let mut cursor = 0u64;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&full_pattern)
                .arg("COUNT")
                .arg(1000)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::Redis(e))?;

            all_keys.extend(keys.into_iter().map(|k| self.strip_prefix(&k)));

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(all_keys)
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
        let mut conn = self.connection.clone();
        let mut count = 0;

        for tag in tags {
            let tag_key = format!("{}tag:{}", self.config.key_prefix, tag);

            // Get all keys with this tag
            let keys: Vec<String> = redis::cmd("SMEMBERS")
                .arg(&tag_key)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::Redis(e))?;

            if !keys.is_empty() {
                let deleted: i32 = redis::cmd("DEL")
                    .arg(&keys)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| CacheError::Redis(e))?;
                count += deleted as usize;

                // Remove the tag set
                redis::cmd("DEL")
                    .arg(&tag_key)
                    .query_async::<_, ()>(&mut conn)
                    .await
                    .map_err(|e| CacheError::Redis(e))?;
            }
        }

        Ok(count)
    }

    async fn size(&self) -> CacheResult<usize> {
        let mut conn = self.connection.clone();
        let pattern = format!("{}*", self.config.key_prefix);

        let mut total_size = 0usize;
        let mut cursor = 0u64;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(1000)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::Redis(e))?;

            for key in keys {
                let size: Option<usize> = redis::cmd("STRLEN")
                    .arg(&key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| CacheError::Redis(e))?;
                total_size += size.unwrap_or(0);
            }

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(total_size)
    }

    async fn len(&self) -> CacheResult<usize> {
        let mut conn = self.connection.clone();
        let pattern = format!("{}*", self.config.key_prefix);

        let mut count = 0usize;
        let mut cursor = 0u64;

        loop {
            let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(1000)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::Redis(e))?;

            count += keys.len();

            cursor = new_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(count)
    }

    async fn expire(&self, key: &str, ttl: Duration) -> CacheResult<bool> {
        let full_key = self.prefixed_key(key);
        let mut conn = self.connection.clone();

        let result: bool = redis::cmd("EXPIRE")
            .arg(&full_key)
            .arg(ttl.as_secs())
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::Redis(e))?;

        Ok(result)
    }

    async fn ttl(&self, key: &str) -> CacheResult<Option<Duration>> {
        let full_key = self.prefixed_key(key);
        let mut conn = self.connection.clone();

        let ttl_secs: i64 = redis::cmd("TTL")
            .arg(&full_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::Redis(e))?;

        if ttl_secs < 0 {
            Ok(None)
        } else {
            Ok(Some(Duration::from_secs(ttl_secs as u64)))
        }
    }

    async fn mget(&self, keys: &[String]) -> CacheResult<Vec<Option<CacheEntry>>> {
        let mut conn = self.connection.clone();
        let full_keys: Vec<String> = keys.iter().map(|k| self.prefixed_key(k)).collect();

        let values: Vec<Option<Vec<u8>>> = redis::cmd("MGET")
            .arg(&full_keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::Redis(e))?;

        let mut results = Vec::with_capacity(values.len());
        for value in values {
            if let Some(data) = value {
                let entry = self.deserialize_entry(&data)?;
                results.push(Some(entry));
                self.stats.write().hits += 1;
            } else {
                results.push(None);
                self.stats.write().misses += 1;
            }
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
        // Redis writes are synchronous, so no-op
        Ok(())
    }

    async fn close(&self) -> CacheResult<()> {
        // Connection manager handles cleanup
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis instance
    async fn test_redis_cache_basic() -> CacheResult<()> {
        let cache = RedisCache::connect("redis://127.0.0.1:6379").await?;

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
