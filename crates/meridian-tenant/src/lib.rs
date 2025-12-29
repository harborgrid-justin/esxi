//! # Meridian Tenant - Multi-tenant Architecture System
//!
//! A comprehensive multi-tenant architecture system for the Meridian GIS Platform,
//! providing tenant isolation, provisioning, resource management, and more.
//!
//! ## Features
//!
//! - **Tenant Isolation**: Multiple isolation strategies (shared database, separate schema, dedicated database)
//! - **Provisioning**: Automated tenant provisioning and lifecycle management
//! - **Resource Quotas**: Flexible quota management and usage tracking
//! - **Tenant-aware Routing**: HTTP routing middleware with multiple resolution strategies
//! - **Cross-tenant Sharing**: Secure data sharing with granular permissions
//! - **Configuration Management**: Per-tenant configuration with templates
//! - **White-labeling**: Full branding and customization per tenant
//! - **Billing Integration**: Usage tracking and billing hooks
//! - **Analytics**: Comprehensive analytics and usage metrics
//! - **Hierarchical Tenants**: Support for organizations, teams, and projects
//! - **Feature Flags**: Fine-grained feature control with rollout strategies
//!
//! ## Quick Start
//!
//! ```rust
//! use meridian_tenant::{
//!     tenant::{Tenant, TenantBuilder, TenantTier},
//!     provisioning::{DefaultProvisioningProvider, ProvisioningRequest, ProvisioningOptions},
//!     isolation::{SchemaIsolationManager, DatabaseIsolationManager},
//! };
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new tenant
//! let tenant = TenantBuilder::new()
//!     .slug("acme-corp")
//!     .name("ACME Corporation")
//!     .contact_email("admin@acme.com")
//!     .tier(TenantTier::Professional)
//!     .build()?;
//!
//! // Set up provisioning
//! let schema_manager = SchemaIsolationManager::new("public");
//! let db_manager = DatabaseIsolationManager::new("postgresql://localhost/master");
//! let provider = DefaultProvisioningProvider::new(schema_manager, db_manager);
//!
//! // Provision the tenant
//! let request = ProvisioningRequest {
//!     tenant: tenant.clone(),
//!     options: ProvisioningOptions::default(),
//! };
//!
//! let result = provider.provision(request).await?;
//! println!("Tenant provisioned: {:?}", result);
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! The crate is organized into the following modules:
//!
//! - [`tenant`]: Core tenant model and lifecycle
//! - [`isolation`]: Data isolation strategies
//! - [`provisioning`]: Tenant provisioning and deprovisioning
//! - [`quota`]: Resource quotas and usage limits
//! - [`routing`]: HTTP routing and tenant resolution
//! - [`sharing`]: Cross-tenant data sharing
//! - [`config`]: Per-tenant configuration
//! - [`branding`]: White-labeling and branding
//! - [`billing`]: Billing integration and usage tracking
//! - [`analytics`]: Analytics and metrics
//! - [`hierarchy`]: Hierarchical tenant structures
//! - [`features`]: Feature flags and capabilities

#![warn(missing_docs)]
#![warn(clippy::all)]

// Re-export lazy_static and regex for tenant module
#[doc(hidden)]
pub use lazy_static;
#[doc(hidden)]
pub use regex;

/// Error types for multi-tenant operations.
pub mod error;

/// Core tenant model and lifecycle management.
pub mod tenant;

/// Data isolation strategies and policies.
pub mod isolation;

/// Tenant provisioning and deprovisioning.
pub mod provisioning;

/// Resource quotas and usage limits.
pub mod quota;

/// Tenant-aware routing and middleware.
pub mod routing;

/// Cross-tenant data sharing with permissions.
pub mod sharing;

/// Per-tenant configuration management.
pub mod config;

/// White-labeling and branding.
pub mod branding;

/// Billing integration and usage tracking.
pub mod billing;

/// Analytics and usage metrics.
pub mod analytics;

/// Hierarchical tenant structures.
pub mod hierarchy;

/// Feature flags and capability management.
pub mod features;

// Re-export commonly used types
pub use error::{TenantError, TenantResult};
pub use tenant::{IsolationStrategy, Tenant, TenantBuilder, TenantStatus, TenantTier};

/// Prelude module for convenient imports.
pub mod prelude {
    //! Convenient re-exports of commonly used types.
    //!
    //! # Example
    //!
    //! ```
    //! use meridian_tenant::prelude::*;
    //! ```

    pub use crate::analytics::{AnalyticsEvent, AnalyticsManager, EventType, MetricsSummary};
    pub use crate::billing::{BillingAccount, BillingManager, BillingStatus, UsageEvent};
    pub use crate::branding::{BrandingManager, TenantBranding};
    pub use crate::config::{ConfigManager, TenantConfig};
    pub use crate::error::{TenantError, TenantResult};
    pub use crate::features::{FeatureFlag, FeatureManager, FeatureRollout};
    pub use crate::hierarchy::{HierarchyLevel, HierarchyManager, TenantNode};
    pub use crate::isolation::{
        DatabaseIsolationManager, IsolationContext, SchemaIsolationManager,
    };
    pub use crate::provisioning::{
        DefaultProvisioningProvider, ProvisioningOptions, ProvisioningProvider,
        ProvisioningRequest,
    };
    pub use crate::quota::{QuotaManager, ResourceType, ResourceUsage, TierQuotas};
    pub use crate::routing::{TenantContext, TenantResolutionStrategy, TenantResolver};
    pub use crate::sharing::{PermissionLevel, SharedResource, SharingManager};
    pub use crate::tenant::{IsolationStrategy, Tenant, TenantBuilder, TenantStatus, TenantTier};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tenant_creation() {
        let tenant = TenantBuilder::new()
            .slug("test-tenant")
            .name("Test Tenant")
            .contact_email("test@example.com")
            .tier(TenantTier::Free)
            .build()
            .unwrap();

        assert_eq!(tenant.slug, "test-tenant");
        assert_eq!(tenant.name, "Test Tenant");
        assert_eq!(tenant.tier, TenantTier::Free);
    }

    #[test]
    fn test_isolation_strategies() {
        let tenant1 = TenantBuilder::new()
            .slug("tenant1")
            .name("Tenant 1")
            .contact_email("t1@example.com")
            .isolation_strategy(IsolationStrategy::SharedDatabase)
            .build()
            .unwrap();

        let tenant2 = TenantBuilder::new()
            .slug("tenant2")
            .name("Tenant 2")
            .contact_email("t2@example.com")
            .isolation_strategy(IsolationStrategy::SeparateSchema)
            .build()
            .unwrap();

        assert_eq!(tenant1.isolation_strategy, IsolationStrategy::SharedDatabase);
        assert_eq!(tenant2.isolation_strategy, IsolationStrategy::SeparateSchema);
    }
}
