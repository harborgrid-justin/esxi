//! Full-text search with custom analyzers.

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use elasticsearch::SearchParts;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};

/// Full-text search builder.
pub struct FullTextSearch {
    client: SearchClient,
    index: String,
    queries: Vec<TextQuery>,
    filters: Vec<Value>,
    size: usize,
    from: usize,
    highlight: Option<HighlightConfig>,
    min_score: Option<f64>,
}

impl FullTextSearch {
    /// Create a new full-text search.
    pub fn new(client: SearchClient, index: impl Into<String>) -> Self {
        Self {
            client,
            index: index.into(),
            queries: Vec::new(),
            filters: Vec::new(),
            size: 10,
            from: 0,
            highlight: None,
            min_score: None,
        }
    }

    /// Add a match query (analyzed full-text search).
    pub fn match_query(
        mut self,
        field: impl Into<String>,
        query: impl Into<String>,
        operator: MatchOperator,
    ) -> Self {
        self.queries.push(TextQuery::Match {
            field: field.into(),
            query: query.into(),
            operator,
            boost: 1.0,
        });
        self
    }

    /// Add a multi-match query across multiple fields.
    pub fn multi_match(
        mut self,
        fields: Vec<String>,
        query: impl Into<String>,
        match_type: MultiMatchType,
    ) -> Self {
        self.queries.push(TextQuery::MultiMatch {
            fields,
            query: query.into(),
            match_type,
            boost: 1.0,
        });
        self
    }

    /// Add a match phrase query (exact phrase matching).
    pub fn match_phrase(mut self, field: impl Into<String>, query: impl Into<String>) -> Self {
        self.queries.push(TextQuery::MatchPhrase {
            field: field.into(),
            query: query.into(),
            slop: 0,
            boost: 1.0,
        });
        self
    }

    /// Add a query string query (supports operators).
    pub fn query_string(mut self, query: impl Into<String>, fields: Vec<String>) -> Self {
        self.queries.push(TextQuery::QueryString {
            query: query.into(),
            fields,
            default_operator: MatchOperator::Or,
            boost: 1.0,
        });
        self
    }

    /// Add a simple query string (safer version without full syntax).
    pub fn simple_query_string(mut self, query: impl Into<String>, fields: Vec<String>) -> Self {
        self.queries.push(TextQuery::SimpleQueryString {
            query: query.into(),
            fields,
            default_operator: MatchOperator::Or,
            boost: 1.0,
        });
        self
    }

    /// Add a term filter (exact match, not analyzed).
    pub fn filter_term(mut self, field: impl Into<String>, value: impl Into<Value>) -> Self {
        self.filters.push(json!({
            "term": {
                field.into(): value.into()
            }
        }));
        self
    }

    /// Add a terms filter (match any of the values).
    pub fn filter_terms(mut self, field: impl Into<String>, values: Vec<Value>) -> Self {
        self.filters.push(json!({
            "terms": {
                field.into(): values
            }
        }));
        self
    }

    /// Add a range filter.
    pub fn filter_range(mut self, field: impl Into<String>, range: RangeFilter) -> Self {
        let mut range_obj = json!({});

        if let Some(gte) = range.gte {
            range_obj["gte"] = gte;
        }
        if let Some(gt) = range.gt {
            range_obj["gt"] = gt;
        }
        if let Some(lte) = range.lte {
            range_obj["lte"] = lte;
        }
        if let Some(lt) = range.lt {
            range_obj["lt"] = lt;
        }

        self.filters.push(json!({
            "range": {
                field.into(): range_obj
            }
        }));
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

    /// Enable highlighting for matching terms.
    pub fn highlight(mut self, config: HighlightConfig) -> Self {
        self.highlight = Some(config);
        self
    }

    /// Set minimum score threshold.
    pub fn min_score(mut self, score: f64) -> Self {
        self.min_score = Some(score);
        self
    }

    /// Execute the search.
    pub async fn execute<T>(&self) -> SearchResult<SearchResults<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        info!(
            "Executing full-text search on index '{}' with {} queries",
            self.index,
            self.queries.len()
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
                "filter": self.filters
            }
        });

        // Add text queries to "must" clause
        for query in &self.queries {
            bool_query["bool"]["must"]
                .as_array_mut()
                .unwrap()
                .push(query.to_json());
        }

        let mut search_body = json!({
            "query": bool_query,
            "size": self.size,
            "from": self.from
        });

        // Add minimum score if specified
        if let Some(min_score) = self.min_score {
            search_body["min_score"] = json!(min_score);
        }

        // Add highlighting if configured
        if let Some(highlight) = &self.highlight {
            search_body["highlight"] = highlight.to_json();
        }

        search_body
    }

    /// Parse search results.
    fn parse_results<T>(&self, body: Value) -> SearchResult<SearchResults<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let hits = body["hits"]["hits"]
            .as_array()
            .ok_or_else(|| SearchError::QueryParseError("Missing hits array".to_string()))?;

        let total = body["hits"]["total"]["value"]
            .as_u64()
            .unwrap_or(hits.len() as u64);

        let max_score = body["hits"]["max_score"].as_f64();

        let results: Vec<SearchHit<T>> = hits
            .iter()
            .map(|hit| {
                let source: T = serde_json::from_value(hit["_source"].clone())?;
                let score = hit["_score"].as_f64();
                let highlight = hit["highlight"]
                    .as_object()
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| {
                                let fragments = v
                                    .as_array()
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|v| v.as_str().map(String::from))
                                            .collect()
                                    })
                                    .unwrap_or_default();
                                (k.clone(), fragments)
                            })
                            .collect()
                    });

                Ok(SearchHit {
                    id: hit["_id"].as_str().unwrap_or("").to_string(),
                    source,
                    score,
                    highlight,
                })
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()?;

        Ok(SearchResults {
            total,
            max_score,
            hits: results,
        })
    }
}

