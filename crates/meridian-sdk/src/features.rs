use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use crate::client::Client;
use crate::error::Result;

/// GeoJSON Feature
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Feature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub id: Option<JsonValue>,
    pub geometry: Option<Geometry>,
    pub properties: Option<JsonValue>,
}

/// GeoJSON Geometry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub geometry_type: String,
    pub coordinates: JsonValue,
}

/// Feature collection
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    pub collection_type: String,
    pub features: Vec<Feature>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_count: Option<u64>,
}

/// Feature creation request
#[derive(Debug, Clone, Serialize)]
pub struct CreateFeatureRequest {
    pub geometry: Geometry,
    pub properties: JsonValue,
}

/// Feature update request
#[derive(Debug, Clone, Serialize)]
pub struct UpdateFeatureRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<Geometry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<JsonValue>,
}

/// Client for feature operations
pub struct FeatureClient<'a> {
    client: &'a Client,
    layer_name: String,
}

impl<'a> FeatureClient<'a> {
    /// Create a new feature client
    pub(crate) fn new(client: &'a Client, layer_name: String) -> Self {
        Self { client, layer_name }
    }

    /// List all features in the layer
    pub async fn list(&self) -> Result<FeatureCollection> {
        let path = format!("/api/v1/layers/{}/features", self.layer_name);
        self.client.get(&path).await
    }

    /// Get a specific feature by ID
    pub async fn get(&self, feature_id: &str) -> Result<Feature> {
        let path = format!("/api/v1/layers/{}/features/{}", self.layer_name, feature_id);
        self.client.get(&path).await
    }

    /// Create a new feature
    pub async fn create(&self, request: CreateFeatureRequest) -> Result<Feature> {
        let path = format!("/api/v1/layers/{}/features", self.layer_name);
        self.client.post(&path, &request).await
    }

    /// Create multiple features in bulk
    pub async fn create_bulk(&self, requests: Vec<CreateFeatureRequest>) -> Result<FeatureCollection> {
        let path = format!("/api/v1/layers/{}/features/bulk", self.layer_name);
        self.client.post(&path, &requests).await
    }

    /// Update a feature
    pub async fn update(&self, feature_id: &str, request: UpdateFeatureRequest) -> Result<Feature> {
        let path = format!("/api/v1/layers/{}/features/{}", self.layer_name, feature_id);
        self.client.put(&path, &request).await
    }

    /// Delete a feature
    pub async fn delete(&self, feature_id: &str) -> Result<()> {
        let path = format!("/api/v1/layers/{}/features/{}", self.layer_name, feature_id);
        self.client.delete(&path).await
    }

    /// Delete multiple features by IDs
    pub async fn delete_bulk(&self, feature_ids: Vec<String>) -> Result<()> {
        let path = format!("/api/v1/layers/{}/features/bulk", self.layer_name);
        #[derive(Serialize)]
        struct DeleteRequest {
            ids: Vec<String>,
        }
        let request = DeleteRequest { ids: feature_ids };
        self.client.post(&path, &request).await
    }

    /// Count features in the layer
    pub async fn count(&self) -> Result<u64> {
        let path = format!("/api/v1/layers/{}/features/count", self.layer_name);
        #[derive(Deserialize)]
        struct CountResponse {
            count: u64,
        }
        let response: CountResponse = self.client.get(&path).await?;
        Ok(response.count)
    }
}

impl Feature {
    /// Create a new feature
    pub fn new(geometry: Geometry, properties: JsonValue) -> Self {
        Self {
            feature_type: "Feature".to_string(),
            id: None,
            geometry: Some(geometry),
            properties: Some(properties),
        }
    }

    /// Create a feature with an ID
    pub fn with_id(mut self, id: impl Into<JsonValue>) -> Self {
        self.id = Some(id.into());
        self
    }
}

impl Geometry {
    /// Create a Point geometry
    pub fn point(x: f64, y: f64) -> Self {
        Self {
            geometry_type: "Point".to_string(),
            coordinates: serde_json::json!([x, y]),
        }
    }

    /// Create a LineString geometry
    pub fn line_string(coordinates: Vec<[f64; 2]>) -> Self {
        Self {
            geometry_type: "LineString".to_string(),
            coordinates: serde_json::to_value(coordinates).unwrap(),
        }
    }

    /// Create a Polygon geometry
    pub fn polygon(rings: Vec<Vec<[f64; 2]>>) -> Self {
        Self {
            geometry_type: "Polygon".to_string(),
            coordinates: serde_json::to_value(rings).unwrap(),
        }
    }
}

impl CreateFeatureRequest {
    /// Create a new feature request
    pub fn new(geometry: Geometry, properties: JsonValue) -> Self {
        Self {
            geometry,
            properties,
        }
    }
}

impl UpdateFeatureRequest {
    /// Create a new update request
    pub fn new() -> Self {
        Self {
            geometry: None,
            properties: None,
        }
    }

    /// Set the geometry
    pub fn with_geometry(mut self, geometry: Geometry) -> Self {
        self.geometry = Some(geometry);
        self
    }

    /// Set the properties
    pub fn with_properties(mut self, properties: JsonValue) -> Self {
        self.properties = Some(properties);
        self
    }
}

impl Default for UpdateFeatureRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_geometry() {
        let geom = Geometry::point(10.0, 20.0);
        assert_eq!(geom.geometry_type, "Point");
    }

    #[test]
    fn test_feature_creation() {
        let geom = Geometry::point(10.0, 20.0);
        let props = serde_json::json!({"name": "Test Point"});
        let feature = Feature::new(geom, props);

        assert_eq!(feature.feature_type, "Feature");
        assert!(feature.geometry.is_some());
        assert!(feature.properties.is_some());
    }
}
