//! SLA monitoring example demonstrating threshold-based alerting.

use meridian_metrics::{
    AlertSeverity, AlertStatus, SlaMonitor, SlaThreshold, ThresholdComparison,
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("=== Meridian Metrics - SLA Monitoring Example ===\n");

    let monitor = Arc::new(SlaMonitor::default());

    // 1. Define SLA thresholds
    println!("1. Defining SLA Thresholds");

    let query_latency_sla = SlaThreshold::new(
        "query_latency_sla",
        "query_latency_ms",
        100.0,
        ThresholdComparison::GreaterThan,
    )
    .with_description("Query latency must be under 100ms")
    .with_severity(AlertSeverity::Warning)
    .with_window(60);

    monitor.add_threshold(query_latency_sla);
    println!("   ✓ Query latency SLA: < 100ms (Warning)");

    let tile_render_sla = SlaThreshold::new(
        "tile_render_sla",
        "tile_render_ms",
        200.0,
        ThresholdComparison::GreaterThan,
    )
    .with_description("Tile rendering must be under 200ms")
    .with_severity(AlertSeverity::Warning)
    .with_window(60);

    monitor.add_threshold(tile_render_sla);
    println!("   ✓ Tile render SLA: < 200ms (Warning)");

    let error_rate_sla = SlaThreshold::new(
        "error_rate_sla",
        "error_count",
        10.0,
        ThresholdComparison::GreaterThan,
    )
    .with_description("Error rate must be minimal")
    .with_severity(AlertSeverity::Error)
    .with_window(300)
    .with_min_violations(3);

    monitor.add_threshold(error_rate_sla);
    println!("   ✓ Error rate SLA: < 10 errors (Error, 3 violations)");

    let critical_memory_sla = SlaThreshold::new(
        "critical_memory_sla",
        "memory_usage_percent",
        95.0,
        ThresholdComparison::GreaterThanOrEqual,
    )
    .with_description("Memory usage must not exceed 95%")
    .with_severity(AlertSeverity::Critical);

    monitor.add_threshold(critical_memory_sla);
    println!("   ✓ Memory usage SLA: < 95% (Critical)");

    // 2. Register alert callback
    println!("\n2. Registering Alert Callback");
    monitor.on_alert(|alert| {
        println!(
            "   [ALERT] {} - {} {} threshold {} (actual: {})",
            alert.severity().as_str().to_uppercase(),
            alert.threshold.metric_name,
            alert.threshold.comparison.description(),
            alert.threshold.threshold,
            alert.actual_value
        );
    });
    println!("   ✓ Alert callback registered");

    // Start alert processor
    Arc::clone(&monitor).start_alert_processor().await;
    println!("   ✓ Alert processor started");

    // 3. Simulate metric checks
    println!("\n3. Simulating Metric Checks\n");

    // Good query latency
    println!("   Checking query_latency_ms = 50.0 (OK)");
    let alerts = monitor.check_metric("query_latency_ms", 50.0)?;
    println!("   Alerts triggered: {}", alerts.len());

    sleep(Duration::from_millis(100)).await;

    // Bad query latency (SLA violation)
    println!("\n   Checking query_latency_ms = 150.0 (VIOLATION)");
    let alerts = monitor.check_metric("query_latency_ms", 150.0)?;
    println!("   Alerts triggered: {}", alerts.len());

    sleep(Duration::from_millis(100)).await;

    // Critical memory usage
    println!("\n   Checking memory_usage_percent = 97.0 (CRITICAL)");
    let alerts = monitor.check_metric("memory_usage_percent", 97.0)?;
    println!("   Alerts triggered: {}", alerts.len());

    sleep(Duration::from_millis(100)).await;

    // Multiple violations for error count
    println!("\n   Testing minimum violations threshold:");
    for i in 1..=5 {
        println!("   Check {}: error_count = 15.0", i);
        let alerts = monitor.check_metric("error_count", 15.0)?;
        if !alerts.is_empty() {
            println!("   → Alert triggered after {} violations", i);
            break;
        } else {
            println!("   → No alert (violation {} of 3)", i);
        }
        sleep(Duration::from_millis(50)).await;
    }

    // 4. View all alerts
    println!("\n4. Active Alerts");
    let active_alerts = monitor.active_alerts();
    println!("   Total active alerts: {}", active_alerts.len());

    for (i, alert) in active_alerts.iter().enumerate() {
        println!(
            "   {}. {} - {} (severity: {})",
            i + 1,
            alert.threshold.name,
            alert.threshold.metric_name,
            alert.severity().as_str()
        );
    }

    // 5. Acknowledge alerts
    if !active_alerts.is_empty() {
        println!("\n5. Acknowledging Alerts");
        let first_alert = &active_alerts[0];
        monitor.acknowledge_alert(&first_alert.id)?;
        println!("   ✓ Acknowledged alert: {}", first_alert.id);

        let all_alerts = monitor.alerts();
        let acked = all_alerts
            .iter()
            .filter(|a| a.status == AlertStatus::Acknowledged)
            .count();
        println!("   Acknowledged alerts: {}", acked);
    }

    // 6. Resolve alerts
    if active_alerts.len() > 1 {
        println!("\n6. Resolving Alerts");
        let second_alert = &active_alerts[1];
        monitor.resolve_alert(&second_alert.id)?;
        println!("   ✓ Resolved alert: {}", second_alert.id);

        let all_alerts = monitor.alerts();
        let resolved = all_alerts
            .iter()
            .filter(|a| a.status == AlertStatus::Resolved)
            .count();
        println!("   Resolved alerts: {}", resolved);
    }

    // 7. Filter alerts by severity
    println!("\n7. Filtering Alerts by Severity");
    for severity in [
        AlertSeverity::Info,
        AlertSeverity::Warning,
        AlertSeverity::Error,
        AlertSeverity::Critical,
    ] {
        let filtered = monitor.alerts_by_severity(severity);
        println!("   {} alerts: {}", severity.as_str(), filtered.len());
    }

    // 8. View all thresholds
    println!("\n8. All SLA Thresholds");
    let thresholds = monitor.thresholds();
    for threshold in thresholds {
        println!(
            "   - {}: {} {} {} ({})",
            threshold.name,
            threshold.metric_name,
            threshold.comparison.description(),
            threshold.threshold,
            threshold.severity.as_str()
        );
    }

    println!("\n=== Example Complete ===");
    println!("Total alerts: {}", monitor.alerts().len());
    println!("Active alerts: {}", monitor.active_alerts().len());

    Ok(())
}
