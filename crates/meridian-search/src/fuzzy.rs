//! Fuzzy matching and typo tolerance.

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use elasticsearch::SearchParts;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, info};

/// Fuzzy search builder.
pub struct FuzzySearch {
    client: SearchClient,
    index: String,
    queries: Vec<FuzzyQuery>,
    filters: Vec<Value>,
    size: usize,
    from: usize,
}

impl FuzzySearch {
    /// Create a new fuzzy search.
    pub fn new(client: SearchClient, index: impl Into<String>) -> Self {
        Self {
            client,
            index: index.into(),
            queries: Vec::new(),
            filters: Vec::new(),
            size: 10,
            from: 0,
        }
    }

    /// Add a fuzzy query with auto fuzziness.
    pub fn fuzzy(
        mut self,
        field: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.queries.push(FuzzyQuery {
            field: field.into(),
            value: value.into(),
            fuzziness: Fuzziness::Auto,
            prefix_length: 0,
            max_expansions: 50,
            transpositions: true,
            boost: 1.0,
        });
        self
    }

    /// Add a fuzzy query with specific edit distance.
    pub fn fuzzy_with_distance(
        mut self,
        field: impl Into<String>,
        value: impl Into<String>,
        distance: u32,
    ) -> Self {
        self.queries.push(FuzzyQuery {
            field: field.into(),
            value: value.into(),
            fuzziness: Fuzziness::Distance(distance),
            prefix_length: 0,
            max_expansions: 50,
            transpositions: true,
            boost: 1.0,
        });
        self
    }

    /// Add a fuzzy query with custom parameters.
    pub fn fuzzy_custom(
        mut self,
        field: impl Into<String>,
        value: impl Into<String>,
        config: FuzzyConfig,
    ) -> Self {
        self.queries.push(FuzzyQuery {
            field: field.into(),
            value: value.into(),
            fuzziness: config.fuzziness,
            prefix_length: config.prefix_length,
            max_expansions: config.max_expansions,
            transpositions: config.transpositions,
            boost: config.boost,
        });
        self
    }

    /// Add a wildcard query for pattern matching.
    pub fn wildcard(
        mut self,
        field: impl Into<String>,
        pattern: impl Into<String>,
    ) -> Self {
        self.filters.push(json!({
            "wildcard": {
                field.into(): {
                    "value": pattern.into(),
                    "boost": 1.0
                }
            }
        }));
        self
    }

    /// Add a regexp query for advanced pattern matching.
    pub fn regexp(
        mut self,
        field: impl Into<String>,
        pattern: impl Into<String>,
    ) -> Self {
        self.filters.push(json!({
            "regexp": {
                field.into(): {
                    "value": pattern.into(),
                    "flags": "ALL",
                    "max_determinized_states": 10000
                }
            }
        }));
        self
    }

    /// Add a prefix query.
    pub fn prefix(
        mut self,
        field: impl Into<String>,
        prefix: impl Into<String>,
    ) -> Self {
        self.filters.push(json!({
            "prefix": {
                field.into(): {
                    "value": prefix.into()
                }
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

    /// Execute the fuzzy search.
    pub async fn execute<T>(&self) -> SearchResult<FuzzySearchResults<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        info!(
            "Executing fuzzy search on index '{}' with {} queries",
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
                "Fuzzy search failed: {}",
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
                "should": [],
                "filter": self.filters
            }
        });

        // Add fuzzy queries to "should" clause
        for query in &self.queries {
            bool_query["bool"]["should"]
                .as_array_mut()
                .unwrap()
                .push(query.to_json());
        }

        json!({
            "query": bool_query,
            "size": self.size,
            "from": self.from
        })
    }

    /// Parse search results.
    fn parse_results<T>(&self, body: Value) -> SearchResult<FuzzySearchResults<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let hits = body["hits"]["hits"]
            .as_array()
            .ok_or_else(|| SearchError::QueryParseError("Missing hits array".to_string()))?;

        let total = body["hits"]["total"]["value"]
            .as_u64()
            .unwrap_or(hits.len() as u64);

        let results: Vec<FuzzySearchHit<T>> = hits
            .iter()
            .map(|hit| {
                let source: T = serde_json::from_value(hit["_source"].clone())?;
                let score = hit["_score"].as_f64();

                Ok(FuzzySearchHit {
                    id: hit["_id"].as_str().unwrap_or("").to_string(),
                    source,
                    score,
                })
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()?;

        Ok(FuzzySearchResults {
            total,
            hits: results,
        })
    }
}

/// Fuzzy query configuration.
#[derive(Debug, Clone)]
struct FuzzyQuery {
    field: String,
    value: String,
    fuzziness: Fuzziness,
    prefix_length: usize,
    max_expansions: usize,
    transpositions: bool,
    boost: f64,
}

impl FuzzyQuery {
    fn to_json(&self) -> Value {
        let mut fuzzy = json!({
            "value": self.value,
            "fuzziness": self.fuzziness.to_string(),
            "prefix_length": self.prefix_length,
            "max_expansions": self.max_expansions,
            "transpositions": self.transpositions
        });

        if self.boost != 1.0 {
            fuzzy["boost"] = json!(self.boost);
        }

        json!({
            "fuzzy": {
                self.field.clone(): fuzzy
            }
        })
    }
}

/// Fuzziness parameter for edit distance.
#[derive(Debug, Clone, Copy)]
pub enum Fuzziness {
    /// Automatic fuzziness based on term length
    Auto,
    /// Specific edit distance (0-2)
    Distance(u32),
}

impl Fuzziness {
    fn to_string(&self) -> String {
        match self {
            Self::Auto => "AUTO".to_string(),
            Self::Distance(d) => d.to_string(),
        }
    }
}

/// Custom fuzzy configuration.
#[derive(Debug, Clone)]
pub struct FuzzyConfig {
    pub fuzziness: Fuzziness,
    pub prefix_length: usize,
    pub max_expansions: usize,
    pub transpositions: bool,
    pub boost: f64,
}

impl Default for FuzzyConfig {
    fn default() -> Self {
        Self {
            fuzziness: Fuzziness::Auto,
            prefix_length: 0,
            max_expansions: 50,
            transpositions: true,
            boost: 1.0,
        }
    }
}

/// Fuzzy search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzySearchResults<T> {
    pub total: u64,
    pub hits: Vec<FuzzySearchHit<T>>,
}

/// Individual fuzzy search hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzySearchHit<T> {
    pub id: String,
    pub source: T,
    pub score: Option<f64>,
}

/// Typo tolerance helper for common patterns.
pub struct TypoTolerance {
    client: SearchClient,
}

impl TypoTolerance {
    /// Create a new typo tolerance helper.
    pub fn new(client: SearchClient) -> Self {
        Self { client }
    }

