//! Circuit Breaker Implementation
//!
//! Enterprise-grade circuit breaker with half-open state and automatic recovery.

use crate::config::CircuitBreakerConfig;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Circuit breaker errors
#[derive(Debug, Error)]
pub enum CircuitBreakerError {
    /// Circuit is open, requests are rejected
    #[error("Circuit breaker is open")]
    Open,
    /// Too many requests in half-open state
    #[error("Too many requests in half-open state")]
    HalfOpenLimitExceeded,
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

impl CircuitBreakerState {
    /// Get string representation of state
    pub fn as_str(&self) -> &str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
            Self::HalfOpen => "half_open",
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
struct Statistics {
    total_requests: u32,
    failed_requests: u32,
    window_start: Instant,
}

impl Statistics {
    fn new() -> Self {
        Self {
            total_requests: 0,
            failed_requests: 0,
            window_start: Instant::now(),
        }
    }

    fn reset(&mut self) {
        self.total_requests = 0;
        self.failed_requests = 0;
        self.window_start = Instant::now();
    }

    fn record_success(&mut self) {
        self.total_requests += 1;
    }

    fn record_failure(&mut self) {
        self.total_requests += 1;
        self.failed_requests += 1;
    }

    fn failure_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.failed_requests as f64 / self.total_requests as f64) * 100.0
    }

    fn should_reset(&self, window: Duration) -> bool {
        self.window_start.elapsed() >= window
    }
}

/// Internal state
struct InternalState {
    state: CircuitBreakerState,
    stats: Statistics,
    opened_at: Option<Instant>,
    half_open_requests: u32,
}

/// Circuit Breaker
///
/// Implements the circuit breaker pattern for fault tolerance.
/// Transitions between Closed -> Open -> HalfOpen -> Closed states.
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<InternalState>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(InternalState {
                state: CircuitBreakerState::Closed,
                stats: Statistics::new(),
                opened_at: None,
                half_open_requests: 0,
            })),
        }
    }

    /// Check if request can proceed
    pub fn allow_request(&self) -> Result<(), CircuitBreakerError> {
        let mut state = self.state.write();

        match state.state {
            CircuitBreakerState::Closed => {
                // Check if we need to reset the statistics window
                if state.stats.should_reset(self.config.window) {
                    state.stats.reset();
                }
                Ok(())
            }
            CircuitBreakerState::Open => {
                // Check if recovery timeout has elapsed
                if let Some(opened_at) = state.opened_at {
                    if opened_at.elapsed() >= self.config.recovery_timeout {
                        // Transition to half-open
                        state.state = CircuitBreakerState::HalfOpen;
                        state.half_open_requests = 0;
                        state.stats.reset();
                        Ok(())
                    } else {
                        Err(CircuitBreakerError::Open)
                    }
                } else {
                    Err(CircuitBreakerError::Open)
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Allow limited requests in half-open state
                if state.half_open_requests < self.config.half_open_max_requests {
                    state.half_open_requests += 1;
                    Ok(())
                } else {
                    Err(CircuitBreakerError::HalfOpenLimitExceeded)
                }
            }
        }
    }

    /// Record a successful request
    pub fn record_success(&self) {
        let mut state = self.state.write();

        match state.state {
            CircuitBreakerState::Closed => {
                state.stats.record_success();
            }
            CircuitBreakerState::HalfOpen => {
                state.stats.record_success();
                // If all half-open requests succeed, transition to closed
                if state.half_open_requests >= self.config.half_open_max_requests {
                    state.state = CircuitBreakerState::Closed;
                    state.stats.reset();
                }
            }
            CircuitBreakerState::Open => {
                // Should not happen, but handle gracefully
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&self) {
        let mut state = self.state.write();

        match state.state {
            CircuitBreakerState::Closed => {
                state.stats.record_failure();
                // Check if we should trip the breaker
                if self.should_trip(&state.stats) {
                    state.state = CircuitBreakerState::Open;
                    state.opened_at = Some(Instant::now());
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Any failure in half-open state trips back to open
                state.state = CircuitBreakerState::Open;
                state.opened_at = Some(Instant::now());
                state.stats.reset();
            }
            CircuitBreakerState::Open => {
                // Already open, no action needed
            }
        }
    }

    /// Get current state
    pub fn state(&self) -> CircuitBreakerState {
        self.state.read().state
    }

    /// Get current statistics
    pub fn stats(&self) -> (u32, u32, f64) {
        let state = self.state.read();
        (
            state.stats.total_requests,
            state.stats.failed_requests,
            state.stats.failure_rate(),
        )
    }

    /// Manually reset the circuit breaker
    pub fn reset(&self) {
        let mut state = self.state.write();
        state.state = CircuitBreakerState::Closed;
        state.stats.reset();
        state.opened_at = None;
        state.half_open_requests = 0;
    }

    /// Check if circuit should trip
    fn should_trip(&self, stats: &Statistics) -> bool {
        // Need minimum number of requests
        if stats.total_requests < self.config.min_requests {
            return false;
        }

        // Check failure threshold
        stats.failure_rate() >= self.config.failure_threshold
    }
}

impl Clone for CircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: Arc::clone(&self.state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            enabled: true,
            failure_threshold: 50.0,
            min_requests: 5,
            window: Duration::from_secs(60),
            recovery_timeout: Duration::from_millis(100),
            half_open_max_requests: 3,
        }
    }

    #[test]
    fn test_circuit_breaker_closed() {
        let breaker = CircuitBreaker::new(test_config());
        assert_eq!(breaker.state(), CircuitBreakerState::Closed);
        assert!(breaker.allow_request().is_ok());
    }

    #[test]
    fn test_circuit_breaker_trips() {
        let breaker = CircuitBreaker::new(test_config());

        // Record failures to trip the breaker
        for _ in 0..3 {
            breaker.allow_request().unwrap();
            breaker.record_failure();
        }
        for _ in 0..2 {
            breaker.allow_request().unwrap();
            breaker.record_success();
        }

        // Should still be closed (50% failure rate with 5 requests)
        assert_eq!(breaker.state(), CircuitBreakerState::Closed);

        // One more failure should trip it (60% failure rate)
        breaker.allow_request().unwrap();
        breaker.record_failure();

        assert_eq!(breaker.state(), CircuitBreakerState::Open);
        assert!(breaker.allow_request().is_err());
    }

    #[test]
    fn test_circuit_breaker_half_open() {
        let breaker = CircuitBreaker::new(test_config());

        // Trip the breaker
        for _ in 0..10 {
            breaker.allow_request().unwrap();
            breaker.record_failure();
        }
        assert_eq!(breaker.state(), CircuitBreakerState::Open);

        // Wait for recovery timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should transition to half-open
        assert!(breaker.allow_request().is_ok());
        assert_eq!(breaker.state(), CircuitBreakerState::HalfOpen);
    }

    #[test]
    fn test_circuit_breaker_recovery() {
        let breaker = CircuitBreaker::new(test_config());

        // Trip the breaker
        for _ in 0..10 {
            breaker.allow_request().unwrap();
            breaker.record_failure();
        }

        // Wait for recovery timeout
        std::thread::sleep(Duration::from_millis(150));

        // Succeed in half-open state
        for _ in 0..3 {
            breaker.allow_request().unwrap();
            breaker.record_success();
        }

        // Should transition back to closed
        assert_eq!(breaker.state(), CircuitBreakerState::Closed);
    }
}
