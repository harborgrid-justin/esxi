/**
 * WebhookChannel - Generic webhook notification delivery
 * Supports custom HTTP webhooks with flexible authentication
 */

import { EventEmitter } from 'events';
import {
  Notification,
  NotificationChannelType,
  DeliveryAttempt,
  WebhookChannelConfig,
} from '../types';

export interface WebhookPayload {
  notification: Notification;
  timestamp: string;
  signature?: string;
}

export interface WebhookResponse {
  status: number;
  statusText: string;
  body?: unknown;
  headers?: Record<string, string>;
}

export class WebhookChannel extends EventEmitter {
  private config: WebhookChannelConfig;
  private isInitialized: boolean;

  constructor(config: WebhookChannelConfig) {
    super();
    this.config = config;
    this.isInitialized = false;
  }

  /**
   * Initialize webhook channel
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    // Validate configuration
    if (!this.config.url) {
      throw new Error('Webhook URL is required');
    }

    try {
      new URL(this.config.url);
    } catch (error) {
      throw new Error('Invalid webhook URL');
    }

    this.isInitialized = true;
    this.emit('initialized');
  }

  /**
   * Send webhook notification
   */
  async send(notification: Notification): Promise<DeliveryAttempt> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const attempt: DeliveryAttempt = {
      id: this.generateAttemptId(),
      notificationId: notification.id,
      channel: NotificationChannelType.WEBHOOK,
      recipientId: notification.recipients[0]?.id ?? '',
      status: 'pending',
      attemptNumber: 1,
      scheduledFor: new Date(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    try {
      const payload = this.buildPayload(notification);
      const headers = this.buildHeaders(payload);

      const response = await this.sendRequest(payload, headers);

      attempt.status = 'sent';
      attempt.sentAt = new Date();
      attempt.response = {
        status: response.status,
        statusText: response.statusText,
        body: response.body,
      };

      this.emit('sent', attempt);
    } catch (error) {
      attempt.status = 'failed';
      attempt.failedAt = new Date();
      attempt.error = error instanceof Error ? error.message : String(error);

      // Check if should retry
      if (this.shouldRetry(error, attempt.attemptNumber)) {
        this.scheduleRetry(notification, attempt.attemptNumber);
      }

      this.emit('failed', attempt, error);
      throw error;
    } finally {
      attempt.updatedAt = new Date();
    }

    return attempt;
  }

  /**
   * Send HTTP request
   */
  private async sendRequest(
    payload: WebhookPayload,
    headers: Record<string, string>
  ): Promise<WebhookResponse> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 30000); // 30 second timeout

    try {
      const response = await fetch(this.config.url, {
        method: this.config.method,
        headers: {
          'Content-Type': 'application/json',
          'User-Agent': 'HarborGrid-Notifications/1.0',
          ...headers,
        },
        body: JSON.stringify(payload),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      let body: unknown;
      const contentType = response.headers.get('content-type');

      if (contentType?.includes('application/json')) {
        body = await response.json();
      } else {
        body = await response.text();
      }

      if (!response.ok) {
        throw new Error(`Webhook request failed: ${response.status} ${response.statusText}`);
      }

      return {
        status: response.status,
        statusText: response.statusText,
        body,
        headers: Object.fromEntries(response.headers.entries()),
      };
    } catch (error) {
      clearTimeout(timeoutId);
      throw error;
    }
  }

  /**
   * Build webhook payload
   */
  private buildPayload(notification: Notification): WebhookPayload {
    return {
      notification: {
        ...notification,
        // Remove sensitive data if needed
      },
      timestamp: new Date().toISOString(),
      signature: this.generateSignature(notification),
    };
  }

  /**
   * Build request headers
   */
  private buildHeaders(payload: WebhookPayload): Record<string, string> {
    const headers: Record<string, string> = {
      ...this.config.headers,
    };

    // Add authentication headers
    if (this.config.auth) {
      switch (this.config.auth.type) {
        case 'basic':
          if (this.config.auth.username && this.config.auth.password) {
            const credentials = Buffer.from(
              `${this.config.auth.username}:${this.config.auth.password}`
            ).toString('base64');
            headers['Authorization'] = `Basic ${credentials}`;
          }
          break;

        case 'bearer':
          if (this.config.auth.token) {
            headers['Authorization'] = `Bearer ${this.config.auth.token}`;
          }
          break;

        case 'api-key':
          if (this.config.auth.apiKey && this.config.auth.apiKeyHeader) {
            headers[this.config.auth.apiKeyHeader] = this.config.auth.apiKey;
          }
          break;
      }
    }

    // Add signature header
    if (payload.signature) {
      headers['X-Webhook-Signature'] = payload.signature;
    }

    return headers;
  }

  /**
   * Generate webhook signature (HMAC-SHA256)
   */
  private generateSignature(notification: Notification): string {
    // Simple signature generation - in production, use proper HMAC
    const payload = JSON.stringify(notification);
    return Buffer.from(payload).toString('base64').substring(0, 32);
  }

  /**
   * Check if request should be retried
   */
  private shouldRetry(error: unknown, attemptNumber: number): boolean {
    const maxRetries = this.config.retryConfig?.maxRetries ?? 3;

    if (attemptNumber >= maxRetries) {
      return false;
    }

    // Retry on network errors or 5xx status codes
    if (error instanceof Error) {
      if (error.message.includes('timeout') || error.message.includes('ECONNREFUSED')) {
        return true;
      }
      if (error.message.includes('5')) {
        // 5xx error
        return true;
      }
    }

    return false;
  }

  /**
   * Schedule retry
   */
  private scheduleRetry(notification: Notification, attemptNumber: number): void {
    const baseDelay = this.config.retryConfig?.retryDelay ?? 1000;
    const backoff = this.config.retryConfig?.backoffMultiplier ?? 2;
    const delay = baseDelay * Math.pow(backoff, attemptNumber);

    setTimeout(() => {
      this.send(notification).catch(error => {
        this.emit('retry:failed', notification, error);
      });
    }, delay);

    this.emit('retry:scheduled', notification, attemptNumber, delay);
  }

  /**
   * Verify webhook endpoint
   */
  async verify(): Promise<boolean> {
    try {
      const testPayload = {
        notification: {
          id: 'test',
          type: 'verification',
          title: 'Webhook Verification',
          message: 'This is a test notification',
        },
        timestamp: new Date().toISOString(),
      };

      const headers = this.buildHeaders(testPayload as WebhookPayload);
      const response = await this.sendRequest(testPayload as WebhookPayload, headers);

      return response.status >= 200 && response.status < 300;
    } catch (error) {
      this.emit('verification:failed', error);
      return false;
    }
  }

  /**
   * Check if channel is healthy
   */
  async isHealthy(): Promise<boolean> {
    if (!this.isInitialized) {
      return false;
    }

    try {
      return await this.verify();
    } catch (error) {
      this.emit('health:unhealthy', error);
      return false;
    }
  }

  /**
   * Generate unique attempt ID
   */
  private generateAttemptId(): string {
    return `webhook_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Cleanup
   */
  async close(): Promise<void> {
    this.isInitialized = false;
    this.emit('closed');
  }
}

export default WebhookChannel;
