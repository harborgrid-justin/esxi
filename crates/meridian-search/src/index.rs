//! Index management and aliases.

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use elasticsearch::indices::{IndicesCreateParts, IndicesDeleteParts, IndicesExistsParts, IndicesPutAliasParts, IndicesDeleteAliasParts, IndicesGetAliasParts};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::{debug, info};

/// Index manager for creating and managing Elasticsearch indices.
pub struct IndexManager {
    client: SearchClient,
}

impl IndexManager {
    /// Create a new index manager.
    pub fn new(client: SearchClient) -> Self {
        Self { client }
    }

    /// Create a new index.
    pub async fn create_index(
        &self,
        name: &str,
        settings: IndexSettings,
        mappings: IndexMappings,
    ) -> SearchResult<()> {
        info!("Creating index: {}", name);

        let body = json!({
            "settings": settings.to_json(),
            "mappings": mappings.to_json()
        });

        debug!("Index definition: {}", serde_json::to_string_pretty(&body).unwrap());

        let response = self
            .client
            .client()
            .indices()
            .create(IndicesCreateParts::Index(name))
            .body(body)
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to create index: {}",
                error_text
            )));
        }

        info!("Index '{}' created successfully", name);
        Ok(())
    }

    /// Delete an index.
    pub async fn delete_index(&self, name: &str) -> SearchResult<()> {
        info!("Deleting index: {}", name);

        let response = self
            .client
            .client()
            .indices()
            .delete(IndicesDeleteParts::Index(&[name]))
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to delete index: {}",
                error_text
            )));
        }

        info!("Index '{}' deleted successfully", name);
        Ok(())
    }

    /// Check if an index exists.
    pub async fn index_exists(&self, name: &str) -> SearchResult<bool> {
        debug!("Checking if index exists: {}", name);

        let response = self
            .client
            .client()
            .indices()
            .exists(IndicesExistsParts::Index(&[name]))
            .send()
            .await?;

        Ok(response.status_code().is_success())
    }

    /// Create an alias.
    pub async fn create_alias(&self, index: &str, alias: &str) -> SearchResult<()> {
        info!("Creating alias '{}' for index '{}'", alias, index);

        let response = self
            .client
            .client()
            .indices()
            .put_alias(IndicesPutAliasParts::IndexName(&[index], alias))
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to create alias: {}",
                error_text
            )));
        }

        info!("Alias '{}' created successfully", alias);
        Ok(())
    }

    /// Delete an alias.
    pub async fn delete_alias(&self, index: &str, alias: &str) -> SearchResult<()> {
        info!("Deleting alias '{}' from index '{}'", alias, index);

        let response = self
            .client
            .client()
            .indices()
            .delete_alias(IndicesDeleteAliasParts::IndexName(&[index], &[alias]))
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to delete alias: {}",
                error_text
            )));
        }

        info!("Alias '{}' deleted successfully", alias);
        Ok(())
    }

    /// Get aliases for an index.
    pub async fn get_aliases(&self, index: &str) -> SearchResult<HashMap<String, Value>> {
        debug!("Getting aliases for index: {}", index);

        let response = self
            .client
            .client()
            .indices()
            .get_alias(IndicesGetAliasParts::Index(&[index]))
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to get aliases: {}",
                error_text
            )));
        }

        let body: HashMap<String, Value> = response.json().await?;
        Ok(body)
    }

    /// Perform atomic alias swap (zero-downtime reindexing).
    pub async fn swap_alias(&self, old_index: &str, new_index: &str, alias: &str) -> SearchResult<()> {
        info!(
            "Swapping alias '{}' from '{}' to '{}'",
            alias, old_index, new_index
        );

        let body = json!({
            "actions": [
                { "remove": { "index": old_index, "alias": alias } },
                { "add": { "index": new_index, "alias": alias } }
            ]
        });

        let response = self
            .client
            .client()
            .indices()
            .update_aliases()
            .body(body)
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to swap alias: {}",
                error_text
            )));
        }

        info!("Alias swap completed successfully");
        Ok(())
    }

    /// Refresh an index (make recent changes searchable).
    pub async fn refresh_index(&self, index: &str) -> SearchResult<()> {
        debug!("Refreshing index: {}", index);

        let response = self
            .client
            .client()
            .indices()
            .refresh(elasticsearch::indices::IndicesRefreshParts::Index(&[index]))
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to refresh index: {}",
                error_text
            )));
        }

        debug!("Index '{}' refreshed successfully", index);
        Ok(())
    }
}

