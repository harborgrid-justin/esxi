//! Search analytics and query logging.

use crate::client::SearchClient;
use crate::error::{SearchError, SearchResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use metrics::{counter, histogram, gauge};

/// Search analytics tracker.
pub struct SearchAnalytics {
    client: SearchClient,
    analytics_index: String,
    cache: Arc<RwLock<AnalyticsCache>>,
}

impl SearchAnalytics {
    /// Create a new search analytics tracker.
    pub fn new(client: SearchClient, analytics_index: impl Into<String>) -> Self {
        Self {
            client,
            analytics_index: analytics_index.into(),
            cache: Arc::new(RwLock::new(AnalyticsCache::new())),
        }
    }

    /// Log a search query.
    pub async fn log_query(&self, log: QueryLog) -> SearchResult<()> {
        debug!("Logging search query: {}", log.query);

        // Update metrics
        counter!("search.queries.total", 1);
        histogram!("search.queries.duration_ms", log.duration_ms as f64);

        if log.result_count == 0 {
            counter!("search.queries.no_results", 1);
        }

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.add_query(&log);
        }

        // Index the log entry
        let response = self
            .client
            .client()
            .index(elasticsearch::IndexParts::Index(&self.analytics_index))
            .body(&log)
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to log query: {}",
                error_text
            )));
        }

        Ok(())
    }

    /// Get popular queries.
    pub async fn popular_queries(&self, limit: usize) -> SearchResult<Vec<QueryStats>> {
        info!("Fetching top {} popular queries", limit);

        let cache = self.cache.read().await;
        let mut queries: Vec<_> = cache
            .query_counts
            .iter()
            .map(|(query, count)| QueryStats {
                query: query.clone(),
                count: *count,
                avg_duration_ms: cache
                    .query_durations
                    .get(query)
                    .map(|durations| {
                        durations.iter().sum::<u64>() as f64 / durations.len() as f64
                    })
                    .unwrap_or(0.0),
            })
            .collect();

        queries.sort_by(|a, b| b.count.cmp(&a.count));
        queries.truncate(limit);

        Ok(queries)
    }

    /// Get queries with no results.
    pub async fn no_result_queries(&self, limit: usize) -> SearchResult<Vec<String>> {
        info!("Fetching queries with no results");

        let cache = self.cache.read().await;
        let mut queries: Vec<_> = cache.no_result_queries.iter().cloned().collect();
        queries.truncate(limit);

        Ok(queries)
    }

    /// Get search statistics.
    pub async fn statistics(&self) -> SearchResult<SearchStatistics> {
        info!("Calculating search statistics");

        let cache = self.cache.read().await;

        let total_queries = cache.query_counts.values().sum::<u64>();
        let unique_queries = cache.query_counts.len();
        let no_result_count = cache.no_result_queries.len();

        let avg_duration = if total_queries > 0 {
            let total_duration: u64 = cache
                .query_durations
                .values()
                .flat_map(|v| v.iter())
                .sum();
            total_duration as f64 / total_queries as f64
        } else {
            0.0
        };

        let avg_results = if total_queries > 0 {
            let total_results: u64 = cache.result_counts.values().sum();
            total_results as f64 / total_queries as f64
        } else {
            0.0
        };

        // Update gauges
        gauge!("search.stats.total_queries", total_queries as f64);
        gauge!("search.stats.unique_queries", unique_queries as f64);
        gauge!("search.stats.avg_duration_ms", avg_duration);
        gauge!("search.stats.avg_results", avg_results);

        Ok(SearchStatistics {
            total_queries,
            unique_queries,
            no_result_count,
            avg_duration_ms: avg_duration,
            avg_result_count: avg_results,
        })
    }

    /// Clear analytics cache.
    pub async fn clear_cache(&self) {
        info!("Clearing analytics cache");
        let mut cache = self.cache.write().await;
        *cache = AnalyticsCache::new();
    }
}

