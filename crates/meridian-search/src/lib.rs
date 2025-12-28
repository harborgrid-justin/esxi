//! Meridian Search - Enterprise Search System
//!
//! A comprehensive search system for the Meridian GIS Platform with Elasticsearch integration.
//!
//! # Features
//!
//! - **Elasticsearch Integration**: Full-featured client with connection pooling
//! - **Geo-spatial Search**: Search by location, distance, bounding boxes, and shapes
//! - **Full-text Search**: Advanced text search with custom analyzers and highlighting
//! - **Faceted Search**: Aggregations and facets for filtering and analytics
//! - **Search Suggestions**: Autocomplete and spell correction
//! - **Fuzzy Matching**: Typo tolerance and approximate matching
//! - **Result Ranking**: Custom scoring and boosting algorithms
//! - **Index Management**: Index creation, aliases, and zero-downtime updates
//! - **Bulk Indexing**: High-performance batch operations
//! - **Real-time Updates**: Immediate document indexing and updates
//! - **Search Analytics**: Query logging and performance tracking
//! - **Multi-language Support**: Language detection and multi-language search
//!
//! # Example
//!
//! ```rust,no_run
//! use meridian_search::{SearchClient, SearchConfig, FullTextSearch, MatchOperator};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a search client
//!     let config = SearchConfig {
//!         nodes: vec!["http://localhost:9200".to_string()],
//!         username: None,
//!         password: None,
//!         timeout: 30,
//!         max_retries: 3,
//!         compression: true,
//!         cert_validation: true,
//!     };
//!
//!     let client = SearchClient::new(config)?;
//!
//!     // Perform a search
//!     let results = FullTextSearch::new(client, "my_index")
//!         .match_query("title", "GIS mapping", MatchOperator::And)
//!         .size(20)
//!         .execute::<serde_json::Value>()
//!         .await?;
//!
//!     println!("Found {} results", results.total);
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod analytics;
pub mod bulk;
pub mod client;
pub mod error;
pub mod facets;
pub mod fulltext;
pub mod fuzzy;
pub mod geo;
pub mod i18n;
pub mod index;
pub mod ranking;
pub mod realtime;
pub mod suggest;

// Re-export commonly used types
pub use client::{ClusterHealth, HealthStatus, SearchClient, SearchConfig};
pub use error::{SearchError, SearchResult};

// Geo-spatial
pub use geo::{
    GeoSearch, GeoSearchHit, GeoSearchResults, GeoShape, ShapeRelation, SortOrder,
};

// Full-text search
pub use fulltext::{
    FullTextSearch, HighlightConfig, MatchOperator, MultiMatchType, RangeFilter, SearchHit,
    SearchResults,
};

// Faceted search
pub use facets::{
    Bucket, DateInterval, DistanceRange, DistanceUnit, FacetResult, FacetResults, FacetedSearch,
    GeoPoint, RangeBucket, StatsResult,
};

// Suggestions
pub use suggest::{
    Autocomplete, SuggestMode, SuggestionOption, SuggestionResults, SuggestionSearch,
};

// Fuzzy matching
pub use fuzzy::{
    Fuzziness, FuzzyConfig, FuzzySearch, FuzzySearchHit, FuzzySearchResults, LevenshteinDistance,
    TypoTolerance,
};

// Ranking
pub use ranking::{
    BoostMode, DecayType, FieldModifier, GeoLocation, RankingHit, RankingResults, RankingSearch,
    ScoreMode,
};

// Index management
pub use index::{
    Analyzer, AnalysisSettings, FieldMapping, IndexManager, IndexMappings, IndexSettings,
    TokenFilter, Tokenizer,
};

// Bulk operations
pub use bulk::{
    BulkError, BulkIndexStats, BulkIndexer, BulkResponse, ParallelBulkIndexer,
};

// Real-time operations
pub use realtime::{
    ChangeTracker, DeleteResponse, DocumentResponse, IndexResponse, RealtimeIndexer,
    RefreshPolicy, ScriptUpdate, UpdateResponse,
};

// Analytics
pub use analytics::{
    ABTesting, ClickEvent, ClickTracker, Experiment, QueryLog, QueryStats, SearchAnalytics,
    SearchStatistics,
};

// Internationalization
pub use i18n::{
    Language, LanguageDetector, MultiLanguageHit, MultiLanguageResults, MultiLanguageSearch,
    TextAnalyzer, TranslationHelper,
};

/// Version of the meridian-search crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports.
pub mod prelude {
    //! Prelude module containing commonly used types and traits.

    pub use crate::analytics::{QueryLog, SearchAnalytics};
    pub use crate::bulk::{BulkIndexer, ParallelBulkIndexer};
    pub use crate::client::{SearchClient, SearchConfig};
    pub use crate::error::{SearchError, SearchResult};
    pub use crate::facets::FacetedSearch;
    pub use crate::fulltext::{FullTextSearch, MatchOperator};
    pub use crate::fuzzy::FuzzySearch;
    pub use crate::geo::GeoSearch;
    pub use crate::i18n::{Language, LanguageDetector};
    pub use crate::index::IndexManager;
    pub use crate::ranking::RankingSearch;
    pub use crate::realtime::{RealtimeIndexer, RefreshPolicy};
    pub use crate::suggest::{Autocomplete, SuggestionSearch};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.starts_with("0.1"));
    }

    #[test]
    fn test_config_default() {
        let config = SearchConfig::default();
        assert_eq!(config.nodes.len(), 1);
        assert!(config.nodes[0].contains("localhost"));
    }
}