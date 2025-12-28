//! Error types for the Meridian search system.

use thiserror::Error;

/// Result type for search operations.
pub type SearchResult<T> = Result<T, SearchError>;

/// Errors that can occur during search operations.
#[derive(Error, Debug)]
pub enum SearchError {
    /// Elasticsearch connection error
    #[error("Elasticsearch connection error: {0}")]
    ConnectionError(String),

    /// Query parsing error
    #[error("Query parsing error: {0}")]
    QueryParseError(String),

    /// Index error
    #[error("Index error: {0}")]
    IndexError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Elasticsearch API error
    #[error("Elasticsearch API error: {0}")]
    ElasticsearchError(String),

    /// Geo-spatial query error
    #[error("Geo-spatial query error: {0}")]
    GeoError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Bulk operation error
    #[error("Bulk operation error: {0}")]
    BulkError(String),

    /// Index not found
    #[error("Index not found: {0}")]
    IndexNotFound(String),

    /// Document not found
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Invalid search parameters
    #[error("Invalid search parameters: {0}")]
    InvalidParameters(String),

    /// Timeout error
    #[error("Operation timeout: {0}")]
    Timeout(String),

    /// Language detection error
    #[error("Language detection error: {0}")]
    LanguageError(String),

    /// Pool error
    #[error("Connection pool error: {0}")]
    PoolError(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<elasticsearch::Error> for SearchError {
    fn from(err: elasticsearch::Error) -> Self {
        SearchError::ElasticsearchError(err.to_string())
    }
}

impl From<url::ParseError> for SearchError {
    fn from(err: url::ParseError) -> Self {
        SearchError::ConfigError(format!("URL parse error: {}", err))
    }
}

impl From<anyhow::Error> for SearchError {
    fn from(err: anyhow::Error) -> Self {
        SearchError::Internal(err.to_string())
    }
}
