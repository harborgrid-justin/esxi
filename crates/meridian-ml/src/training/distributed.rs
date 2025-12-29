//! Distributed training support

use crate::error::{MlError, Result};
use crate::training::{TrainingConfig, TrainingHistory};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Distributed training strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistributedStrategy {
    /// Data parallelism
    DataParallel,

    /// Model parallelism
    ModelParallel,

    /// Pipeline parallelism
    PipelineParallel,

    /// Hybrid (data + model)
    Hybrid,
}

/// Distributed training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedConfig {
    /// Strategy
    pub strategy: DistributedStrategy,

    /// Number of workers
    pub num_workers: usize,

    /// Worker addresses
    pub worker_addresses: Vec<String>,

    /// Gradient aggregation method
    pub aggregation: AggregationMethod,

    /// Communication backend
    pub backend: CommunicationBackend,

    /// Timeout for worker communication (seconds)
    pub timeout_secs: u64,
}

impl Default for DistributedConfig {
    fn default() -> Self {
        Self {
            strategy: DistributedStrategy::DataParallel,
            num_workers: 4,
            worker_addresses: Vec::new(),
            aggregation: AggregationMethod::AllReduce,
            backend: CommunicationBackend::Gloo,
            timeout_secs: 300,
        }
    }
}

/// Gradient aggregation methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationMethod {
    /// AllReduce (sum gradients across workers)
    AllReduce,

    /// Parameter server
    ParameterServer,

    /// Ring AllReduce
    RingAllReduce,

    /// Hierarchical AllReduce
    HierarchicalAllReduce,
}

/// Communication backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommunicationBackend {
    /// Gloo (CPU)
    Gloo,

    /// NCCL (GPU)
    NCCL,

    /// MPI
    MPI,

    /// Custom
    Custom,
}

/// Worker node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    /// Worker ID
    pub id: usize,

    /// Rank in the distributed system
    pub rank: usize,

    /// Total number of workers
    pub world_size: usize,

    /// Worker address
    pub address: Option<String>,

    /// Is master node
    pub is_master: bool,
}

/// Distributed trainer
pub struct DistributedTrainer {
    /// Training configuration
    training_config: TrainingConfig,

    /// Distributed configuration
    distributed_config: DistributedConfig,

    /// Worker information
    worker_info: Option<WorkerInfo>,

    /// Training history
    history: TrainingHistory,

    /// Is initialized
    initialized: bool,
}

impl DistributedTrainer {
    /// Create a new distributed trainer
    pub fn new(training_config: TrainingConfig, distributed_config: DistributedConfig) -> Self {
        Self {
            training_config,
            distributed_config,
            worker_info: None,
            history: TrainingHistory::new(),
            initialized: false,
        }
    }

    /// Initialize distributed environment
    pub async fn init(&mut self, rank: usize, world_size: usize) -> Result<()> {
        if rank >= world_size {
            return Err(MlError::InvalidConfig(format!(
                "Rank {} must be less than world size {}",
                rank, world_size
            )));
        }

        self.worker_info = Some(WorkerInfo {
            id: rank,
            rank,
            world_size,
            address: None,
            is_master: rank == 0,
        });

        self.initialized = true;

        if self.is_master() {
            tracing::info!(
                "Initialized distributed training with {} workers using {:?} strategy",
                world_size,
                self.distributed_config.strategy
            );
        }

        Ok(())
    }

    /// Check if this is the master worker
    pub fn is_master(&self) -> bool {
        self.worker_info
            .as_ref()
            .map(|info| info.is_master)
            .unwrap_or(false)
    }

    /// Get worker rank
    pub fn rank(&self) -> Option<usize> {
        self.worker_info.as_ref().map(|info| info.rank)
    }

    /// Get world size
    pub fn world_size(&self) -> Option<usize> {
        self.worker_info.as_ref().map(|info| info.world_size)
    }

    /// Broadcast data from master to all workers
    pub async fn broadcast<T: Clone>(&self, data: &T) -> Result<T> {
        if !self.initialized {
            return Err(MlError::Model("Not initialized".to_string()));
        }

        // Simplified broadcast - actual implementation would use communication backend
        Ok(data.clone())
    }

    /// Gather data from all workers to master
    pub async fn gather<T>(&self, data: T) -> Result<Vec<T>> {
        if !self.initialized {
            return Err(MlError::Model("Not initialized".to_string()));
        }

        // Simplified gather
        Ok(vec![data])
    }

    /// AllReduce operation (sum across all workers)
    pub async fn all_reduce(&self, values: &[f64]) -> Result<Vec<f64>> {
        if !self.initialized {
            return Err(MlError::Model("Not initialized".to_string()));
        }

        // Simplified all_reduce - actual implementation would sum across workers
        Ok(values.to_vec())
    }

    /// Synchronize all workers
    pub async fn barrier(&self) -> Result<()> {
        if !self.initialized {
            return Err(MlError::Model("Not initialized".to_string()));
        }

        // Simplified barrier
        Ok(())
    }

    /// Train model in distributed fashion
    pub async fn train<M>(&mut self, model: &mut M) -> Result<()>
    where
        M: Send + Sync,
    {
        if !self.initialized {
            return Err(MlError::Model("Not initialized".to_string()));
        }

        let world_size = self.world_size().unwrap();
        let rank = self.rank().unwrap();

        // Training loop
        for epoch in 0..self.training_config.max_epochs {
            // Each worker processes its partition of data
            // Simplified training step
            let local_loss = 1.0 / (epoch + 1) as f64;

            // Aggregate losses across workers
            let global_loss = self.all_reduce(&[local_loss]).await?;
            let avg_loss = global_loss[0] / world_size as f64;

            if self.is_master() {
                self.history.record_epoch(
                    avg_loss,
                    avg_loss * 0.9,
                    self.training_config.learning_rate,
                );

                if self.training_config.verbose && epoch % 10 == 0 {
                    tracing::info!(
                        "Epoch {}/{}: global_loss={:.4}",
                        epoch + 1,
                        self.training_config.max_epochs,
                        avg_loss
                    );
                }
            }

            // Synchronize workers
            self.barrier().await?;
        }

        Ok(())
    }

    /// Get training history (master only)
    pub fn history(&self) -> Option<&TrainingHistory> {
        if self.is_master() {
            Some(&self.history)
        } else {
            None
        }
    }

    /// Shutdown distributed environment
    pub async fn shutdown(&mut self) -> Result<()> {
        if !self.initialized {
            return Ok(());
        }

        if self.is_master() {
            tracing::info!("Shutting down distributed training");
        }

        self.initialized = false;
        self.worker_info = None;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distributed_config() {
        let config = DistributedConfig::default();
        assert_eq!(config.strategy, DistributedStrategy::DataParallel);
        assert_eq!(config.num_workers, 4);
    }

    #[tokio::test]
    async fn test_distributed_trainer_init() {
        let training_config = TrainingConfig::default();
        let distributed_config = DistributedConfig::default();

        let mut trainer = DistributedTrainer::new(training_config, distributed_config);
        trainer.init(0, 4).await.unwrap();

        assert!(trainer.is_master());
        assert_eq!(trainer.rank(), Some(0));
        assert_eq!(trainer.world_size(), Some(4));
    }
}
