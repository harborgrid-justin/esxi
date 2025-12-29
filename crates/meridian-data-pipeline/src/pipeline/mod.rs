//! Pipeline orchestration module.
//!
//! This module provides the core pipeline abstractions for building and executing
//! data processing workflows.

pub mod builder;
pub mod executor;
pub mod scheduler;

pub use builder::PipelineBuilder;
pub use executor::PipelineExecutor;
pub use scheduler::PipelineScheduler;

use crate::error::{PipelineError, Result};
use crate::monitoring::metrics::PipelineMetrics;
use crate::monitoring::lineage::DataLineage;
use crate::{BatchStats, ExecutionMode, PipelineState};
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// A data processing pipeline.
#[derive(Clone)]
pub struct Pipeline {
    /// Unique pipeline identifier.
    pub id: Uuid,
    /// Pipeline name.
    pub name: String,
    /// Pipeline version.
    pub version: String,
    /// Pipeline configuration.
    pub config: Arc<PipelineConfig>,
    /// Pipeline state.
    pub state: Arc<RwLock<PipelineState>>,
    /// Pipeline metrics.
    pub metrics: Arc<PipelineMetrics>,
    /// Data lineage tracker.
    pub lineage: Arc<RwLock<DataLineage>>,
}

/// Pipeline configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Execution mode.
    pub execution_mode: ExecutionMode,
    /// Number of parallel workers.
    pub parallelism: usize,
    /// Enable checkpointing.
    pub checkpointing: bool,
    /// Checkpoint directory.
    pub checkpoint_dir: Option<String>,
    /// Batch size for micro-batch processing.
    pub batch_size: usize,
    /// Maximum retries on failure.
    pub max_retries: u32,
    /// Timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Enable data validation.
    pub validate_data: bool,
    /// Enable metrics collection.
    pub collect_metrics: bool,
    /// Enable data lineage tracking.
    pub track_lineage: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            execution_mode: ExecutionMode::Batch,
            parallelism: num_cpus::get(),
            checkpointing: false,
            checkpoint_dir: None,
            batch_size: 1000,
            max_retries: 3,
            timeout_secs: None,
            validate_data: true,
            collect_metrics: true,
            track_lineage: true,
        }
    }
}

impl Pipeline {
    /// Create a new pipeline.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            config: Arc::new(PipelineConfig::default()),
            state: Arc::new(RwLock::new(PipelineState::Initializing)),
            metrics: Arc::new(PipelineMetrics::new()),
            lineage: Arc::new(RwLock::new(DataLineage::new())),
        }
    }

    /// Create a new pipeline with custom configuration.
    pub fn with_config(
        name: impl Into<String>,
        version: impl Into<String>,
        config: PipelineConfig,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: version.into(),
            config: Arc::new(config),
            state: Arc::new(RwLock::new(PipelineState::Initializing)),
            metrics: Arc::new(PipelineMetrics::new()),
            lineage: Arc::new(RwLock::new(DataLineage::new())),
        }
    }

    /// Get the current pipeline state.
    pub async fn get_state(&self) -> PipelineState {
        *self.state.read().await
    }

    /// Set the pipeline state.
    pub async fn set_state(&self, new_state: PipelineState) {
        let mut state = self.state.write().await;
        *state = new_state;
    }

    /// Check if the pipeline is running.
    pub async fn is_running(&self) -> bool {
        matches!(*self.state.read().await, PipelineState::Running)
    }

    /// Check if the pipeline is completed.
    pub async fn is_completed(&self) -> bool {
        matches!(*self.state.read().await, PipelineState::Completed)
    }

    /// Check if the pipeline has failed.
    pub async fn is_failed(&self) -> bool {
        matches!(*self.state.read().await, PipelineState::Failed)
    }

    /// Get pipeline statistics.
    pub fn get_stats(&self) -> BatchStats {
        self.metrics.get_stats()
    }

    /// Execute the pipeline (to be implemented by executor).
    pub async fn execute(&self) -> Result<()> {
        let executor = PipelineExecutor::new(self.clone());
        executor.execute().await
    }

    /// Pause the pipeline.
    pub async fn pause(&self) -> Result<()> {
        self.set_state(PipelineState::Paused).await;
        tracing::info!(pipeline = %self.name, "Pipeline paused");
        Ok(())
    }

    /// Resume the pipeline.
    pub async fn resume(&self) -> Result<()> {
        self.set_state(PipelineState::Running).await;
        tracing::info!(pipeline = %self.name, "Pipeline resumed");
        Ok(())
    }

    /// Cancel the pipeline.
    pub async fn cancel(&self) -> Result<()> {
        self.set_state(PipelineState::Cancelled).await;
        tracing::info!(pipeline = %self.name, "Pipeline cancelled");
        Ok(())
    }

    /// Create a checkpoint of the current pipeline state.
    pub async fn checkpoint(&self) -> Result<()> {
        if !self.config.checkpointing {
            return Ok(());
        }

        let checkpoint_dir = self
            .config
            .checkpoint_dir
            .as_ref()
            .ok_or_else(|| PipelineError::Checkpoint("Checkpoint directory not configured".into()))?;

        tracing::info!(
            pipeline = %self.name,
            checkpoint_dir = %checkpoint_dir,
            "Creating checkpoint"
        );

        // Checkpoint logic would be implemented here
        Ok(())
    }

    /// Restore pipeline from checkpoint.
    pub async fn restore_from_checkpoint(&self, checkpoint_id: &str) -> Result<()> {
        if !self.config.checkpointing {
            return Err(PipelineError::Checkpoint(
                "Checkpointing is not enabled".into(),
            ));
        }

        tracing::info!(
            pipeline = %self.name,
            checkpoint_id = %checkpoint_id,
            "Restoring from checkpoint"
        );

        // Restore logic would be implemented here
        Ok(())
    }
}

// Helper to get number of CPUs
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_creation() {
        let pipeline = Pipeline::new("test-pipeline", "1.0.0");
        assert_eq!(pipeline.name, "test-pipeline");
        assert_eq!(pipeline.version, "1.0.0");
        assert_eq!(pipeline.get_state().await, PipelineState::Initializing);
    }

    #[tokio::test]
    async fn test_pipeline_state_transitions() {
        let pipeline = Pipeline::new("test-pipeline", "1.0.0");

        pipeline.set_state(PipelineState::Running).await;
        assert!(pipeline.is_running().await);

        pipeline.pause().await.unwrap();
        assert_eq!(pipeline.get_state().await, PipelineState::Paused);

        pipeline.resume().await.unwrap();
        assert!(pipeline.is_running().await);

        pipeline.cancel().await.unwrap();
        assert_eq!(pipeline.get_state().await, PipelineState::Cancelled);
    }

    #[test]
    fn test_pipeline_config_default() {
        let config = PipelineConfig::default();
        assert_eq!(config.execution_mode, ExecutionMode::Batch);
        assert!(config.parallelism > 0);
        assert_eq!(config.batch_size, 1000);
    }
}
