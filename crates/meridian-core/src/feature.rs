//! GeoJSON-style Feature with properties.
//!
//! This module provides a Feature type that combines geometry with arbitrary
//! properties, similar to GeoJSON features. Features are the primary data
//! structure for representing spatial objects with attributes.
//!
//! # Examples
//!
//! ```ignore
//! use meridian_core::feature::Feature;
//! use meridian_core::geometry::{Point, Geometry};
//! use meridian_core::crs::Crs;
//! use serde_json::json;
//!
//! let point = Point::new(-122.4194, 37.7749, Crs::wgs84());
//! let mut feature = Feature::new(Geometry::Point(point));
//!
//! // Add properties
//! feature.set_property("name", json!("San Francisco"));
//! feature.set_property("population", json!(883305));
//!
//! // Retrieve properties
//! let name = feature.get_property("name");
//! ```

use crate::bbox::BoundingBox;
use crate::crs::Crs;
use crate::error::{MeridianError, Result};
use crate::geometry::Geometry;
use crate::traits::{Bounded, Transformable};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt;

/// A geographic feature with geometry and properties.
///
/// Features combine a geometry with a set of named properties (attributes).
/// This is similar to the GeoJSON Feature specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    /// Optional unique identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,

    /// The geometry of the feature
    pub geometry: Geometry,

    /// Arbitrary properties (attributes) as key-value pairs
    #[serde(default)]
    pub properties: Map<String, Value>,
}

impl Feature {
    /// Creates a new feature with the given geometry and no properties.
    ///
    /// # Arguments
    ///
    /// * `geometry` - The geometry for this feature
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use meridian_core::feature::Feature;
    /// use meridian_core::geometry::{Point, Geometry};
    /// use meridian_core::crs::Crs;
    ///
    /// let point = Point::new(10.0, 20.0, Crs::wgs84());
    /// let feature = Feature::new(Geometry::Point(point));
    /// ```
    pub fn new(geometry: Geometry) -> Self {
        Self {
            id: None,
            geometry,
            properties: Map::new(),
        }
    }

    /// Creates a new feature with an ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The feature identifier
    /// * `geometry` - The geometry for this feature
    pub fn with_id(id: impl Into<Value>, geometry: Geometry) -> Self {
        Self {
            id: Some(id.into()),
            geometry,
            properties: Map::new(),
        }
    }

    /// Creates a new feature with geometry and properties.
    ///
    /// # Arguments
    ///
    /// * `geometry` - The geometry for this feature
    /// * `properties` - Initial properties map
    pub fn with_properties(geometry: Geometry, properties: Map<String, Value>) -> Self {
        Self {
            id: None,
            geometry,
            properties,
        }
    }

    /// Sets the feature ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The new ID value
    pub fn set_id(&mut self, id: impl Into<Value>) {
        self.id = Some(id.into());
    }

    /// Gets a property value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The property key
    ///
    /// # Returns
    ///
    /// The property value if it exists, None otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(name) = feature.get_property("name") {
    ///     println!("Feature name: {}", name);
    /// }
    /// ```
    pub fn get_property(&self, key: &str) -> Option<&Value> {
        self.properties.get(key)
    }

    /// Gets a property value as a specific type.
    ///
    /// # Arguments
    ///
    /// * `key` - The property key
    ///
    /// # Returns
    ///
    /// The typed property value, or an error if the property doesn't exist
    /// or cannot be deserialized to the target type
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let population: i64 = feature.get_property_as("population")?;
    /// let name: String = feature.get_property_as("name")?;
    /// ```
    pub fn get_property_as<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T> {
        let value = self
            .properties
            .get(key)
            .ok_or_else(|| MeridianError::PropertyError(format!("Property '{}' not found", key)))?;

        serde_json::from_value(value.clone())
            .map_err(|e| MeridianError::PropertyError(format!("Failed to deserialize property '{}': {}", key, e)))
    }

