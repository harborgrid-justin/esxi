/**
 * DeliveryEngine - Multi-channel delivery orchestration
 * Manages delivery attempts, retries, and tracking
 */

import { EventEmitter } from 'events';
import {
  Notification,
  NotificationChannelType,
  NotificationStatus,
  DeliveryAttempt,
  DeliveryReceipt,
} from '../types';

export interface DeliveryEngineConfig {
  maxRetries: number;
  retryDelay: number;
  retryBackoff: number;
  timeout: number;
  trackDelivery: boolean;
  trackReads: boolean;
  trackClicks: boolean;
}

export interface ChannelDeliveryHandler {
  type: NotificationChannelType;
  deliver(
    notification: Notification,
    recipient: { id: string; identifier: string }
  ): Promise<DeliveryAttempt>;
  cancel?(deliveryId: string): Promise<boolean>;
  getStatus?(deliveryId: string): Promise<DeliveryAttempt>;
}

export class DeliveryEngine extends EventEmitter {
  private config: DeliveryEngineConfig;
  private handlers: Map<NotificationChannelType, ChannelDeliveryHandler>;
  private deliveryAttempts: Map<string, DeliveryAttempt>;
  private retryQueue: Map<string, NodeJS.Timeout>;

  constructor(config: Partial<DeliveryEngineConfig> = {}) {
    super();
    this.config = {
      maxRetries: config.maxRetries ?? 3,
      retryDelay: config.retryDelay ?? 5000,
      retryBackoff: config.retryBackoff ?? 2,
      timeout: config.timeout ?? 30000,
      trackDelivery: config.trackDelivery ?? true,
      trackReads: config.trackReads ?? true,
      trackClicks: config.trackClicks ?? true,
    };

    this.handlers = new Map();
    this.deliveryAttempts = new Map();
    this.retryQueue = new Map();
  }

  /**
   * Register a channel delivery handler
   */
  registerHandler(handler: ChannelDeliveryHandler): void {
    this.handlers.set(handler.type, handler);
    this.emit('handler:registered', handler.type);
  }

  /**
   * Deliver notification through all channels
   */
  async deliver(notification: Notification): Promise<DeliveryAttempt[]> {
    const attempts: DeliveryAttempt[] = [];

    for (const channel of notification.channels) {
      const handler = this.handlers.get(channel);
      if (!handler) {
        this.emit('error', new Error(`No handler for channel: ${channel}`));
        continue;
      }

      for (const recipient of notification.recipients) {
        try {
          const attempt = await this.deliverToRecipient(notification, channel, recipient, handler);
          attempts.push(attempt);
        } catch (error) {
          this.emit('delivery:failed', notification, channel, recipient, error);
        }
      }
    }

    return attempts;
  }

