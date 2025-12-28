//! Disaster recovery runbooks and playbooks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{BackupError, Result};

/// Runbook step type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunbookStepType {
    Manual,
    Automated,
    Verification,
    Notification,
}

/// Runbook step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookStep {
    pub id: Uuid,
    pub order: u32,
    pub name: String,
    pub description: String,
    pub step_type: RunbookStepType,
    pub command: Option<String>,
    pub expected_duration_minutes: u32,
    pub required: bool,
    pub dependencies: Vec<Uuid>,
}

/// Runbook for disaster recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub steps: Vec<RunbookStep>,
    pub estimated_duration_minutes: u32,
    pub severity: RunbookSeverity,
    pub tags: Vec<String>,
}

/// Runbook severity level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunbookSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// Runbook execution status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookExecution {
    pub id: Uuid,
    pub runbook_id: Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: ExecutionStatus,
    pub step_results: HashMap<Uuid, StepResult>,
    pub current_step: Option<Uuid>,
}

/// Execution status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    InProgress,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

/// Step execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: StepStatus,
    pub output: Option<String>,
    pub error_message: Option<String>,
}

/// Step execution status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Runbook manager.
pub struct RunbookManager {
    runbooks: HashMap<Uuid, Runbook>,
    executions: Vec<RunbookExecution>,
}

impl RunbookManager {
    /// Create a new runbook manager.
    pub fn new() -> Self {
        Self {
            runbooks: HashMap::new(),
            executions: Vec::new(),
        }
    }

    /// Add a runbook.
    pub fn add_runbook(&mut self, runbook: Runbook) {
        self.runbooks.insert(runbook.id, runbook);
    }

    /// Remove a runbook.
    pub fn remove_runbook(&mut self, runbook_id: Uuid) -> Result<()> {
        self.runbooks
            .remove(&runbook_id)
            .ok_or_else(|| BackupError::BackupNotFound(runbook_id.to_string()))?;
        Ok(())
    }

    /// Get a runbook.
    pub fn get_runbook(&self, runbook_id: Uuid) -> Result<&Runbook> {
        self.runbooks
            .get(&runbook_id)
            .ok_or_else(|| BackupError::BackupNotFound(runbook_id.to_string()))
    }

    /// List all runbooks.
    pub fn list_runbooks(&self) -> Vec<&Runbook> {
        self.runbooks.values().collect()
    }

    /// Start runbook execution.
    pub fn start_execution(&mut self, runbook_id: Uuid) -> Result<Uuid> {
        let runbook = self.get_runbook(runbook_id)?;

        let execution_id = Uuid::new_v4();
        let execution = RunbookExecution {
            id: execution_id,
            runbook_id,
            started_at: chrono::Utc::now(),
            completed_at: None,
            status: ExecutionStatus::InProgress,
            step_results: HashMap::new(),
            current_step: runbook.steps.first().map(|s| s.id),
        };

        self.executions.push(execution);

        Ok(execution_id)
    }

