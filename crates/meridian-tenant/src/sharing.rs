//! Cross-tenant data sharing with permissions and access control.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::error::{TenantError, TenantResult};

/// Permission level for shared resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionLevel {
    None,
    Read,
    Write,
    Admin,
    Owner,
}

impl PermissionLevel {
    pub fn includes(&self, other: &PermissionLevel) -> bool {
        let self_level = self.to_level();
        let other_level = other.to_level();
        self_level >= other_level
    }

    fn to_level(&self) -> u8 {
        match self {
            PermissionLevel::None => 0,
            PermissionLevel::Read => 1,
            PermissionLevel::Write => 2,
            PermissionLevel::Admin => 3,
            PermissionLevel::Owner => 4,
        }
    }
}

/// Shared resource definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedResource {
    pub id: Uuid,
    pub owner_tenant_id: Uuid,
    pub resource_type: String,
    pub resource_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

/// Sharing permission for a specific tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingPermission {
    pub id: Uuid,
    pub shared_resource_id: Uuid,
    pub target_tenant_id: Uuid,
    pub permission_level: PermissionLevel,
    pub granted_by: Uuid, // User or system ID
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Option<SharingConditions>,
}

/// Conditions and restrictions for sharing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingConditions {
    /// Maximum number of accesses allowed
    pub max_accesses: Option<u64>,
    /// Current access count
    pub access_count: u64,
    /// IP address restrictions
    pub allowed_ips: Option<Vec<String>>,
    /// Time-based restrictions
    pub time_restrictions: Option<TimeRestrictions>,
    /// Custom conditions (JSON)
    pub custom: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestrictions {
    pub allowed_days: Vec<chrono::Weekday>,
    pub allowed_hours: Option<(u8, u8)>, // Start hour, end hour (24h format)
}

/// Sharing request from one tenant to share with another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingRequest {
    pub owner_tenant_id: Uuid,
    pub resource_type: String,
    pub resource_id: String,
    pub target_tenant_id: Uuid,
    pub permission_level: PermissionLevel,
    pub expires_at: Option<DateTime<Utc>>,
    pub conditions: Option<SharingConditions>,
}

/// Manager for cross-tenant sharing.
pub struct SharingManager {
    shared_resources: HashMap<Uuid, SharedResource>,
    permissions: HashMap<Uuid, Vec<SharingPermission>>,
    resource_index: HashMap<(Uuid, String, String), Uuid>, // (owner_tenant, type, id) -> shared_resource_id
}

impl SharingManager {
    pub fn new() -> Self {
        Self {
            shared_resources: HashMap::new(),
            permissions: HashMap::new(),
            resource_index: HashMap::new(),
        }
    }

    /// Shares a resource with another tenant.
    pub fn share_resource(&mut self, request: SharingRequest) -> TenantResult<SharedResource> {
        // Check if resource is already shared
        let key = (
            request.owner_tenant_id,
            request.resource_type.clone(),
            request.resource_id.clone(),
        );

        let shared_resource = if let Some(&resource_id) = self.resource_index.get(&key) {
            self.shared_resources.get(&resource_id).unwrap().clone()
        } else {
            // Create new shared resource
            let resource = SharedResource {
                id: Uuid::new_v4(),
                owner_tenant_id: request.owner_tenant_id,
                resource_type: request.resource_type.clone(),
                resource_id: request.resource_id.clone(),
                name: format!("Shared {}", request.resource_type),
                description: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                expires_at: None,
                metadata: serde_json::json!({}),
            };

            self.resource_index.insert(key, resource.id);
            self.shared_resources.insert(resource.id, resource.clone());
            resource
        };

        // Create permission
        let permission = SharingPermission {
            id: Uuid::new_v4(),
            shared_resource_id: shared_resource.id,
            target_tenant_id: request.target_tenant_id,
            permission_level: request.permission_level,
            granted_by: request.owner_tenant_id, // Simplified
            granted_at: Utc::now(),
            expires_at: request.expires_at,
            conditions: request.conditions,
        };

        self.permissions
            .entry(shared_resource.id)
            .or_insert_with(Vec::new)
            .push(permission);

        Ok(shared_resource)
    }

    /// Revokes sharing permission.
    pub fn revoke_permission(&mut self, permission_id: Uuid) -> TenantResult<()> {
        for permissions in self.permissions.values_mut() {
            permissions.retain(|p| p.id != permission_id);
        }
        Ok(())
    }

