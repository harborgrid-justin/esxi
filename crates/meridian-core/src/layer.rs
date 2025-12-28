//! Feature collection and layer management.
//!
//! This module provides the Layer type, which is a collection of features with
//! optional spatial indexing. Layers are the primary container for managing
//! groups of related spatial features.
//!
//! # Examples
//!
//! ```ignore
//! use meridian_core::layer::Layer;
//! use meridian_core::feature::Feature;
//! use meridian_core::geometry::{Point, Geometry};
//! use meridian_core::crs::Crs;
//!
//! let mut layer = Layer::new("cities", Crs::wgs84());
//!
//! // Add features
//! let sf = Point::new(-122.4194, 37.7749, Crs::wgs84());
//! layer.add_feature(Feature::new(Geometry::Point(sf)));
//!
//! // Enable spatial indexing
//! layer.build_index();
//!
//! // Query nearest feature
//! let nearest = layer.query_nearest(&[-122.4, 37.8]);
//! ```

use crate::bbox::BoundingBox;
use crate::crs::Crs;
use crate::error::Result;
use crate::feature::Feature;
use crate::spatial_index::SpatialIndex;
use crate::traits::{Bounded, Transformable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// A collection of features with optional spatial indexing.
///
/// A layer represents a thematic collection of geographic features, similar to
/// a GeoJSON FeatureCollection. Layers support spatial indexing for fast queries.
#[derive(Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Name of the layer
    pub name: String,

    /// Coordinate reference system for this layer
    pub crs: Crs,

    /// Features in the layer
    features: Vec<Feature>,

    /// Optional spatial index for fast queries
    #[serde(skip)]
    spatial_index: Option<SpatialIndex>,

    /// Metadata about the layer
    #[serde(default)]
    pub metadata: serde_json::Map<String, Value>,
}

impl Layer {
    /// Creates a new empty layer.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the layer
    /// * `crs` - Coordinate reference system
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use meridian_core::layer::Layer;
    /// use meridian_core::crs::Crs;
    ///
    /// let layer = Layer::new("roads", Crs::wgs84());
    /// ```
    pub fn new(name: impl Into<String>, crs: Crs) -> Self {
        Self {
            name: name.into(),
            crs,
            features: Vec::new(),
            spatial_index: None,
            metadata: serde_json::Map::new(),
        }
    }

    /// Creates a layer with an initial capacity.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the layer
    /// * `crs` - Coordinate reference system
    /// * `capacity` - Initial capacity for features
    pub fn with_capacity(name: impl Into<String>, crs: Crs, capacity: usize) -> Self {
        Self {
            name: name.into(),
            crs,
            features: Vec::with_capacity(capacity),
            spatial_index: None,
            metadata: serde_json::Map::new(),
        }
    }

    /// Creates a layer from a collection of features.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the layer
    /// * `crs` - Coordinate reference system
    /// * `features` - Vector of features
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let features = vec![feature1, feature2, feature3];
    /// let layer = Layer::from_features("my_layer", Crs::wgs84(), features);
    /// ```
    pub fn from_features(name: impl Into<String>, crs: Crs, features: Vec<Feature>) -> Self {
        Self {
            name: name.into(),
            crs,
            features,
            spatial_index: None,
            metadata: serde_json::Map::new(),
        }
    }

    /// Adds a feature to the layer.
    ///
    /// Note: If a spatial index exists, it will be invalidated and need to be rebuilt.
    ///
    /// # Arguments
    ///
    /// * `feature` - The feature to add
    ///
    /// # Examples
    ///
    /// ```ignore
    /// layer.add_feature(feature);
    /// ```
    pub fn add_feature(&mut self, feature: Feature) {
        self.features.push(feature);
        // Invalidate spatial index
        self.spatial_index = None;
    }

    /// Adds multiple features to the layer.
    ///
    /// # Arguments
    ///
    /// * `features` - Iterator of features to add
    pub fn add_features<I>(&mut self, features: I)
    where
        I: IntoIterator<Item = Feature>,
    {
        self.features.extend(features);
        // Invalidate spatial index
        self.spatial_index = None;
    }

    /// Returns the number of features in the layer.
    pub fn len(&self) -> usize {
        self.features.len()
    }

