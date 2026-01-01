/**
 * NotificationService - Core notification operations service
 * Provides high-level API for notification management
 */

import { EventEmitter } from 'events';
import {
  Notification,
  NotificationTemplate,
  NotificationPreference,
  DeliveryAttempt,
} from '../types';
import { NotificationEngine } from '../engine/NotificationEngine';
import { TemplateEngine } from '../engine/TemplateEngine';
import { DeliveryEngine } from '../engine/DeliveryEngine';

export class NotificationService extends EventEmitter {
  private notificationEngine: NotificationEngine;
  private templateEngine: TemplateEngine;
  private deliveryEngine: DeliveryEngine;
  private notifications: Map<string, Notification>;
  private templates: Map<string, NotificationTemplate>;
  private preferences: Map<string, NotificationPreference>;

  constructor(
    notificationEngine: NotificationEngine,
    templateEngine: TemplateEngine,
    deliveryEngine: DeliveryEngine
  ) {
    super();
    this.notificationEngine = notificationEngine;
    this.templateEngine = templateEngine;
    this.deliveryEngine = deliveryEngine;
    this.notifications = new Map();
    this.templates = new Map();
    this.preferences = new Map();
  }

  /**
   * Send notification
   */
  async send(notification: Partial<Notification>): Promise<Notification> {
    // Apply user preferences
    const filtered = await this.applyPreferences(notification);
    if (!filtered) {
      throw new Error('Notification filtered by user preferences');
    }

    // Send through engine
    const sent = await this.notificationEngine.send(filtered);

    // Store notification
    this.notifications.set(sent.id, sent);

    this.emit('notification:sent', sent);
    return sent;
  }

  /**
   * Send from template
   */
  async sendFromTemplate(
    templateId: string,
    data: Record<string, unknown>,
    recipients: Notification['recipients'],
    options?: Partial<Notification>
  ): Promise<Notification> {
    const template = this.templates.get(templateId);
    if (!template) {
      throw new Error(`Template ${templateId} not found`);
    }

    // Render template for primary channel
    const primaryChannel = template.defaultChannels[0];
    if (!primaryChannel) {
      throw new Error('No default channel in template');
    }

    const rendered = this.templateEngine.render(template, primaryChannel, { data });

    // Create notification from template
    const notification: Partial<Notification> = {
      ...options,
      templateId,
      title: rendered.subject ?? template.name,
      message: rendered.plainText ?? rendered.body,
      html: rendered.html,
      channels: options?.channels ?? template.defaultChannels,
      priority: options?.priority ?? template.defaultPriority,
      recipients,
      templateData: data,
    };

    return await this.send(notification);
  }

  /**
   * Get notification by ID
   */
  async getNotification(notificationId: string): Promise<Notification | undefined> {
    return this.notifications.get(notificationId);
  }

  /**
   * Get notifications for user
   */
  async getUserNotifications(
    userId: string,
    filter?: {
      status?: string[];
      priority?: string[];
      startDate?: Date;
      endDate?: Date;
    }
  ): Promise<Notification[]> {
    let notifications = Array.from(this.notifications.values()).filter(n =>
      n.recipients.some(r => r.id === userId)
    );

    if (filter) {
      notifications = notifications.filter(n => {
        if (filter.status && !filter.status.includes(n.status)) {
          return false;
        }
        if (filter.priority && !filter.priority.includes(n.priority)) {
          return false;
        }
        if (filter.startDate && n.createdAt < filter.startDate) {
          return false;
        }
        if (filter.endDate && n.createdAt > filter.endDate) {
          return false;
        }
        return true;
      });
    }

    return notifications.sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());
  }

  /**
   * Save template
   */
  async saveTemplate(template: NotificationTemplate): Promise<void> {
    this.templates.set(template.id, template);
    this.emit('template:saved', template);
  }

  /**
   * Get template
   */
  async getTemplate(templateId: string): Promise<NotificationTemplate | undefined> {
    return this.templates.get(templateId);
  }

  /**
   * Save user preferences
   */
  async savePreferences(preferences: NotificationPreference): Promise<void> {
    const key = `${preferences.userId}:${preferences.tenantId}`;
    this.preferences.set(key, preferences);
    this.emit('preferences:saved', preferences);
  }

  /**
   * Get user preferences
   */
  async getPreferences(userId: string, tenantId: string): Promise<NotificationPreference | undefined> {
    const key = `${userId}:${tenantId}`;
    return this.preferences.get(key);
  }

  /**
   * Apply user preferences to notification
   */
  private async applyPreferences(
    notification: Partial<Notification>
  ): Promise<Partial<Notification> | null> {
    if (!notification.userId || !notification.tenantId) {
      return notification;
    }

    const preferences = await this.getPreferences(notification.userId, notification.tenantId);
    if (!preferences || !preferences.enabled) {
      return null;
    }

    // Check if muted
    if (preferences.mutedUntil && preferences.mutedUntil > new Date()) {
      return null;
    }

    // Check quiet hours
    if (preferences.quietHours?.enabled) {
      const inQuietHours = this.isInQuietHours(preferences.quietHours);
      if (inQuietHours) {
        const allowedPriorities = ['urgent', 'critical'];
        if (!notification.priority || !allowedPriorities.includes(notification.priority)) {
          return null;
        }
      }
    }

    return notification;
  }

  /**
   * Check if current time is in quiet hours
   */
  private isInQuietHours(quietHours: NonNullable<NotificationPreference['quietHours']>): boolean {
    const now = new Date();
    const currentDay = now.getDay();

    // Check if current day is in quiet hours
    if (quietHours.days && !quietHours.days.includes(currentDay)) {
      return false;
    }

    const currentTime = `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;

    return currentTime >= quietHours.startTime && currentTime <= quietHours.endTime;
  }
}

export default NotificationService;
