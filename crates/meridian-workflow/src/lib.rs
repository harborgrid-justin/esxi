//! # Meridian Workflow Engine
//!
//! A comprehensive enterprise workflow engine for the Meridian GIS Platform.
//!
//! ## Features
//!
//! - **DAG-based Workflows**: Define complex workflows using directed acyclic graphs
//! - **Cron Scheduling**: Schedule workflows with cron expressions and timezone support
//! - **Distributed Task Queue**: Priority-based task queue with concurrency control
//! - **State Machine**: Robust workflow and task state management with persistence
//! - **Retry Policies**: Configurable retry strategies with exponential backoff
//! - **Dead Letter Queue**: Handle failed tasks with comprehensive error tracking
//! - **Versioning**: Workflow version management with migration support
//! - **Event Triggers**: Event-driven workflow execution
//! - **Templates**: Reusable workflow templates with parameterization
//! - **Progress Tracking**: Real-time progress monitoring with ETA estimation
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use meridian_workflow::*;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a workflow
//!     let mut workflow = dag::WorkflowDag::new("data_pipeline");
//!
//!     let task1 = dag::Task::new("extract", "data_extract")
//!         .with_config(serde_json::json!({"source": "database"}));
//!     let task2 = dag::Task::new("transform", "data_transform");
//!     let task3 = dag::Task::new("load", "data_load");
//!
//!     let id1 = workflow.add_task(task1);
//!     let id2 = workflow.add_task(task2);
//!     let id3 = workflow.add_task(task3);
//!
//!     workflow.add_dependency(id1, id2, dag::DependencyType::Sequential)?;
//!     workflow.add_dependency(id2, id3, dag::DependencyType::Sequential)?;
//!
//!     // Validate the workflow
//!     workflow.validate()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The workflow engine consists of several key components:
//!
//! - **DAG Module**: Workflow definition and validation
//! - **Scheduler**: Cron-based job scheduling
//! - **Queue**: Distributed task queue with priorities
//! - **Executor**: Workflow execution engine
//! - **State Manager**: Workflow and task state tracking
//! - **Retry Module**: Configurable retry policies
//! - **DLQ**: Dead letter queue for failed tasks
//! - **Versioning**: Workflow version control
//! - **Triggers**: Event-driven workflow execution
//! - **Templates**: Reusable workflow patterns
//! - **Progress**: Real-time progress tracking
//!
//! ## Examples
//!
//! ### Creating a Sequential Workflow
//!
//! ```rust
//! use meridian_workflow::dag::{WorkflowDag, Task, DependencyType};
//!
//! let mut workflow = WorkflowDag::new("sequential_pipeline");
//!
//! let task1 = Task::new("step1", "processing");
//! let task2 = Task::new("step2", "analysis");
//! let task3 = Task::new("step3", "finalize");
//!
//! let id1 = workflow.add_task(task1);
//! let id2 = workflow.add_task(task2);
//! let id3 = workflow.add_task(task3);
//!
//! workflow.add_dependency(id1, id2, DependencyType::Sequential).unwrap();
//! workflow.add_dependency(id2, id3, DependencyType::Sequential).unwrap();
//!
//! assert!(workflow.validate().is_ok());
//! ```
//!
//! ### Scheduling a Workflow
//!
//! ```rust,no_run
//! use meridian_workflow::scheduler::{CronScheduler, ScheduleConfig};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let callback = Arc::new(|workflow_id: String| {
//!         tokio::spawn(async move {
//!             println!("Executing workflow: {}", workflow_id);
//!             Ok(())
//!         })
//!     });
//!
//!     let mut scheduler = CronScheduler::new(callback);
//!
//!     let config = ScheduleConfig::new("0 0 * * *", "UTC"); // Daily at midnight
//!     scheduler.add_schedule(config, "my_workflow".to_string()).await.unwrap();
//!
//!     scheduler.start();
//!
//!     // Keep running...
//! }
//! ```
//!
//! ### Using Templates
//!
//! ```rust
//! use meridian_workflow::templates::{TemplateRegistry, StandardTemplates};
//! use std::collections::HashMap;
//!
//! # #[tokio::main]
//! # async fn main() {
//! let registry = TemplateRegistry::new();
//!
//! // Register a standard template
//! let template = StandardTemplates::sequential_pipeline();
//! let template_id = registry.register_template(template).await.unwrap();
//!
//! // Render a workflow from the template
//! let mut params = HashMap::new();
//! params.insert("task_type_1".to_string(), serde_json::json!("extract"));
//! params.insert("task_type_2".to_string(), serde_json::json!("transform"));
//! params.insert("task_type_3".to_string(), serde_json::json!("load"));
//!
//! let workflow = registry.render_workflow(template_id, params).await.unwrap();
//! # }
//! ```

pub mod dag;
pub mod dlq;
pub mod error;
pub mod executor;
pub mod progress;
pub mod queue;
pub mod retry;
pub mod scheduler;
pub mod state;
pub mod templates;
pub mod triggers;
pub mod versioning;

