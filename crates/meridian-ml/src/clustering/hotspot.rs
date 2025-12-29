//! Hotspot analysis (Getis-Ord Gi*)

use crate::error::{MlError, Result};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Hotspot statistic types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HotspotStatistic {
    /// Getis-Ord Gi*
    GetisOrd,

    /// Local Moran's I
    LocalMorans,

    /// Local Geary's C
    LocalGearys,
}

/// Hotspot classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HotspotClass {
    /// Significant hotspot (high values, high clustering)
    HotSpot,

    /// Significant coldspot (low values, high clustering)
    ColdSpot,

    /// Not significant
    NotSignificant,

    /// High-low outlier
    HighLow,

    /// Low-high outlier
    LowHigh,
}

/// Hotspot analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotResult {
    /// Statistic values (z-scores)
    pub z_scores: Array1<f64>,

    /// P-values
    pub p_values: Array1<f64>,

    /// Hotspot classifications
    pub classifications: Vec<HotspotClass>,

    /// Significance level used
    pub significance_level: f64,
}

/// Hotspot analyzer
pub struct HotspotAnalyzer {
    /// Statistic type
    statistic: HotspotStatistic,

    /// Significance level (default 0.05)
    significance_level: f64,

    /// Distance threshold for neighbors
    distance_threshold: Option<f64>,

    /// Number of nearest neighbors
    k_neighbors: Option<usize>,
}

impl HotspotAnalyzer {
    /// Create a new hotspot analyzer
    pub fn new(statistic: HotspotStatistic) -> Self {
        Self {
            statistic,
            significance_level: 0.05,
            distance_threshold: None,
            k_neighbors: Some(8),
        }
    }

    /// Set significance level
    pub fn with_significance(mut self, level: f64) -> Self {
        self.significance_level = level;
        self
    }

    /// Set distance threshold for neighbors
    pub fn with_distance_threshold(mut self, threshold: f64) -> Self {
        self.distance_threshold = Some(threshold);
        self.k_neighbors = None;
        self
    }

    /// Set number of nearest neighbors
    pub fn with_k_neighbors(mut self, k: usize) -> Self {
        self.k_neighbors = Some(k);
        self.distance_threshold = None;
        self
    }

    /// Analyze hotspots
    pub fn analyze(&self, coords: &Array2<f64>, values: &Array1<f64>) -> Result<HotspotResult> {
        if coords.nrows() != values.len() {
            return Err(MlError::InvalidInput(
                "Coordinates and values must have same length".to_string(),
            ));
        }

        match self.statistic {
            HotspotStatistic::GetisOrd => self.getis_ord(coords, values),
            HotspotStatistic::LocalMorans => self.local_morans(coords, values),
            HotspotStatistic::LocalGearys => self.local_gearys(coords, values),
        }
    }

    /// Getis-Ord Gi* statistic
    fn getis_ord(&self, coords: &Array2<f64>, values: &Array1<f64>) -> Result<HotspotResult> {
        let n = coords.nrows();
        let mut z_scores = Array1::zeros(n);
        let mut p_values = Array1::zeros(n);

        // Build spatial weights matrix
        let weights = self.build_weights_matrix(coords)?;

        // Calculate mean and std of all values
        let mean = values.mean().unwrap_or(0.0);
        let std = values.std(0.0);

        if std == 0.0 {
            return Err(MlError::DivisionByZero);
        }

        // Calculate Gi* for each location
        for i in 0..n {
            let mut local_sum = 0.0;
            let mut w_sum = 0.0;
            let mut w_sq_sum = 0.0;

            for j in 0..n {
                let w = weights[[i, j]];
                local_sum += w * values[j];
                w_sum += w;
                w_sq_sum += w * w;
            }

            // Gi* formula
            let numerator = local_sum - mean * w_sum;
            let denominator = std * ((n as f64 * w_sq_sum - w_sum * w_sum) / (n as f64 - 1.0)).sqrt();

            z_scores[i] = if denominator != 0.0 {
                numerator / denominator
            } else {
                0.0
            };

            // Calculate p-value from z-score (two-tailed)
            p_values[i] = self.z_to_p_value(z_scores[i]);
        }

        // Classify hotspots
        let classifications = self.classify_hotspots(&z_scores, &p_values);

        Ok(HotspotResult {
            z_scores,
            p_values,
            classifications,
            significance_level: self.significance_level,
        })
    }

