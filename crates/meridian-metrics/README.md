# Meridian Metrics

Enterprise metrics and telemetry system for the Meridian GIS Platform v0.1.5.

## Features

- **OpenTelemetry Integration**: Full distributed tracing with OTLP export support
- **Prometheus Metrics**: Industry-standard metrics exposition format
- **GIS-Specific Metrics**: Query latency, tile rendering time, spatial operation duration
- **Health Check System**: Detailed component health monitoring with automatic checks
- **Performance Profiling**: CPU profiling with flamegraph generation
- **SLA Monitoring**: Threshold-based alerting with configurable severity levels
- **Real-time Streaming**: WebSocket-based metrics streaming to clients
- **Metric Aggregation**: Automatic rollup strategies with configurable retention
- **Custom Metric Types**: Counters, Gauges, Histograms, and Summaries
- **Multi-tenant Support**: Label-based filtering and metric dimensions

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
meridian-metrics = { path = "../meridian-metrics" }
```

Basic usage:

```rust
use meridian_metrics::{MetricsSystem, GisMetricType};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the complete metrics system
    let system = Arc::new(MetricsSystem::init_with_defaults()?);

    // Start all services
    system.start_all().await?;

    // Use metrics
    let counter = system.collector.register_counter(
        "requests_total",
        "Total requests"
    )?;
    counter.inc();

    // Record GIS-specific metrics
    system.collector.record_gis_metric(GisMetricType::QueryLatency(123.45))?;

    Ok(())
}
```

## Architecture

The metrics system consists of several integrated components:

- **Collector**: Central hub for metric registration and collection
- **Exporters**: Prometheus HTTP endpoint and OpenTelemetry OTLP export
- **Health**: System health monitoring with configurable checks
- **SLA Monitor**: Threshold-based alerting with customizable rules
- **Streaming**: WebSocket server for real-time metric streaming
- **Aggregator**: Time-series aggregation with rollup strategies
- **Profiler**: CPU profiling with flamegraph generation

## Metric Types

### Counter

Monotonically increasing counter for events:

```rust
let requests = collector.register_counter("http_requests_total", "Total HTTP requests")?;
requests.inc();
requests.add(5);
```

### Gauge

Value that can go up or down:

```rust
let memory = collector.register_gauge("memory_usage_bytes", "Memory usage")?;
memory.set(1024.0);
memory.add(512.0);
```

### Histogram

Statistical distribution of values:

```rust
let latency = collector.register_histogram("query_latency_ms", "Query latency")?;
latency.observe(123)?;
println!("P95: {}", latency.percentile(0.95));
```

### Summary

Streaming quantiles over a sliding window:

```rust
let summary = collector.register_summary("response_time_ms", "Response time")?;
summary.observe(45.6);
let stats = summary.get_stats();
println!("Mean: {:.2}ms, P99: {:.2}ms", stats.mean, stats.p99);
```

## GIS-Specific Metrics

```rust
use meridian_metrics::GisMetricType;

// Record tile rendering time
collector.record_gis_metric(GisMetricType::TileRenderTime {
    zoom: 12,
    x: 1234,
    y: 5678,
    duration_ms: 45.2,
})?;

// Record spatial operation
collector.record_gis_metric(GisMetricType::SpatialOperation {
    operation: "intersection".to_string(),
    geometry_count: 1000,
    duration_ms: 234.5,
})?;

// Record cache metrics
collector.record_gis_metric(GisMetricType::CacheHitRate {
    cache_type: "tile_cache".to_string(),
    hits: 950,
    misses: 50,
})?;
```

## SLA Monitoring

Define and monitor SLA thresholds:

```rust
use meridian_metrics::{SlaThreshold, ThresholdComparison, AlertSeverity};

let threshold = SlaThreshold::new(
    "query_latency_sla",
    "query_latency_ms",
    100.0,
    ThresholdComparison::GreaterThan,
)
.with_description("Query latency must be under 100ms")
.with_severity(AlertSeverity::Warning)
.with_window(60);

monitor.add_threshold(threshold);

// Register alert callback
monitor.on_alert(|alert| {
    println!("SLA VIOLATION: {} - {}", alert.threshold.name, alert.actual_value);
});
```

## Health Checks

Implement custom health checkers:

```rust
use meridian_metrics::{HealthChecker, ComponentHealth};
use async_trait::async_trait;

struct DatabaseHealthChecker;

#[async_trait]
impl HealthChecker for DatabaseHealthChecker {
    async fn check(&self) -> Result<ComponentHealth> {
        // Perform health check
        Ok(ComponentHealth::healthy("database"))
    }

