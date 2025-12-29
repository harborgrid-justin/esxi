//! Data isolation strategies for multi-tenant architecture.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{TenantError, TenantResult};
use crate::tenant::{IsolationStrategy, Tenant};

/// Isolation context for database operations.
#[derive(Debug, Clone)]
pub struct IsolationContext {
    pub tenant_id: Uuid,
    pub strategy: IsolationStrategy,
    pub schema_name: Option<String>,
    pub database_url: Option<String>,
    pub row_level_filter: Option<String>,
}

impl IsolationContext {
    pub fn new(tenant: &Tenant) -> Self {
        Self {
            tenant_id: tenant.id,
            strategy: tenant.isolation_strategy,
            schema_name: tenant.schema_name.clone(),
            database_url: tenant.database_url.clone(),
            row_level_filter: None,
        }
    }

    /// Builds a SQL query with tenant isolation applied.
    pub fn apply_isolation(&self, base_query: &str) -> String {
        match self.strategy {
            IsolationStrategy::SharedDatabase => {
                // Add row-level WHERE clause
                if base_query.to_lowercase().contains("where") {
                    format!("{} AND tenant_id = '{}'", base_query, self.tenant_id)
                } else {
                    format!("{} WHERE tenant_id = '{}'", base_query, self.tenant_id)
                }
            }
            IsolationStrategy::SeparateSchema => {
                // Prepend schema name to tables
                if let Some(schema) = &self.schema_name {
                    self.apply_schema_prefix(base_query, schema)
                } else {
                    base_query.to_string()
                }
            }
            IsolationStrategy::DedicatedDatabase | IsolationStrategy::Hybrid => {
                // No modification needed, database connection handles isolation
                base_query.to_string()
            }
        }
    }

    fn apply_schema_prefix(&self, query: &str, schema: &str) -> String {
        // Simple schema prefix application
        // In production, use a proper SQL parser
        query.replace("FROM ", &format!("FROM {}.", schema))
             .replace("JOIN ", &format!("JOIN {}.", schema))
    }

    /// Gets the database connection string for this isolation context.
    pub fn get_connection_string(&self, base_url: &str) -> String {
        match self.strategy {
            IsolationStrategy::DedicatedDatabase => {
                self.database_url.clone().unwrap_or_else(|| base_url.to_string())
            }
            _ => base_url.to_string(),
        }
    }
}

/// Trait for isolation-aware data access.
#[async_trait]
pub trait IsolationProvider: Send + Sync {
    /// Validates that data access respects tenant isolation.
    async fn validate_access(&self, context: &IsolationContext, resource_id: &str) -> TenantResult<bool>;

    /// Applies isolation filters to a query.
    fn apply_filters(&self, context: &IsolationContext, query: &str) -> String;

    /// Checks for cross-tenant data leakage.
    async fn check_isolation_integrity(&self, tenant_id: Uuid) -> TenantResult<IsolationIntegrityReport>;
}

/// Report on tenant isolation integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationIntegrityReport {
    pub tenant_id: Uuid,
    pub strategy: IsolationStrategy,
    pub checked_at: chrono::DateTime<chrono::Utc>,
    pub total_records: u64,
    pub violations: Vec<IsolationViolation>,
    pub is_compliant: bool,
}

/// Describes an isolation violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationViolation {
    pub resource_type: String,
    pub resource_id: String,
    pub expected_tenant: Uuid,
    pub actual_tenant: Option<Uuid>,
    pub severity: ViolationSeverity,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Row-Level Security (RLS) policy generator.
pub struct RowLevelSecurity {
    policies: HashMap<String, RLSPolicy>,
}

#[derive(Debug, Clone)]
pub struct RLSPolicy {
    pub table_name: String,
    pub tenant_column: String,
    pub policy_name: String,
    pub using_clause: String,
    pub with_check: Option<String>,
}

impl RowLevelSecurity {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    /// Adds a row-level security policy for a table.
    pub fn add_policy(&mut self, policy: RLSPolicy) {
        self.policies.insert(policy.table_name.clone(), policy);
    }

    /// Generates PostgreSQL RLS policy SQL.
    pub fn generate_postgres_policy(&self, table_name: &str) -> Option<String> {
        self.policies.get(table_name).map(|policy| {
            let mut sql = format!(
                "ALTER TABLE {} ENABLE ROW LEVEL SECURITY;\n",
                policy.table_name
            );

            sql.push_str(&format!(
                "CREATE POLICY {} ON {}\n",
                policy.policy_name, policy.table_name
            ));

            sql.push_str(&format!(
                "USING ({});\n",
                policy.using_clause
            ));

            if let Some(with_check) = &policy.with_check {
                sql.push_str(&format!("WITH CHECK ({});\n", with_check));
            }

            sql
        })
    }

