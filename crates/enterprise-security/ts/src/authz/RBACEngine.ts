/**
 * RBAC Engine - Role-Based Access Control
 * Enterprise role and permission management
 */

import { nanoid } from 'nanoid';
import { Permission, Role } from '../types';

// ============================================================================
// Types
// ============================================================================

export interface RoleAssignment {
  id: string;
  userId: string;
  roleId: string;
  assignedBy: string;
  assignedAt: Date;
  expiresAt?: Date;
  conditions?: AssignmentCondition[];
  metadata: Record<string, unknown>;
}

export interface AssignmentCondition {
  type: 'TIME' | 'LOCATION' | 'IP' | 'DEVICE';
  value: string;
  operator: 'EQUALS' | 'NOT_EQUALS' | 'IN' | 'NOT_IN';
}

export interface RoleHierarchy {
  roleId: string;
  parentRoleIds: string[];
  childRoleIds: string[];
}

// ============================================================================
// RBAC Engine Implementation
// ============================================================================

export class RBACEngine {
  private roles: Map<string, Role> = new Map();
  private assignments: Map<string, RoleAssignment[]> = new Map();
  private hierarchies: Map<string, RoleHierarchy> = new Map();

  /**
   * Create role
   */
  async createRole(
    name: string,
    description: string,
    permissions: Permission[],
    inherits?: string[]
  ): Promise<Role> {
    const role: Role = {
      id: nanoid(),
      name,
      description,
      permissions,
      inherits,
      metadata: {},
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    this.roles.set(role.id, role);

    // Update hierarchy
    if (inherits && inherits.length > 0) {
      const hierarchy: RoleHierarchy = {
        roleId: role.id,
        parentRoleIds: inherits,
        childRoleIds: [],
      };
      this.hierarchies.set(role.id, hierarchy);

      // Update parent hierarchies
      for (const parentId of inherits) {
        const parentHierarchy = this.hierarchies.get(parentId) || {
          roleId: parentId,
          parentRoleIds: [],
          childRoleIds: [],
        };
        parentHierarchy.childRoleIds.push(role.id);
        this.hierarchies.set(parentId, parentHierarchy);
      }
    }

    return role;
  }

  /**
   * Update role
   */
  async updateRole(
    roleId: string,
    updates: Partial<Omit<Role, 'id' | 'createdAt'>>
  ): Promise<Role> {
    const role = this.roles.get(roleId);
    if (!role) {
      throw new Error('Role not found');
    }

    const updatedRole: Role = {
      ...role,
      ...updates,
      id: role.id,
      createdAt: role.createdAt,
      updatedAt: new Date(),
    };

    this.roles.set(roleId, updatedRole);

    // Update hierarchy if inherits changed
    if (updates.inherits !== undefined) {
      this.updateHierarchy(roleId, updates.inherits || []);
    }

    return updatedRole;
  }

  /**
   * Delete role
   */
  async deleteRole(roleId: string): Promise<void> {
    const role = this.roles.get(roleId);
    if (!role) {
      return;
    }

    // Check if role is assigned to any users
    for (const assignments of this.assignments.values()) {
      if (assignments.some((a) => a.roleId === roleId)) {
        throw new Error('Cannot delete role that is assigned to users');
      }
    }

    // Check if role has children in hierarchy
    const hierarchy = this.hierarchies.get(roleId);
    if (hierarchy && hierarchy.childRoleIds.length > 0) {
      throw new Error('Cannot delete role with child roles');
    }

    // Remove from parent hierarchies
    if (hierarchy) {
      for (const parentId of hierarchy.parentRoleIds) {
        const parentHierarchy = this.hierarchies.get(parentId);
        if (parentHierarchy) {
          parentHierarchy.childRoleIds = parentHierarchy.childRoleIds.filter(
            (id) => id !== roleId
          );
        }
      }
    }

    this.roles.delete(roleId);
    this.hierarchies.delete(roleId);
  }

  /**
   * Assign role to user
   */
  async assignRole(
    userId: string,
    roleId: string,
    assignedBy: string,
    expiresAt?: Date,
    conditions?: AssignmentCondition[]
  ): Promise<RoleAssignment> {
    const role = this.roles.get(roleId);
    if (!role) {
      throw new Error('Role not found');
    }

    const assignment: RoleAssignment = {
      id: nanoid(),
      userId,
      roleId,
      assignedBy,
      assignedAt: new Date(),
      expiresAt,
      conditions,
      metadata: {},
    };

    const userAssignments = this.assignments.get(userId) || [];
    userAssignments.push(assignment);
    this.assignments.set(userId, userAssignments);

    return assignment;
  }

  /**
   * Revoke role from user
   */
  async revokeRole(userId: string, roleId: string): Promise<void> {
    const assignments = this.assignments.get(userId);
    if (!assignments) {
      return;
    }

    const filtered = assignments.filter((a) => a.roleId !== roleId);
    if (filtered.length > 0) {
      this.assignments.set(userId, filtered);
    } else {
      this.assignments.delete(userId);
    }
  }

  /**
   * Get user roles (including inherited)
   */
  getUserRoles(userId: string): Role[] {
    const assignments = this.getActiveAssignments(userId);
    const roleIds = new Set<string>();

    // Get directly assigned roles
    for (const assignment of assignments) {
      roleIds.add(assignment.roleId);

      // Get inherited roles
      const inherited = this.getInheritedRoles(assignment.roleId);
      for (const inheritedRole of inherited) {
        roleIds.add(inheritedRole.id);
      }
    }

    return Array.from(roleIds)
      .map((id) => this.roles.get(id))
      .filter((role): role is Role => role !== undefined);
  }

  /**
   * Get user permissions (aggregated from all roles)
   */
  getUserPermissions(userId: string): Permission[] {
    const roles = this.getUserRoles(userId);
    const permissions = new Set<Permission>();

    for (const role of roles) {
      for (const permission of role.permissions) {
        permissions.add(permission);
      }
    }

    return Array.from(permissions);
  }

  /**
   * Check if user has permission
   */
  hasPermission(userId: string, permission: Permission): boolean {
    const permissions = this.getUserPermissions(userId);
    return permissions.includes(permission);
  }

  /**
   * Check if user has all permissions
   */
  hasAllPermissions(userId: string, permissions: Permission[]): boolean {
    const userPermissions = this.getUserPermissions(userId);
    return permissions.every((p) => userPermissions.includes(p));
  }

  /**
   * Check if user has any permission
   */
  hasAnyPermission(userId: string, permissions: Permission[]): boolean {
    const userPermissions = this.getUserPermissions(userId);
    return permissions.some((p) => userPermissions.includes(p));
  }

  /**
   * Check if user has role
   */
  hasRole(userId: string, roleId: string): boolean {
    const roles = this.getUserRoles(userId);
    return roles.some((r) => r.id === roleId);
  }

  /**
   * Get all roles
   */
  getAllRoles(): Role[] {
    return Array.from(this.roles.values());
  }

  /**
   * Get role by ID
   */
  getRole(roleId: string): Role | undefined {
    return this.roles.get(roleId);
  }

  /**
   * Get role by name
   */
  getRoleByName(name: string): Role | undefined {
    return Array.from(this.roles.values()).find((r) => r.name === name);
  }

  /**
   * Get role hierarchy
   */
  getRoleHierarchy(roleId: string): RoleHierarchy | undefined {
    return this.hierarchies.get(roleId);
  }

  /**
   * Get all user assignments
   */
  getUserAssignments(userId: string): RoleAssignment[] {
    return this.assignments.get(userId) || [];
  }

  // ============================================================================
  // Private Helper Methods
  // ============================================================================

  private getActiveAssignments(userId: string): RoleAssignment[] {
    const assignments = this.assignments.get(userId) || [];
    const now = new Date();

    return assignments.filter((a) => {
      // Check expiration
      if (a.expiresAt && a.expiresAt < now) {
        return false;
      }

      // Could check conditions here
      return true;
    });
  }

  private getInheritedRoles(roleId: string): Role[] {
    const role = this.roles.get(roleId);
    if (!role || !role.inherits || role.inherits.length === 0) {
      return [];
    }

    const inherited: Role[] = [];
    const visited = new Set<string>();

    const traverse = (id: string) => {
      if (visited.has(id)) {
        return; // Prevent circular inheritance
      }
      visited.add(id);

      const r = this.roles.get(id);
      if (!r) {
        return;
      }

      inherited.push(r);

      if (r.inherits) {
        for (const parentId of r.inherits) {
          traverse(parentId);
        }
      }
    };

    for (const parentId of role.inherits) {
      traverse(parentId);
    }

    return inherited;
  }

  private updateHierarchy(roleId: string, newParents: string[]): void {
    const hierarchy = this.hierarchies.get(roleId) || {
      roleId,
      parentRoleIds: [],
      childRoleIds: [],
    };

    // Remove from old parents
    for (const oldParentId of hierarchy.parentRoleIds) {
      if (!newParents.includes(oldParentId)) {
        const parentHierarchy = this.hierarchies.get(oldParentId);
        if (parentHierarchy) {
          parentHierarchy.childRoleIds = parentHierarchy.childRoleIds.filter(
            (id) => id !== roleId
          );
        }
      }
    }

    // Add to new parents
    for (const newParentId of newParents) {
      if (!hierarchy.parentRoleIds.includes(newParentId)) {
        const parentHierarchy = this.hierarchies.get(newParentId) || {
          roleId: newParentId,
          parentRoleIds: [],
          childRoleIds: [],
        };
        parentHierarchy.childRoleIds.push(roleId);
        this.hierarchies.set(newParentId, parentHierarchy);
      }
    }

    hierarchy.parentRoleIds = newParents;
    this.hierarchies.set(roleId, hierarchy);
  }

  /**
   * Clean up expired assignments
   */
  async cleanupExpiredAssignments(): Promise<number> {
    let cleaned = 0;
    const now = new Date();

    for (const [userId, assignments] of this.assignments.entries()) {
      const filtered = assignments.filter((a) => !a.expiresAt || a.expiresAt >= now);

      if (filtered.length < assignments.length) {
        cleaned += assignments.length - filtered.length;
        if (filtered.length > 0) {
          this.assignments.set(userId, filtered);
        } else {
          this.assignments.delete(userId);
        }
      }
    }

    return cleaned;
  }
}

// Export singleton instance
export const rbacEngine = new RBACEngine();
