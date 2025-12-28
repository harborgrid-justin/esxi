//! Data stewardship workflows and responsibilities

use crate::error::{GovernanceError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Data stewardship manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StewardshipManager {
    /// Data stewards
    stewards: HashMap<String, DataSteward>,
    /// Stewardship assignments
    assignments: HashMap<String, StewardshipAssignment>,
    /// Workflow tasks
    tasks: HashMap<String, StewardshipTask>,
}

/// Data steward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSteward {
    /// Steward identifier
    pub id: String,
    /// Steward name
    pub name: String,
    /// Email address
    pub email: String,
    /// Department
    pub department: String,
    /// Stewardship domains
    pub domains: Vec<String>,
    /// Responsibilities
    pub responsibilities: Vec<Responsibility>,
    /// Active status
    pub active: bool,
    /// Delegate steward (for backup)
    pub delegate: Option<String>,
}

/// Stewardship responsibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Responsibility {
    DataQuality,
    Metadata,
    Classification,
    Compliance,
    AccessControl,
    DataLifecycle,
    Documentation,
    IssueResolution,
}

/// Stewardship assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StewardshipAssignment {
    /// Assignment identifier
    pub id: String,
    /// Data asset identifier (dataset, table, etc.)
    pub asset_id: String,
    /// Asset type
    pub asset_type: AssetType,
    /// Primary steward
    pub primary_steward: String,
    /// Secondary stewards
    pub secondary_stewards: Vec<String>,
    /// Assignment date
    pub assigned_at: DateTime<Utc>,
    /// Assignment status
    pub status: AssignmentStatus,
    /// Notes
    pub notes: Option<String>,
}

/// Asset type for stewardship
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssetType {
    Dataset,
    Table,
    Schema,
    Domain,
    Application,
}

/// Assignment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssignmentStatus {
    Active,
    PendingTransfer,
    Inactive,
}

/// Stewardship task/workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StewardshipTask {
    /// Task identifier
    pub id: String,
    /// Task type
    pub task_type: TaskType,
    /// Task title
    pub title: String,
    /// Task description
    pub description: String,
    /// Related asset
    pub asset_id: String,
    /// Assigned steward
    pub assigned_to: String,
    /// Task priority
    pub priority: Priority,
    /// Task status
    pub status: TaskStatus,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Due date
    pub due_date: Option<DateTime<Utc>>,
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Resolution notes
    pub resolution: Option<String>,
    /// Task metadata
    pub metadata: HashMap<String, String>,
}

/// Task type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskType {
    /// Review and update metadata
    MetadataReview,
    /// Classify data sensitivity
    ClassificationReview,
    /// Address data quality issue
    QualityIssue,
    /// Review access requests
    AccessReview,
    /// Certify data accuracy
    DataCertification,
    /// Respond to data subject request
    SubjectRequest,
    /// Document business rules
    Documentation,
    /// Review retention policy
    RetentionReview,
    /// Custom task
    Custom(String),
}

/// Task priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Open,
    InProgress,
    Blocked,
    Completed,
    Cancelled,
}

/// Stewardship metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StewardshipMetrics {
    /// Steward identifier
    pub steward_id: String,
    /// Number of assigned assets
    pub total_assignments: usize,
    /// Active tasks
    pub active_tasks: usize,
    /// Completed tasks
    pub completed_tasks: usize,
    /// Overdue tasks
    pub overdue_tasks: usize,
    /// Average task completion time (in days)
    pub avg_completion_time: f64,
    /// Task completion rate
    pub completion_rate: f64,
}

