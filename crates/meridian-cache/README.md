# Meridian Cache

Enterprise distributed caching system for the Meridian GIS Platform v0.1.5.

## Features

### Multi-Tier Caching Architecture
- **L1 In-Memory Cache**: Lightning-fast LRU, LFU, and ARC eviction strategies
- **L2 Redis Cache**: Distributed caching with cluster support and automatic failover
- **L3 Disk Cache**: Persistent cache with sharding and efficient indexing

### Advanced Caching Strategies
- **Eviction Algorithms**: LRU (Least Recently Used), LFU (Least Frequently Used), ARC (Adaptive Replacement Cache)
- **Write Policies**: Write-through, write-back, and write-around
- **TTL Management**: Flexible time-to-live configuration

### Cache Invalidation
- **TTL-based**: Automatic expiration based on time-to-live
- **Dependency-based**: Cascade invalidation with dependency tracking
- **Event-based**: Subscribe to events for automatic invalidation
- **Tag-based**: Group and invalidate cache entries by tags
- **Query invalidation**: Smart invalidation for database query results

### Compression Support
- **LZ4**: Fast compression for frequently accessed data
- **Zstd**: High compression ratio for larger datasets
- **Adaptive**: Automatically selects the best algorithm
- **Configurable thresholds**: Only compress when beneficial

### Cache Warming
- **Eager loading**: Pre-load all critical data at startup
- **Lazy loading**: Load data on-demand with prioritization
- **Scheduled warmup**: Periodic background refresh
- **Batch processing**: Efficient concurrent loading

### GIS Optimization
- **Tile caching**: Specialized caching for map tiles
- **Tile pyramid support**: Multi-resolution tile management
- **Compression for vector tiles**: Efficient storage of PBF, GeoJSON
- **Neighbor pre-fetching**: Intelligent tile prefetching
- **Spatial invalidation**: Zoom-level and bbox-based invalidation

### Monitoring & Metrics
- **Hit/miss ratio tracking**: Real-time cache performance metrics
- **Latency monitoring**: Average GET/SET operation times
- **Memory usage**: Current and peak memory consumption
- **Eviction tracking**: Monitor cache churn
- **Performance analysis**: Automated recommendations

## Quick Start

### Basic In-Memory Cache

```rust
use meridian_cache::prelude::*;
use bytes::Bytes;

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Create an LRU cache with 1000 entries
    let cache = MemoryCache::lru(1000);

    // Store a value
    cache.set(
        "user:123",
        Bytes::from("user data"),
        CacheOptions::default(),
    ).await?;

    // Retrieve the value
    if let Some(entry) = cache.get("user:123").await? {
        println!("Found: {:?}", entry.value);
    }

    // Delete a value
    cache.delete("user:123").await?;

    Ok(())
}
```

### Multi-Tier Cache

```rust
use meridian_cache::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> CacheResult<()> {
    // Create L1 in-memory cache (fast, small)
    let l1 = Arc::new(MemoryCache::lru(1000));

    // Create L2 Redis cache (distributed, medium)
    let l2 = Arc::new(RedisCache::connect("redis://localhost:6379").await?);

    // Create L3 disk cache (persistent, large)
    let l3 = Arc::new(DiskCache::with_dir("/var/cache/meridian").await?);

    // Build tiered cache with write-through policy
    let cache = TieredCacheBuilder::new(WritePolicy::WriteThrough)
        .add_tier(
            TierConfig {
                name: "L1-Memory".to_string(),
                priority: 0,
                promote_on_hit: true,
                write_policy: WritePolicy::WriteThrough,
            },
            l1,
        )
        .add_tier(
            TierConfig {
                name: "L2-Redis".to_string(),
                priority: 1,
                promote_on_hit: true,
                write_policy: WritePolicy::WriteThrough,
            },
            l2,
        )
        .add_tier(
            TierConfig {
                name: "L3-Disk".to_string(),
                priority: 2,
                promote_on_hit: false,
                write_policy: WritePolicy::WriteThrough,
            },
            l3,
        )
        .build();

    // Use the cache - automatically manages all tiers
    cache.set("key", bytes::Bytes::from("value"), CacheOptions::default()).await?;

    // Get from cache - checks L1, then L2, then L3
    if let Some(entry) = cache.get("key").await? {
        println!("Cache hit: {:?}", entry.value);
    }

    Ok(())
}
```

### Redis Cluster

```rust
use meridian_cache::prelude::*;

#[tokio::main]
async fn main() -> CacheResult<()> {
    let cache = RedisCache::connect_cluster(vec![
        "redis://node1:6379".to_string(),
        "redis://node2:6379".to_string(),
        "redis://node3:6379".to_string(),
    ]).await?;

    cache.set("key", bytes::Bytes::from("value"), CacheOptions::default()).await?;

    Ok(())
}
```

### GIS Tile Caching

```rust
use meridian_cache::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> CacheResult<()> {
    let backend = Arc::new(MemoryCache::lru(10000));
    let tile_cache = TileCache::new(backend, TileCacheConfig::default());

    // Define a tile coordinate (z/x/y)
    let coord = TileCoord::new(10, 512, 256);

    // Cache a tile
    let tile_data = bytes::Bytes::from("tile image data");
    tile_cache.set_tile("osm", coord, tile_data, TileFormat::PNG).await?;

    // Retrieve a tile
    if let Some(data) = tile_cache.get_tile("osm", coord).await? {
        println!("Tile size: {} bytes", data.len());
    }

    // Invalidate all tiles for a layer
    tile_cache.invalidate_layer("osm").await?;

    // Invalidate tiles in a zoom range
    tile_cache.invalidate_zoom_range("osm", 0, 5).await?;

    Ok(())
}
```

