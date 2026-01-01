//! Cache Policy
//!
//! Defines caching policies and strategies for different routes and responses.

use crate::config::CachePolicy as ConfigCachePolicy;
use http::{HeaderMap, StatusCode};
use std::time::Duration;

/// Cache decision
#[derive(Debug, Clone, PartialEq)]
pub enum CacheDecision {
    /// Cache the response with TTL
    Cache(Duration),

    /// Do not cache the response
    NoCache,

    /// Use stale cache if available
    Stale,
}

/// Cache Policy Manager
pub struct CachePolicyManager {
    default_ttl: Duration,
    cacheable_status_codes: Vec<u16>,
}

impl CachePolicyManager {
    /// Create a new cache policy manager
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            default_ttl,
            cacheable_status_codes: vec![200, 203, 204, 206, 300, 301, 404, 405, 410, 414, 501],
        }
    }

    /// Decide if response should be cached
    pub fn should_cache(
        &self,
        status: StatusCode,
        headers: &HeaderMap,
        route_policy: Option<&ConfigCachePolicy>,
    ) -> CacheDecision {
        // Check route policy first
        if let Some(policy) = route_policy {
            if !policy.enabled {
                return CacheDecision::NoCache;
            }

            // Check status code
            if !policy.cache_status_codes.is_empty()
                && !policy.cache_status_codes.contains(&status.as_u16())
            {
                return CacheDecision::NoCache;
            }
        }

        // Check Cache-Control header
        if let Some(cache_control) = headers.get("cache-control") {
            if let Ok(value) = cache_control.to_str() {
                if value.contains("no-cache")
                    || value.contains("no-store")
                    || value.contains("private")
                {
                    return CacheDecision::NoCache;
                }

                // Extract max-age
                if let Some(max_age) = self.extract_max_age(value) {
                    return CacheDecision::Cache(Duration::from_secs(max_age));
                }
            }
        }

        // Check if status code is cacheable
        if !self.cacheable_status_codes.contains(&status.as_u16()) {
            return CacheDecision::NoCache;
        }

        // Use route policy TTL or default
        let ttl = route_policy
            .map(|p| p.ttl)
            .unwrap_or(self.default_ttl);

        CacheDecision::Cache(ttl)
    }

    /// Extract max-age from Cache-Control header
    fn extract_max_age(&self, cache_control: &str) -> Option<u64> {
        for directive in cache_control.split(',') {
            let directive = directive.trim();
            if let Some(value) = directive.strip_prefix("max-age=") {
                if let Ok(age) = value.parse::<u64>() {
                    return Some(age);
                }
            }
        }
        None
    }

    /// Get cache key vary headers from response
    pub fn get_vary_headers(&self, headers: &HeaderMap) -> Vec<String> {
        if let Some(vary) = headers.get("vary") {
            if let Ok(value) = vary.to_str() {
                return value
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .collect();
            }
        }
        vec![]
    }

    /// Check if cache is stale but usable
    pub fn can_use_stale(
        &self,
        headers: &HeaderMap,
        cached_age: Duration,
    ) -> bool {
        if let Some(cache_control) = headers.get("cache-control") {
            if let Ok(value) = cache_control.to_str() {
                // Check stale-while-revalidate
                if let Some(stale_time) = self.extract_stale_while_revalidate(value) {
                    return cached_age <= Duration::from_secs(stale_time);
                }

                // Check stale-if-error
                if value.contains("stale-if-error") {
                    return true;
                }
            }
        }
        false
    }

    /// Extract stale-while-revalidate from Cache-Control
    fn extract_stale_while_revalidate(&self, cache_control: &str) -> Option<u64> {
        for directive in cache_control.split(',') {
            let directive = directive.trim();
            if let Some(value) = directive.strip_prefix("stale-while-revalidate=") {
                if let Ok(age) = value.parse::<u64>() {
                    return Some(age);
                }
            }
        }
        None
    }

    /// Add cache headers to response
    pub fn add_cache_headers(
        &self,
        headers: &mut HeaderMap,
        ttl: Duration,
        hit: bool,
    ) {
        // Add X-Cache header
        let cache_status = if hit { "HIT" } else { "MISS" };
        headers.insert(
            "x-cache",
            cache_status.parse().unwrap(),
        );

        // Add Age header
        if hit {
            headers.insert(
                "age",
                "0".parse().unwrap(),
            );
        }

        // Add Cache-Control if not present
        if !headers.contains_key("cache-control") {
            let cache_control = format!("public, max-age={}", ttl.as_secs());
            headers.insert(
                "cache-control",
                cache_control.parse().unwrap(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_cache_with_no_cache() {
        let manager = CachePolicyManager::new(Duration::from_secs(300));
        let mut headers = HeaderMap::new();
        headers.insert("cache-control", "no-cache".parse().unwrap());

        let decision = manager.should_cache(StatusCode::OK, &headers, None);
        assert_eq!(decision, CacheDecision::NoCache);
    }

    #[test]
    fn test_should_cache_with_max_age() {
        let manager = CachePolicyManager::new(Duration::from_secs(300));
        let mut headers = HeaderMap::new();
        headers.insert("cache-control", "max-age=600".parse().unwrap());

        let decision = manager.should_cache(StatusCode::OK, &headers, None);
        assert_eq!(decision, CacheDecision::Cache(Duration::from_secs(600)));
    }

    #[test]
    fn test_should_cache_default() {
        let manager = CachePolicyManager::new(Duration::from_secs(300));
        let headers = HeaderMap::new();

        let decision = manager.should_cache(StatusCode::OK, &headers, None);
        assert_eq!(decision, CacheDecision::Cache(Duration::from_secs(300)));
    }

    #[test]
    fn test_should_not_cache_bad_status() {
        let manager = CachePolicyManager::new(Duration::from_secs(300));
        let headers = HeaderMap::new();

        let decision = manager.should_cache(StatusCode::INTERNAL_SERVER_ERROR, &headers, None);
        assert_eq!(decision, CacheDecision::NoCache);
    }

    #[test]
    fn test_vary_headers() {
        let manager = CachePolicyManager::new(Duration::from_secs(300));
        let mut headers = HeaderMap::new();
        headers.insert("vary", "Accept, Accept-Encoding".parse().unwrap());

        let vary = manager.get_vary_headers(&headers);
        assert_eq!(vary, vec!["accept", "accept-encoding"]);
    }
}
