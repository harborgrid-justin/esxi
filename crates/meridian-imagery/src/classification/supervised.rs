//! Supervised classification algorithms

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;
use super::{ClassificationResult, TrainingData, TrainingSample};

/// Supervised classifier trait
pub trait SupervisedClassifier {
    /// Train the classifier
    fn train(&mut self, training_data: &TrainingData) -> Result<()>;

    /// Classify an image
    fn classify(&self, image: &MultiBandImage) -> Result<ClassificationResult>;
}

/// Maximum Likelihood classifier
pub struct MaximumLikelihood {
    means: Vec<Vec<f32>>,
    covariances: Vec<Vec<Vec<f32>>>,
    class_names: Vec<String>,
    trained: bool,
}

impl MaximumLikelihood {
    /// Create a new Maximum Likelihood classifier
    pub fn new() -> Self {
        Self {
            means: Vec::new(),
            covariances: Vec::new(),
            class_names: Vec::new(),
            trained: false,
        }
    }

    /// Calculate mean vector for a class
    fn calculate_mean(samples: &[&TrainingSample], num_bands: usize) -> Vec<f32> {
        let mut mean = vec![0.0; num_bands];
        let n = samples.len() as f32;

        for sample in samples {
            for (i, &val) in sample.values.iter().enumerate() {
                mean[i] += val / n;
            }
        }

        mean
    }

    /// Calculate covariance matrix for a class
    fn calculate_covariance(
        samples: &[&TrainingSample],
        mean: &[f32],
        num_bands: usize,
    ) -> Vec<Vec<f32>> {
        let n = samples.len() as f32;
        let mut cov = vec![vec![0.0; num_bands]; num_bands];

        for sample in samples {
            for i in 0..num_bands {
                for j in 0..num_bands {
                    let diff_i = sample.values[i] - mean[i];
                    let diff_j = sample.values[j] - mean[j];
                    cov[i][j] += (diff_i * diff_j) / n;
                }
            }
        }

        cov
    }

    /// Calculate discriminant function
    fn discriminant(&self, pixel: &[f32], class: usize) -> f32 {
        let mean = &self.means[class];
        let cov = &self.covariances[class];

        // Simplified: -0.5 * (x - μ)^T * Σ^-1 * (x - μ)
        // For full implementation, would need matrix inversion

        let mut sum = 0.0;
        for i in 0..pixel.len() {
            let diff = pixel[i] - mean[i];
            // Simplified: use diagonal elements only
            if cov[i][i] > 0.0 {
                sum += diff * diff / cov[i][i];
            }
        }

        -0.5 * sum
    }
}

impl Default for MaximumLikelihood {
    fn default() -> Self {
        Self::new()
    }
}

impl SupervisedClassifier for MaximumLikelihood {
    fn train(&mut self, training_data: &TrainingData) -> Result<()> {
        if training_data.samples.is_empty() {
            return Err(ImageryError::Classification("No training samples provided".to_string()));
        }

        let num_classes = training_data.num_classes();
        let num_bands = training_data.samples[0].values.len();

        self.class_names = training_data.class_names.clone();
        self.means = Vec::with_capacity(num_classes);
        self.covariances = Vec::with_capacity(num_classes);

        for class in 0..num_classes {
            let class_samples = training_data.get_class_samples(class as u32);

            if class_samples.is_empty() {
                return Err(ImageryError::Classification(
                    format!("No samples for class {}", class)
                ));
            }

            let mean = Self::calculate_mean(&class_samples, num_bands);
            let cov = Self::calculate_covariance(&class_samples, &mean, num_bands);

            self.means.push(mean);
            self.covariances.push(cov);
        }

        self.trained = true;
        Ok(())
    }