    /// Sets a property value.
    ///
    /// # Arguments
    ///
    /// * `key` - The property key
    /// * `value` - The property value (any JSON-serializable type)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// feature.set_property("name", json!("San Francisco"));
    /// feature.set_property("population", json!(883305));
    /// feature.set_property("is_capital", json!(false));
    /// ```
    pub fn set_property(&mut self, key: impl Into<String>, value: Value) {
        self.properties.insert(key.into(), value);
    }

    /// Removes a property.
    ///
    /// # Arguments
    ///
    /// * `key` - The property key to remove
    ///
    /// # Returns
    ///
    /// The removed value if it existed, None otherwise
    pub fn remove_property(&mut self, key: &str) -> Option<Value> {
        self.properties.remove(key)
    }

    /// Checks if a property exists.
    ///
    /// # Arguments
    ///
    /// * `key` - The property key to check
    ///
    /// # Returns
    ///
    /// `true` if the property exists
    pub fn has_property(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    /// Returns the number of properties.
    pub fn property_count(&self) -> usize {
        self.properties.len()
    }

    /// Returns an iterator over property keys.
    pub fn property_keys(&self) -> impl Iterator<Item = &String> {
        self.properties.keys()
    }

    /// Returns an iterator over property key-value pairs.
    pub fn properties_iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.properties.iter()
    }

    /// Clears all properties.
    pub fn clear_properties(&mut self) {
        self.properties.clear();
    }

    /// Converts this feature to a GeoJSON Value.
    ///
    /// # Returns
    ///
    /// A serde_json::Value representing the GeoJSON feature
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let geojson = feature.to_geojson()?;
    /// println!("{}", serde_json::to_string_pretty(&geojson)?);
    /// ```
    pub fn to_geojson(&self) -> Result<Value> {
        serde_json::to_value(self).map_err(|e| MeridianError::SerializationError(e.to_string()))
    }

    /// Creates a feature from a GeoJSON Value.
    ///
    /// # Arguments
    ///
    /// * `value` - A GeoJSON feature as a serde_json::Value
    ///
    /// # Returns
    ///
    /// A Feature instance
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a valid GeoJSON feature
    pub fn from_geojson(value: &Value) -> Result<Self> {
        serde_json::from_value(value.clone())
            .map_err(|e| MeridianError::DeserializationError(e.to_string()))
    }

    /// Returns a reference to the geometry.
    pub fn geometry(&self) -> &Geometry {
        &self.geometry
    }

    /// Returns a mutable reference to the geometry.
    pub fn geometry_mut(&mut self) -> &mut Geometry {
        &mut self.geometry
    }

    /// Sets the geometry.
    pub fn set_geometry(&mut self, geometry: Geometry) {
        self.geometry = geometry;
    }

    /// Clones the feature with a new geometry.
    ///
    /// # Arguments
    ///
    /// * `geometry` - The new geometry
    ///
    /// # Returns
    ///
    /// A new feature with the same ID and properties but different geometry
    pub fn with_geometry(&self, geometry: Geometry) -> Self {
        Self {
            id: self.id.clone(),
            geometry,
            properties: self.properties.clone(),
        }
    }
}

impl Bounded for Feature {
    fn bounds(&self) -> BoundingBox {
        self.geometry.bounds()
    }
}

impl Transformable for Feature {
    fn transform(&self, target_crs: &Crs) -> Result<Self> {
        Ok(Self {
            id: self.id.clone(),
            geometry: self.geometry.transform(target_crs)?,
            properties: self.properties.clone(),
        })
    }

    fn transform_inplace(&mut self, target_crs: &Crs) -> Result<()> {
        self.geometry.transform_inplace(target_crs)
    }
}

impl fmt::Display for Feature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref id) = self.id {
            write!(f, "Feature[id={}, properties={}]", id, self.properties.len())
        } else {
            write!(f, "Feature[properties={}]", self.properties.len())
        }
    }
}