    /// Search with automatic typo tolerance.
    pub async fn search<T>(
        &self,
        index: &str,
        field: &str,
        query: &str,
    ) -> SearchResult<FuzzySearchResults<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let fuzziness = Self::calculate_fuzziness(query);

        FuzzySearch::new(self.client.clone(), index)
            .fuzzy_with_distance(field, query, fuzziness)
            .size(10)
            .execute()
            .await
    }

    /// Calculate appropriate fuzziness based on query length.
    fn calculate_fuzziness(query: &str) -> u32 {
        let len = query.len();
        if len <= 2 {
            0
        } else if len <= 5 {
            1
        } else {
            2
        }
    }

    /// Search with phonetic matching (requires phonetic analyzer).
    pub async fn phonetic_search<T>(
        &self,
        index: &str,
        field: &str,
        query: &str,
    ) -> SearchResult<FuzzySearchResults<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Use a phonetic field variant if available
        let phonetic_field = format!("{}.phonetic", field);

        FuzzySearch::new(self.client.clone(), index)
            .fuzzy(phonetic_field, query)
            .size(10)
            .execute()
            .await
    }
}

/// Levenshtein distance calculator for similarity scoring.
pub struct LevenshteinDistance;

impl LevenshteinDistance {
    /// Calculate Levenshtein distance between two strings.
    pub fn distance(a: &str, b: &str) -> usize {
        let len_a = a.chars().count();
        let len_b = b.chars().count();

        if len_a == 0 {
            return len_b;
        }
        if len_b == 0 {
            return len_a;
        }

        let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];

        for i in 0..=len_a {
            matrix[i][0] = i;
        }
        for j in 0..=len_b {
            matrix[0][j] = j;
        }

        let chars_a: Vec<char> = a.chars().collect();
        let chars_b: Vec<char> = b.chars().collect();

        for (i, ca) in chars_a.iter().enumerate() {
            for (j, cb) in chars_b.iter().enumerate() {
                let cost = if ca == cb { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        matrix[i][j + 1] + 1,     // deletion
                        matrix[i + 1][j] + 1,     // insertion
                    ),
                    matrix[i][j] + cost,          // substitution
                );
            }
        }

        matrix[len_a][len_b]
    }

    /// Calculate similarity score (0.0 to 1.0).
    pub fn similarity(a: &str, b: &str) -> f64 {
        let distance = Self::distance(a, b);
        let max_len = std::cmp::max(a.len(), b.len());

        if max_len == 0 {
            return 1.0;
        }

        1.0 - (distance as f64 / max_len as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzziness_to_string() {
        assert_eq!(Fuzziness::Auto.to_string(), "AUTO");
        assert_eq!(Fuzziness::Distance(2).to_string(), "2");
    }

    #[test]
    fn test_fuzzy_config_default() {
        let config = FuzzyConfig::default();
        assert_eq!(config.prefix_length, 0);
        assert_eq!(config.max_expansions, 50);
        assert!(config.transpositions);
    }

    #[test]
    fn test_typo_tolerance_fuzziness_calculation() {
        assert_eq!(TypoTolerance::calculate_fuzziness("ab"), 0);
        assert_eq!(TypoTolerance::calculate_fuzziness("test"), 1);
        assert_eq!(TypoTolerance::calculate_fuzziness("testing"), 2);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(LevenshteinDistance::distance("kitten", "sitting"), 3);
        assert_eq!(LevenshteinDistance::distance("saturday", "sunday"), 3);
        assert_eq!(LevenshteinDistance::distance("", "abc"), 3);
        assert_eq!(LevenshteinDistance::distance("abc", "abc"), 0);
    }

    #[test]
    fn test_levenshtein_similarity() {
        let sim = LevenshteinDistance::similarity("test", "test");
        assert!((sim - 1.0).abs() < 0.001);

        let sim = LevenshteinDistance::similarity("test", "text");
        assert!(sim > 0.7 && sim < 0.8);
    }
}
