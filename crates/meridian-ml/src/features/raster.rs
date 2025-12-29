//! Raster feature extraction

use crate::error::{MlError, Result};
use crate::features::{FeatureExtractor, FeatureInput, FeatureSet};
use ndarray::{Array1, Array2, ArrayView2};
use serde::{Deserialize, Serialize};

/// Raster features extracted from imagery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RasterFeatures {
    /// Feature matrix
    pub features: Array2<f64>,

    /// Feature names
    pub names: Vec<String>,

    /// Window size used for extraction
    pub window_size: usize,
}

/// Raster feature extractor
pub struct RasterFeatureExtractor {
    /// Window size for local statistics
    window_size: usize,

    /// Feature types to extract
    feature_types: Vec<RasterFeatureType>,

    /// Whether to extract texture features
    extract_texture: bool,
}

/// Types of raster features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterFeatureType {
    /// Mean value
    Mean,

    /// Standard deviation
    StdDev,

    /// Minimum value
    Min,

    /// Maximum value
    Max,

    /// Range (max - min)
    Range,

    /// Median value
    Median,

    /// Variance
    Variance,

    /// Skewness
    Skewness,

    /// Kurtosis
    Kurtosis,

    /// Entropy
    Entropy,

    /// Edge density
    EdgeDensity,
}

impl RasterFeatureType {
    /// Get the feature name
    pub fn name(&self) -> &str {
        match self {
            Self::Mean => "mean",
            Self::StdDev => "std_dev",
            Self::Min => "min",
            Self::Max => "max",
            Self::Range => "range",
            Self::Median => "median",
            Self::Variance => "variance",
            Self::Skewness => "skewness",
            Self::Kurtosis => "kurtosis",
            Self::Entropy => "entropy",
            Self::EdgeDensity => "edge_density",
        }
    }

    /// Get all basic feature types
    pub fn basic() -> Vec<Self> {
        vec![Self::Mean, Self::StdDev, Self::Min, Self::Max]
    }

    /// Get all statistical feature types
    pub fn statistical() -> Vec<Self> {
        vec![
            Self::Mean,
            Self::StdDev,
            Self::Min,
            Self::Max,
            Self::Range,
            Self::Median,
            Self::Variance,
            Self::Skewness,
            Self::Kurtosis,
        ]
    }
}

impl RasterFeatureExtractor {
    /// Create a new raster feature extractor
    pub fn new() -> Self {
        Self {
            window_size: 3,
            feature_types: RasterFeatureType::basic(),
            extract_texture: false,
        }
    }

