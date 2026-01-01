/**
 * Permission Resolver - Dynamic Permission Resolution
 * Resolve and cache permissions with support for hierarchies and conditions
 */

import { Permission } from '../types';
import { rbacEngine } from './RBACEngine';

// ============================================================================
// Types
// ============================================================================

export interface PermissionDefinition {
  permission: Permission;
  description: string;
  category: string;
  implies?: Permission[]; // Permissions this grants
  requires?: Permission[]; // Permissions required to have this
  metadata: Record<string, unknown>;
}

export interface ResolvedPermissions {
  direct: Permission[];
  implied: Permission[];
  all: Permission[];
  metadata: Record<Permission, PermissionDefinition>;
}

export interface PermissionCheck {
  userId: string;
  permission: Permission;
  resource?: string;
  context?: Record<string, unknown>;
}

export interface PermissionCheckResult {
  allowed: boolean;
  source: 'DIRECT' | 'IMPLIED' | 'DENIED';
  reason: string;
}

// ============================================================================
// Permission Resolver Implementation
// ============================================================================

export class PermissionResolver {
  private definitions: Map<Permission, PermissionDefinition> = new Map();
  private cache: Map<string, ResolvedPermissions> = new Map();
  private cacheTTL = 5 * 60 * 1000; // 5 minutes
  private cacheTimestamps: Map<string, number> = new Map();

  constructor() {
    this.initializeDefaultPermissions();
  }

  /**
   * Define permission
   */
  definePermission(definition: PermissionDefinition): void {
    this.definitions.set(definition.permission, definition);
    this.invalidateCache();
  }

  /**
   * Get permission definition
   */
  getDefinition(permission: Permission): PermissionDefinition | undefined {
    return this.definitions.get(permission);
  }

  /**
   * Resolve user permissions (with implications)
   */
  async resolvePermissions(userId: string): Promise<ResolvedPermissions> {
    // Check cache
    const cached = this.getFromCache(userId);
    if (cached) {
      return cached;
    }

    // Get direct permissions from RBAC
    const directPermissions = rbacEngine.getUserPermissions(userId);

    // Resolve implied permissions
    const impliedPermissions = this.resolveImpliedPermissions(directPermissions);

    // Combine all permissions
    const allPermissions = Array.from(
      new Set([...directPermissions, ...impliedPermissions])
    );

    // Build metadata map
    const metadata: Record<Permission, PermissionDefinition> = {};
    for (const permission of allPermissions) {
      const def = this.definitions.get(permission);
      if (def) {
        metadata[permission] = def;
      }
    }

    const result: ResolvedPermissions = {
      direct: directPermissions,
      implied: impliedPermissions,
      all: allPermissions,
      metadata,
    };

    // Cache result
    this.setCache(userId, result);

    return result;
  }

  /**
   * Check permission
   */
  async check(check: PermissionCheck): Promise<PermissionCheckResult> {
    const resolved = await this.resolvePermissions(check.userId);

    // Check direct permission
    if (resolved.direct.includes(check.permission)) {
      return {
        allowed: true,
        source: 'DIRECT',
        reason: `User has direct ${check.permission} permission`,
      };
    }

    // Check implied permission
    if (resolved.implied.includes(check.permission)) {
      return {
        allowed: true,
        source: 'IMPLIED',
        reason: `User has ${check.permission} through permission implication`,
      };
    }

    return {
      allowed: false,
      source: 'DENIED',
      reason: `User does not have ${check.permission} permission`,
    };
  }

  /**
   * Check if user has permission
   */
  async hasPermission(userId: string, permission: Permission): Promise<boolean> {
    const result = await this.check({ userId, permission });
    return result.allowed;
  }

  /**
   * Check if user has all permissions
   */
  async hasAllPermissions(userId: string, permissions: Permission[]): Promise<boolean> {
    const resolved = await this.resolvePermissions(userId);
    return permissions.every((p) => resolved.all.includes(p));
  }

  /**
   * Check if user has any permission
   */
  async hasAnyPermission(userId: string, permissions: Permission[]): Promise<boolean> {
    const resolved = await this.resolvePermissions(userId);
    return permissions.some((p) => resolved.all.includes(p));
  }

  /**
   * Get missing permissions
   */
  async getMissingPermissions(
    userId: string,
    required: Permission[]
  ): Promise<Permission[]> {
    const resolved = await this.resolvePermissions(userId);
    return required.filter((p) => !resolved.all.includes(p));
  }

  /**
   * Get permissions by category
   */
  async getPermissionsByCategory(
    userId: string,
    category: string
  ): Promise<Permission[]> {
    const resolved = await this.resolvePermissions(userId);
    return resolved.all.filter((p) => {
      const def = this.definitions.get(p);
      return def?.category === category;
    });
  }

