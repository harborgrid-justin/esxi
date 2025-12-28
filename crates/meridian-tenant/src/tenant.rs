//! Core tenant model and lifecycle management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::error::{TenantError, TenantResult};

/// Tenant status enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenantStatus {
    /// Tenant is being provisioned
    Provisioning,
    /// Tenant is active and operational
    Active,
    /// Tenant is suspended (temporary)
    Suspended,
    /// Tenant is marked for deletion
    Deleting,
    /// Tenant is deleted (soft delete)
    Deleted,
    /// Tenant is in maintenance mode
    Maintenance,
}

/// Tenant tier/plan enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenantTier {
    Free,
    Starter,
    Professional,
    Enterprise,
    Custom,
}

/// Isolation strategy for tenant data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationStrategy {
    /// Shared database, shared schema (row-level)
    SharedDatabase,
    /// Shared database, separate schema per tenant
    SeparateSchema,
    /// Dedicated database per tenant
    DedicatedDatabase,
    /// Hybrid approach
    Hybrid,
}

/// Core tenant model representing a multi-tenant customer.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Tenant {
    /// Unique tenant identifier
    pub id: Uuid,

    /// Human-readable tenant slug (e.g., "acme-corp")
    #[validate(length(min = 3, max = 63))]
    #[validate(regex = "SLUG_REGEX")]
    pub slug: String,

    /// Tenant display name
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// Tenant description
    pub description: Option<String>,

    /// Current tenant status
    pub status: TenantStatus,

    /// Tenant tier/plan
    pub tier: TenantTier,

    /// Data isolation strategy
    pub isolation_strategy: IsolationStrategy,

    /// Parent tenant ID (for hierarchical tenants)
    pub parent_id: Option<Uuid>,

    /// Primary contact email
    #[validate(email)]
    pub contact_email: String,

    /// Tenant creation timestamp
    pub created_at: DateTime<Utc>,

    /// Tenant last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Tenant suspension timestamp (if suspended)
    pub suspended_at: Option<DateTime<Utc>>,

    /// Tenant deletion timestamp (if deleted)
    pub deleted_at: Option<DateTime<Utc>>,

    /// Database connection string (for dedicated databases)
    #[serde(skip_serializing)]
    pub database_url: Option<String>,

    /// Schema name (for separate schema strategy)
    pub schema_name: Option<String>,

    /// Custom metadata (JSON)
    pub metadata: serde_json::Value,

    /// Feature flags (bitmask or JSON)
    pub features: u64,

    /// Tenant tags for categorization
    pub tags: Vec<String>,
}

lazy_static::lazy_static! {
    static ref SLUG_REGEX: regex::Regex = regex::Regex::new(r"^[a-z0-9-]+$").unwrap();
}

impl Tenant {
    /// Creates a new tenant with default values.
    pub fn new(slug: impl Into<String>, name: impl Into<String>, contact_email: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            slug: slug.into(),
            name: name.into(),
            description: None,
            status: TenantStatus::Provisioning,
            tier: TenantTier::Free,
            isolation_strategy: IsolationStrategy::SharedDatabase,
            parent_id: None,
            contact_email: contact_email.into(),
            created_at: now,
            updated_at: now,
            suspended_at: None,
            deleted_at: None,
            database_url: None,
            schema_name: None,
            metadata: serde_json::json!({}),
            features: 0,
            tags: Vec::new(),
        }
    }

    /// Checks if tenant is active and operational.
    pub fn is_active(&self) -> bool {
        self.status == TenantStatus::Active
    }

    /// Checks if tenant is suspended.
    pub fn is_suspended(&self) -> bool {
        self.status == TenantStatus::Suspended
    }

    /// Checks if tenant is deleted.
    pub fn is_deleted(&self) -> bool {
        self.status == TenantStatus::Deleted
    }

    /// Validates tenant can perform operations.
    pub fn validate_operational(&self) -> TenantResult<()> {
        match self.status {
            TenantStatus::Active => Ok(()),
            TenantStatus::Suspended => Err(TenantError::TenantSuspended(self.slug.clone())),
            TenantStatus::Deleted => Err(TenantError::TenantDeleted(self.slug.clone())),
            TenantStatus::Provisioning => Err(TenantError::Internal(
                format!("Tenant {} is still provisioning", self.slug)
            )),
            TenantStatus::Deleting => Err(TenantError::TenantDeleted(self.slug.clone())),
            TenantStatus::Maintenance => Err(TenantError::Internal(
                format!("Tenant {} is in maintenance mode", self.slug)
            )),
        }
    }

    /// Activates the tenant.
    pub fn activate(&mut self) {
        self.status = TenantStatus::Active;
        self.suspended_at = None;
        self.updated_at = Utc::now();
    }

    /// Suspends the tenant.
    pub fn suspend(&mut self, reason: Option<String>) {
        self.status = TenantStatus::Suspended;
        self.suspended_at = Some(Utc::now());
        self.updated_at = Utc::now();

        if let Some(reason) = reason {
            if let Some(obj) = self.metadata.as_object_mut() {
                obj.insert("suspension_reason".to_string(), serde_json::json!(reason));
            }
        }
    }

    /// Marks tenant for deletion (soft delete).
    pub fn mark_deleted(&mut self) {
        self.status = TenantStatus::Deleted;
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Sets tenant to maintenance mode.
    pub fn set_maintenance(&mut self, enabled: bool) {
        self.status = if enabled {
            TenantStatus::Maintenance
        } else {
            TenantStatus::Active
        };
        self.updated_at = Utc::now();
    }

    /// Gets the database identifier for this tenant.
    pub fn get_database_identifier(&self) -> String {
        match self.isolation_strategy {
            IsolationStrategy::SharedDatabase => "shared".to_string(),
            IsolationStrategy::SeparateSchema => {
                self.schema_name.clone()
                    .unwrap_or_else(|| format!("tenant_{}", self.slug))
            }
            IsolationStrategy::DedicatedDatabase => {
                format!("tenant_db_{}", self.slug)
            }
            IsolationStrategy::Hybrid => {
                self.schema_name.clone()
                    .unwrap_or_else(|| format!("tenant_{}", self.slug))
            }
        }
    }
}

