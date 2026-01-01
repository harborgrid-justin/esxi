/**
 * NotificationEngine - Core notification dispatcher and orchestrator
 * Manages notification lifecycle, routing, and delivery coordination
 */

import { EventEmitter } from 'events';
import {
  Notification,
  NotificationStatus,
  NotificationChannelType,
  NotificationPriority,
  DeliveryAttempt,
} from '../types';

export interface NotificationEngineConfig {
  maxConcurrent: number;
  retryAttempts: number;
  retryDelay: number;
  retryBackoff: number;
  defaultPriority: NotificationPriority;
  enableDeduplication: boolean;
  deduplicationWindow: number; // milliseconds
  enableBatching: boolean;
  batchSize: number;
  batchWindow: number; // milliseconds
}

export interface ChannelHandler {
  type: NotificationChannelType;
  send(notification: Notification): Promise<DeliveryAttempt>;
  supports(notification: Notification): boolean;
  isHealthy(): Promise<boolean>;
}

export class NotificationEngine extends EventEmitter {
  private config: NotificationEngineConfig;
  private channels: Map<NotificationChannelType, ChannelHandler>;
  private queue: Map<NotificationPriority, Notification[]>;
  private processing: Set<string>;
  private deduplicationCache: Map<string, Date>;
  private isRunning: boolean;
  private processInterval?: NodeJS.Timeout;

  constructor(config: Partial<NotificationEngineConfig> = {}) {
    super();
    this.config = {
      maxConcurrent: config.maxConcurrent ?? 10,
      retryAttempts: config.retryAttempts ?? 3,
      retryDelay: config.retryDelay ?? 1000,
      retryBackoff: config.retryBackoff ?? 2,
      defaultPriority: config.defaultPriority ?? NotificationPriority.NORMAL,
      enableDeduplication: config.enableDeduplication ?? true,
      deduplicationWindow: config.deduplicationWindow ?? 300000, // 5 minutes
      enableBatching: config.enableBatching ?? true,
      batchSize: config.batchSize ?? 100,
      batchWindow: config.batchWindow ?? 5000, // 5 seconds
    };

    this.channels = new Map();
    this.queue = new Map([
      [NotificationPriority.CRITICAL, []],
      [NotificationPriority.URGENT, []],
      [NotificationPriority.HIGH, []],
      [NotificationPriority.NORMAL, []],
      [NotificationPriority.LOW, []],
    ]);
    this.processing = new Set();
    this.deduplicationCache = new Map();
    this.isRunning = false;
  }

  /**
   * Register a channel handler
   */
  registerChannel(handler: ChannelHandler): void {
    this.channels.set(handler.type, handler);
    this.emit('channel:registered', handler.type);
  }

  /**
   * Unregister a channel handler
   */
  unregisterChannel(type: NotificationChannelType): void {
    this.channels.delete(type);
    this.emit('channel:unregistered', type);
  }

  /**
   * Start the notification engine
   */
  async start(): Promise<void> {
    if (this.isRunning) {
      throw new Error('NotificationEngine is already running');
    }

    this.isRunning = true;
    this.processInterval = setInterval(() => {
      this.processQueue().catch(error => {
        this.emit('error', error);
      });
    }, 100);

    // Clean deduplication cache periodically
    setInterval(() => {
      this.cleanDeduplicationCache();
    }, 60000); // Every minute

    this.emit('started');
  }

  /**
   * Stop the notification engine
   */
  async stop(): Promise<void> {
    if (!this.isRunning) {
      return;
    }

    this.isRunning = false;
    if (this.processInterval) {
      clearInterval(this.processInterval);
    }

    // Wait for in-flight notifications to complete
    while (this.processing.size > 0) {
      await new Promise(resolve => setTimeout(resolve, 100));
    }

    this.emit('stopped');
  }

  /**
   * Send a notification
   */
  async send(notification: Partial<Notification>): Promise<Notification> {
    const fullNotification = this.prepareNotification(notification);

    // Check for duplicates
    if (this.config.enableDeduplication && fullNotification.deduplicationKey) {
      if (this.isDuplicate(fullNotification)) {
        this.emit('notification:deduplicated', fullNotification);
        throw new Error('Duplicate notification within deduplication window');
      }
    }

    // Validate notification
    this.validateNotification(fullNotification);

    // Add to queue
    this.enqueue(fullNotification);

    this.emit('notification:queued', fullNotification);
    return fullNotification;
  }

