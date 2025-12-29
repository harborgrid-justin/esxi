//! Geo-spatial search functionality.

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use elasticsearch::SearchParts;
use geo_types::{Coord, Point, Polygon};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};

/// Geo-spatial search builder.
pub struct GeoSearch {
    client: SearchClient,
    index: String,
    query: GeoQuery,
    size: usize,
    from: usize,
    sort: Vec<SortField>,
}

impl GeoSearch {
    /// Create a new geo-spatial search.
    pub fn new(client: SearchClient, index: impl Into<String>) -> Self {
        Self {
            client,
            index: index.into(),
            query: GeoQuery::default(),
            size: 10,
            from: 0,
            sort: Vec::new(),
        }
    }

    /// Add a geo-distance query (search within radius of a point).
    pub fn within_distance(
        mut self,
        field: impl Into<String>,
        point: Point,
        distance: impl Into<String>,
    ) -> Self {
        self.query.filters.push(GeoFilter::Distance {
            field: field.into(),
            lat: point.y(),
            lon: point.x(),
            distance: distance.into(),
        });
        self
    }

    /// Add a geo-bounding box query.
    pub fn within_bbox(
        mut self,
        field: impl Into<String>,
        top_left: Coord,
        bottom_right: Coord,
    ) -> Self {
        self.query.filters.push(GeoFilter::BoundingBox {
            field: field.into(),
            top_left,
            bottom_right,
        });
        self
    }

    /// Add a geo-polygon query.
    pub fn within_polygon(mut self, field: impl Into<String>, polygon: Polygon) -> Self {
        self.query.filters.push(GeoFilter::Polygon {
            field: field.into(),
            polygon,
        });
        self
    }

    /// Add a geo-shape query.
    pub fn within_shape(
        mut self,
        field: impl Into<String>,
        shape: GeoShape,
        relation: ShapeRelation,
    ) -> Self {
        self.query.filters.push(GeoFilter::Shape {
            field: field.into(),
            shape,
            relation,
        });
        self
    }

    /// Set the number of results to return.
    pub fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    /// Set the offset for pagination.
    pub fn from(mut self, from: usize) -> Self {
        self.from = from;
        self
    }

    /// Sort by distance from a point.
    pub fn sort_by_distance(mut self, field: impl Into<String>, point: Point) -> Self {
        self.sort.push(SortField::GeoDistance {
            field: field.into(),
            lat: point.y(),
            lon: point.x(),
            order: SortOrder::Asc,
        });
        self
    }

    /// Sort by a field.
    pub fn sort_by(mut self, field: impl Into<String>, order: SortOrder) -> Self {
        self.sort.push(SortField::Field {
            field: field.into(),
            order,
        });
        self
    }

    /// Execute the search.
    pub async fn execute<T>(&self) -> SearchResult<GeoSearchResults<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        info!(
            "Executing geo-spatial search on index '{}' with {} filters",
            self.index,
            self.query.filters.len()
        );

        let query_body = self.build_query();
        debug!("Query body: {}", serde_json::to_string_pretty(&query_body).unwrap());

        let response = self
            .client
            .client()
            .search(SearchParts::Index(&[&self.index]))
            .body(query_body)
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::ElasticsearchError(format!(
                "Search failed: {}",
                error_text
            )));
        }

        let body: Value = response.json().await?;
        self.parse_results(body)
    }

    /// Build the Elasticsearch query.
    fn build_query(&self) -> Value {
        let mut bool_query = json!({
            "bool": {
                "must": [],
                "filter": []
            }
        });

        // Add geo filters
        for filter in &self.query.filters {
            match filter {
                GeoFilter::Distance { field, lat, lon, distance } => {
                    bool_query["bool"]["filter"].as_array_mut().unwrap().push(json!({
                        "geo_distance": {
                            "distance": distance,
                            field: {
                                "lat": lat,
                                "lon": lon
                            }
                        }
                    }));
                }
                GeoFilter::BoundingBox { field, top_left, bottom_right } => {
                    bool_query["bool"]["filter"].as_array_mut().unwrap().push(json!({
                        "geo_bounding_box": {
                            field: {
                                "top_left": {
                                    "lat": top_left.y,
                                    "lon": top_left.x
                                },
                                "bottom_right": {
                                    "lat": bottom_right.y,
                                    "lon": bottom_right.x
                                }
                            }
                        }
                    }));
                }
                GeoFilter::Polygon { field, polygon } => {
                    let points: Vec<_> = polygon
                        .exterior()
                        .points()
                        .map(|p| json!([p.x(), p.y()]))
                        .collect();

                    bool_query["bool"]["filter"].as_array_mut().unwrap().push(json!({
                        "geo_polygon": {
                            field: {
                                "points": points
                            }
                        }
                    }));
                }
                GeoFilter::Shape { field, shape, relation } => {
                    bool_query["bool"]["filter"].as_array_mut().unwrap().push(json!({
                        "geo_shape": {
                            field: {
                                "shape": shape.to_geojson(),
                                "relation": relation.as_str()
                            }
                        }
                    }));
                }
            }
        }

        let mut query = json!({
            "query": bool_query,
            "size": self.size,
            "from": self.from
        });

        // Add sorting
        if !self.sort.is_empty() {
            let sort: Vec<Value> = self.sort.iter().map(|s| s.to_json()).collect();
            query["sort"] = json!(sort);
        }

        query
    }

    /// Parse search results.
    fn parse_results<T>(&self, body: Value) -> SearchResult<GeoSearchResults<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let hits = body["hits"]["hits"]
            .as_array()
            .ok_or_else(|| SearchError::QueryParseError("Missing hits array".to_string()))?;

        let total = body["hits"]["total"]["value"]
            .as_u64()
            .unwrap_or(hits.len() as u64);

        let results: Vec<GeoSearchHit<T>> = hits
            .iter()
            .map(|hit| {
                let source: T = serde_json::from_value(hit["_source"].clone())?;
                let score = hit["_score"].as_f64();
                let sort_values = hit["sort"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect())
                    .unwrap_or_default();

                Ok(GeoSearchHit {
                    id: hit["_id"].as_str().unwrap_or("").to_string(),
                    source,
                    score,
                    sort: sort_values,
                })
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()?;

        Ok(GeoSearchResults {
            total,
            hits: results,
        })
    }
}

