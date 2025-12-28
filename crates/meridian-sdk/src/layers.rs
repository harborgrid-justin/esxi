use serde::{Deserialize, Serialize};
use crate::client::Client;
use crate::error::Result;

/// Layer metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub geometry_type: String,
    pub srid: u32,
    pub feature_count: Option<u64>,
    pub bounds: Option<Bounds>,
    pub created_at: String,
    pub updated_at: String,
}

/// Bounding box
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Bounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

/// Layer creation request
#[derive(Debug, Clone, Serialize)]
pub struct CreateLayerRequest {
    pub name: String,
    pub description: Option<String>,
    pub geometry_type: String,
    pub srid: u32,
}

/// Layer update request
#[derive(Debug, Clone, Serialize)]
pub struct UpdateLayerRequest {
    pub description: Option<String>,
}

/// Client for layer operations
pub struct LayerClient<'a> {
    client: &'a Client,
}

impl<'a> LayerClient<'a> {
    /// Create a new layer client
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all layers
    pub async fn list(&self) -> Result<Vec<Layer>> {
        self.client.get("/api/v1/layers").await
    }

    /// Get a specific layer by name
    pub async fn get(&self, name: &str) -> Result<Layer> {
        let path = format!("/api/v1/layers/{}", name);
        self.client.get(&path).await
    }

    /// Create a new layer
    pub async fn create(&self, request: CreateLayerRequest) -> Result<Layer> {
        self.client.post("/api/v1/layers", &request).await
    }

    /// Update a layer
    pub async fn update(&self, name: &str, request: UpdateLayerRequest) -> Result<Layer> {
        let path = format!("/api/v1/layers/{}", name);
        self.client.put(&path, &request).await
    }

    /// Delete a layer
    pub async fn delete(&self, name: &str) -> Result<()> {
        let path = format!("/api/v1/layers/{}", name);
        self.client.delete(&path).await
    }

    /// Get layer statistics
    pub async fn stats(&self, name: &str) -> Result<LayerStats> {
        let path = format!("/api/v1/layers/{}/stats", name);
        self.client.get(&path).await
    }

    /// Get layer schema (attribute definitions)
    pub async fn schema(&self, name: &str) -> Result<LayerSchema> {
        let path = format!("/api/v1/layers/{}/schema", name);
        self.client.get(&path).await
    }
}

/// Layer statistics
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayerStats {
    pub feature_count: u64,
    pub bounds: Option<Bounds>,
    pub geometry_types: Vec<String>,
    pub srid: u32,
    pub total_size_bytes: u64,
}

/// Layer schema
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayerSchema {
    pub fields: Vec<FieldDefinition>,
}

/// Field definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: String,
    pub nullable: bool,
}

impl CreateLayerRequest {
    /// Create a new layer request
    pub fn new(name: impl Into<String>, geometry_type: impl Into<String>, srid: u32) -> Self {
        Self {
            name: name.into(),
            description: None,
            geometry_type: geometry_type.into(),
            srid,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl UpdateLayerRequest {
    /// Create a new update request
    pub fn new() -> Self {
        Self {
            description: None,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl Default for UpdateLayerRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_layer_request() {
        let request = CreateLayerRequest::new("test_layer", "Point", 4326)
            .with_description("Test layer");

        assert_eq!(request.name, "test_layer");
        assert_eq!(request.geometry_type, "Point");
        assert_eq!(request.srid, 4326);
        assert_eq!(request.description, Some("Test layer".to_string()));
    }
}