/// A builder for constructing features with a fluent API.
///
/// # Examples
///
/// ```ignore
/// use meridian_core::feature::FeatureBuilder;
/// use meridian_core::geometry::{Point, Geometry};
/// use meridian_core::crs::Crs;
/// use serde_json::json;
///
/// let feature = FeatureBuilder::new(Geometry::Point(Point::new(10.0, 20.0, Crs::wgs84())))
///     .id(42)
///     .property("name", json!("Location A"))
///     .property("type", json!("marker"))
///     .build();
/// ```
pub struct FeatureBuilder {
    feature: Feature,
}

impl FeatureBuilder {
    /// Creates a new feature builder with the given geometry.
    pub fn new(geometry: Geometry) -> Self {
        Self {
            feature: Feature::new(geometry),
        }
    }

    /// Sets the feature ID.
    pub fn id(mut self, id: impl Into<Value>) -> Self {
        self.feature.set_id(id);
        self
    }

    /// Adds a property.
    pub fn property(mut self, key: impl Into<String>, value: Value) -> Self {
        self.feature.set_property(key, value);
        self
    }

    /// Adds multiple properties from a map.
    pub fn properties(mut self, properties: Map<String, Value>) -> Self {
        self.feature.properties = properties;
        self
    }

    /// Builds the feature.
    pub fn build(self) -> Feature {
        self.feature
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::Crs;
    use crate::geometry::Point;
    use serde_json::json;

    #[test]
    fn test_feature_creation() {
        let point = Point::new(10.0, 20.0, Crs::wgs84());
        let feature = Feature::new(Geometry::Point(point));

        assert!(feature.id.is_none());
        assert_eq!(feature.property_count(), 0);
    }

    #[test]
    fn test_feature_with_id() {
        let point = Point::new(10.0, 20.0, Crs::wgs84());
        let feature = Feature::with_id(42, Geometry::Point(point));

        assert_eq!(feature.id, Some(json!(42)));
    }

    #[test]
    fn test_feature_properties() {
        let point = Point::new(10.0, 20.0, Crs::wgs84());
        let mut feature = Feature::new(Geometry::Point(point));

        feature.set_property("name", json!("Test Point"));
        feature.set_property("value", json!(123));

        assert_eq!(feature.property_count(), 2);
        assert_eq!(feature.get_property("name"), Some(&json!("Test Point")));
        assert!(feature.has_property("name"));
        assert!(!feature.has_property("missing"));
    }

    #[test]
    fn test_feature_property_typed() {
        let point = Point::new(10.0, 20.0, Crs::wgs84());
        let mut feature = Feature::new(Geometry::Point(point));

        feature.set_property("name", json!("Test"));
        feature.set_property("count", json!(42));

        let name: String = feature.get_property_as("name").unwrap();
        assert_eq!(name, "Test");

        let count: i64 = feature.get_property_as("count").unwrap();
        assert_eq!(count, 42);
    }

    #[test]
    fn test_feature_remove_property() {
        let point = Point::new(10.0, 20.0, Crs::wgs84());
        let mut feature = Feature::new(Geometry::Point(point));

        feature.set_property("temp", json!("temporary"));
        assert!(feature.has_property("temp"));

        let removed = feature.remove_property("temp");
        assert_eq!(removed, Some(json!("temporary")));
        assert!(!feature.has_property("temp"));
    }

    #[test]
    fn test_feature_builder() {
        let point = Point::new(10.0, 20.0, Crs::wgs84());
        let feature = FeatureBuilder::new(Geometry::Point(point))
            .id(1)
            .property("name", json!("Test"))
            .property("value", json!(100))
            .build();

        assert_eq!(feature.id, Some(json!(1)));
        assert_eq!(feature.property_count(), 2);
        assert_eq!(feature.get_property("name"), Some(&json!("Test")));
    }

    #[test]
    fn test_feature_bounds() {
        let point = Point::new(10.0, 20.0, Crs::wgs84());
        let feature = Feature::new(Geometry::Point(point));
        let bounds = feature.bounds();

        assert_eq!(bounds.min_x, 10.0);
        assert_eq!(bounds.max_x, 10.0);
        assert_eq!(bounds.min_y, 20.0);
        assert_eq!(bounds.max_y, 20.0);
    }
}
