//! Model serving infrastructure
//!
//! Provides batch and streaming inference capabilities

pub mod batch;
pub mod stream;

pub use batch::{BatchInferenceEngine, BatchRequest, BatchResponse};
pub use stream::{StreamingInferenceEngine, StreamConfig};

use crate::Result;
use serde::{Deserialize, Serialize};

/// Serving mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServingMode {
    /// Batch inference mode
    Batch,

    /// Streaming inference mode
    Stream,

    /// Hybrid mode (both batch and stream)
    Hybrid,
}

/// Serving configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServingConfig {
    /// Serving mode
    pub mode: ServingMode,

    /// Maximum requests per second
    pub max_rps: usize,

    /// Request timeout in milliseconds
    pub timeout_ms: u64,

    /// Enable request queuing
    pub enable_queuing: bool,

    /// Queue size
    pub queue_size: usize,

    /// Enable autoscaling
    pub enable_autoscaling: bool,

    /// Minimum replicas
    pub min_replicas: usize,

    /// Maximum replicas
    pub max_replicas: usize,
}

impl Default for ServingConfig {
    fn default() -> Self {
        Self {
            mode: ServingMode::Batch,
            max_rps: 1000,
            timeout_ms: 5000,
            enable_queuing: true,
            queue_size: 10000,
            enable_autoscaling: false,
            min_replicas: 1,
            max_replicas: 10,
        }
    }
}

/// Serving statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServingStats {
    /// Total requests served
    pub total_requests: u64,

    /// Total successful requests
    pub successful_requests: u64,

    /// Total failed requests
    pub failed_requests: u64,

    /// Average latency in milliseconds
    pub avg_latency_ms: f64,

    /// P95 latency in milliseconds
    pub p95_latency_ms: f64,

    /// P99 latency in milliseconds
    pub p99_latency_ms: f64,

    /// Current requests per second
    pub current_rps: f64,

    /// Current queue depth
    pub queue_depth: usize,

    /// Active replicas
    pub active_replicas: usize,
}

impl Default for ServingStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            p99_latency_ms: 0.0,
            current_rps: 0.0,
            queue_depth: 0,
            active_replicas: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serving_mode_variants() {
        assert_eq!(ServingMode::Batch, ServingMode::Batch);
        assert_ne!(ServingMode::Batch, ServingMode::Stream);
    }

    #[test]
    fn test_serving_config_default() {
        let config = ServingConfig::default();
        assert_eq!(config.mode, ServingMode::Batch);
        assert_eq!(config.max_rps, 1000);
        assert!(config.enable_queuing);
    }

    #[test]
    fn test_serving_stats_default() {
        let stats = ServingStats::default();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.active_replicas, 1);
    }
}
