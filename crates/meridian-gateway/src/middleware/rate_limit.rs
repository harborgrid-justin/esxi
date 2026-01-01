//! Rate Limiting Middleware
//!
//! Enterprise rate limiting using token bucket algorithm with per-user/route limits.

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use dashmap::DashMap;
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// Rate limit errors
#[derive(Debug, Error)]
pub enum RateLimitError {
    /// Rate limit has been exceeded
    #[error("Rate limit exceeded")]
    Exceeded,
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response()
    }
}

type Limiter = Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>;

/// Rate limit key
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum RateLimitKey {
    Route(String),
    UserRoute(String, String),
}

impl RateLimitKey {
    fn to_string(&self) -> String {
        match self {
            Self::Route(route) => format!("route:{}", route),
            Self::UserRoute(user, route) => format!("user:{}:route:{}", user, route),
        }
    }
}

/// Rate Limiter
pub struct RateLimitMiddleware {
    /// Per-route limiters
    limiters: Arc<DashMap<String, Limiter>>,

    /// Default quota
    default_quota: Quota,

    /// Per-route quotas
    route_quotas: Arc<DashMap<String, Quota>>,
}

impl RateLimitMiddleware {
    /// Create a new rate limiter
    pub fn new(requests: u32, window: Duration) -> Self {
        let quota = Quota::with_period(window)
            .unwrap()
            .allow_burst(NonZeroU32::new(requests).unwrap());

        Self {
            limiters: Arc::new(DashMap::new()),
            default_quota: quota,
            route_quotas: Arc::new(DashMap::new()),
        }
    }

    /// Configure rate limit for specific route
    pub fn with_route_limit(self, route: String, requests: u32, window: Duration) -> Self {
        let quota = Quota::with_period(window)
            .unwrap()
            .allow_burst(NonZeroU32::new(requests).unwrap());

        self.route_quotas.insert(route, quota);
        self
    }

    /// Get or create limiter for key
    fn get_limiter(&self, key: &RateLimitKey, quota: Quota) -> Limiter {
        let key_str = key.to_string();

        self.limiters
            .entry(key_str)
            .or_insert_with(|| Arc::new(GovernorRateLimiter::direct(quota)))
            .clone()
    }

    /// Check rate limit for request
    pub fn check_limit(
        &self,
        route: &str,
        user_id: Option<&str>,
    ) -> Result<(), RateLimitError> {
        // Determine which quota to use
        let quota = self
            .route_quotas
            .get(route)
            .map(|q| *q)
            .unwrap_or(self.default_quota);

        // Build rate limit key
        let key = if let Some(user) = user_id {
            RateLimitKey::UserRoute(user.to_string(), route.to_string())
        } else {
            RateLimitKey::Route(route.to_string())
        };

        // Get limiter and check
        let limiter = self.get_limiter(&key, quota);

        limiter
            .check()
            .map_err(|_| RateLimitError::Exceeded)?;

        Ok(())
    }

    /// Get remaining quota
    pub fn remaining(&self, route: &str, user_id: Option<&str>) -> Option<u32> {
        let _quota = self
            .route_quotas
            .get(route)
            .map(|q| *q)
            .unwrap_or(self.default_quota);

        let key = if let Some(user) = user_id {
            RateLimitKey::UserRoute(user.to_string(), route.to_string())
        } else {
            RateLimitKey::Route(route.to_string())
        };

        let key_str = key.to_string();
        self.limiters.get(&key_str).and_then(|limiter| {
            // Try to check without consuming
            limiter.check().ok().map(|_| 1) // Simplified
        })
    }

    /// Reset limits for a key
    pub fn reset(&self, route: &str, user_id: Option<&str>) {
        let key = if let Some(user) = user_id {
            RateLimitKey::UserRoute(user.to_string(), route.to_string())
        } else {
            RateLimitKey::Route(route.to_string())
        };

        let key_str = key.to_string();
        self.limiters.remove(&key_str);
    }
}

impl Clone for RateLimitMiddleware {
    fn clone(&self) -> Self {
        Self {
            limiters: Arc::clone(&self.limiters),
            default_quota: self.default_quota,
            route_quotas: Arc::clone(&self.route_quotas),
        }
    }
}

/// Middleware handler
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimitMiddleware>>,
    request: Request,
    next: Next,
) -> Result<Response, RateLimitError> {
    // Extract route from request
    let route = request.uri().path().to_string();

    // Extract user ID from auth context if available
    let user_id = request
        .extensions()
        .get::<crate::middleware::auth::AuthContext>()
        .map(|ctx| ctx.user_id.as_str());

    // Check rate limit
    limiter.check_limit(&route, user_id)?;

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimitMiddleware::new(5, Duration::from_secs(60));

        // Should allow 5 requests
        for _ in 0..5 {
            assert!(limiter.check_limit("/api/users", None).is_ok());
        }

        // 6th request should be rate limited
        assert!(limiter.check_limit("/api/users", None).is_err());
    }

    #[test]
    fn test_rate_limiter_per_route() {
        let limiter = RateLimitMiddleware::new(5, Duration::from_secs(60))
            .with_route_limit("/api/premium".to_string(), 10, Duration::from_secs(60));

        // Default route: 5 requests
        for _ in 0..5 {
            assert!(limiter.check_limit("/api/users", None).is_ok());
        }
        assert!(limiter.check_limit("/api/users", None).is_err());

        // Premium route: 10 requests
        for _ in 0..10 {
            assert!(limiter.check_limit("/api/premium", None).is_ok());
        }
        assert!(limiter.check_limit("/api/premium", None).is_err());
    }

    #[test]
    fn test_rate_limiter_per_user() {
        let limiter = RateLimitMiddleware::new(5, Duration::from_secs(60));

        // User A: 5 requests
        for _ in 0..5 {
            assert!(limiter.check_limit("/api/users", Some("user-a")).is_ok());
        }
        assert!(limiter.check_limit("/api/users", Some("user-a")).is_err());

        // User B: still has quota
        assert!(limiter.check_limit("/api/users", Some("user-b")).is_ok());
    }
}