/// In-memory analytics cache for quick aggregations.
struct AnalyticsCache {
    query_counts: HashMap<String, u64>,
    query_durations: HashMap<String, Vec<u64>>,
    result_counts: HashMap<String, u64>,
    no_result_queries: Vec<String>,
    max_cache_size: usize,
}

impl AnalyticsCache {
    fn new() -> Self {
        Self {
            query_counts: HashMap::new(),
            query_durations: HashMap::new(),
            result_counts: HashMap::new(),
            no_result_queries: Vec::new(),
            max_cache_size: 10000,
        }
    }

    fn add_query(&mut self, log: &QueryLog) {
        // Increment query count
        *self.query_counts.entry(log.query.clone()).or_insert(0) += 1;

        // Add duration
        self.query_durations
            .entry(log.query.clone())
            .or_insert_with(Vec::new)
            .push(log.duration_ms);

        // Add result count
        *self
            .result_counts
            .entry(log.query.clone())
            .or_insert(0) += log.result_count;

        // Track no-result queries
        if log.result_count == 0 && !self.no_result_queries.contains(&log.query) {
            self.no_result_queries.push(log.query.clone());
        }

        // Limit cache size
        if self.query_counts.len() > self.max_cache_size {
            self.evict_oldest();
        }
    }

    fn evict_oldest(&mut self) {
        // Simple eviction: remove queries with lowest counts
        if let Some(min_query) = self
            .query_counts
            .iter()
            .min_by_key(|(_, count)| *count)
            .map(|(query, _)| query.clone())
        {
            self.query_counts.remove(&min_query);
            self.query_durations.remove(&min_query);
            self.result_counts.remove(&min_query);
        }
    }
}

/// Query log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLog {
    pub query: String,
    pub index: String,
    pub timestamp: DateTime<Utc>,
    pub duration_ms: u64,
    pub result_count: u64,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub filters: Option<HashMap<String, serde_json::Value>>,
    pub sort: Option<Vec<String>>,
    pub from: usize,
    pub size: usize,
}

impl QueryLog {
    /// Create a new query log entry.
    pub fn new(query: impl Into<String>, index: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            index: index.into(),
            timestamp: Utc::now(),
            duration_ms: 0,
            result_count: 0,
            user_id: None,
            session_id: None,
            filters: None,
            sort: None,
            from: 0,
            size: 10,
        }
    }

    /// Set the duration.
    pub fn duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Set the result count.
    pub fn result_count(mut self, count: u64) -> Self {
        self.result_count = count;
        self
    }

    /// Set the user ID.
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the session ID.
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set filters.
    pub fn filters(mut self, filters: HashMap<String, serde_json::Value>) -> Self {
        self.filters = Some(filters);
        self
    }
}

/// Query statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub query: String,
    pub count: u64,
    pub avg_duration_ms: f64,
}

/// Overall search statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStatistics {
    pub total_queries: u64,
    pub unique_queries: usize,
    pub no_result_count: usize,
    pub avg_duration_ms: f64,
    pub avg_result_count: f64,
}

/// Click tracking for search results.
pub struct ClickTracker {
    client: SearchClient,
    clicks_index: String,
}

impl ClickTracker {
    /// Create a new click tracker.
    pub fn new(client: SearchClient, clicks_index: impl Into<String>) -> Self {
        Self {
            client,
            clicks_index: clicks_index.into(),
        }
    }

    /// Track a click on a search result.
    pub async fn track_click(&self, click: ClickEvent) -> SearchResult<()> {
        debug!(
            "Tracking click: query='{}', document_id='{}', position={}",
            click.query, click.document_id, click.position
        );

        counter!("search.clicks.total", 1);
        histogram!("search.clicks.position", click.position as f64);

        let response = self
            .client
            .client()
            .index(elasticsearch::IndexParts::Index(&self.clicks_index))
            .body(&click)
            .send()
            .await?;

        if !response.status_code().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(SearchError::IndexError(format!(
                "Failed to track click: {}",
                error_text
            )));
        }

