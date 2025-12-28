//! Complete metrics system example with all features enabled.

use meridian_metrics::{
    AlertSeverity, AggregationFunc, GisMetricType, MetricsSystem, RollupRule, SlaThreshold,
    ThresholdComparison, TimeWindow,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for observability
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Meridian Metrics - Complete System Example ===\n");

    // 1. Initialize the complete metrics system with defaults
    println!("1. Initializing Metrics System");
    let system = Arc::new(MetricsSystem::init_with_defaults()?);
    println!("   ✓ Metrics system initialized");
    println!("   ✓ Default GIS metrics registered");
    println!("   ✓ Default SLA thresholds configured");
    println!("   ✓ Default rollup rules configured");

    // 2. Start all background services
    println!("\n2. Starting Background Services");
    system.start_all().await?;
    println!("   ✓ Prometheus exporter started on :9090");
    println!("   ✓ WebSocket streaming started on :9091");
    println!("   ✓ Health checks enabled");
    println!("   ✓ SLA monitoring active");
    println!("   ✓ Metric aggregation running");

    // 3. Register custom metrics
    println!("\n3. Registering Custom Metrics");
    let request_counter = system.collector.counter("custom_requests_total")?;
    let active_users = system.collector.gauge("active_users")?;
    let processing_time = system.collector.histogram("processing_time_ms")?;
    println!("   ✓ Custom metrics registered");

    // 4. Add custom SLA threshold
    println!("\n4. Adding Custom SLA Threshold");
    system.sla.add_threshold(
        SlaThreshold::new(
            "processing_time_sla",
            "processing_time_ms",
            500.0,
            ThresholdComparison::GreaterThan,
        )
        .with_description("Processing time must be under 500ms")
        .with_severity(AlertSeverity::Warning),
    );
    println!("   ✓ Custom SLA threshold added");

    // 5. Add custom rollup rule
    println!("\n5. Adding Custom Rollup Rule");
    system.aggregator.add_rule(RollupRule {
        name: "15m_p99_processing".to_string(),
        metric_pattern: "processing_*".to_string(),
        function: AggregationFunc::P99,
        window: TimeWindow::FifteenMinutes,
        retention_secs: 86400,
        enabled: true,
    });
    println!("   ✓ Custom rollup rule added");

    // 6. Simulate application workload
    println!("\n6. Simulating Application Workload (10 seconds)");
    println!("   Press Ctrl+C to stop...\n");

    for i in 0..10 {
        // Increment request counter
        request_counter.inc();

        // Update active users (simulate fluctuation)
        let users = 100.0 + (i as f64 * 5.0);
        active_users.set(users);

        // Record processing time
        let proc_time = 200 + (i * 30);
        processing_time.observe(proc_time)?;

        // Record GIS metrics
        system.collector.record_gis_metric(GisMetricType::QueryLatency(
            50.0 + (i as f64 * 10.0),
        ))?;

        system
            .collector
            .record_gis_metric(GisMetricType::TileRenderTime {
                zoom: 12,
                x: 1000 + i,
                y: 2000 + i,
                duration_ms: 40.0 + (i as f64 * 5.0),
            })?;

        // Check SLA violations
        let query_latency = 50.0 + (i as f64 * 10.0);
        if let Ok(alerts) = system.sla.check_metric("gis_query_latency_ms", query_latency) {
            if !alerts.is_empty() {
                println!("   [{}s] ⚠️  SLA violation detected!", i + 1);
            }
        }

        println!(
            "   [{}s] Requests: {}, Users: {:.0}, Processing: {}ms",
            i + 1,
            request_counter.get(),
            users,
            proc_time
        );

        sleep(Duration::from_secs(1)).await;
    }

    // 7. Check health status
    println!("\n7. Health Check");
    if let Ok(health) = system.health.check().await {
        println!("   Overall status: {:?}", health.status);
        println!("   Uptime: {} seconds", health.uptime_secs);
        println!("   CPU usage: {:.1}%", health.system.cpu_usage);
        println!(
            "   Memory usage: {:.1}%",
            health.system.memory_percent
        );
        println!("   Components checked: {}", health.components.len());

        for component in &health.components {
            println!(
                "     - {}: {:?}",
                component.name, component.status
            );
        }
    }

    // 8. View metrics summary
    println!("\n8. Metrics Summary");
    let snapshots = system.collector.collect_snapshots();
    println!("   Total metrics: {}", snapshots.len());

    // Count by type
    let mut counters = 0;
    let mut gauges = 0;
    let mut histograms = 0;

    for snapshot in &snapshots {
        match snapshot.value {
            meridian_metrics::MetricValue::Counter { .. } => counters += 1,
            meridian_metrics::MetricValue::Gauge { .. } => gauges += 1,
            meridian_metrics::MetricValue::Histogram { .. } => histograms += 1,
            meridian_metrics::MetricValue::Summary { .. } => {}
        }
    }

    println!("   Counters: {}", counters);
    println!("   Gauges: {}", gauges);
    println!("   Histograms: {}", histograms);

    // 9. View SLA alerts
    println!("\n9. SLA Alerts");
    let alerts = system.sla.active_alerts();
    println!("   Active alerts: {}", alerts.len());

    for (i, alert) in alerts.iter().take(5).enumerate() {
        println!(
            "   {}. {} - {} (severity: {})",
            i + 1,
            alert.threshold.name,
            alert.threshold.metric_name,
            alert.severity().as_str()
        );
    }

    // 10. View GIS metrics
    println!("\n10. GIS Metrics");
    let gis_metrics = system.collector.get_gis_metrics();
    println!("   Total GIS metrics: {}", gis_metrics.len());

    // 11. System info
    println!("\n11. System Information");
    println!("   Metrics endpoints:");
    println!("     - Prometheus: http://localhost:9090/metrics");
    println!("     - WebSocket: ws://localhost:9091/ws");
    println!("     - Health: http://localhost:9091/health");
    println!("\n   Try accessing these endpoints while the system is running!");

    // Keep running to allow external access
    println!("\n12. Running...");
    println!("   System is now running. Check the endpoints above.");
    println!("   Press Ctrl+C to stop.\n");

    // Keep alive for 60 seconds to allow testing
    sleep(Duration::from_secs(60)).await;

    println!("\n=== Example Complete ===");

    Ok(())
}
