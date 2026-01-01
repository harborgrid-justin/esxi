/**
 * Permission Service
 * Handles access control and permission management
 */

import {
  Permission,
  PermissionSet,
  PermissionAction,
  ParticipantRole,
  Participant,
  CollaborationError,
  ErrorCode,
} from '../types';

export interface PermissionServiceConfig {
  strictMode?: boolean;
  inheritPermissions?: boolean;
}

export class PermissionService {
  private permissions: Map<string, PermissionSet> = new Map();
  private rolePermissions: Map<ParticipantRole, Set<PermissionAction>> = new Map();
  private config: Required<PermissionServiceConfig>;

  constructor(config: PermissionServiceConfig = {}) {
    this.config = {
      strictMode: config.strictMode ?? true,
      inheritPermissions: config.inheritPermissions ?? true,
    };

    this.initializeRolePermissions();
  }

  /**
   * Initialize default role permissions
   */
  private initializeRolePermissions(): void {
    // Owner has all permissions
    this.rolePermissions.set(
      ParticipantRole.OWNER,
      new Set([
        PermissionAction.READ,
        PermissionAction.WRITE,
        PermissionAction.DELETE,
        PermissionAction.COMMENT,
        PermissionAction.SHARE,
        PermissionAction.ADMIN,
      ])
    );

    // Editor can read, write, and comment
    this.rolePermissions.set(
      ParticipantRole.EDITOR,
      new Set([
        PermissionAction.READ,
        PermissionAction.WRITE,
        PermissionAction.COMMENT,
      ])
    );

    // Viewer can only read
    this.rolePermissions.set(
      ParticipantRole.VIEWER,
      new Set([PermissionAction.READ])
    );

    // Commenter can read and comment
    this.rolePermissions.set(
      ParticipantRole.COMMENTER,
      new Set([PermissionAction.READ, PermissionAction.COMMENT])
    );
  }

  /**
   * Check if a participant has permission for an action
   */
  hasPermission(
    participant: Participant,
    action: PermissionAction,
    resource: string
  ): boolean {
    // Check role-based permissions
    const rolePerms = this.rolePermissions.get(participant.role);
    if (rolePerms && rolePerms.has(action)) {
      return true;
    }

    // Check explicit permissions
    const permissionSet = this.permissions.get(participant.id);
    if (permissionSet) {
      const permission = permissionSet.permissions.find(
        (p) => p.resource === resource && p.action === action
      );

      if (permission) {
        return permission.granted && this.checkConditions(permission);
      }
    }

    // Check inherited permissions
    if (this.config.inheritPermissions && permissionSet?.inheritedFrom) {
      for (const inheritedId of permissionSet.inheritedFrom) {
        const inheritedSet = this.permissions.get(inheritedId);
        if (inheritedSet) {
          const permission = inheritedSet.permissions.find(
            (p) => p.resource === resource && p.action === action
          );

          if (permission && permission.granted) {
            return this.checkConditions(permission);
          }
        }
      }
    }

    // Strict mode: deny by default
    return !this.config.strictMode;
  }

  /**
   * Grant permission to a participant
   */
  grantPermission(
    participantId: string,
    action: PermissionAction,
    resource: string,
    conditions?: Record<string, unknown>
  ): void {
    const permissionSet = this.permissions.get(participantId) || {
      participantId,
      permissions: [],
    };

    const existingIndex = permissionSet.permissions.findIndex(
      (p) => p.resource === resource && p.action === action
    );

    const permission: Permission = {
      action,
      resource,
      granted: true,
      conditions,
    };

    if (existingIndex >= 0) {
      permissionSet.permissions[existingIndex] = permission;
    } else {
      permissionSet.permissions.push(permission);
    }

    this.permissions.set(participantId, permissionSet);
  }

  /**
   * Revoke permission from a participant
   */
  revokePermission(
    participantId: string,
    action: PermissionAction,
    resource: string
  ): void {
    const permissionSet = this.permissions.get(participantId);
    if (!permissionSet) return;

    permissionSet.permissions = permissionSet.permissions.filter(
      (p) => !(p.resource === resource && p.action === action)
    );

    this.permissions.set(participantId, permissionSet);
  }

  /**
   * Get all permissions for a participant
   */
  getPermissions(participantId: string): PermissionSet | undefined {
    return this.permissions.get(participantId);
  }

  /**
   * Set permissions for a participant
   */
  setPermissions(permissionSet: PermissionSet): void {
    this.permissions.set(permissionSet.participantId, permissionSet);
  }

  /**
   * Check permission conditions
   */
  private checkConditions(permission: Permission): boolean {
    if (!permission.conditions) {
      return true;
    }

    // Example conditions:
    // - timeRange: { start: Date, end: Date }
    // - ipWhitelist: string[]
    // - maxOperations: number

    if (permission.conditions.timeRange) {
      const { start, end } = permission.conditions.timeRange as {
        start: Date;
        end: Date;
      };
      const now = new Date();
      if (now < start || now > end) {
        return false;
      }
    }

    return true;
  }

  /**
   * Check if participant can perform action, throw error if not
   */
  requirePermission(
    participant: Participant,
    action: PermissionAction,
    resource: string
  ): void {
    if (!this.hasPermission(participant, action, resource)) {
      throw new CollaborationError(
        ErrorCode.PERMISSION_DENIED,
        `Permission denied: ${action} on ${resource}`,
        {
          participantId: participant.id,
          action,
          resource,
        }
      );
    }
  }

  /**
   * Get all actions a participant can perform on a resource
   */
  getAllowedActions(
    participant: Participant,
    resource: string
  ): PermissionAction[] {
    const actions = Object.values(PermissionAction);
    return actions.filter((action) =>
      this.hasPermission(participant, action, resource)
    );
  }

  /**
   * Check if participant has any of the specified permissions
   */
  hasAnyPermission(
    participant: Participant,
    actions: PermissionAction[],
    resource: string
  ): boolean {
    return actions.some((action) =>
      this.hasPermission(participant, action, resource)
    );
  }

  /**
   * Check if participant has all of the specified permissions
   */
  hasAllPermissions(
    participant: Participant,
    actions: PermissionAction[],
    resource: string
  ): boolean {
    return actions.every((action) =>
      this.hasPermission(participant, action, resource)
    );
  }

  /**
   * Clone permissions from one participant to another
   */
  clonePermissions(fromId: string, toId: string): void {
    const sourceSet = this.permissions.get(fromId);
    if (!sourceSet) return;

    const clonedSet: PermissionSet = {
      participantId: toId,
      permissions: [...sourceSet.permissions],
      inheritedFrom: [fromId],
    };

    this.permissions.set(toId, clonedSet);
  }

  /**
   * Set permission inheritance
   */
  setInheritance(participantId: string, inheritFrom: string[]): void {
    const permissionSet = this.permissions.get(participantId) || {
      participantId,
      permissions: [],
    };

    permissionSet.inheritedFrom = inheritFrom;
    this.permissions.set(participantId, permissionSet);
  }

  /**
   * Clear all permissions for a participant
   */
  clearPermissions(participantId: string): void {
    this.permissions.delete(participantId);
  }

  /**
   * Reset service state
   */
  reset(): void {
    this.permissions.clear();
    this.initializeRolePermissions();
  }

  /**
   * Get role permissions
   */
  getRolePermissions(role: ParticipantRole): Set<PermissionAction> {
    return this.rolePermissions.get(role) || new Set();
  }

  /**
   * Update role permissions
   */
  updateRolePermissions(
    role: ParticipantRole,
    actions: PermissionAction[]
  ): void {
    this.rolePermissions.set(role, new Set(actions));
  }
}
