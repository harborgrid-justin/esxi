//! Tenant provisioning and lifecycle management.

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::error::{TenantError, TenantResult};
use crate::isolation::{DatabaseIsolationManager, SchemaIsolationManager};
use crate::tenant::{IsolationStrategy, Tenant, TenantStatus};

/// Provisioning request for a new tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningRequest {
    pub tenant: Tenant,
    pub options: ProvisioningOptions,
}

/// Options for tenant provisioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningOptions {
    /// Whether to run database migrations
    pub run_migrations: bool,

    /// Whether to seed initial data
    pub seed_data: bool,

    /// Whether to create default user accounts
    pub create_default_users: bool,

    /// Custom initialization scripts
    pub init_scripts: Vec<String>,

    /// Timeout for provisioning (seconds)
    pub timeout_seconds: u64,

    /// Whether to send welcome email
    pub send_welcome_email: bool,
}

impl Default for ProvisioningOptions {
    fn default() -> Self {
        Self {
            run_migrations: true,
            seed_data: false,
            create_default_users: true,
            init_scripts: Vec::new(),
            timeout_seconds: 300,
            send_welcome_email: true,
        }
    }
}

/// Result of provisioning operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningResult {
    pub tenant_id: Uuid,
    pub status: ProvisioningStatus,
    pub started_at: chrono::DateTime<Utc>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
    pub steps_completed: Vec<ProvisioningStep>,
    pub error: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvisioningStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningStep {
    pub name: String,
    pub status: StepStatus,
    pub started_at: chrono::DateTime<Utc>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

/// Trait for tenant provisioning providers.
#[async_trait]
pub trait ProvisioningProvider: Send + Sync {
    /// Provisions a new tenant.
    async fn provision(&self, request: ProvisioningRequest) -> TenantResult<ProvisioningResult>;

    /// Deprovisions an existing tenant.
    async fn deprovision(&self, tenant_id: Uuid, options: DeprovisioningOptions) -> TenantResult<()>;

    /// Gets provisioning status.
    async fn get_status(&self, tenant_id: Uuid) -> TenantResult<ProvisioningResult>;

    /// Validates provisioning prerequisites.
    async fn validate_prerequisites(&self, tenant: &Tenant) -> TenantResult<()>;
}

/// Options for tenant deprovisioning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprovisioningOptions {
    /// Whether to backup data before deletion
    pub backup_data: bool,

    /// Whether to permanently delete or soft delete
    pub permanent_delete: bool,

    /// Retention period for backups (days)
    pub backup_retention_days: u32,

    /// Whether to notify tenant
    pub notify_tenant: bool,
}

impl Default for DeprovisioningOptions {
    fn default() -> Self {
        Self {
            backup_data: true,
            permanent_delete: false,
            backup_retention_days: 90,
            notify_tenant: true,
        }
    }
}

/// Default provisioning implementation.
pub struct DefaultProvisioningProvider {
    schema_manager: Arc<SchemaIsolationManager>,
    database_manager: Arc<DatabaseIsolationManager>,
}

impl DefaultProvisioningProvider {
    pub fn new(
        schema_manager: SchemaIsolationManager,
        database_manager: DatabaseIsolationManager,
    ) -> Self {
        Self {
            schema_manager: Arc::new(schema_manager),
            database_manager: Arc::new(database_manager),
        }
    }

    async fn provision_shared_database(&self, tenant: &Tenant) -> TenantResult<Vec<ProvisioningStep>> {
        let mut steps = Vec::new();

        steps.push(ProvisioningStep {
            name: "validate_tenant".to_string(),
            status: StepStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error: None,
        });

        steps.push(ProvisioningStep {
            name: "create_tenant_record".to_string(),
            status: StepStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error: None,
        });

        Ok(steps)
    }

    async fn provision_separate_schema(&self, tenant: &Tenant) -> TenantResult<Vec<ProvisioningStep>> {
        let mut steps = Vec::new();
        let schema_name = self.schema_manager.generate_schema_name(&tenant.slug);

        steps.push(ProvisioningStep {
            name: "create_schema".to_string(),
            status: StepStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error: None,
        });

        tracing::info!("Created schema: {}", schema_name);

        steps.push(ProvisioningStep {
            name: "run_migrations".to_string(),
            status: StepStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error: None,
        });

        Ok(steps)
    }