### Cache Invalidation

```rust
use meridian_cache::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> CacheResult<()> {
    let cache = Arc::new(MemoryCache::lru(1000));
    let policy = Arc::new(InvalidationPolicy::dependency());

    // Set up dependency: user profile depends on user data
    policy.add_dependency("user:123:profile", vec!["user:123".to_string()]);
    policy.add_dependency("user:123:settings", vec!["user:123".to_string()]);

    // When user:123 is invalidated, all dependents are also invalidated
    let count = policy.invalidate_cascade(&*cache, "user:123").await?;
    println!("Invalidated {} cache entries", count);

    Ok(())
}
```

### Compression

```rust
use meridian_cache::prelude::*;

fn main() -> CacheResult<()> {
    let compressor = AdaptiveCompressor::default();

    let data = b"Some large data to compress".repeat(100);
    let compressed = compressor.compress(&data)?;

    println!("Original size: {} bytes", data.len());
    println!("Compressed size: {} bytes", compressed.data.len());
    println!("Compression ratio: {:.2}%", compressed.compression_ratio() * 100.0);

    // Decompress
    let decompressed = compressor.decompress(&compressed)?;
    assert_eq!(decompressed, data);

    Ok(())
}
```

### Cache Warming

```rust
use meridian_cache::prelude::*;
use std::sync::Arc;

// Implement a warmup source
struct MyDataSource;

#[async_trait::async_trait]
impl WarmupSource for MyDataSource {
    async fn get_keys(&self) -> CacheResult<Vec<String>> {
        Ok(vec!["key1".to_string(), "key2".to_string()])
    }

    async fn fetch(&self, key: &str) -> CacheResult<bytes::Bytes> {
        // Fetch from database or external service
        Ok(bytes::Bytes::from(format!("data for {}", key)))
    }
}

#[tokio::main]
async fn main() -> CacheResult<()> {
    let cache = Arc::new(MemoryCache::lru(1000));
    let source = Arc::new(MyDataSource);

    let warmer = CacheWarmer::new(cache.clone(), WarmupConfig::default());
    let stats = warmer.warmup(source).await?;

    println!("Warmed up {} keys", stats.succeeded);
    println!("Hit rate: {:.2}%", stats.success_rate());

    Ok(())
}
```

### Metrics and Statistics

```rust
use meridian_cache::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> CacheResult<()> {
    let cache = MemoryCache::lru(1000);

    // Perform some operations
    cache.set("key1", bytes::Bytes::from("value1"), CacheOptions::default()).await?;
    cache.get("key1").await?;
    cache.get("key2").await?; // miss

    // Get statistics
    let stats = cache.stats().await?;
    println!("Hits: {}", stats.hits);
    println!("Misses: {}", stats.misses);
    println!("Hit rate: {:.2}%", stats.hit_rate());

    // Use metrics collector for detailed tracking
    let collector = MetricsCollector::default();
    collector.record_get(Duration::from_micros(100), true);
    collector.record_set(Duration::from_micros(200));

    let metrics = collector.get_metrics();
    println!("Avg GET latency: {:.2} μs", metrics.avg_get_latency_us());
    println!("Ops/sec: {:.2}", metrics.ops_per_second());

    Ok(())
}
```

## Architecture

### Cache Backends

- **Memory**: In-process cache with LRU/LFU/ARC eviction
- **Redis**: Distributed cache with cluster support
- **Disk**: Persistent file-based cache with sharding

### Tiered Caching

The tiered cache manager coordinates multiple cache backends:

1. **L1 (Memory)**: Fastest, smallest capacity
2. **L2 (Redis)**: Fast, distributed, medium capacity
3. **L3 (Disk)**: Slower, largest capacity, persistent

On read:
- Check L1 → L2 → L3 in order
- Promote hits to higher tiers (optional)

On write:
- Write-through: Write to all tiers synchronously
- Write-back: Write to L1, async to others
- Write-around: Write only to persistent storage

### Compression

The compression layer automatically selects the best algorithm:

- **LZ4**: For small data and speed-critical operations
- **Zstd**: For large data requiring high compression

Adaptive compression benchmarks both algorithms and selects the optimal one.

## Performance

### Benchmarks

- **Memory cache**: ~500ns per GET operation
- **Redis cache**: ~1ms per GET operation (local)
- **Disk cache**: ~5ms per GET operation (SSD)

### Optimization Tips

1. **Use tiered caching** for best balance of speed and capacity
2. **Enable compression** for data larger than 1KB
3. **Configure appropriate TTLs** to balance freshness and hit rate
4. **Use cache warming** for predictable access patterns
5. **Monitor metrics** and adjust based on performance analysis

## Dependencies

- `tokio`: Async runtime
- `redis`: Redis client with cluster support
- `lru`: LRU cache implementation
- `lz4`: Fast compression
- `zstd`: High-ratio compression
- `dashmap`: Concurrent hash map
- `parking_lot`: High-performance synchronization

## License

MIT OR Apache-2.0

## Version

0.1.5
