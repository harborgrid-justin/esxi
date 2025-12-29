/**
 * Permission Utilities
 * Enterprise-grade permission checking and validation
 */

import {
  Permission,
  PermissionAction,
  ResourceType,
  User,
  UserRole,
  PermissionCondition,
} from '../types';

/**
 * Check if user has specific permission
 */
export function hasPermission(
  user: User,
  resource: ResourceType,
  action: PermissionAction,
  context?: Record<string, any>
): boolean {
  // Super admin has all permissions
  if (user.role === UserRole.SUPER_ADMIN) {
    return true;
  }

  // Check explicit permissions
  const hasExplicitPermission = user.permissions.some((permission) => {
    if (permission.resource !== resource || permission.action !== action) {
      return false;
    }

    // Check conditions if present
    if (permission.conditions && context) {
      return evaluateConditions(permission.conditions, context);
    }

    return true;
  });

  if (hasExplicitPermission) {
    return true;
  }

  // Check role-based permissions
  return hasRolePermission(user.role, resource, action);
}

/**
 * Check if user role has permission
 */
export function hasRolePermission(
  role: UserRole,
  resource: ResourceType,
  action: PermissionAction
): boolean {
  const rolePermissions = ROLE_PERMISSIONS[role] || [];

  return rolePermissions.some(
    (perm) => perm.resource === resource && perm.action === action
  );
}

/**
 * Evaluate permission conditions
 */
export function evaluateConditions(
  conditions: PermissionCondition[],
  context: Record<string, any>
): boolean {
  return conditions.every((condition) => {
    const contextValue = getNestedValue(context, condition.field);

    switch (condition.operator) {
      case 'equals':
        return contextValue === condition.value;
      case 'not_equals':
        return contextValue !== condition.value;
      case 'in':
        return Array.isArray(condition.value) && condition.value.includes(contextValue);
      case 'not_in':
        return Array.isArray(condition.value) && !condition.value.includes(contextValue);
      case 'contains':
        return String(contextValue).includes(String(condition.value));
      case 'greater_than':
        return Number(contextValue) > Number(condition.value);
      case 'less_than':
        return Number(contextValue) < Number(condition.value);
      default:
        return false;
    }
  });
}

/**
 * Get nested value from object using dot notation
 */
function getNestedValue(obj: Record<string, any>, path: string): any {
  return path.split('.').reduce((current, key) => current?.[key], obj);
}

/**
 * Check if user can manage tenant
 */
export function canManageTenant(user: User): boolean {
  return [UserRole.SUPER_ADMIN, UserRole.TENANT_ADMIN].includes(user.role);
}

/**
 * Check if user can manage organization
 */
export function canManageOrganization(user: User, organizationId?: string): boolean {
  if ([UserRole.SUPER_ADMIN, UserRole.TENANT_ADMIN].includes(user.role)) {
    return true;
  }

  if (user.role === UserRole.ORG_ADMIN) {
    return !organizationId || user.organizationId === organizationId;
  }

  return false;
}

/**
 * Check if user can manage users
 */
export function canManageUsers(user: User, targetOrganizationId?: string): boolean {
  if ([UserRole.SUPER_ADMIN, UserRole.TENANT_ADMIN].includes(user.role)) {
    return true;
  }

  if ([UserRole.ORG_ADMIN, UserRole.ORG_MANAGER].includes(user.role)) {
    return !targetOrganizationId || user.organizationId === targetOrganizationId;
  }

  return false;
}

/**
 * Check if user can view billing
 */
export function canViewBilling(user: User): boolean {
  return [
    UserRole.SUPER_ADMIN,
    UserRole.TENANT_ADMIN,
    UserRole.ORG_ADMIN,
  ].includes(user.role);
}

/**
 * Check if user can configure SSO
 */
export function canConfigureSSO(user: User): boolean {
  return [UserRole.SUPER_ADMIN, UserRole.TENANT_ADMIN, UserRole.ORG_ADMIN].includes(
    user.role
  );
}

/**
 * Check if user can view audit logs
 */
export function canViewAuditLogs(user: User): boolean {
  return [
    UserRole.SUPER_ADMIN,
    UserRole.TENANT_ADMIN,
    UserRole.ORG_ADMIN,
  ].includes(user.role);
}

/**
 * Filter resources based on user permissions
 */
export function filterByPermissions<T extends { id: string }>(
  user: User,
  resources: T[],
  resource: ResourceType,
  action: PermissionAction
): T[] {
  if (user.role === UserRole.SUPER_ADMIN) {
    return resources;
  }

  return resources.filter((item) =>
    hasPermission(user, resource, action, { resourceId: item.id })
  );
}

/**
 * Get all permissions for a user
 */
export function getAllPermissions(user: User): Permission[] {
  const rolePermissions = ROLE_PERMISSIONS[user.role] || [];
  return [...rolePermissions, ...user.permissions];
}

/**
 * Check if user has any of the specified roles
 */
export function hasAnyRole(user: User, roles: UserRole[]): boolean {
  return roles.includes(user.role) || user.customRoles.some((r) => roles.includes(r as UserRole));
}

/**
 * Check if user has all of the specified roles
 */
export function hasAllRoles(user: User, roles: UserRole[]): boolean {
  const userRoles = [user.role, ...user.customRoles];
  return roles.every((role) => userRoles.includes(role));
}

/**
 * Default role-based permissions matrix
 */