    fn name(&self) -> &str {
        "database"
    }
}

// Register the checker
health.register_checker(Box::new(DatabaseHealthChecker)).await;
```

## Performance Profiling

CPU profiling with flamegraph generation:

```rust
let profiler = Profiler::default()?;

// Manual profiling
let session_id = profiler.start_profile("my_operation")?;
// ... perform work ...
let session = profiler.stop_profile()?;

println!("Profile saved to: {:?}", session.flamegraph_path);

// Scoped profiling
{
    let _profile = ScopedProfile::new(Arc::new(profiler), "scoped_work")?;
    // ... work is automatically profiled ...
}
```

## Real-time Streaming

WebSocket-based metrics streaming:

```rust
// Server side
let streaming = Arc::new(StreamingServer::default(collector));
streaming.start().await?;

// Client connects to ws://localhost:9091/ws
// Receives real-time metric updates
```

## Metric Aggregation

Configure automatic rollup rules:

```rust
use meridian_metrics::{RollupRule, AggregationFunc, TimeWindow};

let rule = RollupRule {
    name: "5m_avg_latency".to_string(),
    metric_pattern: "*_latency_*".to_string(),
    function: AggregationFunc::Average,
    window: TimeWindow::FiveMinutes,
    retention_secs: 86400,
    enabled: true,
};

aggregator.add_rule(rule);
```

## Prometheus Export

Metrics are automatically exported in Prometheus format at:

```
http://localhost:9090/metrics
```

Example output:

```
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total 42

# HELP query_latency_ms Query latency
# TYPE query_latency_ms histogram
query_latency_ms_sum 1234.5
query_latency_ms_count 100
query_latency_ms{quantile="0.95"} 125.3
query_latency_ms{quantile="0.99"} 145.7
```

## OpenTelemetry Export

Configure OTLP export:

```rust
let config = ExporterConfig {
    otlp_enabled: true,
    otlp_endpoint: "http://localhost:4317".to_string(),
    export_interval_secs: 15,
    ..Default::default()
};
```

## Multi-tenant Filtering

Use labels for multi-dimensional metrics:

```rust
use std::collections::HashMap;

let mut labels = HashMap::new();
labels.insert("tenant".to_string(), "acme_corp".to_string());
labels.insert("region".to_string(), "us-west".to_string());

let counter = collector.register_counter_with_labels(
    "requests",
    "Request count",
    labels
)?;
```

## Configuration

All components support configuration:

```rust
use meridian_metrics::*;

let system = MetricsSystem::with_config(
    CollectorConfig {
        enabled: true,
        interval_secs: 15,
        max_metrics: 10000,
        enable_gis_metrics: true,
        buffer_size: 1000,
    },
    ExporterConfig {
        prometheus_enabled: true,
        prometheus_port: 9090,
        otlp_enabled: true,
        otlp_endpoint: "http://localhost:4317".to_string(),
        export_interval_secs: 15,
        timeout_secs: 10,
    },
    HealthCheckConfig {
        enabled: true,
        interval_secs: 30,
        timeout_secs: 5,
        cpu_degraded_threshold: 70.0,
        cpu_unhealthy_threshold: 90.0,
        memory_degraded_threshold: 75.0,
        memory_unhealthy_threshold: 90.0,
    },
    SlaMonitorConfig {
        enabled: true,
        check_interval_secs: 10,
        max_active_alerts: 1000,
        alert_retention_secs: 86400,
    },
    StreamingConfig {
        enabled: true,
        port: 9091,
        interval_ms: 1000,
        max_connections: 100,
        buffer_size: 1000,
        compression: true,
    },
    ProfilerConfig {
        enabled: true,
        output_dir: PathBuf::from("/tmp/profiles"),
        frequency: 100,
        enable_flamegraph: true,
        auto_profile: false,
        auto_profile_duration_secs: 60,
    },
)?;
```

## Examples

See the `examples/` directory for complete examples:

- `basic_metrics.rs` - Basic metric collection
- `gis_metrics.rs` - GIS-specific metrics
- `sla_monitoring.rs` - SLA threshold monitoring
- `streaming.rs` - WebSocket streaming
- `profiling.rs` - Performance profiling

## Testing

Run tests:

```bash
cargo test -p meridian-metrics
```

Run benchmarks:

```bash
cargo bench -p meridian-metrics
```

## Performance

The metrics system is designed for minimal overhead:

- Lock-free counters using atomic operations
- Efficient concurrent hash maps for metric storage
- Batch export to reduce network overhead
- Configurable sampling and aggregation

## License

MIT OR Apache-2.0

## Contributing

See the main Meridian repository for contribution guidelines.
