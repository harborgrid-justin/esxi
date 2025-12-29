use crate::events::{EventBus, MonitorEvent};
use crate::monitor::MonitorEngine;
use crate::types::*;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Scan scheduler for managing recurring scans
pub struct ScanScheduler {
    engine: Arc<MonitorEngine>,
    event_bus: EventBus,
    schedules: Arc<DashMap<Uuid, ScanSchedule>>,
    running: Arc<RwLock<bool>>,
}

impl ScanScheduler {
    /// Create a new scheduler
    pub fn new(engine: Arc<MonitorEngine>, event_bus: EventBus) -> Self {
        Self {
            engine,
            event_bus,
            schedules: Arc::new(DashMap::new()),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<(), SchedulerError> {
        let mut running = self.running.write().await;
        if *running {
            return Err(SchedulerError::AlreadyRunning);
        }

        *running = true;
        tracing::info!("Scan scheduler started");

        // Spawn background task to check schedules
        let schedules = self.schedules.clone();
        let engine = self.engine.clone();
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            while *running_flag.read().await {
                Self::check_schedules(&schedules, &engine).await;
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });

        Ok(())
    }

    /// Stop the scheduler
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        tracing::info!("Scan scheduler stopped");
    }

    /// Add a new schedule
    pub fn add_schedule(&self, schedule: ScanSchedule) -> Result<Uuid, SchedulerError> {
        // Validate cron expression
        cron::Schedule::from_str(&schedule.cron)
            .map_err(|e| SchedulerError::InvalidCron(e.to_string()))?;

        let id = schedule.id;
        self.schedules.insert(id, schedule);

        tracing::info!(schedule_id = %id, "Schedule added");
        Ok(id)
    }

    /// Remove a schedule
    pub fn remove_schedule(&self, id: Uuid) -> Option<ScanSchedule> {
        self.schedules.remove(&id).map(|(_, s)| s)
    }

    /// Get a schedule
    pub fn get_schedule(&self, id: Uuid) -> Option<ScanSchedule> {
        self.schedules.get(&id).map(|s| s.value().clone())
    }

    /// Get all schedules
    pub fn get_schedules(&self) -> Vec<ScanSchedule> {
        self.schedules
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Enable a schedule
    pub fn enable_schedule(&self, id: Uuid) -> Result<(), SchedulerError> {
        self.schedules
            .get_mut(&id)
            .map(|mut s| s.enabled = true)
            .ok_or(SchedulerError::ScheduleNotFound(id))
    }

    /// Disable a schedule
    pub fn disable_schedule(&self, id: Uuid) -> Result<(), SchedulerError> {
        self.schedules
            .get_mut(&id)
            .map(|mut s| s.enabled = false)
            .ok_or(SchedulerError::ScheduleNotFound(id))
    }

    /// Check all schedules and trigger scans if needed
    async fn check_schedules(schedules: &DashMap<Uuid, ScanSchedule>, engine: &MonitorEngine) {
        let now = Utc::now();

        for mut entry in schedules.iter_mut() {
            let schedule = entry.value_mut();

            if !schedule.enabled {
                continue;
            }

            // Parse cron expression
            let cron_schedule = match cron::Schedule::from_str(&schedule.cron) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(
                        schedule_id = %schedule.id,
                        error = ?e,
                        "Invalid cron expression"
                    );
                    continue;
                }
            };

            // Calculate next run time if not set
            if schedule.next_run.is_none() {
                schedule.next_run = cron_schedule.upcoming(Utc).next();
            }

            // Check if it's time to run
            if let Some(next_run) = schedule.next_run {
                if next_run <= now {
                    tracing::info!(
                        schedule_id = %schedule.id,
                        schedule_name = %schedule.name,
                        "Triggering scheduled scan"
                    );

                    // Start the scan
                    match engine.start_scan(schedule.config.clone()).await {
                        Ok(scan_id) => {
                            schedule.last_run = Some(now);
                            schedule.next_run = cron_schedule.upcoming(Utc).next();
                            tracing::info!(
                                schedule_id = %schedule.id,
                                scan_id = %scan_id,
                                next_run = ?schedule.next_run,
                                "Scheduled scan started"
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                schedule_id = %schedule.id,
                                error = ?e,
                                "Failed to start scheduled scan"
                            );
                        }
                    }
                }
            }
        }
    }

    /// Get the next scheduled run time for a schedule
    pub fn get_next_run(&self, id: Uuid) -> Option<DateTime<Utc>> {
        self.schedules.get(&id).and_then(|s| s.next_run)
    }

    /// Trigger a schedule immediately (ignore timing)
    pub async fn trigger_now(&self, id: Uuid) -> Result<Uuid, SchedulerError> {
        let schedule = self
            .schedules
            .get(&id)
            .ok_or(SchedulerError::ScheduleNotFound(id))?;

        let scan_id = self
            .engine
            .start_scan(schedule.config.clone())
            .await
            .map_err(|e| SchedulerError::ScanFailed(e.to_string()))?;

        tracing::info!(
            schedule_id = %id,
            scan_id = %scan_id,
            "Schedule triggered manually"
        );

        Ok(scan_id)
    }
}

/// Scheduler errors
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("Scheduler already running")]
    AlreadyRunning,

    #[error("Schedule not found: {0}")]
    ScheduleNotFound(Uuid),

    #[error("Invalid cron expression: {0}")]
    InvalidCron(String),

    #[error("Scan failed: {0}")]
    ScanFailed(String),
}

use std::str::FromStr;

impl FromStr for cron::Schedule {
    type Err = cron::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler() {
        let event_bus = EventBus::new(100);
        let engine = Arc::new(MonitorEngine::new(event_bus.clone()));
        let scheduler = ScanScheduler::new(engine, event_bus);

        let schedule = ScanSchedule {
            id: Uuid::new_v4(),
            name: "Daily Scan".to_string(),
            cron: "0 0 * * *".to_string(), // Daily at midnight
            config: ScanConfig::default(),
            enabled: true,
            next_run: None,
            last_run: None,
        };

        let id = scheduler.add_schedule(schedule).unwrap();
        assert!(scheduler.get_schedule(id).is_some());

        scheduler.remove_schedule(id);
        assert!(scheduler.get_schedule(id).is_none());
    }
}
