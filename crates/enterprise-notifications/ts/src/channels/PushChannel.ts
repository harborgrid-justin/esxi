/**
 * PushChannel - Web and mobile push notification delivery
 * Supports Web Push, FCM (Firebase), and APNS (Apple)
 */

import { EventEmitter } from 'events';
import webpush from 'web-push';
import {
  Notification,
  NotificationChannelType,
  DeliveryAttempt,
  PushChannelConfig,
} from '../types';

export interface PushSubscription {
  endpoint: string;
  keys: {
    p256dh: string;
    auth: string;
  };
}

export interface PushPayload {
  title: string;
  body: string;
  icon?: string;
  badge?: string;
  image?: string;
  data?: Record<string, unknown>;
  actions?: Array<{
    action: string;
    title: string;
    icon?: string;
  }>;
  tag?: string;
  requireInteraction?: boolean;
  silent?: boolean;
  vibrate?: number[];
}

export class PushChannel extends EventEmitter {
  private config: PushChannelConfig;
  private isInitialized: boolean;

  constructor(config: PushChannelConfig) {
    super();
    this.config = config;
    this.isInitialized = false;
  }

  /**
   * Initialize push service
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      if (this.config.platform === 'web' || this.config.platform === 'all') {
        // Initialize Web Push
        if (
          this.config.vapidPublicKey &&
          this.config.vapidPrivateKey &&
          this.config.vapidSubject
        ) {
          webpush.setVapidDetails(
            this.config.vapidSubject,
            this.config.vapidPublicKey,
            this.config.vapidPrivateKey
          );
        }
      }

      // Initialize FCM for Android/iOS
      if (
        (this.config.platform === 'android' ||
          this.config.platform === 'ios' ||
          this.config.platform === 'all') &&
        this.config.fcmServerKey
      ) {
        // FCM initialization would go here
      }

      // Initialize APNS for iOS
      if (
        (this.config.platform === 'ios' || this.config.platform === 'all') &&
        this.config.apnsKeyId
      ) {
        // APNS initialization would go here
      }

      this.isInitialized = true;
      this.emit('initialized');
    } catch (error) {
      this.emit('error', error);
      throw error;
    }
  }

  /**
   * Send push notification
   */
  async send(
    notification: Notification,
    subscriptions: PushSubscription[]
  ): Promise<DeliveryAttempt> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const attempt: DeliveryAttempt = {
      id: this.generateAttemptId(),
      notificationId: notification.id,
      channel: NotificationChannelType.PUSH,
      recipientId: notification.recipients[0]?.id ?? '',
      status: 'pending',
      attemptNumber: 1,
      scheduledFor: new Date(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    try {
      if (subscriptions.length === 0) {
        throw new Error('No push subscriptions found');
      }

      // Prepare payload
      const payload = this.preparePayload(notification);

      // Send to all subscriptions
      const results = await Promise.allSettled(
        subscriptions.map(subscription => this.sendToSubscription(subscription, payload))
      );

      const successful = results.filter(r => r.status === 'fulfilled').length;

      if (successful > 0) {
        attempt.status = 'sent';
        attempt.sentAt = new Date();
        attempt.response = {
          sent: successful,
          total: subscriptions.length,
        };
        this.emit('sent', attempt);
      } else {
        throw new Error('All push deliveries failed');
      }
    } catch (error) {
      attempt.status = 'failed';
      attempt.failedAt = new Date();
      attempt.error = error instanceof Error ? error.message : String(error);
      this.emit('failed', attempt, error);
      throw error;
    } finally {
      attempt.updatedAt = new Date();
    }

    return attempt;
  }

  /**
   * Send to single subscription
   */
  private async sendToSubscription(
    subscription: PushSubscription,
    payload: PushPayload
  ): Promise<void> {
    const options = {
      TTL: 86400, // 24 hours
      vapidDetails: this.config.vapidPublicKey
        ? {
            subject: this.config.vapidSubject!,
            publicKey: this.config.vapidPublicKey,
            privateKey: this.config.vapidPrivateKey!,
          }
        : undefined,
    };

    try {
      await webpush.sendNotification(subscription, JSON.stringify(payload), options);
      this.emit('push:sent', subscription.endpoint);
    } catch (error) {
      // Handle expired subscriptions
      if (
        error instanceof Error &&
        (error.message.includes('410') || error.message.includes('expired'))
      ) {
        this.emit('subscription:expired', subscription);
      }
      throw error;
    }
  }

  /**
   * Prepare push payload
   */
  private preparePayload(notification: Notification): PushPayload {
    const payload: PushPayload = {
      title: notification.title,
      body: notification.message,
      icon: notification.icon,
      badge: notification.badge?.toString(),
      image: notification.image,
      data: {
        notificationId: notification.id,
        type: notification.type,
        url: notification.actionUrl,
        ...notification.data,
      },
      tag: notification.groupKey,
      requireInteraction: notification.priority === 'urgent' || notification.priority === 'critical',
      silent: notification.silent ?? false,
    };

    // Add action buttons
    if (notification.links && notification.links.length > 0) {
      payload.actions = notification.links.slice(0, 2).map(link => ({
        action: link.action ?? link.url,
        title: link.label,
      }));
    } else if (notification.actionUrl && notification.actionLabel) {
      payload.actions = [
        {
          action: notification.actionUrl,
          title: notification.actionLabel,
        },
      ];
    }

    // Add vibration pattern for urgent notifications
    if (notification.priority === 'urgent' || notification.priority === 'critical') {
      payload.vibrate = [200, 100, 200];
    }

    return payload;
  }

  /**
   * Send to FCM (Firebase Cloud Messaging)
   */
  async sendFCM(tokens: string[], payload: PushPayload): Promise<{
    successCount: number;
    failureCount: number;
  }> {
    // Mock implementation - real implementation would use FCM SDK
    return {
      successCount: tokens.length,
      failureCount: 0,
    };
  }

  /**
   * Send to APNS (Apple Push Notification Service)
   */
  async sendAPNS(deviceTokens: string[], payload: PushPayload): Promise<{
    successCount: number;
    failureCount: number;
  }> {
    // Mock implementation - real implementation would use APNS SDK
    return {
      successCount: deviceTokens.length,
      failureCount: 0,
    };
  }

  /**
   * Validate push subscription
   */
  async validateSubscription(subscription: PushSubscription): Promise<boolean> {
    try {
      // Send a test notification
      const testPayload = {
        title: 'Test',
        body: 'Subscription validation',
        silent: true,
      };

      await this.sendToSubscription(subscription, testPayload);
      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * Check if channel is healthy
   */
  async isHealthy(): Promise<boolean> {
    return this.isInitialized;
  }

  /**
   * Generate unique attempt ID
   */
  private generateAttemptId(): string {
    return `push_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Cleanup
   */
  async close(): Promise<void> {
    this.isInitialized = false;
    this.emit('closed');
  }
}

export default PushChannel;
