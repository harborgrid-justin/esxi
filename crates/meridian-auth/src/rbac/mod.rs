//! Role-Based Access Control (RBAC) system

pub mod policy;

use crate::error::{AuthError, AuthResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Permission definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    /// Resource type (e.g., "layer", "map", "user")
    pub resource: String,
    /// Action (e.g., "read", "write", "delete")
    pub action: String,
    /// Optional scope/context (e.g., "own", "all")
    pub scope: Option<String>,
}

impl Permission {
    /// Create a new permission
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
            scope: None,
        }
    }

    /// Create a permission with scope
    pub fn with_scope(
        resource: impl Into<String>,
        action: impl Into<String>,
        scope: impl Into<String>,
    ) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
            scope: Some(scope.into()),
        }
    }

    /// Check if this permission matches another
    pub fn matches(&self, other: &Permission) -> bool {
        self.resource == other.resource
            && self.action == other.action
            && (self.scope.is_none() || self.scope == other.scope)
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match &self.scope {
            Some(scope) => format!("{}:{}:{}", self.resource, self.action, scope),
            None => format!("{}:{}", self.resource, self.action),
        }
    }

    /// Parse from string representation
    pub fn from_string(s: &str) -> AuthResult<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        match parts.len() {
            2 => Ok(Self::new(parts[0], parts[1])),
            3 => Ok(Self::with_scope(parts[0], parts[1], parts[2])),
            _ => Err(AuthError::InvalidInput(format!(
                "Invalid permission format: {}",
                s
            ))),
        }
    }
}

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role name
    pub name: String,
    /// Role description
    pub description: Option<String>,
    /// Permissions granted by this role
    pub permissions: HashSet<Permission>,
    /// Parent roles (for role hierarchy)
    pub inherits_from: Vec<String>,
}

impl Role {
    /// Create a new role
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            permissions: HashSet::new(),
            inherits_from: Vec::new(),
        }
    }

    /// Add description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a permission
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Remove a permission
    pub fn remove_permission(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    /// Add parent role for inheritance
    pub fn add_parent(&mut self, role_name: String) {
        if !self.inherits_from.contains(&role_name) {
            self.inherits_from.push(role_name);
        }
    }

    /// Check if role has a specific permission
    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| p.matches(permission))
    }
}

/// Pre-defined roles for GIS platform
pub fn default_roles() -> HashMap<String, Role> {
    let mut roles = HashMap::new();

    // Viewer role - read-only access
    let mut viewer = Role::new("viewer").with_description("Read-only access to GIS data");
    viewer.add_permission(Permission::new("layer", "read"));
    viewer.add_permission(Permission::new("map", "read"));
    viewer.add_permission(Permission::new("feature", "read"));
    viewer.add_permission(Permission::new("style", "read"));
    roles.insert("viewer".to_string(), viewer);

    // Editor role - can create and edit GIS data
    let mut editor = Role::new("editor").with_description("Create and edit GIS data");
    editor.add_parent("viewer".to_string());
    editor.add_permission(Permission::new("layer", "create"));
    editor.add_permission(Permission::new("layer", "update"));
    editor.add_permission(Permission::with_scope("layer", "delete", "own"));
    editor.add_permission(Permission::new("map", "create"));
    editor.add_permission(Permission::new("map", "update"));
    editor.add_permission(Permission::with_scope("map", "delete", "own"));
    editor.add_permission(Permission::new("feature", "create"));
    editor.add_permission(Permission::new("feature", "update"));
    editor.add_permission(Permission::new("feature", "delete"));
    editor.add_permission(Permission::new("style", "create"));
    editor.add_permission(Permission::new("style", "update"));
    roles.insert("editor".to_string(), editor);

    // Publisher role - can publish and share data
    let mut publisher = Role::new("publisher").with_description("Publish and share GIS data");
    publisher.add_parent("editor".to_string());
    publisher.add_permission(Permission::new("layer", "publish"));
    publisher.add_permission(Permission::new("map", "publish"));
    publisher.add_permission(Permission::new("layer", "share"));
    publisher.add_permission(Permission::new("map", "share"));
    roles.insert("publisher".to_string(), publisher);

    // Admin role - full system access
    let mut admin = Role::new("admin").with_description("Full system administration access");
    admin.add_parent("publisher".to_string());
    admin.add_permission(Permission::new("layer", "delete"));
    admin.add_permission(Permission::new("map", "delete"));
    admin.add_permission(Permission::new("user", "read"));
    admin.add_permission(Permission::new("user", "create"));
    admin.add_permission(Permission::new("user", "update"));
    admin.add_permission(Permission::new("user", "delete"));
    admin.add_permission(Permission::new("role", "read"));
    admin.add_permission(Permission::new("role", "create"));
    admin.add_permission(Permission::new("role", "update"));
    admin.add_permission(Permission::new("role", "delete"));
    admin.add_permission(Permission::new("system", "configure"));
    admin.add_permission(Permission::new("audit", "read"));
    roles.insert("admin".to_string(), admin);

    // Analyst role - advanced analysis capabilities
    let mut analyst = Role::new("analyst").with_description("Perform spatial analysis");
    analyst.add_parent("editor".to_string());
    analyst.add_permission(Permission::new("analysis", "execute"));
    analyst.add_permission(Permission::new("analysis", "create"));
    analyst.add_permission(Permission::new("analysis", "read"));
    analyst.add_permission(Permission::new("query", "execute"));
    roles.insert("analyst".to_string(), analyst);

    // API User role - programmatic access
    let mut api_user = Role::new("api_user").with_description("API access for integrations");
    api_user.add_parent("viewer".to_string());
    api_user.add_permission(Permission::new("api", "read"));
    api_user.add_permission(Permission::new("api", "write"));
    roles.insert("api_user".to_string(), api_user);

    roles
}

