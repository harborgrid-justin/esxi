//! Spatial feature extraction

use crate::error::{MlError, Result};
use crate::features::{FeatureExtractor, FeatureInput, FeatureSet};
use geo::{Area, Centroid, EuclideanDistance, Point};
use geo_types::{Geometry, LineString, Polygon};
use ndarray::{Array1, Array2};
use serde::{Deserialize, Serialize};

/// Spatial features extracted from geometries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialFeatures {
    /// Feature matrix
    pub features: Array2<f64>,

    /// Feature names
    pub names: Vec<String>,

    /// Source geometries
    pub geometries: Vec<SpatialGeometry>,
}

/// Simplified geometry representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpatialGeometry {
    Point { x: f64, y: f64 },
    LineString { points: Vec<(f64, f64)> },
    Polygon { exterior: Vec<(f64, f64)>, holes: Vec<Vec<(f64, f64)>> },
}

/// Spatial feature extractor
pub struct SpatialFeatureExtractor {
    /// Features to extract
    feature_types: Vec<SpatialFeatureType>,

    /// Reference point for distance calculations
    reference_point: Option<(f64, f64)>,
}

/// Types of spatial features
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialFeatureType {
    /// X coordinate (centroid)
    CoordinateX,

    /// Y coordinate (centroid)
    CoordinateY,

    /// Area (for polygons)
    Area,

    /// Perimeter (for polygons)
    Perimeter,

    /// Length (for linestrings)
    Length,

    /// Distance to reference point
    DistanceToReference,

    /// Compactness ratio (area / perimeter^2)
    Compactness,

    /// Shape index
    ShapeIndex,

    /// Number of vertices
    VertexCount,

    /// Bounding box width
    BboxWidth,

    /// Bounding box height
    BboxHeight,

    /// Bounding box aspect ratio
    BboxAspectRatio,
}

impl SpatialFeatureType {
    /// Get the feature name
    pub fn name(&self) -> &str {
        match self {
            Self::CoordinateX => "coord_x",
            Self::CoordinateY => "coord_y",
            Self::Area => "area",
            Self::Perimeter => "perimeter",
            Self::Length => "length",
            Self::DistanceToReference => "dist_to_ref",
            Self::Compactness => "compactness",
            Self::ShapeIndex => "shape_index",
            Self::VertexCount => "vertex_count",
            Self::BboxWidth => "bbox_width",
            Self::BboxHeight => "bbox_height",
            Self::BboxAspectRatio => "bbox_aspect_ratio",
        }
    }

    /// Get all available feature types
    pub fn all() -> Vec<Self> {
        vec![
            Self::CoordinateX,
            Self::CoordinateY,
            Self::Area,
            Self::Perimeter,
            Self::Length,
            Self::DistanceToReference,
            Self::Compactness,
            Self::ShapeIndex,
            Self::VertexCount,
            Self::BboxWidth,
            Self::BboxHeight,
            Self::BboxAspectRatio,
        ]
    }
}

impl SpatialFeatureExtractor {
    /// Create a new spatial feature extractor
    pub fn new() -> Self {
        Self {
            feature_types: vec![
                SpatialFeatureType::CoordinateX,
                SpatialFeatureType::CoordinateY,
                SpatialFeatureType::Area,
            ],
            reference_point: None,
        }
    }

    /// Set feature types to extract
    pub fn with_features(mut self, types: Vec<SpatialFeatureType>) -> Self {
        self.feature_types = types;
        self
    }

    /// Set reference point for distance calculations
    pub fn with_reference_point(mut self, x: f64, y: f64) -> Self {
        self.reference_point = Some((x, y));
        self
    }

