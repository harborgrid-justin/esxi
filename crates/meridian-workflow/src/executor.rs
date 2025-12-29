//! Workflow executor for running DAG-based workflows with parallel and sequential execution.

use crate::dag::{DependencyType, Task, TaskId, WorkflowDag, WorkflowId};
use crate::dlq::{DeadLetter, DeadLetterQueue, DeadLetterReason};
use crate::error::{WorkflowError, WorkflowResult};
use crate::queue::{Priority, QueuedTask, TaskQueue};
use crate::retry::{RetryPolicy, RetryState};
use crate::state::{StateManager, TaskResult, TaskState, WorkflowState};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout, Duration};
use tracing::{debug, error, info, warn};

/// Context passed to task handlers during execution.
#[derive(Debug, Clone)]
pub struct TaskContext {
    /// Workflow ID.
    pub workflow_id: WorkflowId,

    /// Execution ID.
    pub execution_id: String,

    /// Task ID.
    pub task_id: TaskId,

    /// Task configuration.
    pub config: serde_json::Value,

    /// Input data from dependencies.
    pub inputs: HashMap<TaskId, serde_json::Value>,

    /// Metadata.
    pub metadata: HashMap<String, String>,

    /// Current attempt number.
    pub attempt: u32,
}

/// Trait for task execution handlers.
#[async_trait]
pub trait TaskHandler: Send + Sync {
    /// Executes a task with the given context.
    async fn execute(&self, ctx: TaskContext) -> WorkflowResult<serde_json::Value>;

    /// Gets the task type this handler can process.
    fn task_type(&self) -> &str;
}

/// Execution options for a workflow.
#[derive(Debug, Clone)]
pub struct ExecutionOptions {
    /// Maximum concurrent tasks.
    pub max_concurrent: usize,

    /// Default retry policy for tasks.
    pub default_retry_policy: RetryPolicy,

    /// Whether to stop execution on first failure.
    pub fail_fast: bool,

    /// Execution timeout.
    pub timeout: Option<Duration>,

    /// Worker ID for tracking.
    pub worker_id: Option<String>,
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            default_retry_policy: RetryPolicy::default(),
            fail_fast: false,
            timeout: Some(Duration::from_secs(3600)), // 1 hour
            worker_id: None,
        }
    }
}

/// Workflow executor.
pub struct WorkflowExecutor {
    /// Task handlers by task type.
    handlers: Arc<RwLock<HashMap<String, Arc<dyn TaskHandler>>>>,

    /// State manager.
    state_manager: Arc<StateManager>,

    /// Task queue.
    task_queue: Arc<TaskQueue<TaskExecution>>,

    /// Dead letter queue.
    dlq: Arc<DeadLetterQueue<TaskExecution>>,

    /// Retry policies by task type.
    retry_policies: Arc<RwLock<HashMap<String, RetryPolicy>>>,

    /// Worker pool size.
    worker_pool_size: usize,
}

/// Internal task execution wrapper.
#[derive(Debug, Clone)]
struct TaskExecution {
    workflow_id: WorkflowId,
    execution_id: String,
    task: Task,
    inputs: HashMap<TaskId, serde_json::Value>,
    retry_state: RetryState,
}

