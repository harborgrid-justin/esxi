//! Faceted search and aggregations.

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use elasticsearch::SearchParts;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info};

/// Faceted search builder.
pub struct FacetedSearch {
    client: SearchClient,
    index: String,
    query: Option<Value>,
    aggregations: HashMap<String, Aggregation>,
    size: usize,
}

impl FacetedSearch {
    /// Create a new faceted search.
    pub fn new(client: SearchClient, index: impl Into<String>) -> Self {
        Self {
            client,
            index: index.into(),
            query: None,
            aggregations: HashMap::new(),
            size: 0, // Don't return documents by default, just aggregations
        }
    }

    /// Set the base query for filtering.
    pub fn query(mut self, query: Value) -> Self {
        self.query = Some(query);
        self
    }

    /// Add a terms aggregation (facet by field values).
    pub fn terms_facet(
        mut self,
        name: impl Into<String>,
        field: impl Into<String>,
        size: usize,
    ) -> Self {
        self.aggregations.insert(
            name.into(),
            Aggregation::Terms {
                field: field.into(),
                size,
                min_doc_count: Some(1),
                order: None,
            },
        );
        self
    }

    /// Add a histogram aggregation (numeric ranges).
    pub fn histogram(
        mut self,
        name: impl Into<String>,
        field: impl Into<String>,
        interval: f64,
    ) -> Self {
        self.aggregations.insert(
            name.into(),
            Aggregation::Histogram {
                field: field.into(),
                interval,
                min_doc_count: Some(1),
            },
        );
        self
    }

    /// Add a date histogram aggregation.
    pub fn date_histogram(
        mut self,
        name: impl Into<String>,
        field: impl Into<String>,
        interval: DateInterval,
    ) -> Self {
        self.aggregations.insert(
            name.into(),
            Aggregation::DateHistogram {
                field: field.into(),
                interval,
                min_doc_count: Some(1),
            },
        );
        self
    }

    /// Add a range aggregation.
    pub fn range(
        mut self,
        name: impl Into<String>,
        field: impl Into<String>,
        ranges: Vec<RangeBucket>,
    ) -> Self {
        self.aggregations.insert(
            name.into(),
            Aggregation::Range {
                field: field.into(),
                ranges,
            },
        );
        self
    }

    /// Add a geo-distance aggregation.
    pub fn geo_distance(
        mut self,
        name: impl Into<String>,
        field: impl Into<String>,
        origin: GeoPoint,
        ranges: Vec<DistanceRange>,
    ) -> Self {
        self.aggregations.insert(
            name.into(),
            Aggregation::GeoDistance {
                field: field.into(),
                origin,
                ranges,
                unit: DistanceUnit::Kilometers,
            },
        );
        self
    }

    /// Add a statistics aggregation.
    pub fn stats(mut self, name: impl Into<String>, field: impl Into<String>) -> Self {
        self.aggregations.insert(
            name.into(),
            Aggregation::Stats {
                field: field.into(),
            },
        );
        self
    }

    /// Add an extended statistics aggregation.
    pub fn extended_stats(mut self, name: impl Into<String>, field: impl Into<String>) -> Self {
        self.aggregations.insert(
            name.into(),
            Aggregation::ExtendedStats {
                field: field.into(),
            },
        );
        self
    }

    /// Add a cardinality aggregation (approximate unique count).
    pub fn cardinality(mut self, name: impl Into<String>, field: impl Into<String>) -> Self {
        self.aggregations.insert(
            name.into(),
            Aggregation::Cardinality {
                field: field.into(),
            },
        );
        self
    }

    /// Add a nested aggregation.
    pub fn nested(
        mut self,
        name: impl Into<String>,
        path: impl Into<String>,
        sub_agg: Aggregation,
    ) -> Self {
        self.aggregations.insert(
            name.into(),
            Aggregation::Nested {
                path: path.into(),
                aggs: Box::new(sub_agg),
            },
        );
        self
    }