impl StewardshipManager {
    /// Create a new stewardship manager
    pub fn new() -> Self {
        Self {
            stewards: HashMap::new(),
            assignments: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    /// Register a data steward
    pub fn register_steward(&mut self, steward: DataSteward) -> Result<()> {
        if self.stewards.contains_key(&steward.id) {
            return Err(GovernanceError::Stewardship(format!(
                "Steward already exists: {}",
                steward.id
            )));
        }

        self.stewards.insert(steward.id.clone(), steward);
        Ok(())
    }

    /// Get a data steward
    pub fn get_steward(&self, steward_id: &str) -> Result<&DataSteward> {
        self.stewards.get(steward_id).ok_or_else(|| {
            GovernanceError::Stewardship(format!("Steward not found: {}", steward_id))
        })
    }

    /// Update a data steward
    pub fn update_steward(&mut self, steward: DataSteward) -> Result<()> {
        if !self.stewards.contains_key(&steward.id) {
            return Err(GovernanceError::Stewardship(format!(
                "Steward not found: {}",
                steward.id
            )));
        }

        self.stewards.insert(steward.id.clone(), steward);
        Ok(())
    }

    /// Assign stewardship for an asset
    pub fn assign_stewardship(
        &mut self,
        asset_id: String,
        asset_type: AssetType,
        primary_steward: String,
        secondary_stewards: Vec<String>,
    ) -> Result<String> {
        // Verify primary steward exists
        if !self.stewards.contains_key(&primary_steward) {
            return Err(GovernanceError::Stewardship(format!(
                "Primary steward not found: {}",
                primary_steward
            )));
        }

        // Verify secondary stewards exist
        for steward_id in &secondary_stewards {
            if !self.stewards.contains_key(steward_id) {
                return Err(GovernanceError::Stewardship(format!(
                    "Secondary steward not found: {}",
                    steward_id
                )));
            }
        }

        let assignment = StewardshipAssignment {
            id: Uuid::new_v4().to_string(),
            asset_id: asset_id.clone(),
            asset_type,
            primary_steward,
            secondary_stewards,
            assigned_at: Utc::now(),
            status: AssignmentStatus::Active,
            notes: None,
        };

        let id = assignment.id.clone();
        self.assignments.insert(asset_id, assignment);
        Ok(id)
    }

    /// Get stewardship assignment for an asset
    pub fn get_assignment(&self, asset_id: &str) -> Result<&StewardshipAssignment> {
        self.assignments.get(asset_id).ok_or_else(|| {
            GovernanceError::Stewardship(format!("No stewardship assigned for asset: {}", asset_id))
        })
    }

    /// Transfer stewardship to another steward
    pub fn transfer_stewardship(
        &mut self,
        asset_id: &str,
        new_steward: String,
    ) -> Result<()> {
        // Verify new steward exists
        if !self.stewards.contains_key(&new_steward) {
            return Err(GovernanceError::Stewardship(format!(
                "Steward not found: {}",
                new_steward
            )));
        }

        let assignment = self.assignments.get_mut(asset_id).ok_or_else(|| {
            GovernanceError::Stewardship(format!("No stewardship assigned for asset: {}", asset_id))
        })?;

        assignment.primary_steward = new_steward;
        assignment.status = AssignmentStatus::Active;

        Ok(())
    }

    /// Create a stewardship task
    pub fn create_task(&mut self, mut task: StewardshipTask) -> Result<String> {
        // Verify steward exists
        if !self.stewards.contains_key(&task.assigned_to) {
            return Err(GovernanceError::Stewardship(format!(
                "Steward not found: {}",
                task.assigned_to
            )));
        }

        if task.id.is_empty() {
            task.id = Uuid::new_v4().to_string();
        }

        let id = task.id.clone();
        self.tasks.insert(id.clone(), task);
        Ok(id)
    }

    /// Get a task
    pub fn get_task(&self, task_id: &str) -> Result<&StewardshipTask> {
        self.tasks.get(task_id).ok_or_else(|| {
            GovernanceError::Stewardship(format!("Task not found: {}", task_id))
        })
    }

    /// Update task status
    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<()> {
        let task = self.tasks.get_mut(task_id).ok_or_else(|| {
            GovernanceError::Stewardship(format!("Task not found: {}", task_id))
        })?;

        task.status = status.clone();

        if status == TaskStatus::Completed {
            task.completed_at = Some(Utc::now());
        }

        Ok(())
    }

    /// Complete a task with resolution notes
    pub fn complete_task(&mut self, task_id: &str, resolution: String) -> Result<()> {
        let task = self.tasks.get_mut(task_id).ok_or_else(|| {
            GovernanceError::Stewardship(format!("Task not found: {}", task_id))
        })?;

        task.status = TaskStatus::Completed;
        task.completed_at = Some(Utc::now());
        task.resolution = Some(resolution);

        Ok(())
    }

    /// Get tasks assigned to a steward
    pub fn get_steward_tasks(&self, steward_id: &str) -> Vec<&StewardshipTask> {
        self.tasks
            .values()
            .filter(|task| task.assigned_to == steward_id)
            .collect()
    }

    /// Get open tasks for a steward
    pub fn get_open_tasks(&self, steward_id: &str) -> Vec<&StewardshipTask> {
        self.tasks
            .values()
            .filter(|task| {
                task.assigned_to == steward_id
                    && (task.status == TaskStatus::Open || task.status == TaskStatus::InProgress)
            })
            .collect()
    }

    /// Get overdue tasks for a steward
    pub fn get_overdue_tasks(&self, steward_id: &str) -> Vec<&StewardshipTask> {
        let now = Utc::now();
        self.tasks
            .values()
            .filter(|task| {
                task.assigned_to == steward_id
                    && task.status != TaskStatus::Completed
                    && task.status != TaskStatus::Cancelled
                    && task.due_date.map(|due| due < now).unwrap_or(false)
            })
            .collect()
    }

    /// Get assets assigned to a steward
    pub fn get_steward_assets(&self, steward_id: &str) -> Vec<&StewardshipAssignment> {
        self.assignments
            .values()
            .filter(|assignment| {
                assignment.primary_steward == steward_id
                    || assignment.secondary_stewards.contains(&steward_id.to_string())
            })
            .collect()
    }

    /// Calculate stewardship metrics for a steward
    pub fn calculate_metrics(&self, steward_id: &str) -> Result<StewardshipMetrics> {
        let assignments = self.get_steward_assets(steward_id);
        let all_tasks = self.get_steward_tasks(steward_id);
        let active_tasks = all_tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Open || t.status == TaskStatus::InProgress)
            .count();
        let completed_tasks = all_tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let overdue_tasks = self.get_overdue_tasks(steward_id).len();

        // Calculate average completion time
        let mut total_completion_time = 0i64;
        let mut completion_count = 0;

        for task in all_tasks.iter() {
            if let Some(completed_at) = task.completed_at {
                let duration = (completed_at - task.created_at).num_days();
                total_completion_time += duration;
                completion_count += 1;
            }
        }

        let avg_completion_time = if completion_count > 0 {
            total_completion_time as f64 / completion_count as f64
        } else {
            0.0
        };

        let total_tasks = all_tasks.len();
        let completion_rate = if total_tasks > 0 {
            completed_tasks as f64 / total_tasks as f64
        } else {
            0.0
        };

        Ok(StewardshipMetrics {
            steward_id: steward_id.to_string(),
            total_assignments: assignments.len(),
            active_tasks,
            completed_tasks,
            overdue_tasks,
            avg_completion_time,
            completion_rate,
        })
    }

    /// List all stewards
    pub fn list_stewards(&self) -> Vec<&DataSteward> {
        self.stewards.values().collect()
    }

    /// List active stewards
    pub fn list_active_stewards(&self) -> Vec<&DataSteward> {
        self.stewards
            .values()
            .filter(|s| s.active)
            .collect()
    }

    /// List all tasks
    pub fn list_tasks(&self) -> Vec<&StewardshipTask> {
        self.tasks.values().collect()
    }
}

impl Default for StewardshipManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stewardship_manager_creation() {
        let manager = StewardshipManager::new();
        assert_eq!(manager.list_stewards().len(), 0);
    }

    #[test]
    fn test_register_steward() {
        let mut manager = StewardshipManager::new();
        let steward = DataSteward {
            id: "steward1".to_string(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            department: "Data Team".to_string(),
            domains: vec!["customer".to_string()],
            responsibilities: vec![Responsibility::DataQuality],
            active: true,
            delegate: None,
        };

        manager.register_steward(steward).unwrap();
        assert_eq!(manager.list_stewards().len(), 1);
    }
}