    /// Checks if the layer is empty.
    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }

    /// Gets a feature by index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the feature
    ///
    /// # Returns
    ///
    /// The feature if the index is valid, None otherwise
    pub fn get(&self, index: usize) -> Option<&Feature> {
        self.features.get(index)
    }

    /// Gets a mutable reference to a feature by index.
    ///
    /// Note: Modifying a feature will invalidate the spatial index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Feature> {
        self.spatial_index = None;
        self.features.get_mut(index)
    }

    /// Removes a feature by index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the feature to remove
    ///
    /// # Returns
    ///
    /// The removed feature if the index was valid
    pub fn remove(&mut self, index: usize) -> Option<Feature> {
        if index < self.features.len() {
            self.spatial_index = None;
            Some(self.features.remove(index))
        } else {
            None
        }
    }

    /// Returns an iterator over the features.
    pub fn iter(&self) -> impl Iterator<Item = &Feature> {
        self.features.iter()
    }

    /// Returns a mutable iterator over the features.
    ///
    /// Note: Using this iterator may invalidate the spatial index.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Feature> {
        self.spatial_index = None;
        self.features.iter_mut()
    }

    /// Clears all features from the layer.
    pub fn clear(&mut self) {
        self.features.clear();
        self.spatial_index = None;
    }

    /// Builds a spatial index for fast queries.
    ///
    /// This should be called after adding features and before performing spatial queries
    /// for optimal performance.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// layer.add_features(my_features);
    /// layer.build_index();
    /// let nearest = layer.query_nearest(&[10.0, 20.0]);
    /// ```
    pub fn build_index(&mut self) {
        let geometries = self.features.iter().map(|f| f.geometry.clone());
        self.spatial_index = Some(SpatialIndex::from_geometries(geometries));
    }

    /// Checks if the layer has a spatial index.
    pub fn has_index(&self) -> bool {
        self.spatial_index.is_some()
    }

    /// Ensures the spatial index exists, building it if necessary.
    fn ensure_index(&mut self) {
        if self.spatial_index.is_none() {
            self.build_index();
        }
    }

    /// Queries features that intersect with a bounding box.
    ///
    /// If no spatial index exists, one will be built automatically.
    ///
    /// # Arguments
    ///
    /// * `bbox` - The query bounding box
    ///
    /// # Returns
    ///
    /// Vector of features that intersect the bounding box
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use meridian_core::bbox::BoundingBox;
    ///
    /// let bbox = BoundingBox::new(-122.5, 37.7, -122.3, 37.9);
    /// let results = layer.query_bbox(&bbox);
    /// ```
    pub fn query_bbox(&mut self, bbox: &BoundingBox) -> Vec<&Feature> {
        self.ensure_index();

        if let Some(ref index) = self.spatial_index {
            index
                .query_bbox(bbox)
                .filter_map(|indexed| self.features.get(indexed.id as usize))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Queries the nearest feature to a point.
    ///
    /// If no spatial index exists, one will be built automatically.
    ///
    /// # Arguments
    ///
    /// * `point` - The query point [x, y]
    ///
    /// # Returns
    ///
    /// The nearest feature, or None if the layer is empty
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(nearest) = layer.query_nearest(&[-122.4, 37.8]) {
    ///     println!("Nearest feature: {}", nearest);
    /// }
    /// ```
    pub fn query_nearest(&mut self, point: &[f64; 2]) -> Option<&Feature> {
        self.ensure_index();

        if let Some(ref index) = self.spatial_index {
            index
                .nearest_neighbor_point(point)
                .and_then(|indexed| self.features.get(indexed.id as usize))
        } else {
            None
        }
    }

    /// Queries the k nearest features to a point.
    ///
    /// # Arguments
    ///
    /// * `point` - The query point [x, y]
    /// * `k` - Number of nearest features to return
    ///
    /// # Returns
    ///
    /// Vector of up to k nearest features
    pub fn query_k_nearest(&mut self, point: &[f64; 2], k: usize) -> Vec<&Feature> {
        self.ensure_index();

        if let Some(ref index) = self.spatial_index {
            index
                .k_nearest_neighbors(point, k)
                .iter()
                .filter_map(|indexed| self.features.get(indexed.id as usize))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Queries features within a distance of a point.
    ///
    /// # Arguments
    ///
    /// * `point` - The query point [x, y]
    /// * `distance` - The search radius
    ///
    /// # Returns
    ///
    /// Vector of features within the specified distance
    pub fn query_within_distance(&mut self, point: &[f64; 2], distance: f64) -> Vec<&Feature> {
        self.ensure_index();

        if let Some(ref index) = self.spatial_index {
            index
                .query_within_distance(point, distance)
                .into_iter()
                .filter_map(|indexed| self.features.get(indexed.id as usize))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Filters features by a predicate function.
    ///
    /// # Arguments
    ///
    /// * `predicate` - Function that returns true for features to keep
    ///
    /// # Returns
    ///
    /// A new layer containing only features that match the predicate
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Filter features with population > 100000
    /// let large_cities = layer.filter(|f| {
    ///     f.get_property_as::<i64>("population")
    ///         .map(|p| p > 100000)
    ///         .unwrap_or(false)
    /// });
    /// ```
    pub fn filter<F>(&self, predicate: F) -> Self
    where
        F: Fn(&Feature) -> bool,
    {
        let filtered_features: Vec<_> = self.features.iter().filter(|f| predicate(f)).cloned().collect();

        Layer::from_features(&self.name, self.crs.clone(), filtered_features)
    }

    /// Returns the bounding box of all features in the layer.
    pub fn bounds(&self) -> Option<BoundingBox> {
        if self.features.is_empty() {
            return None;
        }

        let mut bbox = self.features[0].bounds();
        for feature in &self.features[1..] {
            bbox.expand_to_include_bbox(&feature.bounds());
        }
        Some(bbox)
    }

    /// Transforms all features in the layer to a new CRS.
    ///
    /// # Arguments
    ///
    /// * `target_crs` - The target coordinate reference system
    ///
    /// # Returns
    ///
    /// A new layer with transformed features
    ///
    /// # Errors
    ///
    /// Returns an error if any feature transformation fails
    pub fn transform(&self, target_crs: &Crs) -> Result<Self> {
        let transformed_features: Result<Vec<_>> = self
            .features
            .iter()
            .map(|f| f.transform(target_crs))
            .collect();

        Ok(Layer::from_features(
            &self.name,
            target_crs.clone(),
            transformed_features?,
        ))
    }

    /// Transforms all features in place to a new CRS.
    ///
    /// # Arguments
    ///
    /// * `target_crs` - The target coordinate reference system
    ///
    /// # Errors
    ///
    /// Returns an error if any feature transformation fails
    pub fn transform_inplace(&mut self, target_crs: &Crs) -> Result<()> {
        for feature in &mut self.features {
            feature.transform_inplace(target_crs)?;
        }
        self.crs = target_crs.clone();
        self.spatial_index = None;
        Ok(())
    }

    /// Converts the layer to a GeoJSON FeatureCollection.
    ///
    /// # Returns
    ///
    /// A JSON Value representing a GeoJSON FeatureCollection
    pub fn to_geojson(&self) -> Result<Value> {
        let features: Result<Vec<_>> = self.features.iter().map(|f| f.to_geojson()).collect();

        Ok(serde_json::json!({
            "type": "FeatureCollection",
            "name": self.name,
            "features": features?,
        }))
    }

    /// Sets a metadata value.
    ///
    /// # Arguments
    ///
    /// * `key` - The metadata key
    /// * `value` - The metadata value
    pub fn set_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
    }

    /// Gets a metadata value.
    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }
}

impl fmt::Debug for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Layer")
            .field("name", &self.name)
            .field("crs", &self.crs)
            .field("feature_count", &self.features.len())
            .field("has_index", &self.has_index())
            .finish()
    }
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Layer '{}' [{} features, {}]",
            self.name,
            self.features.len(),
            self.crs
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::Crs;
    use crate::geometry::Point;
    use serde_json::json;

    #[test]
    fn test_layer_creation() {
        let layer = Layer::new("test", Crs::wgs84());
        assert_eq!(layer.name, "test");
        assert_eq!(layer.len(), 0);
        assert!(layer.is_empty());
    }

    #[test]
    fn test_layer_add_feature() {
        let mut layer = Layer::new("test", Crs::wgs84());
        let point = Point::new(10.0, 20.0, Crs::wgs84());
        let feature = Feature::new(Geometry::Point(point));

        layer.add_feature(feature);
        assert_eq!(layer.len(), 1);
    }

    #[test]
    fn test_layer_bounds() {
        let mut layer = Layer::new("test", Crs::wgs84());

        layer.add_feature(Feature::new(Geometry::Point(Point::new(0.0, 0.0, Crs::wgs84()))));
        layer.add_feature(Feature::new(Geometry::Point(Point::new(10.0, 10.0, Crs::wgs84()))));

        let bounds = layer.bounds().unwrap();
        assert_eq!(bounds.min_x, 0.0);
        assert_eq!(bounds.max_x, 10.0);
        assert_eq!(bounds.min_y, 0.0);
        assert_eq!(bounds.max_y, 10.0);
    }

    #[test]
    fn test_layer_filter() {
        let mut layer = Layer::new("test", Crs::wgs84());

        let mut f1 = Feature::new(Geometry::Point(Point::new(0.0, 0.0, Crs::wgs84())));
        f1.set_property("value", json!(5));

        let mut f2 = Feature::new(Geometry::Point(Point::new(10.0, 10.0, Crs::wgs84())));
        f2.set_property("value", json!(15));

        layer.add_feature(f1);
        layer.add_feature(f2);

        let filtered = layer.filter(|f| {
            f.get_property_as::<i64>("value")
                .map(|v| v > 10)
                .unwrap_or(false)
        });

        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_layer_spatial_index() {
        let mut layer = Layer::new("test", Crs::wgs84());

        layer.add_feature(Feature::new(Geometry::Point(Point::new(0.0, 0.0, Crs::wgs84()))));
        layer.add_feature(Feature::new(Geometry::Point(Point::new(10.0, 10.0, Crs::wgs84()))));

        layer.build_index();
        assert!(layer.has_index());

        let nearest = layer.query_nearest(&[1.0, 1.0]).unwrap();
        let bounds = nearest.bounds();
        assert_eq!(bounds.min_x, 0.0);
    }
}