    /// Set window size
    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size;
        self
    }

    /// Set feature types
    pub fn with_features(mut self, types: Vec<RasterFeatureType>) -> Self {
        self.feature_types = types;
        self
    }

    /// Enable texture feature extraction
    pub fn with_texture(mut self, enable: bool) -> Self {
        self.extract_texture = enable;
        self
    }

    /// Extract features from a single window
    fn extract_window_features(&self, window: ArrayView2<f64>) -> Array1<f64> {
        let mut features = Vec::new();

        for feature_type in &self.feature_types {
            let value = match feature_type {
                RasterFeatureType::Mean => {
                    window.mean().unwrap_or(0.0)
                }
                RasterFeatureType::StdDev => {
                    window.std(0.0)
                }
                RasterFeatureType::Min => {
                    window.iter().cloned().fold(f64::INFINITY, f64::min)
                }
                RasterFeatureType::Max => {
                    window.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
                }
                RasterFeatureType::Range => {
                    let min = window.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max = window.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    max - min
                }
                RasterFeatureType::Median => {
                    let mut values: Vec<f64> = window.iter().cloned().collect();
                    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let mid = values.len() / 2;
                    if values.len() % 2 == 0 {
                        (values[mid - 1] + values[mid]) / 2.0
                    } else {
                        values[mid]
                    }
                }
                RasterFeatureType::Variance => {
                    let mean = window.mean().unwrap_or(0.0);
                    let variance = window
                        .iter()
                        .map(|&x| (x - mean).powi(2))
                        .sum::<f64>()
                        / window.len() as f64;
                    variance
                }
                RasterFeatureType::Skewness => {
                    let mean = window.mean().unwrap_or(0.0);
                    let std = window.std(0.0);
                    if std > 0.0 {
                        let n = window.len() as f64;
                        let skewness = window
                            .iter()
                            .map(|&x| ((x - mean) / std).powi(3))
                            .sum::<f64>()
                            / n;
                        skewness
                    } else {
                        0.0
                    }
                }
                RasterFeatureType::Kurtosis => {
                    let mean = window.mean().unwrap_or(0.0);
                    let std = window.std(0.0);
                    if std > 0.0 {
                        let n = window.len() as f64;
                        let kurtosis = window
                            .iter()
                            .map(|&x| ((x - mean) / std).powi(4))
                            .sum::<f64>()
                            / n
                            - 3.0;
                        kurtosis
                    } else {
                        0.0
                    }
                }
                RasterFeatureType::Entropy => {
                    // Calculate Shannon entropy
                    let mut histogram = std::collections::HashMap::new();
                    for &value in window.iter() {
                        let bin = (value * 10.0).round() as i32;
                        *histogram.entry(bin).or_insert(0) += 1;
                    }
                    let total = window.len() as f64;
                    let entropy: f64 = histogram
                        .values()
                        .map(|&count| {
                            let p = count as f64 / total;
                            -p * p.log2()
                        })
                        .sum();
                    entropy
                }
                RasterFeatureType::EdgeDensity => {
                    // Simple edge detection using gradient magnitude
                    self.calculate_edge_density(window)
                }
            };
            features.push(value);
        }

        if self.extract_texture {
            // Add GLCM-based texture features
            features.extend(self.extract_texture_features(window));
        }

        Array1::from_vec(features)
    }

    /// Calculate edge density
    fn calculate_edge_density(&self, window: ArrayView2<f64>) -> f64 {
        let (rows, cols) = window.dim();
        if rows < 2 || cols < 2 {
            return 0.0;
        }

        let mut edge_count = 0.0;
        let threshold = 0.1;

        for i in 0..rows - 1 {
            for j in 0..cols - 1 {
                let gx = window[[i + 1, j]] - window[[i, j]];
                let gy = window[[i, j + 1]] - window[[i, j]];
                let magnitude = (gx * gx + gy * gy).sqrt();
                if magnitude > threshold {
                    edge_count += 1.0;
                }
            }
        }

        edge_count / ((rows - 1) * (cols - 1)) as f64
    }

    /// Extract texture features (simplified GLCM)
    fn extract_texture_features(&self, window: ArrayView2<f64>) -> Vec<f64> {
        // Simplified texture features - contrast, homogeneity, energy
        let mut features = Vec::new();

        // Normalize to 0-255 range
        let min_val = window.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = window.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max_val - min_val;

        if range == 0.0 {
            return vec![0.0, 1.0, 1.0]; // contrast, homogeneity, energy
        }

        // Build simplified co-occurrence matrix (horizontal, distance=1)
        let levels = 8; // Quantization levels
        let mut glcm = Array2::zeros((levels, levels));

        let (rows, cols) = window.dim();
        for i in 0..rows {
            for j in 0..cols - 1 {
                let val1 = ((window[[i, j]] - min_val) / range * (levels - 1) as f64) as usize;
                let val2 = ((window[[i, j + 1]] - min_val) / range * (levels - 1) as f64) as usize;
                glcm[[val1.min(levels - 1), val2.min(levels - 1)]] += 1.0;
            }
        }

        // Normalize GLCM
        let sum: f64 = glcm.sum();
        if sum > 0.0 {
            glcm /= sum;
        }

        // Calculate texture metrics
        let mut contrast = 0.0;
        let mut homogeneity = 0.0;
        let mut energy = 0.0;

        for i in 0..levels {
            for j in 0..levels {
                let p = glcm[[i, j]];
                contrast += ((i as i32 - j as i32).pow(2) as f64) * p;
                homogeneity += p / (1.0 + ((i as i32 - j as i32).abs() as f64));
                energy += p * p;
            }
        }

        features.push(contrast);
        features.push(homogeneity);
        features.push(energy);

        features
    }

    /// Extract features from raster data
    pub fn extract_from_raster(&self, raster: &Array2<f64>) -> Result<FeatureSet> {
        let (rows, cols) = raster.dim();
        let half_window = self.window_size / 2;

        if rows < self.window_size || cols < self.window_size {
            return Err(MlError::InvalidInput(format!(
                "Raster too small for window size {}",
                self.window_size
            )));
        }

        let mut features = Vec::new();

        // Extract features for each valid position
        for i in half_window..rows - half_window {
            for j in half_window..cols - half_window {
                let window = raster.slice(ndarray::s![
                    i - half_window..=i + half_window,
                    j - half_window..=j + half_window
                ]);
                features.push(self.extract_window_features(window));
            }
        }

        let n_samples = features.len();
        if n_samples == 0 {
            return Err(MlError::EmptyDataset);
        }

        let n_features = features[0].len();
        let mut feature_matrix = Array2::zeros((n_samples, n_features));

        for (i, feat) in features.iter().enumerate() {
            feature_matrix.row_mut(i).assign(feat);
        }

        let mut names: Vec<String> = self
            .feature_types
            .iter()
            .map(|t| t.name().to_string())
            .collect();

        if self.extract_texture {
            names.extend(vec![
                "texture_contrast".to_string(),
                "texture_homogeneity".to_string(),
                "texture_energy".to_string(),
            ]);
        }

        Ok(FeatureSet::new(feature_matrix, names))
    }
}

impl Default for RasterFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureExtractor for RasterFeatureExtractor {
    fn extract(&self, input: &FeatureInput) -> Result<FeatureSet> {
        match input {
            FeatureInput::Raster(raster) => self.extract_from_raster(raster),
            _ => Err(MlError::FeatureExtraction(
                "Expected raster input".to_string(),
            )),
        }
    }

    fn num_features(&self) -> usize {
        let base = self.feature_types.len();
        if self.extract_texture {
            base + 3
        } else {
            base
        }
    }

    fn feature_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .feature_types
            .iter()
            .map(|t| t.name().to_string())
            .collect();

        if self.extract_texture {
            names.extend(vec![
                "texture_contrast".to_string(),
                "texture_homogeneity".to_string(),
                "texture_energy".to_string(),
            ]);
        }

        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array;

    #[test]
    fn test_raster_feature_extractor() {
        let extractor = RasterFeatureExtractor::new();
        assert_eq!(extractor.window_size, 3);
    }

    #[test]
    fn test_feature_extraction() {
        let raster = Array::from_shape_fn((10, 10), |(i, j)| (i + j) as f64);
        let extractor = RasterFeatureExtractor::new();

        let features = extractor.extract_from_raster(&raster).unwrap();
        assert!(features.num_samples() > 0);
        assert_eq!(features.num_features(), 4); // mean, std, min, max
    }
}
