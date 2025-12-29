//! Error types for the Meridian workflow engine.

use std::fmt;
use thiserror::Error;

/// Result type alias for workflow operations.
pub type WorkflowResult<T> = Result<T, WorkflowError>;

/// Main error type for the workflow engine.
#[derive(Error, Debug, Clone)]
pub enum WorkflowError {
    /// Workflow not found.
    #[error("Workflow not found: {0}")]
    NotFound(String),

    /// Invalid workflow definition.
    #[error("Invalid workflow definition: {0}")]
    InvalidDefinition(String),

    /// Cycle detected in DAG.
    #[error("Cycle detected in workflow DAG")]
    CycleDetected,

    /// Task execution failed.
    #[error("Task execution failed: {task_id}, reason: {reason}")]
    TaskExecutionFailed { task_id: String, reason: String },

    /// Workflow execution timeout.
    #[error("Workflow execution timeout: {0}")]
    ExecutionTimeout(String),

    /// Invalid cron expression.
    #[error("Invalid cron expression: {0}")]
    InvalidCronExpression(String),

    /// Scheduling error.
    #[error("Scheduling error: {0}")]
    SchedulingError(String),

    /// Queue operation failed.
    #[error("Queue operation failed: {0}")]
    QueueError(String),

    /// State persistence error.
    #[error("State persistence error: {0}")]
    PersistenceError(String),

    /// Retry exhausted.
    #[error("Retry attempts exhausted for task: {0}")]
    RetryExhausted(String),

    /// Version mismatch.
    #[error("Workflow version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },

    /// Migration failed.
    #[error("Workflow migration failed: {0}")]
    MigrationFailed(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error.
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Invalid state transition.
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    /// Trigger condition not met.
    #[error("Trigger condition not met: {0}")]
    TriggerConditionNotMet(String),

    /// Template not found.
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Template rendering error.
    #[error("Template rendering error: {0}")]
    TemplateRenderError(String),

    /// Dependency not satisfied.
    #[error("Task dependency not satisfied: {0}")]
    DependencyNotSatisfied(String),

    /// Deadlock detected.
    #[error("Deadlock detected in workflow execution")]
    DeadlockDetected,

    /// Resource limit exceeded.
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// Cancellation requested.
    #[error("Workflow execution cancelled: {0}")]
    Cancelled(String),

    /// Internal error.
    #[error("Internal workflow engine error: {0}")]
    Internal(String),
}

impl WorkflowError {
    /// Returns true if the error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            WorkflowError::TaskExecutionFailed { .. }
                | WorkflowError::ExecutionTimeout(_)
                | WorkflowError::QueueError(_)
                | WorkflowError::PersistenceError(_)
                | WorkflowError::Internal(_)
        )
    }

    /// Returns true if the error should be sent to dead letter queue.
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            WorkflowError::InvalidDefinition(_)
                | WorkflowError::CycleDetected
                | WorkflowError::RetryExhausted(_)
                | WorkflowError::VersionMismatch { .. }
                | WorkflowError::InvalidStateTransition { .. }
                | WorkflowError::DeadlockDetected
        )
    }
}

impl From<serde_json::Error> for WorkflowError {
    fn from(err: serde_json::Error) -> Self {
        WorkflowError::SerializationError(err.to_string())
    }
}

impl From<bincode::Error> for WorkflowError {
    fn from(err: bincode::Error) -> Self {
        WorkflowError::SerializationError(err.to_string())
    }
}

impl From<cron::error::Error> for WorkflowError {
    fn from(err: cron::error::Error) -> Self {
        WorkflowError::InvalidCronExpression(err.to_string())
    }
}