impl WorkflowExecutor {
    /// Creates a new workflow executor.
    pub fn new(state_manager: Arc<StateManager>) -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            state_manager,
            task_queue: Arc::new(TaskQueue::new("workflow_tasks", 100)),
            dlq: Arc::new(DeadLetterQueue::new("workflow_dlq")),
            retry_policies: Arc::new(RwLock::new(HashMap::new())),
            worker_pool_size: 10,
        }
    }

    /// Sets the worker pool size.
    pub fn with_worker_pool_size(mut self, size: usize) -> Self {
        self.worker_pool_size = size;
        self
    }

    /// Registers a task handler.
    pub async fn register_handler(&self, handler: Arc<dyn TaskHandler>) {
        let task_type = handler.task_type().to_string();
        let mut handlers = self.handlers.write().await;
        handlers.insert(task_type.clone(), handler);
        info!("Registered handler for task type: {}", task_type);
    }

    /// Sets a retry policy for a specific task type.
    pub async fn set_retry_policy(&self, task_type: impl Into<String>, policy: RetryPolicy) {
        let mut policies = self.retry_policies.write().await;
        policies.insert(task_type.into(), policy);
    }

    /// Executes a workflow.
    pub async fn execute(
        &self,
        workflow: &WorkflowDag,
        options: ExecutionOptions,
    ) -> WorkflowResult<String> {
        // Validate the workflow
        workflow.validate()?;

        // Create execution
        let execution_id = self
            .state_manager
            .create_execution(workflow.id, workflow.version)
            .await?;

        info!(
            "Starting workflow {} execution {}",
            workflow.id, execution_id
        );

        // Transition to running state
        self.state_manager
            .transition_workflow(&execution_id, WorkflowState::Running)
            .await?;

        // Execute with timeout if specified
        let result = if let Some(exec_timeout) = options.timeout {
            match timeout(
                exec_timeout,
                self.execute_workflow(workflow, &execution_id, &options),
            )
            .await
            {
                Ok(result) => result,
                Err(_) => {
                    self.handle_workflow_timeout(&execution_id).await?;
                    return Err(WorkflowError::ExecutionTimeout(execution_id.clone()));
                }
            }
        } else {
            self.execute_workflow(workflow, &execution_id, &options)
                .await
        };

        // Handle result
        match result {
            Ok(()) => {
                self.state_manager
                    .transition_workflow(&execution_id, WorkflowState::Completed)
                    .await?;
                info!("Workflow execution {} completed successfully", execution_id);
                Ok(execution_id)
            }
            Err(e) => {
                self.state_manager
                    .transition_workflow(&execution_id, WorkflowState::Failed)
                    .await?;
                error!("Workflow execution {} failed: {}", execution_id, e);
                Err(e)
            }
        }
    }

    /// Internal workflow execution logic.
    async fn execute_workflow(
        &self,
        workflow: &WorkflowDag,
        execution_id: &str,
        options: &ExecutionOptions,
    ) -> WorkflowResult<()> {
        let mut completed_tasks: HashSet<TaskId> = HashSet::new();
        let mut failed_tasks: HashSet<TaskId> = HashSet::new();
        let total_tasks = workflow.tasks().len();

        // Get topological order for proper sequencing
        let topo_order = workflow.topological_sort()?;

        while completed_tasks.len() + failed_tasks.len() < total_tasks {
            // Get tasks that can be executed now
            let ready_tasks = workflow.get_parallel_tasks(&completed_tasks);

            if ready_tasks.is_empty() {
                // Check if we're stuck (remaining tasks have unsatisfied dependencies)
                if completed_tasks.len() + failed_tasks.len() < total_tasks {
                    // Some tasks are blocked by failed dependencies
                    if options.fail_fast || !failed_tasks.is_empty() {
                        return Err(WorkflowError::DependencyNotSatisfied(
                            "Some tasks cannot proceed due to failed dependencies".to_string(),
                        ));
                    }
                }
                break;
            }

            // Execute ready tasks in parallel (up to max_concurrent)
            let mut handles = Vec::new();

            for task in ready_tasks.iter().take(options.max_concurrent) {
                let task_id = task.id;

                // Get inputs from completed dependencies
                let inputs = self
                    .get_task_inputs(execution_id, task_id, workflow)
                    .await?;

                // Prepare task execution
                let task_exec = TaskExecution {
                    workflow_id: workflow.id,
                    execution_id: execution_id.to_string(),
                    task: (*task).clone(),
                    inputs,
                    retry_state: RetryState::new(),
                };

                // Execute task
                let executor = self.clone_for_task();
                let exec_id = execution_id.to_string();
                let opts = options.clone();

                let handle = tokio::spawn(async move {
                    executor.execute_task(task_exec, &exec_id, &opts).await
                });

                handles.push((task_id, handle));
            }

            // Wait for tasks to complete
            for (task_id, handle) in handles {
                match handle.await {
                    Ok(Ok(())) => {
                        completed_tasks.insert(task_id);
                        debug!("Task {} completed successfully", task_id);
                    }
                    Ok(Err(e)) => {
                        failed_tasks.insert(task_id);
                        error!("Task {} failed: {}", task_id, e);

                        if options.fail_fast {
                            return Err(e);
                        }
                    }
                    Err(e) => {
                        failed_tasks.insert(task_id);
                        error!("Task {} panicked: {}", task_id, e);

                        if options.fail_fast {
                            return Err(WorkflowError::TaskExecutionFailed {
                                task_id: task_id.to_string(),
                                reason: format!("Task panicked: {}", e),
                            });
                        }
                    }
                }
            }

            // Small delay to prevent tight loop
            sleep(Duration::from_millis(100)).await;
        }

        // Check if all tasks completed successfully
        if !failed_tasks.is_empty() {
            return Err(WorkflowError::TaskExecutionFailed {
                task_id: "multiple".to_string(),
                reason: format!("{} tasks failed", failed_tasks.len()),
            });
        }

        Ok(())
    }

    /// Executes a single task with retry logic.
    async fn execute_task(
        &self,
        mut task_exec: TaskExecution,
        execution_id: &str,
        options: &ExecutionOptions,
    ) -> WorkflowResult<()> {
        let task_id = task_exec.task.id;
        let task_type = &task_exec.task.task_type;

        // Get handler
        let handlers = self.handlers.read().await;
        let handler = handlers.get(task_type).ok_or_else(|| {
            WorkflowError::InvalidDefinition(format!("No handler for task type: {}", task_type))
        })?;
        let handler = Arc::clone(handler);
        drop(handlers);

        // Get retry policy
        let retry_policy = {
            let policies = self.retry_policies.read().await;
            policies
                .get(task_type)
                .cloned()
                .unwrap_or_else(|| options.default_retry_policy.clone())
        };

        let start_time = Utc::now();

        loop {
            // Update state to running
            self.state_manager
                .update_task(execution_id, task_id, |task_state| {
                    task_state.start(options.worker_id.clone());
                })
                .await?;

            // Prepare context
            let ctx = TaskContext {
                workflow_id: task_exec.workflow_id,
                execution_id: execution_id.to_string(),
                task_id,
                config: task_exec.task.config.clone(),
                inputs: task_exec.inputs.clone(),
                metadata: task_exec.task.metadata.clone(),
                attempt: task_exec.retry_state.attempt + 1,
            };

            // Execute with timeout
            let exec_start = std::time::Instant::now();
            let result = if let Some(timeout_secs) = task_exec.task.timeout_secs {
                match timeout(Duration::from_secs(timeout_secs), handler.execute(ctx)).await {
                    Ok(result) => result,
                    Err(_) => Err(WorkflowError::ExecutionTimeout(format!(
                        "Task {} timeout after {} seconds",
                        task_id, timeout_secs
                    ))),
                }
            } else {
                handler.execute(ctx).await
            };

            let duration_ms = exec_start.elapsed().as_millis() as u64;

            // Handle result
            match result {
                Ok(output) => {
                    // Task succeeded
                    let task_result = TaskResult {
                        output,
                        duration_ms,
                        error: None,
                        metrics: HashMap::new(),
                    };

                    self.state_manager
                        .update_task(execution_id, task_id, |task_state| {
                            task_state.complete(task_result);
                        })
                        .await?;

                    info!("Task {} completed in {}ms", task_id, duration_ms);
                    return Ok(());
                }
                Err(e) => {
                    // Task failed
                    task_exec.retry_state.update_elapsed(start_time);

                    let elapsed_ms = task_exec.retry_state.elapsed_ms;
                    let should_retry = retry_policy.should_retry(
                        task_exec.retry_state.attempt + 1,
                        &e,
                        elapsed_ms,
                    );

                    if should_retry && task_exec.task.retryable {
                        // Schedule retry
                        let delay = task_exec
                            .retry_state
                            .record_failure(e.to_string(), &retry_policy);

                        if let Some(delay) = delay {
                            warn!(
                                "Task {} failed, retrying in {:?} (attempt {})",
                                task_id,
                                delay,
                                task_exec.retry_state.attempt
                            );

                            self.state_manager
                                .update_task(execution_id, task_id, |task_state| {
                                    task_state.retry();
                                })
                                .await?;

                            sleep(delay).await;
                            continue;
                        }
                    }

                    // No more retries or not retryable
                    self.state_manager
                        .update_task(execution_id, task_id, |task_state| {
                            task_state.fail(e.to_string());
                        })
                        .await?;

                    // Send to DLQ if fatal or retries exhausted
                    if e.is_fatal() || task_exec.retry_state.is_exhausted(&retry_policy) {
                        let reason = if e.is_fatal() {
                            DeadLetterReason::FatalError {
                                error: e.to_string(),
                            }
                        } else {
                            DeadLetterReason::RetriesExhausted
                        };

                        let dead_letter = DeadLetter::new(
                            task_exec.clone(),
                            reason,
                            e.to_string(),
                            task_exec.retry_state.attempt,
                            start_time,
                        );

                        self.dlq.add(dead_letter).await?;
                    }

                    return Err(e);
                }
            }
        }
    }

    /// Gets input data for a task from its dependencies.
    async fn get_task_inputs(
        &self,
        execution_id: &str,
        task_id: TaskId,
        workflow: &WorkflowDag,
    ) -> WorkflowResult<HashMap<TaskId, serde_json::Value>> {
        let mut inputs = HashMap::new();
        let dependencies = workflow.get_task_dependencies(task_id);

        let execution = self.state_manager.get_execution(execution_id).await?;

        for (dep_task_id, dep_type) in dependencies {
            let task_state = execution
                .tasks
                .get(&dep_task_id)
                .ok_or_else(|| {
                    WorkflowError::DependencyNotSatisfied(format!(
                        "Dependency task {} not found",
                        dep_task_id
                    ))
                })?;

            if let Some(ref result) = task_state.result {
                inputs.insert(dep_task_id, result.output.clone());
            }
        }

        Ok(inputs)
    }

    /// Handles workflow timeout.
    async fn handle_workflow_timeout(&self, execution_id: &str) -> WorkflowResult<()> {
        error!("Workflow execution {} timed out", execution_id);

        let mut execution = self.state_manager.get_execution(execution_id).await?;
        execution.error = Some("Workflow execution timed out".to_string());

        self.state_manager.update_execution(execution).await?;
        Ok(())
    }

    /// Clones the executor for task execution (lighter weight).
    fn clone_for_task(&self) -> Self {
        Self {
            handlers: Arc::clone(&self.handlers),
            state_manager: Arc::clone(&self.state_manager),
            task_queue: Arc::clone(&self.task_queue),
            dlq: Arc::clone(&self.dlq),
            retry_policies: Arc::clone(&self.retry_policies),
            worker_pool_size: self.worker_pool_size,
        }
    }

    /// Gets the state manager.
    pub fn state_manager(&self) -> &Arc<StateManager> {
        &self.state_manager
    }

    /// Gets the dead letter queue.
    pub fn dlq(&self) -> &Arc<DeadLetterQueue<TaskExecution>> {
        &self.dlq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTaskHandler;

    #[async_trait]
    impl TaskHandler for MockTaskHandler {
        async fn execute(&self, _ctx: TaskContext) -> WorkflowResult<serde_json::Value> {
            Ok(serde_json::json!({"status": "ok"}))
        }

        fn task_type(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_executor_creation() {
        let state_manager = Arc::new(StateManager::new());
        let executor = WorkflowExecutor::new(state_manager);

        executor.register_handler(Arc::new(MockTaskHandler)).await;
    }
}
