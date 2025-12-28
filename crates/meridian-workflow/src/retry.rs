//! Retry policies with exponential backoff and jitter.

use crate::error::{WorkflowError, WorkflowResult};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

/// Retry strategy for failed tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// No retries.
    None,

    /// Fixed delay between retries.
    Fixed {
        /// Delay in milliseconds.
        delay_ms: u64,
        /// Maximum number of retries.
        max_retries: u32,
    },

    /// Exponential backoff with optional jitter.
    Exponential {
        /// Initial delay in milliseconds.
        initial_delay_ms: u64,
        /// Multiplier for each retry (typically 2.0 for doubling).
        multiplier: f64,
        /// Maximum delay in milliseconds.
        max_delay_ms: u64,
        /// Maximum number of retries.
        max_retries: u32,
        /// Whether to add jitter to prevent thundering herd.
        jitter: bool,
    },

    /// Linear backoff.
    Linear {
        /// Initial delay in milliseconds.
        initial_delay_ms: u64,
        /// Increment per retry in milliseconds.
        increment_ms: u64,
        /// Maximum delay in milliseconds.
        max_delay_ms: u64,
        /// Maximum number of retries.
        max_retries: u32,
    },

    /// Custom retry delays.
    Custom {
        /// List of delays in milliseconds for each retry attempt.
        delays_ms: Vec<u64>,
    },
}

impl Default for RetryStrategy {
    fn default() -> Self {
        RetryStrategy::Exponential {
            initial_delay_ms: 1000,
            multiplier: 2.0,
            max_delay_ms: 60000,
            max_retries: 3,
            jitter: true,
        }
    }
}

impl RetryStrategy {
    /// Creates a retry strategy with no retries.
    pub fn none() -> Self {
        RetryStrategy::None
    }

    /// Creates a fixed delay retry strategy.
    pub fn fixed(delay_ms: u64, max_retries: u32) -> Self {
        RetryStrategy::Fixed {
            delay_ms,
            max_retries,
        }
    }

    /// Creates an exponential backoff retry strategy.
    pub fn exponential(
        initial_delay_ms: u64,
        multiplier: f64,
        max_delay_ms: u64,
        max_retries: u32,
    ) -> Self {
        RetryStrategy::Exponential {
            initial_delay_ms,
            multiplier,
            max_delay_ms,
            max_retries,
            jitter: true,
        }
    }

    /// Creates a linear backoff retry strategy.
    pub fn linear(initial_delay_ms: u64, increment_ms: u64, max_delay_ms: u64, max_retries: u32) -> Self {
        RetryStrategy::Linear {
            initial_delay_ms,
            increment_ms,
            max_delay_ms,
            max_retries,
        }
    }

    /// Creates a custom retry strategy with specific delays.
    pub fn custom(delays_ms: Vec<u64>) -> Self {
        RetryStrategy::Custom { delays_ms }
    }

    /// Gets the maximum number of retries.
    pub fn max_retries(&self) -> u32 {
        match self {
            RetryStrategy::None => 0,
            RetryStrategy::Fixed { max_retries, .. } => *max_retries,
            RetryStrategy::Exponential { max_retries, .. } => *max_retries,
            RetryStrategy::Linear { max_retries, .. } => *max_retries,
            RetryStrategy::Custom { delays_ms } => delays_ms.len() as u32,
        }
    }

    /// Calculates the delay for a specific retry attempt.
    pub fn calculate_delay(&self, attempt: u32) -> Option<StdDuration> {
        if attempt == 0 {
            return None;
        }

        let delay_ms = match self {
            RetryStrategy::None => return None,

            RetryStrategy::Fixed {
                delay_ms,
                max_retries,
            } => {
                if attempt > *max_retries {
                    return None;
                }
                *delay_ms
            }

            RetryStrategy::Exponential {
                initial_delay_ms,
                multiplier,
                max_delay_ms,
                max_retries,
                jitter,
            } => {
                if attempt > *max_retries {
                    return None;
                }

                let exponential_delay =
                    (*initial_delay_ms as f64) * multiplier.powi((attempt - 1) as i32);
                let mut delay = exponential_delay.min(*max_delay_ms as f64) as u64;

                // Add jitter (random variation up to 50% of the delay)
                if *jitter {
                    let jitter_range = (delay as f64 * 0.5) as u64;
                    let random_jitter = rand::random::<u64>() % (jitter_range + 1);
                    delay = delay.saturating_add(random_jitter);
                }

                delay.min(*max_delay_ms)
            }

            RetryStrategy::Linear {
                initial_delay_ms,
                increment_ms,
                max_delay_ms,
                max_retries,
            } => {
                if attempt > *max_retries {
                    return None;
                }

                let linear_delay = initial_delay_ms + (increment_ms * (attempt - 1) as u64);
                linear_delay.min(*max_delay_ms)
            }

            RetryStrategy::Custom { delays_ms } => {
                let index = (attempt - 1) as usize;
                if index >= delays_ms.len() {
                    return None;
                }
                delays_ms[index]
            }
        };

        Some(StdDuration::from_millis(delay_ms))
    }

