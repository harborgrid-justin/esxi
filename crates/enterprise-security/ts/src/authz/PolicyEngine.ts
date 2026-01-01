/**
 * Policy Engine - Unified Policy Evaluation
 * Combines RBAC and ABAC for comprehensive access control
 */

import { rbacEngine } from './RBACEngine';
import { abacEngine, type AttributeContext } from './ABACEngine';
import { Permission } from '../types';

// ============================================================================
// Types
// ============================================================================

export enum PolicyMode {
  RBAC_ONLY = 'RBAC_ONLY',
  ABAC_ONLY = 'ABAC_ONLY',
  RBAC_AND_ABAC = 'RBAC_AND_ABAC', // Both must allow
  RBAC_OR_ABAC = 'RBAC_OR_ABAC', // Either can allow
}

export interface PolicyEvaluationContext {
  userId: string;
  action: string;
  resource: string;
  resourceAttributes?: Record<string, any>;
  userAttributes?: Record<string, any>;
  environmentAttributes?: Record<string, any>;
  requiredPermissions?: Permission[];
}

export interface PolicyEvaluationResult {
  allowed: boolean;
  mode: PolicyMode;
  rbacResult?: {
    allowed: boolean;
    permissions: Permission[];
    roles: string[];
  };
  abacResult?: {
    allowed: boolean;
    effect: 'ALLOW' | 'DENY' | 'NOT_APPLICABLE';
    matchedPolicies: string[];
  };
  reason: string;
  metadata: Record<string, unknown>;
}

export interface PolicyEngineConfig {
  mode: PolicyMode;
  defaultAllow: boolean;
  enableAudit: boolean;
  strictMode: boolean;
}

// ============================================================================
// Policy Engine Implementation
// ============================================================================

export class PolicyEngine {
  private config: PolicyEngineConfig = {
    mode: PolicyMode.RBAC_AND_ABAC,
    defaultAllow: false,
    enableAudit: true,
    strictMode: true,
  };

  private auditLog: PolicyEvaluationResult[] = [];

  /**
   * Configure policy engine
   */
  configure(config: Partial<PolicyEngineConfig>): void {
    this.config = {
      ...this.config,
      ...config,
    };
  }

  /**
   * Evaluate access request
   */
  async evaluate(context: PolicyEvaluationContext): Promise<PolicyEvaluationResult> {
    const result: PolicyEvaluationResult = {
      allowed: false,
      mode: this.config.mode,
      reason: '',
      metadata: {
        evaluatedAt: new Date().toISOString(),
        context,
      },
    };

    switch (this.config.mode) {
      case PolicyMode.RBAC_ONLY:
        result.rbacResult = await this.evaluateRBAC(context);
        result.allowed = result.rbacResult.allowed;
        result.reason = this.generateRBACReason(result.rbacResult);
        break;

      case PolicyMode.ABAC_ONLY:
        result.abacResult = await this.evaluateABAC(context);
        result.allowed = result.abacResult.allowed;
        result.reason = this.generateABACReason(result.abacResult);
        break;

      case PolicyMode.RBAC_AND_ABAC:
        result.rbacResult = await this.evaluateRBAC(context);
        result.abacResult = await this.evaluateABAC(context);
        result.allowed = result.rbacResult.allowed && result.abacResult.allowed;
        result.reason = this.generateCombinedReason(result, 'AND');
        break;

      case PolicyMode.RBAC_OR_ABAC:
        result.rbacResult = await this.evaluateRBAC(context);
        result.abacResult = await this.evaluateABAC(context);
        result.allowed = result.rbacResult.allowed || result.abacResult.allowed;
        result.reason = this.generateCombinedReason(result, 'OR');
        break;
    }

    // Apply default policy if nothing matched
    if (!result.allowed && this.config.defaultAllow) {
      result.allowed = true;
      result.reason += ' (default allow policy applied)';
    }

    // Audit logging
    if (this.config.enableAudit) {
      this.auditLog.push(result);
      this.cleanupAuditLog();
    }

    return result;
  }

  /**
   * Check if action is allowed
   */
  async isAllowed(context: PolicyEvaluationContext): Promise<boolean> {
    const result = await this.evaluate(context);
    return result.allowed;
  }

  /**
   * Check if user can perform action on resource
   */
  async can(
    userId: string,
    action: string,
    resource: string,
    attributes?: {
      resource?: Record<string, any>;
      user?: Record<string, any>;
      environment?: Record<string, any>;
    }
  ): Promise<boolean> {
    return this.isAllowed({
      userId,
      action,
      resource,
      resourceAttributes: attributes?.resource,
      userAttributes: attributes?.user,
      environmentAttributes: attributes?.environment,
    });
  }

  /**
   * Check multiple permissions
   */
  async canAll(
    userId: string,
    permissions: Permission[],
    resource?: string
  ): Promise<boolean> {
    return this.isAllowed({
      userId,
      action: 'check_permissions',
      resource: resource || 'system',
      requiredPermissions: permissions,
    });
  }

