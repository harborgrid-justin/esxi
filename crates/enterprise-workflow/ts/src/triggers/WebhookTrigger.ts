/**
 * Webhook Trigger - HTTP webhook-based workflow triggering
 */

import { EventEmitter } from 'eventemitter3';
import crypto from 'crypto';
import { Trigger, WebhookTriggerConfig, Context } from '../types';

export interface WebhookRequest {
  method: string;
  headers: Record<string, string>;
  body: any;
  query: Record<string, string>;
  ip: string;
  timestamp: Date;
}

export class WebhookTrigger extends EventEmitter {
  private triggers: Map<string, Trigger>;

  constructor() {
    super();
    this.triggers = new Map();
  }

  /**
   * Register a webhook trigger
   */
  register(trigger: Trigger): void {
    const config = trigger.config as WebhookTriggerConfig;

    if (!config.url) {
      throw new Error('Webhook trigger must have a URL');
    }

    this.triggers.set(trigger.id, trigger);
    this.emit('webhook:registered', { triggerId: trigger.id, url: config.url });
  }

  /**
   * Unregister a webhook trigger
   */
  unregister(triggerId: string): void {
    this.triggers.delete(triggerId);
    this.emit('webhook:unregistered', { triggerId });
  }

  /**
   * Handle incoming webhook request
   */
  async handleRequest(
    url: string,
    request: WebhookRequest
  ): Promise<{ triggered: boolean; triggerId?: string; context?: Partial<Context> }> {
    // Find matching trigger
    const trigger = Array.from(this.triggers.values()).find(t => {
      const config = t.config as WebhookTriggerConfig;
      return config.url === url && config.method === request.method;
    });

    if (!trigger || !trigger.enabled) {
      return { triggered: false };
    }

    const config = trigger.config as WebhookTriggerConfig;

    // Validate secret if configured
    if (config.secret && !this.validateSecret(config.secret, request)) {
      this.emit('webhook:unauthorized', {
        triggerId: trigger.id,
        url,
        ip: request.ip
      });
      throw new Error('Invalid webhook secret');
    }

    // Validate headers if configured
    if (config.headers && !this.validateHeaders(config.headers, request.headers)) {
      this.emit('webhook:invalid_headers', {
        triggerId: trigger.id,
        url
      });
      throw new Error('Invalid webhook headers');
    }

    // Validate payload if custom validation is configured
    if (config.validation && !config.validation(request.body)) {
      this.emit('webhook:invalid_payload', {
        triggerId: trigger.id,
        url
      });
      throw new Error('Invalid webhook payload');
    }

    // Create execution context from webhook data
    const context: Partial<Context> = {
      variables: new Map([
        ['webhook_payload', request.body],
        ['webhook_headers', request.headers],
        ['webhook_query', request.query],
        ['webhook_ip', request.ip],
        ['webhook_timestamp', request.timestamp]
      ]),
      metadata: {
        source: 'webhook',
        url,
        method: request.method
      }
    };

    this.emit('webhook:triggered', {
      triggerId: trigger.id,
      url,
      context
    });

    return {
      triggered: true,
      triggerId: trigger.id,
      context
    };
  }

  /**
   * Validate webhook secret
   */
  private validateSecret(secret: string, request: WebhookRequest): boolean {
    const signature = request.headers['x-webhook-signature'] ||
                     request.headers['x-hub-signature-256'];

    if (!signature) {
      return false;
    }

    // Calculate expected signature
    const payload = JSON.stringify(request.body);
    const expectedSignature = crypto
      .createHmac('sha256', secret)
      .update(payload)
      .digest('hex');

    // Compare signatures (constant-time comparison)
    return crypto.timingSafeEqual(
      Buffer.from(signature),
      Buffer.from(`sha256=${expectedSignature}`)
    );
  }

  /**
   * Validate required headers
   */
  private validateHeaders(
    required: Record<string, string>,
    actual: Record<string, string>
  ): boolean {
    return Object.entries(required).every(([key, value]) => {
      const actualValue = actual[key.toLowerCase()];
      return actualValue === value;
    });
  }

  /**
   * Generate webhook URL
   */
  generateUrl(baseUrl: string, triggerId: string): string {
    return `${baseUrl}/webhooks/${triggerId}`;
  }

  /**
   * Generate webhook secret
   */
  generateSecret(): string {
    return crypto.randomBytes(32).toString('hex');
  }

  /**
   * Get all registered webhooks
   */
  getRegistered(): Trigger[] {
    return Array.from(this.triggers.values());
  }

  /**
   * Get webhook by ID
   */
  getById(triggerId: string): Trigger | undefined {
    return this.triggers.get(triggerId);
  }

  /**
   * Get webhook by URL
   */
  getByUrl(url: string): Trigger | undefined {
    return Array.from(this.triggers.values()).find(t => {
      const config = t.config as WebhookTriggerConfig;
      return config.url === url;
    });
  }

  /**
   * Validate webhook configuration
   */
  validate(config: WebhookTriggerConfig): string[] {
    const errors: string[] = [];

    if (!config.url) {
      errors.push('Webhook URL is required');
    }

    if (!config.method) {
      errors.push('HTTP method is required');
    } else if (!['GET', 'POST', 'PUT', 'PATCH', 'DELETE'].includes(config.method)) {
      errors.push('Invalid HTTP method');
    }

    return errors;
  }
}
