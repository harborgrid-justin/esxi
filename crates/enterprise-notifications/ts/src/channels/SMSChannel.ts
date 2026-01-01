/**
 * SMSChannel - SMS notification delivery via Twilio and other providers
 * Supports international messaging with validation
 */

import { EventEmitter } from 'events';
import {
  Notification,
  NotificationChannelType,
  DeliveryAttempt,
  SMSChannelConfig,
} from '../types';

export interface SMSOptions {
  from?: string;
  mediaUrls?: string[];
  statusCallback?: string;
}

export class SMSChannel extends EventEmitter {
  private config: SMSChannelConfig;
  private client?: unknown; // Twilio client would be typed properly
  private isInitialized: boolean;

  constructor(config: SMSChannelConfig) {
    super();
    this.config = config;
    this.isInitialized = false;
  }

  /**
   * Initialize SMS provider
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      switch (this.config.provider) {
        case 'twilio':
          // Initialize Twilio client
          // const twilio = require('twilio');
          // this.client = twilio(this.config.accountSid, this.config.authToken);
          break;

        case 'aws-sns':
          // Initialize AWS SNS client
          break;

        case 'nexmo':
          // Initialize Nexmo/Vonage client
          break;

        case 'messagebird':
          // Initialize MessageBird client
          break;

        default:
          throw new Error(`Unsupported SMS provider: ${this.config.provider}`);
      }

      this.isInitialized = true;
      this.emit('initialized');
    } catch (error) {
      this.emit('error', error);
      throw error;
    }
  }

  /**
   * Send SMS notification
   */
  async send(notification: Notification, options: SMSOptions = {}): Promise<DeliveryAttempt> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    const attempt: DeliveryAttempt = {
      id: this.generateAttemptId(),
      notificationId: notification.id,
      channel: NotificationChannelType.SMS,
      recipientId: notification.recipients[0]?.id ?? '',
      status: 'pending',
      attemptNumber: 1,
      scheduledFor: new Date(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    try {
      // Collect phone numbers
      const recipients = notification.recipients.filter(r => r.phone);

      if (recipients.length === 0) {
        throw new Error('No phone number recipients found');
      }

      // Send to each recipient
      const results = await Promise.allSettled(
        recipients.map(recipient =>
          this.sendToRecipient(
            recipient.phone!,
            notification.message,
            options.from ?? this.config.from,
            options
          )
        )
      );

      const successful = results.filter(r => r.status === 'fulfilled');

      if (successful.length > 0) {
        attempt.status = 'sent';
        attempt.sentAt = new Date();
        attempt.response = {
          sent: successful.length,
          total: recipients.length,
        };
        this.emit('sent', attempt);
      } else {
        throw new Error('All SMS deliveries failed');
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
   * Send SMS to single recipient
   */
  private async sendToRecipient(
    to: string,
    message: string,
    from: string,
    options: SMSOptions
  ): Promise<{ sid: string; status: string }> {
    // Validate phone number
    const validTo = this.formatPhoneNumber(to);
    const validFrom = this.formatPhoneNumber(from);

    if (!validTo || !validFrom) {
      throw new Error('Invalid phone number format');
    }

    // Truncate message if needed (160 chars for standard SMS)
    const truncatedMessage = message.length > 160 ? message.substring(0, 157) + '...' : message;

    // Mock sending (real implementation would use actual provider SDK)
    const result = {
      sid: this.generateMessageId(),
      status: 'queued',
      to: validTo,
      from: validFrom,
      body: truncatedMessage,
    };

    this.emit('sms:sent', result);
    return result;
  }

  /**
   * Send bulk SMS
   */
  async sendBulk(
    notifications: Notification[],
    options: SMSOptions = {}
  ): Promise<DeliveryAttempt[]> {
    const results: DeliveryAttempt[] = [];

    for (const notification of notifications) {
      try {
        const attempt = await this.send(notification, options);
        results.push(attempt);
      } catch (error) {
        this.emit('error', error);
      }
    }

    return results;
  }

  /**
   * Format phone number to E.164 format
   */
  private formatPhoneNumber(phone: string): string | null {
    // Remove all non-digit characters except +
    const cleaned = phone.replace(/[^\d+]/g, '');

    // Check if it's already in E.164 format
    if (cleaned.startsWith('+') && cleaned.length >= 10 && cleaned.length <= 15) {
      return cleaned;
    }

    // If no country code, assume US (+1)
    if (!cleaned.startsWith('+')) {
      if (cleaned.length === 10) {
        return `+1${cleaned}`;
      } else if (cleaned.length === 11 && cleaned.startsWith('1')) {
        return `+${cleaned}`;
      }
    }

    return null;
  }

  /**
   * Validate phone number
   */
  async validatePhoneNumber(phone: string): Promise<boolean> {
    const formatted = this.formatPhoneNumber(phone);
    return formatted !== null;
  }

  /**
   * Get SMS delivery status
   */
  async getDeliveryStatus(messageId: string): Promise<{
    status: string;
    errorCode?: string;
    errorMessage?: string;
  }> {
    // Mock implementation
    return {
      status: 'delivered',
    };
  }

  /**
   * Check if channel is healthy
   */
  async isHealthy(): Promise<boolean> {
    if (!this.isInitialized) {
      return false;
    }

    try {
      // In real implementation, would verify account balance/credentials
      return true;
    } catch (error) {
      this.emit('health:unhealthy', error);
      return false;
    }
  }

  /**
   * Generate unique attempt ID
   */
  private generateAttemptId(): string {
    return `sms_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Generate message ID
   */
  private generateMessageId(): string {
    return `SM${Math.random().toString(36).substr(2, 32).toUpperCase()}`;
  }

  /**
   * Get account balance (provider specific)
   */
  async getBalance(): Promise<{ balance: number; currency: string }> {
    // Mock implementation
    return {
      balance: 100.0,
      currency: 'USD',
    };
  }

  /**
   * Cleanup
   */
  async close(): Promise<void> {
    this.isInitialized = false;
    this.emit('closed');
  }
}

export default SMSChannel;
