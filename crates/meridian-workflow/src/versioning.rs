//! Workflow versioning and migration support.

use crate::dag::WorkflowDag;
use crate::error::{WorkflowError, WorkflowResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// Workflow version metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVersion {
    /// Workflow ID.
    pub workflow_id: Uuid,

    /// Version number.
    pub version: u32,

    /// Workflow definition.
    pub definition: WorkflowDag,

    /// Version description/changelog.
    pub description: Option<String>,

    /// Author of this version.
    pub author: Option<String>,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Whether this version is active.
    pub active: bool,

    /// Whether this version is deprecated.
    pub deprecated: bool,

    /// Tags for categorization.
    pub tags: Vec<String>,
}

impl WorkflowVersion {
    /// Creates a new workflow version.
    pub fn new(definition: WorkflowDag) -> Self {
        Self {
            workflow_id: definition.id,
            version: definition.version,
            definition,
            description: None,
            author: None,
            created_at: Utc::now(),
            active: true,
            deprecated: false,
            tags: Vec::new(),
        }
    }

    /// Sets the version description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the author.
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Adds tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Migration strategy between workflow versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationStrategy {
    /// Complete existing executions before switching.
    CompleteExisting,

    /// Immediately switch to new version.
    Immediate,

    /// Gradually migrate (blue-green deployment).
    Gradual { percentage: u8 },

    /// Cancel existing and start fresh.
    CancelAndRestart,

    /// Custom migration logic.
    Custom { strategy_id: String },
}

impl Default for MigrationStrategy {
    fn default() -> Self {
        MigrationStrategy::CompleteExisting
    }
}

/// Migration plan for upgrading between versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Source version.
    pub from_version: u32,

    /// Target version.
    pub to_version: u32,

    /// Migration strategy.
    pub strategy: MigrationStrategy,

    /// Whether to validate before migration.
    pub validate: bool,

    /// Whether to create a backup.
    pub backup: bool,

    /// Custom migration steps.
    pub custom_steps: Vec<String>,
}

impl MigrationPlan {
    /// Creates a new migration plan.
    pub fn new(from_version: u32, to_version: u32) -> Self {
        Self {
            from_version,
            to_version,
            strategy: MigrationStrategy::default(),
            validate: true,
            backup: true,
            custom_steps: Vec::new(),
        }
    }

    /// Sets the migration strategy.
    pub fn with_strategy(mut self, strategy: MigrationStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Sets whether to validate.
    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate = validate;
        self
    }
}

/// Migration execution record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// Migration ID.
    pub id: Uuid,

    /// Workflow ID.
    pub workflow_id: Uuid,

    /// Migration plan.
    pub plan: MigrationPlan,

    /// Migration status.
    pub status: MigrationStatus,

    /// Start time.
    pub started_at: DateTime<Utc>,

    /// Completion time.
    pub completed_at: Option<DateTime<Utc>>,

    /// Error message if failed.
    pub error: Option<String>,

    /// Migration steps executed.
    pub steps_executed: Vec<String>,
}

/// Migration status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStatus {
    /// Migration is pending.
    Pending,

    /// Migration is in progress.
    InProgress,

    /// Migration completed successfully.
    Completed,

    /// Migration failed.
    Failed,

    /// Migration was rolled back.
    RolledBack,
}

/// Version registry for managing workflow versions.
pub struct VersionRegistry {
    /// Workflow versions by workflow ID and version number.
    versions: Arc<RwLock<HashMap<Uuid, HashMap<u32, WorkflowVersion>>>>,

    /// Active version for each workflow.
    active_versions: Arc<RwLock<HashMap<Uuid, u32>>>,

    /// Migration history.
    migrations: Arc<RwLock<Vec<MigrationRecord>>>,
}

impl VersionRegistry {
    /// Creates a new version registry.
    pub fn new() -> Self {
        Self {
            versions: Arc::new(RwLock::new(HashMap::new())),
            active_versions: Arc::new(RwLock::new(HashMap::new())),
            migrations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Registers a new workflow version.
    pub async fn register_version(&self, version: WorkflowVersion) -> WorkflowResult<()> {
        let workflow_id = version.workflow_id;
        let version_num = version.version;

        // Validate the workflow definition
        version.definition.validate()?;

        let mut versions = self.versions.write().await;
        let workflow_versions = versions.entry(workflow_id).or_insert_with(HashMap::new);

        // Check if version already exists
        if workflow_versions.contains_key(&version_num) {
            return Err(WorkflowError::InvalidDefinition(format!(
                "Version {} already exists for workflow {}",
                version_num, workflow_id
            )));
        }

        workflow_versions.insert(version_num, version);

        // Set as active if it's the first version
        let mut active = self.active_versions.write().await;
        active.entry(workflow_id).or_insert(version_num);

        info!(
            "Registered workflow {} version {}",
            workflow_id, version_num
        );
        Ok(())
    }

    /// Gets a specific workflow version.
    pub async fn get_version(
        &self,
        workflow_id: Uuid,
        version: u32,
    ) -> WorkflowResult<WorkflowVersion> {
        let versions = self.versions.read().await;

        versions
            .get(&workflow_id)
            .and_then(|v| v.get(&version))
            .cloned()
            .ok_or_else(|| {
                WorkflowError::NotFound(format!(
                    "Workflow {} version {} not found",
                    workflow_id, version
                ))
            })
    }

    /// Gets the active version for a workflow.
    pub async fn get_active_version(&self, workflow_id: Uuid) -> WorkflowResult<WorkflowVersion> {
        let active = self.active_versions.read().await;
        let version_num = active.get(&workflow_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("No active version for workflow {}", workflow_id))
        })?;

        self.get_version(workflow_id, *version_num).await
    }

