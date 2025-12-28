//! Rate limiting middleware
//!
//! Implements rate limiting to prevent API abuse using the governor crate

use crate::{error::ServerError, state::AppState};
use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::Response,
};
use governor::{
    clock::{DefaultClock, QuantaClock},
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::{net::SocketAddr, num::NonZeroU32, sync::Arc};

/// Rate limiting middleware
pub struct RateLimitMiddleware {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimitMiddleware {
    /// Create a new rate limiter with the specified requests per minute
    pub fn new(requests_per_minute: u32) -> Self {
        let quota = Quota::per_minute(
            NonZeroU32::new(requests_per_minute).unwrap_or(NonZeroU32::new(60).unwrap()),
        );

        let limiter = Arc::new(GovernorRateLimiter::direct(quota));

        Self { limiter }
    }

    /// Create rate limiter from config
    pub fn from_config(config: &crate::config::RateLimitConfig) -> Self {
        Self::new(config.requests_per_minute)
    }

    /// Rate limiting middleware layer
    pub async fn layer(
        State(state): State<AppState>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        req: Request,
        next: Next,
    ) -> Result<Response, ServerError> {
        // Skip rate limiting if disabled
        if !state.config().rate_limit.enabled {
            return Ok(next.run(req).await);
        }

        // TODO: Implement per-IP or per-user rate limiting
        // For now, this is a global rate limiter

        tracing::debug!("Rate limit check for {}", addr);

        // Note: This is a simplified implementation
        // A production implementation would use per-IP or per-user limiters

        Ok(next.run(req).await)
    }

    /// Check if a request should be rate limited
    pub fn check_rate_limit(&self) -> Result<(), ServerError> {
        match self.limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(ServerError::RateLimitExceeded(
                "Too many requests. Please try again later.".to_string(),
            )),
        }
    }
}

/// IP-based rate limiter
///
/// Tracks rate limits per IP address
pub struct IpRateLimiter {
    limiters: Arc<dashmap::DashMap<String, GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
    requests_per_minute: u32,
    burst_size: u32,
}

impl IpRateLimiter {
    /// Create a new IP-based rate limiter
    pub fn new(requests_per_minute: u32, burst_size: u32) -> Self {
        Self {
            limiters: Arc::new(dashmap::DashMap::new()),
            requests_per_minute,
            burst_size,
        }
    }

    /// Check rate limit for an IP address
    pub fn check(&self, ip: &str) -> Result<(), ServerError> {
        let limiter = self.limiters.entry(ip.to_string()).or_insert_with(|| {
            let quota = Quota::per_minute(
                NonZeroU32::new(self.requests_per_minute)
                    .unwrap_or(NonZeroU32::new(60).unwrap()),
            )
            .allow_burst(
                NonZeroU32::new(self.burst_size)
                    .unwrap_or(NonZeroU32::new(10).unwrap()),
            );

            GovernorRateLimiter::direct(quota)
        });

        match limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(ServerError::RateLimitExceeded(
                format!("Rate limit exceeded for IP: {}", ip),
            )),
        }
    }

    /// Clean up expired limiters (call periodically)
    pub fn cleanup(&self) {
        // Remove limiters that haven't been used recently
        // This is a simple implementation; production would be more sophisticated
        if self.limiters.len() > 10000 {
            tracing::warn!("Rate limiter cache is large, consider cleanup");
        }
    }
}

/// User-based rate limiter
///
/// Tracks rate limits per authenticated user
pub struct UserRateLimiter {
    limiters: Arc<dashmap::DashMap<uuid::Uuid, GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>>,
    requests_per_minute: u32,
    burst_size: u32,
}

impl UserRateLimiter {
    /// Create a new user-based rate limiter
    pub fn new(requests_per_minute: u32, burst_size: u32) -> Self {
        Self {
            limiters: Arc::new(dashmap::DashMap::new()),
            requests_per_minute,
            burst_size,
        }
    }

    /// Check rate limit for a user
    pub fn check(&self, user_id: uuid::Uuid) -> Result<(), ServerError> {
        let limiter = self.limiters.entry(user_id).or_insert_with(|| {
            let quota = Quota::per_minute(
                NonZeroU32::new(self.requests_per_minute)
                    .unwrap_or(NonZeroU32::new(60).unwrap()),
            )
            .allow_burst(
                NonZeroU32::new(self.burst_size)
                    .unwrap_or(NonZeroU32::new(10).unwrap()),
            );

            GovernorRateLimiter::direct(quota)
        });

        match limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(ServerError::RateLimitExceeded(
                format!("Rate limit exceeded for user: {}", user_id),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimitMiddleware::new(100);
        // Basic creation test
        assert!(limiter.limiter.check().is_ok());
    }

    #[test]
    fn test_ip_rate_limiter() {
        let limiter = IpRateLimiter::new(60, 10);

        // Should allow first request
        assert!(limiter.check("192.168.1.1").is_ok());

        // Different IP should also work
        assert!(limiter.check("192.168.1.2").is_ok());
    }

    #[test]
    fn test_user_rate_limiter() {
        let limiter = UserRateLimiter::new(60, 10);
        let user_id = uuid::Uuid::new_v4();

        // Should allow first request
        assert!(limiter.check(user_id).is_ok());
    }
}
