/**
 * Tenant Manager - Manages tenant lifecycle and operations
 */

import { v4 as uuidv4 } from 'uuid';
import {
  Tenant,
  TenantStatus,
  Subscription,
  Plan,
  PaginationParams,
  PaginatedResult,
} from '../types';

export interface CreateTenantParams {
  organizationName: string;
  slug: string;
  billingEmail: string;
  currency?: string;
  locale?: string;
  metadata?: Record<string, any>;
}

export interface UpdateTenantParams {
  organizationName?: string;
  billingEmail?: string;
  billingAddress?: Tenant['billingAddress'];
  taxId?: string;
  metadata?: Record<string, any>;
}

export class TenantManager {
  private tenants: Map<string, Tenant> = new Map();
  private slugToId: Map<string, string> = new Map();

  /**
   * Create a new tenant
   */
  async createTenant(params: CreateTenantParams): Promise<Tenant> {
    // Check if slug is already taken
    if (this.slugToId.has(params.slug)) {
      throw new Error(`Slug '${params.slug}' is already taken`);
    }

    const tenant: Tenant = {
      id: uuidv4(),
      organizationName: params.organizationName,
      slug: params.slug,
      status: TenantStatus.TRIAL,
      billingEmail: params.billingEmail,
      currency: params.currency || 'USD',
      locale: params.locale || 'en-US',
      metadata: params.metadata || {},
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    this.tenants.set(tenant.id, tenant);
    this.slugToId.set(tenant.slug, tenant.id);

    return tenant;
  }

  /**
   * Get tenant by ID
   */
  getTenant(tenantId: string): Tenant | undefined {
    return this.tenants.get(tenantId);
  }

  /**
   * Get tenant by slug
   */
  getTenantBySlug(slug: string): Tenant | undefined {
    const tenantId = this.slugToId.get(slug);
    return tenantId ? this.tenants.get(tenantId) : undefined;
  }

  /**
   * Update tenant
   */
  async updateTenant(
    tenantId: string,
    updates: UpdateTenantParams
  ): Promise<Tenant> {
    const tenant = this.tenants.get(tenantId);
    if (!tenant) {
      throw new Error(`Tenant ${tenantId} not found`);
    }

    const updated: Tenant = {
      ...tenant,
      ...updates,
      updatedAt: new Date(),
    };

    this.tenants.set(tenantId, updated);
    return updated;
  }

  /**
   * Delete tenant (soft delete)
   */
  async deleteTenant(tenantId: string): Promise<void> {
    const tenant = this.tenants.get(tenantId);
    if (!tenant) {
      throw new Error(`Tenant ${tenantId} not found`);
    }

    tenant.deletedAt = new Date();
    tenant.status = TenantStatus.CHURNED;
    tenant.updatedAt = new Date();

    this.tenants.set(tenantId, tenant);
  }

  /**
   * Hard delete tenant (GDPR compliance)
   */
  async hardDeleteTenant(tenantId: string): Promise<void> {
    const tenant = this.tenants.get(tenantId);
    if (!tenant) {
      throw new Error(`Tenant ${tenantId} not found`);
    }

    this.slugToId.delete(tenant.slug);
    this.tenants.delete(tenantId);
  }

  /**
   * Suspend tenant
   */
  async suspendTenant(tenantId: string, reason?: string): Promise<Tenant> {
    const tenant = this.tenants.get(tenantId);
    if (!tenant) {
      throw new Error(`Tenant ${tenantId} not found`);
    }

    tenant.status = TenantStatus.SUSPENDED;
    tenant.metadata = {
      ...tenant.metadata,
      suspensionReason: reason,
      suspendedAt: new Date(),
    };
    tenant.updatedAt = new Date();

    this.tenants.set(tenantId, tenant);
    return tenant;
  }

  /**
   * Reactivate suspended tenant
   */
  async reactivateTenant(tenantId: string): Promise<Tenant> {
    const tenant = this.tenants.get(tenantId);
    if (!tenant) {
      throw new Error(`Tenant ${tenantId} not found`);
    }

    if (tenant.status !== TenantStatus.SUSPENDED) {
      throw new Error('Can only reactivate suspended tenants');
    }

    tenant.status = TenantStatus.ACTIVE;
    tenant.metadata = {
      ...tenant.metadata,
      reactivatedAt: new Date(),
    };
    tenant.updatedAt = new Date();

    this.tenants.set(tenantId, tenant);
    return tenant;
  }

  /**
   * Convert trial to active
   */
  async convertTrialToActive(tenantId: string): Promise<Tenant> {
    const tenant = this.tenants.get(tenantId);
    if (!tenant) {
      throw new Error(`Tenant ${tenantId} not found`);
    }

    if (tenant.status !== TenantStatus.TRIAL) {
      throw new Error('Tenant is not in trial status');
    }

    tenant.status = TenantStatus.ACTIVE;
    tenant.metadata = {
      ...tenant.metadata,
      trialConvertedAt: new Date(),
    };
    tenant.updatedAt = new Date();

    this.tenants.set(tenantId, tenant);
    return tenant;
  }

  /**
   * Link subscription to tenant
   */
  async linkSubscription(tenantId: string, subscriptionId: string): Promise<Tenant> {
    const tenant = this.tenants.get(tenantId);
    if (!tenant) {
      throw new Error(`Tenant ${tenantId} not found`);
    }

    tenant.subscriptionId = subscriptionId;
    tenant.updatedAt = new Date();

    this.tenants.set(tenantId, tenant);
    return tenant;
  }

  /**
   * List all tenants with pagination
   */
  async listTenants(params: PaginationParams): Promise<PaginatedResult<Tenant>> {
    let tenants = Array.from(this.tenants.values()).filter((t) => !t.deletedAt);

    // Sort
    if (params.sortBy) {
      tenants.sort((a, b) => {
        const aVal = (a as any)[params.sortBy!];
        const bVal = (b as any)[params.sortBy!];

        if (aVal < bVal) return params.sortOrder === 'asc' ? -1 : 1;
        if (aVal > bVal) return params.sortOrder === 'asc' ? 1 : -1;
        return 0;
      });
    }

    const total = tenants.length;
    const totalPages = Math.ceil(total / params.limit);

    // Paginate
    const start = (params.page - 1) * params.limit;
    const end = start + params.limit;
    const data = tenants.slice(start, end);

    return {
      data,
      total,
      page: params.page,
      limit: params.limit,
      totalPages,
    };
  }

  /**
   * Get tenants by status
   */
  getTenantsByStatus(status: TenantStatus): Tenant[] {
    return Array.from(this.tenants.values()).filter(
      (t) => t.status === status && !t.deletedAt
    );
  }

  /**
   * Get active tenant count
   */
  getActiveTenantCount(): number {
    return this.getTenantsByStatus(TenantStatus.ACTIVE).length;
  }

  /**
   * Get trial tenant count
   */
  getTrialTenantCount(): number {
    return this.getTenantsByStatus(TenantStatus.TRIAL).length;
  }

  /**
   * Search tenants
   */
  searchTenants(query: string): Tenant[] {
    const lowerQuery = query.toLowerCase();

    return Array.from(this.tenants.values()).filter(
      (t) =>
        !t.deletedAt &&
        (t.organizationName.toLowerCase().includes(lowerQuery) ||
          t.slug.toLowerCase().includes(lowerQuery) ||
          t.billingEmail.toLowerCase().includes(lowerQuery))
    );
  }

  /**
   * Validate slug format
   */
  validateSlug(slug: string): { valid: boolean; error?: string } {
    if (slug.length < 3) {
      return { valid: false, error: 'Slug must be at least 3 characters' };
    }

    if (slug.length > 63) {
      return { valid: false, error: 'Slug must be at most 63 characters' };
    }

    if (!/^[a-z0-9-]+$/.test(slug)) {
      return {
        valid: false,
        error: 'Slug can only contain lowercase letters, numbers, and hyphens',
      };
    }

    if (slug.startsWith('-') || slug.endsWith('-')) {
      return { valid: false, error: 'Slug cannot start or end with a hyphen' };
    }

    if (this.slugToId.has(slug)) {
      return { valid: false, error: 'Slug is already taken' };
    }

    return { valid: true };
  }

  /**
   * Generate unique slug from organization name
   */
  async generateSlug(organizationName: string): Promise<string> {
    let baseSlug = organizationName
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '');

    if (baseSlug.length < 3) {
      baseSlug = `tenant-${baseSlug}`;
    }

    let slug = baseSlug;
    let counter = 1;

    while (this.slugToId.has(slug)) {
      slug = `${baseSlug}-${counter}`;
      counter++;
    }

    return slug;
  }

  /**
   * Get tenant statistics
   */
  getStatistics(): {
    total: number;
    active: number;
    trial: number;
    suspended: number;
    churned: number;
  } {
    const tenants = Array.from(this.tenants.values()).filter((t) => !t.deletedAt);

    return {
      total: tenants.length,
      active: tenants.filter((t) => t.status === TenantStatus.ACTIVE).length,
      trial: tenants.filter((t) => t.status === TenantStatus.TRIAL).length,
      suspended: tenants.filter((t) => t.status === TenantStatus.SUSPENDED).length,
      churned: tenants.filter((t) => t.status === TenantStatus.CHURNED).length,
    };
  }
}