/// RBAC manager
pub struct RbacManager {
    /// Available roles
    roles: HashMap<String, Role>,
}

impl RbacManager {
    /// Create a new RBAC manager with default roles
    pub fn new() -> Self {
        Self {
            roles: default_roles(),
        }
    }

    /// Create an empty RBAC manager
    pub fn empty() -> Self {
        Self {
            roles: HashMap::new(),
        }
    }

    /// Add or update a role
    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    /// Remove a role
    pub fn remove_role(&mut self, role_name: &str) -> AuthResult<()> {
        self.roles.remove(role_name);
        Ok(())
    }

    /// Get a role by name
    pub fn get_role(&self, role_name: &str) -> Option<&Role> {
        self.roles.get(role_name)
    }

    /// Get all permissions for a role (including inherited)
    pub fn get_all_permissions(&self, role_name: &str) -> HashSet<Permission> {
        let mut permissions = HashSet::new();
        let mut visited = HashSet::new();
        self.collect_permissions(role_name, &mut permissions, &mut visited);
        permissions
    }

    /// Recursively collect permissions from role and parents
    fn collect_permissions(
        &self,
        role_name: &str,
        permissions: &mut HashSet<Permission>,
        visited: &mut HashSet<String>,
    ) {
        // Avoid infinite loops in case of circular inheritance
        if visited.contains(role_name) {
            return;
        }
        visited.insert(role_name.to_string());

        if let Some(role) = self.roles.get(role_name) {
            // Add role's own permissions
            permissions.extend(role.permissions.clone());

            // Recursively add parent permissions
            for parent in &role.inherits_from {
                self.collect_permissions(parent, permissions, visited);
            }
        }
    }

    /// Check if a user with given roles has a specific permission
    pub fn has_permission(&self, user_roles: &[String], permission: &Permission) -> bool {
        for role_name in user_roles {
            let all_permissions = self.get_all_permissions(role_name);
            if all_permissions.iter().any(|p| p.matches(permission)) {
                return true;
            }
        }
        false
    }

    /// Check if user has any of the specified permissions
    pub fn has_any_permission(&self, user_roles: &[String], permissions: &[Permission]) -> bool {
        permissions
            .iter()
            .any(|p| self.has_permission(user_roles, p))
    }

    /// Check if user has all specified permissions
    pub fn has_all_permissions(&self, user_roles: &[String], permissions: &[Permission]) -> bool {
        permissions
            .iter()
            .all(|p| self.has_permission(user_roles, p))
    }

    /// Authorize a user action
    pub fn authorize(
        &self,
        user_roles: &[String],
        resource: &str,
        action: &str,
    ) -> AuthResult<()> {
        let permission = Permission::new(resource, action);
        if self.has_permission(user_roles, &permission) {
            Ok(())
        } else {
            Err(AuthError::InsufficientPermissions(format!(
                "Missing permission: {}:{}",
                resource, action
            )))
        }
    }