    fn classify(&self, image: &MultiBandImage) -> Result<ClassificationResult> {
        if !self.trained {
            return Err(ImageryError::Classification("Classifier not trained".to_string()));
        }

        let size = (image.metadata.width * image.metadata.height) as usize;
        let mut labels = vec![0u32; size];
        let mut confidence = vec![0.0f32; size];

        for pixel_idx in 0..size {
            let mut pixel_values = Vec::with_capacity(image.bands.len());
            for band in &image.bands {
                pixel_values.push(band[pixel_idx]);
            }

            // Find class with maximum discriminant
            let mut max_disc = f32::NEG_INFINITY;
            let mut best_class = 0u32;

            for class in 0..self.means.len() {
                let disc = self.discriminant(&pixel_values, class);
                if disc > max_disc {
                    max_disc = disc;
                    best_class = class as u32;
                }
            }

            labels[pixel_idx] = best_class;
            confidence[pixel_idx] = max_disc.exp(); // Convert to probability-like value
        }

        Ok(ClassificationResult {
            labels,
            width: image.metadata.width,
            height: image.metadata.height,
            class_names: self.class_names.clone(),
            confidence: Some(confidence),
        })
    }
}

/// Minimum Distance classifier
pub struct MinimumDistance {
    means: Vec<Vec<f32>>,
    class_names: Vec<String>,
    trained: bool,
}

impl MinimumDistance {
    /// Create a new Minimum Distance classifier
    pub fn new() -> Self {
        Self {
            means: Vec::new(),
            class_names: Vec::new(),
            trained: false,
        }
    }

    /// Calculate Euclidean distance
    fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

impl Default for MinimumDistance {
    fn default() -> Self {
        Self::new()
    }
}

impl SupervisedClassifier for MinimumDistance {
    fn train(&mut self, training_data: &TrainingData) -> Result<()> {
        if training_data.samples.is_empty() {
            return Err(ImageryError::Classification("No training samples provided".to_string()));
        }

        let num_classes = training_data.num_classes();
        let num_bands = training_data.samples[0].values.len();

        self.class_names = training_data.class_names.clone();
        self.means = Vec::with_capacity(num_classes);

        for class in 0..num_classes {
            let class_samples = training_data.get_class_samples(class as u32);

            if class_samples.is_empty() {
                return Err(ImageryError::Classification(
                    format!("No samples for class {}", class)
                ));
            }

            let mean = MaximumLikelihood::calculate_mean(&class_samples, num_bands);
            self.means.push(mean);
        }

        self.trained = true;
        Ok(())
    }

    fn classify(&self, image: &MultiBandImage) -> Result<ClassificationResult> {
        if !self.trained {
            return Err(ImageryError::Classification("Classifier not trained".to_string()));
        }

        let size = (image.metadata.width * image.metadata.height) as usize;
        let mut labels = vec![0u32; size];

        for pixel_idx in 0..size {
            let mut pixel_values = Vec::with_capacity(image.bands.len());
            for band in &image.bands {
                pixel_values.push(band[pixel_idx]);
            }

            // Find class with minimum distance
            let mut min_dist = f32::INFINITY;
            let mut best_class = 0u32;

            for (class, mean) in self.means.iter().enumerate() {
                let dist = Self::euclidean_distance(&pixel_values, mean);
                if dist < min_dist {
                    min_dist = dist;
                    best_class = class as u32;
                }
            }

            labels[pixel_idx] = best_class;
        }

        Ok(ClassificationResult {
            labels,
            width: image.metadata.width,
            height: image.metadata.height,
            class_names: self.class_names.clone(),
            confidence: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maximum_likelihood_training() {
        let mut classifier = MaximumLikelihood::new();
        let mut training_data = TrainingData::new(vec!["Class1".to_string(), "Class2".to_string()]);

        training_data.add_sample(TrainingSample {
            values: vec![10.0, 20.0],
            class: 0,
        });

        training_data.add_sample(TrainingSample {
            values: vec![100.0, 200.0],
            class: 1,
        });

        let result = classifier.train(&training_data);
        assert!(result.is_ok());
        assert!(classifier.trained);
    }
}
