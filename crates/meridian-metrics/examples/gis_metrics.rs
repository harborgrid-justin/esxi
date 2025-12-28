//! GIS-specific metrics example demonstrating spatial operation tracking.

use meridian_metrics::{GisMetricType, MetricsCollector};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("=== Meridian Metrics - GIS Example ===\n");

    let collector = Arc::new(MetricsCollector::default());

    // 1. Track query latency
    println!("1. Query Latency Tracking");
    for i in 0..5 {
        let latency = 50.0 + (i as f64 * 10.0);
        collector.record_gis_metric(GisMetricType::QueryLatency(latency))?;
        println!("   Query {} latency: {:.2}ms", i + 1, latency);
    }

    // 2. Track tile rendering
    println!("\n2. Tile Rendering Performance");
    let tiles = vec![
        (12, 1234, 5678, 45.2),
        (12, 1235, 5678, 42.8),
        (12, 1234, 5679, 48.5),
        (13, 2468, 11356, 52.3),
    ];

    for (zoom, x, y, duration) in tiles {
        collector.record_gis_metric(GisMetricType::TileRenderTime {
            zoom,
            x,
            y,
            duration_ms: duration,
        })?;
        println!(
            "   Rendered tile z{}/x{}/y{} in {:.2}ms",
            zoom, x, y, duration
        );
    }

    // 3. Track spatial operations
    println!("\n3. Spatial Operations");
    let operations = vec![
        ("intersection", 1000, 234.5),
        ("buffer", 500, 156.3),
        ("union", 2000, 456.7),
        ("simplify", 1500, 89.4),
    ];

    for (op, geom_count, duration) in operations {
        collector.record_gis_metric(GisMetricType::SpatialOperation {
            operation: op.to_string(),
            geometry_count: geom_count,
            duration_ms: duration,
        })?;
        println!(
            "   {} on {} geometries: {:.2}ms",
            op, geom_count, duration
        );
    }

    // 4. Track cache performance
    println!("\n4. Cache Performance");
    collector.record_gis_metric(GisMetricType::CacheHitRate {
        cache_type: "tile_cache".to_string(),
        hits: 950,
        misses: 50,
    })?;

    println!("   Tile cache: 950 hits, 50 misses (95% hit rate)");

    collector.record_gis_metric(GisMetricType::CacheHitRate {
        cache_type: "geometry_cache".to_string(),
        hits: 1200,
        misses: 300,
    })?;

    println!("   Geometry cache: 1200 hits, 300 misses (80% hit rate)");

    // 5. Track data loading
    println!("\n5. Data Loading");
    let datasets = vec![
        ("shapefile", 1024 * 1024, 1234.5),
        ("geojson", 512 * 1024, 567.8),
        ("geopackage", 2 * 1024 * 1024, 2345.6),
    ];

    for (source, bytes, duration) in datasets {
        collector.record_gis_metric(GisMetricType::DataLoadTime {
            source: source.to_string(),
            bytes,
            duration_ms: duration,
        })?;
        println!(
            "   Loaded {} ({} KB) in {:.2}ms",
            source,
            bytes / 1024,
            duration
        );
    }

    // 6. Retrieve and display all GIS metrics
    println!("\n6. All GIS Metrics");
    let gis_metrics = collector.get_gis_metrics();
    println!("   Total GIS metrics recorded: {}", gis_metrics.len());

    // Simulate continuous monitoring
    println!("\n7. Continuous Monitoring (5 seconds)");
    for i in 0..5 {
        sleep(Duration::from_secs(1)).await;

        // Simulate varying query latency
        let latency = 50.0 + (rand::random::<f64>() * 50.0);
        collector.record_gis_metric(GisMetricType::QueryLatency(latency))?;

        println!("   [{}s] Query latency: {:.2}ms", i + 1, latency);
    }

    println!("\n=== Example Complete ===");
    println!("Total GIS metrics: {}", collector.get_gis_metrics().len());

    Ok(())
}

// Simple random number generator (avoiding external deps)
mod rand {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};
    use std::time::SystemTime;

    pub fn random<T: Hash + Default>() -> f64 {
        let mut hasher = RandomState::new().build_hasher();
        SystemTime::now().hash(&mut hasher);
        (hasher.finish() % 1000) as f64 / 1000.0
    }
}
