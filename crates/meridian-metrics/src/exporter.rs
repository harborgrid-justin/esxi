//! Metric exporters for Prometheus and OpenTelemetry.

use crate::collector::MetricsCollector;
use crate::error::{MetricsError, Result};
use crate::types::MetricSnapshot;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use opentelemetry::{
    global,
    metrics::{Meter, MeterProvider},
    KeyValue,
};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::{
    metrics::{
        reader::{DefaultAggregationSelector, DefaultTemporalitySelector},
        PeriodicReader, SdkMeterProvider,
    },
    Resource,
};
use prometheus::{Encoder, Registry, TextEncoder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tracing::{error, info};

/// Exporter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExporterConfig {
    /// Enable Prometheus exporter
    pub prometheus_enabled: bool,

    /// Prometheus HTTP port
    pub prometheus_port: u16,

    /// Enable OpenTelemetry exporter
    pub otlp_enabled: bool,

    /// OTLP endpoint
    pub otlp_endpoint: String,

    /// Export interval in seconds
    pub export_interval_secs: u64,

    /// Timeout for exports in seconds
    pub timeout_secs: u64,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            prometheus_enabled: true,
            prometheus_port: 9090,
            otlp_enabled: false,
            otlp_endpoint: "http://localhost:4317".to_string(),
            export_interval_secs: 15,
            timeout_secs: 10,
        }
    }
}

/// Prometheus exporter
pub struct PrometheusExporter {
    registry: Registry,
    collector: Arc<MetricsCollector>,
}

impl PrometheusExporter {
    /// Create a new Prometheus exporter
    pub fn new(collector: Arc<MetricsCollector>) -> Result<Self> {
        let registry = Registry::new();

        info!("Prometheus exporter initialized");

        Ok(Self {
            registry,
            collector,
        })
    }

    /// Export metrics in Prometheus format
    pub fn export(&self) -> Result<String> {
        // Collect snapshots from the collector
        let snapshots = self.collector.collect_snapshots();

        // Build Prometheus text format
        let mut output = String::new();

        for snapshot in snapshots {
            let metric_name = sanitize_metric_name(&snapshot.name);
            let labels = format_labels(&snapshot.labels);

            // Add HELP line
            output.push_str(&format!("# HELP {} {}\n", metric_name, snapshot.help));

            // Add TYPE line
            let metric_type = match snapshot.value {
                crate::types::MetricValue::Counter { .. } => "counter",
                crate::types::MetricValue::Gauge { .. } => "gauge",
                crate::types::MetricValue::Histogram { .. } => "histogram",
                crate::types::MetricValue::Summary { .. } => "summary",
            };
            output.push_str(&format!("# TYPE {} {}\n", metric_name, metric_type));

            // Add metric value(s)
            match snapshot.value {
                crate::types::MetricValue::Counter { value } => {
                    output.push_str(&format!("{}{} {}\n", metric_name, labels, value));
                }
                crate::types::MetricValue::Gauge { value } => {
                    output.push_str(&format!("{}{} {}\n", metric_name, labels, value));
                }
                crate::types::MetricValue::Histogram { stats } => {
                    output.push_str(&format!("{}_sum{} {}\n", metric_name, labels, stats.sum));
                    output.push_str(&format!("{}_count{} {}\n", metric_name, labels, stats.count));
                    output.push_str(&format!("{}_min{} {}\n", metric_name, labels, stats.min));
                    output.push_str(&format!("{}_max{} {}\n", metric_name, labels, stats.max));
                    output.push_str(&format!(
                        "{}{{quantile=\"0.5\"{}}} {}\n",
                        metric_name,
                        format_labels_with_quantile(&snapshot.labels, 0.5),
                        stats.p50
                    ));
                    output.push_str(&format!(
                        "{}{{quantile=\"0.9\"{}}} {}\n",
                        metric_name,
                        format_labels_with_quantile(&snapshot.labels, 0.9),
                        stats.p90
                    ));
                    output.push_str(&format!(
                        "{}{{quantile=\"0.95\"{}}} {}\n",
                        metric_name,
                        format_labels_with_quantile(&snapshot.labels, 0.95),
                        stats.p95
                    ));
                    output.push_str(&format!(
                        "{}{{quantile=\"0.99\"{}}} {}\n",
                        metric_name,
                        format_labels_with_quantile(&snapshot.labels, 0.99),
                        stats.p99
                    ));
                }
                crate::types::MetricValue::Summary { stats } => {
                    output.push_str(&format!("{}_sum{} {}\n", metric_name, labels, stats.sum));
                    output.push_str(&format!("{}_count{} {}\n", metric_name, labels, stats.count));
                    output.push_str(&format!(
                        "{}{{quantile=\"0.5\"{}}} {}\n",
                        metric_name,
                        format_labels_with_quantile(&snapshot.labels, 0.5),
                        stats.p50
                    ));
                    output.push_str(&format!(
                        "{}{{quantile=\"0.9\"{}}} {}\n",
                        metric_name,
                        format_labels_with_quantile(&snapshot.labels, 0.9),
                        stats.p90
                    ));
                    output.push_str(&format!(
                        "{}{{quantile=\"0.99\"{}}} {}\n",
                        metric_name,
                        format_labels_with_quantile(&snapshot.labels, 0.99),
                        stats.p99
                    ));
                }
            }

            output.push('\n');
        }

        Ok(output)
    }