    /// Checks if a tenant has access to a shared resource.
    pub fn check_access(
        &self,
        tenant_id: Uuid,
        resource_type: &str,
        resource_id: &str,
        required_permission: PermissionLevel,
    ) -> TenantResult<bool> {
        // Find shared resource
        let shared_resource = self.shared_resources.values().find(|r| {
            r.resource_type == resource_type && r.resource_id == resource_id
        });

        if let Some(resource) = shared_resource {
            // Check if tenant is owner
            if resource.owner_tenant_id == tenant_id {
                return Ok(true);
            }

            // Check permissions
            if let Some(permissions) = self.permissions.get(&resource.id) {
                for permission in permissions {
                    if permission.target_tenant_id == tenant_id {
                        // Check expiration
                        if let Some(expires_at) = permission.expires_at {
                            if Utc::now() > expires_at {
                                continue;
                            }
                        }

                        // Check permission level
                        if permission.permission_level.includes(&required_permission) {
                            // Check conditions
                            if let Some(conditions) = &permission.conditions {
                                if let Some(max_accesses) = conditions.max_accesses {
                                    if conditions.access_count >= max_accesses {
                                        continue;
                                    }
                                }
                            }

                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Gets all resources shared with a tenant.
    pub fn get_shared_with_tenant(&self, tenant_id: Uuid) -> Vec<(SharedResource, PermissionLevel)> {
        let mut results = Vec::new();

        for (resource_id, permissions) in &self.permissions {
            for permission in permissions {
                if permission.target_tenant_id == tenant_id {
                    if let Some(resource) = self.shared_resources.get(resource_id) {
                        // Check expiration
                        if let Some(expires_at) = permission.expires_at {
                            if Utc::now() > expires_at {
                                continue;
                            }
                        }
                        results.push((resource.clone(), permission.permission_level));
                    }
                }
            }
        }

        results
    }

    /// Gets all resources shared by a tenant.
    pub fn get_shared_by_tenant(&self, tenant_id: Uuid) -> Vec<SharedResource> {
        self.shared_resources
            .values()
            .filter(|r| r.owner_tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    /// Records an access to a shared resource.
    pub fn record_access(
        &mut self,
        tenant_id: Uuid,
        shared_resource_id: Uuid,
    ) -> TenantResult<()> {
        if let Some(permissions) = self.permissions.get_mut(&shared_resource_id) {
            for permission in permissions {
                if permission.target_tenant_id == tenant_id {
                    if let Some(conditions) = &mut permission.conditions {
                        conditions.access_count += 1;
                    }
                    return Ok(());
                }
            }
        }

        Err(TenantError::PermissionDenied(
            "No permission found for tenant".to_string()
        ))
    }
}

impl Default for SharingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Sharing policy for defining organization-wide rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub allow_cross_tenant_sharing: bool,
    pub default_permission_level: PermissionLevel,
    pub require_approval: bool,
    pub allowed_resource_types: Option<HashSet<String>>,
    pub max_share_duration_days: Option<u32>,
}

impl SharingPolicy {
    pub fn default_policy() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Default Sharing Policy".to_string(),
            description: Some("Default cross-tenant sharing policy".to_string()),
            allow_cross_tenant_sharing: true,
            default_permission_level: PermissionLevel::Read,
            require_approval: false,
            allowed_resource_types: None,
            max_share_duration_days: Some(90),
        }
    }

    pub fn validate_sharing_request(&self, request: &SharingRequest) -> TenantResult<()> {
        if !self.allow_cross_tenant_sharing {
            return Err(TenantError::PermissionDenied(
                "Cross-tenant sharing is not allowed".to_string()
            ));
        }

        if let Some(allowed_types) = &self.allowed_resource_types {
            if !allowed_types.contains(&request.resource_type) {
                return Err(TenantError::PermissionDenied(
                    format!("Resource type '{}' cannot be shared", request.resource_type)
                ));
            }
        }

        if let Some(max_days) = self.max_share_duration_days {
            if let Some(expires_at) = request.expires_at {
                let duration = expires_at - Utc::now();
                if duration.num_days() > max_days as i64 {
                    return Err(TenantError::ValidationError(
                        format!("Share duration exceeds maximum of {} days", max_days)
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_level() {
        assert!(PermissionLevel::Owner.includes(&PermissionLevel::Read));
        assert!(PermissionLevel::Write.includes(&PermissionLevel::Read));
        assert!(!PermissionLevel::Read.includes(&PermissionLevel::Write));
    }

    #[test]
    fn test_sharing_manager() {
        let mut manager = SharingManager::new();
        let owner = Uuid::new_v4();
        let target = Uuid::new_v4();

        let request = SharingRequest {
            owner_tenant_id: owner,
            resource_type: "gis_layer".to_string(),
            resource_id: "layer_123".to_string(),
            target_tenant_id: target,
            permission_level: PermissionLevel::Read,
            expires_at: None,
            conditions: None,
        };

        let result = manager.share_resource(request);
        assert!(result.is_ok());

        let has_access = manager.check_access(
            target,
            "gis_layer",
            "layer_123",
            PermissionLevel::Read,
        );
        assert!(has_access.unwrap());
    }

    #[test]
    fn test_sharing_policy() {
        let policy = SharingPolicy::default_policy();

        let request = SharingRequest {
            owner_tenant_id: Uuid::new_v4(),
            resource_type: "gis_layer".to_string(),
            resource_id: "layer_123".to_string(),
            target_tenant_id: Uuid::new_v4(),
            permission_level: PermissionLevel::Read,
            expires_at: None,
            conditions: None,
        };

        assert!(policy.validate_sharing_request(&request).is_ok());
    }
}