// Re-export commonly used types
pub use dag::{DependencyType, Task, TaskId, WorkflowDag, WorkflowId};
pub use error::{WorkflowError, WorkflowResult};
pub use executor::{ExecutionOptions, TaskContext, TaskHandler, WorkflowExecutor};
pub use progress::{ProgressTracker, WorkflowProgress};
pub use queue::{Priority, QueuedTask, TaskQueue};
pub use retry::{RetryPolicy, RetryStrategy};
pub use scheduler::{CronScheduler, ScheduleConfig};
pub use state::{StateManager, TaskState, WorkflowExecution, WorkflowState};
pub use templates::{TemplateRegistry, WorkflowTemplate};
pub use triggers::{Event, Trigger, TriggerCondition, TriggerManager};
pub use versioning::{MigrationPlan, VersionRegistry, WorkflowVersion};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::dag::{DependencyType, Task, TaskId, WorkflowDag, WorkflowId};
    pub use crate::error::{WorkflowError, WorkflowResult};
    pub use crate::executor::{ExecutionOptions, TaskContext, TaskHandler, WorkflowExecutor};
    pub use crate::progress::{ProgressTracker, WorkflowProgress};
    pub use crate::queue::{Priority, QueuedTask, TaskQueue};
    pub use crate::retry::{RetryPolicy, RetryStrategy};
    pub use crate::scheduler::{CronScheduler, ScheduleConfig};
    pub use crate::state::{StateManager, TaskState, WorkflowExecution, WorkflowState};
    pub use crate::templates::{TemplateRegistry, WorkflowTemplate};
    pub use crate::triggers::{Event, Trigger, TriggerCondition, TriggerManager};
    pub use crate::versioning::{MigrationPlan, VersionRegistry, WorkflowVersion};
}

/// Current version of the workflow engine.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Workflow engine builder for easy initialization.
pub struct WorkflowEngineBuilder {
    worker_pool_size: usize,
    max_queue_size: usize,
    enable_scheduler: bool,
    enable_triggers: bool,
}

impl WorkflowEngineBuilder {
    /// Creates a new workflow engine builder.
    pub fn new() -> Self {
        Self {
            worker_pool_size: 10,
            max_queue_size: 1000,
            enable_scheduler: true,
            enable_triggers: true,
        }
    }

    /// Sets the worker pool size.
    pub fn worker_pool_size(mut self, size: usize) -> Self {
        self.worker_pool_size = size;
        self
    }

    /// Sets the maximum queue size.
    pub fn max_queue_size(mut self, size: usize) -> Self {
        self.max_queue_size = size;
        self
    }

    /// Enables or disables the scheduler.
    pub fn enable_scheduler(mut self, enable: bool) -> Self {
        self.enable_scheduler = enable;
        self
    }

    /// Enables or disables triggers.
    pub fn enable_triggers(mut self, enable: bool) -> Self {
        self.enable_triggers = enable;
        self
    }

    /// Builds the workflow engine.
    pub fn build(self) -> WorkflowEngine {
        let state_manager = std::sync::Arc::new(StateManager::new());
        let executor = WorkflowExecutor::new(state_manager.clone())
            .with_worker_pool_size(self.worker_pool_size);

        WorkflowEngine {
            executor: std::sync::Arc::new(executor),
            state_manager,
            version_registry: std::sync::Arc::new(VersionRegistry::new()),
            template_registry: std::sync::Arc::new(TemplateRegistry::new()),
        }
    }
}

impl Default for WorkflowEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Main workflow engine facade.
pub struct WorkflowEngine {
    executor: std::sync::Arc<WorkflowExecutor>,
    state_manager: std::sync::Arc<StateManager>,
    version_registry: std::sync::Arc<VersionRegistry>,
    template_registry: std::sync::Arc<TemplateRegistry>,
}

impl WorkflowEngine {
    /// Creates a new workflow engine with default settings.
    pub fn new() -> Self {
        WorkflowEngineBuilder::new().build()
    }

    /// Returns a builder for customizing the engine.
    pub fn builder() -> WorkflowEngineBuilder {
        WorkflowEngineBuilder::new()
    }

    /// Gets the workflow executor.
    pub fn executor(&self) -> &std::sync::Arc<WorkflowExecutor> {
        &self.executor
    }

    /// Gets the state manager.
    pub fn state_manager(&self) -> &std::sync::Arc<StateManager> {
        &self.state_manager
    }

    /// Gets the version registry.
    pub fn version_registry(&self) -> &std::sync::Arc<VersionRegistry> {
        &self.version_registry
    }

    /// Gets the template registry.
    pub fn template_registry(&self) -> &std::sync::Arc<TemplateRegistry> {
        &self.template_registry
    }

    /// Executes a workflow.
    pub async fn execute(
        &self,
        workflow: &WorkflowDag,
        options: ExecutionOptions,
    ) -> WorkflowResult<String> {
        self.executor.execute(workflow, options).await
    }

    /// Gets the current version.
    pub fn version(&self) -> &str {
        VERSION
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = WorkflowEngine::new();
        assert_eq!(engine.version(), VERSION);
    }

    #[test]
    fn test_engine_builder() {
        let engine = WorkflowEngine::builder()
            .worker_pool_size(20)
            .max_queue_size(2000)
            .build();

        assert_eq!(engine.version(), VERSION);
    }

    #[tokio::test]
    async fn test_basic_workflow() {
        let mut workflow = WorkflowDag::new("test");
        let task = Task::new("task1", "test_type");
        workflow.add_task(task);

        assert!(workflow.validate().is_ok());
    }
}