    /// Get the registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

/// OpenTelemetry exporter
pub struct OtlpExporter {
    meter: Meter,
    collector: Arc<MetricsCollector>,
}

impl OtlpExporter {
    /// Create a new OTLP exporter
    pub fn new(config: &ExporterConfig, collector: Arc<MetricsCollector>) -> Result<Self> {
        let export_config = ExportConfig {
            endpoint: config.otlp_endpoint.clone(),
            timeout: Duration::from_secs(config.timeout_secs),
            ..Default::default()
        };

        let exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_export_config(export_config);

        let reader = PeriodicReader::builder(
            exporter.build_metrics_exporter(
                Box::new(DefaultAggregationSelector::new()),
                Box::new(DefaultTemporalitySelector::new()),
            )
            .map_err(|e| MetricsError::OpenTelemetry(e.to_string()))?,
            opentelemetry_sdk::runtime::Tokio,
        )
        .with_interval(Duration::from_secs(config.export_interval_secs))
        .build();

        let provider = SdkMeterProvider::builder()
            .with_reader(reader)
            .with_resource(Resource::new(vec![
                KeyValue::new("service.name", "meridian-gis"),
                KeyValue::new("service.version", "0.1.5"),
            ]))
            .build();

        let meter = provider.meter("meridian-metrics");

        global::set_meter_provider(provider);

        info!("OpenTelemetry OTLP exporter initialized");

        Ok(Self { meter, collector })
    }

    /// Export metrics via OTLP
    pub async fn export(&self) -> Result<()> {
        let snapshots = self.collector.collect_snapshots();

        for snapshot in snapshots {
            let labels: Vec<KeyValue> = snapshot
                .labels
                .iter()
                .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
                .collect();

            match snapshot.value {
                crate::types::MetricValue::Counter { value } => {
                    let counter = self.meter.u64_counter(snapshot.name).init();
                    counter.add(value, &labels);
                }
                crate::types::MetricValue::Gauge { value } => {
                    let gauge = self.meter.f64_up_down_counter(snapshot.name).init();
                    gauge.add(value, &labels);
                }
                crate::types::MetricValue::Histogram { stats } => {
                    let histogram = self.meter.f64_histogram(snapshot.name).init();
                    // Record mean for simplicity
                    histogram.record(stats.mean, &labels);
                }
                crate::types::MetricValue::Summary { stats } => {
                    let histogram = self.meter.f64_histogram(snapshot.name).init();
                    histogram.record(stats.mean, &labels);
                }
            }
        }

        Ok(())
    }

    /// Get the meter
    pub fn meter(&self) -> &Meter {
        &self.meter
    }
}

/// Combined exporter manager
pub struct ExporterManager {
    config: ExporterConfig,
    prometheus: Option<Arc<PrometheusExporter>>,
    otlp: Option<Arc<OtlpExporter>>,
    collector: Arc<MetricsCollector>,
}

impl ExporterManager {
    /// Create a new exporter manager
    pub fn new(config: ExporterConfig, collector: Arc<MetricsCollector>) -> Result<Self> {
        let prometheus = if config.prometheus_enabled {
            Some(Arc::new(PrometheusExporter::new(Arc::clone(&collector))?))
        } else {
            None
        };

        let otlp = if config.otlp_enabled {
            Some(Arc::new(OtlpExporter::new(&config, Arc::clone(&collector))?))
        } else {
            None
        };

        Ok(Self {
            config,
            prometheus,
            otlp,
            collector,
        })
    }