  /**
   * Deliver to a single recipient
   */
  private async deliverToRecipient(
    notification: Notification,
    channel: NotificationChannelType,
    recipient: { id: string; identifier: string },
    handler: ChannelDeliveryHandler
  ): Promise<DeliveryAttempt> {
    const attemptId = this.generateAttemptId();
    const attempt: DeliveryAttempt = {
      id: attemptId,
      notificationId: notification.id,
      channel,
      recipientId: recipient.id,
      status: 'pending',
      attemptNumber: 1,
      scheduledFor: new Date(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    this.deliveryAttempts.set(attemptId, attempt);
    this.emit('delivery:started', attempt);

    try {
      // Set timeout
      const deliveryPromise = handler.deliver(notification, recipient);
      const timeoutPromise = new Promise<never>((_, reject) =>
        setTimeout(() => reject(new Error('Delivery timeout')), this.config.timeout)
      );

      const result = await Promise.race([deliveryPromise, timeoutPromise]);

      attempt.status = 'sent';
      attempt.sentAt = new Date();
      attempt.response = result.response;
      attempt.externalId = result.externalId;

      this.emit('delivery:sent', attempt);
    } catch (error) {
      attempt.status = 'failed';
      attempt.failedAt = new Date();
      attempt.error = error instanceof Error ? error.message : String(error);

      // Schedule retry if applicable
      if (attempt.attemptNumber < this.config.maxRetries) {
        this.scheduleRetry(notification, channel, recipient, handler, attempt.attemptNumber);
      } else {
        this.emit('delivery:failed:final', attempt);
      }

      throw error;
    } finally {
      attempt.updatedAt = new Date();
      this.deliveryAttempts.set(attemptId, attempt);
    }

    return attempt;
  }

  /**
   * Schedule retry for failed delivery
   */
  private scheduleRetry(
    notification: Notification,
    channel: NotificationChannelType,
    recipient: { id: string; identifier: string },
    handler: ChannelDeliveryHandler,
    attemptNumber: number
  ): void {
    const delay = this.config.retryDelay * Math.pow(this.config.retryBackoff, attemptNumber);
    const retryKey = `${notification.id}:${channel}:${recipient.id}`;

    this.emit('delivery:retry:scheduled', notification, channel, recipient, attemptNumber, delay);

    const timeout = setTimeout(() => {
      this.deliverToRecipient(notification, channel, recipient, handler)
        .then(attempt => {
          attempt.attemptNumber = attemptNumber + 1;
          this.emit('delivery:retry:success', attempt);
        })
        .catch(error => {
          this.emit('delivery:retry:failed', notification, channel, recipient, error);
        });
      this.retryQueue.delete(retryKey);
    }, delay);

    this.retryQueue.set(retryKey, timeout);
  }

  /**
   * Cancel pending delivery
   */
  async cancelDelivery(attemptId: string): Promise<boolean> {
    const attempt = this.deliveryAttempts.get(attemptId);
    if (!attempt) {
      return false;
    }

    if (attempt.status !== 'pending') {
      return false;
    }

    const handler = this.handlers.get(attempt.channel);
    if (handler?.cancel) {
      try {
        const cancelled = await handler.cancel(attempt.externalId ?? attemptId);
        if (cancelled) {
          attempt.status = 'failed';
          attempt.error = 'Cancelled';
          attempt.updatedAt = new Date();
          this.emit('delivery:cancelled', attempt);
          return true;
        }
      } catch (error) {
        this.emit('error', error);
      }
    }

    return false;
  }

  /**
   * Get delivery attempt status
   */
  async getDeliveryStatus(attemptId: string): Promise<DeliveryAttempt | undefined> {
    const attempt = this.deliveryAttempts.get(attemptId);
    if (!attempt) {
      return undefined;
    }

    const handler = this.handlers.get(attempt.channel);
    if (handler?.getStatus && attempt.externalId) {
      try {
        const updated = await handler.getStatus(attempt.externalId);
        this.deliveryAttempts.set(attemptId, updated);
        return updated;
      } catch (error) {
        this.emit('error', error);
      }
    }

    return attempt;
  }

  /**
   * Record delivery receipt
   */
  recordReceipt(receipt: Partial<DeliveryReceipt>): DeliveryReceipt {
    const fullReceipt: DeliveryReceipt = {
      id: receipt.id ?? this.generateReceiptId(),
      notificationId: receipt.notificationId ?? '',
      deliveryAttemptId: receipt.deliveryAttemptId ?? '',
      channel: receipt.channel ?? NotificationChannelType.EMAIL,
      event: receipt.event ?? 'delivered',
      timestamp: receipt.timestamp ?? new Date(),
      details: receipt.details,
      ipAddress: receipt.ipAddress,
      userAgent: receipt.userAgent,
      location: receipt.location,
      createdAt: new Date(),
    };

    this.emit('receipt:recorded', fullReceipt);

    // Update delivery attempt status
    if (fullReceipt.deliveryAttemptId) {
      const attempt = this.deliveryAttempts.get(fullReceipt.deliveryAttemptId);
      if (attempt) {
        if (fullReceipt.event === 'delivered') {
          attempt.status = 'delivered';
          attempt.deliveredAt = fullReceipt.timestamp;
        } else if (fullReceipt.event === 'bounced') {
          attempt.status = 'bounced';
        } else if (fullReceipt.event === 'failed') {
          attempt.status = 'failed';
        }
        attempt.updatedAt = new Date();
        this.deliveryAttempts.set(fullReceipt.deliveryAttemptId, attempt);
      }
    }

    return fullReceipt;
  }

  /**
   * Get delivery statistics
   */
  getStats(): {
    total: number;
    pending: number;
    sent: number;
    delivered: number;
    failed: number;
    bounced: number;
  } {
    const stats = {
      total: 0,
      pending: 0,
      sent: 0,
      delivered: 0,
      failed: 0,
      bounced: 0,
    };

    for (const attempt of this.deliveryAttempts.values()) {
      stats.total++;
      switch (attempt.status) {
        case 'pending':
          stats.pending++;
          break;
        case 'sent':
          stats.sent++;
          break;
        case 'delivered':
          stats.delivered++;
          break;
        case 'failed':
          stats.failed++;
          break;
        case 'bounced':
          stats.bounced++;
          break;
      }
    }

    return stats;
  }

  /**
   * Clear old delivery attempts
   */
  clearOldAttempts(olderThan: Date): number {
    let cleared = 0;
    for (const [id, attempt] of this.deliveryAttempts.entries()) {
      if (attempt.createdAt < olderThan) {
        this.deliveryAttempts.delete(id);
        cleared++;
      }
    }
    return cleared;
  }

  /**
   * Generate unique attempt ID
   */
  private generateAttemptId(): string {
    return `attempt_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Generate unique receipt ID
   */
  private generateReceiptId(): string {
    return `receipt_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Cleanup on destroy
   */
  destroy(): void {
    // Clear all retry timers
    for (const timeout of this.retryQueue.values()) {
      clearTimeout(timeout);
    }
    this.retryQueue.clear();
    this.deliveryAttempts.clear();
    this.handlers.clear();
  }
}

export default DeliveryEngine;