/// Text query types.
#[derive(Debug, Clone)]
enum TextQuery {
    Match {
        field: String,
        query: String,
        operator: MatchOperator,
        boost: f64,
    },
    MultiMatch {
        fields: Vec<String>,
        query: String,
        match_type: MultiMatchType,
        boost: f64,
    },
    MatchPhrase {
        field: String,
        query: String,
        slop: u32,
        boost: f64,
    },
    QueryString {
        query: String,
        fields: Vec<String>,
        default_operator: MatchOperator,
        boost: f64,
    },
    SimpleQueryString {
        query: String,
        fields: Vec<String>,
        default_operator: MatchOperator,
        boost: f64,
    },
}

impl TextQuery {
    fn to_json(&self) -> Value {
        match self {
            Self::Match { field, query, operator, boost } => {
                json!({
                    "match": {
                        field: {
                            "query": query,
                            "operator": operator.as_str(),
                            "boost": boost
                        }
                    }
                })
            }
            Self::MultiMatch { fields, query, match_type, boost } => {
                json!({
                    "multi_match": {
                        "query": query,
                        "fields": fields,
                        "type": match_type.as_str(),
                        "boost": boost
                    }
                })
            }
            Self::MatchPhrase { field, query, slop, boost } => {
                json!({
                    "match_phrase": {
                        field: {
                            "query": query,
                            "slop": slop,
                            "boost": boost
                        }
                    }
                })
            }
            Self::QueryString { query, fields, default_operator, boost } => {
                json!({
                    "query_string": {
                        "query": query,
                        "fields": fields,
                        "default_operator": default_operator.as_str(),
                        "boost": boost
                    }
                })
            }
            Self::SimpleQueryString { query, fields, default_operator, boost } => {
                json!({
                    "simple_query_string": {
                        "query": query,
                        "fields": fields,
                        "default_operator": default_operator.as_str(),
                        "boost": boost
                    }
                })
            }
        }
    }
}

/// Match operator for boolean queries.
#[derive(Debug, Clone, Copy)]
pub enum MatchOperator {
    And,
    Or,
}

impl MatchOperator {
    fn as_str(&self) -> &str {
        match self {
            Self::And => "and",
            Self::Or => "or",
        }
    }
}

/// Multi-match query types.
#[derive(Debug, Clone, Copy)]
pub enum MultiMatchType {
    BestFields,
    MostFields,
    CrossFields,
    Phrase,
    PhrasePrefix,
    BoolPrefix,
}

impl MultiMatchType {
    fn as_str(&self) -> &str {
        match self {
            Self::BestFields => "best_fields",
            Self::MostFields => "most_fields",
            Self::CrossFields => "cross_fields",
            Self::Phrase => "phrase",
            Self::PhrasePrefix => "phrase_prefix",
            Self::BoolPrefix => "bool_prefix",
        }
    }
}

/// Range filter builder.
#[derive(Debug, Clone, Default)]
pub struct RangeFilter {
    pub gte: Option<Value>,
    pub gt: Option<Value>,
    pub lte: Option<Value>,
    pub lt: Option<Value>,
}

impl RangeFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gte(mut self, value: impl Into<Value>) -> Self {
        self.gte = Some(value.into());
        self
    }

    pub fn gt(mut self, value: impl Into<Value>) -> Self {
        self.gt = Some(value.into());
        self
    }

    pub fn lte(mut self, value: impl Into<Value>) -> Self {
        self.lte = Some(value.into());
        self
    }

    pub fn lt(mut self, value: impl Into<Value>) -> Self {
        self.lt = Some(value.into());
        self
    }
}

/// Highlight configuration.
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    pub fields: Vec<String>,
    pub pre_tags: Vec<String>,
    pub post_tags: Vec<String>,
    pub fragment_size: usize,
    pub number_of_fragments: usize,
}

impl HighlightConfig {
    pub fn new(fields: Vec<String>) -> Self {
        Self {
            fields,
            pre_tags: vec!["<em>".to_string()],
            post_tags: vec!["</em>".to_string()],
            fragment_size: 150,
            number_of_fragments: 3,
        }
    }

    fn to_json(&self) -> Value {
        let fields: Value = self
            .fields
            .iter()
            .map(|f| {
                (
                    f.clone(),
                    json!({
                        "fragment_size": self.fragment_size,
                        "number_of_fragments": self.number_of_fragments
                    }),
                )
            })
            .collect();

        json!({
            "pre_tags": self.pre_tags,
            "post_tags": self.post_tags,
            "fields": fields
        })
    }
}

/// Full-text search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults<T> {
    pub total: u64,
    pub max_score: Option<f64>,
    pub hits: Vec<SearchHit<T>>,
}

/// Individual search hit with highlighting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit<T> {
    pub id: String,
    pub source: T,
    pub score: Option<f64>,
    pub highlight: Option<std::collections::HashMap<String, Vec<String>>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_operator() {
        assert_eq!(MatchOperator::And.as_str(), "and");
        assert_eq!(MatchOperator::Or.as_str(), "or");
    }

    #[test]
    fn test_range_filter_builder() {
        let range = RangeFilter::new().gte(10).lte(100);
        assert!(range.gte.is_some());
        assert!(range.lte.is_some());
        assert!(range.gt.is_none());
    }

    #[test]
    fn test_highlight_config() {
        let config = HighlightConfig::new(vec!["title".to_string(), "content".to_string()]);
        assert_eq!(config.fields.len(), 2);
        assert_eq!(config.pre_tags[0], "<em>");
    }
}