  /**
   * Send batch of notifications
   */
  async sendBatch(notifications: Partial<Notification>[]): Promise<Notification[]> {
    const prepared = notifications.map(n => this.prepareNotification(n));
    const valid: Notification[] = [];

    for (const notification of prepared) {
      try {
        this.validateNotification(notification);
        valid.push(notification);
      } catch (error) {
        this.emit('notification:invalid', notification, error);
      }
    }

    // Add all valid notifications to queue
    for (const notification of valid) {
      this.enqueue(notification);
    }

    this.emit('batch:queued', valid.length);
    return valid;
  }

  /**
   * Cancel a notification
   */
  async cancel(notificationId: string): Promise<boolean> {
    // Remove from queues
    for (const [priority, queue] of this.queue.entries()) {
      const index = queue.findIndex(n => n.id === notificationId);
      if (index !== -1) {
        queue.splice(index, 1);
        this.emit('notification:cancelled', notificationId);
        return true;
      }
    }

    return false;
  }

  /**
   * Get queue stats
   */
  getStats(): {
    queued: Record<NotificationPriority, number>;
    processing: number;
    total: number;
  } {
    const queued = {
      [NotificationPriority.CRITICAL]: this.queue.get(NotificationPriority.CRITICAL)?.length ?? 0,
      [NotificationPriority.URGENT]: this.queue.get(NotificationPriority.URGENT)?.length ?? 0,
      [NotificationPriority.HIGH]: this.queue.get(NotificationPriority.HIGH)?.length ?? 0,
      [NotificationPriority.NORMAL]: this.queue.get(NotificationPriority.NORMAL)?.length ?? 0,
      [NotificationPriority.LOW]: this.queue.get(NotificationPriority.LOW)?.length ?? 0,
    };

    const total = Object.values(queued).reduce((sum, count) => sum + count, 0);

    return {
      queued,
      processing: this.processing.size,
      total: total + this.processing.size,
    };
  }

  /**
   * Prepare notification with defaults
   */
  private prepareNotification(partial: Partial<Notification>): Notification {
    const now = new Date();
    return {
      id: partial.id ?? this.generateId(),
      tenantId: partial.tenantId ?? '',
      userId: partial.userId,
      type: partial.type ?? 'default',
      category: partial.category,
      priority: partial.priority ?? this.config.defaultPriority,
      status: NotificationStatus.PENDING,
      title: partial.title ?? '',
      message: partial.message ?? '',
      html: partial.html,
      data: partial.data,
      metadata: partial.metadata,
      channels: partial.channels ?? [NotificationChannelType.EMAIL],
      recipients: partial.recipients ?? [],
      scheduledFor: partial.scheduledFor,
      expiresAt: partial.expiresAt,
      templateId: partial.templateId,
      templateData: partial.templateData,
      attempts: 0,
      maxAttempts: partial.maxAttempts ?? this.config.retryAttempts,
      actionUrl: partial.actionUrl,
      actionLabel: partial.actionLabel,
      links: partial.links,
      silent: partial.silent,
      badge: partial.badge,
      sound: partial.sound,
      icon: partial.icon,
      image: partial.image,
      deduplicationKey: partial.deduplicationKey,
      groupKey: partial.groupKey,
      notificationIds: [],
      createdAt: now,
      updatedAt: now,
    };
  }

  /**
   * Validate notification
   */
  private validateNotification(notification: Notification): void {
    if (!notification.tenantId) {
      throw new Error('tenantId is required');
    }

    if (!notification.title && !notification.message) {
      throw new Error('Either title or message is required');
    }

    if (notification.recipients.length === 0) {
      throw new Error('At least one recipient is required');
    }

    if (notification.channels.length === 0) {
      throw new Error('At least one channel is required');
    }

    // Validate channels are registered
    for (const channel of notification.channels) {
      if (!this.channels.has(channel)) {
        throw new Error(`Channel ${channel} is not registered`);
      }
    }
  }

