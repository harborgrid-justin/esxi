//! Basic metrics example demonstrating counters, gauges, histograms, and summaries.

use meridian_metrics::{MetricsCollector, MetricsSystem};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Meridian Metrics - Basic Example ===\n");

    // Create a metrics collector
    let collector = Arc::new(MetricsCollector::default());

    // Register a counter
    println!("1. Counter Example");
    let requests_counter = collector.register_counter(
        "http_requests_total",
        "Total number of HTTP requests",
    )?;

    requests_counter.inc();
    requests_counter.inc();
    requests_counter.add(3);

    println!("   Requests counter: {}", requests_counter.get());

    // Register a gauge
    println!("\n2. Gauge Example");
    let memory_gauge = collector.register_gauge(
        "memory_usage_bytes",
        "Current memory usage in bytes",
    )?;

    memory_gauge.set(1024.0);
    memory_gauge.add(512.0);
    memory_gauge.sub(256.0);

    println!("   Memory gauge: {} bytes", memory_gauge.get());

    // Register a histogram
    println!("\n3. Histogram Example");
    let latency_histogram = collector.register_histogram(
        "request_latency_ms",
        "Request latency in milliseconds",
    )?;

    // Simulate some requests with varying latency
    for latency in [10, 15, 20, 25, 30, 35, 40, 50, 75, 100] {
        latency_histogram.observe(latency)?;
    }

    println!("   Histogram stats:");
    println!("     Count: {}", latency_histogram.count());
    println!("     Min: {} ms", latency_histogram.min());
    println!("     Max: {} ms", latency_histogram.max());
    println!("     Mean: {:.2} ms", latency_histogram.mean());
    println!("     P50: {} ms", latency_histogram.percentile(0.5));
    println!("     P95: {} ms", latency_histogram.percentile(0.95));
    println!("     P99: {} ms", latency_histogram.percentile(0.99));

    // Register a summary
    println!("\n4. Summary Example");
    let response_summary = collector.register_summary(
        "response_time_ms",
        "Response time in milliseconds",
    )?;

    for time in [45, 50, 55, 60, 65, 70, 75, 80, 85, 90] {
        response_summary.observe(time as f64);
    }

    let stats = response_summary.get_stats();
    println!("   Summary stats:");
    println!("     Count: {}", stats.count);
    println!("     Mean: {:.2} ms", stats.mean);
    println!("     Min: {:.2} ms", stats.min);
    println!("     Max: {:.2} ms", stats.max);
    println!("     P50: {:.2} ms", stats.p50);
    println!("     P90: {:.2} ms", stats.p90);
    println!("     P99: {:.2} ms", stats.p99);

    // Collect all snapshots
    println!("\n5. Metric Snapshots");
    let snapshots = collector.collect_snapshots();
    println!("   Total metrics collected: {}", snapshots.len());

    for snapshot in snapshots.iter().take(3) {
        println!("   - {}: {:?}", snapshot.name, snapshot.value);
    }

    println!("\n6. Metrics with Labels");
    use std::collections::HashMap;

    let mut labels = HashMap::new();
    labels.insert("tenant".to_string(), "acme_corp".to_string());
    labels.insert("region".to_string(), "us-west".to_string());

    let tenant_counter = collector.register_counter_with_labels(
        "tenant_requests",
        "Requests per tenant",
        labels,
    )?;

    tenant_counter.add(100);
    println!("   Tenant requests: {}", tenant_counter.get());

    println!("\n=== Example Complete ===");

    Ok(())
}