/// Builder for creating tenants with a fluent API.
#[derive(Debug, Default)]
pub struct TenantBuilder {
    slug: Option<String>,
    name: Option<String>,
    description: Option<String>,
    tier: Option<TenantTier>,
    isolation_strategy: Option<IsolationStrategy>,
    parent_id: Option<Uuid>,
    contact_email: Option<String>,
    metadata: serde_json::Value,
    tags: Vec<String>,
}

impl TenantBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn slug(mut self, slug: impl Into<String>) -> Self {
        self.slug = Some(slug.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn tier(mut self, tier: TenantTier) -> Self {
        self.tier = Some(tier);
        self
    }

    pub fn isolation_strategy(mut self, strategy: IsolationStrategy) -> Self {
        self.isolation_strategy = Some(strategy);
        self
    }

    pub fn parent_id(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn contact_email(mut self, email: impl Into<String>) -> Self {
        self.contact_email = Some(email.into());
        self
    }

    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn build(self) -> TenantResult<Tenant> {
        let slug = self.slug.ok_or_else(|| {
            TenantError::ValidationError("Slug is required".to_string())
        })?;

        let name = self.name.ok_or_else(|| {
            TenantError::ValidationError("Name is required".to_string())
        })?;

        let contact_email = self.contact_email.ok_or_else(|| {
            TenantError::ValidationError("Contact email is required".to_string())
        })?;

        let mut tenant = Tenant::new(slug, name, contact_email);

        tenant.description = self.description;
        if let Some(tier) = self.tier {
            tenant.tier = tier;
        }
        if let Some(strategy) = self.isolation_strategy {
            tenant.isolation_strategy = strategy;
        }
        tenant.parent_id = self.parent_id;
        tenant.metadata = self.metadata;
        tenant.tags = self.tags;

        tenant.validate()?;
        Ok(tenant)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_creation() {
        let tenant = Tenant::new("acme-corp", "ACME Corporation", "admin@acme.com");
        assert_eq!(tenant.slug, "acme-corp");
        assert_eq!(tenant.name, "ACME Corporation");
        assert_eq!(tenant.status, TenantStatus::Provisioning);
    }

    #[test]
    fn test_tenant_lifecycle() {
        let mut tenant = Tenant::new("test", "Test Tenant", "test@example.com");

        tenant.activate();
        assert!(tenant.is_active());

        tenant.suspend(Some("Payment overdue".to_string()));
        assert!(tenant.is_suspended());

        tenant.mark_deleted();
        assert!(tenant.is_deleted());
    }

    #[test]
    fn test_tenant_builder() {
        let tenant = TenantBuilder::new()
            .slug("acme-corp")
            .name("ACME Corporation")
            .contact_email("admin@acme.com")
            .tier(TenantTier::Enterprise)
            .add_tag("production")
            .build()
            .unwrap();

        assert_eq!(tenant.slug, "acme-corp");
        assert_eq!(tenant.tier, TenantTier::Enterprise);
        assert_eq!(tenant.tags.len(), 1);
    }
}
