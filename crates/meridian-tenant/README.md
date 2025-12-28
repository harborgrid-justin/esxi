# Meridian Tenant - Multi-tenant Architecture System

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.5-green.svg)](Cargo.toml)

A comprehensive multi-tenant architecture system for the Meridian GIS Platform, providing enterprise-grade tenant isolation, provisioning, resource management, and security features.

## Features

### Core Multi-tenancy
- **Multiple Isolation Strategies**: Shared database, separate schema, or dedicated database per tenant
- **Tenant Lifecycle Management**: Complete provisioning and deprovisioning workflows
- **Hierarchical Tenants**: Support for organizations, divisions, departments, teams, and projects
- **Tenant Status Management**: Active, suspended, maintenance, and soft-delete states

### Resource Management
- **Resource Quotas**: Flexible quota system with hard and soft limits
- **Usage Tracking**: Real-time usage monitoring across multiple resource types
- **Tier-based Limits**: Automatic quota assignment based on tenant tier
- **Usage Analytics**: Comprehensive analytics and reporting

### Security & Isolation
- **Row-Level Security**: PostgreSQL RLS policy generation
- **Cross-tenant Access Control**: Granular permissions for data sharing
- **Isolation Validation**: Automated integrity checks
- **Secure Routing**: Tenant-aware HTTP middleware

### Configuration & Customization
- **Per-tenant Configuration**: Flexible key-value configuration system
- **Configuration Templates**: Quick setup with predefined templates
- **White-labeling**: Complete branding and theming per tenant
- **Custom Domains**: Support for tenant-specific domains

### Billing & Analytics
- **Usage-based Billing**: Track billable events and generate invoices
- **Subscription Management**: Flexible subscription plans and tiers
- **Analytics Engine**: Comprehensive usage analytics and metrics
- **Funnel Analysis**: Conversion tracking and user behavior analysis

### Feature Management
- **Feature Flags**: Fine-grained feature control per tenant
- **Rollout Strategies**: Percentage-based, whitelist, and gradual rollouts
- **Tier-based Features**: Automatic feature availability by subscription tier
- **Feature Configuration**: Per-tenant feature customization

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
meridian-tenant = "0.1.5"
```

With all features:

```toml
[dependencies]
meridian-tenant = { version = "0.1.5", features = ["full"] }
```

## Quick Start

### Creating a Tenant

```rust
use meridian_tenant::prelude::*;

// Create a new tenant using the builder pattern
let tenant = TenantBuilder::new()
    .slug("acme-corp")
    .name("ACME Corporation")
    .contact_email("admin@acme.com")
    .tier(TenantTier::Professional)
    .isolation_strategy(IsolationStrategy::SeparateSchema)
    .add_tag("production")
    .build()?;
```

### Provisioning a Tenant

```rust
use meridian_tenant::prelude::*;

// Set up provisioning infrastructure
let schema_manager = SchemaIsolationManager::new("public");
let db_manager = DatabaseIsolationManager::new("postgresql://localhost/master");
let provider = DefaultProvisioningProvider::new(schema_manager, db_manager);

// Provision the tenant
let request = ProvisioningRequest {
    tenant: tenant.clone(),
    options: ProvisioningOptions::default(),
};

let result = provider.provision(request).await?;
```

### Managing Quotas

```rust
use meridian_tenant::prelude::*;

let mut quota_manager = QuotaManager::new();

// Record API usage
quota_manager.record_usage(
    tenant.id,
    TenantTier::Professional,
    ResourceType::ApiRequests,
    1,
)?;

// Check if quota is available
let can_proceed = quota_manager.check_quota(
    tenant.id,
    TenantTier::Professional,
    ResourceType::ApiRequests,
    100,
)?;
```

### Cross-tenant Sharing

```rust
use meridian_tenant::prelude::*;

let mut sharing_manager = SharingManager::new();