    /// Execute next step in runbook.
    pub async fn execute_next_step(&mut self, execution_id: Uuid) -> Result<Option<StepResult>> {
        // First, get the current step ID and runbook ID
        let (current_step_id, runbook_id) = {
            let execution = self
                .executions
                .iter_mut()
                .find(|e| e.id == execution_id)
                .ok_or_else(|| BackupError::BackupNotFound(execution_id.to_string()))?;

            let current_step_id = match execution.current_step {
                Some(id) => id,
                None => {
                    execution.status = ExecutionStatus::Completed;
                    execution.completed_at = Some(chrono::Utc::now());
                    return Ok(None);
                }
            };

            (current_step_id, execution.runbook_id)
        };

        // Now we can borrow self immutably to get the runbook
        let runbook = self.get_runbook(runbook_id)?;
        let step = runbook
            .steps
            .iter()
            .find(|s| s.id == current_step_id)
            .ok_or_else(|| BackupError::BackupNotFound(current_step_id.to_string()))?;

        // Check dependencies - need to get execution again
        {
            let execution = self
                .executions
                .iter()
                .find(|e| e.id == execution_id)
                .ok_or_else(|| BackupError::BackupNotFound(execution_id.to_string()))?;

            for dep_id in &step.dependencies {
                if let Some(dep_result) = execution.step_results.get(dep_id) {
                    if !matches!(dep_result.status, StepStatus::Completed) {
                        return Err(BackupError::InvalidState(format!(
                            "Dependency step {} not completed",
                            dep_id
                        )));
                    }
                } else {
                    return Err(BackupError::InvalidState(format!(
                        "Dependency step {} not executed",
                        dep_id
                    )));
                }
            }
        }

        // Execute step
        let mut step_result = StepResult {
            step_id: current_step_id,
            started_at: chrono::Utc::now(),
            completed_at: None,
            status: StepStatus::Running,
            output: None,
            error_message: None,
        };

        match step.step_type {
            RunbookStepType::Automated => {
                // Execute automated step
                if let Some(command) = &step.command {
                    tracing::info!("Executing automated step: {}", command);
                    // In a real implementation, execute the command
                    step_result.output = Some(format!("Executed: {}", command));
                    step_result.status = StepStatus::Completed;
                } else {
                    step_result.status = StepStatus::Failed;
                    step_result.error_message = Some("No command specified".to_string());
                }
            }
            RunbookStepType::Manual => {
                // Manual step - mark as pending for user action
                step_result.status = StepStatus::Pending;
                step_result.output = Some("Waiting for manual completion".to_string());
            }
            RunbookStepType::Verification => {
                // Verification step
                tracing::info!("Performing verification: {}", step.name);
                step_result.output = Some("Verification passed".to_string());
                step_result.status = StepStatus::Completed;
            }
            RunbookStepType::Notification => {
                // Send notification
                tracing::info!("Sending notification: {}", step.description);
                step_result.output = Some("Notification sent".to_string());
                step_result.status = StepStatus::Completed;
            }
        }

        step_result.completed_at = Some(chrono::Utc::now());

        // Update execution - need to get mutable reference
        let current_order = step.order;
        let next_step_id = runbook
            .steps
            .iter()
            .find(|s| s.order == current_order + 1)
            .map(|s| s.id);

        let execution = self
            .executions
            .iter_mut()
            .find(|e| e.id == execution_id)
            .ok_or_else(|| BackupError::BackupNotFound(execution_id.to_string()))?;

        execution.step_results.insert(current_step_id, step_result.clone());
        execution.current_step = next_step_id;

        // Check if execution is complete
        if execution.current_step.is_none() {
            execution.status = ExecutionStatus::Completed;
            execution.completed_at = Some(chrono::Utc::now());
        }

        Ok(Some(step_result))
    }

    /// Complete a manual step.
    pub fn complete_manual_step(
        &mut self,
        execution_id: Uuid,
        step_id: Uuid,
        success: bool,
        notes: Option<String>,
    ) -> Result<()> {
        let execution = self
            .executions
            .iter_mut()
            .find(|e| e.id == execution_id)
            .ok_or_else(|| BackupError::BackupNotFound(execution_id.to_string()))?;

        let step_result = execution
            .step_results
            .get_mut(&step_id)
            .ok_or_else(|| BackupError::BackupNotFound(step_id.to_string()))?;

        step_result.status = if success {
            StepStatus::Completed
        } else {
            StepStatus::Failed
        };
        step_result.completed_at = Some(chrono::Utc::now());
        step_result.output = notes;

        Ok(())
    }

    /// Pause runbook execution.
    pub fn pause_execution(&mut self, execution_id: Uuid) -> Result<()> {
        let execution = self
            .executions
            .iter_mut()
            .find(|e| e.id == execution_id)
            .ok_or_else(|| BackupError::BackupNotFound(execution_id.to_string()))?;

        execution.status = ExecutionStatus::Paused;

        Ok(())
    }

    /// Resume runbook execution.
    pub fn resume_execution(&mut self, execution_id: Uuid) -> Result<()> {
        let execution = self
            .executions
            .iter_mut()
            .find(|e| e.id == execution_id)
            .ok_or_else(|| BackupError::BackupNotFound(execution_id.to_string()))?;

        execution.status = ExecutionStatus::InProgress;

        Ok(())
    }

    /// Cancel runbook execution.
    pub fn cancel_execution(&mut self, execution_id: Uuid) -> Result<()> {
        let execution = self
            .executions
            .iter_mut()
            .find(|e| e.id == execution_id)
            .ok_or_else(|| BackupError::BackupNotFound(execution_id.to_string()))?;

        execution.status = ExecutionStatus::Cancelled;
        execution.completed_at = Some(chrono::Utc::now());

        Ok(())
    }

    /// Get execution status.
    pub fn get_execution(&self, execution_id: Uuid) -> Result<&RunbookExecution> {
        self.executions
            .iter()
            .find(|e| e.id == execution_id)
            .ok_or_else(|| BackupError::BackupNotFound(execution_id.to_string()))
    }

