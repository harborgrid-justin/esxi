//! Integration tests for the Meridian Metrics system.

use meridian_metrics::{
    AlertSeverity, AggregationFunc, GisMetricType, MetricsCollector, MetricsSystem, RollupRule,
    SlaThreshold, ThresholdComparison, TimeWindow,
};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_metrics_system_initialization() {
    let system = MetricsSystem::new().unwrap();
    assert_eq!(system.collector.metrics_count(), 0);

    system.register_default_gis_metrics().unwrap();
    assert!(system.collector.metrics_count() > 0);
}

#[tokio::test]
async fn test_counter_gauge_histogram() {
    let collector = Arc::new(MetricsCollector::default());

    // Counter
    let counter = collector.register_counter("test_counter", "Test").unwrap();
    counter.inc();
    counter.add(5);
    assert_eq!(counter.get(), 6);

    // Gauge
    let gauge = collector.register_gauge("test_gauge", "Test").unwrap();
    gauge.set(10.0);
    gauge.add(5.0);
    assert_eq!(gauge.get(), 15.0);

    // Histogram
    let histogram = collector
        .register_histogram("test_histogram", "Test")
        .unwrap();
    for i in 1..=100 {
        histogram.observe(i).unwrap();
    }
    assert_eq!(histogram.count(), 100);
    assert!(histogram.mean() > 49.0 && histogram.mean() < 51.0);
}

#[tokio::test]
async fn test_gis_metrics() {
    let collector = Arc::new(MetricsCollector::default());

    // Query latency
    collector
        .record_gis_metric(GisMetricType::QueryLatency(123.45))
        .unwrap();

    // Tile rendering
    collector
        .record_gis_metric(GisMetricType::TileRenderTime {
            zoom: 12,
            x: 1234,
            y: 5678,
            duration_ms: 45.2,
        })
        .unwrap();

    // Spatial operation
    collector
        .record_gis_metric(GisMetricType::SpatialOperation {
            operation: "intersection".to_string(),
            geometry_count: 1000,
            duration_ms: 234.5,
        })
        .unwrap();

    let metrics = collector.get_gis_metrics();
    assert_eq!(metrics.len(), 3);
}

#[tokio::test]
async fn test_sla_monitoring() {
    let system = MetricsSystem::new().unwrap();

    let threshold = SlaThreshold::new(
        "test_sla",
        "test_metric",
        100.0,
        ThresholdComparison::GreaterThan,
    )
    .with_severity(AlertSeverity::Warning);

    system.sla.add_threshold(threshold);

    // Should not violate
    let alerts = system.sla.check_metric("test_metric", 50.0).unwrap();
    assert_eq!(alerts.len(), 0);

    // Should violate
    let alerts = system.sla.check_metric("test_metric", 150.0).unwrap();
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].threshold.name, "test_sla");
    assert_eq!(alerts[0].actual_value, 150.0);
}

#[tokio::test]
async fn test_health_checks() {
    let system = MetricsSystem::new().unwrap();

    let report = system.health.check().await.unwrap();

    assert!(report.components.len() > 0);
    assert!(report.uptime_secs >= 0);
    assert!(report.system.cpu_cores > 0);
}