    /// Set the number of documents to return.
    pub fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    /// Execute the faceted search.
    pub async fn execute(&self) -> SearchResult<FacetResults> {
        info!(
            "Executing faceted search on index '{}' with {} aggregations",
            self.index,
            self.aggregations.len()
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
                "Faceted search failed: {}",
                error_text
            )));
        }

        let body: Value = response.json().await?;
        self.parse_results(body)
    }

    /// Build the Elasticsearch query.
    fn build_query(&self) -> Value {
        let query = self.query.clone().unwrap_or_else(|| json!({ "match_all": {} }));

        let aggs: HashMap<String, Value> = self
            .aggregations
            .iter()
            .map(|(name, agg)| (name.clone(), agg.to_json()))
            .collect();

        json!({
            "query": query,
            "size": self.size,
            "aggs": aggs
        })
    }

    /// Parse facet results.
    fn parse_results(&self, body: Value) -> SearchResult<FacetResults> {
        let aggregations = body["aggregations"]
            .as_object()
            .ok_or_else(|| SearchError::QueryParseError("Missing aggregations".to_string()))?;

        let total = body["hits"]["total"]["value"]
            .as_u64()
            .unwrap_or(0);

        let mut facets = HashMap::new();

        for (name, agg_result) in aggregations {
            let facet = self.parse_aggregation_result(agg_result)?;
            facets.insert(name.clone(), facet);
        }

        Ok(FacetResults { total, facets })
    }

    /// Parse individual aggregation result.
    fn parse_aggregation_result(&self, result: &Value) -> SearchResult<FacetResult> {
        // Check for terms aggregation
        if let Some(buckets) = result["buckets"].as_array() {
            let buckets: Vec<Bucket> = buckets
                .iter()
                .map(|b| Bucket {
                    key: b["key"].clone(),
                    key_as_string: b["key_as_string"].as_str().map(String::from),
                    doc_count: b["doc_count"].as_u64().unwrap_or(0),
                    sub_aggregations: b["aggregations"]
                        .as_object()
                        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
                })
                .collect();

            return Ok(FacetResult::Buckets(buckets));
        }

        // Check for stats aggregation
        if result["count"].is_number() && result["min"].is_number() {
            let stats = StatsResult {
                count: result["count"].as_u64().unwrap_or(0),
                min: result["min"].as_f64(),
                max: result["max"].as_f64(),
                avg: result["avg"].as_f64(),
                sum: result["sum"].as_f64(),
                std_deviation: result["std_deviation"].as_f64(),
                variance: result["variance"].as_f64(),
            };
            return Ok(FacetResult::Stats(stats));
        }

        // Check for cardinality aggregation
        if let Some(value) = result["value"].as_u64() {
            return Ok(FacetResult::Value(value));
        }

        Ok(FacetResult::Raw(result.clone()))
    }
}

/// Aggregation types.
#[derive(Debug, Clone)]
pub enum Aggregation {
    Terms {
        field: String,
        size: usize,
        min_doc_count: Option<u64>,
        order: Option<AggOrder>,
    },
    Histogram {
        field: String,
        interval: f64,
        min_doc_count: Option<u64>,
    },
    DateHistogram {
        field: String,
        interval: DateInterval,
        min_doc_count: Option<u64>,
    },
    Range {
        field: String,
        ranges: Vec<RangeBucket>,
    },
    GeoDistance {
        field: String,
        origin: GeoPoint,
        ranges: Vec<DistanceRange>,
        unit: DistanceUnit,
    },
    Stats {
        field: String,
    },
    ExtendedStats {
        field: String,
    },
    Cardinality {
        field: String,
    },
    Nested {
        path: String,
        aggs: Box<Aggregation>,
    },
}

impl Aggregation {
    fn to_json(&self) -> Value {
        match self {
            Self::Terms { field, size, min_doc_count, order } => {
                let mut terms = json!({
                    "field": field,
                    "size": size
                });
                if let Some(min_doc) = min_doc_count {
                    terms["min_doc_count"] = json!(min_doc);
                }
                if let Some(ord) = order {
                    terms["order"] = ord.to_json();
                }
                json!({ "terms": terms })
            }
            Self::Histogram { field, interval, min_doc_count } => {
                let mut hist = json!({
                    "field": field,
                    "interval": interval
                });
                if let Some(min_doc) = min_doc_count {
                    hist["min_doc_count"] = json!(min_doc);
                }
                json!({ "histogram": hist })
            }
            Self::DateHistogram { field, interval, min_doc_count } => {
                let mut hist = json!({
                    "field": field,
                    "calendar_interval": interval.as_str()
                });
                if let Some(min_doc) = min_doc_count {
                    hist["min_doc_count"] = json!(min_doc);
                }
                json!({ "date_histogram": hist })
            }
            Self::Range { field, ranges } => {
                let ranges: Vec<Value> = ranges.iter().map(|r| r.to_json()).collect();
                json!({
                    "range": {
                        "field": field,
                        "ranges": ranges
                    }
                })
            }
            Self::GeoDistance { field, origin, ranges, unit } => {
                let ranges: Vec<Value> = ranges.iter().map(|r| r.to_json()).collect();
                json!({
                    "geo_distance": {
                        "field": field,
                        "origin": origin.to_json(),
                        "ranges": ranges,
                        "unit": unit.as_str()
                    }
                })
            }
            Self::Stats { field } => {
                json!({ "stats": { "field": field } })
            }
            Self::ExtendedStats { field } => {
                json!({ "extended_stats": { "field": field } })
            }
            Self::Cardinality { field } => {
                json!({ "cardinality": { "field": field } })
            }
            Self::Nested { path, aggs } => {
                json!({
                    "nested": { "path": path },
                    "aggs": { "nested_agg": aggs.to_json() }
                })
            }
        }
    }
}

