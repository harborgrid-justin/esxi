/**
 * Tenant Service
 * API service for tenant management operations
 */

import axios, { AxiosInstance } from 'axios';
import {
  Tenant,
  TenantSettings,
  TenantBranding,
  ApiResponse,
  PaginationParams,
  FilterParams,
  UsageMetrics,
} from '../types';

export class TenantService {
  private client: AxiosInstance;

  constructor(baseURL: string, getAuthToken: () => string | null) {
    this.client = axios.create({
      baseURL,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Add auth interceptor
    this.client.interceptors.request.use((config) => {
      const token = getAuthToken();
      if (token) {
        config.headers.Authorization = `Bearer ${token}`;
      }
      return config;
    });

    // Add response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          // Handle unauthorized
          window.dispatchEvent(new CustomEvent('auth:unauthorized'));
        }
        return Promise.reject(error);
      }
    );
  }

  /**
   * Get current tenant
   */
  async getCurrentTenant(): Promise<ApiResponse<Tenant>> {
    const response = await this.client.get<ApiResponse<Tenant>>('/api/tenants/current');
    return response.data;
  }

  /**
   * Get tenant by ID
   */
  async getTenant(tenantId: string): Promise<ApiResponse<Tenant>> {
    const response = await this.client.get<ApiResponse<Tenant>>(`/api/tenants/${tenantId}`);
    return response.data;
  }

  /**
   * List all tenants (super admin only)
   */
  async listTenants(
    params?: PaginationParams & FilterParams
  ): Promise<ApiResponse<Tenant[]>> {
    const response = await this.client.get<ApiResponse<Tenant[]>>('/api/tenants', {
      params,
    });
    return response.data;
  }

  /**
   * Create new tenant
   */
  async createTenant(tenant: Partial<Tenant>): Promise<ApiResponse<Tenant>> {
    const response = await this.client.post<ApiResponse<Tenant>>('/api/tenants', tenant);
    return response.data;
  }

  /**
   * Update tenant
   */
  async updateTenant(
    tenantId: string,
    updates: Partial<Tenant>
  ): Promise<ApiResponse<Tenant>> {
    const response = await this.client.patch<ApiResponse<Tenant>>(
      `/api/tenants/${tenantId}`,
      updates
    );
    return response.data;
  }

  /**
   * Delete tenant
   */
  async deleteTenant(tenantId: string): Promise<ApiResponse<void>> {
    const response = await this.client.delete<ApiResponse<void>>(
      `/api/tenants/${tenantId}`
    );
    return response.data;
  }

  /**
   * Update tenant settings
   */
  async updateSettings(
    tenantId: string,
    settings: Partial<TenantSettings>
  ): Promise<ApiResponse<Tenant>> {
    const response = await this.client.patch<ApiResponse<Tenant>>(
      `/api/tenants/${tenantId}/settings`,
      settings
    );
    return response.data;
  }

  /**
   * Update tenant branding
   */
  async updateBranding(
    tenantId: string,
    branding: Partial<TenantBranding>
  ): Promise<ApiResponse<Tenant>> {
    const response = await this.client.patch<ApiResponse<Tenant>>(
      `/api/tenants/${tenantId}/branding`,
      branding
    );
    return response.data;
  }

  /**
   * Upload tenant logo
   */
  async uploadLogo(tenantId: string, file: File): Promise<ApiResponse<{ url: string }>> {
    const formData = new FormData();
    formData.append('logo', file);

    const response = await this.client.post<ApiResponse<{ url: string }>(
      `/api/tenants/${tenantId}/logo`,
      formData,
      {
        headers: {
          'Content-Type': 'multipart/form-data',
        },
      }
    );
    return response.data;
  }

  /**
   * Get tenant usage metrics
   */
  async getUsageMetrics(tenantId: string): Promise<ApiResponse<UsageMetrics>> {
    const response = await this.client.get<ApiResponse<UsageMetrics>>(
      `/api/tenants/${tenantId}/usage`
    );
    return response.data;
  }

  /**
   * Verify custom domain
   */
  async verifyCustomDomain(
    tenantId: string,
    domain: string
  ): Promise<ApiResponse<{ verified: boolean; records: Record<string, string> }>> {
    const response = await this.client.post<
      ApiResponse<{ verified: boolean; records: Record<string, string> }>
    >(`/api/tenants/${tenantId}/domain/verify`, { domain });
    return response.data;
  }

  /**
   * Set custom domain
   */
  async setCustomDomain(
    tenantId: string,
    domain: string
  ): Promise<ApiResponse<Tenant>> {
    const response = await this.client.post<ApiResponse<Tenant>>(
      `/api/tenants/${tenantId}/domain`,
      { domain }
    );
    return response.data;
  }

  /**
   * Remove custom domain
   */
  async removeCustomDomain(tenantId: string): Promise<ApiResponse<Tenant>> {
    const response = await this.client.delete<ApiResponse<Tenant>>(
      `/api/tenants/${tenantId}/domain`
    );
    return response.data;
  }

  /**
   * Suspend tenant
   */
  async suspendTenant(tenantId: string, reason?: string): Promise<ApiResponse<Tenant>> {
    const response = await this.client.post<ApiResponse<Tenant>>(
      `/api/tenants/${tenantId}/suspend`,
      { reason }
    );
    return response.data;
  }

  /**
   * Activate tenant
   */
  async activateTenant(tenantId: string): Promise<ApiResponse<Tenant>> {
    const response = await this.client.post<ApiResponse<Tenant>>(
      `/api/tenants/${tenantId}/activate`
    );
    return response.data;
  }
}

export default TenantService;