    /// Extract features from a polygon
    fn extract_polygon_features(&self, polygon: &Polygon<f64>) -> Result<Array1<f64>> {
        let mut features = Vec::new();

        for feature_type in &self.feature_types {
            let value = match feature_type {
                SpatialFeatureType::CoordinateX => {
                    polygon.centroid().map(|c| c.x()).unwrap_or(0.0)
                }
                SpatialFeatureType::CoordinateY => {
                    polygon.centroid().map(|c| c.y()).unwrap_or(0.0)
                }
                SpatialFeatureType::Area => polygon.unsigned_area(),
                SpatialFeatureType::Perimeter => {
                    let exterior_length: f64 = polygon
                        .exterior()
                        .lines()
                        .map(|line| {
                            let p1 = Point::new(line.start.x, line.start.y);
                            let p2 = Point::new(line.end.x, line.end.y);
                            p1.euclidean_distance(&p2)
                        })
                        .sum();
                    exterior_length
                }
                SpatialFeatureType::Compactness => {
                    let area = polygon.unsigned_area();
                    let perimeter: f64 = polygon
                        .exterior()
                        .lines()
                        .map(|line| {
                            let p1 = Point::new(line.start.x, line.start.y);
                            let p2 = Point::new(line.end.x, line.end.y);
                            p1.euclidean_distance(&p2)
                        })
                        .sum();
                    if perimeter > 0.0 {
                        4.0 * std::f64::consts::PI * area / (perimeter * perimeter)
                    } else {
                        0.0
                    }
                }
                SpatialFeatureType::VertexCount => polygon.exterior().coords_count() as f64,
                SpatialFeatureType::DistanceToReference => {
                    if let Some((ref_x, ref_y)) = self.reference_point {
                        let centroid = polygon.centroid().unwrap_or(Point::new(0.0, 0.0));
                        let ref_point = Point::new(ref_x, ref_y);
                        centroid.euclidean_distance(&ref_point)
                    } else {
                        0.0
                    }
                }
                SpatialFeatureType::BboxWidth => {
                    let coords: Vec<_> = polygon.exterior().coords().collect();
                    if coords.is_empty() {
                        0.0
                    } else {
                        let min_x = coords.iter().map(|c| c.x).fold(f64::INFINITY, f64::min);
                        let max_x = coords.iter().map(|c| c.x).fold(f64::NEG_INFINITY, f64::max);
                        max_x - min_x
                    }
                }
                SpatialFeatureType::BboxHeight => {
                    let coords: Vec<_> = polygon.exterior().coords().collect();
                    if coords.is_empty() {
                        0.0
                    } else {
                        let min_y = coords.iter().map(|c| c.y).fold(f64::INFINITY, f64::min);
                        let max_y = coords.iter().map(|c| c.y).fold(f64::NEG_INFINITY, f64::max);
                        max_y - min_y
                    }
                }
                SpatialFeatureType::BboxAspectRatio => {
                    let coords: Vec<_> = polygon.exterior().coords().collect();
                    if coords.is_empty() {
                        0.0
                    } else {
                        let min_x = coords.iter().map(|c| c.x).fold(f64::INFINITY, f64::min);
                        let max_x = coords.iter().map(|c| c.x).fold(f64::NEG_INFINITY, f64::max);
                        let min_y = coords.iter().map(|c| c.y).fold(f64::INFINITY, f64::min);
                        let max_y = coords.iter().map(|c| c.y).fold(f64::NEG_INFINITY, f64::max);
                        let width = max_x - min_x;
                        let height = max_y - min_y;
                        if height > 0.0 {
                            width / height
                        } else {
                            0.0
                        }
                    }
                }
                _ => 0.0,
            };
            features.push(value);
        }

        Ok(Array1::from_vec(features))
    }

    /// Extract features from geometries
    pub fn extract_from_geometries(&self, geometries: &[Polygon<f64>]) -> Result<FeatureSet> {
        if geometries.is_empty() {
            return Err(MlError::EmptyDataset);
        }

        let n_samples = geometries.len();
        let n_features = self.feature_types.len();
        let mut features = Array2::zeros((n_samples, n_features));

        for (i, geom) in geometries.iter().enumerate() {
            let feats = self.extract_polygon_features(geom)?;
            features.row_mut(i).assign(&feats);
        }

        let names: Vec<String> = self.feature_types.iter().map(|t| t.name().to_string()).collect();

        Ok(FeatureSet::new(features, names))
    }
}

impl Default for SpatialFeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureExtractor for SpatialFeatureExtractor {
    fn extract(&self, _input: &FeatureInput) -> Result<FeatureSet> {
        // Placeholder - would need to convert FeatureInput to geometries
        Err(MlError::FeatureExtraction(
            "Not implemented for generic FeatureInput".to_string(),
        ))
    }

    fn num_features(&self) -> usize {
        self.feature_types.len()
    }

    fn feature_names(&self) -> Vec<String> {
        self.feature_types.iter().map(|t| t.name().to_string()).collect()
    }
}

/// Calculate spatial autocorrelation (Moran's I)
pub fn morans_i(values: &[f64], coords: &[(f64, f64)]) -> Result<f64> {
    let n = values.len();
    if n != coords.len() {
        return Err(MlError::InvalidInput(
            "Values and coordinates must have same length".to_string(),
        ));
    }

    if n < 3 {
        return Err(MlError::InsufficientData {
            required: 3,
            actual: n,
        });
    }

    // Calculate mean
    let mean = values.iter().sum::<f64>() / n as f64;

    // Calculate spatial weights (inverse distance)
    let mut weights = Array2::zeros((n, n));
    for i in 0..n {
        for j in 0..n {
            if i != j {
                let dx = coords[i].0 - coords[j].0;
                let dy = coords[i].1 - coords[j].1;
                let dist = (dx * dx + dy * dy).sqrt();
                weights[[i, j]] = if dist > 0.0 { 1.0 / dist } else { 0.0 };
            }
        }
    }

    let w_sum: f64 = weights.sum();

    // Calculate Moran's I
    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for i in 0..n {
        for j in 0..n {
            numerator += weights[[i, j]] * (values[i] - mean) * (values[j] - mean);
        }
        denominator += (values[i] - mean).powi(2);
    }

    if denominator == 0.0 {
        return Err(MlError::DivisionByZero);
    }

    let morans_i = (n as f64 / w_sum) * (numerator / denominator);

    Ok(morans_i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_feature_extractor() {
        let extractor = SpatialFeatureExtractor::new();
        assert_eq!(extractor.num_features(), 3);
    }

    #[test]
    fn test_feature_type_names() {
        assert_eq!(SpatialFeatureType::CoordinateX.name(), "coord_x");
        assert_eq!(SpatialFeatureType::Area.name(), "area");
    }

    #[test]
    fn test_morans_i() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let coords = vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0), (4.0, 0.0)];

        let i = morans_i(&values, &coords).unwrap();
        // Should be positive for spatially autocorrelated data
        assert!(i > 0.0);
    }
}
