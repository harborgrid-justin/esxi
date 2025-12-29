//! # Accessibility Real-time Monitoring System
//!
//! Enterprise-grade real-time accessibility monitoring with WebSocket support,
//! automated scheduling, and comprehensive alerting.
//!
//! ## Features
//!
//! - **Real-time Monitoring**: Live WebSocket updates for scan progress and issues
//! - **Automated Scheduling**: Cron-based scan scheduling with flexible configuration
//! - **Alert Management**: Multi-channel alerting with throttling and conditions
//! - **Metrics Collection**: Prometheus-compatible metrics for observability
//! - **Event-Driven Architecture**: Comprehensive event system for extensibility
//!
//! ## Example
//!
//! ```rust,no_run
//! use accessibility_realtime::{
//!     MonitorEngine, WebSocketServer, ScanScheduler, AlertManager,
//!     EventBus, MetricsCollector, ScanConfig, ScanType,
//! };
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize event bus
//!     let event_bus = EventBus::new(1000);
//!
//!     // Create monitoring engine
//!     let engine = Arc::new(MonitorEngine::new(event_bus.clone()));
//!
//!     // Create WebSocket server
//!     let ws_server = WebSocketServer::new(engine.clone(), event_bus.clone());
//!
//!     // Create scheduler
//!     let scheduler = ScanScheduler::new(engine.clone(), event_bus.clone());
//!     scheduler.start().await?;
//!
//!     // Create alert manager
//!     let alerts = AlertManager::new(event_bus.clone());
//!
//!     // Initialize metrics
//!     MetricsCollector::init_prometheus_exporter(9090)?;
//!
//!     // Start a scan
//!     let config = ScanConfig::default();
//!     let scan_id = engine.start_scan(config).await?;
//!
//!     println!("Scan started: {}", scan_id);
//!
//!     Ok(())
//! }
//! ```

pub mod alerts;
pub mod events;
pub mod metrics;
pub mod monitor;
pub mod scheduler;
pub mod types;
pub mod websocket;

// Re-export main types
pub use alerts::{AlertError, AlertManager};
pub use events::{EventBus, EventHandler, EventProcessor, MonitorEvent};
pub use metrics::{IssueStatistics, MetricsCollector, MetricsError, ScanStatistics};
pub use monitor::{MonitorEngine, MonitorError, ScanContext};
pub use scheduler::{ScanScheduler, SchedulerError};
pub use types::*;
pub use websocket::{WebSocketError, WebSocketServer};

/// Version of the crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize tracing subscriber with default configuration
pub fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "accessibility_realtime=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