    /// Local Moran's I
    fn local_morans(&self, coords: &Array2<f64>, values: &Array1<f64>) -> Result<HotspotResult> {
        let n = coords.nrows();
        let mut z_scores = Array1::zeros(n);
        let mut p_values = Array1::zeros(n);

        let weights = self.build_weights_matrix(coords)?;
        let mean = values.mean().unwrap_or(0.0);
        let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n as f64;

        if variance == 0.0 {
            return Err(MlError::DivisionByZero);
        }

        // Calculate Local Moran's I for each location
        for i in 0..n {
            let zi = values[i] - mean;
            let mut local_sum = 0.0;

            for j in 0..n {
                let zj = values[j] - mean;
                local_sum += weights[[i, j]] * zj;
            }

            z_scores[i] = (zi / variance) * local_sum;
            p_values[i] = self.z_to_p_value(z_scores[i]);
        }

        let classifications = self.classify_hotspots(&z_scores, &p_values);

        Ok(HotspotResult {
            z_scores,
            p_values,
            classifications,
            significance_level: self.significance_level,
        })
    }

    /// Local Geary's C
    fn local_gearys(&self, coords: &Array2<f64>, values: &Array1<f64>) -> Result<HotspotResult> {
        let n = coords.nrows();
        let mut z_scores = Array1::zeros(n);
        let mut p_values = Array1::zeros(n);

        let weights = self.build_weights_matrix(coords)?;
        let mean = values.mean().unwrap_or(0.0);
        let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n as f64;

        if variance == 0.0 {
            return Err(MlError::DivisionByZero);
        }

        // Calculate Local Geary's C for each location
        for i in 0..n {
            let mut local_sum = 0.0;

            for j in 0..n {
                local_sum += weights[[i, j]] * (values[i] - values[j]).powi(2);
            }

            z_scores[i] = local_sum / variance;
            p_values[i] = self.z_to_p_value(z_scores[i]);
        }

        let classifications = self.classify_hotspots(&z_scores, &p_values);

        Ok(HotspotResult {
            z_scores,
            p_values,
            classifications,
            significance_level: self.significance_level,
        })
    }

    /// Build spatial weights matrix
    fn build_weights_matrix(&self, coords: &Array2<f64>) -> Result<Array2<f64>> {
        let n = coords.nrows();
        let mut weights = Array2::zeros((n, n));

        for i in 0..n {
            if let Some(threshold) = self.distance_threshold {
                // Distance threshold
                for j in 0..n {
                    if i != j {
                        let dist = self.euclidean_distance(coords.row(i), coords.row(j));
                        if dist <= threshold {
                            weights[[i, j]] = 1.0;
                        }
                    }
                }
            } else if let Some(k) = self.k_neighbors {
                // K nearest neighbors
                let mut distances: Vec<(usize, f64)> = (0..n)
                    .filter(|&j| j != i)
                    .map(|j| {
                        let dist = self.euclidean_distance(coords.row(i), coords.row(j));
                        (j, dist)
                    })
                    .collect();

                distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

                for (j, _) in distances.iter().take(k) {
                    weights[[i, *j]] = 1.0;
                }
            }

            // Row-normalize weights
            let row_sum: f64 = weights.row(i).sum();
            if row_sum > 0.0 {
                for j in 0..n {
                    weights[[i, j]] /= row_sum;
                }
            }
        }

        Ok(weights)
    }

    /// Calculate Euclidean distance
    fn euclidean_distance(&self, p1: ndarray::ArrayView1<f64>, p2: ndarray::ArrayView1<f64>) -> f64 {
        p1.iter()
            .zip(p2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    /// Convert z-score to p-value (approximation)
    fn z_to_p_value(&self, z: f64) -> f64 {
        // Simplified normal CDF approximation
        let abs_z = z.abs();
        if abs_z > 6.0 {
            return 0.0;
        }

        // Using error function approximation
        let t = 1.0 / (1.0 + 0.2316419 * abs_z);
        let d = 0.3989423 * (-abs_z * abs_z / 2.0).exp();
        let p = d * t * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));

        2.0 * p // Two-tailed
    }

    /// Classify hotspots based on z-scores and p-values
    fn classify_hotspots(&self, z_scores: &Array1<f64>, p_values: &Array1<f64>) -> Vec<HotspotClass> {
        z_scores
            .iter()
            .zip(p_values.iter())
            .map(|(&z, &p)| {
                if p > self.significance_level {
                    HotspotClass::NotSignificant
                } else if z > 0.0 {
                    HotspotClass::HotSpot
                } else {
                    HotspotClass::ColdSpot
                }
            })
            .collect()
    }
}

impl Default for HotspotAnalyzer {
    fn default() -> Self {
        Self::new(HotspotStatistic::GetisOrd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn test_hotspot_analyzer() {
        let analyzer = HotspotAnalyzer::new(HotspotStatistic::GetisOrd);
        assert_eq!(analyzer.significance_level, 0.05);
    }
}