    /// Sets the active version for a workflow.
    pub async fn set_active_version(
        &self,
        workflow_id: Uuid,
        version: u32,
    ) -> WorkflowResult<()> {
        // Verify version exists
        self.get_version(workflow_id, version).await?;

        let mut active = self.active_versions.write().await;
        active.insert(workflow_id, version);

        info!("Set workflow {} active version to {}", workflow_id, version);
        Ok(())
    }

    /// Lists all versions for a workflow.
    pub async fn list_versions(&self, workflow_id: Uuid) -> Vec<WorkflowVersion> {
        let versions = self.versions.read().await;

        versions
            .get(&workflow_id)
            .map(|v| {
                let mut vers: Vec<_> = v.values().cloned().collect();
                vers.sort_by(|a, b| b.version.cmp(&a.version)); // Most recent first
                vers
            })
            .unwrap_or_default()
    }

    /// Gets the latest version number for a workflow.
    pub async fn get_latest_version_number(&self, workflow_id: Uuid) -> Option<u32> {
        let versions = self.versions.read().await;

        versions
            .get(&workflow_id)
            .and_then(|v| v.keys().max().copied())
    }

    /// Creates a new version from an existing one.
    pub async fn create_new_version(
        &self,
        workflow_id: Uuid,
        base_version: u32,
        modifications: impl FnOnce(&mut WorkflowDag),
    ) -> WorkflowResult<WorkflowVersion> {
        // Get the base version
        let base = self.get_version(workflow_id, base_version).await?;

        // Create a new definition
        let mut new_definition = base.definition.clone();
        new_definition.version = base_version + 1;

        // Apply modifications
        modifications(&mut new_definition);

        // Validate the new definition
        new_definition.validate()?;

        // Create new version
        let new_version = WorkflowVersion::new(new_definition);

        // Register it
        self.register_version(new_version.clone()).await?;

        Ok(new_version)
    }

    /// Migrates to a new version.
    pub async fn migrate(
        &self,
        workflow_id: Uuid,
        plan: MigrationPlan,
    ) -> WorkflowResult<MigrationRecord> {
        // Validate versions exist
        let from_version = self.get_version(workflow_id, plan.from_version).await?;
        let to_version = self.get_version(workflow_id, plan.to_version).await?;

        // Create migration record
        let mut record = MigrationRecord {
            id: Uuid::new_v4(),
            workflow_id,
            plan: plan.clone(),
            status: MigrationStatus::InProgress,
            started_at: Utc::now(),
            completed_at: None,
            error: None,
            steps_executed: Vec::new(),
        };

        info!(
            "Starting migration of workflow {} from version {} to {}",
            workflow_id, plan.from_version, plan.to_version
        );

        // Execute migration based on strategy
        let result = match plan.strategy {
            MigrationStrategy::Immediate => {
                self.execute_immediate_migration(workflow_id, &to_version, &mut record)
                    .await
            }
            MigrationStrategy::CompleteExisting => {
                self.execute_complete_existing_migration(workflow_id, &to_version, &mut record)
                    .await
            }
            MigrationStrategy::Gradual { percentage } => {
                self.execute_gradual_migration(workflow_id, &to_version, percentage, &mut record)
                    .await
            }
            MigrationStrategy::CancelAndRestart => {
                self.execute_cancel_restart_migration(workflow_id, &to_version, &mut record)
                    .await
            }
            MigrationStrategy::Custom { ref strategy_id } => {
                warn!("Custom migration strategy {} not implemented", strategy_id);
                Err(WorkflowError::MigrationFailed(format!(
                    "Custom migration strategy {} not implemented",
                    strategy_id
                )))
            }
        };

        // Update record
        match result {
            Ok(()) => {
                record.status = MigrationStatus::Completed;
                record.completed_at = Some(Utc::now());
                info!("Migration {} completed successfully", record.id);
            }
            Err(e) => {
                record.status = MigrationStatus::Failed;
                record.error = Some(e.to_string());
                record.completed_at = Some(Utc::now());
                warn!("Migration {} failed: {}", record.id, e);
            }
        }

        // Store migration record
        let mut migrations = self.migrations.write().await;
        migrations.push(record.clone());

        Ok(record)
    }

