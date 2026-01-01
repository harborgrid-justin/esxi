//! Circuit Breaker Module
//!
//! Enterprise circuit breaker implementation for fault tolerance.

pub mod breaker;
pub mod health;

pub use breaker::{CircuitBreaker, CircuitBreakerState};
pub use health::{HealthChecker, HealthStatus};
