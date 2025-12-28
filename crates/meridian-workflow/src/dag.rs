//! DAG-based workflow definition and validation.

use crate::error::{WorkflowError, WorkflowResult};
use daggy::{Dag, NodeIndex, Walker};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

/// A unique identifier for a task in the workflow.
pub type TaskId = Uuid;

/// A unique identifier for a workflow.
pub type WorkflowId = Uuid;

/// Task definition in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier.
    pub id: TaskId,

    /// Human-readable task name.
    pub name: String,

    /// Task type or category.
    pub task_type: String,

    /// Task-specific configuration.
    pub config: serde_json::Value,

    /// Maximum execution time in seconds.
    pub timeout_secs: Option<u64>,

    /// Whether this task can be retried on failure.
    pub retryable: bool,

    /// Maximum number of retry attempts.
    pub max_retries: Option<u32>,

    /// Task priority (higher values = higher priority).
    pub priority: i32,

    /// Custom metadata.
    pub metadata: HashMap<String, String>,
}

impl Task {
    /// Creates a new task with the given name and type.
    pub fn new(name: impl Into<String>, task_type: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            task_type: task_type.into(),
            config: serde_json::Value::Null,
            timeout_secs: None,
            retryable: true,
            max_retries: Some(3),
            priority: 0,
            metadata: HashMap::new(),
        }
    }

    /// Sets the task configuration.
    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = config;
        self
    }

    /// Sets the task timeout.
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }

    /// Sets the retry policy.
    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.retryable = true;
        self.max_retries = Some(max_retries);
        self
    }

    /// Sets the task priority.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Edge type representing task dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Sequential dependency - wait for completion.
    Sequential,

    /// Conditional dependency - wait only if condition is true.
    Conditional { condition: String },

    /// Data dependency - pass output from parent to child.
    Data { mapping: HashMap<String, String> },
}

/// Workflow definition using a DAG structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDag {
    /// Unique workflow identifier.
    pub id: WorkflowId,

    /// Workflow name.
    pub name: String,

    /// Workflow description.
    pub description: Option<String>,

    /// Workflow version.
    pub version: u32,

    /// Tasks in the workflow.
    tasks: Vec<Task>,

    /// Dependencies between tasks (from_task_id, to_task_id, dependency_type).
    dependencies: Vec<(TaskId, TaskId, DependencyType)>,

    /// Custom metadata.
    pub metadata: HashMap<String, String>,
}

