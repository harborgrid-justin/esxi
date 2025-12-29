//! Search suggestions and autocomplete.

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use elasticsearch::SearchParts;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info};

/// Suggestion builder.
pub struct SuggestionSearch {
    client: SearchClient,
    index: String,
    suggesters: HashMap<String, Suggester>,
}

impl SuggestionSearch {
    /// Create a new suggestion search.
    pub fn new(client: SearchClient, index: impl Into<String>) -> Self {
        Self {
            client,
            index: index.into(),
            suggesters: HashMap::new(),
        }
    }

    /// Add a term suggester (spell correction).
    pub fn term_suggester(
        mut self,
        name: impl Into<String>,
        field: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        self.suggesters.insert(
            name.into(),
            Suggester::Term {
                field: field.into(),
                text: text.into(),
                size: 5,
                max_edits: 2,
                min_word_length: 3,
                prefix_length: 1,
                suggest_mode: SuggestMode::Missing,
            },
        );
        self
    }

    /// Add a phrase suggester (multi-word correction).
    pub fn phrase_suggester(
        mut self,
        name: impl Into<String>,
        field: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        self.suggesters.insert(
            name.into(),
            Suggester::Phrase {
                field: field.into(),
                text: text.into(),
                size: 5,
                max_errors: 2.0,
                confidence: 1.0,
                highlight: Some(HighlightTags {
                    pre_tag: "<em>".to_string(),
                    post_tag: "</em>".to_string(),
                }),
            },
        );
        self
    }

    /// Add a completion suggester (autocomplete).
    pub fn completion_suggester(
        mut self,
        name: impl Into<String>,
        field: impl Into<String>,
        prefix: impl Into<String>,
    ) -> Self {
        self.suggesters.insert(
            name.into(),
            Suggester::Completion {
                field: field.into(),
                prefix: prefix.into(),
                size: 10,
                skip_duplicates: true,
                fuzzy: None,
            },
        );
        self
    }

    /// Add a fuzzy completion suggester.
    pub fn fuzzy_completion(
        mut self,
        name: impl Into<String>,
        field: impl Into<String>,
        prefix: impl Into<String>,
        fuzziness: u32,
    ) -> Self {
        self.suggesters.insert(
            name.into(),
            Suggester::Completion {
                field: field.into(),
                prefix: prefix.into(),
                size: 10,
                skip_duplicates: true,
                fuzzy: Some(FuzzyOptions {
                    fuzziness,
                    transpositions: true,
                    min_length: 3,
                    prefix_length: 1,
                }),
            },
        );
        self
    }

    /// Execute the suggestion search.
    pub async fn execute(&self) -> SearchResult<SuggestionResults> {
        info!(
            "Executing suggestion search on index '{}' with {} suggesters",
            self.index,
            self.suggesters.len()
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
                "Suggestion search failed: {}",
                error_text
            )));
        }

        let body: Value = response.json().await?;
        self.parse_results(body)
    }

    /// Build the Elasticsearch query.
    fn build_query(&self) -> Value {
        let suggest: HashMap<String, Value> = self
            .suggesters
            .iter()
            .map(|(name, suggester)| (name.clone(), suggester.to_json()))
            .collect();

        json!({
            "suggest": suggest
        })
    }

    /// Parse suggestion results.
    fn parse_results(&self, body: Value) -> SearchResult<SuggestionResults> {
        let suggest = body["suggest"]
            .as_object()
            .ok_or_else(|| SearchError::QueryParseError("Missing suggest object".to_string()))?;

        let mut suggestions = HashMap::new();

        for (name, result) in suggest {
            let options = result
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|item| item["options"].as_array())
                .ok_or_else(|| {
                    SearchError::QueryParseError("Invalid suggestion format".to_string())
                })?;

            let parsed_options: Vec<SuggestionOption> = options
                .iter()
                .map(|opt| {
                    Ok(SuggestionOption {
                        text: opt["text"]
                            .as_str()
                            .ok_or_else(|| {
                                SearchError::QueryParseError("Missing text".to_string())
                            })?
                            .to_string(),
                        highlighted: opt["highlighted"]
                            .as_str()
                            .map(String::from),
                        score: opt["score"].as_f64().unwrap_or(0.0),
                        freq: opt["freq"].as_u64(),
                        collate_match: opt["collate_match"].as_bool(),
                    })
                })
                .collect::<SearchResult<Vec<_>>>()?;

            suggestions.insert(name.clone(), parsed_options);
        }

        Ok(SuggestionResults { suggestions })
    }
}

/// Suggester types.
#[derive(Debug, Clone)]
enum Suggester {
    Term {
        field: String,
        text: String,
        size: usize,
        max_edits: u32,
        min_word_length: usize,
        prefix_length: usize,
        suggest_mode: SuggestMode,
    },
    Phrase {
        field: String,
        text: String,
        size: usize,
        max_errors: f64,
        confidence: f64,
        highlight: Option<HighlightTags>,
    },
    Completion {
        field: String,
        prefix: String,
        size: usize,
        skip_duplicates: bool,
        fuzzy: Option<FuzzyOptions>,
    },
}