  /**
   * Check if notification is duplicate
   */
  private isDuplicate(notification: Notification): boolean {
    if (!notification.deduplicationKey) {
      return false;
    }

    const lastSent = this.deduplicationCache.get(notification.deduplicationKey);
    if (!lastSent) {
      return false;
    }

    const elapsed = Date.now() - lastSent.getTime();
    return elapsed < this.config.deduplicationWindow;
  }

  /**
   * Add notification to deduplication cache
   */
  private addToDeduplicationCache(notification: Notification): void {
    if (notification.deduplicationKey) {
      this.deduplicationCache.set(notification.deduplicationKey, new Date());
    }
  }

  /**
   * Clean expired entries from deduplication cache
   */
  private cleanDeduplicationCache(): void {
    const now = Date.now();
    const toDelete: string[] = [];

    for (const [key, timestamp] of this.deduplicationCache.entries()) {
      if (now - timestamp.getTime() > this.config.deduplicationWindow) {
        toDelete.push(key);
      }
    }

    for (const key of toDelete) {
      this.deduplicationCache.delete(key);
    }
  }

  /**
   * Enqueue notification
   */
  private enqueue(notification: Notification): void {
    const queue = this.queue.get(notification.priority);
    if (queue) {
      queue.push(notification);
    }
  }

  /**
   * Process queue
   */
  private async processQueue(): Promise<void> {
    if (this.processing.size >= this.config.maxConcurrent) {
      return;
    }

    // Process in priority order
    const priorities: NotificationPriority[] = [
      NotificationPriority.CRITICAL,
      NotificationPriority.URGENT,
      NotificationPriority.HIGH,
      NotificationPriority.NORMAL,
      NotificationPriority.LOW,
    ];

    for (const priority of priorities) {
      const queue = this.queue.get(priority);
      if (!queue || queue.length === 0) {
        continue;
      }

      while (queue.length > 0 && this.processing.size < this.config.maxConcurrent) {
        const notification = queue.shift();
        if (notification) {
          this.processNotification(notification).catch(error => {
            this.emit('error', error);
          });
        }
      }

      if (this.processing.size >= this.config.maxConcurrent) {
        break;
      }
    }
  }

  /**
   * Process individual notification
   */
  private async processNotification(notification: Notification): Promise<void> {
    this.processing.add(notification.id);
    this.emit('notification:processing', notification);

    try {
      // Check if scheduled
      if (notification.scheduledFor && notification.scheduledFor > new Date()) {
        this.enqueue(notification);
        this.processing.delete(notification.id);
        return;
      }

      // Check if expired
      if (notification.expiresAt && notification.expiresAt < new Date()) {
        notification.status = NotificationStatus.FAILED;
        this.emit('notification:expired', notification);
        this.processing.delete(notification.id);
        return;
      }

      // Send through all channels
      const results = await Promise.allSettled(
        notification.channels.map(async channelType => {
          const handler = this.channels.get(channelType);
          if (!handler || !handler.supports(notification)) {
            throw new Error(`Channel ${channelType} not available`);
          }

          const isHealthy = await handler.isHealthy();
          if (!isHealthy) {
            throw new Error(`Channel ${channelType} is not healthy`);
          }

          return await handler.send(notification);
        })
      );

      // Check results
      const successful = results.filter(r => r.status === 'fulfilled').length;
      const failed = results.filter(r => r.status === 'rejected').length;

      if (successful > 0) {
        notification.status = NotificationStatus.SENT;
        notification.sentAt = new Date();
        this.addToDeduplicationCache(notification);
        this.emit('notification:sent', notification);
      } else {
        throw new Error(`All channels failed: ${failed} failures`);
      }
    } catch (error) {
      notification.attempts++;
      notification.lastAttemptAt = new Date();

      if (notification.attempts < notification.maxAttempts) {
        // Retry with exponential backoff
        const delay =
          this.config.retryDelay * Math.pow(this.config.retryBackoff, notification.attempts - 1);
        setTimeout(() => {
          this.enqueue(notification);
        }, delay);

        this.emit('notification:retry', notification, error);
      } else {
        notification.status = NotificationStatus.FAILED;
        this.emit('notification:failed', notification, error);
      }
    } finally {
      this.processing.delete(notification.id);
    }
  }

  /**
   * Generate unique ID
   */
  private generateId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}

export default NotificationEngine;