    /// Checks if a retry should be attempted.
    pub fn should_retry(&self, attempt: u32, error: &WorkflowError) -> bool {
        // Don't retry fatal errors
        if error.is_fatal() {
            return false;
        }

        // Check if we have retries left
        self.calculate_delay(attempt).is_some()
    }
}

/// Retry policy for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Retry strategy.
    pub strategy: RetryStrategy,

    /// Whether to retry on all errors or only retryable ones.
    pub retry_all_errors: bool,

    /// Optional timeout for the entire retry sequence.
    pub total_timeout_ms: Option<u64>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            strategy: RetryStrategy::default(),
            retry_all_errors: false,
            total_timeout_ms: Some(300000), // 5 minutes
        }
    }
}

impl RetryPolicy {
    /// Creates a new retry policy with the given strategy.
    pub fn new(strategy: RetryStrategy) -> Self {
        Self {
            strategy,
            retry_all_errors: false,
            total_timeout_ms: Some(300000),
        }
    }

    /// Sets whether to retry all errors.
    pub fn with_retry_all_errors(mut self, retry_all: bool) -> Self {
        self.retry_all_errors = retry_all;
        self
    }

    /// Sets the total timeout for retries.
    pub fn with_total_timeout(mut self, timeout_ms: u64) -> Self {
        self.total_timeout_ms = Some(timeout_ms);
        self
    }

    /// Checks if a retry should be attempted.
    pub fn should_retry(&self, attempt: u32, error: &WorkflowError, elapsed_ms: u64) -> bool {
        // Check total timeout
        if let Some(total_timeout) = self.total_timeout_ms {
            if elapsed_ms >= total_timeout {
                return false;
            }
        }

        // Check if error is retryable
        if !self.retry_all_errors && !error.is_retryable() {
            return false;
        }

        // Check retry strategy
        self.strategy.should_retry(attempt, error)
    }

    /// Calculates the delay for the next retry.
    pub fn calculate_delay(&self, attempt: u32) -> Option<StdDuration> {
        self.strategy.calculate_delay(attempt)
    }
}

/// Retry state tracker for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryState {
    /// Current attempt number (0 = first attempt, 1 = first retry, etc.).
    pub attempt: u32,

    /// Total elapsed time in milliseconds.
    pub elapsed_ms: u64,

    /// Last error message.
    pub last_error: Option<String>,

    /// Timestamp of the last attempt.
    pub last_attempt_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Next retry scheduled time.
    pub next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for RetryState {
    fn default() -> Self {
        Self {
            attempt: 0,
            elapsed_ms: 0,
            last_error: None,
            last_attempt_at: None,
            next_retry_at: None,
        }
    }
}

impl RetryState {
    /// Creates a new retry state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a failed attempt.
    pub fn record_failure(&mut self, error: String, policy: &RetryPolicy) -> Option<StdDuration> {
        self.attempt += 1;
        self.last_error = Some(error);
        self.last_attempt_at = Some(Utc::now());

        // Calculate delay for next retry
        if let Some(delay) = policy.calculate_delay(self.attempt) {
            let next_retry = Utc::now() + Duration::milliseconds(delay.as_millis() as i64);
            self.next_retry_at = Some(next_retry);
            Some(delay)
        } else {
            self.next_retry_at = None;
            None
        }
    }

