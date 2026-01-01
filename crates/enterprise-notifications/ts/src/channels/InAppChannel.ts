/**
 * InAppChannel - In-app notification delivery
 * Manages persistent notifications within the application
 */

import { EventEmitter } from 'events';
import {
  Notification,
  NotificationChannelType,
  NotificationStatus,
  DeliveryAttempt,
  InAppChannelConfig,
} from '../types';

export interface StoredNotification extends Notification {
  isRead: boolean;
  readAt?: Date;
  deletedAt?: Date;
  expiresAt?: Date;
}

export interface NotificationFilter {
  userId?: string;
  tenantId?: string;
  type?: string;
  category?: string;
  priority?: string[];
  isRead?: boolean;
  startDate?: Date;
  endDate?: Date;
}

export class InAppChannel extends EventEmitter {
  private config: InAppChannelConfig;
  private notifications: Map<string, StoredNotification>;
  private userNotifications: Map<string, Set<string>>; // userId -> notificationIds
  private isInitialized: boolean;
  private cleanupInterval?: NodeJS.Timeout;

  constructor(config: InAppChannelConfig) {
    super();
    this.config = config;
    this.notifications = new Map();
    this.userNotifications = new Map();
    this.isInitialized = false;
  }

  /**
   * Initialize in-app channel
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    // Start cleanup interval if persistence is enabled
    if (this.config.enablePersistence) {
      this.cleanupInterval = setInterval(() => {
        this.cleanupExpiredNotifications();
      }, 3600000); // Every hour
    }

    this.isInitialized = true;
    this.emit('initialized');
  }

  /**
   * Send in-app notification
   */
  async send(notification: Notification): Promise<DeliveryAttempt> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const attempt: DeliveryAttempt = {
      id: this.generateAttemptId(),
      notificationId: notification.id,
      channel: NotificationChannelType.IN_APP,
      recipientId: notification.recipients[0]?.id ?? '',
      status: 'pending',
      attemptNumber: 1,
      scheduledFor: new Date(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    try {
      // Store notification for each recipient
      for (const recipient of notification.recipients) {
        await this.storeNotification(notification, recipient.id);
      }

      attempt.status = 'delivered';
      attempt.sentAt = new Date();
      attempt.deliveredAt = new Date();

      this.emit('sent', attempt);
      this.emit('notification:new', notification);
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
   * Store notification for user
   */
  private async storeNotification(notification: Notification, userId: string): Promise<void> {
    const stored: StoredNotification = {
      ...notification,
      userId,
      isRead: false,
      expiresAt: this.calculateExpiry(),
    };

    this.notifications.set(notification.id, stored);

    // Update user index
    let userNotifs = this.userNotifications.get(userId);
    if (!userNotifs) {
      userNotifs = new Set();
      this.userNotifications.set(userId, userNotifs);
    }
    userNotifs.add(notification.id);

    this.emit('notification:stored', stored);
  }

  /**
   * Get notifications for user
   */
  async getNotifications(
    userId: string,
    filter?: NotificationFilter,
    limit: number = 50,
    offset: number = 0
  ): Promise<StoredNotification[]> {
    const userNotifIds = this.userNotifications.get(userId);
    if (!userNotifIds) {
      return [];
    }

    let notifications = Array.from(userNotifIds)
      .map(id => this.notifications.get(id))
      .filter((n): n is StoredNotification => n !== undefined && !n.deletedAt);

    // Apply filters
    if (filter) {
      notifications = this.applyFilters(notifications, filter);
    }

    // Sort by creation date (newest first)
    notifications.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());

    // Apply pagination
    return notifications.slice(offset, offset + limit);
  }

  /**
   * Get unread count for user
   */
  async getUnreadCount(userId: string, filter?: NotificationFilter): Promise<number> {
    const notifications = await this.getNotifications(userId, {
      ...filter,
      isRead: false,
    });
    return notifications.length;
  }

  /**
   * Mark notification as read
   */
  async markAsRead(notificationId: string, userId: string): Promise<boolean> {
    const notification = this.notifications.get(notificationId);
    if (!notification || notification.userId !== userId) {
      return false;
    }

    notification.isRead = true;
    notification.readAt = new Date();
    notification.status = NotificationStatus.READ;

    this.emit('notification:read', notification);
    return true;
  }

  /**
   * Mark all notifications as read
   */
  async markAllAsRead(userId: string, filter?: NotificationFilter): Promise<number> {
    const notifications = await this.getNotifications(userId, {
      ...filter,
      isRead: false,
    });

    for (const notification of notifications) {
      notification.isRead = true;
      notification.readAt = new Date();
      notification.status = NotificationStatus.READ;
    }

    this.emit('notifications:read:bulk', userId, notifications.length);
    return notifications.length;
  }

  /**
   * Delete notification
   */
  async deleteNotification(notificationId: string, userId: string): Promise<boolean> {
    const notification = this.notifications.get(notificationId);
    if (!notification || notification.userId !== userId) {
      return false;
    }

    if (this.config.enablePersistence) {
      // Soft delete
      notification.deletedAt = new Date();
    } else {
      // Hard delete
      this.notifications.delete(notificationId);
      const userNotifs = this.userNotifications.get(userId);
      if (userNotifs) {
        userNotifs.delete(notificationId);
      }
    }

    this.emit('notification:deleted', notificationId);
    return true;
  }

  /**
   * Delete all notifications for user
   */
  async deleteAllNotifications(userId: string, filter?: NotificationFilter): Promise<number> {
    const notifications = await this.getNotifications(userId, filter);

    for (const notification of notifications) {
      if (this.config.enablePersistence) {
        notification.deletedAt = new Date();
      } else {
        this.notifications.delete(notification.id);
      }
    }

    if (!this.config.enablePersistence) {
      this.userNotifications.delete(userId);
    }

    this.emit('notifications:deleted:bulk', userId, notifications.length);
    return notifications.length;
  }

  /**
   * Apply filters to notifications
   */
  private applyFilters(
    notifications: StoredNotification[],
    filter: NotificationFilter
  ): StoredNotification[] {
    return notifications.filter(notification => {
      if (filter.type && notification.type !== filter.type) {
        return false;
      }

      if (filter.category && notification.category !== filter.category) {
        return false;
      }

      if (filter.priority && !filter.priority.includes(notification.priority)) {
        return false;
      }

      if (filter.isRead !== undefined && notification.isRead !== filter.isRead) {
        return false;
      }

      if (filter.startDate && notification.createdAt < filter.startDate) {
        return false;
      }

      if (filter.endDate && notification.createdAt > filter.endDate) {
        return false;
      }

      return true;
    });
  }

  /**
   * Calculate expiry date
   */
  private calculateExpiry(): Date {
    const now = new Date();
    const days = this.config.persistDays;
    return new Date(now.getTime() + days * 24 * 60 * 60 * 1000);
  }

  /**
   * Cleanup expired notifications
   */
  private cleanupExpiredNotifications(): void {
    const now = new Date();
    let cleaned = 0;

    for (const [id, notification] of this.notifications.entries()) {
      if (notification.expiresAt && notification.expiresAt < now) {
        this.notifications.delete(id);

        if (notification.userId) {
          const userNotifs = this.userNotifications.get(notification.userId);
          if (userNotifs) {
            userNotifs.delete(id);
          }
        }

        cleaned++;
      }
    }

    if (cleaned > 0) {
      this.emit('cleanup:completed', cleaned);
    }
  }

  /**
   * Get statistics
   */
  getStats(userId?: string): {
    total: number;
    unread: number;
    read: number;
    deleted: number;
  } {
    let notifications: StoredNotification[];

    if (userId) {
      const userNotifIds = this.userNotifications.get(userId) ?? new Set();
      notifications = Array.from(userNotifIds)
        .map(id => this.notifications.get(id))
        .filter((n): n is StoredNotification => n !== undefined);
    } else {
      notifications = Array.from(this.notifications.values());
    }

    return {
      total: notifications.length,
      unread: notifications.filter(n => !n.isRead && !n.deletedAt).length,
      read: notifications.filter(n => n.isRead && !n.deletedAt).length,
      deleted: notifications.filter(n => n.deletedAt !== undefined).length,
    };
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
    return `inapp_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Export notifications (for backup/migration)
   */
  exportNotifications(userId?: string): StoredNotification[] {
    if (userId) {
      const userNotifIds = this.userNotifications.get(userId) ?? new Set();
      return Array.from(userNotifIds)
        .map(id => this.notifications.get(id))
        .filter((n): n is StoredNotification => n !== undefined);
    }

    return Array.from(this.notifications.values());
  }

  /**
   * Import notifications (for backup/migration)
   */
  importNotifications(notifications: StoredNotification[]): number {
    let imported = 0;

    for (const notification of notifications) {
      this.notifications.set(notification.id, notification);

      if (notification.userId) {
        let userNotifs = this.userNotifications.get(notification.userId);
        if (!userNotifs) {
          userNotifs = new Set();
          this.userNotifications.set(notification.userId, userNotifs);
        }
        userNotifs.add(notification.id);
      }

      imported++;
    }

    this.emit('import:completed', imported);
    return imported;
  }

  /**
   * Cleanup
   */
  async close(): Promise<void> {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
    }

    this.notifications.clear();
    this.userNotifications.clear();
    this.isInitialized = false;
    this.emit('closed');
  }
}

export default InAppChannel;
