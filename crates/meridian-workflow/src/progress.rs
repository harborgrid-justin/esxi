//! Progress tracking and ETA estimation for workflows.

use crate::dag::{TaskId, WorkflowId};
use crate::state::{TaskState, WorkflowExecution};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Progress information for a workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    /// Workflow ID.
    pub workflow_id: WorkflowId,

    /// Execution ID.
    pub execution_id: String,

    /// Overall progress percentage (0-100).
    pub percentage: f64,

    /// Number of completed tasks.
    pub completed_tasks: usize,

    /// Total number of tasks.
    pub total_tasks: usize,

    /// Number of running tasks.
    pub running_tasks: usize,

    /// Number of pending tasks.
    pub pending_tasks: usize,

    /// Number of failed tasks.
    pub failed_tasks: usize,

    /// Estimated time of completion.
    pub eta: Option<DateTime<Utc>>,

    /// Elapsed time in milliseconds.
    pub elapsed_ms: u64,

    /// Average task duration in milliseconds.
    pub avg_task_duration_ms: Option<u64>,

    /// Current phase or milestone.
    pub current_phase: Option<String>,

    /// Detailed task progress.
    pub task_progress: HashMap<TaskId, TaskProgress>,
}

impl WorkflowProgress {
    /// Creates progress from a workflow execution.
    pub fn from_execution(execution: &WorkflowExecution) -> Self {
        let total_tasks = execution.tasks.len();
        let completed_tasks = execution
            .tasks
            .values()
            .filter(|t| t.state == TaskState::Completed)
            .count();

        let running_tasks = execution
            .tasks
            .values()
            .filter(|t| t.state == TaskState::Running)
            .count();

        let pending_tasks = execution
            .tasks
            .values()
            .filter(|t| t.state == TaskState::Pending)
            .count();

        let failed_tasks = execution
            .tasks
            .values()
            .filter(|t| t.state == TaskState::Failed)
            .count();

        let percentage = if total_tasks > 0 {
            (completed_tasks as f64 / total_tasks as f64) * 100.0
        } else {
            0.0
        };

        // Calculate elapsed time
        let elapsed_ms = if let Some(started_at) = execution.started_at {
            let now = Utc::now();
            now.signed_duration_since(started_at)
                .num_milliseconds()
                .max(0) as u64
        } else {
            0
        };

        // Calculate average task duration
        let completed_durations: Vec<u64> = execution
            .tasks
            .values()
            .filter(|t| t.state == TaskState::Completed)
            .filter_map(|t| t.duration_ms())
            .collect();

        let avg_task_duration_ms = if !completed_durations.is_empty() {
            let sum: u64 = completed_durations.iter().sum();
            Some(sum / completed_durations.len() as u64)
        } else {
            None
        };

        // Estimate ETA
        let eta = Self::calculate_eta(
            completed_tasks,
            total_tasks,
            avg_task_duration_ms,
            execution.started_at,
        );

        // Build task progress
        let task_progress = execution
            .tasks
            .iter()
            .map(|(id, task_exec)| (*id, TaskProgress::from_task_execution(task_exec)))
            .collect();

        Self {
            workflow_id: execution.workflow_id,
            execution_id: execution.execution_id.clone(),
            percentage,
            completed_tasks,
            total_tasks,
            running_tasks,
            pending_tasks,
            failed_tasks,
            eta,
            elapsed_ms,
            avg_task_duration_ms,
            current_phase: None,
            task_progress,
        }
    }

    /// Calculates estimated time of completion.
    fn calculate_eta(
        completed: usize,
        total: usize,
        avg_duration: Option<u64>,
        started_at: Option<DateTime<Utc>>,
    ) -> Option<DateTime<Utc>> {
        if completed >= total || avg_duration.is_none() || started_at.is_none() {
            return None;
        }

        let avg_ms = avg_duration?;
        let remaining_tasks = total - completed;
        let estimated_remaining_ms = (remaining_tasks as u64) * avg_ms;

        Some(Utc::now() + Duration::milliseconds(estimated_remaining_ms as i64))
    }

    /// Checks if the workflow is on track based on expected duration.
    pub fn is_on_track(&self, expected_duration_ms: u64) -> bool {
        if self.percentage == 0.0 {
            return true;
        }

        let expected_elapsed = (expected_duration_ms as f64 * self.percentage / 100.0) as u64;
        self.elapsed_ms <= expected_elapsed * 120 / 100 // Allow 20% buffer
    }

