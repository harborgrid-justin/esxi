//! Workflow state machine with persistence support.

use crate::dag::{TaskId, WorkflowId};
use crate::error::{WorkflowError, WorkflowResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Execution state for a workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowState {
    /// Workflow is pending execution.
    Pending,

    /// Workflow is currently running.
    Running,

    /// Workflow completed successfully.
    Completed,

    /// Workflow failed.
    Failed,

    /// Workflow was cancelled.
    Cancelled,

    /// Workflow is paused.
    Paused,

    /// Workflow is waiting for external event.
    Waiting,
}

impl WorkflowState {
    /// Returns true if the state is terminal (no further transitions possible).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            WorkflowState::Completed | WorkflowState::Failed | WorkflowState::Cancelled
        )
    }

    /// Returns true if the workflow is active (running or paused).
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            WorkflowState::Running | WorkflowState::Paused | WorkflowState::Waiting
        )
    }

    /// Validates a state transition.
    pub fn can_transition_to(&self, new_state: &WorkflowState) -> bool {
        match (self, new_state) {
            // From Pending
            (WorkflowState::Pending, WorkflowState::Running) => true,
            (WorkflowState::Pending, WorkflowState::Cancelled) => true,

            // From Running
            (WorkflowState::Running, WorkflowState::Completed) => true,
            (WorkflowState::Running, WorkflowState::Failed) => true,
            (WorkflowState::Running, WorkflowState::Cancelled) => true,
            (WorkflowState::Running, WorkflowState::Paused) => true,
            (WorkflowState::Running, WorkflowState::Waiting) => true,

            // From Paused
            (WorkflowState::Paused, WorkflowState::Running) => true,
            (WorkflowState::Paused, WorkflowState::Cancelled) => true,

            // From Waiting
            (WorkflowState::Waiting, WorkflowState::Running) => true,
            (WorkflowState::Waiting, WorkflowState::Cancelled) => true,
            (WorkflowState::Waiting, WorkflowState::Failed) => true,

            // No transitions from terminal states
            (WorkflowState::Completed, _) => false,
            (WorkflowState::Failed, _) => false,
            (WorkflowState::Cancelled, _) => false,

            // Same state is always allowed
            (a, b) if a == b => true,

            // All other transitions are invalid
            _ => false,
        }
    }
}

/// Execution state for a task.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskState {
    /// Task is pending execution.
    Pending,

    /// Task is currently running.
    Running,

    /// Task completed successfully.
    Completed,

    /// Task failed.
    Failed,

    /// Task was skipped (e.g., due to conditional branch).
    Skipped,

    /// Task is being retried.
    Retrying,
}

impl TaskState {
    /// Returns true if the state is terminal.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskState::Completed | TaskState::Failed | TaskState::Skipped
        )
    }
}

/// Result of a task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task output data.
    pub output: serde_json::Value,

    /// Execution duration in milliseconds.
    pub duration_ms: u64,

    /// Error message if the task failed.
    pub error: Option<String>,

    /// Custom metrics.
    pub metrics: HashMap<String, f64>,
}

impl Default for TaskResult {
    fn default() -> Self {
        Self {
            output: serde_json::Value::Null,
            duration_ms: 0,
            error: None,
            metrics: HashMap::new(),
        }
    }
}

/// Execution context for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    /// Task ID.
    pub task_id: TaskId,

    /// Current state.
    pub state: TaskState,

    /// Start time.
    pub started_at: Option<DateTime<Utc>>,

    /// Completion time.
    pub completed_at: Option<DateTime<Utc>>,

    /// Number of attempts.
    pub attempt_count: u32,

    /// Task result (if completed or failed).
    pub result: Option<TaskResult>,

    /// Worker ID executing this task.
    pub worker_id: Option<String>,
}

