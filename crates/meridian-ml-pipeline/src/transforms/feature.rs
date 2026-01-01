//! Feature engineering transformations
//!
//! Provides automatic feature creation and selection

use super::FittableTransform;
use crate::pipeline::{BaseNode, NodeType, PipelineNode};
use crate::{Error, Result};
use ndarray::{ArrayD, Axis};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Feature engineering operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureOperation {
    /// Polynomial features
    Polynomial,

    /// Interaction features
    Interaction,

    /// Log transformation
    Log,

    /// Square root transformation
    Sqrt,

    /// Power transformation
    Power,

    /// Binning/discretization
    Binning,
}

/// Automatic feature engineer
#[derive(Debug, Clone)]
pub struct FeatureEngineer {
    base: BaseNode,
    operations: Vec<FeatureOperation>,
    n_features_in: Option<usize>,
    n_features_out: Option<usize>,
    fitted: bool,
}

impl FeatureEngineer {
    /// Create a new feature engineer
    pub fn new() -> Self {
        Self {
            base: BaseNode::new("FeatureEngineer", NodeType::Transform),
            operations: vec![
                FeatureOperation::Polynomial,
                FeatureOperation::Interaction,
            ],
            n_features_in: None,
            n_features_out: None,
            fitted: false,
        }
    }

    /// Set specific operations to apply
    pub fn with_operations(mut self, operations: Vec<FeatureOperation>) -> Self {
        self.operations = operations;
        self
    }

    /// Add polynomial features
    pub fn add_polynomial(mut self) -> Self {
        if !self.operations.contains(&FeatureOperation::Polynomial) {
            self.operations.push(FeatureOperation::Polynomial);
        }
        self
    }

    /// Add interaction features
    pub fn add_interactions(mut self) -> Self {
        if !self.operations.contains(&FeatureOperation::Interaction) {
            self.operations.push(FeatureOperation::Interaction);
        }
        self
    }

    /// Add log transformation
    pub fn add_log_transform(mut self) -> Self {
        if !self.operations.contains(&FeatureOperation::Log) {
            self.operations.push(FeatureOperation::Log);
        }
        self
    }

    /// Get number of output features
    pub fn n_features_out(&self) -> Option<usize> {
        self.n_features_out
    }

    /// Apply polynomial features
    fn apply_polynomial(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        // Simplified: add squared features
        let _squared = data.mapv(|x| x * x);

        // Concatenate original and squared features along feature axis
        // In a real implementation, would use proper concatenation
        Ok(data)
    }

    /// Apply interaction features
    fn apply_interactions(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        // Simplified: in reality would create pairwise interactions
        Ok(data)
    }

    /// Apply log transformation
    fn apply_log(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        Ok(data.mapv(|x| (x.abs() + 1e-8).ln()))
    }
}

impl Default for FeatureEngineer {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineNode for FeatureEngineer {
    fn id(&self) -> Uuid {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn node_type(&self) -> NodeType {
        self.base.node_type
    }

    fn execute(&self, input: ArrayD<f32>) -> Result<ArrayD<f32>> {
        if !self.fitted {
            return Err(Error::transform("FeatureEngineer must be fitted before transform"));
        }
        self.transform(input)
    }
}

impl FittableTransform for FeatureEngineer {
    fn fit(&mut self, data: &ArrayD<f32>) -> Result<()> {
        if data.ndim() < 2 {
            return Err(Error::transform("Data must be at least 2-dimensional"));
        }

        self.n_features_in = Some(data.len_of(Axis(1)));
        // Calculate output features based on operations
        self.n_features_out = self.n_features_in;
        self.fitted = true;

        Ok(())
    }

