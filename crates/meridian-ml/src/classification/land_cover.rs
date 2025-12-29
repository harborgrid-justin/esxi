//! Land cover classification

use crate::classification::{Classifier, ClassificationResult};
use crate::error::{MlError, Result};
use crate::features::FeatureSet;
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Land cover class types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LandCoverClass {
    /// Water bodies
    Water = 0,

    /// Forest/trees
    Forest = 1,

    /// Grassland
    Grassland = 2,

    /// Agricultural land
    Agriculture = 3,

    /// Urban/built-up
    Urban = 4,

    /// Barren land
    Barren = 5,

    /// Wetlands
    Wetland = 6,

    /// Snow/ice
    Snow = 7,

    /// Unknown/unclassified
    Unknown = 255,
}

impl LandCoverClass {
    /// Get class name
    pub fn name(&self) -> &str {
        match self {
            Self::Water => "Water",
            Self::Forest => "Forest",
            Self::Grassland => "Grassland",
            Self::Agriculture => "Agriculture",
            Self::Urban => "Urban",
            Self::Barren => "Barren",
            Self::Wetland => "Wetland",
            Self::Snow => "Snow",
            Self::Unknown => "Unknown",
        }
    }

    /// Get all standard classes
    pub fn all() -> Vec<Self> {
        vec![
            Self::Water,
            Self::Forest,
            Self::Grassland,
            Self::Agriculture,
            Self::Urban,
            Self::Barren,
            Self::Wetland,
            Self::Snow,
        ]
    }

    /// Convert to index
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    /// Convert from index
    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Water),
            1 => Some(Self::Forest),
            2 => Some(Self::Grassland),
            3 => Some(Self::Agriculture),
            4 => Some(Self::Urban),
            5 => Some(Self::Barren),
            6 => Some(Self::Wetland),
            7 => Some(Self::Snow),
            _ => None,
        }
    }
}

/// Land cover classifier configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandCoverConfig {
    /// Classes to use
    pub classes: Vec<LandCoverClass>,

    /// Use spectral indices (NDVI, NDWI, etc.)
    pub use_spectral_indices: bool,

    /// Use texture features
    pub use_texture: bool,

    /// Spatial context window size
    pub context_window: usize,

    /// Post-processing: minimum mapping unit (pixels)
    pub min_mapping_unit: usize,

    /// Post-processing: apply majority filter
    pub apply_majority_filter: bool,
}

impl Default for LandCoverConfig {
    fn default() -> Self {
        Self {
            classes: LandCoverClass::all(),
            use_spectral_indices: true,
            use_texture: true,
            context_window: 5,
            min_mapping_unit: 9,
            apply_majority_filter: true,
        }
    }
}

/// Land cover classifier
pub struct LandCoverClassifier {
    /// Configuration
    config: LandCoverConfig,

    /// Underlying classifier
    classifier: Box<dyn Classifier>,

    /// Class mapping
    class_mapping: HashMap<usize, LandCoverClass>,

    /// Training statistics
    training_stats: Option<TrainingStats>,
}

/// Training statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TrainingStats {
    /// Samples per class
    samples_per_class: HashMap<LandCoverClass, usize>,

    /// Overall accuracy
    accuracy: f64,

    /// Per-class accuracy
    class_accuracy: HashMap<LandCoverClass, f64>,
}