impl TaskExecution {
    /// Creates a new task execution.
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            state: TaskState::Pending,
            started_at: None,
            completed_at: None,
            attempt_count: 0,
            result: None,
            worker_id: None,
        }
    }

    /// Marks the task as started.
    pub fn start(&mut self, worker_id: Option<String>) {
        self.state = TaskState::Running;
        self.started_at = Some(Utc::now());
        self.worker_id = worker_id;
        self.attempt_count += 1;
    }

    /// Marks the task as completed.
    pub fn complete(&mut self, result: TaskResult) {
        self.state = TaskState::Completed;
        self.completed_at = Some(Utc::now());
        self.result = Some(result);
    }

    /// Marks the task as failed.
    pub fn fail(&mut self, error: String) {
        self.state = TaskState::Failed;
        self.completed_at = Some(Utc::now());
        self.result = Some(TaskResult {
            error: Some(error),
            ..Default::default()
        });
    }

    /// Marks the task for retry.
    pub fn retry(&mut self) {
        self.state = TaskState::Retrying;
    }

    /// Marks the task as skipped.
    pub fn skip(&mut self) {
        self.state = TaskState::Skipped;
        self.completed_at = Some(Utc::now());
    }

    /// Gets the execution duration in milliseconds.
    pub fn duration_ms(&self) -> Option<u64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => {
                let duration = end.signed_duration_since(start);
                Some(duration.num_milliseconds() as u64)
            }
            _ => None,
        }
    }
}

/// Workflow execution state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// Workflow ID.
    pub workflow_id: WorkflowId,

    /// Execution ID (unique for each run).
    pub execution_id: String,

    /// Current workflow state.
    pub state: WorkflowState,

    /// Workflow version.
    pub version: u32,

    /// Start time.
    pub started_at: Option<DateTime<Utc>>,

    /// Completion time.
    pub completed_at: Option<DateTime<Utc>>,

    /// Task executions.
    pub tasks: HashMap<TaskId, TaskExecution>,

    /// Workflow-level output data.
    pub output: serde_json::Value,

    /// Error message if the workflow failed.
    pub error: Option<String>,

    /// Custom metadata.
    pub metadata: HashMap<String, String>,
}

impl WorkflowExecution {
    /// Creates a new workflow execution.
    pub fn new(workflow_id: WorkflowId, version: u32) -> Self {
        Self {
            workflow_id,
            execution_id: uuid::Uuid::new_v4().to_string(),
            state: WorkflowState::Pending,
            version,
            started_at: None,
            completed_at: None,
            tasks: HashMap::new(),
            output: serde_json::Value::Null,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Transitions to a new state.
    pub fn transition(&mut self, new_state: WorkflowState) -> WorkflowResult<()> {
        if !self.state.can_transition_to(&new_state) {
            return Err(WorkflowError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: format!("{:?}", new_state),
            });
        }

        info!(
            "Workflow {} execution {} transitioning from {:?} to {:?}",
            self.workflow_id, self.execution_id, self.state, new_state
        );

        self.state = new_state;

        // Update timestamps
        match self.state {
            WorkflowState::Running if self.started_at.is_none() => {
                self.started_at = Some(Utc::now());
            }
            WorkflowState::Completed | WorkflowState::Failed | WorkflowState::Cancelled => {
                self.completed_at = Some(Utc::now());
            }
            _ => {}
        }

        Ok(())
    }

    /// Gets or creates a task execution.
    pub fn get_or_create_task(&mut self, task_id: TaskId) -> &mut TaskExecution {
        self.tasks
            .entry(task_id)
            .or_insert_with(|| TaskExecution::new(task_id))
    }

    /// Gets the execution duration in milliseconds.
    pub fn duration_ms(&self) -> Option<u64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => {
                let duration = end.signed_duration_since(start);
                Some(duration.num_milliseconds() as u64)
            }
            _ => None,
        }
    }

    /// Gets the completion percentage (0-100).
    pub fn completion_percentage(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }

        let completed = self
            .tasks
            .values()
            .filter(|t| t.state.is_terminal())
            .count();

        (completed as f64 / self.tasks.len() as f64) * 100.0
    }

    /// Gets counts of tasks by state.
    pub fn task_state_counts(&self) -> HashMap<TaskState, usize> {
        let mut counts = HashMap::new();
        for task in self.tasks.values() {
            *counts.entry(task.state.clone()).or_insert(0) += 1;
        }
        counts
    }
}

/// State manager for workflow executions.
pub struct StateManager {
    /// Active workflow executions.
    executions: Arc<RwLock<HashMap<String, WorkflowExecution>>>,
}