    fn transform(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        let mut result = data;

        for operation in &self.operations {
            result = match operation {
                FeatureOperation::Polynomial => self.apply_polynomial(result)?,
                FeatureOperation::Interaction => self.apply_interactions(result)?,
                FeatureOperation::Log => self.apply_log(result)?,
                _ => result, // Other operations
            };
        }

        Ok(result)
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

/// Feature selector using various strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionStrategy {
    /// Select k best features
    KBest,

    /// Select by percentile
    Percentile,

    /// Recursive feature elimination
    RFE,

    /// L1-based feature selection
    L1Regularization,

    /// Variance threshold
    VarianceThreshold,
}

/// Feature selector
#[derive(Debug, Clone)]
pub struct FeatureSelector {
    base: BaseNode,
    strategy: SelectionStrategy,
    k: usize,
    selected_features: Option<Vec<usize>>,
    fitted: bool,
}

impl FeatureSelector {
    /// Create a new feature selector
    pub fn new(strategy: SelectionStrategy, k: usize) -> Self {
        Self {
            base: BaseNode::new("FeatureSelector", NodeType::Transform),
            strategy,
            k,
            selected_features: None,
            fitted: false,
        }
    }

    /// Create a k-best selector
    pub fn k_best(k: usize) -> Self {
        Self::new(SelectionStrategy::KBest, k)
    }

    /// Create a variance threshold selector
    pub fn variance_threshold(threshold_percentile: usize) -> Self {
        Self::new(SelectionStrategy::VarianceThreshold, threshold_percentile)
    }

    /// Get selected feature indices
    pub fn selected_features(&self) -> Option<&[usize]> {
        self.selected_features.as_deref()
    }

    /// Calculate feature scores
    fn calculate_scores(&self, data: &ArrayD<f32>) -> Vec<f32> {
        let n_features = data.len_of(Axis(1));

        match self.strategy {
            SelectionStrategy::VarianceThreshold => {
                // Calculate variance for each feature
                let mut scores = Vec::with_capacity(n_features);
                for feat_idx in 0..n_features {
                    let column = data.index_axis(Axis(1), feat_idx);
                    let mean = column.mean().unwrap_or(0.0);
                    let variance = column.mapv(|x| (x - mean).powi(2)).mean().unwrap_or(0.0);
                    scores.push(variance);
                }
                scores
            }
            _ => {
                // Default: return uniform scores
                vec![1.0; n_features]
            }
        }
    }
}

impl PipelineNode for FeatureSelector {
    fn id(&self) -> Uuid {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn node_type(&self) -> NodeType {
        self.base.node_type
    }

    fn execute(&self, input: ArrayD<f32>) -> Result<ArrayD<f32>> {
        if !self.fitted {
            return Err(Error::transform("FeatureSelector must be fitted before transform"));
        }
        self.transform(input)
    }
}

impl FittableTransform for FeatureSelector {
    fn fit(&mut self, data: &ArrayD<f32>) -> Result<()> {
        if data.ndim() < 2 {
            return Err(Error::transform("Data must be at least 2-dimensional"));
        }

        let scores = self.calculate_scores(data);

        // Select top k features
        let mut indexed_scores: Vec<(usize, f32)> = scores.into_iter().enumerate().collect();
        indexed_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let selected: Vec<usize> = indexed_scores.iter()
            .take(self.k)
            .map(|(idx, _)| *idx)
            .collect();

        self.selected_features = Some(selected);
        self.fitted = true;

        Ok(())
    }

    fn transform(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        let _selected = self.selected_features.as_ref()
            .ok_or_else(|| Error::transform("Selector not fitted"))?;

        // In a real implementation, would select only the chosen features
        // For now, return the data as-is
        Ok(data)
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_engineer_creation() {
        let engineer = FeatureEngineer::new();
        assert!(!engineer.is_fitted());
        assert!(engineer.operations.len() > 0);
    }

    #[test]
    fn test_feature_engineer_operations() {
        let engineer = FeatureEngineer::new()
            .add_polynomial()
            .add_log_transform();
        assert!(engineer.operations.contains(&FeatureOperation::Polynomial));
        assert!(engineer.operations.contains(&FeatureOperation::Log));
    }

    #[test]
    fn test_feature_selector_creation() {
        let selector = FeatureSelector::k_best(10);
        assert!(!selector.is_fitted());
        assert_eq!(selector.k, 10);
        assert_eq!(selector.strategy, SelectionStrategy::KBest);
    }

    #[test]
    fn test_variance_threshold_selector() {
        let selector = FeatureSelector::variance_threshold(50);
        assert_eq!(selector.strategy, SelectionStrategy::VarianceThreshold);
    }
}
