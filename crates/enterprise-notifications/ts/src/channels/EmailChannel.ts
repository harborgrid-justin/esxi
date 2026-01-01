/**
 * EmailChannel - Email notification delivery via SMTP/SES
 * Supports multiple email providers and templates
 */

import nodemailer from 'nodemailer';
import type { Transporter } from 'nodemailer';
import { EventEmitter } from 'events';
import {
  Notification,
  NotificationChannelType,
  DeliveryAttempt,
  EmailChannelConfig,
} from '../types';

export interface EmailAttachment {
  filename: string;
  content?: Buffer | string;
  path?: string;
  contentType?: string;
}

export interface EmailOptions {
  from?: string;
  fromName?: string;
  replyTo?: string;
  cc?: string[];
  bcc?: string[];
  attachments?: EmailAttachment[];
  headers?: Record<string, string>;
}

export class EmailChannel extends EventEmitter {
  private config: EmailChannelConfig;
  private transporter?: Transporter;
  private isInitialized: boolean;

  constructor(config: EmailChannelConfig) {
    super();
    this.config = config;
    this.isInitialized = false;
  }

  /**
   * Initialize email transport
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      switch (this.config.provider) {
        case 'smtp':
          this.transporter = nodemailer.createTransport({
            host: this.config.host,
            port: this.config.port ?? 587,
            secure: this.config.secure ?? false,
            auth: this.config.auth,
          });
          break;

        case 'ses':
          // AWS SES transport would be configured here
          // For now, using SMTP interface
          this.transporter = nodemailer.createTransport({
            host: `email-smtp.${this.config.region}.amazonaws.com`,
            port: 587,
            secure: false,
            auth: {
              user: this.config.accessKeyId ?? '',
              pass: this.config.secretAccessKey ?? '',
            },
          });
          break;

        case 'sendgrid':
          this.transporter = nodemailer.createTransport({
            host: 'smtp.sendgrid.net',
            port: 587,
            auth: {
              user: 'apikey',
              pass: this.config.apiKey ?? '',
            },
          });
          break;

        case 'mailgun':
          this.transporter = nodemailer.createTransport({
            host: 'smtp.mailgun.org',
            port: 587,
            auth: {
              user: `postmaster@${this.config.domain}`,
              pass: this.config.apiKey ?? '',
            },
          });
          break;

        default:
          throw new Error(`Unsupported email provider: ${this.config.provider}`);
      }

      // Verify connection
      if (this.transporter) {
        await this.transporter.verify();
      }

      this.isInitialized = true;
      this.emit('initialized');
    } catch (error) {
      this.emit('error', error);
      throw error;
    }
  }

  /**
   * Send email notification
   */
  async send(notification: Notification, options: EmailOptions = {}): Promise<DeliveryAttempt> {
    if (!this.isInitialized) {
      await this.initialize();
    }

    if (!this.transporter) {
      throw new Error('Email transporter not initialized');
    }

    const attempt: DeliveryAttempt = {
      id: this.generateAttemptId(),
      notificationId: notification.id,
      channel: NotificationChannelType.EMAIL,
      recipientId: notification.recipients[0]?.id ?? '',
      status: 'pending',
      attemptNumber: 1,
      scheduledFor: new Date(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    try {
      // Collect email addresses
      const to = notification.recipients
        .filter(r => r.email)
        .map(r => (r.name ? `"${r.name}" <${r.email}>` : r.email!));

      if (to.length === 0) {
        throw new Error('No email recipients found');
      }

      // Prepare email
      const mailOptions = {
        from: options.from ?? this.config.from,
        to: to.join(', '),
        cc: options.cc?.join(', '),
        bcc: options.bcc?.join(', '),
        replyTo: options.replyTo ?? this.config.replyTo,
        subject: notification.title,
        text: notification.message,
        html: notification.html,
        attachments: options.attachments,
        headers: {
          'X-Notification-ID': notification.id,
          'X-Tenant-ID': notification.tenantId,
          'X-Priority': this.getPriorityHeader(notification.priority),
          ...options.headers,
        },
      };

      // Send email
      const info = await this.transporter.sendMail(mailOptions);

      attempt.status = 'sent';
      attempt.sentAt = new Date();
      attempt.externalId = info.messageId;
      attempt.response = {
        messageId: info.messageId,
        accepted: info.accepted,
        rejected: info.rejected,
        response: info.response,
      };

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
   * Send bulk emails
   */
  async sendBulk(
    notifications: Notification[],
    options: EmailOptions = {}
  ): Promise<DeliveryAttempt[]> {
    const results: DeliveryAttempt[] = [];

    for (const notification of notifications) {
      try {
        const attempt = await this.send(notification, options);
        results.push(attempt);
      } catch (error) {
        // Continue with next notification
        this.emit('error', error);
      }
    }

    return results;
  }

  /**
   * Verify email address
   */
  async verifyEmail(email: string): Promise<boolean> {
    // Basic email validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  }

  /**
   * Check if channel is healthy
   */
  async isHealthy(): Promise<boolean> {
    if (!this.isInitialized || !this.transporter) {
      return false;
    }

    try {
      await this.transporter.verify();
      return true;
    } catch (error) {
      this.emit('health:unhealthy', error);
      return false;
    }
  }

  /**
   * Get priority header value
   */
  private getPriorityHeader(priority: string): string {
    const priorityMap: Record<string, string> = {
      critical: '1 (Highest)',
      urgent: '2 (High)',
      high: '3 (Normal)',
      normal: '3 (Normal)',
      low: '5 (Lowest)',
    };
    return priorityMap[priority] ?? '3 (Normal)';
  }

  /**
   * Generate unique attempt ID
   */
  private generateAttemptId(): string {
    return `email_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Close transport
   */
  async close(): Promise<void> {
    if (this.transporter) {
      this.transporter.close();
      this.isInitialized = false;
      this.emit('closed');
    }
  }
}

export default EmailChannel;
