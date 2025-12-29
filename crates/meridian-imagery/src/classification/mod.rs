//! Image classification
//!
//! Supervised and unsupervised classification algorithms for land cover mapping.

pub mod supervised;
pub mod unsupervised;

pub use supervised::{SupervisedClassifier, MaximumLikelihood};
pub use unsupervised::{UnsupervisedClassifier, KMeans, Isodata};

use crate::error::Result;
use crate::MultiBandImage;

/// Classification result
#[derive(Debug, Clone)]
pub struct ClassificationResult {
    /// Class labels (one per pixel)
    pub labels: Vec<u32>,
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
    /// Class names
    pub class_names: Vec<String>,
    /// Confidence scores (optional)
    pub confidence: Option<Vec<f32>>,
}

impl ClassificationResult {
    /// Get class label at pixel coordinates
    pub fn get_label(&self, x: u32, y: u32) -> Option<u32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = (y * self.width + x) as usize;
        self.labels.get(idx).copied()
    }

    /// Get confidence at pixel coordinates
    pub fn get_confidence(&self, x: u32, y: u32) -> Option<f32> {
        if let Some(ref conf) = self.confidence {
            if x >= self.width || y >= self.height {
                return None;
            }
            let idx = (y * self.width + x) as usize;
            conf.get(idx).copied()
        } else {
            None
        }
    }

    /// Calculate class statistics
    pub fn class_statistics(&self) -> Vec<ClassStats> {
        let num_classes = self.class_names.len();
        let mut stats = vec![ClassStats::default(); num_classes];

        for &label in &self.labels {
            if (label as usize) < num_classes {
                stats[label as usize].count += 1;
            }
        }

        let total = self.labels.len() as f32;
        for stat in &mut stats {
            stat.percentage = (stat.count as f32 / total) * 100.0;
        }

        stats
    }
}

/// Class statistics
#[derive(Debug, Clone, Default)]
pub struct ClassStats {
    /// Number of pixels
    pub count: usize,
    /// Percentage of total area
    pub percentage: f32,
}

/// Training sample
#[derive(Debug, Clone)]
pub struct TrainingSample {
    /// Spectral values (one per band)
    pub values: Vec<f32>,
    /// Class label
    pub class: u32,
}

/// Training dataset
#[derive(Debug, Clone)]
pub struct TrainingData {
    /// Samples
    pub samples: Vec<TrainingSample>,
    /// Class names
    pub class_names: Vec<String>,
}

impl TrainingData {
    /// Create new training data
    pub fn new(class_names: Vec<String>) -> Self {
        Self {
            samples: Vec::new(),
            class_names,
        }
    }

    /// Add a training sample
    pub fn add_sample(&mut self, sample: TrainingSample) {
        self.samples.push(sample);
    }

    /// Get samples for a specific class
    pub fn get_class_samples(&self, class: u32) -> Vec<&TrainingSample> {
        self.samples.iter()
            .filter(|s| s.class == class)
            .collect()
    }

    /// Get number of classes
    pub fn num_classes(&self) -> usize {
        self.class_names.len()
    }
}