/// Geo-spatial query.
#[derive(Debug, Clone, Default)]
struct GeoQuery {
    filters: Vec<GeoFilter>,
}

/// Geo-spatial filter types.
#[derive(Debug, Clone)]
enum GeoFilter {
    Distance {
        field: String,
        lat: f64,
        lon: f64,
        distance: String,
    },
    BoundingBox {
        field: String,
        top_left: Coord,
        bottom_right: Coord,
    },
    Polygon {
        field: String,
        polygon: Polygon,
    },
    Shape {
        field: String,
        shape: GeoShape,
        relation: ShapeRelation,
    },
}

/// Geo-shape definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GeoShape {
    Point { coordinates: [f64; 2] },
    LineString { coordinates: Vec<[f64; 2]> },
    Polygon { coordinates: Vec<Vec<[f64; 2]>> },
    MultiPoint { coordinates: Vec<[f64; 2]> },
    MultiLineString { coordinates: Vec<Vec<[f64; 2]>> },
    MultiPolygon { coordinates: Vec<Vec<Vec<[f64; 2]>>> },
}

impl GeoShape {
    fn to_geojson(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

/// Spatial relation for geo-shape queries.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ShapeRelation {
    Intersects,
    Disjoint,
    Within,
    Contains,
}

impl ShapeRelation {
    fn as_str(&self) -> &str {
        match self {
            Self::Intersects => "INTERSECTS",
            Self::Disjoint => "DISJOINT",
            Self::Within => "WITHIN",
            Self::Contains => "CONTAINS",
        }
    }
}

/// Sort field.
#[derive(Debug, Clone)]
enum SortField {
    Field { field: String, order: SortOrder },
    GeoDistance { field: String, lat: f64, lon: f64, order: SortOrder },
}

impl SortField {
    fn to_json(&self) -> Value {
        match self {
            Self::Field { field, order } => {
                json!({ field: { "order": order.as_str() } })
            }
            Self::GeoDistance { field, lat, lon, order } => {
                json!({
                    "_geo_distance": {
                        field: {
                            "lat": lat,
                            "lon": lon
                        },
                        "order": order.as_str(),
                        "unit": "km"
                    }
                })
            }
        }
    }
}

/// Sort order.
#[derive(Debug, Clone, Copy)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    fn as_str(&self) -> &str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// Geo-spatial search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoSearchResults<T> {
    pub total: u64,
    pub hits: Vec<GeoSearchHit<T>>,
}

/// Individual search hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoSearchHit<T> {
    pub id: String,
    pub source: T,
    pub score: Option<f64>,
    pub sort: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geo_shape_serialization() {
        let shape = GeoShape::Point {
            coordinates: [1.0, 2.0],
        };
        let json = serde_json::to_string(&shape).unwrap();
        assert!(json.contains("Point"));
        assert!(json.contains("coordinates"));
    }

    #[test]
    fn test_shape_relation() {
        assert_eq!(ShapeRelation::Intersects.as_str(), "INTERSECTS");
        assert_eq!(ShapeRelation::Within.as_str(), "WITHIN");
    }
}
