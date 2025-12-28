use serde::{Deserialize, Serialize};
use crate::client::Client;
use crate::features::FeatureCollection;
use crate::error::Result;

/// Spatial predicate types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpatialPredicate {
    Intersects,
    Contains,
    Within,
    Touches,
    Crosses,
    Overlaps,
    Disjoint,
    Equals,
}

/// Query builder for spatial and attribute queries
pub struct QueryBuilder<'a> {
    client: &'a Client,
    layer_name: String,
    filter: Option<String>,
    spatial_filter: Option<SpatialFilter>,
    bbox: Option<BBox>,
    limit: Option<u32>,
    offset: Option<u32>,
    order_by: Option<String>,
    fields: Option<Vec<String>>,
}

/// Spatial filter
#[derive(Debug, Clone, Serialize)]
pub struct SpatialFilter {
    pub predicate: SpatialPredicate,
    pub geometry: String, // WKT format
}

/// Bounding box filter
#[derive(Debug, Clone, Serialize)]
pub struct BBox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

/// Query request sent to the API
#[derive(Debug, Clone, Serialize)]
struct QueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spatial_filter: Option<SpatialFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bbox: Option<BBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    order_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fields: Option<Vec<String>>,
}

impl<'a> QueryBuilder<'a> {
    /// Create a new query builder
    pub(crate) fn new(client: &'a Client, layer_name: String) -> Self {
        Self {
            client,
            layer_name,
            filter: None,
            spatial_filter: None,
            bbox: None,
            limit: None,
            offset: None,
            order_by: None,
            fields: None,
        }
    }

    /// Add an attribute filter (SQL WHERE clause)
    pub fn filter(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }

    /// Add a spatial filter
    pub fn spatial(mut self, predicate: SpatialPredicate, geometry_wkt: impl Into<String>) -> Self {
        self.spatial_filter = Some(SpatialFilter {
            predicate,
            geometry: geometry_wkt.into(),
        });
        self
    }

    /// Filter by bounding box
    pub fn bbox(mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        self.bbox = Some(BBox {
            min_x,
            min_y,
            max_x,
            max_y,
        });
        self
    }

    /// Set the maximum number of results
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the offset for pagination
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set the order by clause
    pub fn order_by(mut self, order: impl Into<String>) -> Self {
        self.order_by = Some(order.into());
        self
    }

    /// Select specific fields
    pub fn fields(mut self, fields: Vec<String>) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Execute the query
    pub async fn execute(self) -> Result<FeatureCollection> {
        let path = format!("/api/v1/layers/{}/query", self.layer_name);

        let request = QueryRequest {
            filter: self.filter,
            spatial_filter: self.spatial_filter,
            bbox: self.bbox,
            limit: self.limit,
            offset: self.offset,
            order_by: self.order_by,
            fields: self.fields,
        };

        self.client.post(&path, &request).await
    }

    /// Execute the query and return only the count
    pub async fn count(self) -> Result<u64> {
        let path = format!("/api/v1/layers/{}/query/count", self.layer_name);

        let request = QueryRequest {
            filter: self.filter,
            spatial_filter: self.spatial_filter,
            bbox: self.bbox,
            limit: None,
            offset: None,
            order_by: None,
            fields: None,
        };

        #[derive(Deserialize)]
        struct CountResponse {
            count: u64,
        }

        let response: CountResponse = self.client.post(&path, &request).await?;
        Ok(response.count)
    }
}

impl BBox {
    /// Create a new bounding box
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox_creation() {
        let bbox = BBox::new(0.0, 0.0, 10.0, 10.0);
        assert_eq!(bbox.min_x, 0.0);
        assert_eq!(bbox.min_y, 0.0);
        assert_eq!(bbox.max_x, 10.0);
        assert_eq!(bbox.max_y, 10.0);
    }
}