impl StateManager {
    /// Creates a new state manager.
    pub fn new() -> Self {
        Self {
            executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Creates a new workflow execution.
    pub async fn create_execution(
        &self,
        workflow_id: WorkflowId,
        version: u32,
    ) -> WorkflowResult<String> {
        let execution = WorkflowExecution::new(workflow_id, version);
        let execution_id = execution.execution_id.clone();

        let mut executions = self.executions.write().await;
        executions.insert(execution_id.clone(), execution);

        info!("Created workflow execution {}", execution_id);
        Ok(execution_id)
    }

    /// Gets a workflow execution.
    pub async fn get_execution(&self, execution_id: &str) -> WorkflowResult<WorkflowExecution> {
        let executions = self.executions.read().await;
        executions
            .get(execution_id)
            .cloned()
            .ok_or_else(|| WorkflowError::NotFound(format!("Execution {} not found", execution_id)))
    }

    /// Updates a workflow execution.
    pub async fn update_execution(&self, execution: WorkflowExecution) -> WorkflowResult<()> {
        let mut executions = self.executions.write().await;
        executions.insert(execution.execution_id.clone(), execution);
        Ok(())
    }

    /// Transitions a workflow to a new state.
    pub async fn transition_workflow(
        &self,
        execution_id: &str,
        new_state: WorkflowState,
    ) -> WorkflowResult<()> {
        let mut executions = self.executions.write().await;
        let execution = executions.get_mut(execution_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Execution {} not found", execution_id))
        })?;

        execution.transition(new_state)?;
        Ok(())
    }

    /// Updates a task execution state.
    pub async fn update_task(
        &self,
        execution_id: &str,
        task_id: TaskId,
        update_fn: impl FnOnce(&mut TaskExecution),
    ) -> WorkflowResult<()> {
        let mut executions = self.executions.write().await;
        let execution = executions.get_mut(execution_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Execution {} not found", execution_id))
        })?;

        let task_exec = execution.get_or_create_task(task_id);
        update_fn(task_exec);

        debug!("Updated task {} in execution {}", task_id, execution_id);
        Ok(())
    }

    /// Deletes a workflow execution.
    pub async fn delete_execution(&self, execution_id: &str) -> WorkflowResult<()> {
        let mut executions = self.executions.write().await;
        executions
            .remove(execution_id)
            .ok_or_else(|| WorkflowError::NotFound(format!("Execution {} not found", execution_id)))?;

        info!("Deleted workflow execution {}", execution_id);
        Ok(())
    }

    /// Lists all executions for a workflow.
    pub async fn list_executions(&self, workflow_id: WorkflowId) -> Vec<WorkflowExecution> {
        let executions = self.executions.read().await;
        executions
            .values()
            .filter(|e| e.workflow_id == workflow_id)
            .cloned()
            .collect()
    }

    /// Lists all active executions.
    pub async fn list_active_executions(&self) -> Vec<WorkflowExecution> {
        let executions = self.executions.read().await;
        executions
            .values()
            .filter(|e| e.state.is_active())
            .cloned()
            .collect()
    }

    /// Gets execution statistics.
    pub async fn get_stats(&self) -> HashMap<String, usize> {
        let executions = self.executions.read().await;
        let mut stats = HashMap::new();

        stats.insert("total".to_string(), executions.len());

        let mut state_counts: HashMap<String, usize> = HashMap::new();
        for execution in executions.values() {
            let state_name = format!("{:?}", execution.state);
            *state_counts.entry(state_name).or_insert(0) += 1;
        }

        stats.extend(state_counts);
        stats
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_state_transitions() {
        let mut state = WorkflowState::Pending;

        assert!(state.can_transition_to(&WorkflowState::Running));
        state = WorkflowState::Running;

        assert!(state.can_transition_to(&WorkflowState::Completed));
        assert!(state.can_transition_to(&WorkflowState::Failed));
        assert!(state.can_transition_to(&WorkflowState::Paused));

        state = WorkflowState::Completed;
        assert!(!state.can_transition_to(&WorkflowState::Running));
    }

    #[test]
    fn test_task_execution() {
        let task_id = uuid::Uuid::new_v4();
        let mut task_exec = TaskExecution::new(task_id);

        task_exec.start(Some("worker-1".to_string()));
        assert_eq!(task_exec.state, TaskState::Running);
        assert_eq!(task_exec.attempt_count, 1);

        task_exec.complete(TaskResult::default());
        assert_eq!(task_exec.state, TaskState::Completed);
        assert!(task_exec.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_state_manager() {
        let manager = StateManager::new();
        let workflow_id = uuid::Uuid::new_v4();

        let execution_id = manager.create_execution(workflow_id, 1).await.unwrap();

        let execution = manager.get_execution(&execution_id).await.unwrap();
        assert_eq!(execution.state, WorkflowState::Pending);

        manager
            .transition_workflow(&execution_id, WorkflowState::Running)
            .await
            .unwrap();

        let execution = manager.get_execution(&execution_id).await.unwrap();
        assert_eq!(execution.state, WorkflowState::Running);
    }
}
