/**
 * Organization Service
 * API service for organization management operations
 */

import axios, { AxiosInstance } from 'axios';
import {
  Organization,
  OrganizationSettings,
  OrganizationBranding,
  ApiResponse,
  PaginationParams,
  FilterParams,
} from '../types';

export class OrganizationService {
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

    // Add response interceptor
    this.client.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.status === 401) {
          window.dispatchEvent(new CustomEvent('auth:unauthorized'));
        }
        return Promise.reject(error);
      }
    );
  }

  /**
   * Get organization by ID
   */
  async getOrganization(organizationId: string): Promise<ApiResponse<Organization>> {
    const response = await this.client.get<ApiResponse<Organization>>(
      `/api/organizations/${organizationId}`
    );
    return response.data;
  }

  /**
   * List organizations
   */
  async listOrganizations(
    params?: PaginationParams & FilterParams
  ): Promise<ApiResponse<Organization[]>> {
    const response = await this.client.get<ApiResponse<Organization[]>>(
      '/api/organizations',
      { params }
    );
    return response.data;
  }

  /**
   * Create organization
   */
  async createOrganization(
    organization: Partial<Organization>
  ): Promise<ApiResponse<Organization>> {
    const response = await this.client.post<ApiResponse<Organization>>(
      '/api/organizations',
      organization
    );
    return response.data;
  }

  /**
   * Update organization
   */
  async updateOrganization(
    organizationId: string,
    updates: Partial<Organization>
  ): Promise<ApiResponse<Organization>> {
    const response = await this.client.patch<ApiResponse<Organization>>(
      `/api/organizations/${organizationId}`,
      updates
    );
    return response.data;
  }

  /**
   * Delete organization
   */
  async deleteOrganization(organizationId: string): Promise<ApiResponse<void>> {
    const response = await this.client.delete<ApiResponse<void>>(
      `/api/organizations/${organizationId}`
    );
    return response.data;
  }

  /**
   * Update organization settings
   */
  async updateSettings(
    organizationId: string,
    settings: Partial<OrganizationSettings>
  ): Promise<ApiResponse<Organization>> {
    const response = await this.client.patch<ApiResponse<Organization>>(
      `/api/organizations/${organizationId}/settings`,
      settings
    );
    return response.data;
  }

  /**
   * Update organization branding
   */
  async updateBranding(
    organizationId: string,
    branding: Partial<OrganizationBranding>
  ): Promise<ApiResponse<Organization>> {
    const response = await this.client.patch<ApiResponse<Organization>>(
      `/api/organizations/${organizationId}/branding`,
      branding
    );
    return response.data;
  }

  /**
   * Upload organization logo
   */
  async uploadLogo(
    organizationId: string,
    file: File
  ): Promise<ApiResponse<{ url: string }>> {
    const formData = new FormData();
    formData.append('logo', file);

    const response = await this.client.post<ApiResponse<{ url: string }>>(
      `/api/organizations/${organizationId}/logo`,
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
   * Verify custom domain
   */
  async verifyCustomDomain(
    organizationId: string,
    domain: string
  ): Promise<ApiResponse<{ verified: boolean; records: Record<string, string> }>> {
    const response = await this.client.post<
      ApiResponse<{ verified: boolean; records: Record<string, string> }>
    >(`/api/organizations/${organizationId}/domain/verify`, { domain });
    return response.data;
  }

  /**
   * Set custom domain
   */
  async setCustomDomain(
    organizationId: string,
    domain: string
  ): Promise<ApiResponse<Organization>> {
    const response = await this.client.post<ApiResponse<Organization>>(
      `/api/organizations/${organizationId}/domain`,
      { domain }
    );
    return response.data;
  }

  /**
   * Remove custom domain
   */
  async removeCustomDomain(organizationId: string): Promise<ApiResponse<Organization>> {
    const response = await this.client.delete<ApiResponse<Organization>>(
      `/api/organizations/${organizationId}/domain`
    );
    return response.data;
  }

  /**
   * Activate organization
   */
  async activateOrganization(
    organizationId: string
  ): Promise<ApiResponse<Organization>> {
    const response = await this.client.post<ApiResponse<Organization>>(
      `/api/organizations/${organizationId}/activate`
    );
    return response.data;
  }

  /**
   * Deactivate organization
   */
  async deactivateOrganization(
    organizationId: string
  ): Promise<ApiResponse<Organization>> {
    const response = await this.client.post<ApiResponse<Organization>>(
      `/api/organizations/${organizationId}/deactivate`
    );
    return response.data;
  }

  /**
   * Get organization statistics
   */
  async getStatistics(
    organizationId: string
  ): Promise<
    ApiResponse<{
      totalUsers: number;
      activeUsers: number;
      totalRoles: number;
      storageUsed: number;
    }>
  > {
    const response = await this.client.get<
      ApiResponse<{
        totalUsers: number;
        activeUsers: number;
        totalRoles: number;
        storageUsed: number;
      }>
    >(`/api/organizations/${organizationId}/statistics`);
    return response.data;
  }
}

export default OrganizationService;
