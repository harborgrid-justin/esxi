//! Backup scheduling and automation.

use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{BackupError, Result};
use crate::incremental::BackupType;

/// Backup schedule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub id: Uuid,
    pub name: String,
    pub cron_expression: String,
    pub backup_type: BackupType,
    pub enabled: bool,
    pub retention_days: u32,
    pub tags: Vec<String>,
    pub notification_email: Option<String>,
}

/// Schedule execution record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleExecution {
    pub schedule_id: Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: ExecutionStatus,
    pub backup_id: Option<Uuid>,
    pub error_message: Option<String>,
}

/// Status of a schedule execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Backup scheduler.
pub struct BackupScheduler {
    schedules: Arc<RwLock<HashMap<Uuid, BackupSchedule>>>,
    executions: Arc<RwLock<Vec<ScheduleExecution>>>,
    running: Arc<RwLock<bool>>,
}

impl BackupScheduler {
    /// Create a new backup scheduler.
    pub fn new() -> Self {
        Self {
            schedules: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Add a backup schedule.
    pub async fn add_schedule(&self, schedule: BackupSchedule) -> Result<()> {
        // Validate cron expression
        Schedule::from_str(&schedule.cron_expression).map_err(|e| {
            BackupError::Scheduling(format!("Invalid cron expression: {}", e))
        })?;

        let mut schedules = self.schedules.write().await;
        schedules.insert(schedule.id, schedule);

        Ok(())
    }

    /// Remove a backup schedule.
    pub async fn remove_schedule(&self, schedule_id: Uuid) -> Result<()> {
        let mut schedules = self.schedules.write().await;
        schedules
            .remove(&schedule_id)
            .ok_or_else(|| BackupError::BackupNotFound(schedule_id.to_string()))?;

        Ok(())
    }

    /// Update a backup schedule.
    pub async fn update_schedule(&self, schedule: BackupSchedule) -> Result<()> {
        // Validate cron expression
        Schedule::from_str(&schedule.cron_expression).map_err(|e| {
            BackupError::Scheduling(format!("Invalid cron expression: {}", e))
        })?;

        let mut schedules = self.schedules.write().await;
        if !schedules.contains_key(&schedule.id) {
            return Err(BackupError::BackupNotFound(schedule.id.to_string()));
        }

        schedules.insert(schedule.id, schedule);

        Ok(())
    }

    /// Enable a schedule.
    pub async fn enable_schedule(&self, schedule_id: Uuid) -> Result<()> {
        let mut schedules = self.schedules.write().await;
        let schedule = schedules
            .get_mut(&schedule_id)
            .ok_or_else(|| BackupError::BackupNotFound(schedule_id.to_string()))?;

        schedule.enabled = true;

        Ok(())
    }

    /// Disable a schedule.
    pub async fn disable_schedule(&self, schedule_id: Uuid) -> Result<()> {
        let mut schedules = self.schedules.write().await;
        let schedule = schedules
            .get_mut(&schedule_id)
            .ok_or_else(|| BackupError::BackupNotFound(schedule_id.to_string()))?;

        schedule.enabled = false;

        Ok(())
    }

    /// Start the scheduler.
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        *running = true;

        let schedules = self.schedules.clone();
        let executions = self.executions.clone();
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            while *running_flag.read().await {
                let now = chrono::Utc::now();

                let schedules_snapshot = schedules.read().await.clone();

                for (schedule_id, schedule) in schedules_snapshot {
                    if !schedule.enabled {
                        continue;
                    }

                    // Parse cron schedule
                    if let Ok(cron_schedule) = Schedule::from_str(&schedule.cron_expression) {
                        // Check if schedule should run now
                        if Self::should_run(&cron_schedule, now) {
                            // Execute backup
                            let execution = ScheduleExecution {
                                schedule_id,
                                started_at: now,
                                completed_at: None,
                                status: ExecutionStatus::Running,
                                backup_id: None,
                                error_message: None,
                            };

                            let mut execs = executions.write().await;
                            execs.push(execution);
                            drop(execs);

                            // Spawn backup task
                            let executions_clone = executions.clone();
                            tokio::spawn(async move {
                                // Simulate backup execution
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                                let mut execs = executions_clone.write().await;
                                if let Some(exec) = execs.last_mut() {
                                    exec.completed_at = Some(chrono::Utc::now());
                                    exec.status = ExecutionStatus::Completed;
                                    exec.backup_id = Some(Uuid::new_v4());
                                }
                            });
                        }
                    }
                }

                // Sleep for 1 minute before checking again
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }

    /// Stop the scheduler.
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    /// Check if a schedule should run at a given time.
    fn should_run(schedule: &Schedule, now: chrono::DateTime<chrono::Utc>) -> bool {
        // Check if the current minute matches the schedule
        if let Some(next_run) = schedule.upcoming(chrono::Utc).next() {
            // Run if next scheduled time is within the next minute
            let diff = next_run - now;
            diff.num_seconds() < 60 && diff.num_seconds() >= 0
        } else {
            false
        }
    }

    /// Get next scheduled run time for a schedule.
    pub async fn get_next_run_time(
        &self,
        schedule_id: Uuid,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        let schedules = self.schedules.read().await;
        let schedule = schedules
            .get(&schedule_id)
            .ok_or_else(|| BackupError::BackupNotFound(schedule_id.to_string()))?;

        if !schedule.enabled {
            return Ok(None);
        }

        let cron_schedule = Schedule::from_str(&schedule.cron_expression)
            .map_err(|e| BackupError::Scheduling(format!("Invalid cron expression: {}", e)))?;

        Ok(cron_schedule.upcoming(chrono::Utc).next())
    }

    /// List all schedules.
    pub async fn list_schedules(&self) -> Vec<BackupSchedule> {
        let schedules = self.schedules.read().await;
        schedules.values().cloned().collect()
    }

    /// Get schedule by ID.
    pub async fn get_schedule(&self, schedule_id: Uuid) -> Result<BackupSchedule> {
        let schedules = self.schedules.read().await;
        schedules
            .get(&schedule_id)
            .cloned()
            .ok_or_else(|| BackupError::BackupNotFound(schedule_id.to_string()))
    }

    /// Get execution history for a schedule.
    pub async fn get_execution_history(
        &self,
        schedule_id: Uuid,
        limit: usize,
    ) -> Vec<ScheduleExecution> {
        let executions = self.executions.read().await;
        executions
            .iter()
            .filter(|e| e.schedule_id == schedule_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get recent executions.
    pub async fn get_recent_executions(&self, limit: usize) -> Vec<ScheduleExecution> {
        let executions = self.executions.read().await;
        executions.iter().rev().take(limit).cloned().collect()
    }

    /// Get scheduler statistics.
    pub async fn statistics(&self) -> SchedulerStatistics {
        let schedules = self.schedules.read().await;
        let executions = self.executions.read().await;

        let total_schedules = schedules.len();
        let enabled_schedules = schedules.values().filter(|s| s.enabled).count();

        let total_executions = executions.len();
        let successful_executions = executions
            .iter()
            .filter(|e| matches!(e.status, ExecutionStatus::Completed))
            .count();
        let failed_executions = executions
            .iter()
            .filter(|e| matches!(e.status, ExecutionStatus::Failed))
            .count();

        SchedulerStatistics {
            total_schedules,
            enabled_schedules,
            total_executions,
            successful_executions,
            failed_executions,
        }
    }
}

impl Default for BackupScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Scheduler statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatistics {
    pub total_schedules: usize,
    pub enabled_schedules: usize,
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
}

/// Pre-defined schedule templates.
pub struct ScheduleTemplates;

impl ScheduleTemplates {
    /// Hourly backup schedule.
    pub fn hourly() -> &'static str {
        "0 * * * *"
    }

    /// Daily backup at midnight.
    pub fn daily() -> &'static str {
        "0 0 * * *"
    }

    /// Weekly backup on Sunday at midnight.
    pub fn weekly() -> &'static str {
        "0 0 * * 0"
    }

    /// Monthly backup on the 1st at midnight.
    pub fn monthly() -> &'static str {
        "0 0 1 * *"
    }

    /// Every 6 hours.
    pub fn every_6_hours() -> &'static str {
        "0 */6 * * *"
    }

    /// Every 12 hours.
    pub fn every_12_hours() -> &'static str {
        "0 */12 * * *"
    }
}