impl LandCoverClassifier {
    /// Create a new land cover classifier
    pub fn new() -> Self {
        Self::with_config(LandCoverConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: LandCoverConfig) -> Self {
        use crate::classification::RandomForestClassifier;

        let class_mapping: HashMap<usize, LandCoverClass> = config
            .classes
            .iter()
            .enumerate()
            .map(|(i, &c)| (i, c))
            .collect();

        Self {
            config,
            classifier: Box::new(RandomForestClassifier::new(100)),
            class_mapping,
            training_stats: None,
        }
    }

    /// Set the underlying classifier
    pub fn with_classifier(mut self, classifier: Box<dyn Classifier>) -> Self {
        self.classifier = classifier;
        self
    }

    /// Train on labeled data
    pub fn train(&mut self, features: &FeatureSet, labels: &[LandCoverClass]) -> Result<()> {
        // Convert land cover classes to indices
        let label_indices: Vec<usize> = labels
            .iter()
            .map(|lc| {
                self.class_mapping
                    .iter()
                    .find(|(_, &c)| c == *lc)
                    .map(|(&i, _)| i)
                    .ok_or_else(|| {
                        MlError::InvalidInput(format!("Unknown land cover class: {:?}", lc))
                    })
            })
            .collect::<Result<Vec<_>>>()?;

        let label_array = Array1::from_vec(label_indices.clone());

        // Train classifier
        self.classifier.train(features, &label_array)?;

        // Calculate training statistics
        let mut samples_per_class = HashMap::new();
        for &label in labels {
            *samples_per_class.entry(label).or_insert(0) += 1;
        }

        self.training_stats = Some(TrainingStats {
            samples_per_class,
            accuracy: 0.0, // Would be calculated with cross-validation
            class_accuracy: HashMap::new(),
        });

        Ok(())
    }

    /// Classify features
    pub fn classify(&self, features: &FeatureSet) -> Result<Vec<LandCoverClass>> {
        let predictions = self.classifier.predict(features)?;

        // Convert indices back to land cover classes
        let land_cover: Vec<LandCoverClass> = predictions
            .iter()
            .map(|&idx| {
                self.class_mapping
                    .get(&idx)
                    .copied()
                    .unwrap_or(LandCoverClass::Unknown)
            })
            .collect();

        Ok(land_cover)
    }

    /// Classify with probabilities
    pub fn classify_with_confidence(
        &self,
        features: &FeatureSet,
    ) -> Result<(Vec<LandCoverClass>, Vec<f64>)> {
        let predictions = self.classifier.predict(features)?;
        let probabilities = self.classifier.predict_proba(features)?;

        let land_cover: Vec<LandCoverClass> = predictions
            .iter()
            .map(|&idx| {
                self.class_mapping
                    .get(&idx)
                    .copied()
                    .unwrap_or(LandCoverClass::Unknown)
            })
            .collect();

        let confidence: Vec<f64> = probabilities
            .iter()
            .map(|probs| probs.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
            .collect();

        Ok((land_cover, confidence))
    }

    /// Get training statistics
    pub fn training_stats(&self) -> Option<&TrainingStats> {
        self.training_stats.as_ref()
    }

    /// Apply post-processing
    pub fn post_process(&self, classification: Vec<LandCoverClass>, width: usize, height: usize) -> Result<Vec<LandCoverClass>> {
        if classification.len() != width * height {
            return Err(MlError::InvalidInput(
                "Classification size doesn't match dimensions".to_string(),
            ));
        }

        let mut result = classification.clone();

        // Apply majority filter if enabled
        if self.config.apply_majority_filter {
            result = self.apply_majority_filter(&result, width, height);
        }

        // Apply minimum mapping unit if configured
        if self.config.min_mapping_unit > 1 {
            result = self.apply_mmu(&result, width, height);
        }

        Ok(result)
    }

    /// Apply majority filter
    fn apply_majority_filter(&self, classification: &[LandCoverClass], width: usize, height: usize) -> Vec<LandCoverClass> {
        let mut filtered = classification.to_vec();
        let window = 1; // 3x3 window

        for y in window..height - window {
            for x in window..width - window {
                let mut class_counts: HashMap<LandCoverClass, usize> = HashMap::new();

                // Count classes in neighborhood
                for dy in -(window as isize)..=(window as isize) {
                    for dx in -(window as isize)..=(window as isize) {
                        let nx = (x as isize + dx) as usize;
                        let ny = (y as isize + dy) as usize;
                        let idx = ny * width + nx;
                        *class_counts.entry(classification[idx]).or_insert(0) += 1;
                    }
                }

                // Find majority class
                if let Some((&majority_class, _)) = class_counts.iter().max_by_key(|(_, &count)| count) {
                    filtered[y * width + x] = majority_class;
                }
            }
        }

        filtered
    }

    /// Apply minimum mapping unit
    fn apply_mmu(&self, classification: &[LandCoverClass], width: usize, height: usize) -> Vec<LandCoverClass> {
        // Simplified MMU: merge small regions into neighbors
        // A full implementation would use region growing/connected components
        classification.to_vec()
    }
}

impl Default for LandCoverClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_land_cover_class() {
        assert_eq!(LandCoverClass::Forest.name(), "Forest");
        assert_eq!(LandCoverClass::from_index(1), Some(LandCoverClass::Forest));
        assert_eq!(LandCoverClass::Forest.to_index(), 1);
    }

    #[test]
    fn test_land_cover_classifier() {
        let classifier = LandCoverClassifier::new();
        assert!(!classifier.classifier.is_trained());
    }
}
