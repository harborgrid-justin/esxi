//! Search result ranking and boosting.

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use elasticsearch::SearchParts;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info};

/// Ranking search builder with custom scoring.
pub struct RankingSearch {
    client: SearchClient,
    index: String,
    query: Option<Value>,
    function_score: Vec<ScoreFunction>,
    boost_mode: BoostMode,
    score_mode: ScoreMode,
    min_score: Option<f64>,
    rescore: Option<RescoreQuery>,
    size: usize,
    from: usize,
}

impl RankingSearch {
    /// Create a new ranking search.
    pub fn new(client: SearchClient, index: impl Into<String>) -> Self {
        Self {
            client,
            index: index.into(),
            query: None,
            function_score: Vec::new(),
            boost_mode: BoostMode::Multiply,
            score_mode: ScoreMode::Sum,
            min_score: None,
            rescore: None,
            size: 10,
            from: 0,
        }
    }

    /// Set the base query.
    pub fn query(mut self, query: Value) -> Self {
        self.query = Some(query);
        self
    }

    /// Add a field value factor for boosting by numeric field.
    pub fn boost_by_field(
        mut self,
        field: impl Into<String>,
        factor: f64,
        modifier: FieldModifier,
    ) -> Self {
        self.function_score.push(ScoreFunction::FieldValueFactor {
            field: field.into(),
            factor,
            modifier,
            missing: None,
        });
        self
    }

    /// Add a decay function for geo-distance based boosting.
    pub fn boost_by_geo_distance(
        mut self,
        field: impl Into<String>,
        origin: GeoLocation,
        scale: String,
        offset: Option<String>,
        decay: f64,
    ) -> Self {
        self.function_score.push(ScoreFunction::GeoDecay {
            field: field.into(),
            origin,
            scale,
            offset,
            decay,
            decay_type: DecayType::Gauss,
        });
        self
    }

    /// Add a decay function for date-based boosting (favor recent).
    pub fn boost_by_recency(
        mut self,
        field: impl Into<String>,
        origin: String,
        scale: String,
        decay: f64,
    ) -> Self {
        self.function_score.push(ScoreFunction::DateDecay {
            field: field.into(),
            origin,
            scale,
            offset: None,
            decay,
            decay_type: DecayType::Exp,
        });
        self
    }

    /// Add a linear decay function for numeric fields.
    pub fn boost_by_numeric_decay(
        mut self,
        field: impl Into<String>,
        origin: f64,
        scale: f64,
        decay: f64,
    ) -> Self {
        self.function_score.push(ScoreFunction::NumericDecay {
            field: field.into(),
            origin,
            scale,
            offset: None,
            decay,
            decay_type: DecayType::Linear,
        });
        self
    }

    /// Add a random score for randomization.
    pub fn add_random_score(mut self, seed: Option<u64>) -> Self {
        self.function_score.push(ScoreFunction::RandomScore { seed });
        self
    }

    /// Add a script score for custom scoring logic.
    pub fn add_script_score(mut self, script: impl Into<String>) -> Self {
        self.function_score.push(ScoreFunction::ScriptScore {
            script: script.into(),
            params: HashMap::new(),
        });
        self
    }

    /// Add a weight boost for matching a filter.
    pub fn boost_matching(mut self, filter: Value, weight: f64) -> Self {
        self.function_score.push(ScoreFunction::Weight { filter, weight });
        self
    }

    /// Set how scores are combined.
    pub fn boost_mode(mut self, mode: BoostMode) -> Self {
        self.boost_mode = mode;
        self
    }

    /// Set how function scores are combined.
    pub fn score_mode(mut self, mode: ScoreMode) -> Self {
        self.score_mode = mode;
        self
    }

    /// Set minimum score threshold.
    pub fn min_score(mut self, score: f64) -> Self {
        self.min_score = Some(score);
        self
    }

    /// Add a rescore query for two-phase ranking.
    pub fn rescore(mut self, query: Value, window_size: usize, query_weight: f64, rescore_query_weight: f64) -> Self {
        self.rescore = Some(RescoreQuery {
            query,
            window_size,
            query_weight,
            rescore_query_weight,
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

    /// Execute the ranking search.
    pub async fn execute<T>(&self) -> SearchResult<RankingResults<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        info!(
            "Executing ranking search on index '{}' with {} score functions",
            self.index,
            self.function_score.len()
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
                "Ranking search failed: {}",
                error_text
            )));
        }

