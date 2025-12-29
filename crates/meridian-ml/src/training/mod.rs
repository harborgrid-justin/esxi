//! Model training and optimization

pub mod distributed;

pub use distributed::{DistributedTrainer, DistributedConfig};

use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Training configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Maximum number of epochs
    pub max_epochs: usize,

    /// Learning rate
    pub learning_rate: f64,

    /// Batch size
    pub batch_size: usize,

    /// Validation split ratio
    pub validation_split: f64,

    /// Early stopping patience
    pub early_stopping_patience: Option<usize>,

    /// Learning rate schedule
    pub lr_schedule: LearningRateSchedule,

    /// Optimizer type
    pub optimizer: OptimizerType,

    /// Regularization
    pub regularization: RegularizationType,

    /// Random seed
    pub random_seed: Option<u64>,

    /// Verbose output
    pub verbose: bool,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            max_epochs: 100,
            learning_rate: 0.001,
            batch_size: 32,
            validation_split: 0.2,
            early_stopping_patience: Some(10),
            lr_schedule: LearningRateSchedule::Constant,
            optimizer: OptimizerType::Adam,
            regularization: RegularizationType::None,
            random_seed: Some(42),
            verbose: true,
        }
    }
}

/// Learning rate schedule
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LearningRateSchedule {
    /// Constant learning rate
    Constant,

    /// Step decay
    StepDecay { step_size: usize, gamma: f64 },

    /// Exponential decay
    ExponentialDecay { gamma: f64 },

    /// Cosine annealing
    CosineAnnealing { t_max: usize },

    /// Reduce on plateau
    ReduceOnPlateau { factor: f64, patience: usize },
}

/// Optimizer type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizerType {
    /// Stochastic Gradient Descent
    SGD,

    /// SGD with momentum
    SGDMomentum { momentum: u32 }, // Using u32 for serialization

    /// Adam optimizer
    Adam,

    /// AdaGrad
    AdaGrad,

    /// RMSprop
    RMSprop,
}

/// Regularization type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RegularizationType {
    /// No regularization
    None,

    /// L1 regularization (Lasso)
    L1 { alpha: f64 },

    /// L2 regularization (Ridge)
    L2 { alpha: f64 },

    /// Elastic Net (L1 + L2)
    ElasticNet { l1_ratio: f64, alpha: f64 },

    /// Dropout
    Dropout { rate: f64 },
}

/// Training history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingHistory {
    /// Training loss per epoch
    pub train_loss: Vec<f64>,

    /// Validation loss per epoch
    pub val_loss: Vec<f64>,

    /// Training metrics per epoch
    pub train_metrics: HashMap<String, Vec<f64>>,

    /// Validation metrics per epoch
    pub val_metrics: HashMap<String, Vec<f64>>,

    /// Learning rates per epoch
    pub learning_rates: Vec<f64>,

    /// Best epoch
    pub best_epoch: usize,

    /// Best validation loss
    pub best_val_loss: f64,
}

impl TrainingHistory {
    /// Create a new training history
    pub fn new() -> Self {
        Self {
            train_loss: Vec::new(),
            val_loss: Vec::new(),
            train_metrics: HashMap::new(),
            val_metrics: HashMap::new(),
            learning_rates: Vec::new(),
            best_epoch: 0,
            best_val_loss: f64::INFINITY,
        }
    }

    /// Record an epoch
    pub fn record_epoch(
        &mut self,
        train_loss: f64,
        val_loss: f64,
        learning_rate: f64,
    ) {
        self.train_loss.push(train_loss);
        self.val_loss.push(val_loss);
        self.learning_rates.push(learning_rate);

        if val_loss < self.best_val_loss {
            self.best_val_loss = val_loss;
            self.best_epoch = self.train_loss.len() - 1;
        }
    }

    /// Add a metric
    pub fn add_metric(&mut self, name: String, train_value: f64, val_value: f64) {
        self.train_metrics
            .entry(name.clone())
            .or_insert_with(Vec::new)
            .push(train_value);

        self.val_metrics
            .entry(name)
            .or_insert_with(Vec::new)
            .push(val_value);
    }

    /// Get number of epochs
    pub fn num_epochs(&self) -> usize {
        self.train_loss.len()
    }

    /// Check if training should stop early
    pub fn should_early_stop(&self, patience: usize) -> bool {
        if self.num_epochs() < patience {
            return false;
        }

        let current_epoch = self.num_epochs() - 1;
        current_epoch - self.best_epoch >= patience
    }
}