    /// Generates policies for all registered tables.
    pub fn generate_all_policies(&self) -> String {
        self.policies
            .keys()
            .filter_map(|table| self.generate_postgres_policy(table))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for RowLevelSecurity {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema isolation manager for separate schema strategy.
pub struct SchemaIsolationManager {
    base_schema: String,
}

impl SchemaIsolationManager {
    pub fn new(base_schema: impl Into<String>) -> Self {
        Self {
            base_schema: base_schema.into(),
        }
    }

    /// Generates schema name for a tenant.
    pub fn generate_schema_name(&self, tenant_slug: &str) -> String {
        format!("tenant_{}", tenant_slug.replace('-', "_"))
    }

    /// Creates schema isolation DDL statements.
    pub fn generate_schema_ddl(&self, schema_name: &str) -> String {
        format!(
            r#"
-- Create tenant schema
CREATE SCHEMA IF NOT EXISTS {};

-- Set search path for tenant
ALTER ROLE tenant_user SET search_path TO {}, public;

-- Grant permissions
GRANT USAGE ON SCHEMA {} TO tenant_user;
GRANT ALL ON ALL TABLES IN SCHEMA {} TO tenant_user;
GRANT ALL ON ALL SEQUENCES IN SCHEMA {} TO tenant_user;
"#,
            schema_name,
            schema_name,
            schema_name,
            schema_name,
            schema_name
        )
    }

    /// Drops a tenant schema (use with extreme caution).
    pub fn generate_drop_schema_ddl(&self, schema_name: &str) -> String {
        format!("DROP SCHEMA IF EXISTS {} CASCADE;", schema_name)
    }
}

/// Database isolation manager for dedicated database strategy.
pub struct DatabaseIsolationManager {
    master_connection_url: String,
}

impl DatabaseIsolationManager {
    pub fn new(master_connection_url: impl Into<String>) -> Self {
        Self {
            master_connection_url: master_connection_url.into(),
        }
    }

    /// Generates database name for a tenant.
    pub fn generate_database_name(&self, tenant_slug: &str) -> String {
        format!("tenant_db_{}", tenant_slug.replace('-', "_"))
    }

    /// Creates database isolation DDL statements.
    pub fn generate_database_ddl(&self, database_name: &str, owner: &str) -> String {
        format!(
            r#"
-- Create dedicated tenant database
CREATE DATABASE {} OWNER {};

-- Connect to new database and run schema migrations
\c {}

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "postgis";

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE {} TO {};
"#,
            database_name, owner, database_name, database_name, owner
        )
    }

    /// Generates connection URL for tenant database.
    pub fn generate_connection_url(&self, database_name: &str) -> String {
        // Parse master URL and replace database name
        if let Some(idx) = self.master_connection_url.rfind('/') {
            format!("{}/{}", &self.master_connection_url[..idx], database_name)
        } else {
            format!("{}/{}", self.master_connection_url, database_name)
        }
    }
}

/// Cross-tenant access validator.
pub struct CrossTenantAccessValidator {
    allowed_cross_tenant_access: HashMap<Uuid, Vec<Uuid>>,
}

impl CrossTenantAccessValidator {
    pub fn new() -> Self {
        Self {
            allowed_cross_tenant_access: HashMap::new(),
        }
    }

    /// Grants cross-tenant access permission.
    pub fn grant_access(&mut self, source_tenant: Uuid, target_tenant: Uuid) {
        self.allowed_cross_tenant_access
            .entry(source_tenant)
            .or_insert_with(Vec::new)
            .push(target_tenant);
    }

    /// Revokes cross-tenant access permission.
    pub fn revoke_access(&mut self, source_tenant: Uuid, target_tenant: Uuid) {
        if let Some(allowed) = self.allowed_cross_tenant_access.get_mut(&source_tenant) {
            allowed.retain(|&id| id != target_tenant);
        }
    }

    /// Validates if cross-tenant access is allowed.
    pub fn validate_access(&self, source_tenant: Uuid, target_tenant: Uuid) -> TenantResult<()> {
        if source_tenant == target_tenant {
            return Ok(()); // Same tenant, always allowed
        }

        if let Some(allowed) = self.allowed_cross_tenant_access.get(&source_tenant) {
            if allowed.contains(&target_tenant) {
                return Ok(());
            }
        }

        Err(TenantError::CrossTenantAccessDenied {
            source_tenant: source_tenant.to_string(),
            target: target_tenant.to_string(),
        })
    }
}

impl Default for CrossTenantAccessValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolation_context_shared_database() {
        let tenant = crate::tenant::Tenant::new("test", "Test", "test@example.com");
        let context = IsolationContext::new(&tenant);

        let query = "SELECT * FROM users";
        let isolated = context.apply_isolation(query);

        assert!(isolated.contains("WHERE tenant_id"));
    }

    #[test]
    fn test_schema_isolation_manager() {
        let manager = SchemaIsolationManager::new("public");
        let schema_name = manager.generate_schema_name("acme-corp");

        assert_eq!(schema_name, "tenant_acme_corp");

        let ddl = manager.generate_schema_ddl(&schema_name);
        assert!(ddl.contains("CREATE SCHEMA"));
    }

    #[test]
    fn test_cross_tenant_access_validator() {
        let mut validator = CrossTenantAccessValidator::new();
        let tenant1 = Uuid::new_v4();
        let tenant2 = Uuid::new_v4();

        // Initially denied
        assert!(validator.validate_access(tenant1, tenant2).is_err());

        // Grant access
        validator.grant_access(tenant1, tenant2);
        assert!(validator.validate_access(tenant1, tenant2).is_ok());

        // Revoke access
        validator.revoke_access(tenant1, tenant2);
        assert!(validator.validate_access(tenant1, tenant2).is_err());
    }
}
