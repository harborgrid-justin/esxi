//! Categorical encoding transformations
//!
//! Provides various encoding strategies for categorical variables

use super::FittableTransform;
use crate::pipeline::{BaseNode, NodeType, PipelineNode};
use crate::{Error, Result};
use ndarray::{ArrayD, Array2};
use std::collections::HashMap;
use uuid::Uuid;

/// Categorical encoder for string/label encoding
#[derive(Debug, Clone)]
pub struct CategoricalEncoder {
    base: BaseNode,
    columns: Vec<String>,
    mappings: HashMap<String, HashMap<String, usize>>,
    fitted: bool,
}

impl CategoricalEncoder {
    /// Create a new categorical encoder
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            base: BaseNode::new("CategoricalEncoder", NodeType::Transform),
            columns,
            mappings: HashMap::new(),
            fitted: false,
        }
    }
}

impl PipelineNode for CategoricalEncoder {
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
        // For numerical data, pass through
        // In a real implementation, this would handle mixed-type dataframes
        Ok(input)
    }
}

impl FittableTransform for CategoricalEncoder {
    fn fit(&mut self, _data: &ArrayD<f32>) -> Result<()> {
        // In a real implementation, this would learn category mappings
        self.fitted = true;
        Ok(())
    }

    fn transform(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        // Pass through for now
        Ok(data)
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

/// One-hot encoder
///
/// Converts categorical variables into binary vectors
#[derive(Debug, Clone)]
pub struct OneHotEncoder {
    base: BaseNode,
    n_categories: usize,
    categories: Option<Vec<Vec<String>>>,
    fitted: bool,
}

impl OneHotEncoder {
    /// Create a new one-hot encoder
    pub fn new(n_categories: usize) -> Self {
        Self {
            base: BaseNode::new("OneHotEncoder", NodeType::Transform),
            n_categories,
            categories: None,
            fitted: false,
        }
    }

    /// Get the number of output features after encoding
    pub fn n_features_out(&self) -> Option<usize> {
        self.categories.as_ref().map(|cats| {
            cats.iter().map(|c| c.len()).sum()
        })
    }
}

impl PipelineNode for OneHotEncoder {
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
            return Err(Error::transform("OneHotEncoder must be fitted before transform"));
        }
        self.transform(input)
    }
}

impl FittableTransform for OneHotEncoder {
    fn fit(&mut self, _data: &ArrayD<f32>) -> Result<()> {
        // In a real implementation, discover unique categories
        self.fitted = true;
        Ok(())
    }

    fn transform(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        // In a real implementation, perform one-hot encoding
        // For now, pass through
        Ok(data)
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

/// Label encoder
///
/// Encodes categorical labels as integers
#[derive(Debug, Clone)]
pub struct LabelEncoder {
    base: BaseNode,
    label_map: HashMap<String, usize>,
    inverse_map: HashMap<usize, String>,
    fitted: bool,
}

impl LabelEncoder {
    /// Create a new label encoder
    pub fn new() -> Self {
        Self {
            base: BaseNode::new("LabelEncoder", NodeType::Transform),
            label_map: HashMap::new(),
            inverse_map: HashMap::new(),
            fitted: false,
        }
    }

    /// Encode a single label
    pub fn encode_label(&self, label: &str) -> Option<usize> {
        self.label_map.get(label).copied()
    }

    /// Decode a single encoded value
    pub fn decode_label(&self, value: usize) -> Option<&str> {
        self.inverse_map.get(&value).map(|s| s.as_str())
    }

    /// Get the number of unique labels
    pub fn n_labels(&self) -> usize {
        self.label_map.len()
    }
}

impl Default for LabelEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineNode for LabelEncoder {
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
            return Err(Error::transform("LabelEncoder must be fitted before transform"));
        }
        self.transform(input)
    }
}

impl FittableTransform for LabelEncoder {
    fn fit(&mut self, _data: &ArrayD<f32>) -> Result<()> {
        // In a real implementation, build label mappings
        self.fitted = true;
        Ok(())
    }

    fn transform(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
        // Pass through for numerical data
        Ok(data)
    }

    fn is_fitted(&self) -> bool {
        self.fitted
    }
}

/// Target encoder
///
/// Encodes categorical variables using target statistics
#[derive(Debug, Clone)]
pub struct TargetEncoder {
    base: BaseNode,
    target_means: HashMap<String, f32>,
    global_mean: f32,
    fitted: bool,
}

impl TargetEncoder {
    /// Create a new target encoder
    pub fn new() -> Self {
        Self {
            base: BaseNode::new("TargetEncoder", NodeType::Transform),
            target_means: HashMap::new(),
            global_mean: 0.0,
            fitted: false,
        }
    }
}

impl Default for TargetEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineNode for TargetEncoder {
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
        self.transform(input)
    }
}

impl FittableTransform for TargetEncoder {
    fn fit(&mut self, _data: &ArrayD<f32>) -> Result<()> {
        self.fitted = true;
        Ok(())
    }

    fn transform(&self, data: ArrayD<f32>) -> Result<ArrayD<f32>> {
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
    fn test_categorical_encoder_creation() {
        let encoder = CategoricalEncoder::new(vec!["category".to_string()]);
        assert!(!encoder.is_fitted());
        assert_eq!(encoder.columns.len(), 1);
    }

    #[test]
    fn test_onehot_encoder_creation() {
        let encoder = OneHotEncoder::new(5);
        assert_eq!(encoder.n_categories, 5);
        assert!(!encoder.is_fitted());
    }

    #[test]
    fn test_label_encoder_creation() {
        let encoder = LabelEncoder::new();
        assert!(!encoder.is_fitted());
        assert_eq!(encoder.n_labels(), 0);
    }

    #[test]
    fn test_target_encoder_creation() {
        let encoder = TargetEncoder::new();
        assert!(!encoder.is_fitted());
    }
}
