//! Fluent pipeline builder for constructing data processing pipelines.

use crate::error::{PipelineError, Result};
use crate::pipeline::{Pipeline, PipelineConfig};
use crate::{ExecutionMode, PipelineState};
use std::path::PathBuf;

/// A builder for constructing data processing pipelines.
///
/// Provides a fluent API for configuring and building pipelines.
///
/// # Example
///
/// ```rust,no_run
/// use meridian_data_pipeline::PipelineBuilder;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let pipeline = PipelineBuilder::new("my-pipeline")
///     .version("1.0.0")
///     .execution_mode(meridian_data_pipeline::ExecutionMode::Batch)
///     .with_parallelism(8)
///     .with_checkpointing(true)
///     .checkpoint_dir("/tmp/checkpoints")
///     .batch_size(5000)
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct PipelineBuilder {
    name: String,
    version: String,
    config: PipelineConfig,
}

impl PipelineBuilder {
    /// Create a new pipeline builder with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            config: PipelineConfig::default(),
        }
    }

    /// Set the pipeline version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set the execution mode.
    pub fn execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.config.execution_mode = mode;
        self
    }

    /// Set the number of parallel workers.
    pub fn with_parallelism(mut self, parallelism: usize) -> Self {
        self.config.parallelism = parallelism;
        self
    }

    /// Enable or disable checkpointing.
    pub fn with_checkpointing(mut self, enabled: bool) -> Self {
        self.config.checkpointing = enabled;
        self
    }

    /// Set the checkpoint directory.
    pub fn checkpoint_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.config.checkpoint_dir = Some(dir.into().to_string_lossy().to_string());
        self
    }

    /// Set the batch size for micro-batch processing.
    pub fn batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    /// Set the maximum number of retries on failure.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Set the timeout in seconds.
    pub fn timeout_secs(mut self, timeout: u64) -> Self {
        self.config.timeout_secs = Some(timeout);
        self
    }

    /// Enable or disable data validation.
    pub fn with_validation(mut self, enabled: bool) -> Self {
        self.config.validate_data = enabled;
        self
    }

    /// Enable or disable metrics collection.
    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.config.collect_metrics = enabled;
        self
    }

    /// Enable or disable data lineage tracking.
    pub fn with_lineage(mut self, enabled: bool) -> Self {
        self.config.track_lineage = enabled;
        self
    }

    /// Set streaming mode.
    pub fn streaming(mut self) -> Self {
        self.config.execution_mode = ExecutionMode::Streaming;
        self
    }

    /// Set batch mode.
    pub fn batch(mut self) -> Self {
        self.config.execution_mode = ExecutionMode::Batch;
        self
    }

    /// Set micro-batch mode.
    pub fn micro_batch(mut self) -> Self {
        self.config.execution_mode = ExecutionMode::MicroBatch;
        self
    }

    /// Build the pipeline.
    pub fn build(self) -> Result<Pipeline> {
        // Validate configuration
        if self.name.is_empty() {
            return Err(PipelineError::Execution(
                "Pipeline name cannot be empty".into(),
            ));
        }

        if self.config.parallelism == 0 {
            return Err(PipelineError::Execution(
                "Parallelism must be greater than 0".into(),
            ));
        }

        if self.config.batch_size == 0 {
            return Err(PipelineError::Execution(
                "Batch size must be greater than 0".into(),
            ));
        }

        if self.config.checkpointing && self.config.checkpoint_dir.is_none() {
            return Err(PipelineError::Execution(
                "Checkpoint directory must be specified when checkpointing is enabled".into(),
            ));
        }

        let pipeline = Pipeline::with_config(self.name, self.version, self.config);

        tracing::info!(
            pipeline = %pipeline.name,
            version = %pipeline.version,
            id = %pipeline.id,
            "Pipeline created"
        );

        Ok(pipeline)
    }
}

/// Extended builder methods for source, transform, and sink configuration.
impl PipelineBuilder {
    /// Add a source to the pipeline (placeholder for future implementation).
    #[allow(unused_variables)]
    pub fn source<S>(self, source: S) -> Self {
        // This would store the source in the builder
        // For now, we just return self
        self
    }

    /// Add a transform to the pipeline (placeholder for future implementation).
    #[allow(unused_variables)]
    pub fn transform<T>(self, transform: T) -> Self {
        // This would store the transform in the builder
        // For now, we just return self
        self
    }

    /// Add a sink to the pipeline (placeholder for future implementation).
    #[allow(unused_variables)]
    pub fn sink<S>(self, sink: S) -> Self {
        // This would store the sink in the builder
        // For now, we just return self
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let pipeline = PipelineBuilder::new("test-pipeline")
            .version("1.0.0")
            .build()
            .unwrap();

        assert_eq!(pipeline.name, "test-pipeline");
        assert_eq!(pipeline.version, "1.0.0");
    }

    #[test]
    fn test_builder_with_config() {
        let pipeline = PipelineBuilder::new("test-pipeline")
            .version("2.0.0")
            .with_parallelism(8)
            .batch_size(2000)
            .max_retries(5)
            .timeout_secs(300)
            .with_validation(true)
            .with_metrics(true)
            .with_lineage(true)
            .build()
            .unwrap();

        assert_eq!(pipeline.config.parallelism, 8);
        assert_eq!(pipeline.config.batch_size, 2000);
        assert_eq!(pipeline.config.max_retries, 5);
        assert_eq!(pipeline.config.timeout_secs, Some(300));
        assert!(pipeline.config.validate_data);
        assert!(pipeline.config.collect_metrics);
        assert!(pipeline.config.track_lineage);
    }

    #[test]
    fn test_builder_execution_modes() {
        let pipeline = PipelineBuilder::new("test-pipeline")
            .streaming()
            .build()
            .unwrap();
        assert_eq!(pipeline.config.execution_mode, ExecutionMode::Streaming);

        let pipeline = PipelineBuilder::new("test-pipeline")
            .batch()
            .build()
            .unwrap();
        assert_eq!(pipeline.config.execution_mode, ExecutionMode::Batch);

        let pipeline = PipelineBuilder::new("test-pipeline")
            .micro_batch()
            .build()
            .unwrap();
        assert_eq!(pipeline.config.execution_mode, ExecutionMode::MicroBatch);
    }

    #[test]
    fn test_builder_checkpointing() {
        let pipeline = PipelineBuilder::new("test-pipeline")
            .with_checkpointing(true)
            .checkpoint_dir("/tmp/checkpoints")
            .build()
            .unwrap();

        assert!(pipeline.config.checkpointing);
        assert_eq!(
            pipeline.config.checkpoint_dir.as_deref(),
            Some("/tmp/checkpoints")
        );
    }

    #[test]
    fn test_builder_validation_empty_name() {
        let result = PipelineBuilder::new("")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_validation_zero_parallelism() {
        let result = PipelineBuilder::new("test")
            .with_parallelism(0)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_validation_checkpointing_without_dir() {
        let result = PipelineBuilder::new("test")
            .with_checkpointing(true)
            .build();

        assert!(result.is_err());
    }
}
