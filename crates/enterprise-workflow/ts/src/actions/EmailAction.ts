/**
 * Email Action - Send emails via SMTP or email service
 */

import { EmailActionConfig, Context } from '../types';

export class EmailAction {
  /**
   * Execute email sending
   */
  async execute(config: EmailActionConfig, context: Context): Promise<any> {
    // Interpolate email fields with context variables
    const email = {
      to: this.interpolateRecipients(config.to, context),
      cc: config.cc ? this.interpolateRecipients(config.cc, context) : undefined,
      bcc: config.bcc ? this.interpolateRecipients(config.bcc, context) : undefined,
      subject: this.interpolateString(config.subject, context),
      body: this.interpolateString(config.body, context),
      html: config.html ? this.interpolateString(config.html, context) : undefined,
      attachments: config.attachments
    };

    // In production, this would use a real email service (SendGrid, SES, etc.)
    return this.sendEmail(email);
  }

  /**
   * Send email (placeholder implementation)
   */
  private async sendEmail(email: any): Promise<any> {
    // This is a placeholder. In production, integrate with actual email service:
    // - AWS SES
    // - SendGrid
    // - Mailgun
    // - SMTP server
    // - etc.

    console.log('Sending email:', {
      to: email.to,
      subject: email.subject
    });

    return {
      success: true,
      messageId: `msg_${Date.now()}`,
      timestamp: new Date(),
      recipients: email.to
    };
  }

  /**
   * Interpolate recipients with context variables
   */
  private interpolateRecipients(
    recipients: string | string[],
    context: Context
  ): string[] {
    const recipientList = Array.isArray(recipients) ? recipients : [recipients];
    return recipientList.map(r => this.interpolateString(r, context));
  }

  /**
   * Interpolate string with variables
   */
  private interpolateString(str: string, context: Context): string {
    return str.replace(/\${([^}]+)}/g, (match, varName) => {
      const value = context.variables.get(varName.trim());
      return value !== undefined ? String(value) : match;
    });
  }

  /**
   * Validate email action configuration
   */
  validate(config: EmailActionConfig): string[] {
    const errors: string[] = [];

    if (!config.to || (Array.isArray(config.to) && config.to.length === 0)) {
      errors.push('At least one recipient is required');
    }

    if (!config.subject) {
      errors.push('Email subject is required');
    }

    if (!config.body && !config.html) {
      errors.push('Email body or HTML content is required');
    }

    // Validate email addresses
    const allRecipients = [
      ...(Array.isArray(config.to) ? config.to : [config.to]),
      ...(config.cc ? (Array.isArray(config.cc) ? config.cc : [config.cc]) : []),
      ...(config.bcc ? (Array.isArray(config.bcc) ? config.bcc : [config.bcc]) : [])
    ];

    allRecipients.forEach(email => {
      if (!this.isValidEmail(email) && !email.includes('${')) {
        errors.push(`Invalid email address: ${email}`);
      }
    });

    return errors;
  }

  /**
   * Validate email address format
   */
  private isValidEmail(email: string): boolean {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  }
}