        Ok(())
    }

    /// Get click-through rate for a query.
    pub async fn click_through_rate(&self, query: &str) -> SearchResult<f64> {
        // This would require aggregating data from both queries and clicks
        // For now, return a placeholder
        info!("Calculating CTR for query: {}", query);
        Ok(0.0)
    }
}

/// Click event on a search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickEvent {
    pub query: String,
    pub document_id: String,
    pub position: usize,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

impl ClickEvent {
    /// Create a new click event.
    pub fn new(
        query: impl Into<String>,
        document_id: impl Into<String>,
        position: usize,
    ) -> Self {
        Self {
            query: query.into(),
            document_id: document_id.into(),
            position,
            timestamp: Utc::now(),
            user_id: None,
            session_id: None,
        }
    }

    /// Set the user ID.
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the session ID.
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}

/// A/B testing for search algorithms.
pub struct ABTesting {
    experiments: HashMap<String, Experiment>,
}

impl ABTesting {
    /// Create a new A/B testing manager.
    pub fn new() -> Self {
        Self {
            experiments: HashMap::new(),
        }
    }

    /// Add an experiment.
    pub fn add_experiment(&mut self, name: impl Into<String>, experiment: Experiment) {
        self.experiments.insert(name.into(), experiment);
    }

    /// Get variant for a user.
    pub fn get_variant(&self, experiment_name: &str, user_id: &str) -> Option<&str> {
        self.experiments
            .get(experiment_name)
            .map(|exp| exp.get_variant(user_id))
    }
}

impl Default for ABTesting {
    fn default() -> Self {
        Self::new()
    }
}

/// A/B test experiment.
#[derive(Debug, Clone)]
pub struct Experiment {
    pub name: String,
    pub variants: Vec<String>,
    pub split: Vec<f64>, // Percentage split (must sum to 1.0)
}

impl Experiment {
    /// Get the variant for a user (deterministic based on user_id).
    pub fn get_variant(&self, user_id: &str) -> &str {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        let hash = hasher.finish();

        let percentage = (hash % 100) as f64 / 100.0;
        let mut cumulative = 0.0;

        for (idx, split) in self.split.iter().enumerate() {
            cumulative += split;
            if percentage < cumulative {
                return &self.variants[idx];
            }
        }

        &self.variants[self.variants.len() - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_log_builder() {
        let log = QueryLog::new("test query", "test_index")
            .duration(100)
            .result_count(42)
            .user_id("user123");

        assert_eq!(log.query, "test query");
        assert_eq!(log.duration_ms, 100);
        assert_eq!(log.result_count, 42);
        assert_eq!(log.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_click_event_builder() {
        let click = ClickEvent::new("test query", "doc123", 5)
            .user_id("user123")
            .session_id("session456");

        assert_eq!(click.query, "test query");
        assert_eq!(click.document_id, "doc123");
        assert_eq!(click.position, 5);
        assert_eq!(click.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_experiment_variant_distribution() {
        let experiment = Experiment {
            name: "test_experiment".to_string(),
            variants: vec!["A".to_string(), "B".to_string()],
            split: vec![0.5, 0.5],
        };

        // Test that same user always gets same variant
        let variant1 = experiment.get_variant("user123");
        let variant2 = experiment.get_variant("user123");
        assert_eq!(variant1, variant2);

        // Test that variant is one of the expected values
        assert!(variant1 == "A" || variant1 == "B");
    }

    #[test]
    fn test_analytics_cache() {
        let mut cache = AnalyticsCache::new();

        let log = QueryLog::new("test", "index").duration(100).result_count(10);
        cache.add_query(&log);

        assert_eq!(cache.query_counts.get("test"), Some(&1));
        assert_eq!(cache.query_durations.get("test").unwrap().len(), 1);
    }
}