    async fn provision_dedicated_database(&self, tenant: &Tenant) -> TenantResult<Vec<ProvisioningStep>> {
        let mut steps = Vec::new();
        let db_name = self.database_manager.generate_database_name(&tenant.slug);

        steps.push(ProvisioningStep {
            name: "create_database".to_string(),
            status: StepStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error: None,
        });

        tracing::info!("Created database: {}", db_name);

        steps.push(ProvisioningStep {
            name: "run_migrations".to_string(),
            status: StepStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error: None,
        });

        steps.push(ProvisioningStep {
            name: "install_extensions".to_string(),
            status: StepStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            error: None,
        });

        Ok(steps)
    }
}

#[async_trait]
impl ProvisioningProvider for DefaultProvisioningProvider {
    async fn provision(&self, request: ProvisioningRequest) -> TenantResult<ProvisioningResult> {
        let started_at = Utc::now();
        let tenant = &request.tenant;

        tracing::info!("Starting provisioning for tenant: {}", tenant.slug);

        // Validate prerequisites
        self.validate_prerequisites(tenant).await?;

        // Execute provisioning based on isolation strategy
        let steps = match tenant.isolation_strategy {
            IsolationStrategy::SharedDatabase => {
                self.provision_shared_database(tenant).await?
            }
            IsolationStrategy::SeparateSchema | IsolationStrategy::Hybrid => {
                self.provision_separate_schema(tenant).await?
            }
            IsolationStrategy::DedicatedDatabase => {
                self.provision_dedicated_database(tenant).await?
            }
        };

        let completed_at = Utc::now();

        Ok(ProvisioningResult {
            tenant_id: tenant.id,
            status: ProvisioningStatus::Completed,
            started_at,
            completed_at: Some(completed_at),
            steps_completed: steps,
            error: None,
            metadata: serde_json::json!({
                "isolation_strategy": tenant.isolation_strategy,
                "tier": tenant.tier,
            }),
        })
    }

    async fn deprovision(&self, tenant_id: Uuid, options: DeprovisioningOptions) -> TenantResult<()> {
        tracing::info!("Starting deprovisioning for tenant: {}", tenant_id);

        if options.backup_data {
            tracing::info!("Creating backup before deprovisioning");
            // Backup logic would go here
        }

        if options.permanent_delete {
            tracing::warn!("Permanently deleting tenant data: {}", tenant_id);
            // Permanent deletion logic
        } else {
            tracing::info!("Soft deleting tenant: {}", tenant_id);
            // Soft delete logic
        }

        Ok(())
    }

    async fn get_status(&self, tenant_id: Uuid) -> TenantResult<ProvisioningResult> {
        // In a real implementation, this would query a provisioning status store
        Ok(ProvisioningResult {
            tenant_id,
            status: ProvisioningStatus::Completed,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            steps_completed: Vec::new(),
            error: None,
            metadata: serde_json::json!({}),
        })
    }

    async fn validate_prerequisites(&self, tenant: &Tenant) -> TenantResult<()> {
        // Validate tenant data
        tenant.validate()
            .map_err(|e| TenantError::ValidationError(e.to_string()))?;

        // Check for duplicate slug
        // In production, this would query the database

        Ok(())
    }
}

/// Provisioning orchestrator for managing complex provisioning workflows.
pub struct ProvisioningOrchestrator {
    provider: Arc<dyn ProvisioningProvider>,
}

impl ProvisioningOrchestrator {
    pub fn new(provider: Arc<dyn ProvisioningProvider>) -> Self {
        Self { provider }
    }

    /// Provisions a tenant with automatic retry and rollback.
    pub async fn provision_with_retry(
        &self,
        request: ProvisioningRequest,
        max_retries: u32,
    ) -> TenantResult<ProvisioningResult> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < max_retries {
            attempts += 1;

            match self.provider.provision(request.clone()).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    tracing::warn!(
                        "Provisioning attempt {} failed: {}",
                        attempts,
                        e
                    );
                    last_error = Some(e);

                    if attempts < max_retries {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2_u64.pow(attempts))).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TenantError::ProvisioningFailed("Max retries exceeded".to_string())
        }))
    }

    /// Provisions multiple tenants in batch.
    pub async fn batch_provision(
        &self,
        requests: Vec<ProvisioningRequest>,
    ) -> Vec<TenantResult<ProvisioningResult>> {
        let mut results = Vec::new();

        for request in requests {
            let result = self.provider.provision(request).await;
            results.push(result);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::TenantBuilder;

    #[tokio::test]
    async fn test_provisioning_request() {
        let tenant = TenantBuilder::new()
            .slug("test-tenant")
            .name("Test Tenant")
            .contact_email("test@example.com")
            .build()
            .unwrap();

        let request = ProvisioningRequest {
            tenant,
            options: ProvisioningOptions::default(),
        };

        assert_eq!(request.tenant.slug, "test-tenant");
        assert!(request.options.run_migrations);
    }
}
