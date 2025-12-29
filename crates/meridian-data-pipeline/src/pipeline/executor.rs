//! Pipeline executor for running data processing workflows.

use crate::error::{PipelineError, Result};
use crate::pipeline::Pipeline;
use crate::{BatchStats, ExecutionMode, PipelineState};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::timeout;
use tracing::{error, info, warn};

/// Executes data processing pipelines.
pub struct PipelineExecutor {
    pipeline: Pipeline,
}

impl PipelineExecutor {
    /// Create a new pipeline executor.
    pub fn new(pipeline: Pipeline) -> Self {
        Self { pipeline }
    }

    /// Execute the pipeline.
    pub async fn execute(&self) -> Result<()> {
        info!(
            pipeline = %self.pipeline.name,
            version = %self.pipeline.version,
            mode = ?self.pipeline.config.execution_mode,
            "Starting pipeline execution"
        );

        // Set state to running
        self.pipeline.set_state(PipelineState::Running).await;

        let start_time = Instant::now();
        let mut result = Ok(());

        // Execute based on mode with timeout if configured
        let execution_future = match self.pipeline.config.execution_mode {
            ExecutionMode::Batch => self.execute_batch(),
            ExecutionMode::Streaming => self.execute_streaming(),
            ExecutionMode::MicroBatch => self.execute_micro_batch(),
        };

        // Apply timeout if configured
        if let Some(timeout_secs) = self.pipeline.config.timeout_secs {
            match timeout(Duration::from_secs(timeout_secs), execution_future).await {
                Ok(exec_result) => result = exec_result,
                Err(_) => {
                    result = Err(PipelineError::Timeout(format!(
                        "Pipeline execution exceeded timeout of {} seconds",
                        timeout_secs
                    )));
                }
            }
        } else {
            result = execution_future.await;
        }

        let duration = start_time.elapsed();

        // Update final state based on result
        match &result {
            Ok(_) => {
                self.pipeline.set_state(PipelineState::Completed).await;
                info!(
                    pipeline = %self.pipeline.name,
                    duration_secs = duration.as_secs_f64(),
                    "Pipeline completed successfully"
                );
            }
            Err(e) => {
                self.pipeline.set_state(PipelineState::Failed).await;
                error!(
                    pipeline = %self.pipeline.name,
                    error = %e,
                    duration_secs = duration.as_secs_f64(),
                    "Pipeline failed"
                );
            }
        }

        // Log final statistics
        let stats = self.pipeline.get_stats();
        info!(
            pipeline = %self.pipeline.name,
            records_processed = stats.records_processed,
            records_filtered = stats.records_filtered,
            records_failed = stats.records_failed,
            success_rate = %format!("{:.2}%", stats.success_rate()),
            throughput = %format!("{:.2} rec/s", stats.throughput()),
            "Pipeline statistics"
        );

        result
    }

    /// Execute in batch mode.
    async fn execute_batch(&self) -> Result<()> {
        info!(
            pipeline = %self.pipeline.name,
            parallelism = self.pipeline.config.parallelism,
            "Executing in batch mode"
        );

        // Create a semaphore to limit parallelism
        let semaphore = Arc::new(Semaphore::new(self.pipeline.config.parallelism));

        // In a real implementation, this would:
        // 1. Read all data from sources
        // 2. Apply transforms in parallel
        // 3. Write to sinks
        // 4. Collect metrics and lineage

        // For now, simulate some processing
        self.simulate_processing(1000).await?;

        Ok(())
    }

    /// Execute in streaming mode.
    async fn execute_streaming(&self) -> Result<()> {
        info!(
            pipeline = %self.pipeline.name,
            "Executing in streaming mode"
        );

        // In a real implementation, this would:
        // 1. Subscribe to streaming sources
        // 2. Process records as they arrive
        // 3. Apply transforms
        // 4. Write to sinks continuously
        // 5. Handle backpressure

        // For now, simulate streaming
        self.simulate_streaming().await?;

        Ok(())
    }