/// Aggregation ordering.
#[derive(Debug, Clone)]
pub struct AggOrder {
    pub field: String,
    pub direction: OrderDirection,
}

impl AggOrder {
    fn to_json(&self) -> Value {
        json!({ self.field.clone(): self.direction.as_str() })
    }
}

/// Order direction.
#[derive(Debug, Clone, Copy)]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl OrderDirection {
    fn as_str(&self) -> &str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// Date interval for date histogram.
#[derive(Debug, Clone, Copy)]
pub enum DateInterval {
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl DateInterval {
    fn as_str(&self) -> &str {
        match self {
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Quarter => "quarter",
            Self::Year => "year",
        }
    }
}

/// Range bucket for range aggregations.
#[derive(Debug, Clone)]
pub struct RangeBucket {
    pub key: Option<String>,
    pub from: Option<f64>,
    pub to: Option<f64>,
}

impl RangeBucket {
    fn to_json(&self) -> Value {
        let mut range = json!({});
        if let Some(key) = &self.key {
            range["key"] = json!(key);
        }
        if let Some(from) = self.from {
            range["from"] = json!(from);
        }
        if let Some(to) = self.to {
            range["to"] = json!(to);
        }
        range
    }
}

/// Distance range for geo-distance aggregations.
#[derive(Debug, Clone)]
pub struct DistanceRange {
    pub key: Option<String>,
    pub from: Option<f64>,
    pub to: Option<f64>,
}

impl DistanceRange {
    fn to_json(&self) -> Value {
        let mut range = json!({});
        if let Some(key) = &self.key {
            range["key"] = json!(key);
        }
        if let Some(from) = self.from {
            range["from"] = json!(from);
        }
        if let Some(to) = self.to {
            range["to"] = json!(to);
        }
        range
    }
}

/// Geographic point.
#[derive(Debug, Clone)]
pub struct GeoPoint {
    pub lat: f64,
    pub lon: f64,
}

impl GeoPoint {
    fn to_json(&self) -> Value {
        json!({
            "lat": self.lat,
            "lon": self.lon
        })
    }
}

/// Distance unit.
#[derive(Debug, Clone, Copy)]
pub enum DistanceUnit {
    Meters,
    Kilometers,
    Miles,
}

impl DistanceUnit {
    fn as_str(&self) -> &str {
        match self {
            Self::Meters => "m",
            Self::Kilometers => "km",
            Self::Miles => "mi",
        }
    }
}

/// Faceted search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetResults {
    pub total: u64,
    pub facets: HashMap<String, FacetResult>,
}

/// Individual facet result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FacetResult {
    Buckets(Vec<Bucket>),
    Stats(StatsResult),
    Value(u64),
    Raw(Value),
}

/// Aggregation bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub key: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_as_string: Option<String>,
    pub doc_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_aggregations: Option<HashMap<String, Value>>,
}

/// Statistics result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResult {
    pub count: u64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub avg: Option<f64>,
    pub sum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub std_deviation: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variance: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_interval() {
        assert_eq!(DateInterval::Day.as_str(), "day");
        assert_eq!(DateInterval::Month.as_str(), "month");
    }

    #[test]
    fn test_distance_unit() {
        assert_eq!(DistanceUnit::Kilometers.as_str(), "km");
        assert_eq!(DistanceUnit::Miles.as_str(), "mi");
    }

    #[test]
    fn test_range_bucket() {
        let bucket = RangeBucket {
            key: Some("0-100".to_string()),
            from: Some(0.0),
            to: Some(100.0),
        };
        let json = bucket.to_json();
        assert!(json["key"].is_string());
        assert_eq!(json["from"].as_f64(), Some(0.0));
    }
}