        let body: Value = response.json().await?;
        self.parse_results(body)
    }

    /// Build the Elasticsearch query.
    fn build_query(&self) -> Value {
        let base_query = self.query.clone().unwrap_or_else(|| json!({ "match_all": {} }));

        let mut search_body = if self.function_score.is_empty() {
            json!({
                "query": base_query,
                "size": self.size,
                "from": self.from
            })
        } else {
            let functions: Vec<Value> = self
                .function_score
                .iter()
                .map(|f| f.to_json())
                .collect();

            json!({
                "query": {
                    "function_score": {
                        "query": base_query,
                        "functions": functions,
                        "boost_mode": self.boost_mode.as_str(),
                        "score_mode": self.score_mode.as_str()
                    }
                },
                "size": self.size,
                "from": self.from
            })
        };

        // Add minimum score if specified
        if let Some(min_score) = self.min_score {
            search_body["min_score"] = json!(min_score);
        }

        // Add rescore if specified
        if let Some(rescore) = &self.rescore {
            search_body["rescore"] = rescore.to_json();
        }

        search_body
    }

    /// Parse search results.
    fn parse_results<T>(&self, body: Value) -> SearchResult<RankingResults<T>>
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

        let results: Vec<RankingHit<T>> = hits
            .iter()
            .map(|hit| {
                let source: T = serde_json::from_value(hit["_source"].clone())?;
                let score = hit["_score"].as_f64().unwrap_or(0.0);

                Ok(RankingHit {
                    id: hit["_id"].as_str().unwrap_or("").to_string(),
                    source,
                    score,
                })
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()?;

        Ok(RankingResults {
            total,
            max_score,
            hits: results,
        })
    }
}

/// Score function types.
#[derive(Debug, Clone)]
enum ScoreFunction {
    FieldValueFactor {
        field: String,
        factor: f64,
        modifier: FieldModifier,
        missing: Option<f64>,
    },
    GeoDecay {
        field: String,
        origin: GeoLocation,
        scale: String,
        offset: Option<String>,
        decay: f64,
        decay_type: DecayType,
    },
    DateDecay {
        field: String,
        origin: String,
        scale: String,
        offset: Option<String>,
        decay: f64,
        decay_type: DecayType,
    },
    NumericDecay {
        field: String,
        origin: f64,
        scale: f64,
        offset: Option<f64>,
        decay: f64,
        decay_type: DecayType,
    },
    RandomScore {
        seed: Option<u64>,
    },
    ScriptScore {
        script: String,
        params: HashMap<String, Value>,
    },
    Weight {
        filter: Value,
        weight: f64,
    },
}

impl ScoreFunction {
    fn to_json(&self) -> Value {
        match self {
            Self::FieldValueFactor { field, factor, modifier, missing } => {
                let mut fvf = json!({
                    "field": field,
                    "factor": factor,
                    "modifier": modifier.as_str()
                });
                if let Some(m) = missing {
                    fvf["missing"] = json!(m);
                }
                json!({ "field_value_factor": fvf })
            }
            Self::GeoDecay { field, origin, scale, offset, decay, decay_type } => {
                let mut decay_obj = json!({
                    field: {
                        "origin": origin.to_json(),
                        "scale": scale,
                        "decay": decay
                    }
                });
                if let Some(off) = offset {
                    decay_obj[field]["offset"] = json!(off);
                }
                json!({ decay_type.as_str(): decay_obj })
            }
            Self::DateDecay { field, origin, scale, offset, decay, decay_type } => {
                let mut decay_obj = json!({
                    field: {
                        "origin": origin,
                        "scale": scale,
                        "decay": decay
                    }
                });
                if let Some(off) = offset {
                    decay_obj[field]["offset"] = json!(off);
                }
                json!({ decay_type.as_str(): decay_obj })
            }
            Self::NumericDecay { field, origin, scale, offset, decay, decay_type } => {
                let mut decay_obj = json!({
                    field: {
                        "origin": origin,
                        "scale": scale,
                        "decay": decay
                    }
                });
                if let Some(off) = offset {
                    decay_obj[field]["offset"] = json!(off);
                }
                json!({ decay_type.as_str(): decay_obj })
            }
            Self::RandomScore { seed } => {
                let mut random = json!({});
                if let Some(s) = seed {
                    random["seed"] = json!(s);
                }
                json!({ "random_score": random })
            }
            Self::ScriptScore { script, params } => {
                json!({
                    "script_score": {
                        "script": {
                            "source": script,
                            "params": params
                        }
                    }
                })
            }
            Self::Weight { filter, weight } => {
                json!({
                    "filter": filter,
                    "weight": weight
                })
            }
        }
    }
}