  /**
   * Get all defined permissions
   */
  getAllPermissions(): PermissionDefinition[] {
    return Array.from(this.definitions.values());
  }

  /**
   * Get permission categories
   */
  getCategories(): string[] {
    const categories = new Set<string>();
    for (const def of this.definitions.values()) {
      categories.add(def.category);
    }
    return Array.from(categories);
  }

  /**
   * Invalidate cache for user
   */
  invalidateCacheForUser(userId: string): void {
    this.cache.delete(userId);
    this.cacheTimestamps.delete(userId);
  }

  /**
   * Invalidate all cache
   */
  invalidateCache(): void {
    this.cache.clear();
    this.cacheTimestamps.clear();
  }

  // ============================================================================
  // Private Helper Methods
  // ============================================================================

  private resolveImpliedPermissions(direct: Permission[]): Permission[] {
    const implied = new Set<Permission>();
    const visited = new Set<Permission>();

    const traverse = (permission: Permission) => {
      if (visited.has(permission)) {
        return; // Prevent circular implications
      }
      visited.add(permission);

      const def = this.definitions.get(permission);
      if (def?.implies) {
        for (const impliedPerm of def.implies) {
          implied.add(impliedPerm);
          traverse(impliedPerm);
        }
      }
    };

    for (const permission of direct) {
      traverse(permission);
    }

    return Array.from(implied);
  }

  private getFromCache(userId: string): ResolvedPermissions | null {
    const cached = this.cache.get(userId);
    const timestamp = this.cacheTimestamps.get(userId);

    if (!cached || !timestamp) {
      return null;
    }

    // Check if cache expired
    if (Date.now() - timestamp > this.cacheTTL) {
      this.cache.delete(userId);
      this.cacheTimestamps.delete(userId);
      return null;
    }

    return cached;
  }

  private setCache(userId: string, permissions: ResolvedPermissions): void {
    this.cache.set(userId, permissions);
    this.cacheTimestamps.set(userId, Date.now());
  }

  private initializeDefaultPermissions(): void {
    // Resource permissions
    this.definePermission({
      permission: Permission.CREATE,
      description: 'Create new resources',
      category: 'Resource',
      metadata: {},
    });

    this.definePermission({
      permission: Permission.READ,
      description: 'Read resources',
      category: 'Resource',
      metadata: {},
    });

    this.definePermission({
      permission: Permission.UPDATE,
      description: 'Update existing resources',
      category: 'Resource',
      metadata: {},
    });

    this.definePermission({
      permission: Permission.DELETE,
      description: 'Delete resources',
      category: 'Resource',
      metadata: {},
    });

    // Admin permissions
    this.definePermission({
      permission: Permission.MANAGE_USERS,
      description: 'Manage user accounts',
      category: 'Administration',
      implies: [Permission.READ],
      metadata: {},
    });

    this.definePermission({
      permission: Permission.MANAGE_ROLES,
      description: 'Manage roles and permissions',
      category: 'Administration',
      implies: [Permission.READ],
      metadata: {},
    });

    this.definePermission({
      permission: Permission.MANAGE_POLICIES,
      description: 'Manage security policies',
      category: 'Administration',
      implies: [Permission.READ],
      metadata: {},
    });

    // Security permissions
    this.definePermission({
      permission: Permission.VIEW_AUDIT_LOGS,
      description: 'View audit logs',
      category: 'Security',
      metadata: {},
    });

    this.definePermission({
      permission: Permission.MANAGE_SECURITY,
      description: 'Manage security settings',
      category: 'Security',
      implies: [Permission.VIEW_AUDIT_LOGS],
      metadata: {},
    });

    this.definePermission({
      permission: Permission.MANAGE_COMPLIANCE,
      description: 'Manage compliance settings',
      category: 'Security',
      implies: [Permission.VIEW_AUDIT_LOGS],
      metadata: {},
    });

    // Data permissions
    this.definePermission({
      permission: Permission.EXPORT_DATA,
      description: 'Export data',
      category: 'Data',
      requires: [Permission.READ],
      metadata: {},
    });

    this.definePermission({
      permission: Permission.IMPORT_DATA,
      description: 'Import data',
      category: 'Data',
      requires: [Permission.CREATE],
      metadata: {},
    });

    this.definePermission({
      permission: Permission.PURGE_DATA,
      description: 'Permanently delete data',
      category: 'Data',
      requires: [Permission.DELETE],
      metadata: {},
    });
  }

  /**
   * Set cache TTL
   */
  setCacheTTL(ttl: number): void {
    this.cacheTTL = ttl;
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): {
    size: number;
    hitRate: number;
  } {
    return {
      size: this.cache.size,
      hitRate: 0, // Would need to track hits/misses
    };
  }
}

// Export singleton instance
export const permissionResolver = new PermissionResolver();