impl WorkflowDag {
    /// Creates a new workflow DAG.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: None,
            version: 1,
            tasks: Vec::new(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Adds a task to the workflow.
    pub fn add_task(&mut self, task: Task) -> TaskId {
        let task_id = task.id;
        self.tasks.push(task);
        task_id
    }

    /// Adds a dependency between two tasks.
    pub fn add_dependency(
        &mut self,
        from: TaskId,
        to: TaskId,
        dep_type: DependencyType,
    ) -> WorkflowResult<()> {
        // Verify both tasks exist
        if !self.tasks.iter().any(|t| t.id == from) {
            return Err(WorkflowError::InvalidDefinition(format!(
                "Task {} not found",
                from
            )));
        }
        if !self.tasks.iter().any(|t| t.id == to) {
            return Err(WorkflowError::InvalidDefinition(format!(
                "Task {} not found",
                to
            )));
        }

        self.dependencies.push((from, to, dep_type));
        Ok(())
    }

    /// Validates the workflow DAG.
    pub fn validate(&self) -> WorkflowResult<()> {
        // Check for at least one task
        if self.tasks.is_empty() {
            return Err(WorkflowError::InvalidDefinition(
                "Workflow must contain at least one task".to_string(),
            ));
        }

        // Build the DAG and check for cycles
        let dag = self.build_internal_dag()?;

        // Verify no cycles exist
        if self.has_cycle(&dag) {
            return Err(WorkflowError::CycleDetected);
        }

        Ok(())
    }

    /// Builds the internal daggy DAG structure.
    fn build_internal_dag(&self) -> WorkflowResult<Dag<TaskId, DependencyType>> {
        let mut dag = Dag::new();
        let mut node_map: HashMap<TaskId, NodeIndex> = HashMap::new();

        // Add nodes
        for task in &self.tasks {
            let node = dag.add_node(task.id);
            node_map.insert(task.id, node);
        }

        // Add edges
        for (from, to, dep_type) in &self.dependencies {
            let from_node = node_map.get(from).ok_or_else(|| {
                WorkflowError::InvalidDefinition(format!("Task {} not found", from))
            })?;
            let to_node = node_map.get(to).ok_or_else(|| {
                WorkflowError::InvalidDefinition(format!("Task {} not found", to))
            })?;

            dag.add_edge(*from_node, *to_node, dep_type.clone())
                .map_err(|_| WorkflowError::CycleDetected)?;
        }

        Ok(dag)
    }

    /// Checks if the DAG contains a cycle.
    fn has_cycle(&self, dag: &Dag<TaskId, DependencyType>) -> bool {
        // Use topological sort to detect cycles
        daggy::petgraph::algo::toposort(dag, None).is_err()
    }

    /// Returns tasks in topological order.
    pub fn topological_sort(&self) -> WorkflowResult<Vec<TaskId>> {
        let dag = self.build_internal_dag()?;

        let sorted = daggy::petgraph::algo::toposort(&dag, None)
            .map_err(|_| WorkflowError::CycleDetected)?;

        Ok(sorted.into_iter().map(|idx| dag[idx]).collect())
    }

    /// Returns all root tasks (tasks with no dependencies).
    pub fn root_tasks(&self) -> Vec<&Task> {
        let dependent_tasks: HashSet<TaskId> =
            self.dependencies.iter().map(|(_, to, _)| *to).collect();

        self.tasks
            .iter()
            .filter(|task| !dependent_tasks.contains(&task.id))
            .collect()
    }

    /// Returns all leaf tasks (tasks with no dependents).
    pub fn leaf_tasks(&self) -> Vec<&Task> {
        let dependency_tasks: HashSet<TaskId> =
            self.dependencies.iter().map(|(from, _, _)| *from).collect();

        self.tasks
            .iter()
            .filter(|task| !dependency_tasks.contains(&task.id))
            .collect()
    }

    /// Gets a task by ID.
    pub fn get_task(&self, task_id: TaskId) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == task_id)
    }

    /// Gets the dependencies for a task.
    pub fn get_task_dependencies(&self, task_id: TaskId) -> Vec<(TaskId, &DependencyType)> {
        self.dependencies
            .iter()
            .filter(|(_, to, _)| *to == task_id)
            .map(|(from, _, dep_type)| (*from, dep_type))
            .collect()
    }

    /// Gets the dependents of a task.
    pub fn get_task_dependents(&self, task_id: TaskId) -> Vec<(TaskId, &DependencyType)> {
        self.dependencies
            .iter()
            .filter(|(from, _, _)| *from == task_id)
            .map(|(_, to, dep_type)| (*to, dep_type))
            .collect()
    }

    /// Returns all tasks.
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Returns mutable access to all tasks.
    pub fn tasks_mut(&mut self) -> &mut [Task] {
        &mut self.tasks
    }

    /// Returns all dependencies.
    pub fn dependencies(&self) -> &[(TaskId, TaskId, DependencyType)] {
        &self.dependencies
    }

    /// Gets tasks that can be executed in parallel at the current state.
    pub fn get_parallel_tasks(&self, completed: &HashSet<TaskId>) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|task| {
                // Task must not be completed
                if completed.contains(&task.id) {
                    return false;
                }

                // All dependencies must be satisfied
                let dependencies = self.get_task_dependencies(task.id);
                dependencies.iter().all(|(dep_id, _)| completed.contains(dep_id))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_creation() {
        let mut workflow = WorkflowDag::new("test_workflow");
        let task1 = Task::new("task1", "processing");
        let task2 = Task::new("task2", "analysis");

        let id1 = workflow.add_task(task1);
        let id2 = workflow.add_task(task2);

        workflow
            .add_dependency(id1, id2, DependencyType::Sequential)
            .unwrap();

        assert!(workflow.validate().is_ok());
    }

    #[test]
    fn test_cycle_detection() {
        let mut workflow = WorkflowDag::new("cyclic_workflow");
        let task1 = Task::new("task1", "processing");
        let task2 = Task::new("task2", "analysis");

        let id1 = workflow.add_task(task1);
        let id2 = workflow.add_task(task2);

        workflow
            .add_dependency(id1, id2, DependencyType::Sequential)
            .unwrap();
        workflow
            .add_dependency(id2, id1, DependencyType::Sequential)
            .unwrap();

        assert!(matches!(workflow.validate(), Err(WorkflowError::CycleDetected)));
    }

    #[test]
    fn test_topological_sort() {
        let mut workflow = WorkflowDag::new("test_workflow");
        let task1 = Task::new("task1", "processing");
        let task2 = Task::new("task2", "analysis");
        let task3 = Task::new("task3", "finalize");

        let id1 = workflow.add_task(task1);
        let id2 = workflow.add_task(task2);
        let id3 = workflow.add_task(task3);

        workflow
            .add_dependency(id1, id2, DependencyType::Sequential)
            .unwrap();
        workflow
            .add_dependency(id2, id3, DependencyType::Sequential)
            .unwrap();

        let sorted = workflow.topological_sort().unwrap();
        assert_eq!(sorted, vec![id1, id2, id3]);
    }
}