/// Index settings configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSettings {
    pub number_of_shards: u32,
    pub number_of_replicas: u32,
    pub refresh_interval: String,
    pub analysis: Option<AnalysisSettings>,
}

impl Default for IndexSettings {
    fn default() -> Self {
        Self {
            number_of_shards: 1,
            number_of_replicas: 1,
            refresh_interval: "1s".to_string(),
            analysis: None,
        }
    }
}

impl IndexSettings {
    fn to_json(&self) -> Value {
        let mut settings = json!({
            "number_of_shards": self.number_of_shards,
            "number_of_replicas": self.number_of_replicas,
            "refresh_interval": self.refresh_interval
        });

        if let Some(analysis) = &self.analysis {
            settings["analysis"] = analysis.to_json();
        }

        settings
    }
}

/// Analysis settings for custom analyzers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSettings {
    pub analyzers: HashMap<String, Analyzer>,
    pub tokenizers: HashMap<String, Tokenizer>,
    pub filters: HashMap<String, TokenFilter>,
}

impl AnalysisSettings {
    fn to_json(&self) -> Value {
        json!({
            "analyzer": self.analyzers.iter().map(|(k, v)| (k.clone(), v.to_json())).collect::<HashMap<_, _>>(),
            "tokenizer": self.tokenizers.iter().map(|(k, v)| (k.clone(), v.to_json())).collect::<HashMap<_, _>>(),
            "filter": self.filters.iter().map(|(k, v)| (k.clone(), v.to_json())).collect::<HashMap<_, _>>()
        })
    }
}

/// Custom analyzer definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analyzer {
    pub tokenizer: String,
    pub filters: Vec<String>,
    pub char_filters: Vec<String>,
}

impl Analyzer {
    fn to_json(&self) -> Value {
        json!({
            "type": "custom",
            "tokenizer": self.tokenizer,
            "filter": self.filters,
            "char_filter": self.char_filters
        })
    }
}

/// Tokenizer definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Tokenizer {
    Standard,
    EdgeNGram { min_gram: usize, max_gram: usize },
    NGram { min_gram: usize, max_gram: usize },
    Whitespace,
    Keyword,
}

impl Tokenizer {
    fn to_json(&self) -> Value {
        match self {
            Self::Standard => json!({ "type": "standard" }),
            Self::EdgeNGram { min_gram, max_gram } => json!({
                "type": "edge_ngram",
                "min_gram": min_gram,
                "max_gram": max_gram
            }),
            Self::NGram { min_gram, max_gram } => json!({
                "type": "ngram",
                "min_gram": min_gram,
                "max_gram": max_gram
            }),
            Self::Whitespace => json!({ "type": "whitespace" }),
            Self::Keyword => json!({ "type": "keyword" }),
        }
    }
}

/// Token filter definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TokenFilter {
    Lowercase,
    Uppercase,
    Stop { stopwords: Vec<String> },
    Stemmer { language: String },
    Snowball { language: String },
    Synonym { synonyms: Vec<String> },
    EdgeNGram { min_gram: usize, max_gram: usize },
    Phonetic { encoder: String },
}

impl TokenFilter {
    fn to_json(&self) -> Value {
        match self {
            Self::Lowercase => json!({ "type": "lowercase" }),
            Self::Uppercase => json!({ "type": "uppercase" }),
            Self::Stop { stopwords } => json!({
                "type": "stop",
                "stopwords": stopwords
            }),
            Self::Stemmer { language } => json!({
                "type": "stemmer",
                "language": language
            }),
            Self::Snowball { language } => json!({
                "type": "snowball",
                "language": language
            }),
            Self::Synonym { synonyms } => json!({
                "type": "synonym",
                "synonyms": synonyms
            }),
            Self::EdgeNGram { min_gram, max_gram } => json!({
                "type": "edge_ngram",
                "min_gram": min_gram,
                "max_gram": max_gram
            }),
            Self::Phonetic { encoder } => json!({
                "type": "phonetic",
                "encoder": encoder
            }),
        }
    }
}

/// Index mappings configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMappings {
    pub properties: HashMap<String, FieldMapping>,
}