    /// Get all role names
    pub fn list_roles(&self) -> Vec<String> {
        self.roles.keys().cloned().collect()
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_creation() {
        let perm = Permission::new("layer", "read");
        assert_eq!(perm.resource, "layer");
        assert_eq!(perm.action, "read");
        assert_eq!(perm.scope, None);

        let perm_scoped = Permission::with_scope("layer", "delete", "own");
        assert_eq!(perm_scoped.scope, Some("own".to_string()));
    }

    #[test]
    fn test_permission_string() {
        let perm = Permission::new("layer", "read");
        assert_eq!(perm.to_string(), "layer:read");

        let perm_scoped = Permission::with_scope("layer", "delete", "own");
        assert_eq!(perm_scoped.to_string(), "layer:delete:own");
    }

    #[test]
    fn test_permission_parsing() {
        let perm = Permission::from_string("layer:read").unwrap();
        assert_eq!(perm.resource, "layer");
        assert_eq!(perm.action, "read");

        let perm_scoped = Permission::from_string("layer:delete:own").unwrap();
        assert_eq!(perm_scoped.scope, Some("own".to_string()));
    }

    #[test]
    fn test_role_creation() {
        let mut role = Role::new("editor");
        role.add_permission(Permission::new("layer", "read"));
        role.add_permission(Permission::new("layer", "write"));

        assert!(role.has_permission(&Permission::new("layer", "read")));
        assert!(role.has_permission(&Permission::new("layer", "write")));
        assert!(!role.has_permission(&Permission::new("user", "delete")));
    }

    #[test]
    fn test_rbac_manager() {
        let rbac = RbacManager::new();

        // Viewer can read layers
        assert!(rbac.has_permission(
            &["viewer".to_string()],
            &Permission::new("layer", "read")
        ));

        // Viewer cannot delete layers
        assert!(!rbac.has_permission(
            &["viewer".to_string()],
            &Permission::new("layer", "delete")
        ));

        // Admin can delete layers
        assert!(rbac.has_permission(
            &["admin".to_string()],
            &Permission::new("layer", "delete")
        ));
    }

    #[test]
    fn test_role_inheritance() {
        let rbac = RbacManager::new();

        // Editor inherits from viewer, so should have viewer permissions
        assert!(rbac.has_permission(
            &["editor".to_string()],
            &Permission::new("layer", "read")
        ));

        // Admin inherits from publisher -> editor -> viewer
        assert!(rbac.has_permission(
            &["admin".to_string()],
            &Permission::new("layer", "read")
        ));
        assert!(rbac.has_permission(
            &["admin".to_string()],
            &Permission::new("layer", "create")
        ));
        assert!(rbac.has_permission(
            &["admin".to_string()],
            &Permission::new("layer", "publish")
        ));
    }

    #[test]
    fn test_authorization() {
        let rbac = RbacManager::new();

        // Viewer authorized to read
        assert!(rbac
            .authorize(&["viewer".to_string()], "layer", "read")
            .is_ok());

        // Viewer not authorized to delete
        assert!(rbac
            .authorize(&["viewer".to_string()], "layer", "delete")
            .is_err());

        // Admin authorized for everything
        assert!(rbac
            .authorize(&["admin".to_string()], "user", "delete")
            .is_ok());
    }

    #[test]
    fn test_multiple_roles() {
        let rbac = RbacManager::new();

        // User with both viewer and analyst roles
        let roles = vec!["viewer".to_string(), "analyst".to_string()];

        // Should have viewer permissions
        assert!(rbac.has_permission(&roles, &Permission::new("layer", "read")));

        // Should have analyst permissions
        assert!(rbac.has_permission(&roles, &Permission::new("analysis", "execute")));
    }

    #[test]
    fn test_default_roles() {
        let roles = default_roles();

        assert!(roles.contains_key("viewer"));
        assert!(roles.contains_key("editor"));
        assert!(roles.contains_key("publisher"));
        assert!(roles.contains_key("admin"));
        assert!(roles.contains_key("analyst"));
        assert!(roles.contains_key("api_user"));
    }
}