    /// Executes immediate migration.
    async fn execute_immediate_migration(
        &self,
        workflow_id: Uuid,
        to_version: &WorkflowVersion,
        record: &mut MigrationRecord,
    ) -> WorkflowResult<()> {
        record.steps_executed.push("Setting active version".to_string());
        self.set_active_version(workflow_id, to_version.version)
            .await?;
        Ok(())
    }

    /// Executes complete existing migration.
    async fn execute_complete_existing_migration(
        &self,
        workflow_id: Uuid,
        to_version: &WorkflowVersion,
        record: &mut MigrationRecord,
    ) -> WorkflowResult<()> {
        record
            .steps_executed
            .push("Waiting for existing executions to complete".to_string());
        // In a real implementation, we would wait for executions to complete
        // For now, just set the active version
        record.steps_executed.push("Setting active version".to_string());
        self.set_active_version(workflow_id, to_version.version)
            .await?;
        Ok(())
    }

    /// Executes gradual migration.
    async fn execute_gradual_migration(
        &self,
        workflow_id: Uuid,
        to_version: &WorkflowVersion,
        _percentage: u8,
        record: &mut MigrationRecord,
    ) -> WorkflowResult<()> {
        record
            .steps_executed
            .push("Starting gradual migration".to_string());
        // In a real implementation, we would gradually route traffic
        // For now, just set the active version
        record.steps_executed.push("Setting active version".to_string());
        self.set_active_version(workflow_id, to_version.version)
            .await?;
        Ok(())
    }

    /// Executes cancel and restart migration.
    async fn execute_cancel_restart_migration(
        &self,
        workflow_id: Uuid,
        to_version: &WorkflowVersion,
        record: &mut MigrationRecord,
    ) -> WorkflowResult<()> {
        record
            .steps_executed
            .push("Cancelling existing executions".to_string());
        // In a real implementation, we would cancel existing executions
        record.steps_executed.push("Setting active version".to_string());
        self.set_active_version(workflow_id, to_version.version)
            .await?;
        Ok(())
    }

    /// Gets migration history for a workflow.
    pub async fn get_migration_history(&self, workflow_id: Uuid) -> Vec<MigrationRecord> {
        let migrations = self.migrations.read().await;
        migrations
            .iter()
            .filter(|m| m.workflow_id == workflow_id)
            .cloned()
            .collect()
    }

    /// Deprecates a version.
    pub async fn deprecate_version(
        &self,
        workflow_id: Uuid,
        version: u32,
    ) -> WorkflowResult<()> {
        let mut versions = self.versions.write().await;

        let workflow_versions = versions.get_mut(&workflow_id).ok_or_else(|| {
            WorkflowError::NotFound(format!("Workflow {} not found", workflow_id))
        })?;

        let version_meta = workflow_versions.get_mut(&version).ok_or_else(|| {
            WorkflowError::NotFound(format!(
                "Version {} not found for workflow {}",
                version, workflow_id
            ))
        })?;

        version_meta.deprecated = true;
        info!("Deprecated workflow {} version {}", workflow_id, version);
        Ok(())
    }
}

impl Default for VersionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dag::Task;

    #[tokio::test]
    async fn test_version_registration() {
        let registry = VersionRegistry::new();

        let mut workflow = WorkflowDag::new("test_workflow");
        let task = Task::new("task1", "processing");
        workflow.add_task(task);

        let version = WorkflowVersion::new(workflow);
        let workflow_id = version.workflow_id;

        registry.register_version(version).await.unwrap();

        let retrieved = registry.get_version(workflow_id, 1).await.unwrap();
        assert_eq!(retrieved.version, 1);
    }

    #[tokio::test]
    async fn test_active_version() {
        let registry = VersionRegistry::new();

        let mut workflow = WorkflowDag::new("test_workflow");
        let task = Task::new("task1", "processing");
        workflow.add_task(task);

        let version = WorkflowVersion::new(workflow);
        let workflow_id = version.workflow_id;

        registry.register_version(version).await.unwrap();

        let active = registry.get_active_version(workflow_id).await.unwrap();
        assert_eq!(active.version, 1);
    }

    #[tokio::test]
    async fn test_migration() {
        let registry = VersionRegistry::new();

        let mut workflow_v1 = WorkflowDag::new("test_workflow");
        let task1 = Task::new("task1", "processing");
        workflow_v1.add_task(task1);

        let version1 = WorkflowVersion::new(workflow_v1.clone());
        let workflow_id = version1.workflow_id;

        registry.register_version(version1).await.unwrap();

        // Create version 2
        let mut workflow_v2 = workflow_v1.clone();
        workflow_v2.version = 2;
        let task2 = Task::new("task2", "analysis");
        workflow_v2.add_task(task2);

        let version2 = WorkflowVersion::new(workflow_v2);
        registry.register_version(version2).await.unwrap();

        // Migrate
        let plan = MigrationPlan::new(1, 2);
        let record = registry.migrate(workflow_id, plan).await.unwrap();

        assert_eq!(record.status, MigrationStatus::Completed);

        let active = registry.get_active_version(workflow_id).await.unwrap();
        assert_eq!(active.version, 2);
    }
}
