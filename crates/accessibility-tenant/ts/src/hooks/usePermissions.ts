/**
 * usePermissions Hook
 * Custom hook for permission checking and management
 */

import { useMemo } from 'react';
import { useTenantContext } from '../context/TenantContext';
import {
  Permission,
  PermissionAction,
  ResourceType,
  UserRole,
} from '../types';
import {
  hasPermission,
  canManageTenant,
  canManageOrganization,
  canManageUsers,
  canViewBilling,
  canConfigureSSO,
  canViewAuditLogs,
  hasAnyRole,
  getAllPermissions,
} from '../utils/permissions';

export function usePermissions() {
  const { user } = useTenantContext();

  const permissions = useMemo(() => {
    if (!user) return [];
    return getAllPermissions(user);
  }, [user]);

  const checkPermission = (
    resource: ResourceType,
    action: PermissionAction,
    context?: Record<string, any>
  ): boolean => {
    if (!user) return false;
    return hasPermission(user, resource, action, context);
  };

  const checkMultiplePermissions = (
    checks: Array<{ resource: ResourceType; action: PermissionAction }>,
    requireAll = false
  ): boolean => {
    if (!user) return false;

    if (requireAll) {
      return checks.every((check) => hasPermission(user, check.resource, check.action));
    }

    return checks.some((check) => hasPermission(user, check.resource, check.action));
  };

  const checkRole = (roles: UserRole | UserRole[]): boolean => {
    if (!user) return false;
    const roleArray = Array.isArray(roles) ? roles : [roles];
    return hasAnyRole(user, roleArray);
  };

  const can = {
    manageTenant: () => (user ? canManageTenant(user) : false),
    manageOrganization: (organizationId?: string) =>
      user ? canManageOrganization(user, organizationId) : false,
    manageUsers: (organizationId?: string) =>
      user ? canManageUsers(user, organizationId) : false,
    viewBilling: () => (user ? canViewBilling(user) : false),
    configureSSO: () => (user ? canConfigureSSO(user) : false),
    viewAuditLogs: () => (user ? canViewAuditLogs(user) : false),
  };

  const isSuperAdmin = user?.role === UserRole.SUPER_ADMIN;
  const isTenantAdmin = user?.role === UserRole.TENANT_ADMIN;
  const isOrgAdmin = user?.role === UserRole.ORG_ADMIN;
  const isOrgManager = user?.role === UserRole.ORG_MANAGER;

  return {
    permissions,
    checkPermission,
    checkMultiplePermissions,
    checkRole,
    can,
    isSuperAdmin,
    isTenantAdmin,
    isOrgAdmin,
    isOrgManager,
    hasPermission: checkPermission,
    hasRole: checkRole,
  };
}

/**
 * Hook for filtering data based on permissions
 */
export function usePermissionFilter() {
  const { user } = useTenantContext();

  const filterByPermission = <T extends { id: string }>(
    items: T[],
    resource: ResourceType,
    action: PermissionAction
  ): T[] => {
    if (!user) return [];

    // Super admin can see everything
    if (user.role === UserRole.SUPER_ADMIN) {
      return items;
    }

    return items.filter((item) =>
      hasPermission(user, resource, action, { resourceId: item.id })
    );
  };

  return {
    filterByPermission,
  };
}

/**
 * Hook for permission-based UI control
 */
export function usePermissionGate(
  resource: ResourceType,
  action: PermissionAction,
  context?: Record<string, any>
) {
  const { user } = useTenantContext();

  const allowed = useMemo(() => {
    if (!user) return false;
    return hasPermission(user, resource, action, context);
  }, [user, resource, action, context]);

  return {
    allowed,
    denied: !allowed,
  };
}

/**
 * Hook for role-based UI control
 */
export function useRoleGate(roles: UserRole | UserRole[]) {
  const { user } = useTenantContext();

  const allowed = useMemo(() => {
    if (!user) return false;
    const roleArray = Array.isArray(roles) ? roles : [roles];
    return hasAnyRole(user, roleArray);
  }, [user, roles]);

  return {
    allowed,
    denied: !allowed,
  };
}

export default usePermissions;
