/**
 * Scope Manager - Resource Scope Management
 * Manage access scopes, ownership, and resource-level permissions
 */

import { nanoid } from 'nanoid';
import { Permission } from '../types';

// ============================================================================
// Types
// ============================================================================

export enum ScopeType {
  GLOBAL = 'GLOBAL',
  ORGANIZATION = 'ORGANIZATION',
  TEAM = 'TEAM',
  PROJECT = 'PROJECT',
  RESOURCE = 'RESOURCE',
  PERSONAL = 'PERSONAL',
}

export interface Scope {
  id: string;
  type: ScopeType;
  name: string;
  description?: string;
  parentId?: string;
  ownerId?: string;
  metadata: Record<string, unknown>;
  createdAt: Date;
  updatedAt: Date;
}

export interface ScopePermission {
  id: string;
  scopeId: string;
  userId?: string;
  roleId?: string;
  permissions: Permission[];
  conditions?: ScopeCondition[];
  grantedBy: string;
  grantedAt: Date;
  expiresAt?: Date;
}

export interface ScopeCondition {
  type: 'TIME' | 'LOCATION' | 'IP' | 'RESOURCE_STATE';
  value: string;
  operator: 'EQUALS' | 'IN' | 'MATCHES';
}

export interface ResourceAccess {
  resource: string;
  scopes: string[];
  permissions: Permission[];
  effectivePermissions: Permission[];
}

// ============================================================================
// Scope Manager Implementation
// ============================================================================

export class ScopeManager {
  private scopes: Map<string, Scope> = new Map();
  private scopePermissions: Map<string, ScopePermission[]> = new Map();
  private resourceScopes: Map<string, string[]> = new Map();