    /// Updates the elapsed time.
    pub fn update_elapsed(&mut self, start_time: chrono::DateTime<chrono::Utc>) {
        let elapsed = Utc::now().signed_duration_since(start_time);
        self.elapsed_ms = elapsed.num_milliseconds().max(0) as u64;
    }

    /// Checks if ready for retry.
    pub fn is_ready_for_retry(&self) -> bool {
        if let Some(next_retry) = self.next_retry_at {
            Utc::now() >= next_retry
        } else {
            false
        }
    }

    /// Checks if retries are exhausted.
    pub fn is_exhausted(&self, policy: &RetryPolicy) -> bool {
        self.attempt >= policy.strategy.max_retries()
            || self.next_retry_at.is_none()
            || policy
                .total_timeout_ms
                .map(|timeout| self.elapsed_ms >= timeout)
                .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_retry_strategy() {
        let strategy = RetryStrategy::fixed(1000, 3);

        assert_eq!(strategy.calculate_delay(0), None);
        assert_eq!(strategy.calculate_delay(1), Some(StdDuration::from_millis(1000)));
        assert_eq!(strategy.calculate_delay(2), Some(StdDuration::from_millis(1000)));
        assert_eq!(strategy.calculate_delay(3), Some(StdDuration::from_millis(1000)));
        assert_eq!(strategy.calculate_delay(4), None);
    }

    #[test]
    fn test_exponential_retry_strategy() {
        let strategy = RetryStrategy::Exponential {
            initial_delay_ms: 100,
            multiplier: 2.0,
            max_delay_ms: 1000,
            max_retries: 5,
            jitter: false,
        };

        assert_eq!(strategy.calculate_delay(1), Some(StdDuration::from_millis(100)));
        assert_eq!(strategy.calculate_delay(2), Some(StdDuration::from_millis(200)));
        assert_eq!(strategy.calculate_delay(3), Some(StdDuration::from_millis(400)));
        assert_eq!(strategy.calculate_delay(4), Some(StdDuration::from_millis(800)));
        assert_eq!(strategy.calculate_delay(5), Some(StdDuration::from_millis(1000))); // capped
    }

    #[test]
    fn test_linear_retry_strategy() {
        let strategy = RetryStrategy::linear(100, 50, 500, 5);

        assert_eq!(strategy.calculate_delay(1), Some(StdDuration::from_millis(100)));
        assert_eq!(strategy.calculate_delay(2), Some(StdDuration::from_millis(150)));
        assert_eq!(strategy.calculate_delay(3), Some(StdDuration::from_millis(200)));
        assert_eq!(strategy.calculate_delay(4), Some(StdDuration::from_millis(250)));
        assert_eq!(strategy.calculate_delay(5), Some(StdDuration::from_millis(300)));
    }

    #[test]
    fn test_custom_retry_strategy() {
        let strategy = RetryStrategy::custom(vec![100, 500, 1000]);

        assert_eq!(strategy.calculate_delay(1), Some(StdDuration::from_millis(100)));
        assert_eq!(strategy.calculate_delay(2), Some(StdDuration::from_millis(500)));
        assert_eq!(strategy.calculate_delay(3), Some(StdDuration::from_millis(1000)));
        assert_eq!(strategy.calculate_delay(4), None);
    }

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::new(RetryStrategy::fixed(1000, 3));

        let error = WorkflowError::TaskExecutionFailed {
            task_id: "test".to_string(),
            reason: "test error".to_string(),
        };

        assert!(policy.should_retry(1, &error, 0));
        assert!(policy.should_retry(2, &error, 1000));
        assert!(policy.should_retry(3, &error, 2000));
        assert!(!policy.should_retry(4, &error, 3000));
    }

    #[test]
    fn test_retry_state() {
        let mut state = RetryState::new();
        let policy = RetryPolicy::new(RetryStrategy::fixed(1000, 3));

        assert_eq!(state.attempt, 0);

        let delay = state.record_failure("error 1".to_string(), &policy);
        assert!(delay.is_some());
        assert_eq!(state.attempt, 1);

        state.record_failure("error 2".to_string(), &policy);
        assert_eq!(state.attempt, 2);

        state.record_failure("error 3".to_string(), &policy);
        assert_eq!(state.attempt, 3);

        let no_delay = state.record_failure("error 4".to_string(), &policy);
        assert!(no_delay.is_none());
    }
}