    /// Execute in micro-batch mode.
    async fn execute_micro_batch(&self) -> Result<()> {
        info!(
            pipeline = %self.pipeline.name,
            batch_size = self.pipeline.config.batch_size,
            "Executing in micro-batch mode"
        );

        // In a real implementation, this would:
        // 1. Read data in small batches
        // 2. Process each batch
        // 3. Write results
        // 4. Repeat until done

        // For now, simulate micro-batch processing
        let num_batches = 10;
        for batch_num in 0..num_batches {
            info!(
                pipeline = %self.pipeline.name,
                batch = batch_num,
                "Processing micro-batch"
            );

            self.simulate_processing(self.pipeline.config.batch_size).await?;

            // Check if pipeline was paused or cancelled
            let state = self.pipeline.get_state().await;
            match state {
                PipelineState::Paused => {
                    warn!(
                        pipeline = %self.pipeline.name,
                        "Pipeline paused, waiting for resume"
                    );
                    self.wait_for_resume().await?;
                }
                PipelineState::Cancelled => {
                    warn!(
                        pipeline = %self.pipeline.name,
                        "Pipeline cancelled, stopping execution"
                    );
                    return Err(PipelineError::Execution("Pipeline was cancelled".into()));
                }
                _ => {}
            }

            // Create checkpoint if enabled
            if self.pipeline.config.checkpointing && batch_num % 5 == 0 {
                self.pipeline.checkpoint().await?;
            }
        }

        Ok(())
    }

    /// Wait for pipeline to be resumed.
    async fn wait_for_resume(&self) -> Result<()> {
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            if self.pipeline.is_running().await {
                info!(pipeline = %self.pipeline.name, "Pipeline resumed");
                break;
            }
            if self.pipeline.get_state().await == PipelineState::Cancelled {
                return Err(PipelineError::Execution(
                    "Pipeline was cancelled while paused".into(),
                ));
            }
        }
        Ok(())
    }

    /// Simulate processing (placeholder for actual implementation).
    async fn simulate_processing(&self, _num_records: usize) -> Result<()> {
        // Simulate some work
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }

    /// Simulate streaming (placeholder for actual implementation).
    async fn simulate_streaming(&self) -> Result<()> {
        // Simulate streaming for a short duration
        for _ in 0..5 {
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Check for cancellation
            if self.pipeline.get_state().await == PipelineState::Cancelled {
                return Err(PipelineError::Execution("Pipeline was cancelled".into()));
            }
        }
        Ok(())
    }

    /// Execute with retry logic.
    pub async fn execute_with_retry(&self) -> Result<()> {
        let max_retries = self.pipeline.config.max_retries;
        let mut attempt = 0;

        loop {
            attempt += 1;
            info!(
                pipeline = %self.pipeline.name,
                attempt = attempt,
                max_retries = max_retries,
                "Attempting pipeline execution"
            );

            match self.execute().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if attempt >= max_retries {
                        error!(
                            pipeline = %self.pipeline.name,
                            error = %e,
                            "Max retries exceeded"
                        );
                        return Err(e);
                    }

                    warn!(
                        pipeline = %self.pipeline.name,
                        error = %e,
                        attempt = attempt,
                        "Execution failed, retrying"
                    );

                    // Exponential backoff
                    let backoff = Duration::from_secs(2_u64.pow(attempt - 1));
                    tokio::time::sleep(backoff).await;

                    // Reset state for retry
                    self.pipeline.set_state(PipelineState::Initializing).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::PipelineBuilder;

    #[tokio::test]
    async fn test_executor_batch_mode() {
        let pipeline = PipelineBuilder::new("test-pipeline")
            .version("1.0.0")
            .batch()
            .with_parallelism(2)
            .build()
            .unwrap();

        let executor = PipelineExecutor::new(pipeline);
        let result = executor.execute().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_executor_streaming_mode() {
        let pipeline = PipelineBuilder::new("test-pipeline")
            .version("1.0.0")
            .streaming()
            .build()
            .unwrap();

        let executor = PipelineExecutor::new(pipeline);
        let result = executor.execute().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_executor_micro_batch_mode() {
        let pipeline = PipelineBuilder::new("test-pipeline")
            .version("1.0.0")
            .micro_batch()
            .batch_size(100)
            .build()
            .unwrap();

        let executor = PipelineExecutor::new(pipeline);
        let result = executor.execute().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_executor_timeout() {
        let pipeline = PipelineBuilder::new("test-pipeline")
            .version("1.0.0")
            .timeout_secs(1)
            .build()
            .unwrap();

        let executor = PipelineExecutor::new(pipeline);
        // This should complete before timeout in our simulation
        let result = executor.execute().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_executor_pause_resume() {
        let pipeline = PipelineBuilder::new("test-pipeline")
            .version("1.0.0")
            .batch()
            .build()
            .unwrap();

        assert_eq!(pipeline.get_state().await, PipelineState::Initializing);

        pipeline.set_state(PipelineState::Running).await;
        assert!(pipeline.is_running().await);

        pipeline.pause().await.unwrap();
        assert_eq!(pipeline.get_state().await, PipelineState::Paused);

        pipeline.resume().await.unwrap();
        assert!(pipeline.is_running().await);
    }
}
