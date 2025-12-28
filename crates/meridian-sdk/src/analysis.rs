use serde::{Deserialize, Serialize};
use crate::client::Client;
use crate::features::{Feature, FeatureCollection};
use crate::error::Result;

/// Client for spatial analysis operations
pub struct AnalysisClient<'a> {
    client: &'a Client,
}

impl<'a> AnalysisClient<'a> {
    /// Create a new analysis client
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a buffer around geometries
    pub async fn buffer(&self, request: BufferRequest) -> Result<FeatureCollection> {
        self.client.post("/api/v1/analysis/buffer", &request).await
    }

    /// Clip geometries by a boundary
    pub async fn clip(&self, request: ClipRequest) -> Result<FeatureCollection> {
        self.client.post("/api/v1/analysis/clip", &request).await
    }

    /// Union geometries together
    pub async fn union(&self, request: UnionRequest) -> Result<Feature> {
        self.client.post("/api/v1/analysis/union", &request).await
    }

    /// Intersect two layers
    pub async fn intersect(&self, request: IntersectRequest) -> Result<FeatureCollection> {
        self.client.post("/api/v1/analysis/intersect", &request).await
    }

    /// Difference operation (subtract one geometry from another)
    pub async fn difference(&self, request: DifferenceRequest) -> Result<FeatureCollection> {
        self.client.post("/api/v1/analysis/difference", &request).await
    }

    /// Simplify geometries
    pub async fn simplify(&self, request: SimplifyRequest) -> Result<FeatureCollection> {
        self.client.post("/api/v1/analysis/simplify", &request).await
    }

    /// Calculate centroids
    pub async fn centroid(&self, request: CentroidRequest) -> Result<FeatureCollection> {
        self.client.post("/api/v1/analysis/centroid", &request).await
    }

    /// Calculate convex hull
    pub async fn convex_hull(&self, request: ConvexHullRequest) -> Result<Feature> {
        self.client.post("/api/v1/analysis/convex-hull", &request).await
    }
}

/// Buffer operation request
#[derive(Debug, Clone, Serialize)]
pub struct BufferRequest {
    pub layer: String,
    pub distance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<BufferOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

/// Buffer options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferOptions {
    /// Number of segments per quadrant (default: 8)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segments: Option<u32>,
    /// Cap style: round, flat, square
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cap_style: Option<String>,
    /// Join style: round, mitre, bevel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_style: Option<String>,
}

/// Clip operation request
#[derive(Debug, Clone, Serialize)]
pub struct ClipRequest {
    pub input_layer: String,
    pub clip_layer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ClipOptions>,
}

/// Clip options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipOptions {
    /// Keep features that are completely outside the clip boundary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_outside: Option<bool>,
}

/// Union operation request
#[derive(Debug, Clone, Serialize)]
pub struct UnionRequest {
    pub layer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

/// Intersect operation request
#[derive(Debug, Clone, Serialize)]
pub struct IntersectRequest {
    pub layer1: String,
    pub layer2: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter2: Option<String>,
}

/// Difference operation request
#[derive(Debug, Clone, Serialize)]
pub struct DifferenceRequest {
    pub input_layer: String,
    pub subtract_layer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter2: Option<String>,
}

/// Simplify operation request
#[derive(Debug, Clone, Serialize)]
pub struct SimplifyRequest {
    pub layer: String,
    pub tolerance: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_topology: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

/// Centroid operation request
#[derive(Debug, Clone, Serialize)]
pub struct CentroidRequest {
    pub layer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

/// Convex hull operation request
#[derive(Debug, Clone, Serialize)]
pub struct ConvexHullRequest {
    pub layer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

impl BufferRequest {
    /// Create a new buffer request
    pub fn new(layer: impl Into<String>, distance: f64) -> Self {
        Self {
            layer: layer.into(),
            distance,
            options: None,
            filter: None,
        }
    }

    /// Set buffer options
    pub fn with_options(mut self, options: BufferOptions) -> Self {
        self.options = Some(options);
        self
    }

    /// Set a filter
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }
}

impl BufferOptions {
    /// Create new buffer options
    pub fn new() -> Self {
        Self {
            segments: None,
            cap_style: None,
            join_style: None,
        }
    }

    /// Set the number of segments
    pub fn segments(mut self, segments: u32) -> Self {
        self.segments = Some(segments);
        self
    }

    /// Set the cap style
    pub fn cap_style(mut self, style: impl Into<String>) -> Self {
        self.cap_style = Some(style.into());
        self
    }

    /// Set the join style
    pub fn join_style(mut self, style: impl Into<String>) -> Self {
        self.join_style = Some(style.into());
        self
    }
}

impl Default for BufferOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipRequest {
    /// Create a new clip request
    pub fn new(input_layer: impl Into<String>, clip_layer: impl Into<String>) -> Self {
        Self {
            input_layer: input_layer.into(),
            clip_layer: clip_layer.into(),
            options: None,
        }
    }

    /// Set clip options
    pub fn with_options(mut self, options: ClipOptions) -> Self {
        self.options = Some(options);
        self
    }
}

impl ClipOptions {
    /// Create new clip options
    pub fn new() -> Self {
        Self {
            keep_outside: None,
        }
    }

    /// Set whether to keep features outside the clip boundary
    pub fn keep_outside(mut self, keep: bool) -> Self {
        self.keep_outside = Some(keep);
        self
    }
}

impl Default for ClipOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl SimplifyRequest {
    /// Create a new simplify request
    pub fn new(layer: impl Into<String>, tolerance: f64) -> Self {
        Self {
            layer: layer.into(),
            tolerance,
            preserve_topology: None,
            filter: None,
        }
    }

    /// Enable topology preservation
    pub fn preserve_topology(mut self, preserve: bool) -> Self {
        self.preserve_topology = Some(preserve);
        self
    }

    /// Set a filter
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_request() {
        let request = BufferRequest::new("test_layer", 100.0);
        assert_eq!(request.layer, "test_layer");
        assert_eq!(request.distance, 100.0);
    }

    #[test]
    fn test_buffer_options() {
        let options = BufferOptions::new()
            .segments(16)
            .cap_style("round")
            .join_style("mitre");

        assert_eq!(options.segments, Some(16));
        assert_eq!(options.cap_style, Some("round".to_string()));
        assert_eq!(options.join_style, Some("mitre".to_string()));
    }
}