const ROLE_PERMISSIONS: Record<UserRole, Permission[]> = {
  [UserRole.SUPER_ADMIN]: [
    // Super admin has all permissions - checked separately
  ],
  [UserRole.TENANT_ADMIN]: [
    { id: '1', resource: ResourceType.TENANT, action: PermissionAction.MANAGE },
    { id: '2', resource: ResourceType.ORGANIZATION, action: PermissionAction.MANAGE },
    { id: '3', resource: ResourceType.USER, action: PermissionAction.MANAGE },
    { id: '4', resource: ResourceType.ROLE, action: PermissionAction.MANAGE },
    { id: '5', resource: ResourceType.PERMISSION, action: PermissionAction.MANAGE },
    { id: '6', resource: ResourceType.BILLING, action: PermissionAction.MANAGE },
    { id: '7', resource: ResourceType.SSO, action: PermissionAction.MANAGE },
    { id: '8', resource: ResourceType.AUDIT, action: PermissionAction.READ },
    { id: '9', resource: ResourceType.SETTINGS, action: PermissionAction.MANAGE },
  ],
  [UserRole.ORG_ADMIN]: [
    { id: '10', resource: ResourceType.ORGANIZATION, action: PermissionAction.UPDATE, scope: 'ORGANIZATION' },
    { id: '11', resource: ResourceType.ORGANIZATION, action: PermissionAction.READ, scope: 'ORGANIZATION' },
    { id: '12', resource: ResourceType.USER, action: PermissionAction.MANAGE, scope: 'ORGANIZATION' },
    { id: '13', resource: ResourceType.ROLE, action: PermissionAction.MANAGE, scope: 'ORGANIZATION' },
    { id: '14', resource: ResourceType.SSO, action: PermissionAction.MANAGE, scope: 'ORGANIZATION' },
    { id: '15', resource: ResourceType.AUDIT, action: PermissionAction.READ, scope: 'ORGANIZATION' },
    { id: '16', resource: ResourceType.SETTINGS, action: PermissionAction.UPDATE, scope: 'ORGANIZATION' },
  ],
  [UserRole.ORG_MANAGER]: [
    { id: '17', resource: ResourceType.ORGANIZATION, action: PermissionAction.READ, scope: 'ORGANIZATION' },
    { id: '18', resource: ResourceType.USER, action: PermissionAction.CREATE, scope: 'ORGANIZATION' },
    { id: '19', resource: ResourceType.USER, action: PermissionAction.READ, scope: 'ORGANIZATION' },
    { id: '20', resource: ResourceType.USER, action: PermissionAction.UPDATE, scope: 'ORGANIZATION' },
    { id: '21', resource: ResourceType.ROLE, action: PermissionAction.READ, scope: 'ORGANIZATION' },
  ],
  [UserRole.ORG_USER]: [
    { id: '22', resource: ResourceType.ORGANIZATION, action: PermissionAction.READ, scope: 'ORGANIZATION' },
    { id: '23', resource: ResourceType.USER, action: PermissionAction.READ, scope: 'ORGANIZATION' },
  ],
  [UserRole.VIEWER]: [
    { id: '24', resource: ResourceType.ORGANIZATION, action: PermissionAction.READ, scope: 'ORGANIZATION' },
    { id: '25', resource: ResourceType.USER, action: PermissionAction.READ, scope: 'USER' },
  ],
};

/**
 * Validate permission scope
 */
export function validatePermissionScope(
  user: User,
  permission: Permission,
  targetUserId?: string,
  targetOrganizationId?: string
): boolean {
  if (!permission.scope || permission.scope === 'GLOBAL') {
    return true;
  }

  if (permission.scope === 'TENANT') {
    return user.tenantId === targetOrganizationId;
  }

  if (permission.scope === 'ORGANIZATION') {
    return user.organizationId === targetOrganizationId;
  }

  if (permission.scope === 'USER') {
    return user.id === targetUserId;
  }

  return false;
}

/**
 * Create permission from resource and action
 */
export function createPermission(
  resource: ResourceType,
  action: PermissionAction,
  scope?: Permission['scope'],
  conditions?: PermissionCondition[]
): Permission {
  return {
    id: `${resource}_${action}_${Date.now()}`,
    resource,
    action,
    scope,
    conditions,
  };
}

/**
 * Merge permissions from multiple sources
 */
export function mergePermissions(...permissionSets: Permission[][]): Permission[] {
  const merged = new Map<string, Permission>();

  for (const set of permissionSets) {
    for (const permission of set) {
      const key = `${permission.resource}_${permission.action}`;

      if (!merged.has(key)) {
        merged.set(key, permission);
      } else {
        // If permission exists, keep the one with broader scope
        const existing = merged.get(key)!;
        if (isBroaderScope(permission.scope, existing.scope)) {
          merged.set(key, permission);
        }
      }
    }
  }

  return Array.from(merged.values());
}

/**
 * Check if scope A is broader than scope B
 */
function isBroaderScope(
  scopeA?: Permission['scope'],
  scopeB?: Permission['scope']
): boolean {
  const scopeOrder = ['GLOBAL', 'TENANT', 'ORGANIZATION', 'USER'];
  const indexA = scopeOrder.indexOf(scopeA || 'GLOBAL');
  const indexB = scopeOrder.indexOf(scopeB || 'GLOBAL');
  return indexA < indexB;
}