  /**
   * Create scope
   */
  async createScope(
    type: ScopeType,
    name: string,
    ownerId?: string,
    parentId?: string,
    description?: string
  ): Promise<Scope> {
    // Validate parent exists if specified
    if (parentId && !this.scopes.has(parentId)) {
      throw new Error('Parent scope not found');
    }

    const scope: Scope = {
      id: nanoid(),
      type,
      name,
      description,
      parentId,
      ownerId,
      metadata: {},
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    this.scopes.set(scope.id, scope);
    return scope;
  }

  /**
   * Update scope
   */
  async updateScope(
    scopeId: string,
    updates: Partial<Omit<Scope, 'id' | 'createdAt'>>
  ): Promise<Scope> {
    const scope = this.scopes.get(scopeId);
    if (!scope) {
      throw new Error('Scope not found');
    }

    const updatedScope: Scope = {
      ...scope,
      ...updates,
      id: scope.id,
      createdAt: scope.createdAt,
      updatedAt: new Date(),
    };

    this.scopes.set(scopeId, updatedScope);
    return updatedScope;
  }

  /**
   * Delete scope
   */
  async deleteScope(scopeId: string): Promise<void> {
    // Check for child scopes
    const children = this.getChildScopes(scopeId);
    if (children.length > 0) {
      throw new Error('Cannot delete scope with child scopes');
    }

    this.scopes.delete(scopeId);
    this.scopePermissions.delete(scopeId);

    // Remove from resource mappings
    for (const [resource, scopes] of this.resourceScopes.entries()) {
      const filtered = scopes.filter((s) => s !== scopeId);
      if (filtered.length > 0) {
        this.resourceScopes.set(resource, filtered);
      } else {
        this.resourceScopes.delete(resource);
      }
    }
  }

  /**
   * Grant permissions in scope
   */
  async grantPermission(
    scopeId: string,
    permissions: Permission[],
    grantedBy: string,
    options: {
      userId?: string;
      roleId?: string;
      conditions?: ScopeCondition[];
      expiresAt?: Date;
    }
  ): Promise<ScopePermission> {
    if (!this.scopes.has(scopeId)) {
      throw new Error('Scope not found');
    }

    if (!options.userId && !options.roleId) {
      throw new Error('Either userId or roleId must be specified');
    }

    const permission: ScopePermission = {
      id: nanoid(),
      scopeId,
      userId: options.userId,
      roleId: options.roleId,
      permissions,
      conditions: options.conditions,
      grantedBy,
      grantedAt: new Date(),
      expiresAt: options.expiresAt,
    };

    const scopePerms = this.scopePermissions.get(scopeId) || [];
    scopePerms.push(permission);
    this.scopePermissions.set(scopeId, scopePerms);

    return permission;
  }

  /**
   * Revoke permission
   */
  async revokePermission(permissionId: string): Promise<void> {
    for (const [scopeId, permissions] of this.scopePermissions.entries()) {
      const filtered = permissions.filter((p) => p.id !== permissionId);
      if (filtered.length < permissions.length) {
        if (filtered.length > 0) {
          this.scopePermissions.set(scopeId, filtered);
        } else {
          this.scopePermissions.delete(scopeId);
        }
        break;
      }
    }
  }

  /**
   * Associate resource with scopes
   */
  async assignResourceToScope(resource: string, scopeId: string): Promise<void> {
    if (!this.scopes.has(scopeId)) {
      throw new Error('Scope not found');
    }

    const scopes = this.resourceScopes.get(resource) || [];
    if (!scopes.includes(scopeId)) {
      scopes.push(scopeId);
      this.resourceScopes.set(resource, scopes);
    }
  }

  /**
   * Remove resource from scope
   */
  async removeResourceFromScope(resource: string, scopeId: string): Promise<void> {
    const scopes = this.resourceScopes.get(resource);
    if (!scopes) {
      return;
    }

    const filtered = scopes.filter((s) => s !== scopeId);
    if (filtered.length > 0) {
      this.resourceScopes.set(resource, filtered);
    } else {
      this.resourceScopes.delete(resource);
    }
  }

  /**
   * Get user permissions in scope
   */
  async getUserPermissionsInScope(
    userId: string,
    scopeId: string
  ): Promise<Permission[]> {
    const permissions = new Set<Permission>();
    const scopes = this.getScopeHierarchy(scopeId);

    for (const scope of scopes) {
      const scopePerms = this.scopePermissions.get(scope.id) || [];

      for (const perm of scopePerms) {
        // Check if permission applies to user
        if (perm.userId === userId || perm.roleId) {
          // Check conditions
          if (!perm.conditions || this.evaluateConditions(perm.conditions)) {
            // Check expiration
            if (!perm.expiresAt || perm.expiresAt > new Date()) {
              for (const p of perm.permissions) {
                permissions.add(p);
              }
            }
          }
        }
      }
    }

    return Array.from(permissions);
  }

  /**
   * Get user access to resource
   */
  async getUserResourceAccess(userId: string, resource: string): Promise<ResourceAccess> {
    const scopeIds = this.resourceScopes.get(resource) || [];
    const allPermissions = new Set<Permission>();

    for (const scopeId of scopeIds) {
      const permissions = await this.getUserPermissionsInScope(userId, scopeId);
      for (const p of permissions) {
        allPermissions.add(p);
      }
    }

    return {
      resource,
      scopes: scopeIds,
      permissions: Array.from(allPermissions),
      effectivePermissions: Array.from(allPermissions),
    };
  }

  /**
   * Check if user can access resource
   */
  async canAccessResource(
    userId: string,
    resource: string,
    permission: Permission
  ): Promise<boolean> {
    const access = await this.getUserResourceAccess(userId, resource);
    return access.effectivePermissions.includes(permission);
  }

  /**
   * Get scope hierarchy (current + parents)
   */
  getScopeHierarchy(scopeId: string): Scope[] {
    const hierarchy: Scope[] = [];
    const visited = new Set<string>();

    let currentId: string | undefined = scopeId;

    while (currentId) {
      if (visited.has(currentId)) {
        break; // Prevent circular references
      }
      visited.add(currentId);

      const scope = this.scopes.get(currentId);
      if (!scope) {
        break;
      }

      hierarchy.push(scope);
      currentId = scope.parentId;
    }

    return hierarchy;
  }

  /**
   * Get child scopes
   */
  getChildScopes(scopeId: string): Scope[] {
    return Array.from(this.scopes.values()).filter((s) => s.parentId === scopeId);
  }

  /**
   * Get all scopes
   */
  getAllScopes(): Scope[] {
    return Array.from(this.scopes.values());
  }

  /**
   * Get scope by ID
   */
  getScope(scopeId: string): Scope | undefined {
    return this.scopes.get(scopeId);
  }

  /**
   * Get scopes by type
   */
  getScopesByType(type: ScopeType): Scope[] {
    return Array.from(this.scopes.values()).filter((s) => s.type === type);
  }

  /**
   * Get scopes by owner
   */
  getScopesByOwner(ownerId: string): Scope[] {
    return Array.from(this.scopes.values()).filter((s) => s.ownerId === ownerId);
  }

  /**
   * Get resource scopes
   */
  getResourceScopes(resource: string): Scope[] {
    const scopeIds = this.resourceScopes.get(resource) || [];
    return scopeIds
      .map((id) => this.scopes.get(id))
      .filter((s): s is Scope => s !== undefined);
  }

  // ============================================================================
  // Private Helper Methods
  // ============================================================================

  private evaluateConditions(conditions: ScopeCondition[]): boolean {
    // Simplified condition evaluation
    // In production, this would evaluate actual conditions
    return true;
  }

  /**
   * Transfer scope ownership
   */
  async transferOwnership(scopeId: string, newOwnerId: string): Promise<void> {
    const scope = this.scopes.get(scopeId);
    if (!scope) {
      throw new Error('Scope not found');
    }

    scope.ownerId = newOwnerId;
    scope.updatedAt = new Date();
  }

  /**
   * Get scope permissions
   */
  getScopePermissions(scopeId: string): ScopePermission[] {
    return this.scopePermissions.get(scopeId) || [];
  }

  /**
   * Clean up expired permissions
   */
  async cleanupExpiredPermissions(): Promise<number> {
    let cleaned = 0;
    const now = new Date();

    for (const [scopeId, permissions] of this.scopePermissions.entries()) {
      const filtered = permissions.filter((p) => !p.expiresAt || p.expiresAt >= now);

      if (filtered.length < permissions.length) {
        cleaned += permissions.length - filtered.length;
        if (filtered.length > 0) {
          this.scopePermissions.set(scopeId, filtered);
        } else {
          this.scopePermissions.delete(scopeId);
        }
      }
    }

    return cleaned;
  }
}

// Export singleton instance
export const scopeManager = new ScopeManager();
