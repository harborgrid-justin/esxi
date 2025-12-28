//! Cron-based job scheduling with timezone support.

use crate::error::{WorkflowError, WorkflowResult};
use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// A unique identifier for a scheduled job.
pub type ScheduleId = Uuid;

/// Schedule configuration for a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// Unique schedule identifier.
    pub id: ScheduleId,

    /// Cron expression.
    pub cron_expr: String,

    /// Timezone for the schedule.
    pub timezone: String,

    /// Whether the schedule is enabled.
    pub enabled: bool,

    /// Optional start time (schedule won't run before this).
    pub start_time: Option<DateTime<Utc>>,

    /// Optional end time (schedule won't run after this).
    pub end_time: Option<DateTime<Utc>>,

    /// Maximum number of concurrent executions.
    pub max_concurrent: Option<usize>,

    /// Custom metadata.
    pub metadata: HashMap<String, String>,
}

impl ScheduleConfig {
    /// Creates a new schedule configuration.
    pub fn new(cron_expr: impl Into<String>, timezone: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            cron_expr: cron_expr.into(),
            timezone: timezone.into(),
            enabled: true,
            start_time: None,
            end_time: None,
            max_concurrent: Some(1),
            metadata: HashMap::new(),
        }
    }

    /// Validates the schedule configuration.
    pub fn validate(&self) -> WorkflowResult<()> {
        // Validate cron expression
        Schedule::from_str(&self.cron_expr).map_err(|e| {
            WorkflowError::InvalidCronExpression(format!("Invalid cron expression: {}", e))
        })?;

        // Validate timezone
        Tz::from_str(&self.timezone).map_err(|_| {
            WorkflowError::InvalidCronExpression(format!("Invalid timezone: {}", self.timezone))
        })?;

        // Validate time range
        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            if start >= end {
                return Err(WorkflowError::InvalidDefinition(
                    "Start time must be before end time".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Gets the next scheduled execution time.
    pub fn next_execution(&self) -> WorkflowResult<Option<DateTime<Utc>>> {
        if !self.enabled {
            return Ok(None);
        }

        let schedule = Schedule::from_str(&self.cron_expr)?;
        let tz: Tz = Tz::from_str(&self.timezone).map_err(|_| {
            WorkflowError::InvalidCronExpression(format!("Invalid timezone: {}", self.timezone))
        })?;

        let now = Utc::now();
        let now_in_tz = now.with_timezone(&tz);

        // Get next occurrence from schedule
        let next = schedule.upcoming(tz).next();

        if let Some(next_time) = next {
            let next_utc = next_time.with_timezone(&Utc);

            // Check against start time
            if let Some(start) = self.start_time {
                if next_utc < start {
                    return Ok(None);
                }
            }

            // Check against end time
            if let Some(end) = self.end_time {
                if next_utc > end {
                    return Ok(None);
                }
            }

            Ok(Some(next_utc))
        } else {
            Ok(None)
        }
    }
}

/// A scheduled job entry.
#[derive(Debug, Clone)]
struct ScheduledJob<T> {
    /// Schedule configuration.
    config: ScheduleConfig,

    /// Workflow or job data.
    data: T,

    /// Number of currently running executions.
    running_count: usize,

    /// Last execution time.
    last_execution: Option<DateTime<Utc>>,

    /// Next scheduled execution time.
    next_execution: Option<DateTime<Utc>>,
}

/// Callback type for scheduled job execution.
pub type ScheduleCallback<T> = Arc<dyn Fn(T) -> tokio::task::JoinHandle<WorkflowResult<()>> + Send + Sync>;

/// Cron-based job scheduler.
pub struct CronScheduler<T: Clone + Send + Sync + 'static> {
    /// Scheduled jobs.
    jobs: Arc<RwLock<HashMap<ScheduleId, ScheduledJob<T>>>>,

    /// Execution callback.
    callback: ScheduleCallback<T>,

    /// Scheduler task handle.
    task_handle: Option<JoinHandle<()>>,

    /// Tick interval in seconds.
    tick_interval: u64,
}

impl<T: Clone + Send + Sync + 'static> CronScheduler<T> {
    /// Creates a new cron scheduler.
    pub fn new(callback: ScheduleCallback<T>) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            callback,
            task_handle: None,
            tick_interval: 60, // Check every minute by default
        }
    }

    /// Sets the tick interval in seconds.
    pub fn with_tick_interval(mut self, interval_secs: u64) -> Self {
        self.tick_interval = interval_secs;
        self
    }

    /// Adds a scheduled job.
    pub async fn add_schedule(&self, config: ScheduleConfig, data: T) -> WorkflowResult<ScheduleId> {
        config.validate()?;

        let next_execution = config.next_execution()?;
        let schedule_id = config.id;

        let job = ScheduledJob {
            config,
            data,
            running_count: 0,
            last_execution: None,
            next_execution,
        };

        let mut jobs = self.jobs.write().await;
        jobs.insert(schedule_id, job);

        info!("Added schedule {} with next execution: {:?}", schedule_id, next_execution);

        Ok(schedule_id)
    }

    /// Removes a scheduled job.
    pub async fn remove_schedule(&self, schedule_id: ScheduleId) -> WorkflowResult<()> {
        let mut jobs = self.jobs.write().await;
        jobs.remove(&schedule_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Schedule {} not found", schedule_id)))?;

        info!("Removed schedule {}", schedule_id);
        Ok(())
    }

    /// Updates a schedule configuration.
    pub async fn update_schedule(&self, schedule_id: ScheduleId, config: ScheduleConfig) -> WorkflowResult<()> {
        config.validate()?;

        let mut jobs = self.jobs.write().await;
        let job = jobs
            .get_mut(&schedule_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Schedule {} not found", schedule_id)))?;

        let next_execution = config.next_execution()?;
        job.config = config;
        job.next_execution = next_execution;

        info!("Updated schedule {} with next execution: {:?}", schedule_id, next_execution);
        Ok(())
    }

    /// Enables a schedule.
    pub async fn enable_schedule(&self, schedule_id: ScheduleId) -> WorkflowResult<()> {
        let mut jobs = self.jobs.write().await;
        let job = jobs
            .get_mut(&schedule_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Schedule {} not found", schedule_id)))?;

        job.config.enabled = true;
        job.next_execution = job.config.next_execution()?;

        info!("Enabled schedule {}", schedule_id);
        Ok(())
    }

    /// Disables a schedule.
    pub async fn disable_schedule(&self, schedule_id: ScheduleId) -> WorkflowResult<()> {
        let mut jobs = self.jobs.write().await;
        let job = jobs
            .get_mut(&schedule_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Schedule {} not found", schedule_id)))?;

        job.config.enabled = false;
        job.next_execution = None;

        info!("Disabled schedule {}", schedule_id);
        Ok(())
    }

    /// Starts the scheduler.
    pub fn start(&mut self) {
        if self.task_handle.is_some() {
            warn!("Scheduler already running");
            return;
        }

        let jobs = Arc::clone(&self.jobs);
        let callback = Arc::clone(&self.callback);
        let tick_interval = self.tick_interval;

        let handle = tokio::spawn(async move {
            info!("Cron scheduler started");

            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(tick_interval));

            loop {
                interval.tick().await;

                let now = Utc::now();
                debug!("Scheduler tick at {}", now);

                // Get jobs that need to be executed
                let mut jobs_to_execute = Vec::new();
                {
                    let mut jobs_guard = jobs.write().await;
                    for (schedule_id, job) in jobs_guard.iter_mut() {
                        if !job.config.enabled {
                            continue;
                        }

                        if let Some(next_exec) = job.next_execution {
                            if next_exec <= now {
                                // Check concurrent execution limit
                                if let Some(max_concurrent) = job.config.max_concurrent {
                                    if job.running_count >= max_concurrent {
                                        warn!(
                                            "Schedule {} skipped: max concurrent executions ({}) reached",
                                            schedule_id, max_concurrent
                                        );
                                        continue;
                                    }
                                }

                                jobs_to_execute.push((*schedule_id, job.data.clone()));
                                job.running_count += 1;
                                job.last_execution = Some(now);

                                // Calculate next execution time
                                match job.config.next_execution() {
                                    Ok(next) => job.next_execution = next,
                                    Err(e) => {
                                        error!("Error calculating next execution for {}: {}", schedule_id, e);
                                        job.config.enabled = false;
                                        job.next_execution = None;
                                    }
                                }
                            }
                        }
                    }
                }

                // Execute jobs
                for (schedule_id, data) in jobs_to_execute {
                    info!("Executing scheduled job {}", schedule_id);
                    let jobs_clone = Arc::clone(&jobs);
                    let callback_clone = Arc::clone(&callback);

                    let handle = callback_clone(data);

                    tokio::spawn(async move {
                        match handle.await {
                            Ok(Ok(())) => {
                                info!("Scheduled job {} completed successfully", schedule_id);
                            }
                            Ok(Err(e)) => {
                                error!("Scheduled job {} failed: {}", schedule_id, e);
                            }
                            Err(e) => {
                                error!("Scheduled job {} panicked: {}", schedule_id, e);
                            }
                        }

                        // Decrement running count
                        let mut jobs_guard = jobs_clone.write().await;
                        if let Some(job) = jobs_guard.get_mut(&schedule_id) {
                            job.running_count = job.running_count.saturating_sub(1);
                        }
                    });
                }
            }
        });

        self.task_handle = Some(handle);
    }

    /// Stops the scheduler.
    pub async fn stop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
            info!("Cron scheduler stopped");
        }
    }

    /// Gets all schedules.
    pub async fn list_schedules(&self) -> Vec<(ScheduleId, ScheduleConfig)> {
        let jobs = self.jobs.read().await;
        jobs.iter()
            .map(|(id, job)| (*id, job.config.clone()))
            .collect()
    }

    /// Gets a specific schedule.
    pub async fn get_schedule(&self, schedule_id: ScheduleId) -> Option<ScheduleConfig> {
        let jobs = self.jobs.read().await;
        jobs.get(&schedule_id).map(|job| job.config.clone())
    }
}

impl<T: Clone + Send + Sync + 'static> Drop for CronScheduler<T> {
    fn drop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_config_validation() {
        let config = ScheduleConfig::new("0 0 * * *", "UTC");
        assert!(config.validate().is_ok());

        let invalid_config = ScheduleConfig::new("invalid", "UTC");
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_next_execution() {
        let config = ScheduleConfig::new("0 0 * * *", "UTC"); // Daily at midnight
        let next = config.next_execution().unwrap();
        assert!(next.is_some());
    }

    #[tokio::test]
    async fn test_scheduler_operations() {
        let callback: ScheduleCallback<String> = Arc::new(|_data| {
            tokio::spawn(async {
                Ok(())
            })
        });

        let scheduler = CronScheduler::new(callback);
        let config = ScheduleConfig::new("0 * * * *", "UTC"); // Every hour
        let schedule_id = scheduler.add_schedule(config, "test".to_string()).await.unwrap();

        let retrieved = scheduler.get_schedule(schedule_id).await;
        assert!(retrieved.is_some());

        scheduler.remove_schedule(schedule_id).await.unwrap();
        let removed = scheduler.get_schedule(schedule_id).await;
        assert!(removed.is_none());
    }
}