impl IndexMappings {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    pub fn add_field(mut self, name: impl Into<String>, mapping: FieldMapping) -> Self {
        self.properties.insert(name.into(), mapping);
        self
    }

    fn to_json(&self) -> Value {
        json!({
            "properties": self.properties.iter().map(|(k, v)| (k.clone(), v.to_json())).collect::<HashMap<_, _>>()
        })
    }
}

impl Default for IndexMappings {
    fn default() -> Self {
        Self::new()
    }
}

/// Field mapping types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FieldMapping {
    Text {
        analyzer: Option<String>,
        search_analyzer: Option<String>,
        fields: Option<HashMap<String, FieldMapping>>,
    },
    Keyword {
        ignore_above: Option<usize>,
    },
    Long,
    Integer,
    Short,
    Byte,
    Double,
    Float,
    Boolean,
    Date {
        format: Option<String>,
    },
    GeoPoint,
    GeoShape,
    Object {
        properties: HashMap<String, FieldMapping>,
    },
    Nested {
        properties: HashMap<String, FieldMapping>,
    },
    Completion {
        analyzer: Option<String>,
        search_analyzer: Option<String>,
    },
}

impl FieldMapping {
    fn to_json(&self) -> Value {
        match self {
            Self::Text { analyzer, search_analyzer, fields } => {
                let mut obj = json!({ "type": "text" });
                if let Some(a) = analyzer {
                    obj["analyzer"] = json!(a);
                }
                if let Some(sa) = search_analyzer {
                    obj["search_analyzer"] = json!(sa);
                }
                if let Some(f) = fields {
                    obj["fields"] = json!(f.iter().map(|(k, v)| (k.clone(), v.to_json())).collect::<HashMap<_, _>>());
                }
                obj
            }
            Self::Keyword { ignore_above } => {
                let mut obj = json!({ "type": "keyword" });
                if let Some(ia) = ignore_above {
                    obj["ignore_above"] = json!(ia);
                }
                obj
            }
            Self::Long => json!({ "type": "long" }),
            Self::Integer => json!({ "type": "integer" }),
            Self::Short => json!({ "type": "short" }),
            Self::Byte => json!({ "type": "byte" }),
            Self::Double => json!({ "type": "double" }),
            Self::Float => json!({ "type": "float" }),
            Self::Boolean => json!({ "type": "boolean" }),
            Self::Date { format } => {
                let mut obj = json!({ "type": "date" });
                if let Some(f) = format {
                    obj["format"] = json!(f);
                }
                obj
            }
            Self::GeoPoint => json!({ "type": "geo_point" }),
            Self::GeoShape => json!({ "type": "geo_shape" }),
            Self::Object { properties } => {
                json!({
                    "type": "object",
                    "properties": properties.iter().map(|(k, v)| (k.clone(), v.to_json())).collect::<HashMap<_, _>>()
                })
            }
            Self::Nested { properties } => {
                json!({
                    "type": "nested",
                    "properties": properties.iter().map(|(k, v)| (k.clone(), v.to_json())).collect::<HashMap<_, _>>()
                })
            }
            Self::Completion { analyzer, search_analyzer } => {
                let mut obj = json!({ "type": "completion" });
                if let Some(a) = analyzer {
                    obj["analyzer"] = json!(a);
                }
                if let Some(sa) = search_analyzer {
                    obj["search_analyzer"] = json!(sa);
                }
                obj
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_index_settings() {
        let settings = IndexSettings::default();
        assert_eq!(settings.number_of_shards, 1);
        assert_eq!(settings.number_of_replicas, 1);
        assert_eq!(settings.refresh_interval, "1s");
    }

    #[test]
    fn test_index_mappings_builder() {
        let mappings = IndexMappings::new()
            .add_field("title", FieldMapping::Text {
                analyzer: Some("standard".to_string()),
                search_analyzer: None,
                fields: None,
            })
            .add_field("count", FieldMapping::Integer);

        assert_eq!(mappings.properties.len(), 2);
        assert!(mappings.properties.contains_key("title"));
        assert!(mappings.properties.contains_key("count"));
    }

    #[test]
    fn test_tokenizer_json() {
        let tokenizer = Tokenizer::EdgeNGram { min_gram: 2, max_gram: 10 };
        let json = tokenizer.to_json();
        assert_eq!(json["type"], "edge_ngram");
        assert_eq!(json["min_gram"], 2);
    }
}
