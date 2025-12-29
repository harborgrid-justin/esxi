/**
 * Billing Service
 * API service for subscription and billing operations
 */

import axios, { AxiosInstance } from 'axios';
import {
  Subscription,
  SubscriptionTier,
  Invoice,
  PaymentMethod,
  UsageMetrics,
  ApiResponse,
  PaginationParams,
} from '../types';

export class BillingService {
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
   * Get current subscription
   */
  async getCurrentSubscription(): Promise<ApiResponse<Subscription>> {
    const response = await this.client.get<ApiResponse<Subscription>>(
      '/api/billing/subscription'
    );
    return response.data;
  }

  /**
   * Get subscription by tenant ID
   */
  async getSubscription(tenantId: string): Promise<ApiResponse<Subscription>> {
    const response = await this.client.get<ApiResponse<Subscription>>(
      `/api/billing/tenants/${tenantId}/subscription`
    );
    return response.data;
  }

  /**
   * Update subscription tier
   */
  async updateSubscriptionTier(
    tenantId: string,
    tier: SubscriptionTier,
    billingCycle: 'monthly' | 'yearly'
  ): Promise<ApiResponse<Subscription>> {
    const response = await this.client.post<ApiResponse<Subscription>>(
      `/api/billing/tenants/${tenantId}/subscription/tier`,
      { tier, billingCycle }
    );
    return response.data;
  }

  /**
   * Cancel subscription
   */
  async cancelSubscription(
    tenantId: string,
    reason?: string,
    cancelImmediately?: boolean
  ): Promise<ApiResponse<Subscription>> {
    const response = await this.client.post<ApiResponse<Subscription>>(
      `/api/billing/tenants/${tenantId}/subscription/cancel`,
      { reason, cancelImmediately }
    );
    return response.data;
  }

  /**
   * Reactivate subscription
   */
  async reactivateSubscription(tenantId: string): Promise<ApiResponse<Subscription>> {
    const response = await this.client.post<ApiResponse<Subscription>>(
      `/api/billing/tenants/${tenantId}/subscription/reactivate`
    );
    return response.data;
  }

  /**
   * Get usage metrics
   */
  async getUsageMetrics(tenantId: string): Promise<ApiResponse<UsageMetrics>> {
    const response = await this.client.get<ApiResponse<UsageMetrics>>(
      `/api/billing/tenants/${tenantId}/usage`
    );
    return response.data;
  }

  /**
   * Get historical usage
   */
  async getHistoricalUsage(
    tenantId: string,
    startDate: Date,
    endDate: Date
  ): Promise<ApiResponse<UsageMetrics[]>> {
    const response = await this.client.get<ApiResponse<UsageMetrics[]>>(
      `/api/billing/tenants/${tenantId}/usage/history`,
      {
        params: {
          startDate: startDate.toISOString(),
          endDate: endDate.toISOString(),
        },
      }
    );
    return response.data;
  }

  /**
   * List invoices
   */
  async listInvoices(
    tenantId: string,
    params?: PaginationParams
  ): Promise<ApiResponse<Invoice[]>> {
    const response = await this.client.get<ApiResponse<Invoice[]>>(
      `/api/billing/tenants/${tenantId}/invoices`,
      { params }
    );
    return response.data;
  }

  /**
   * Get invoice by ID
   */
  async getInvoice(invoiceId: string): Promise<ApiResponse<Invoice>> {
    const response = await this.client.get<ApiResponse<Invoice>>(
      `/api/billing/invoices/${invoiceId}`
    );
    return response.data;
  }

  /**
   * Download invoice PDF
   */
  async downloadInvoice(invoiceId: string): Promise<Blob> {
    const response = await this.client.get<Blob>(
      `/api/billing/invoices/${invoiceId}/pdf`,
      {
        responseType: 'blob',
      }
    );
    return response.data;
  }

  /**
   * Pay invoice
   */
  async payInvoice(invoiceId: string): Promise<ApiResponse<Invoice>> {
    const response = await this.client.post<ApiResponse<Invoice>>(
      `/api/billing/invoices/${invoiceId}/pay`
    );
    return response.data;
  }

  /**
   * Get payment methods
   */
  async getPaymentMethods(tenantId: string): Promise<ApiResponse<PaymentMethod[]>> {
    const response = await this.client.get<ApiResponse<PaymentMethod[]>>(
      `/api/billing/tenants/${tenantId}/payment-methods`
    );
    return response.data;
  }

  /**
   * Add payment method
   */
  async addPaymentMethod(
    tenantId: string,
    paymentMethod: Partial<PaymentMethod>
  ): Promise<ApiResponse<PaymentMethod>> {
    const response = await this.client.post<ApiResponse<PaymentMethod>>(
      `/api/billing/tenants/${tenantId}/payment-methods`,
      paymentMethod
    );
    return response.data;
  }

  /**
   * Set default payment method
   */
  async setDefaultPaymentMethod(
    tenantId: string,
    paymentMethodId: string
  ): Promise<ApiResponse<void>> {
    const response = await this.client.post<ApiResponse<void>>(
      `/api/billing/tenants/${tenantId}/payment-methods/${paymentMethodId}/default`
    );
    return response.data;
  }

  /**
   * Remove payment method
   */
  async removePaymentMethod(
    tenantId: string,
    paymentMethodId: string
  ): Promise<ApiResponse<void>> {
    const response = await this.client.delete<ApiResponse<void>>(
      `/api/billing/tenants/${tenantId}/payment-methods/${paymentMethodId}`
    );
    return response.data;
  }

  /**
   * Get billing portal URL (for Stripe, etc.)
   */
  async getBillingPortalUrl(tenantId: string): Promise<ApiResponse<{ url: string }>> {
    const response = await this.client.post<ApiResponse<{ url: string }>>(
      `/api/billing/tenants/${tenantId}/portal`
    );
    return response.data;
  }

  /**
   * Get pricing plans
   */
  async getPricingPlans(): Promise<
    ApiResponse<
      Array<{
        tier: SubscriptionTier;
        name: string;
        description: string;
        features: string[];
        pricing: {
          monthly: number;
          yearly: number;
          currency: string;
        };
        limits: {
          users: number;
          organizations: number;
          storage: number;
        };
      }>
    >
  > {
    const response = await this.client.get<
      ApiResponse<
        Array<{
          tier: SubscriptionTier;
          name: string;
          description: string;
          features: string[];
          pricing: {
            monthly: number;
            yearly: number;
            currency: string;
          };
          limits: {
            users: number;
            organizations: number;
            storage: number;
          };
        }>
      >
    >('/api/billing/plans');
    return response.data;
  }

  /**
   * Start trial
   */
  async startTrial(tenantId: string, days: number): Promise<ApiResponse<Subscription>> {
    const response = await this.client.post<ApiResponse<Subscription>>(
      `/api/billing/tenants/${tenantId}/trial`,
      { days }
    );
    return response.data;
  }

  /**
   * Apply coupon
   */
  async applyCoupon(
    tenantId: string,
    couponCode: string
  ): Promise<ApiResponse<Subscription>> {
    const response = await this.client.post<ApiResponse<Subscription>>(
      `/api/billing/tenants/${tenantId}/coupon`,
      { couponCode }
    );
    return response.data;
  }
}

export default BillingService;