    /// Start the Prometheus HTTP server
    pub async fn start_prometheus_server(&self) -> Result<()> {
        if let Some(prometheus) = &self.prometheus {
            let app = Router::new()
                .route("/metrics", get(metrics_handler))
                .with_state(Arc::clone(prometheus));

            let addr = format!("0.0.0.0:{}", self.config.prometheus_port);
            let listener = TcpListener::bind(&addr)
                .await
                .map_err(|e| MetricsError::export(format!("Failed to bind to {}: {}", addr, e)))?;

            info!("Prometheus metrics server listening on {}", addr);

            tokio::spawn(async move {
                if let Err(e) = axum::serve(listener, app).await {
                    error!("Prometheus server error: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Start background export tasks
    pub async fn start_exporters(&self) -> Result<()> {
        // Start Prometheus server
        if self.config.prometheus_enabled {
            self.start_prometheus_server().await?;
        }

        // Start OTLP exporter task
        if let Some(otlp) = &self.otlp {
            let otlp = Arc::clone(otlp);
            let interval = Duration::from_secs(self.config.export_interval_secs);

            tokio::spawn(async move {
                let mut ticker = tokio::time::interval(interval);
                loop {
                    ticker.tick().await;
                    if let Err(e) = otlp.export().await {
                        error!("OTLP export error: {}", e);
                    }
                }
            });

            info!("OTLP exporter started");
        }

        Ok(())
    }

    /// Get Prometheus exporter
    pub fn prometheus(&self) -> Option<&Arc<PrometheusExporter>> {
        self.prometheus.as_ref()
    }

    /// Get OTLP exporter
    pub fn otlp(&self) -> Option<&Arc<OtlpExporter>> {
        self.otlp.as_ref()
    }
}

/// HTTP handler for Prometheus metrics endpoint
async fn metrics_handler(
    State(exporter): State<Arc<PrometheusExporter>>,
) -> std::result::Result<Response, StatusCode> {
    exporter
        .export()
        .map(|metrics| (StatusCode::OK, metrics).into_response())
        .map_err(|e| {
            error!("Failed to export metrics: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// Sanitize metric name for Prometheus
fn sanitize_metric_name(name: &str) -> String {
    name.replace('-', "_").replace('.', "_").replace(' ', "_")
}

/// Format labels for Prometheus
fn format_labels(labels: &std::collections::HashMap<String, String>) -> String {
    if labels.is_empty() {
        return String::new();
    }

    let mut parts: Vec<String> = labels
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, v))
        .collect();
    parts.sort();

    format!("{{{}}}", parts.join(","))
}

/// Format labels with quantile for Prometheus
fn format_labels_with_quantile(
    labels: &std::collections::HashMap<String, String>,
    quantile: f64,
) -> String {
    let mut all_labels = labels.clone();
    all_labels.insert("quantile".to_string(), quantile.to_string());
    format_labels(&all_labels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_metric_name() {
        assert_eq!(sanitize_metric_name("test-metric"), "test_metric");
        assert_eq!(sanitize_metric_name("test.metric"), "test_metric");
        assert_eq!(sanitize_metric_name("test metric"), "test_metric");
    }

    #[test]
    fn test_format_labels() {
        let mut labels = std::collections::HashMap::new();
        labels.insert("key1".to_string(), "value1".to_string());
        labels.insert("key2".to_string(), "value2".to_string());

        let formatted = format_labels(&labels);
        assert!(formatted.contains("key1=\"value1\""));
        assert!(formatted.contains("key2=\"value2\""));
    }

    #[test]
    fn test_prometheus_exporter() {
        let collector = Arc::new(MetricsCollector::default());
        let counter = collector.register_counter("test_counter", "Test").unwrap();
        counter.add(5);

        let exporter = PrometheusExporter::new(collector).unwrap();
        let output = exporter.export().unwrap();

        assert!(output.contains("test_counter"));
        assert!(output.contains("5"));
    }
}
