//! File-based tile sources (GeoJSON, FlatGeobuf, etc.)

use crate::encoding::mvt::MvtValue;
use crate::error::{Error, Result};
use crate::generation::SourceFeature;
use crate::source::{TileSource, SourceMetadata};
use crate::tile::bounds::MercatorBounds;
use crate::tile::coordinate::TileCoordinate;
use async_trait::async_trait;
use geo_types::Geometry;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// File-based tile source
pub struct FileSource {
    path: PathBuf,
    layer_name: String,
    features: Vec<SourceFeature>,
}

impl FileSource {
    /// Create a new file source from GeoJSON
    pub async fn from_geojson<P: AsRef<Path>>(
        path: P,
        layer_name: String,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let content = tokio::fs::read_to_string(&path).await?;
        let features = Self::parse_geojson(&content, &layer_name)?;

        Ok(Self {
            path,
            layer_name,
            features,
        })
    }

    /// Parse GeoJSON content
    fn parse_geojson(content: &str, layer_name: &str) -> Result<Vec<SourceFeature>> {
        let geojson: serde_json::Value = serde_json::from_str(content)?;
        let mut features = Vec::new();

        if let Some(feature_array) = geojson.get("features").and_then(|f| f.as_array()) {
            for (idx, feature) in feature_array.iter().enumerate() {
                if let Some(source_feature) = Self::parse_geojson_feature(feature, layer_name, idx)? {
                    features.push(source_feature);
                }
            }
        }

        Ok(features)
    }

    /// Parse a single GeoJSON feature
    fn parse_geojson_feature(
        feature: &serde_json::Value,
        layer_name: &str,
        index: usize,
    ) -> Result<Option<SourceFeature>> {
        // Get geometry
        let geom_value = match feature.get("geometry") {
            Some(g) => g,
            None => return Ok(None),
        };

        let geometry = Self::parse_geojson_geometry(geom_value)?;

        // Get ID
        let id = feature
            .get("id")
            .and_then(|id| id.as_u64())
            .or(Some(index as u64));

        // Get properties
        let mut properties = HashMap::new();
        if let Some(props) = feature.get("properties").and_then(|p| p.as_object()) {
            for (key, value) in props {
                if let Some(mvt_value) = Self::json_to_mvt_value(value) {
                    properties.insert(key.clone(), mvt_value);
                }
            }
        }

        Ok(Some(SourceFeature {
            id,
            layer: layer_name.to_string(),
            geometry,
            properties,
        }))
    }

    /// Parse GeoJSON geometry
    fn parse_geojson_geometry(geom: &serde_json::Value) -> Result<Geometry> {
        let geom_str = serde_json::to_string(geom)?;
        let geojson_geom: geojson::Geometry = serde_json::from_str(&geom_str)?;

        geojson_geom
            .try_into()
            .map_err(|e| Error::geometry(format!("Failed to convert GeoJSON geometry: {:?}", e)))
    }

    /// Convert JSON value to MVT value
    fn json_to_mvt_value(value: &serde_json::Value) -> Option<MvtValue> {
        match value {
            serde_json::Value::String(s) => Some(MvtValue::String(s.clone())),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(MvtValue::Int(i))
                } else if let Some(f) = n.as_f64() {
                    Some(MvtValue::Double(f))
                } else {
                    None
                }
            }
            serde_json::Value::Bool(b) => Some(MvtValue::Bool(*b)),
            _ => None,
        }
    }

    /// Filter features by bounds
    fn filter_by_bounds(
        &self,
        bounds: &MercatorBounds,
    ) -> Vec<SourceFeature> {
        self.features
            .iter()
            .filter(|f| Self::geometry_intersects(&f.geometry, bounds))
            .cloned()
            .collect()
    }

    /// Check if geometry intersects bounds (simple bbox check)
    fn geometry_intersects(geometry: &Geometry, bounds: &MercatorBounds) -> bool {
        use geo::algorithm::bounding_rect::BoundingRect;

        if let Some(bbox) = geometry.bounding_rect() {
            !(bbox.max().x < bounds.min_x
                || bbox.min().x > bounds.max_x
                || bbox.max().y < bounds.min_y
                || bbox.min().y > bounds.max_y)
        } else {
            false
        }
    }
}

#[async_trait]
impl TileSource for FileSource {
    async fn get_features(
        &self,
        _tile: TileCoordinate,
        bounds: &MercatorBounds,
    ) -> Result<Vec<SourceFeature>> {
        Ok(self.filter_by_bounds(bounds))
    }

    async fn layers(&self) -> Result<Vec<String>> {
        Ok(vec![self.layer_name.clone()])
    }

    async fn metadata(&self) -> Result<SourceMetadata> {
        Ok(SourceMetadata {
            name: self.layer_name.clone(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_mvt_value() {
        let s = serde_json::json!("test");
        assert!(matches!(
            FileSource::json_to_mvt_value(&s),
            Some(MvtValue::String(_))
        ));

        let n = serde_json::json!(42);
        assert!(matches!(
            FileSource::json_to_mvt_value(&n),
            Some(MvtValue::Int(_))
        ));

        let b = serde_json::json!(true);
        assert!(matches!(
            FileSource::json_to_mvt_value(&b),
            Some(MvtValue::Bool(_))
        ));
    }
}
