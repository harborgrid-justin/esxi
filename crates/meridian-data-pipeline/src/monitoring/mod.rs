//! Pipeline monitoring and observability.
//!
//! Provides metrics collection, performance tracking, and data lineage tracking.

pub mod lineage;
pub mod metrics;

pub use lineage::DataLineage;
pub use metrics::PipelineMetrics;
