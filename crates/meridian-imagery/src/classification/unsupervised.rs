//! Unsupervised classification algorithms

use crate::error::{ImageryError, Result};
use crate::MultiBandImage;
use super::ClassificationResult;
use rayon::prelude::*;

/// Unsupervised classifier trait
pub trait UnsupervisedClassifier {
    /// Perform classification
    fn classify(&mut self, image: &MultiBandImage, num_classes: usize) -> Result<ClassificationResult>;
}

/// K-Means clustering classifier
pub struct KMeans {
    max_iterations: usize,
    convergence_threshold: f32,
}

impl KMeans {
    /// Create a new K-Means classifier
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
            convergence_threshold: 0.01,
        }
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, iterations: usize) -> Self {
        self.max_iterations = iterations;
        self
    }

    /// Set convergence threshold
    pub fn with_convergence_threshold(mut self, threshold: f32) -> Self {
        self.convergence_threshold = threshold;
        self
    }

    /// Initialize cluster centers randomly
    fn initialize_centers(
        image: &MultiBandImage,
        num_classes: usize,
    ) -> Vec<Vec<f32>> {
        let mut centers = Vec::with_capacity(num_classes);
        let num_bands = image.bands.len();
        let size = (image.metadata.width * image.metadata.height) as usize;

        // Use evenly spaced pixels as initial centers
        for i in 0..num_classes {
            let pixel_idx = (i * size) / num_classes;
            let mut center = Vec::with_capacity(num_bands);

            for band in &image.bands {
                center.push(band[pixel_idx]);
            }

            centers.push(center);
        }

        centers
    }

    /// Assign pixels to nearest cluster
    fn assign_clusters(
        image: &MultiBandImage,
        centers: &[Vec<f32>],
    ) -> Vec<u32> {
        let size = (image.metadata.width * image.metadata.height) as usize;
        let mut labels = vec![0u32; size];

        for pixel_idx in 0..size {
            let mut pixel_values = Vec::with_capacity(image.bands.len());
            for band in &image.bands {
                pixel_values.push(band[pixel_idx]);
            }

            // Find nearest center
            let mut min_dist = f32::INFINITY;
            let mut best_cluster = 0u32;

            for (cluster, center) in centers.iter().enumerate() {
                let dist = Self::euclidean_distance(&pixel_values, center);
                if dist < min_dist {
                    min_dist = dist;
                    best_cluster = cluster as u32;
                }
            }

            labels[pixel_idx] = best_cluster;
        }

        labels
    }

    /// Update cluster centers
    fn update_centers(
        image: &MultiBandImage,
        labels: &[u32],
        num_classes: usize,
    ) -> Vec<Vec<f32>> {
        let num_bands = image.bands.len();
        let size = labels.len();

        let mut new_centers = vec![vec![0.0; num_bands]; num_classes];
        let mut counts = vec![0usize; num_classes];

        for pixel_idx in 0..size {
            let cluster = labels[pixel_idx] as usize;
            counts[cluster] += 1;

            for (band_idx, band) in image.bands.iter().enumerate() {
                new_centers[cluster][band_idx] += band[pixel_idx];
            }
        }

        // Average
        for (cluster, count) in counts.iter().enumerate() {
            if *count > 0 {
                for band_idx in 0..num_bands {
                    new_centers[cluster][band_idx] /= *count as f32;
                }
            }
        }

        new_centers
    }

    /// Calculate Euclidean distance
    fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    /// Calculate change in centers
    fn center_change(old_centers: &[Vec<f32>], new_centers: &[Vec<f32>]) -> f32 {
        old_centers.iter()
            .zip(new_centers.iter())
            .map(|(old, new)| Self::euclidean_distance(old, new))
            .sum::<f32>() / old_centers.len() as f32
    }
}

impl Default for KMeans {
    fn default() -> Self {
        Self::new()
    }
}

impl UnsupervisedClassifier for KMeans {
    fn classify(&mut self, image: &MultiBandImage, num_classes: usize) -> Result<ClassificationResult> {
        if num_classes == 0 {
            return Err(ImageryError::Classification("Number of classes must be > 0".to_string()));
        }

        log::info!("Starting K-Means classification with {} classes", num_classes);

        // Initialize centers
        let mut centers = Self::initialize_centers(image, num_classes);

        // Iterate
        for iteration in 0..self.max_iterations {
            // Assign pixels to clusters
            let labels = Self::assign_clusters(image, &centers);

            // Update centers
            let new_centers = Self::update_centers(image, &labels, num_classes);

            // Check convergence
            let change = Self::center_change(&centers, &new_centers);
            log::debug!("Iteration {}: center change = {}", iteration, change);

            if change < self.convergence_threshold {
                log::info!("K-Means converged at iteration {}", iteration);
                centers = new_centers;
                break;
            }

            centers = new_centers;
        }

        // Final assignment
        let labels = Self::assign_clusters(image, &centers);

        let class_names = (0..num_classes)
            .map(|i| format!("Class {}", i + 1))
            .collect();

        Ok(ClassificationResult {
            labels,
            width: image.metadata.width,
            height: image.metadata.height,
            class_names,
            confidence: None,
        })
    }
}

/// ISODATA clustering classifier
pub struct Isodata {
    max_iterations: usize,
    convergence_threshold: f32,
    min_class_size: usize,
    max_std_dev: f32,
}

impl Isodata {
    /// Create a new ISODATA classifier
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
            convergence_threshold: 0.01,
            min_class_size: 10,
            max_std_dev: 10.0,
        }
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, iterations: usize) -> Self {
        self.max_iterations = iterations;
        self
    }

    /// Set minimum class size
    pub fn with_min_class_size(mut self, size: usize) -> Self {
        self.min_class_size = size;
        self
    }
}

impl Default for Isodata {
    fn default() -> Self {
        Self::new()
    }
}

impl UnsupervisedClassifier for Isodata {
    fn classify(&mut self, image: &MultiBandImage, num_classes: usize) -> Result<ClassificationResult> {
        // ISODATA is similar to K-Means but with split/merge operations
        // For now, use K-Means as base implementation

        let mut kmeans = KMeans::new()
            .with_max_iterations(self.max_iterations)
            .with_convergence_threshold(self.convergence_threshold);

        kmeans.classify(image, num_classes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ImageMetadata, DataType};

    #[test]
    fn test_kmeans_classification() {
        let metadata = ImageMetadata {
            width: 10,
            height: 10,
            bands: 3,
            bits_per_sample: 8,
            geo_transform: None,
            crs: None,
            no_data: None,
            band_names: vec!["R".to_string(), "G".to_string(), "B".to_string()],
        };

        let image = MultiBandImage::new(metadata, DataType::UInt8);
        let mut classifier = KMeans::new();

        let result = classifier.classify(&image, 5);
        assert!(result.is_ok());

        let classification = result.unwrap();
        assert_eq!(classification.labels.len(), 100);
        assert_eq!(classification.class_names.len(), 5);
    }
}
