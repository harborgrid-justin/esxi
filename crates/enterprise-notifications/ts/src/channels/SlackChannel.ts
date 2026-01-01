/**
 * SlackChannel - Slack notification delivery
 * Supports webhooks, bot tokens, and rich message formatting
 */

import { EventEmitter } from 'events';
import { WebClient } from '@slack/web-api';
import {
  Notification,
  NotificationChannelType,
  DeliveryAttempt,
  SlackChannelConfig,
} from '../types';

export interface SlackMessageOptions {
  channel?: string;
  username?: string;
  iconEmoji?: string;
  iconUrl?: string;
  threadTs?: string;
}

export interface SlackBlock {
  type: string;
  text?: {
    type: string;
    text: string;
  };
  fields?: Array<{
    type: string;
    text: string;
  }>;
  accessory?: unknown;
}

export class SlackChannel extends EventEmitter {
  private config: SlackChannelConfig;
  private client?: WebClient;
  private isInitialized: boolean;

  constructor(config: SlackChannelConfig) {
    super();
    this.config = config;
    this.isInitialized = false;
  }

  /**
   * Initialize Slack client
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      if (this.config.token || this.config.botToken) {
        this.client = new WebClient(this.config.token ?? this.config.botToken);
        // Test authentication
        await this.client.auth.test();
      }

      this.isInitialized = true;
      this.emit('initialized');
    } catch (error) {
      this.emit('error', error);
      throw error;
    }
  }

  /**
   * Send Slack notification
   */
  async send(notification: Notification, options: SlackMessageOptions = {}): Promise<DeliveryAttempt> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const attempt: DeliveryAttempt = {
      id: this.generateAttemptId(),
      notificationId: notification.id,
      channel: NotificationChannelType.SLACK,
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
      } else if (this.client) {
        // Use Web API
        result = await this.sendViaAPI(notification, options);
      } else {
        throw new Error('No Slack configuration available');
      }

      attempt.status = 'sent';
      attempt.sentAt = new Date();
      attempt.externalId = result.ts;
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
    options: SlackMessageOptions
  ): Promise<{ ok: boolean; ts?: string }> {
    if (!this.config.webhookUrl) {
      throw new Error('Webhook URL not configured');
    }

    const payload = {
      text: notification.message,
      blocks: this.buildBlocks(notification),
      username: options.username,
      icon_emoji: options.iconEmoji,
      icon_url: options.iconUrl,
      thread_ts: options.threadTs,
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
   * Send via Web API
   */
  private async sendViaAPI(
    notification: Notification,
    options: SlackMessageOptions
  ): Promise<{ ok: boolean; ts: string; channel: string }> {
    if (!this.client) {
      throw new Error('Slack client not initialized');
    }

    const result = await this.client.chat.postMessage({
      channel: options.channel ?? this.config.defaultChannel ?? '#general',
      text: notification.message,
      blocks: this.buildBlocks(notification),
      username: options.username,
      icon_emoji: options.iconEmoji,
      icon_url: options.iconUrl,
      thread_ts: options.threadTs,
    });

    return {
      ok: result.ok,
      ts: result.ts!,
      channel: result.channel!,
    };
  }

  /**
   * Build Slack blocks for rich formatting
   */
  private buildBlocks(notification: Notification): SlackBlock[] {
    const blocks: SlackBlock[] = [];

    // Header
    if (notification.title) {
      blocks.push({
        type: 'header',
        text: {
          type: 'plain_text',
          text: notification.title,
        },
      });
    }

    // Message body
    blocks.push({
      type: 'section',
      text: {
        type: 'mrkdwn',
        text: notification.message,
      },
    });

    // Priority indicator
    const priorityEmoji = this.getPriorityEmoji(notification.priority);
    if (priorityEmoji) {
      blocks.push({
        type: 'context',
        text: {
          type: 'mrkdwn',
          text: `${priorityEmoji} *Priority:* ${notification.priority}`,
        },
      });
    }

    // Additional fields
    if (notification.data) {
      const fields = Object.entries(notification.data).slice(0, 10).map(([key, value]) => ({
        type: 'mrkdwn',
        text: `*${key}:*\n${String(value)}`,
      }));

      if (fields.length > 0) {
        blocks.push({
          type: 'section',
          fields,
        });
      }
    }

    // Action buttons
    if (notification.actionUrl && notification.actionLabel) {
      blocks.push({
        type: 'actions',
        elements: [
          {
            type: 'button',
            text: {
              type: 'plain_text',
              text: notification.actionLabel,
            },
            url: notification.actionUrl,
          },
        ],
      } as unknown as SlackBlock);
    }

    // Divider
    blocks.push({
      type: 'divider',
    });

    return blocks;
  }

  /**
   * Get priority emoji
   */
  private getPriorityEmoji(priority: string): string {
    const emojiMap: Record<string, string> = {
      critical: ':rotating_light:',
      urgent: ':warning:',
      high: ':exclamation:',
      normal: ':information_source:',
      low: ':white_check_mark:',
    };
    return emojiMap[priority] ?? '';
  }

  /**
   * Update message
   */
  async updateMessage(
    channel: string,
    timestamp: string,
    notification: Notification
  ): Promise<void> {
    if (!this.client) {
      throw new Error('Slack client not initialized');
    }

    await this.client.chat.update({
      channel,
      ts: timestamp,
      text: notification.message,
      blocks: this.buildBlocks(notification),
    });

    this.emit('message:updated', { channel, timestamp });
  }

  /**
   * Delete message
   */
  async deleteMessage(channel: string, timestamp: string): Promise<void> {
    if (!this.client) {
      throw new Error('Slack client not initialized');
    }

    await this.client.chat.delete({
      channel,
      ts: timestamp,
    });

    this.emit('message:deleted', { channel, timestamp });
  }

  /**
   * Check if channel is healthy
   */
  async isHealthy(): Promise<boolean> {
    if (!this.isInitialized) {
      return false;
    }

    try {
      if (this.client) {
        await this.client.auth.test();
        return true;
      } else if (this.config.webhookUrl) {
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
    return `slack_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Cleanup
   */
  async close(): Promise<void> {
    this.isInitialized = false;
    this.emit('closed');
  }
}

export default SlackChannel;