    /// Gets the time remaining until completion.
    pub fn time_remaining(&self) -> Option<Duration> {
        self.eta.map(|eta| {
            let now = Utc::now();
            eta.signed_duration_since(now).max(Duration::zero())
        })
    }

    /// Gets a human-readable status string.
    pub fn status_string(&self) -> String {
        if self.failed_tasks > 0 {
            format!(
                "{:.1}% complete ({}/{} tasks, {} failed)",
                self.percentage, self.completed_tasks, self.total_tasks, self.failed_tasks
            )
        } else {
            format!(
                "{:.1}% complete ({}/{} tasks)",
                self.percentage, self.completed_tasks, self.total_tasks
            )
        }
    }

    /// Gets detailed status by phase.
    pub fn phase_summary(&self) -> HashMap<TaskState, usize> {
        let mut summary = HashMap::new();
        summary.insert(TaskState::Completed, self.completed_tasks);
        summary.insert(TaskState::Running, self.running_tasks);
        summary.insert(TaskState::Pending, self.pending_tasks);
        summary.insert(TaskState::Failed, self.failed_tasks);
        summary
    }
}

/// Progress information for a single task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    /// Task ID.
    pub task_id: TaskId,

    /// Current state.
    pub state: TaskState,

    /// Progress percentage (0-100) if available.
    pub percentage: Option<f64>,

    /// Start time.
    pub started_at: Option<DateTime<Utc>>,

    /// Completion time.
    pub completed_at: Option<DateTime<Utc>>,

    /// Duration in milliseconds.
    pub duration_ms: Option<u64>,

    /// Number of attempts.
    pub attempt_count: u32,

    /// Custom progress message.
    pub message: Option<String>,

    /// Metrics.
    pub metrics: HashMap<String, f64>,
}

impl TaskProgress {
    /// Creates progress from a task execution.
    pub fn from_task_execution(task_exec: &crate::state::TaskExecution) -> Self {
        let percentage = match task_exec.state {
            TaskState::Completed => Some(100.0),
            TaskState::Running => Some(50.0), // Default estimate
            TaskState::Failed | TaskState::Skipped => Some(0.0),
            TaskState::Pending | TaskState::Retrying => Some(0.0),
        };

        let metrics = task_exec
            .result
            .as_ref()
            .map(|r| r.metrics.clone())
            .unwrap_or_default();

        Self {
            task_id: task_exec.task_id,
            state: task_exec.state.clone(),
            percentage,
            started_at: task_exec.started_at,
            completed_at: task_exec.completed_at,
            duration_ms: task_exec.duration_ms(),
            attempt_count: task_exec.attempt_count,
            message: None,
            metrics,
        }
    }

    /// Updates the task progress percentage.
    pub fn update_percentage(&mut self, percentage: f64) {
        self.percentage = Some(percentage.clamp(0.0, 100.0));
    }

    /// Sets a progress message.
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = Some(message.into());
    }

    /// Adds a metric.
    pub fn add_metric(&mut self, name: impl Into<String>, value: f64) {
        self.metrics.insert(name.into(), value);
    }
}

/// Progress tracker for monitoring workflow execution.
pub struct ProgressTracker {
    /// Historical progress snapshots.
    snapshots: Vec<ProgressSnapshot>,

    /// Snapshot interval in seconds.
    snapshot_interval_secs: u64,
}

