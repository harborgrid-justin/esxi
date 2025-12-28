//! Hierarchical tenant structures (organizations, teams, projects).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::error::{TenantError, TenantResult};
use crate::tenant::Tenant;

/// Tenant hierarchy level types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HierarchyLevel {
    /// Root organization
    Organization,
    /// Division within organization
    Division,
    /// Department within division
    Department,
    /// Team within department
    Team,
    /// Project within team
    Project,
}

/// Hierarchical tenant node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantNode {
    pub tenant_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub level: HierarchyLevel,
    pub children: Vec<Uuid>,
    pub path: Vec<Uuid>, // Full path from root to this node
    pub depth: usize,
}

impl TenantNode {
    pub fn new(tenant_id: Uuid, level: HierarchyLevel) -> Self {
        Self {
            tenant_id,
            parent_id: None,
            level,
            children: Vec::new(),
            path: vec![tenant_id],
            depth: 0,
        }
    }

    pub fn with_parent(mut self, parent_id: Uuid, parent_path: &[Uuid]) -> Self {
        self.parent_id = Some(parent_id);
        self.path = parent_path.to_vec();
        self.path.push(self.tenant_id);
        self.depth = parent_path.len();
        self
    }

    /// Checks if this node is an ancestor of another node.
    pub fn is_ancestor_of(&self, other: &TenantNode) -> bool {
        other.path.contains(&self.tenant_id) && self.tenant_id != other.tenant_id
    }

    /// Checks if this node is a descendant of another node.
    pub fn is_descendant_of(&self, other: &TenantNode) -> bool {
        other.is_ancestor_of(self)
    }

    /// Checks if this node is a sibling of another node.
    pub fn is_sibling_of(&self, other: &TenantNode) -> bool {
        self.parent_id.is_some() && self.parent_id == other.parent_id
    }
}

/// Tenant hierarchy manager.
pub struct HierarchyManager {
    nodes: HashMap<Uuid, TenantNode>,
    roots: Vec<Uuid>,
}