impl Default for TrainingHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Trainer for machine learning models
pub struct Trainer {
    /// Training configuration
    config: TrainingConfig,

    /// Training history
    history: TrainingHistory,
}

impl Trainer {
    /// Create a new trainer
    pub fn new(config: TrainingConfig) -> Self {
        Self {
            config,
            history: TrainingHistory::new(),
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(TrainingConfig::default())
    }

    /// Set maximum epochs
    pub fn with_max_epochs(mut self, epochs: usize) -> Self {
        self.config.max_epochs = epochs;
        self
    }

    /// Set learning rate
    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.config.learning_rate = lr;
        self
    }

    /// Set batch size
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    /// Set early stopping patience
    pub fn with_early_stopping(mut self, patience: usize) -> Self {
        self.config.early_stopping_patience = Some(patience);
        self
    }

    /// Train a model (placeholder interface)
    pub fn train<M>(&mut self, model: &mut M, features: &FeatureSet, targets: &Array1<f64>) -> Result<()>
    where
        M: Send + Sync,
    {
        // Split data into train and validation
        let (train_features, val_features) = features.train_test_split(
            self.config.validation_split,
            true,
        )?;

        let n_train = train_features.num_samples();
        let n_val = val_features.num_samples();

        let train_targets = targets.slice(ndarray::s![..n_train]).to_owned();
        let val_targets = targets.slice(ndarray::s![n_train..]).to_owned();

        // Training loop
        for epoch in 0..self.config.max_epochs {
            let lr = self.get_learning_rate(epoch);

            // Simplified training step - actual implementation would train the model
            let train_loss = 1.0 / (epoch + 1) as f64; // Dummy loss
            let val_loss = 1.0 / (epoch + 1) as f64; // Dummy loss

            self.history.record_epoch(train_loss, val_loss, lr);

            if self.config.verbose && epoch % 10 == 0 {
                tracing::info!(
                    "Epoch {}/{}: train_loss={:.4}, val_loss={:.4}, lr={:.6}",
                    epoch + 1,
                    self.config.max_epochs,
                    train_loss,
                    val_loss,
                    lr
                );
            }

            // Early stopping
            if let Some(patience) = self.config.early_stopping_patience {
                if self.history.should_early_stop(patience) {
                    if self.config.verbose {
                        tracing::info!("Early stopping at epoch {}", epoch + 1);
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    /// Get learning rate for current epoch
    fn get_learning_rate(&self, epoch: usize) -> f64 {
        match self.config.lr_schedule {
            LearningRateSchedule::Constant => self.config.learning_rate,
            LearningRateSchedule::StepDecay { step_size, gamma } => {
                let steps = epoch / step_size;
                self.config.learning_rate * gamma.powi(steps as i32)
            }
            LearningRateSchedule::ExponentialDecay { gamma } => {
                self.config.learning_rate * gamma.powi(epoch as i32)
            }
            LearningRateSchedule::CosineAnnealing { t_max } => {
                let progress = (epoch % t_max) as f64 / t_max as f64;
                self.config.learning_rate * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos())
            }
            LearningRateSchedule::ReduceOnPlateau { .. } => {
                // Would need to track metric history
                self.config.learning_rate
            }
        }
    }

    /// Get training history
    pub fn history(&self) -> &TrainingHistory {
        &self.history
    }

    /// Get configuration
    pub fn config(&self) -> &TrainingConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_config() {
        let config = TrainingConfig::default();
        assert_eq!(config.max_epochs, 100);
        assert_eq!(config.batch_size, 32);
    }

    #[test]
    fn test_training_history() {
        let mut history = TrainingHistory::new();
        history.record_epoch(1.0, 0.9, 0.001);
        history.record_epoch(0.8, 0.7, 0.001);

        assert_eq!(history.num_epochs(), 2);
        assert_eq!(history.best_epoch, 1);
    }

    #[test]
    fn test_early_stopping() {
        let mut history = TrainingHistory::new();
        for i in 0..10 {
            history.record_epoch(1.0, 1.0, 0.001);
        }
        history.record_epoch(0.5, 0.5, 0.001); // Best

        for i in 0..5 {
            history.record_epoch(1.0, 1.0, 0.001);
        }

        assert!(history.should_early_stop(5));
    }
}
