//! Pipeline execution engine
//!
//! Handles the execution of pipeline nodes with monitoring and error handling

use super::{PipelineNode, NodeMetrics};
use crate::{Error, Result};
use ndarray::ArrayD;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// Configuration for pipeline executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum concurrent executions
    pub max_concurrency: usize,

    /// Enable execution metrics collection
    pub collect_metrics: bool,

    /// Timeout for node execution in milliseconds
    pub timeout_ms: u64,

    /// Enable retry on failure
    pub enable_retry: bool,

    /// Maximum retry attempts
    pub max_retries: usize,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 100,
            collect_metrics: true,
            timeout_ms: 30_000,
            enable_retry: true,
            max_retries: 3,
        }
    }
}

/// Pipeline execution engine
pub struct PipelineExecutor {
    config: ExecutorConfig,
    semaphore: Arc<Semaphore>,
}

impl PipelineExecutor {
    /// Create a new executor with default configuration
    pub fn new() -> Self {
        Self::with_config(ExecutorConfig::default())
    }

    /// Create a new executor with custom configuration
    pub fn with_config(config: ExecutorConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
        Self { config, semaphore }
    }

    /// Execute a pipeline
    pub async fn execute(
        &self,
        nodes: &[Arc<dyn PipelineNode>],
        input: ArrayD<f32>,
    ) -> Result<ArrayD<f32>> {
        if nodes.is_empty() {
            return Err(Error::pipeline("Cannot execute empty pipeline"));
        }

        info!("Executing pipeline with {} nodes", nodes.len());

        let mut data = input;
        let mut all_metrics = Vec::new();

        for (idx, node) in nodes.iter().enumerate() {
            debug!("Executing node {}/{}: {}", idx + 1, nodes.len(), node.name());

            let (output, metrics) = self.execute_node(node.as_ref(), data).await?;

            if self.config.collect_metrics {
                all_metrics.push(metrics);
            }

            data = output;
        }

        info!(
            "Pipeline execution completed successfully. Metrics: {} nodes",
            all_metrics.len()
        );

        Ok(data)
    }

    /// Execute a single node with monitoring
    async fn execute_node(
        &self,
        node: &dyn PipelineNode,
        input: ArrayD<f32>,
    ) -> Result<(ArrayD<f32>, NodeMetrics)> {
        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await
            .map_err(|e| Error::internal(format!("Semaphore error: {}", e)))?;

        let start = Instant::now();
        let input_shape = input.shape().to_vec();
        let node_id = node.id();

        // Execute with retry logic
        let output = if self.config.enable_retry {
            self.execute_with_retry(node, input).await?
        } else {
            node.execute(input)?
        };

        let execution_time = start.elapsed();
        let output_shape = output.shape().to_vec();

        // Estimate memory usage (rough approximation)
        let memory_bytes = output.len() * std::mem::size_of::<f32>();

        let metrics = NodeMetrics {
            node_id,
            execution_time_ms: execution_time.as_millis() as u64,
            input_shape,
            output_shape,
            memory_bytes,
        };

        debug!(
            "Node {} completed in {}ms",
            node.name(),
            metrics.execution_time_ms
        );

        Ok((output, metrics))
    }

    /// Execute node with retry logic
    async fn execute_with_retry(
        &self,
        node: &dyn PipelineNode,
        input: ArrayD<f32>,
    ) -> Result<ArrayD<f32>> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_retries {
            match node.execute(input.clone()) {
                Ok(output) => return Ok(output),
                Err(e) => {
                    attempts += 1;
                    warn!(
                        "Node {} failed (attempt {}/{}): {}",
                        node.name(),
                        attempts,
                        self.config.max_retries,
                        e
                    );
                    last_error = Some(e);

                    if attempts < self.config.max_retries {
                        // Exponential backoff
                        let backoff_ms = 100 * 2_u64.pow(attempts as u32);
                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::internal("Execution failed")))
    }

    /// Execute pipeline in parallel (for batch processing)
    pub async fn execute_batch(
        &self,
        nodes: &[Arc<dyn PipelineNode>],
        inputs: Vec<ArrayD<f32>>,
    ) -> Result<Vec<ArrayD<f32>>> {
        use futures::stream::{self, StreamExt};

        let nodes = Arc::new(nodes.to_vec());

        let results: Vec<Result<ArrayD<f32>>> = stream::iter(inputs)
            .map(|input| {
                let nodes = Arc::clone(&nodes);
                async move {
                    self.execute(&nodes, input).await
                }
            })
            .buffer_unordered(self.config.max_concurrency)
            .collect()
            .await;

        results.into_iter().collect()
    }
}

impl Default for PipelineExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_config_default() {
        let config = ExecutorConfig::default();
        assert_eq!(config.max_concurrency, 100);
        assert!(config.collect_metrics);
        assert!(config.enable_retry);
    }

    #[test]
    fn test_executor_creation() {
        let executor = PipelineExecutor::new();
        assert_eq!(executor.config.max_concurrency, 100);
    }

    #[tokio::test]
    async fn test_empty_pipeline_execution() {
        let executor = PipelineExecutor::new();
        let input = ArrayD::zeros(vec![1, 10]);
        let result = executor.execute(&[], input).await;
        assert!(result.is_err());
    }
}