impl Suggester {
    fn to_json(&self) -> Value {
        match self {
            Self::Term {
                field,
                text,
                size,
                max_edits,
                min_word_length,
                prefix_length,
                suggest_mode,
            } => {
                json!({
                    "text": text,
                    "term": {
                        "field": field,
                        "size": size,
                        "max_edits": max_edits,
                        "min_word_length": min_word_length,
                        "prefix_length": prefix_length,
                        "suggest_mode": suggest_mode.as_str()
                    }
                })
            }
            Self::Phrase {
                field,
                text,
                size,
                max_errors,
                confidence,
                highlight,
            } => {
                let mut phrase = json!({
                    "field": field,
                    "size": size,
                    "max_errors": max_errors,
                    "confidence": confidence
                });

                if let Some(hl) = highlight {
                    phrase["highlight"] = json!({
                        "pre_tag": hl.pre_tag,
                        "post_tag": hl.post_tag
                    });
                }

                json!({
                    "text": text,
                    "phrase": phrase
                })
            }
            Self::Completion {
                field,
                prefix,
                size,
                skip_duplicates,
                fuzzy,
            } => {
                let mut completion = json!({
                    "field": field,
                    "size": size,
                    "skip_duplicates": skip_duplicates
                });

                if let Some(fuzz) = fuzzy {
                    completion["fuzzy"] = json!({
                        "fuzziness": fuzz.fuzziness,
                        "transpositions": fuzz.transpositions,
                        "min_length": fuzz.min_length,
                        "prefix_length": fuzz.prefix_length
                    });
                }

                json!({
                    "prefix": prefix,
                    "completion": completion
                })
            }
        }
    }
}

/// Suggest mode for term suggester.
#[derive(Debug, Clone, Copy)]
pub enum SuggestMode {
    Missing,
    Popular,
    Always,
}

impl SuggestMode {
    fn as_str(&self) -> &str {
        match self {
            Self::Missing => "missing",
            Self::Popular => "popular",
            Self::Always => "always",
        }
    }
}

/// Highlight tags for phrase suggester.
#[derive(Debug, Clone)]
pub struct HighlightTags {
    pub pre_tag: String,
    pub post_tag: String,
}

/// Fuzzy options for completion suggester.
#[derive(Debug, Clone)]
pub struct FuzzyOptions {
    pub fuzziness: u32,
    pub transpositions: bool,
    pub min_length: usize,
    pub prefix_length: usize,
}

/// Suggestion results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionResults {
    pub suggestions: HashMap<String, Vec<SuggestionOption>>,
}

/// Individual suggestion option.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionOption {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlighted: Option<String>,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freq: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collate_match: Option<bool>,
}

/// Autocomplete helper for common use cases.
pub struct Autocomplete {
    client: SearchClient,
}

impl Autocomplete {
    /// Create a new autocomplete helper.
    pub fn new(client: SearchClient) -> Self {
        Self { client }
    }

    /// Perform autocomplete search.
    pub async fn search(
        &self,
        index: &str,
        field: &str,
        prefix: &str,
        fuzzy: bool,
    ) -> SearchResult<Vec<String>> {
        let mut suggester = SuggestionSearch::new(self.client.clone(), index);

        if fuzzy {
            suggester = suggester.fuzzy_completion("autocomplete", field, prefix, 1);
        } else {
            suggester = suggester.completion_suggester("autocomplete", field, prefix);
        }

        let results = suggester.execute().await?;

        let suggestions = results
            .suggestions
            .get("autocomplete")
            .map(|options| options.iter().map(|opt| opt.text.clone()).collect())
            .unwrap_or_default();

        Ok(suggestions)
    }

    /// Perform spell correction.
    pub async fn spell_correct(
        &self,
        index: &str,
        field: &str,
        text: &str,
    ) -> SearchResult<Vec<String>> {
        let suggester = SuggestionSearch::new(self.client.clone(), index)
            .term_suggester("correction", field, text);

        let results = suggester.execute().await?;

        let corrections = results
            .suggestions
            .get("correction")
            .map(|options| options.iter().map(|opt| opt.text.clone()).collect())
            .unwrap_or_default();

        Ok(corrections)
    }

    /// Perform phrase suggestion.
    pub async fn suggest_phrase(
        &self,
        index: &str,
        field: &str,
        text: &str,
    ) -> SearchResult<Vec<String>> {
        let suggester = SuggestionSearch::new(self.client.clone(), index)
            .phrase_suggester("phrase", field, text);

        let results = suggester.execute().await?;

        let suggestions = results
            .suggestions
            .get("phrase")
            .map(|options| {
                options
                    .iter()
                    .map(|opt| opt.highlighted.clone().unwrap_or_else(|| opt.text.clone()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggest_mode() {
        assert_eq!(SuggestMode::Missing.as_str(), "missing");
        assert_eq!(SuggestMode::Popular.as_str(), "popular");
        assert_eq!(SuggestMode::Always.as_str(), "always");
    }

    #[test]
    fn test_fuzzy_options() {
        let opts = FuzzyOptions {
            fuzziness: 2,
            transpositions: true,
            min_length: 3,
            prefix_length: 1,
        };
        assert_eq!(opts.fuzziness, 2);
        assert!(opts.transpositions);
    }
}