impl HierarchyManager {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            roots: Vec::new(),
        }
    }

    /// Adds a root tenant to the hierarchy.
    pub fn add_root(&mut self, tenant: &Tenant) -> TenantResult<()> {
        if tenant.parent_id.is_some() {
            return Err(TenantError::HierarchyError(
                "Root tenant cannot have a parent".to_string()
            ));
        }

        let node = TenantNode::new(tenant.id, HierarchyLevel::Organization);
        self.nodes.insert(tenant.id, node);
        self.roots.push(tenant.id);

        Ok(())
    }

    /// Adds a child tenant to the hierarchy.
    pub fn add_child(
        &mut self,
        tenant: &Tenant,
        level: HierarchyLevel,
    ) -> TenantResult<()> {
        let parent_id = tenant.parent_id.ok_or_else(|| {
            TenantError::HierarchyError("Child tenant must have a parent".to_string())
        })?;

        // Check if parent exists
        let parent_node = self.nodes.get(&parent_id).ok_or_else(|| {
            TenantError::InvalidParent(format!("Parent tenant {} not found", parent_id))
        })?;

        // Check for circular reference
        if parent_node.path.contains(&tenant.id) {
            return Err(TenantError::CircularHierarchy(
                format!("Circular reference detected: {}", tenant.id)
            ));
        }

        let parent_path = parent_node.path.clone();
        let node = TenantNode::new(tenant.id, level).with_parent(parent_id, &parent_path);

        // Add to parent's children
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.children.push(tenant.id);
        }

        self.nodes.insert(tenant.id, node);

        Ok(())
    }

    /// Removes a tenant from the hierarchy.
    pub fn remove(&mut self, tenant_id: Uuid) -> TenantResult<()> {
        let node = self.nodes.get(&tenant_id).ok_or_else(|| {
            TenantError::TenantNotFound(tenant_id.to_string())
        })?;

        // Check if node has children
        if !node.children.is_empty() {
            return Err(TenantError::HierarchyError(
                "Cannot remove tenant with children".to_string()
            ));
        }

        // Remove from parent's children
        if let Some(parent_id) = node.parent_id {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.children.retain(|&id| id != tenant_id);
            }
        } else {
            // Remove from roots
            self.roots.retain(|&id| id != tenant_id);
        }

        self.nodes.remove(&tenant_id);

        Ok(())
    }

    /// Moves a tenant to a new parent.
    pub fn move_tenant(&mut self, tenant_id: Uuid, new_parent_id: Uuid) -> TenantResult<()> {
        // Validate nodes exist
        let node = self.nodes.get(&tenant_id).ok_or_else(|| {
            TenantError::TenantNotFound(tenant_id.to_string())
        })?;

        let new_parent = self.nodes.get(&new_parent_id).ok_or_else(|| {
            TenantError::InvalidParent(format!("Parent tenant {} not found", new_parent_id))
        })?;

        // Check for circular reference
        if new_parent.path.contains(&tenant_id) {
            return Err(TenantError::CircularHierarchy(
                "Cannot move tenant to its descendant".to_string()
            ));
        }

        // Clone the new parent path before mutable borrows
        let new_parent_path = new_parent.path.clone();
        let old_parent_id = node.parent_id;

        // Remove from old parent
        if let Some(old_parent_id) = old_parent_id {
            if let Some(old_parent) = self.nodes.get_mut(&old_parent_id) {
                old_parent.children.retain(|&id| id != tenant_id);
            }
        }

        // Update paths for this node and all descendants
        self.update_paths(tenant_id, &new_parent_path);

        // Add to new parent
        if let Some(new_parent) = self.nodes.get_mut(&new_parent_id) {
            new_parent.children.push(tenant_id);
        }

        // Update node's parent
        if let Some(node) = self.nodes.get_mut(&tenant_id) {
            node.parent_id = Some(new_parent_id);
        }

        Ok(())
    }

    fn update_paths(&mut self, tenant_id: Uuid, parent_path: &[Uuid]) {
        if let Some(node) = self.nodes.get_mut(&tenant_id) {
            node.path = parent_path.to_vec();
            node.path.push(tenant_id);
            node.depth = parent_path.len();

            let children = node.children.clone();
            let node_path = node.path.clone();

            for child_id in children {
                self.update_paths(child_id, &node_path);
            }
        }
    }

    /// Gets a tenant node.
    pub fn get(&self, tenant_id: Uuid) -> Option<&TenantNode> {
        self.nodes.get(&tenant_id)
    }

    /// Gets all root tenants.
    pub fn get_roots(&self) -> Vec<&TenantNode> {
        self.roots
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    /// Gets all children of a tenant.
    pub fn get_children(&self, tenant_id: Uuid) -> Vec<&TenantNode> {
        self.nodes
            .get(&tenant_id)
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|id| self.nodes.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets all descendants of a tenant (recursive).
    pub fn get_descendants(&self, tenant_id: Uuid) -> Vec<&TenantNode> {
        let mut descendants = Vec::new();
        self.collect_descendants(tenant_id, &mut descendants);
        descendants
    }

    fn collect_descendants<'a>(&'a self, tenant_id: Uuid, descendants: &mut Vec<&'a TenantNode>) {
        if let Some(node) = self.nodes.get(&tenant_id) {
            for child_id in &node.children {
                if let Some(child) = self.nodes.get(child_id) {
                    descendants.push(child);
                    self.collect_descendants(*child_id, descendants);
                }
            }
        }
    }

    /// Gets all ancestors of a tenant.
    pub fn get_ancestors(&self, tenant_id: Uuid) -> Vec<&TenantNode> {
        if let Some(node) = self.nodes.get(&tenant_id) {
            node.path[..node.path.len() - 1]
                .iter()
                .filter_map(|id| self.nodes.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Gets all siblings of a tenant.
    pub fn get_siblings(&self, tenant_id: Uuid) -> Vec<&TenantNode> {
        if let Some(node) = self.nodes.get(&tenant_id) {
            if let Some(parent_id) = node.parent_id {
                return self
                    .get_children(parent_id)
                    .into_iter()
                    .filter(|child| child.tenant_id != tenant_id)
                    .collect();
            }
        }
        Vec::new()
    }

    /// Gets the full hierarchy as a tree structure.
    pub fn get_tree(&self) -> Vec<HierarchyTree> {
        self.roots
            .iter()
            .filter_map(|&root_id| self.build_tree(root_id))
            .collect()
    }

    fn build_tree(&self, tenant_id: Uuid) -> Option<HierarchyTree> {
        self.nodes.get(&tenant_id).map(|node| {
            let children = node
                .children
                .iter()
                .filter_map(|&child_id| self.build_tree(child_id))
                .collect();

            HierarchyTree {
                tenant_id: node.tenant_id,
                level: node.level,
                depth: node.depth,
                children,
            }
        })
    }

    /// Validates hierarchy integrity.
    pub fn validate(&self) -> TenantResult<()> {
        for (tenant_id, node) in &self.nodes {
            // Check parent exists
            if let Some(parent_id) = node.parent_id {
                if !self.nodes.contains_key(&parent_id) {
                    return Err(TenantError::HierarchyError(
                        format!("Parent {} not found for tenant {}", parent_id, tenant_id)
                    ));
                }
            }

            // Check for circular references
            let mut visited = HashSet::new();
            let mut current_id = *tenant_id;

            while let Some(current_node) = self.nodes.get(&current_id) {
                if !visited.insert(current_id) {
                    return Err(TenantError::CircularHierarchy(
                        format!("Circular reference detected at {}", current_id)
                    ));
                }

                if let Some(parent_id) = current_node.parent_id {
                    current_id = parent_id;
                } else {
                    break;
                }
            }
        }

        Ok(())
    }
}

impl Default for HierarchyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tree representation of tenant hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyTree {
    pub tenant_id: Uuid,
    pub level: HierarchyLevel,
    pub depth: usize,
    pub children: Vec<HierarchyTree>,
}

impl HierarchyTree {
    /// Flattens the tree into a list.
    pub fn flatten(&self) -> Vec<Uuid> {
        let mut result = vec![self.tenant_id];
        for child in &self.children {
            result.extend(child.flatten());
        }
        result
    }

    /// Counts total nodes in the tree.
    pub fn count_nodes(&self) -> usize {
        1 + self.children.iter().map(|c| c.count_nodes()).sum::<usize>()
    }

    /// Gets maximum depth of the tree.
    pub fn max_depth(&self) -> usize {
        self.children
            .iter()
            .map(|c| c.max_depth())
            .max()
            .unwrap_or(0)
            + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::TenantBuilder;

    #[test]
    fn test_hierarchy_basic() {
        let mut manager = HierarchyManager::new();

        let org = TenantBuilder::new()
            .slug("acme")
            .name("ACME Corp")
            .contact_email("admin@acme.com")
            .build()
            .unwrap();

        assert!(manager.add_root(&org).is_ok());

        let team = TenantBuilder::new()
            .slug("acme-engineering")
            .name("Engineering Team")
            .contact_email("eng@acme.com")
            .parent_id(org.id)
            .build()
            .unwrap();

        assert!(manager.add_child(&team, HierarchyLevel::Team).is_ok());

        let children = manager.get_children(org.id);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].tenant_id, team.id);
    }

    #[test]
    fn test_hierarchy_circular_reference() {
        let mut manager = HierarchyManager::new();

        let org = TenantBuilder::new()
            .slug("acme")
            .name("ACME Corp")
            .contact_email("admin@acme.com")
            .build()
            .unwrap();

        manager.add_root(&org).unwrap();

        let team = TenantBuilder::new()
            .slug("team")
            .name("Team")
            .contact_email("team@acme.com")
            .parent_id(org.id)
            .build()
            .unwrap();

        manager.add_child(&team, HierarchyLevel::Team).unwrap();

        // Try to move org under team (circular reference)
        let result = manager.move_tenant(org.id, team.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_hierarchy_tree() {
        let tree = HierarchyTree {
            tenant_id: Uuid::new_v4(),
            level: HierarchyLevel::Organization,
            depth: 0,
            children: vec![
                HierarchyTree {
                    tenant_id: Uuid::new_v4(),
                    level: HierarchyLevel::Team,
                    depth: 1,
                    children: vec![],
                },
                HierarchyTree {
                    tenant_id: Uuid::new_v4(),
                    level: HierarchyLevel::Team,
                    depth: 1,
                    children: vec![],
                },
            ],
        };

        assert_eq!(tree.count_nodes(), 3);
        assert_eq!(tree.max_depth(), 2);
    }
}