/// Field modifier for field value factor.
#[derive(Debug, Clone, Copy)]
pub enum FieldModifier {
    None,
    Log,
    Log1p,
    Log2p,
    Ln,
    Ln1p,
    Ln2p,
    Square,
    Sqrt,
    Reciprocal,
}

impl FieldModifier {
    fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Log => "log",
            Self::Log1p => "log1p",
            Self::Log2p => "log2p",
            Self::Ln => "ln",
            Self::Ln1p => "ln1p",
            Self::Ln2p => "ln2p",
            Self::Square => "square",
            Self::Sqrt => "sqrt",
            Self::Reciprocal => "reciprocal",
        }
    }
}

/// Decay function type.
#[derive(Debug, Clone, Copy)]
pub enum DecayType {
    Gauss,
    Exp,
    Linear,
}

impl DecayType {
    fn as_str(&self) -> &str {
        match self {
            Self::Gauss => "gauss",
            Self::Exp => "exp",
            Self::Linear => "linear",
        }
    }
}

/// Geographic location for decay functions.
#[derive(Debug, Clone)]
pub struct GeoLocation {
    pub lat: f64,
    pub lon: f64,
}

impl GeoLocation {
    fn to_json(&self) -> Value {
        json!({
            "lat": self.lat,
            "lon": self.lon
        })
    }
}

/// Boost mode for combining query score with function scores.
#[derive(Debug, Clone, Copy)]
pub enum BoostMode {
    Multiply,
    Replace,
    Sum,
    Avg,
    Max,
    Min,
}

impl BoostMode {
    fn as_str(&self) -> &str {
        match self {
            Self::Multiply => "multiply",
            Self::Replace => "replace",
            Self::Sum => "sum",
            Self::Avg => "avg",
            Self::Max => "max",
            Self::Min => "min",
        }
    }
}

/// Score mode for combining multiple function scores.
#[derive(Debug, Clone, Copy)]
pub enum ScoreMode {
    Multiply,
    Sum,
    Avg,
    First,
    Max,
    Min,
}

impl ScoreMode {
    fn as_str(&self) -> &str {
        match self {
            Self::Multiply => "multiply",
            Self::Sum => "sum",
            Self::Avg => "avg",
            Self::First => "first",
            Self::Max => "max",
            Self::Min => "min",
        }
    }
}

/// Rescore query for two-phase ranking.
#[derive(Debug, Clone)]
struct RescoreQuery {
    query: Value,
    window_size: usize,
    query_weight: f64,
    rescore_query_weight: f64,
}

impl RescoreQuery {
    fn to_json(&self) -> Value {
        json!({
            "window_size": self.window_size,
            "query": {
                "rescore_query": self.query,
                "query_weight": self.query_weight,
                "rescore_query_weight": self.rescore_query_weight
            }
        })
    }
}

/// Ranking search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingResults<T> {
    pub total: u64,
    pub max_score: Option<f64>,
    pub hits: Vec<RankingHit<T>>,
}

/// Individual ranking hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingHit<T> {
    pub id: String,
    pub source: T,
    pub score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_modifier() {
        assert_eq!(FieldModifier::Log.as_str(), "log");
        assert_eq!(FieldModifier::Sqrt.as_str(), "sqrt");
    }

    #[test]
    fn test_decay_type() {
        assert_eq!(DecayType::Gauss.as_str(), "gauss");
        assert_eq!(DecayType::Linear.as_str(), "linear");
    }

    #[test]
    fn test_boost_mode() {
        assert_eq!(BoostMode::Multiply.as_str(), "multiply");
        assert_eq!(BoostMode::Sum.as_str(), "sum");
    }

    #[test]
    fn test_score_mode() {
        assert_eq!(ScoreMode::Sum.as_str(), "sum");
        assert_eq!(ScoreMode::Max.as_str(), "max");
    }

    #[test]
    fn test_geo_location() {
        let loc = GeoLocation { lat: 40.7128, lon: -74.0060 };
        let json = loc.to_json();
        assert_eq!(json["lat"].as_f64(), Some(40.7128));
        assert_eq!(json["lon"].as_f64(), Some(-74.0060));
    }
}
