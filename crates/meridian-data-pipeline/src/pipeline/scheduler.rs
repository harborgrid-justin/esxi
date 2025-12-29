//! Pipeline scheduler for running pipelines on a schedule.

use crate::error::{PipelineError, Result};
use crate::pipeline::Pipeline;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// Schedule configuration for pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    /// Run once at a specific time.
    Once { at: DateTime<Utc> },

    /// Run at a fixed interval.
    Interval { interval: Duration },

    /// Run using cron expression (simplified).
    Cron { expression: String },

    /// Run daily at a specific time.
    Daily { hour: u32, minute: u32 },

    /// Run weekly on specific days.
    Weekly {
        days: Vec<chrono::Weekday>,
        hour: u32,
        minute: u32,
    },

    /// Run monthly on a specific day.
    Monthly {
        day: u32,
        hour: u32,
        minute: u32,
    },
}

/// A scheduled pipeline job.
#[derive(Clone)]
pub struct ScheduledJob {
    /// Job ID.
    pub id: Uuid,
    /// Job name.
    pub name: String,
    /// Pipeline to execute.
    pub pipeline: Pipeline,
    /// Schedule configuration.
    pub schedule: Schedule,
    /// Whether the job is enabled.
    pub enabled: bool,
    /// Last execution time.
    pub last_run: Option<DateTime<Utc>>,
    /// Next scheduled execution time.
    pub next_run: Option<DateTime<Utc>>,
    /// Number of times the job has been executed.
    pub execution_count: u64,
}

impl ScheduledJob {
    /// Create a new scheduled job.
    pub fn new(name: impl Into<String>, pipeline: Pipeline, schedule: Schedule) -> Self {
        let next_run = calculate_next_run(&schedule, None);

        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            pipeline,
            schedule,
            enabled: true,
            last_run: None,
            next_run,
            execution_count: 0,
        }
    }

    /// Update the next run time based on the schedule.
    pub fn update_next_run(&mut self) {
        self.next_run = calculate_next_run(&self.schedule, self.last_run);
    }

    /// Check if the job should run now.
    pub fn should_run(&self) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(next_run) = self.next_run {
            Utc::now() >= next_run
        } else {
            false
        }
    }
}

/// Calculate the next run time based on schedule.
fn calculate_next_run(schedule: &Schedule, last_run: Option<DateTime<Utc>>) -> Option<DateTime<Utc>> {
    let now = Utc::now();

    match schedule {
        Schedule::Once { at } => {
            if *at > now {
                Some(*at)
            } else {
                None
            }
        }
        Schedule::Interval { interval } => {
            let base_time = last_run.unwrap_or(now);
            Some(base_time + *interval)
        }
        Schedule::Cron { expression: _ } => {
            // Simplified cron - in production, use a cron parsing library
            Some(now + Duration::hours(1))
        }
        Schedule::Daily { hour, minute } => {
            let mut next = now
                .date_naive()
                .and_hms_opt(*hour, *minute, 0)
                .unwrap()
                .and_utc();

            if next <= now {
                next = next + Duration::days(1);
            }

            Some(next)
        }
        Schedule::Weekly { days, hour, minute } => {
            // Find next occurrence of the specified day
            let mut next = now + Duration::days(1);
            for _ in 0..7 {
                if days.contains(&next.weekday()) {
                    next = next
                        .date_naive()
                        .and_hms_opt(*hour, *minute, 0)
                        .unwrap()
                        .and_utc();
                    if next > now {
                        return Some(next);
                    }
                }
                next = next + Duration::days(1);
            }
            None
        }
        Schedule::Monthly { day, hour, minute } => {
            let mut next = now
                .date_naive()
                .with_day(*day)
                .and_then(|d| d.and_hms_opt(*hour, *minute, 0))
                .unwrap()
                .and_utc();

            if next <= now {
                // Move to next month
                next = (next + Duration::days(32))
                    .date_naive()
                    .with_day(*day)
                    .and_then(|d| d.and_hms_opt(*hour, *minute, 0))
                    .unwrap()
                    .and_utc();
            }

            Some(next)
        }
    }
}

/// Pipeline scheduler for managing scheduled jobs.
pub struct PipelineScheduler {
    jobs: Arc<RwLock<HashMap<Uuid, ScheduledJob>>>,
    running: Arc<RwLock<bool>>,
}

impl PipelineScheduler {
    /// Create a new pipeline scheduler.
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Add a job to the scheduler.
    pub async fn add_job(&self, job: ScheduledJob) -> Result<Uuid> {
        let job_id = job.id;
        let mut jobs = self.jobs.write().await;

        tracing::info!(
            job_id = %job_id,
            job_name = %job.name,
            "Adding scheduled job"
        );

        jobs.insert(job_id, job);
        Ok(job_id)
    }

    /// Remove a job from the scheduler.
    pub async fn remove_job(&self, job_id: &Uuid) -> Result<()> {
        let mut jobs = self.jobs.write().await;

        if jobs.remove(job_id).is_some() {
            tracing::info!(job_id = %job_id, "Removed scheduled job");
            Ok(())
        } else {
            Err(PipelineError::NotFound(format!("Job {} not found", job_id)))
        }
    }