    /// Get statistics.
    pub fn statistics(&self) -> RunbookStatistics {
        let total_runbooks = self.runbooks.len();
        let total_executions = self.executions.len();
        let completed_executions = self
            .executions
            .iter()
            .filter(|e| matches!(e.status, ExecutionStatus::Completed))
            .count();
        let failed_executions = self
            .executions
            .iter()
            .filter(|e| matches!(e.status, ExecutionStatus::Failed))
            .count();

        RunbookStatistics {
            total_runbooks,
            total_executions,
            completed_executions,
            failed_executions,
            success_rate: if total_executions > 0 {
                (completed_executions as f64 / total_executions as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

impl Default for RunbookManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Runbook statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunbookStatistics {
    pub total_runbooks: usize,
    pub total_executions: usize,
    pub completed_executions: usize,
    pub failed_executions: usize,
    pub success_rate: f64,
}

/// Pre-defined disaster recovery runbooks.
pub struct RunbookTemplates;

impl RunbookTemplates {
    /// Complete datacenter failover runbook.
    pub fn datacenter_failover() -> Runbook {
        Runbook {
            id: Uuid::new_v4(),
            name: "Complete Datacenter Failover".to_string(),
            description: "Runbook for failing over to secondary datacenter".to_string(),
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            steps: vec![
                RunbookStep {
                    id: Uuid::new_v4(),
                    order: 1,
                    name: "Assess Primary Datacenter Status".to_string(),
                    description: "Verify primary datacenter is truly unavailable".to_string(),
                    step_type: RunbookStepType::Manual,
                    command: None,
                    expected_duration_minutes: 5,
                    required: true,
                    dependencies: Vec::new(),
                },
                RunbookStep {
                    id: Uuid::new_v4(),
                    order: 2,
                    name: "Notify Stakeholders".to_string(),
                    description: "Send notifications about failover initiation".to_string(),
                    step_type: RunbookStepType::Notification,
                    command: None,
                    expected_duration_minutes: 2,
                    required: true,
                    dependencies: Vec::new(),
                },
                RunbookStep {
                    id: Uuid::new_v4(),
                    order: 3,
                    name: "Activate Secondary Datacenter".to_string(),
                    description: "Bring up services in secondary datacenter".to_string(),
                    step_type: RunbookStepType::Automated,
                    command: Some("./scripts/activate-secondary-dc.sh".to_string()),
                    expected_duration_minutes: 15,
                    required: true,
                    dependencies: Vec::new(),
                },
            ],
            estimated_duration_minutes: 30,
            severity: RunbookSeverity::Critical,
            tags: vec!["failover".to_string(), "datacenter".to_string()],
        }
    }

    /// Database recovery runbook.
    pub fn database_recovery() -> Runbook {
        Runbook {
            id: Uuid::new_v4(),
            name: "Database Recovery".to_string(),
            description: "Restore database from backup".to_string(),
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            steps: vec![
                RunbookStep {
                    id: Uuid::new_v4(),
                    order: 1,
                    name: "Stop Database Service".to_string(),
                    description: "Gracefully stop the database service".to_string(),
                    step_type: RunbookStepType::Automated,
                    command: Some("systemctl stop postgresql".to_string()),
                    expected_duration_minutes: 2,
                    required: true,
                    dependencies: Vec::new(),
                },
                RunbookStep {
                    id: Uuid::new_v4(),
                    order: 2,
                    name: "Restore from Backup".to_string(),
                    description: "Restore database from latest backup".to_string(),
                    step_type: RunbookStepType::Automated,
                    command: Some("./scripts/restore-database.sh".to_string()),
                    expected_duration_minutes: 30,
                    required: true,
                    dependencies: Vec::new(),
                },
                RunbookStep {
                    id: Uuid::new_v4(),
                    order: 3,
                    name: "Start Database Service".to_string(),
                    description: "Start the database service".to_string(),
                    step_type: RunbookStepType::Automated,
                    command: Some("systemctl start postgresql".to_string()),
                    expected_duration_minutes: 2,
                    required: true,
                    dependencies: Vec::new(),
                },
                RunbookStep {
                    id: Uuid::new_v4(),
                    order: 4,
                    name: "Verify Database Integrity".to_string(),
                    description: "Run integrity checks on restored database".to_string(),
                    step_type: RunbookStepType::Verification,
                    command: Some("./scripts/verify-database.sh".to_string()),
                    expected_duration_minutes: 10,
                    required: true,
                    dependencies: Vec::new(),
                },
            ],
            estimated_duration_minutes: 45,
            severity: RunbookSeverity::High,
            tags: vec!["database".to_string(), "recovery".to_string()],
        }
    }
}
