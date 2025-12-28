//! # Meridian Metrics
//!
//! Enterprise metrics and telemetry system for the Meridian GIS Platform.
//!
//! ## Features
//!
//! - **OpenTelemetry Integration**: Distributed tracing with OTLP export
//! - **Prometheus Exporter**: Industry-standard metrics format
//! - **Custom GIS Metrics**: Query latency, tile rendering, spatial operations
//! - **Health Checks**: Detailed system status with component monitoring
//! - **Performance Profiling**: Flamegraph support for CPU profiling
//! - **SLA Monitoring**: Threshold-based alerting system
//! - **Real-time Streaming**: WebSocket-based metrics streaming
//! - **Metric Aggregation**: Automatic rollup and aggregation strategies
//! - **Rich Metric Types**: Counters, Gauges, Histograms, and Summaries
//! - **Multi-tenant Support**: Label-based filtering and dimensions
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use meridian_metrics::{
//!     MetricsCollector,
//!     ExporterManager, ExporterConfig,
//!     HealthCheckSystem,
//!     StreamingServer,
//! };
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the metrics collector
//!     let collector = Arc::new(MetricsCollector::default());
//!
//!     // Register a counter
//!     let requests = collector.register_counter("http_requests_total", "Total HTTP requests")?;
//!     requests.inc();
//!
//!     // Register a histogram
//!     let latency = collector.register_histogram("query_latency_ms", "Query latency")?;
//!     latency.observe(123)?;
//!
//!     // Start exporters (Prometheus + OTLP)
//!     let exporter_config = ExporterConfig::default();
//!     let exporters = ExporterManager::new(exporter_config, Arc::clone(&collector))?;
//!     exporters.start_exporters().await?;
//!
//!     // Start health checks
//!     let health = Arc::new(HealthCheckSystem::default());
//!     health.start_background_checks().await;
//!
//!     // Start streaming server
//!     let streaming = Arc::new(StreamingServer::default(Arc::clone(&collector)));
//!     streaming.start().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## GIS-Specific Metrics
//!
//! ```rust,no_run
//! use meridian_metrics::{MetricsCollector, GisMetricType};
//! use std::sync::Arc;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let collector = Arc::new(MetricsCollector::default());
//!
//! // Record tile rendering time
//! collector.record_gis_metric(GisMetricType::TileRenderTime {
//!     zoom: 12,
//!     x: 1234,
//!     y: 5678,
//!     duration_ms: 45.2,
//! })?;
//!
//! // Record spatial operation
//! collector.record_gis_metric(GisMetricType::SpatialOperation {
//!     operation: "intersection".to_string(),
//!     geometry_count: 1000,
//!     duration_ms: 234.5,
//! })?;
//! # Ok(())
//! # }
//! ```
//!
//! ## SLA Monitoring
//!
//! ```rust,no_run
//! use meridian_metrics::{
//!     SlaMonitor, SlaThreshold, ThresholdComparison, AlertSeverity,
//! };
//! use std::sync::Arc;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let monitor = Arc::new(SlaMonitor::default());
//!
//! // Define an SLA threshold
//! let threshold = SlaThreshold::new(
//!     "query_latency_sla",
//!     "query_latency_ms",
//!     100.0,
//!     ThresholdComparison::GreaterThan,
//! )
//! .with_description("Query latency must be under 100ms")
//! .with_severity(AlertSeverity::Warning);
//!
//! monitor.add_threshold(threshold);
//!
//! // Check metrics
//! let alerts = monitor.check_metric("query_latency_ms", 150.0)?;
//!
//! // Start alert processor
//! monitor.start_alert_processor().await;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod aggregator;
pub mod collector;
pub mod error;
pub mod exporter;
pub mod health;
pub mod profiler;
pub mod sla;
pub mod streaming;
pub mod types;

// Re-export commonly used types
pub use aggregator::{
    AggregatedPoint, AggregationFunc, MetricsAggregator, RollupRule, TimeWindow,
};
pub use collector::{CollectorConfig, MetricsCollector};
pub use error::{MetricsError, Result};
pub use exporter::{ExporterConfig, ExporterManager, OtlpExporter, PrometheusExporter};
pub use health::{
    ComponentHealth, HealthCheckConfig, HealthCheckSystem,
    HealthReport, HealthStatus,
};
pub use profiler::{ProfileSession, Profiler, ProfilerConfig, ScopedProfile, Timer};
pub use sla::{
    AlertSeverity, AlertStatus, SlaAlert, SlaMonitor, SlaMonitorConfig, SlaThreshold,
    ThresholdComparison,
};
pub use streaming::{StreamMessage, StreamingConfig, StreamingServer, SubscriptionFilter};
pub use types::{
    Counter, Gauge, GisMetricType, HistogramMetric, Labels, MetricSnapshot, MetricValue, Summary,
    SummaryStats,
};

use std::sync::Arc;
use tracing::info;

/// Complete metrics system with all components
pub struct MetricsSystem {
    /// Metrics collector
    pub collector: Arc<MetricsCollector>,

    /// Exporter manager
    pub exporters: Arc<ExporterManager>,

    /// Health check system
    pub health: Arc<HealthCheckSystem>,

    /// SLA monitor
    pub sla: Arc<SlaMonitor>,

    /// Streaming server
    pub streaming: Arc<StreamingServer>,

    /// Metrics aggregator
    pub aggregator: Arc<MetricsAggregator>,

    /// Profiler
    pub profiler: Arc<Profiler>,
}

impl MetricsSystem {
    /// Create a new metrics system with default configuration
    pub fn new() -> Result<Self> {
        let collector = Arc::new(MetricsCollector::default());
        let exporters = Arc::new(ExporterManager::new(
            ExporterConfig::default(),
            Arc::clone(&collector),
        )?);
        let health = Arc::new(HealthCheckSystem::default());
        let sla = Arc::new(SlaMonitor::default());
        let streaming = Arc::new(StreamingServer::default(Arc::clone(&collector)));
        let aggregator = Arc::new(MetricsAggregator::default());
        let profiler = Arc::new(Profiler::default()?);

        info!("Meridian Metrics System initialized");

        Ok(Self {
            collector,
            exporters,
            health,
            sla,
            streaming,
            aggregator,
            profiler,
        })
    }

    /// Create a metrics system with custom configuration
    pub fn with_config(
        collector_config: CollectorConfig,
        exporter_config: ExporterConfig,
        health_config: HealthCheckConfig,
        sla_config: SlaMonitorConfig,
        streaming_config: StreamingConfig,
        profiler_config: ProfilerConfig,
    ) -> Result<Self> {
        let collector = Arc::new(MetricsCollector::new(collector_config));
        let exporters = Arc::new(ExporterManager::new(
            exporter_config,
            Arc::clone(&collector),
        )?);
        let health = Arc::new(HealthCheckSystem::new(health_config));
        let sla = Arc::new(SlaMonitor::new(sla_config));
        let streaming = Arc::new(StreamingServer::new(
            streaming_config,
            Arc::clone(&collector),
        ));
        let aggregator = Arc::new(MetricsAggregator::default());
        let profiler = Arc::new(Profiler::new(profiler_config)?);

        info!("Meridian Metrics System initialized with custom config");

        Ok(Self {
            collector,
            exporters,
            health,
            sla,
            streaming,
            aggregator,
            profiler,
        })
    }

    /// Start all background services
    pub async fn start_all(&self) -> Result<()> {
        info!("Starting all metrics system services");

        // Start exporters
        self.exporters.start_exporters().await?;

        // Start health checks
        Arc::clone(&self.health).start_background_checks().await;

        // Start SLA alert processor
        Arc::clone(&self.sla).start_alert_processor().await;

        // Start streaming server
        Arc::clone(&self.streaming).start().await?;

        // Start aggregation
        Arc::clone(&self.aggregator).start_background_aggregation().await;

        info!("All metrics system services started");

        Ok(())
    }

    /// Register default GIS metrics
    pub fn register_default_gis_metrics(&self) -> Result<()> {
        info!("Registering default GIS metrics");

        // Query metrics
        self.collector
            .register_histogram("gis_query_latency_ms", "GIS query execution latency in milliseconds")?;
        self.collector
            .register_counter("gis_queries_total", "Total number of GIS queries executed")?;
        self.collector
            .register_counter("gis_query_errors_total", "Total number of GIS query errors")?;

        // Tile rendering metrics
        self.collector
            .register_histogram("gis_tile_render_ms", "Tile rendering duration in milliseconds")?;
        self.collector
            .register_counter("gis_tiles_rendered_total", "Total number of tiles rendered")?;
        self.collector
            .register_gauge("gis_tile_cache_size", "Current tile cache size in bytes")?;

        // Spatial operation metrics
        self.collector.register_histogram(
            "gis_spatial_op_duration_ms",
            "Spatial operation duration in milliseconds",
        )?;
        self.collector
            .register_counter("gis_spatial_ops_total", "Total number of spatial operations")?;

        // Data loading metrics
        self.collector
            .register_histogram("gis_data_load_ms", "Data loading duration in milliseconds")?;
        self.collector
            .register_gauge("gis_data_size_bytes", "Size of loaded GIS data in bytes")?;

        // Cache metrics
        self.collector
            .register_counter("gis_cache_hits_total", "Total number of cache hits")?;
        self.collector
            .register_counter("gis_cache_misses_total", "Total number of cache misses")?;

        // Connection pool metrics
        self.collector
            .register_gauge("gis_db_connections_active", "Number of active database connections")?;
        self.collector
            .register_gauge("gis_db_connections_idle", "Number of idle database connections")?;

        info!("Default GIS metrics registered");

        Ok(())
    }

    /// Register default SLA thresholds
    pub fn register_default_sla_thresholds(&self) {
        info!("Registering default SLA thresholds");

        // Query latency SLA
        self.sla.add_threshold(
            SlaThreshold::new(
                "query_latency_sla",
                "gis_query_latency_ms",
                100.0,
                ThresholdComparison::GreaterThan,
            )
            .with_description("Query latency should be under 100ms")
            .with_severity(AlertSeverity::Warning)
            .with_window(60),
        );

        // Tile render time SLA
        self.sla.add_threshold(
            SlaThreshold::new(
                "tile_render_sla",
                "gis_tile_render_ms",
                200.0,
                ThresholdComparison::GreaterThan,
            )
            .with_description("Tile rendering should be under 200ms")
            .with_severity(AlertSeverity::Warning)
            .with_window(60),
        );

        // Error rate SLA
        self.sla.add_threshold(
            SlaThreshold::new(
                "error_rate_sla",
                "gis_query_errors_total",
                10.0,
                ThresholdComparison::GreaterThan,
            )
            .with_description("Query errors should be minimal")
            .with_severity(AlertSeverity::Error)
            .with_window(300)
            .with_min_violations(3),
        );

        info!("Default SLA thresholds registered");
    }

    /// Register default rollup rules
    pub fn register_default_rollup_rules(&self) {
        info!("Registering default rollup rules");

        // 1-minute average for all metrics
        self.aggregator.add_rule(RollupRule {
            name: "1m_avg".to_string(),
            metric_pattern: "*".to_string(),
            function: AggregationFunc::Average,
            window: TimeWindow::Minute,
            retention_secs: 3600,
            enabled: true,
        });

        // 5-minute average for all metrics
        self.aggregator.add_rule(RollupRule {
            name: "5m_avg".to_string(),
            metric_pattern: "*".to_string(),
            function: AggregationFunc::Average,
            window: TimeWindow::FiveMinutes,
            retention_secs: 86400,
            enabled: true,
        });

        // Hourly max for latency metrics
        self.aggregator.add_rule(RollupRule {
            name: "1h_max_latency".to_string(),
            metric_pattern: "*_latency_*".to_string(),
            function: AggregationFunc::Max,
            window: TimeWindow::Hour,
            retention_secs: 86400 * 7,
            enabled: true,
        });

        // Daily P95 for latency metrics
        self.aggregator.add_rule(RollupRule {
            name: "1d_p95_latency".to_string(),
            metric_pattern: "*_latency_*".to_string(),
            function: AggregationFunc::P95,
            window: TimeWindow::Day,
            retention_secs: 86400 * 30,
            enabled: true,
        });

        info!("Default rollup rules registered");
    }

    /// Initialize with all defaults
    pub fn init_with_defaults() -> Result<Self> {
        let system = Self::new()?;

        system.register_default_gis_metrics()?;
        system.register_default_sla_thresholds();
        system.register_default_rollup_rules();

        Ok(system)
    }
}

impl Default for MetricsSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create default MetricsSystem")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_system_creation() {
        let system = MetricsSystem::new().unwrap();
        assert_eq!(system.collector.metrics_count(), 0);
    }

    #[test]
    fn test_default_gis_metrics() {
        let system = MetricsSystem::new().unwrap();
        system.register_default_gis_metrics().unwrap();
        assert!(system.collector.metrics_count() > 0);
    }

    #[tokio::test]
    async fn test_metrics_system_startup() {
        let system = MetricsSystem::init_with_defaults().unwrap();

        // Don't actually start services in tests
        // system.start_all().await.unwrap();

        assert!(system.collector.metrics_count() > 0);
    }
}
