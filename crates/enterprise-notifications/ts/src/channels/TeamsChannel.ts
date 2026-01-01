/**
 * TeamsChannel - Microsoft Teams notification delivery
 * Supports webhooks and Microsoft Graph API
 */

import { EventEmitter } from 'events';
import {
  Notification,
  NotificationChannelType,
  DeliveryAttempt,
  TeamsChannelConfig,
} from '../types';

export interface TeamsMessageOptions {
  channel?: string;
  threadId?: string;
}

export interface TeamsAdaptiveCard {
  type: string;
  version: string;
  body: TeamsCardElement[];
  actions?: TeamsCardAction[];
}

export interface TeamsCardElement {
  type: string;
  text?: string;
  weight?: string;
  size?: string;
  color?: string;
  facts?: Array<{ title: string; value: string }>;
}

export interface TeamsCardAction {
  type: string;
  title: string;
  url?: string;
  data?: unknown;
}

export class TeamsChannel extends EventEmitter {
  private config: TeamsChannelConfig;
  private graphClient?: unknown; // Microsoft Graph client
  private isInitialized: boolean;

  constructor(config: TeamsChannelConfig) {
    super();
    this.config = config;
    this.isInitialized = false;
  }

  /**
   * Initialize Teams client
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      if (this.config.tenantId && this.config.clientId && this.config.clientSecret) {
        // Initialize Microsoft Graph client
        // const { Client } = require('@microsoft/microsoft-graph-client');
        // this.graphClient = Client.init({ ... });
      }

      this.isInitialized = true;
      this.emit('initialized');
    } catch (error) {
      this.emit('error', error);
      throw error;
    }
  }

  /**
   * Send Teams notification
   */
  async send(notification: Notification, options: TeamsMessageOptions = {}): Promise<DeliveryAttempt> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const attempt: DeliveryAttempt = {
      id: this.generateAttemptId(),
      notificationId: notification.id,
      channel: NotificationChannelType.TEAMS,
      recipientId: notification.recipients[0]?.id ?? '',
      status: 'pending',
      attemptNumber: 1,
      scheduledFor: new Date(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    try {
      let result;

      if (this.config.webhookUrl) {
        // Use webhook
        result = await this.sendViaWebhook(notification, options);
      } else if (this.graphClient) {
        // Use Graph API
        result = await this.sendViaGraph(notification, options);
      } else {
        throw new Error('No Teams configuration available');
      }

      attempt.status = 'sent';
      attempt.sentAt = new Date();
      attempt.externalId = result.id;
      attempt.response = result;
      this.emit('sent', attempt);
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
   * Send via webhook
   */
  private async sendViaWebhook(
    notification: Notification,
    options: TeamsMessageOptions
  ): Promise<{ ok: boolean; id?: string }> {
    if (!this.config.webhookUrl) {
      throw new Error('Webhook URL not configured');
    }

    const payload = {
      type: 'message',
      attachments: [
        {
          contentType: 'application/vnd.microsoft.card.adaptive',
          content: this.buildAdaptiveCard(notification),
        },
      ],
    };

    const response = await fetch(this.config.webhookUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      throw new Error(`Webhook request failed: ${response.statusText}`);
    }

    return { ok: true };
  }

  /**
   * Send via Graph API
   */
  private async sendViaGraph(
    notification: Notification,
    options: TeamsMessageOptions
  ): Promise<{ ok: boolean; id: string }> {
    // Mock implementation - real implementation would use Microsoft Graph SDK
    return {
      ok: true,
      id: this.generateMessageId(),
    };
  }

  /**
   * Build Microsoft Teams Adaptive Card
   */
  private buildAdaptiveCard(notification: Notification): TeamsAdaptiveCard {
    const body: TeamsCardElement[] = [];

    // Title
    if (notification.title) {
      body.push({
        type: 'TextBlock',
        text: notification.title,
        weight: 'Bolder',
        size: 'Large',
      });
    }

    // Message
    body.push({
      type: 'TextBlock',
        text: notification.message,
    });

    // Priority indicator
    const priorityColor = this.getPriorityColor(notification.priority);
    body.push({
      type: 'TextBlock',
      text: `Priority: ${notification.priority.toUpperCase()}`,
      color: priorityColor,
      weight: 'Bolder',
    });

    // Additional facts
    if (notification.data) {
      const facts = Object.entries(notification.data)
        .slice(0, 10)
        .map(([key, value]) => ({
          title: key,
          value: String(value),
        }));

      if (facts.length > 0) {
        body.push({
          type: 'FactSet',
          facts,
        });
      }
    }

    // Actions
    const actions: TeamsCardAction[] = [];

    if (notification.actionUrl && notification.actionLabel) {
      actions.push({
        type: 'Action.OpenUrl',
        title: notification.actionLabel,
        url: notification.actionUrl,
      });
    }

    if (notification.links && notification.links.length > 0) {
      for (const link of notification.links.slice(0, 4)) {
        actions.push({
          type: 'Action.OpenUrl',
          title: link.label,
          url: link.url,
        });
      }
    }

    return {
      type: 'AdaptiveCard',
      version: '1.4',
      body,
      actions: actions.length > 0 ? actions : undefined,
    };
  }

  /**
   * Get priority color for Teams
   */
  private getPriorityColor(priority: string): string {
    const colorMap: Record<string, string> = {
      critical: 'Attention',
      urgent: 'Warning',
      high: 'Accent',
      normal: 'Default',
      low: 'Good',
    };
    return colorMap[priority] ?? 'Default';
  }

  /**
   * Send to specific team/channel
   */
  async sendToChannel(
    teamId: string,
    channelId: string,
    notification: Notification
  ): Promise<void> {
    if (!this.graphClient) {
      throw new Error('Graph client not initialized');
    }

    // Mock implementation
    this.emit('channel:sent', { teamId, channelId });
  }

  /**
   * Send direct message to user
   */
  async sendDirectMessage(userId: string, notification: Notification): Promise<void> {
    if (!this.graphClient) {
      throw new Error('Graph client not initialized');
    }

    // Mock implementation
    this.emit('dm:sent', { userId });
  }

  /**
   * Check if channel is healthy
   */
  async isHealthy(): Promise<boolean> {
    if (!this.isInitialized) {
      return false;
    }

    try {
      if (this.config.webhookUrl) {
        return true;
      } else if (this.graphClient) {
        // Would verify Graph API access
        return true;
      }
      return false;
    } catch (error) {
      this.emit('health:unhealthy', error);
      return false;
    }
  }

  /**
   * Generate unique attempt ID
   */
  private generateAttemptId(): string {
    return `teams_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Generate message ID
   */
  private generateMessageId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 16)}`;
  }

  /**
   * Cleanup
   */
  async close(): Promise<void> {
    this.isInitialized = false;
    this.emit('closed');
  }
}

export default TeamsChannel;
