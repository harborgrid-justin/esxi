//! Health Checker
//!
//! Periodic health checking for upstream services.

use crate::config::HealthCheckConfig;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::sleep;
use tracing::{debug, error, info};

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is unhealthy
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

impl HealthStatus {
    /// Check if status is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    /// Get string representation of status
    pub fn as_str(&self) -> &str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Unhealthy => "unhealthy",
            HealthStatus::Unknown => "unknown",
        }
    }
}

/// Health check state
struct HealthState {
    status: HealthStatus,
    consecutive_successes: u32,
    consecutive_failures: u32,
    last_check: Option<Instant>,
    last_success: Option<Instant>,
}

impl HealthState {
    fn new() -> Self {
        Self {
            status: HealthStatus::Unknown,
            consecutive_successes: 0,
            consecutive_failures: 0,
            last_check: None,
            last_success: None,
        }
    }
}

/// Health Checker
///
/// Performs periodic health checks on upstream services.
pub struct HealthChecker {
    config: HealthCheckConfig,
    upstream_url: String,
    state: Arc<RwLock<HealthState>>,
    client: reqwest::Client,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(config: HealthCheckConfig, upstream_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config,
            upstream_url,
            state: Arc::new(RwLock::new(HealthState::new())),
            client,
        }
    }

    /// Start health checking in the background
    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                sleep(self.config.interval).await;
                self.check_health().await;
            }
        });
    }

    /// Perform a single health check
    async fn check_health(&self) {
        let url = format!("{}{}", self.upstream_url, self.config.path);
        debug!("Health check: {}", url);

        let result = tokio::time::timeout(
            self.config.timeout,
            self.client.get(&url).send(),
        )
        .await;

        let success = match result {
            Ok(Ok(response)) => response.status().is_success(),
            Ok(Err(e)) => {
                error!("Health check failed for {}: {}", url, e);
                false
            }
            Err(_) => {
                error!("Health check timeout for {}", url);
                false
            }
        };

        self.update_state(success);
    }

    /// Update health state based on check result
    fn update_state(&self, success: bool) {
        let mut state = self.state.write();
        state.last_check = Some(Instant::now());

        if success {
            state.consecutive_successes += 1;
            state.consecutive_failures = 0;
            state.last_success = Some(Instant::now());

            // Transition to healthy if threshold met
            if state.consecutive_successes >= self.config.healthy_threshold
                && state.status != HealthStatus::Healthy
            {
                info!(
                    "Upstream {} is now healthy ({})",
                    self.upstream_url, self.config.path
                );
                state.status = HealthStatus::Healthy;
            }
        } else {
            state.consecutive_failures += 1;
            state.consecutive_successes = 0;

            // Transition to unhealthy if threshold met
            if state.consecutive_failures >= self.config.unhealthy_threshold
                && state.status != HealthStatus::Unhealthy
            {
                error!(
                    "Upstream {} is now unhealthy ({})",
                    self.upstream_url, self.config.path
                );
                state.status = HealthStatus::Unhealthy;
            }
        }
    }

    /// Get current health status
    pub fn status(&self) -> HealthStatus {
        self.state.read().status
    }

    /// Check if upstream is healthy
    pub fn is_healthy(&self) -> bool {
        self.state.read().status.is_healthy()
    }

    /// Get health check statistics
    pub fn stats(&self) -> HealthCheckStats {
        let state = self.state.read();
        HealthCheckStats {
            status: state.status,
            consecutive_successes: state.consecutive_successes,
            consecutive_failures: state.consecutive_failures,
            last_check: state.last_check,
            last_success: state.last_success,
        }
    }

    /// Manually mark as healthy
    pub fn mark_healthy(&self) {
        let mut state = self.state.write();
        state.status = HealthStatus::Healthy;
        state.consecutive_successes = self.config.healthy_threshold;
        state.consecutive_failures = 0;
    }

    /// Manually mark as unhealthy
    pub fn mark_unhealthy(&self) {
        let mut state = self.state.write();
        state.status = HealthStatus::Unhealthy;
        state.consecutive_failures = self.config.unhealthy_threshold;
        state.consecutive_successes = 0;
    }
}

/// Health check statistics
#[derive(Debug, Clone)]
pub struct HealthCheckStats {
    /// Current health status
    pub status: HealthStatus,
    /// Number of consecutive successful checks
    pub consecutive_successes: u32,
    /// Number of consecutive failed checks
    pub consecutive_failures: u32,
    /// Time of last health check
    pub last_check: Option<Instant>,
    /// Time of last successful check
    pub last_success: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> HealthCheckConfig {
        HealthCheckConfig {
            path: "/health".to_string(),
            interval: Duration::from_secs(5),
            timeout: Duration::from_secs(1),
            healthy_threshold: 2,
            unhealthy_threshold: 3,
        }
    }

    #[test]
    fn test_health_checker_initial_state() {
        let checker = HealthChecker::new(
            test_config(),
            "http://localhost:8080".to_string(),
        );
        assert_eq!(checker.status(), HealthStatus::Unknown);
    }

    #[test]
    fn test_manual_state_changes() {
        let checker = HealthChecker::new(
            test_config(),
            "http://localhost:8080".to_string(),
        );

        checker.mark_healthy();
        assert_eq!(checker.status(), HealthStatus::Healthy);
        assert!(checker.is_healthy());

        checker.mark_unhealthy();
        assert_eq!(checker.status(), HealthStatus::Unhealthy);
        assert!(!checker.is_healthy());
    }
}