    /// Enable a job.
    pub async fn enable_job(&self, job_id: &Uuid) -> Result<()> {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(job_id) {
            job.enabled = true;
            job.update_next_run();
            tracing::info!(job_id = %job_id, "Enabled scheduled job");
            Ok(())
        } else {
            Err(PipelineError::NotFound(format!("Job {} not found", job_id)))
        }
    }

    /// Disable a job.
    pub async fn disable_job(&self, job_id: &Uuid) -> Result<()> {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.get_mut(job_id) {
            job.enabled = false;
            tracing::info!(job_id = %job_id, "Disabled scheduled job");
            Ok(())
        } else {
            Err(PipelineError::NotFound(format!("Job {} not found", job_id)))
        }
    }

    /// Get all jobs.
    pub async fn get_jobs(&self) -> Vec<ScheduledJob> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }

    /// Get a specific job.
    pub async fn get_job(&self, job_id: &Uuid) -> Option<ScheduledJob> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).cloned()
    }

    /// Start the scheduler.
    pub async fn start(&self) -> JoinHandle<()> {
        let jobs = self.jobs.clone();
        let running = self.running.clone();

        *running.write().await = true;

        tracing::info!("Starting pipeline scheduler");

        tokio::spawn(async move {
            while *running.read().await {
                // Check for jobs that should run
                let mut jobs_to_run = Vec::new();

                {
                    let mut jobs_lock = jobs.write().await;
                    for job in jobs_lock.values_mut() {
                        if job.should_run() {
                            jobs_to_run.push(job.clone());

                            // Update job metadata
                            job.last_run = Some(Utc::now());
                            job.execution_count += 1;
                            job.update_next_run();
                        }
                    }
                }

                // Execute jobs
                for job in jobs_to_run {
                    tracing::info!(
                        job_id = %job.id,
                        job_name = %job.name,
                        execution_count = job.execution_count,
                        "Executing scheduled job"
                    );

                    let pipeline = job.pipeline.clone();
                    tokio::spawn(async move {
                        if let Err(e) = pipeline.execute().await {
                            tracing::error!(
                                job_id = %job.id,
                                error = %e,
                                "Scheduled job execution failed"
                            );
                        }
                    });
                }

                // Sleep for a short interval before checking again
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }

            tracing::info!("Pipeline scheduler stopped");
        })
    }

    /// Stop the scheduler.
    pub async fn stop(&self) {
        *self.running.write().await = false;
        tracing::info!("Stopping pipeline scheduler");
    }

    /// Check if the scheduler is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

impl Default for PipelineScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::PipelineBuilder;

    #[tokio::test]
    async fn test_scheduler_add_remove_job() {
        let scheduler = PipelineScheduler::new();
        let pipeline = PipelineBuilder::new("test-pipeline")
            .version("1.0.0")
            .build()
            .unwrap();

        let job = ScheduledJob::new(
            "test-job",
            pipeline,
            Schedule::Interval {
                interval: Duration::minutes(5),
            },
        );

        let job_id = scheduler.add_job(job).await.unwrap();

        let jobs = scheduler.get_jobs().await;
        assert_eq!(jobs.len(), 1);

        scheduler.remove_job(&job_id).await.unwrap();

        let jobs = scheduler.get_jobs().await;
        assert_eq!(jobs.len(), 0);
    }

    #[tokio::test]
    async fn test_scheduler_enable_disable() {
        let scheduler = PipelineScheduler::new();
        let pipeline = PipelineBuilder::new("test-pipeline")
            .version("1.0.0")
            .build()
            .unwrap();

        let job = ScheduledJob::new(
            "test-job",
            pipeline,
            Schedule::Daily { hour: 9, minute: 0 },
        );

        let job_id = scheduler.add_job(job).await.unwrap();

        scheduler.disable_job(&job_id).await.unwrap();
        let job = scheduler.get_job(&job_id).await.unwrap();
        assert!(!job.enabled);

        scheduler.enable_job(&job_id).await.unwrap();
        let job = scheduler.get_job(&job_id).await.unwrap();
        assert!(job.enabled);
    }

    #[test]
    fn test_schedule_interval() {
        let schedule = Schedule::Interval {
            interval: Duration::hours(1),
        };

        let next_run = calculate_next_run(&schedule, None);
        assert!(next_run.is_some());
    }

    #[test]
    fn test_schedule_daily() {
        let schedule = Schedule::Daily {
            hour: 9,
            minute: 30,
        };

        let next_run = calculate_next_run(&schedule, None);
        assert!(next_run.is_some());

        if let Some(next) = next_run {
            assert_eq!(next.hour(), 9);
            assert_eq!(next.minute(), 30);
        }
    }

    #[test]
    fn test_scheduled_job_should_run() {
        let pipeline = PipelineBuilder::new("test-pipeline")
            .version("1.0.0")
            .build()
            .unwrap();

        let mut job = ScheduledJob::new(
            "test-job",
            pipeline,
            Schedule::Once {
                at: Utc::now() - Duration::hours(1),
            },
        );

        // Job with past time should not run (next_run will be None)
        assert!(!job.should_run());

        // Disabled job should not run
        job.enabled = false;
        assert!(!job.should_run());
    }
}