#[tokio::test]
async fn test_metric_aggregation() {
    let system = MetricsSystem::new().unwrap();

    let rule = RollupRule {
        name: "test_rule".to_string(),
        metric_pattern: "*".to_string(),
        function: AggregationFunc::Average,
        window: TimeWindow::Minute,
        retention_secs: 3600,
        enabled: true,
    };

    system.aggregator.add_rule(rule);

    let gauge = system.collector.register_gauge("test_metric", "Test").unwrap();

    // Record some values
    for i in 0..10 {
        gauge.set(i as f64);

        let snapshot = meridian_metrics::MetricSnapshot {
            name: "test_metric".to_string(),
            help: "Test".to_string(),
            labels: HashMap::new(),
            value: meridian_metrics::MetricValue::Gauge { value: i as f64 },
            timestamp: chrono::Utc::now(),
        };

        system.aggregator.ingest(&snapshot).unwrap();
    }

    // Wait a bit
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_labels_and_dimensions() {
    let collector = Arc::new(MetricsCollector::default());

    let mut labels = HashMap::new();
    labels.insert("tenant".to_string(), "tenant1".to_string());
    labels.insert("region".to_string(), "us-west".to_string());

    let counter = collector
        .register_counter_with_labels("requests", "Requests", labels.clone())
        .unwrap();

    counter.add(100);

    let snapshots = collector.collect_snapshots();
    let tenant_snapshot = snapshots
        .iter()
        .find(|s| s.labels.get("tenant") == Some(&"tenant1".to_string()));

    assert!(tenant_snapshot.is_some());
}

#[tokio::test]
async fn test_metric_snapshots() {
    let collector = Arc::new(MetricsCollector::default());

    // Create various metrics
    let counter = collector.register_counter("counter", "Test").unwrap();
    counter.add(42);

    let gauge = collector.register_gauge("gauge", "Test").unwrap();
    gauge.set(3.14);

    let histogram = collector.register_histogram("histogram", "Test").unwrap();
    histogram.observe(100).unwrap();

    let snapshots = collector.collect_snapshots();
    assert_eq!(snapshots.len(), 3);

    // Verify snapshot types
    for snapshot in snapshots {
        match snapshot.value {
            meridian_metrics::MetricValue::Counter { value } => {
                assert_eq!(snapshot.name, "counter");
                assert_eq!(value, 42);
            }
            meridian_metrics::MetricValue::Gauge { value } => {
                assert_eq!(snapshot.name, "gauge");
                assert!((value - 3.14).abs() < 0.001);
            }
            meridian_metrics::MetricValue::Histogram { .. } => {
                assert_eq!(snapshot.name, "histogram");
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn test_alert_acknowledgement() {
    let system = MetricsSystem::new().unwrap();

    let threshold = SlaThreshold::new(
        "test",
        "metric",
        10.0,
        ThresholdComparison::GreaterThan,
    );

    system.sla.add_threshold(threshold);

    let alerts = system.sla.check_metric("metric", 20.0).unwrap();
    assert_eq!(alerts.len(), 1);

    let alert_id = &alerts[0].id;
    system.sla.acknowledge_alert(alert_id).unwrap();

    let alert = system
        .sla
        .alerts()
        .into_iter()
        .find(|a| &a.id == alert_id)
        .unwrap();
    assert_eq!(
        alert.status,
        meridian_metrics::AlertStatus::Acknowledged
    );
}

#[test]
fn test_aggregation_functions() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];

    assert_eq!(AggregationFunc::Sum.apply(&values), 15.0);
    assert_eq!(AggregationFunc::Average.apply(&values), 3.0);
    assert_eq!(AggregationFunc::Min.apply(&values), 1.0);
    assert_eq!(AggregationFunc::Max.apply(&values), 5.0);
    assert_eq!(AggregationFunc::Count.apply(&values), 5.0);
    assert_eq!(AggregationFunc::Median.apply(&values), 3.0);
}

#[test]
fn test_time_windows() {
    assert_eq!(TimeWindow::Minute.seconds(), 60);
    assert_eq!(TimeWindow::FiveMinutes.seconds(), 300);
    assert_eq!(TimeWindow::Hour.seconds(), 3600);
    assert_eq!(TimeWindow::Day.seconds(), 86400);
}

#[test]
fn test_threshold_comparison() {
    assert!(ThresholdComparison::GreaterThan.evaluate(10.0, 5.0));
    assert!(!ThresholdComparison::GreaterThan.evaluate(5.0, 10.0));

    assert!(ThresholdComparison::LessThan.evaluate(5.0, 10.0));
    assert!(!ThresholdComparison::LessThan.evaluate(10.0, 5.0));

    assert!(ThresholdComparison::Equal.evaluate(5.0, 5.0));
    assert!(!ThresholdComparison::NotEqual.evaluate(5.0, 5.0));
}