  /**
   * Check if user has any of the permissions
   */
  async canAny(
    userId: string,
    permissions: Permission[],
    resource?: string
  ): Promise<boolean> {
    // Check each permission individually
    for (const permission of permissions) {
      const allowed = await this.isAllowed({
        userId,
        action: 'check_permission',
        resource: resource || 'system',
        requiredPermissions: [permission],
      });

      if (allowed) {
        return true;
      }
    }

    return false;
  }

  /**
   * Get audit log
   */
  getAuditLog(limit?: number): PolicyEvaluationResult[] {
    if (limit) {
      return this.auditLog.slice(-limit);
    }
    return [...this.auditLog];
  }

  /**
   * Clear audit log
   */
  clearAuditLog(): void {
    this.auditLog = [];
  }

  /**
   * Get policy statistics
   */
  getStatistics(): {
    totalEvaluations: number;
    allowed: number;
    denied: number;
    allowRate: number;
  } {
    const total = this.auditLog.length;
    const allowed = this.auditLog.filter((r) => r.allowed).length;
    const denied = total - allowed;

    return {
      totalEvaluations: total,
      allowed,
      denied,
      allowRate: total > 0 ? allowed / total : 0,
    };
  }

  // ============================================================================
  // Private Helper Methods
  // ============================================================================

  private async evaluateRBAC(context: PolicyEvaluationContext): Promise<{
    allowed: boolean;
    permissions: Permission[];
    roles: string[];
  }> {
    const permissions = rbacEngine.getUserPermissions(context.userId);
    const roles = rbacEngine.getUserRoles(context.userId);

    let allowed = false;

    // Check required permissions
    if (context.requiredPermissions && context.requiredPermissions.length > 0) {
      allowed = rbacEngine.hasAllPermissions(context.userId, context.requiredPermissions);
    } else {
      // Check if user has any permissions (basic access)
      allowed = permissions.length > 0;
    }

    return {
      allowed,
      permissions,
      roles: roles.map((r) => r.id),
    };
  }

  private async evaluateABAC(context: PolicyEvaluationContext): Promise<{
    allowed: boolean;
    effect: 'ALLOW' | 'DENY' | 'NOT_APPLICABLE';
    matchedPolicies: string[];
  }> {
    const abacContext: AttributeContext = {
      subject: {
        userId: context.userId,
        ...context.userAttributes,
      },
      resource: {
        type: context.resource,
        ...context.resourceAttributes,
      },
      action: context.action,
      environment: {
        timestamp: Date.now(),
        ...context.environmentAttributes,
      },
    };

    const decision = await abacEngine.evaluate(abacContext);

    return {
      allowed: decision.allowed,
      effect: decision.effect,
      matchedPolicies: decision.matchedPolicies,
    };
  }

  private generateRBACReason(result: {
    allowed: boolean;
    permissions: Permission[];
    roles: string[];
  }): string {
    if (result.allowed) {
      return `RBAC: Access granted with ${result.permissions.length} permission(s) from ${result.roles.length} role(s)`;
    }
    return `RBAC: Access denied - insufficient permissions`;
  }

  private generateABACReason(result: {
    allowed: boolean;
    effect: 'ALLOW' | 'DENY' | 'NOT_APPLICABLE';
    matchedPolicies: string[];
  }): string {
    if (result.allowed) {
      return `ABAC: Access allowed by ${result.matchedPolicies.length} policy(ies)`;
    }
    if (result.effect === 'DENY') {
      return `ABAC: Access explicitly denied by policy`;
    }
    return `ABAC: No applicable policies found`;
  }

  private generateCombinedReason(
    result: PolicyEvaluationResult,
    mode: 'AND' | 'OR'
  ): string {
    const parts: string[] = [];

    if (result.rbacResult) {
      parts.push(this.generateRBACReason(result.rbacResult));
    }

    if (result.abacResult) {
      parts.push(this.generateABACReason(result.abacResult));
    }

    const connector = mode === 'AND' ? ' AND ' : ' OR ';
    return parts.join(connector);
  }

  private cleanupAuditLog(): void {
    // Keep only last 10000 entries
    const maxEntries = 10000;
    if (this.auditLog.length > maxEntries) {
      this.auditLog = this.auditLog.slice(-maxEntries);
    }
  }

  /**
   * Export configuration
   */
  getConfig(): PolicyEngineConfig {
    return { ...this.config };
  }

  /**
   * Test policy evaluation
   */
  async test(context: PolicyEvaluationContext): Promise<{
    result: PolicyEvaluationResult;
    details: {
      rbac?: any;
      abac?: any;
    };
  }> {
    const result = await this.evaluate(context);

    return {
      result,
      details: {
        rbac: result.rbacResult,
        abac: result.abacResult,
      },
    };
  }
}

// Export singleton instance
export const policyEngine = new PolicyEngine();