impl ProgressTracker {
    /// Creates a new progress tracker.
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            snapshot_interval_secs: 60, // 1 minute
        }
    }

    /// Sets the snapshot interval.
    pub fn with_snapshot_interval(mut self, interval_secs: u64) -> Self {
        self.snapshot_interval_secs = interval_secs;
        self
    }

    /// Takes a progress snapshot.
    pub fn take_snapshot(&mut self, progress: WorkflowProgress) {
        let snapshot = ProgressSnapshot {
            timestamp: Utc::now(),
            progress,
        };

        self.snapshots.push(snapshot);

        // Limit snapshots to prevent unbounded growth
        if self.snapshots.len() > 1000 {
            self.snapshots.remove(0);
        }
    }

    /// Gets the progress rate (percentage per minute).
    pub fn progress_rate(&self) -> Option<f64> {
        if self.snapshots.len() < 2 {
            return None;
        }

        let first = self.snapshots.first()?;
        let last = self.snapshots.last()?;

        let duration_secs = last
            .timestamp
            .signed_duration_since(first.timestamp)
            .num_seconds() as f64;

        if duration_secs == 0.0 {
            return None;
        }

        let progress_delta = last.progress.percentage - first.progress.percentage;
        let rate = (progress_delta / duration_secs) * 60.0; // Per minute

        Some(rate)
    }

    /// Predicts completion time based on historical rate.
    pub fn predict_completion(&self) -> Option<DateTime<Utc>> {
        let rate = self.progress_rate()?;
        let last_snapshot = self.snapshots.last()?;

        if rate <= 0.0 {
            return None;
        }

        let remaining_percentage = 100.0 - last_snapshot.progress.percentage;
        let minutes_remaining = remaining_percentage / rate;

        Some(last_snapshot.timestamp + Duration::minutes(minutes_remaining as i64))
    }

    /// Gets all snapshots.
    pub fn snapshots(&self) -> &[ProgressSnapshot] {
        &self.snapshots
    }

    /// Clears all snapshots.
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }

    /// Gets progress trend (improving, stable, degrading).
    pub fn trend(&self) -> ProgressTrend {
        if self.snapshots.len() < 3 {
            return ProgressTrend::Stable;
        }

        let recent_snapshots = &self.snapshots[self.snapshots.len() - 3..];
        let rates: Vec<f64> = recent_snapshots
            .windows(2)
            .map(|window| {
                let duration = window[1]
                    .timestamp
                    .signed_duration_since(window[0].timestamp)
                    .num_seconds() as f64;
                if duration == 0.0 {
                    0.0
                } else {
                    (window[1].progress.percentage - window[0].progress.percentage) / duration
                }
            })
            .collect();

        let avg_rate = rates.iter().sum::<f64>() / rates.len() as f64;
        let first_rate = rates.first().copied().unwrap_or(0.0);
        let last_rate = rates.last().copied().unwrap_or(0.0);

        if last_rate > first_rate * 1.1 {
            ProgressTrend::Improving
        } else if last_rate < first_rate * 0.9 {
            ProgressTrend::Degrading
        } else {
            ProgressTrend::Stable
        }
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// A snapshot of workflow progress at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSnapshot {
    /// Snapshot timestamp.
    pub timestamp: DateTime<Utc>,

    /// Progress state at this time.
    pub progress: WorkflowProgress,
}

/// Progress trend indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressTrend {
    /// Progress is improving (accelerating).
    Improving,

    /// Progress is stable.
    Stable,

    /// Progress is degrading (slowing down).
    Degrading,
}

/// Progress reporter for periodic updates.
pub struct ProgressReporter {
    /// Callback for progress updates.
    callback: Box<dyn Fn(WorkflowProgress) + Send + Sync>,

    /// Report interval in seconds.
    interval_secs: u64,
}

impl ProgressReporter {
    /// Creates a new progress reporter.
    pub fn new<F>(callback: F, interval_secs: u64) -> Self
    where
        F: Fn(WorkflowProgress) + Send + Sync + 'static,
    {
        Self {
            callback: Box::new(callback),
            interval_secs,
        }
    }

    /// Reports progress.
    pub fn report(&self, progress: WorkflowProgress) {
        (self.callback)(progress);
    }

    /// Gets the report interval.
    pub fn interval_secs(&self) -> u64 {
        self.interval_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::WorkflowDag;
    use crate::state::WorkflowExecution;

    #[test]
    fn test_workflow_progress() {
        let workflow_id = uuid::Uuid::new_v4();
        let mut execution = WorkflowExecution::new(workflow_id, 1);

        // Add some tasks
        for i in 0..5 {
            let task_id = uuid::Uuid::new_v4();
            let task_exec = execution.get_or_create_task(task_id);
            if i < 2 {
                task_exec.start(None);
                task_exec.complete(crate::state::TaskResult::default());
            } else if i == 2 {
                task_exec.start(None);
            }
        }

        let progress = WorkflowProgress::from_execution(&execution);

        assert_eq!(progress.total_tasks, 5);
        assert_eq!(progress.completed_tasks, 2);
        assert_eq!(progress.running_tasks, 1);
        assert_eq!(progress.pending_tasks, 2);
        assert_eq!(progress.percentage, 40.0);
    }

    #[test]
    fn test_progress_tracker() {
        let mut tracker = ProgressTracker::new();

        let workflow_id = uuid::Uuid::new_v4();
        let execution = WorkflowExecution::new(workflow_id, 1);

        let progress = WorkflowProgress::from_execution(&execution);
        tracker.take_snapshot(progress);

        assert_eq!(tracker.snapshots().len(), 1);
    }

    #[test]
    fn test_eta_calculation() {
        let eta = WorkflowProgress::calculate_eta(
            5,
            10,
            Some(1000), // 1 second per task
            Some(Utc::now() - Duration::seconds(5)),
        );

        assert!(eta.is_some());
    }
}
