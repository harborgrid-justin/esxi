/**
 * User Service
 * API service for user management operations
 */

import axios, { AxiosInstance } from 'axios';
import {
  User,
  UserRole,
  InviteUser,
  UserInvitation,
  ApiResponse,
  PaginationParams,
  FilterParams,
} from '../types';

export class UserService {
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
   * Get current user
   */
  async getCurrentUser(): Promise<ApiResponse<User>> {
    const response = await this.client.get<ApiResponse<User>>('/api/users/me');
    return response.data;
  }

  /**
   * Get user by ID
   */
  async getUser(userId: string): Promise<ApiResponse<User>> {
    const response = await this.client.get<ApiResponse<User>>(`/api/users/${userId}`);
    return response.data;
  }

  /**
   * List users
   */
  async listUsers(
    params?: PaginationParams & FilterParams
  ): Promise<ApiResponse<User[]>> {
    const response = await this.client.get<ApiResponse<User[]>>('/api/users', {
      params,
    });
    return response.data;
  }

  /**
   * Create user
   */
  async createUser(user: Partial<User>): Promise<ApiResponse<User>> {
    const response = await this.client.post<ApiResponse<User>>('/api/users', user);
    return response.data;
  }

  /**
   * Update user
   */
  async updateUser(userId: string, updates: Partial<User>): Promise<ApiResponse<User>> {
    const response = await this.client.patch<ApiResponse<User>>(
      `/api/users/${userId}`,
      updates
    );
    return response.data;
  }

  /**
   * Delete user
   */
  async deleteUser(userId: string): Promise<ApiResponse<void>> {
    const response = await this.client.delete<ApiResponse<void>>(`/api/users/${userId}`);
    return response.data;
  }

  /**
   * Update user profile
   */
  async updateProfile(userId: string, profile: Partial<User>): Promise<ApiResponse<User>> {
    const response = await this.client.patch<ApiResponse<User>>(
      `/api/users/${userId}/profile`,
      profile
    );
    return response.data;
  }

  /**
   * Upload user avatar
   */
  async uploadAvatar(userId: string, file: File): Promise<ApiResponse<{ url: string }>> {
    const formData = new FormData();
    formData.append('avatar', file);

    const response = await this.client.post<ApiResponse<{ url: string }>>(
      `/api/users/${userId}/avatar`,
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
   * Update user role
   */
  async updateRole(userId: string, role: UserRole): Promise<ApiResponse<User>> {
    const response = await this.client.patch<ApiResponse<User>>(
      `/api/users/${userId}/role`,
      { role }
    );
    return response.data;
  }

  /**
   * Assign custom roles
   */
  async assignRoles(userId: string, roleIds: string[]): Promise<ApiResponse<User>> {
    const response = await this.client.post<ApiResponse<User>>(
      `/api/users/${userId}/roles`,
      { roleIds }
    );
    return response.data;
  }

  /**
   * Remove custom roles
   */
  async removeRoles(userId: string, roleIds: string[]): Promise<ApiResponse<User>> {
    const response = await this.client.delete<ApiResponse<User>>(
      `/api/users/${userId}/roles`,
      { data: { roleIds } }
    );
    return response.data;
  }

  /**
   * Invite user
   */
  async inviteUser(invitation: InviteUser): Promise<ApiResponse<UserInvitation>> {
    const response = await this.client.post<ApiResponse<UserInvitation>>(
      '/api/users/invite',
      invitation
    );
    return response.data;
  }

  /**
   * Resend invitation
   */
  async resendInvitation(invitationId: string): Promise<ApiResponse<UserInvitation>> {
    const response = await this.client.post<ApiResponse<UserInvitation>>(
      `/api/users/invitations/${invitationId}/resend`
    );
    return response.data;
  }

  /**
   * Revoke invitation
   */
  async revokeInvitation(invitationId: string): Promise<ApiResponse<void>> {
    const response = await this.client.delete<ApiResponse<void>>(
      `/api/users/invitations/${invitationId}`
    );
    return response.data;
  }

  /**
   * List invitations
   */
  async listInvitations(
    params?: PaginationParams & FilterParams
  ): Promise<ApiResponse<UserInvitation[]>> {
    const response = await this.client.get<ApiResponse<UserInvitation[]>>(
      '/api/users/invitations',
      { params }
    );
    return response.data;
  }

  /**
   * Accept invitation
   */
  async acceptInvitation(
    token: string,
    userData: { password: string; firstName: string; lastName: string }
  ): Promise<ApiResponse<User>> {
    const response = await this.client.post<ApiResponse<User>>(
      '/api/users/invitations/accept',
      { token, ...userData }
    );
    return response.data;
  }

  /**
   * Suspend user
   */
  async suspendUser(userId: string, reason?: string): Promise<ApiResponse<User>> {
    const response = await this.client.post<ApiResponse<User>>(
      `/api/users/${userId}/suspend`,
      { reason }
    );
    return response.data;
  }

  /**
   * Activate user
   */
  async activateUser(userId: string): Promise<ApiResponse<User>> {
    const response = await this.client.post<ApiResponse<User>>(
      `/api/users/${userId}/activate`
    );
    return response.data;
  }

  /**
   * Change password
   */
  async changePassword(
    userId: string,
    oldPassword: string,
    newPassword: string
  ): Promise<ApiResponse<void>> {
    const response = await this.client.post<ApiResponse<void>>(
      `/api/users/${userId}/password`,
      { oldPassword, newPassword }
    );
    return response.data;
  }

  /**
   * Reset password
   */
  async resetPassword(email: string): Promise<ApiResponse<void>> {
    const response = await this.client.post<ApiResponse<void>>('/api/users/password/reset', {
      email,
    });
    return response.data;
  }

  /**
   * Enable SSO for user
   */
  async enableSSO(userId: string): Promise<ApiResponse<User>> {
    const response = await this.client.post<ApiResponse<User>>(
      `/api/users/${userId}/sso/enable`
    );
    return response.data;
  }

  /**
   * Disable SSO for user
   */
  async disableSSO(userId: string): Promise<ApiResponse<User>> {
    const response = await this.client.post<ApiResponse<User>>(
      `/api/users/${userId}/sso/disable`
    );
    return response.data;
  }
}

export default UserService;
