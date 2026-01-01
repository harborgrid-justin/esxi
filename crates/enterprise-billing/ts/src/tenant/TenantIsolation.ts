/**
 * Tenant Isolation - Ensures data isolation and security between tenants
 */

import { Tenant } from '../types';

export interface IsolationContext {
  tenantId: string;
  userId?: string;
  permissions: string[];
  metadata?: Record<string, any>;
}

export interface DataAccessPolicy {
  allowCrossTenantAccess: boolean;
  sharedResources: string[];
  isolationLevel: 'strict' | 'relaxed' | 'shared';
}

export class TenantIsolation {
  private currentContext: IsolationContext | null = null;
  private accessPolicies: Map<string, DataAccessPolicy> = new Map();

  /**
   * Set current tenant context
   */
  setContext(context: IsolationContext): void {
    this.currentContext = context;
  }

  /**
   * Get current tenant context
   */
  getContext(): IsolationContext {
    if (!this.currentContext) {
      throw new Error('No tenant context set. Call setContext first.');
    }
    return this.currentContext;
  }

  /**
   * Clear tenant context
   */
  clearContext(): void {
    this.currentContext = null;
  }

  /**
   * Check if current context can access resource
   */
  canAccessResource(resourceTenantId: string, resourceType: string): boolean {
    const context = this.getContext();

    // Same tenant - always allowed
    if (context.tenantId === resourceTenantId) {
      return true;
    }

    // Check if cross-tenant access is allowed
    const policy = this.accessPolicies.get(context.tenantId);
    if (!policy) {
      return false; // No policy means strict isolation
    }

    if (!policy.allowCrossTenantAccess) {
      return false;
    }

    // Check if resource type is in shared resources
    return policy.sharedResources.includes(resourceType);
  }

  /**
   * Validate tenant access
   */
  validateAccess(resourceTenantId: string, resourceType: string): void {
    if (!this.canAccessResource(resourceTenantId, resourceType)) {
      throw new Error(
        `Access denied: Tenant ${this.currentContext?.tenantId} cannot access ${resourceType} from tenant ${resourceTenantId}`
      );
    }
  }

  /**
   * Set access policy for a tenant
   */
  setAccessPolicy(tenantId: string, policy: DataAccessPolicy): void {
    this.accessPolicies.set(tenantId, policy);
  }

  /**
   * Get access policy for a tenant
   */
  getAccessPolicy(tenantId: string): DataAccessPolicy | undefined {
    return this.accessPolicies.get(tenantId);
  }

  /**
   * Filter data by tenant
   */
  filterByTenant<T extends { tenantId: string }>(
    data: T[],
    allowCrossTenant: boolean = false
  ): T[] {
    const context = this.getContext();

    return data.filter((item) => {
      if (item.tenantId === context.tenantId) {
        return true;
      }

      if (allowCrossTenant) {
        const policy = this.accessPolicies.get(context.tenantId);
        return policy?.allowCrossTenantAccess || false;
      }

      return false;
    });
  }

  /**
   * Apply tenant filter to query
   */
  applyTenantFilter(query: any): any {
    const context = this.getContext();

    return {
      ...query,
      tenantId: context.tenantId,
    };
  }

  /**
   * Create scoped database connection string
   */
  getScopedConnectionString(baseConnectionString: string): string {
    const context = this.getContext();

    // For shared database with row-level security
    return `${baseConnectionString}?tenant_id=${context.tenantId}`;
  }

  /**
   * Get tenant-specific schema name
   */
  getTenantSchema(tenant: Tenant): string {
    // For schema-based isolation
    return `tenant_${tenant.slug}`;
  }

  /**
   * Get tenant-specific database name
   */
  getTenantDatabase(tenant: Tenant): string {
    // For database-level isolation
    return `db_${tenant.slug}`;
  }

  /**
   * Validate data ownership
   */
  validateOwnership<T extends { tenantId: string }>(resource: T): void {
    const context = this.getContext();

    if (resource.tenantId !== context.tenantId) {
      throw new Error(
        `Ownership validation failed: Resource belongs to tenant ${resource.tenantId}, but context is ${context.tenantId}`
      );
    }
  }

  /**
   * Check if user has permission
   */
  hasPermission(permission: string): boolean {
    const context = this.getContext();
    return context.permissions.includes(permission) ||
           context.permissions.includes('*');
  }

  /**
   * Require permission
   */
  requirePermission(permission: string): void {
    if (!this.hasPermission(permission)) {
      throw new Error(`Permission denied: ${permission} required`);
    }
  }

  /**
   * Create isolation middleware for API requests
   */
  createMiddleware() {
    return (req: any, res: any, next: any) => {
      const tenantId = req.headers['x-tenant-id'] || req.query.tenantId;
      const userId = req.user?.id;

      if (!tenantId) {
        return res.status(400).json({ error: 'Tenant ID required' });
      }

      this.setContext({
        tenantId,
        userId,
        permissions: req.user?.permissions || [],
        metadata: {
          requestId: req.id,
          ip: req.ip,
        },
      });

      // Clear context after request
      res.on('finish', () => {
        this.clearContext();
      });

      next();
    };
  }

  /**
   * Encrypt tenant-specific data
   */
  encryptTenantData(data: string, tenantId: string): string {
    // In production, use proper encryption with tenant-specific keys
    const key = this.getTenantEncryptionKey(tenantId);
    // Placeholder - implement actual encryption
    return Buffer.from(`${key}:${data}`).toString('base64');
  }

  /**
   * Decrypt tenant-specific data
   */
  decryptTenantData(encryptedData: string, tenantId: string): string {
    // In production, use proper decryption with tenant-specific keys
    const key = this.getTenantEncryptionKey(tenantId);
    const decoded = Buffer.from(encryptedData, 'base64').toString();
    // Placeholder - implement actual decryption
    return decoded.replace(`${key}:`, '');
  }

  /**
   * Get tenant-specific encryption key
   */
  private getTenantEncryptionKey(tenantId: string): string {
    // In production, retrieve from secure key management system
    return `tenant_key_${tenantId}`;
  }

  /**
   * Audit tenant access
   */
  async auditAccess(
    action: string,
    resourceType: string,
    resourceId: string,
    result: 'allowed' | 'denied'
  ): Promise<void> {
    const context = this.getContext();

    const auditLog = {
      timestamp: new Date(),
      tenantId: context.tenantId,
      userId: context.userId,
      action,
      resourceType,
      resourceId,
      result,
      metadata: context.metadata,
    };

    // In production, save to audit log storage
    console.log('Audit:', auditLog);
  }

  /**
   * Check rate limits per tenant
   */
  checkRateLimit(limit: number, windowMs: number): boolean {
    const context = this.getContext();
    // In production, use Redis or similar for distributed rate limiting
    // This is a placeholder implementation
    return true;
  }
}
