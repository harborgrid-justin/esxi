/**
 * Webhook Service - Handle and dispatch webhook events
 */

import { WebhookEvent, WebhookPayload } from '../types';
import { v4 as uuidv4 } from 'uuid';
import crypto from 'crypto';

export interface WebhookEndpoint {
  id: string;
  url: string;
  events: WebhookEvent[];
  secret: string;
  enabled: boolean;
  metadata?: Record<string, any>;
}

export interface WebhookDelivery {
  id: string;
  endpointId: string;
  event: WebhookEvent;
  payload: any;
  status: 'pending' | 'delivered' | 'failed';
  attempts: number;
  lastAttemptAt?: Date;
  nextRetryAt?: Date;
  response?: {
    statusCode: number;
    body: string;
  };
  createdAt: Date;
}

export class WebhookService {
  private endpoints: Map<string, WebhookEndpoint> = new Map();
  private deliveries: Map<string, WebhookDelivery> = new Map();
  private maxRetries: number = 3;

  /**
   * Register webhook endpoint
   */
  async registerEndpoint(
    url: string,
    events: WebhookEvent[],
    metadata?: Record<string, any>
  ): Promise<WebhookEndpoint> {
    const endpoint: WebhookEndpoint = {
      id: uuidv4(),
      url,
      events,
      secret: this.generateSecret(),
      enabled: true,
      metadata,
    };

    this.endpoints.set(endpoint.id, endpoint);
    return endpoint;
  }

  /**
   * Update webhook endpoint
   */
  async updateEndpoint(
    endpointId: string,
    updates: Partial<Omit<WebhookEndpoint, 'id' | 'secret'>>
  ): Promise<WebhookEndpoint> {
    const endpoint = this.endpoints.get(endpointId);

    if (!endpoint) {
      throw new Error(`Webhook endpoint ${endpointId} not found`);
    }

    const updated = {
      ...endpoint,
      ...updates,
    };

    this.endpoints.set(endpointId, updated);
    return updated;
  }

  /**
   * Delete webhook endpoint
   */
  async deleteEndpoint(endpointId: string): Promise<void> {
    this.endpoints.delete(endpointId);
  }

  /**
   * Dispatch webhook event
   */
  async dispatchEvent(event: WebhookEvent, data: any): Promise<void> {
    const payload: WebhookPayload = {
      event,
      data,
      timestamp: new Date(),
    };

    // Find all endpoints subscribed to this event
    const subscribedEndpoints = Array.from(this.endpoints.values()).filter(
      (ep) => ep.enabled && ep.events.includes(event)
    );

    // Deliver to each endpoint
    for (const endpoint of subscribedEndpoints) {
      await this.deliverWebhook(endpoint, payload);
    }
  }

  /**
   * Deliver webhook to endpoint
   */
  private async deliverWebhook(
    endpoint: WebhookEndpoint,
    payload: WebhookPayload
  ): Promise<void> {
    const delivery: WebhookDelivery = {
      id: uuidv4(),
      endpointId: endpoint.id,
      event: payload.event,
      payload,
      status: 'pending',
      attempts: 0,
      createdAt: new Date(),
    };

    this.deliveries.set(delivery.id, delivery);

    // Attempt delivery
    await this.attemptDelivery(endpoint, delivery);
  }

  /**
   * Attempt webhook delivery
   */
  private async attemptDelivery(
    endpoint: WebhookEndpoint,
    delivery: WebhookDelivery
  ): Promise<void> {
    delivery.attempts++;
    delivery.lastAttemptAt = new Date();

    try {
      const signature = this.generateSignature(delivery.payload, endpoint.secret);

      // In production, use actual HTTP client
      const response = await this.sendWebhook(endpoint.url, delivery.payload, signature);

      if (response.statusCode >= 200 && response.statusCode < 300) {
        delivery.status = 'delivered';
        delivery.response = response;
      } else {
        throw new Error(`HTTP ${response.statusCode}: ${response.body}`);
      }
    } catch (error: any) {
      delivery.status = 'failed';
      delivery.response = {
        statusCode: 0,
        body: error.message,
      };

      // Schedule retry if not exceeded max attempts
      if (delivery.attempts < this.maxRetries) {
        delivery.nextRetryAt = this.calculateNextRetry(delivery.attempts);
      }
    }

    this.deliveries.set(delivery.id, delivery);
  }