// Share a GIS layer with another tenant
let request = SharingRequest {
    owner_tenant_id: tenant1_id,
    resource_type: "gis_layer".to_string(),
    resource_id: "layer_123".to_string(),
    target_tenant_id: tenant2_id,
    permission_level: PermissionLevel::Read,
    expires_at: None,
    conditions: None,
};

sharing_manager.share_resource(request)?;
```

### Feature Flags

```rust
use meridian_tenant::prelude::*;

let mut feature_manager = FeatureManager::new();

// Check if a feature is enabled for a tenant
if feature_manager.is_enabled(&tenant, "advanced_mapping") {
    // Feature is available
}

// Enable a feature for a specific tenant
feature_manager.enable_feature(tenant.id, "3d_visualization")?;
```

### Analytics

```rust
use meridian_tenant::prelude::*;

let mut analytics = AnalyticsManager::new();

// Track an event
let event = AnalyticsEvent::new(tenant.id, EventType::MapView, "regional_map")
    .with_user(user_id)
    .with_property("zoom_level", json!(12));

analytics.track(event)?;

// Generate summary
let summary = analytics.generate_summary(
    tenant.id,
    start_date,
    end_date,
);
```

## Architecture

### Module Organization

- **`tenant`**: Core tenant model and lifecycle management
- **`error`**: Comprehensive error types
- **`isolation`**: Data isolation strategies and policies
- **`provisioning`**: Automated tenant provisioning
- **`quota`**: Resource quota management
- **`routing`**: Tenant-aware HTTP routing
- **`sharing`**: Cross-tenant data sharing
- **`config`**: Per-tenant configuration
- **`branding`**: White-labeling and theming
- **`billing`**: Billing integration
- **`analytics`**: Usage analytics
- **`hierarchy`**: Hierarchical tenant structures
- **`features`**: Feature flag management

### Isolation Strategies

#### Shared Database (Row-level)
- All tenants share the same database and schema
- Isolation via `tenant_id` column in all tables
- Best for: High tenant density, lower costs
- Security: Row-Level Security (RLS) policies

#### Separate Schema
- Dedicated PostgreSQL schema per tenant
- Shared database, isolated schemas
- Best for: Balance of isolation and efficiency
- Security: Schema-level permissions

#### Dedicated Database
- Completely separate database per tenant
- Maximum isolation and customization
- Best for: Enterprise customers, compliance requirements
- Security: Database-level isolation

## Tenant Tiers

The system supports multiple tenant tiers with different capabilities:

- **Free**: Basic features, limited quotas
- **Starter**: Enhanced features, moderate quotas
- **Professional**: Advanced features, high quotas
- **Enterprise**: All features, unlimited quotas
- **Custom**: Tailored plans with custom quotas

## Security Considerations

### Isolation
- Automatic tenant_id injection in queries
- RLS policy generation for PostgreSQL
- Cross-tenant access validation
- Isolation integrity checks

### Access Control
- Permission-based sharing system
- Time-limited access grants
- IP address restrictions
- Access count limits

### Data Protection
- Soft delete for tenant data
- Backup before deprovisioning
- Encrypted sensitive fields
- Audit logging support

## Performance

### Scalability
- Connection pooling per tenant
- Lazy schema creation
- Caching support
- Async operations throughout

### Optimization
- Indexed tenant_id columns
- Query plan optimization
- Resource usage monitoring
- Performance metrics

## Testing

Run tests with:

```bash
cargo test
```

Run tests with all features:

```bash
cargo test --all-features
```

## Examples

See the `examples/` directory for complete examples:

- `basic_tenant.rs`: Creating and managing tenants
- `provisioning.rs`: Tenant provisioning workflows
- `quotas.rs`: Resource quota management
- `sharing.rs`: Cross-tenant data sharing
- `analytics.rs`: Usage analytics and reporting

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Related Projects

- [meridian-core](../meridian-core): Core GIS functionality
- [meridian-auth](../meridian-auth): Authentication and authorization
- [meridian-db](../meridian-db): Database abstraction layer