  /**
   * Retry failed deliveries
   */
  async retryFailedDeliveries(): Promise<void> {
    const now = new Date();

    for (const [id, delivery] of this.deliveries) {
      if (
        delivery.status === 'failed' &&
        delivery.nextRetryAt &&
        delivery.nextRetryAt <= now &&
        delivery.attempts < this.maxRetries
      ) {
        const endpoint = this.endpoints.get(delivery.endpointId);
        if (endpoint) {
          await this.attemptDelivery(endpoint, delivery);
        }
      }
    }
  }

  /**
   * Generate webhook signature
   */
  generateSignature(payload: any, secret: string): string {
    const payloadString = JSON.stringify(payload);
    const hmac = crypto.createHmac('sha256', secret);
    hmac.update(payloadString);
    return hmac.digest('hex');
  }

  /**
   * Verify webhook signature
   */
  verifySignature(payload: any, signature: string, secret: string): boolean {
    const expectedSignature = this.generateSignature(payload, secret);
    return crypto.timingSafeEqual(
      Buffer.from(signature),
      Buffer.from(expectedSignature)
    );
  }

  /**
   * Get webhook deliveries for an endpoint
   */
  getEndpointDeliveries(endpointId: string): WebhookDelivery[] {
    return Array.from(this.deliveries.values())
      .filter((d) => d.endpointId === endpointId)
      .sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());
  }

  /**
   * Get delivery statistics
   */
  getDeliveryStatistics(endpointId?: string): {
    total: number;
    delivered: number;
    failed: number;
    pending: number;
    successRate: number;
  } {
    let deliveries = Array.from(this.deliveries.values());

    if (endpointId) {
      deliveries = deliveries.filter((d) => d.endpointId === endpointId);
    }

    const total = deliveries.length;
    const delivered = deliveries.filter((d) => d.status === 'delivered').length;
    const failed = deliveries.filter((d) => d.status === 'failed').length;
    const pending = deliveries.filter((d) => d.status === 'pending').length;
    const successRate = total > 0 ? (delivered / total) * 100 : 0;

    return {
      total,
      delivered,
      failed,
      pending,
      successRate,
    };
  }

  /**
   * Generate webhook secret
   */
  private generateSecret(): string {
    return crypto.randomBytes(32).toString('hex');
  }

  /**
   * Calculate next retry time with exponential backoff
   */
  private calculateNextRetry(attempts: number): Date {
    const baseDelay = 60000; // 1 minute
    const delay = baseDelay * Math.pow(2, attempts - 1);
    return new Date(Date.now() + delay);
  }

  /**
   * Mock HTTP request (replace with actual implementation)
   */
  private async sendWebhook(
    url: string,
    payload: any,
    signature: string
  ): Promise<{ statusCode: number; body: string }> {
    // In production, use fetch or axios
    console.log(`Sending webhook to ${url}`, { payload, signature });

    // Simulate successful delivery
    return {
      statusCode: 200,
      body: 'OK',
    };
  }

  /**
   * List all endpoints
   */
  listEndpoints(): WebhookEndpoint[] {
    return Array.from(this.endpoints.values());
  }

  /**
   * Test endpoint
   */
  async testEndpoint(endpointId: string): Promise<boolean> {
    const endpoint = this.endpoints.get(endpointId);

    if (!endpoint) {
      throw new Error(`Webhook endpoint ${endpointId} not found`);
    }

    const testPayload: WebhookPayload = {
      event: WebhookEvent.SUBSCRIPTION_CREATED,
      data: { test: true },
      timestamp: new Date(),
    };

    try {
      const signature = this.generateSignature(testPayload, endpoint.secret);
      const response = await this.sendWebhook(endpoint.url, testPayload, signature);
      return response.statusCode >= 200 && response.statusCode < 300;
    } catch (error) {
      return false;
    }
  }
}
