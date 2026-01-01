/**
 * Payment Retry - Handles failed payment retries with smart retry logic
 */

import { addHours, addDays, isBefore } from 'date-fns';
import { Decimal } from 'decimal.js';
import {
  Payment,
  PaymentStatus,
  Invoice,
  Subscription,
  SubscriptionStatus,
  PaymentIntent,
  BillingConfig,
  WebhookEvent,
} from '../types';
import { PaymentGateway } from './PaymentGateway';

export interface RetryAttempt {
  attemptNumber: number;
  attemptedAt: Date;
  nextRetryAt?: Date;
  status: PaymentStatus;
  errorCode?: string;
  errorMessage?: string;
}

export interface RetryPolicy {
  maxAttempts: number;
  intervalHours: number[];
  backoffMultiplier?: number;
  pauseSubscriptionAfter?: number;
}

export class PaymentRetry {
  private retryAttempts: Map<string, RetryAttempt[]> = new Map();
  private eventHandlers: Map<WebhookEvent, ((data: any) => void)[]> = new Map();

  constructor(
    private config: BillingConfig,
    private paymentGateway: PaymentGateway
  ) {}

  /**
   * Schedule retry for failed payment
   */
  async scheduleRetry(
    payment: Payment,
    invoice: Invoice,
    subscription: Subscription
  ): Promise<RetryAttempt> {
    const attempts = this.retryAttempts.get(payment.id) || [];
    const attemptNumber = attempts.length + 1;

    if (attemptNumber > this.config.retryAttempts) {
      await this.handleMaxRetriesExceeded(payment, invoice, subscription);
      throw new Error('Maximum retry attempts exceeded');
    }

    const nextRetryAt = this.calculateNextRetryTime(attemptNumber);

    const attempt: RetryAttempt = {
      attemptNumber,
      attemptedAt: new Date(),
      nextRetryAt,
      status: PaymentStatus.PENDING,
    };

    attempts.push(attempt);
    this.retryAttempts.set(payment.id, attempts);

    // Emit event for scheduled retry
    await this.emitEvent(WebhookEvent.INVOICE_PAYMENT_FAILED, {
      payment,
      invoice,
      subscription,
      attempt,
    });

    return attempt;
  }

  /**
   * Execute retry attempt
   */
  async executeRetry(
    payment: Payment,
    invoice: Invoice,
    paymentMethodId: string
  ): Promise<RetryAttempt> {
    const attempts = this.retryAttempts.get(payment.id) || [];
    const currentAttempt = attempts[attempts.length - 1];

    if (!currentAttempt) {
      throw new Error('No retry scheduled for this payment');
    }

    try {
      const intent: PaymentIntent = {
        amount: invoice.amountDue,
        currency: invoice.currency,
        paymentMethodId,
        customerId: payment.tenantId,
        description: `Invoice ${invoice.number} - Retry attempt ${currentAttempt.attemptNumber}`,
        metadata: {
          invoiceId: invoice.id,
          paymentId: payment.id,
          retryAttempt: currentAttempt.attemptNumber,
        },
      };

      const result = await this.paymentGateway.processPayment(intent);

      if (result.success) {
        currentAttempt.status = PaymentStatus.SUCCEEDED;
        await this.emitEvent(WebhookEvent.PAYMENT_SUCCEEDED, {
          payment,
          invoice,
          attempt: currentAttempt,
        });
      } else {
        currentAttempt.status = PaymentStatus.FAILED;
        currentAttempt.errorCode = result.errorCode;
        currentAttempt.errorMessage = result.errorMessage;

        // Schedule next retry if not max attempts
        if (currentAttempt.attemptNumber < this.config.retryAttempts) {
          const nextRetryAt = this.calculateNextRetryTime(
            currentAttempt.attemptNumber + 1
          );
          currentAttempt.nextRetryAt = nextRetryAt;
        }
      }
    } catch (error: any) {
      currentAttempt.status = PaymentStatus.FAILED;
      currentAttempt.errorCode = 'RETRY_ERROR';
      currentAttempt.errorMessage = error.message;
    }

    this.retryAttempts.set(payment.id, attempts);
    return currentAttempt;
  }

  /**
   * Process all pending retries
   */
  async processRetries(): Promise<void> {
    const now = new Date();

    for (const [paymentId, attempts] of this.retryAttempts) {
      const latestAttempt = attempts[attempts.length - 1];

      if (
        latestAttempt.status === PaymentStatus.PENDING &&
        latestAttempt.nextRetryAt &&
        isBefore(latestAttempt.nextRetryAt, now)
      ) {
        // This would fetch the payment, invoice, and payment method from storage
        // For now, we'll skip the actual retry execution
        console.log(`Processing retry for payment ${paymentId}`);
      }
    }
  }

  /**
   * Calculate next retry time based on attempt number
   */
  private calculateNextRetryTime(attemptNumber: number): Date {
    const now = new Date();
    const intervalIndex = Math.min(
      attemptNumber - 1,
      this.config.retryIntervalHours.length - 1
    );
    const intervalHours = this.config.retryIntervalHours[intervalIndex];

    return addHours(now, intervalHours);
  }

  /**
   * Handle max retries exceeded
   */
  private async handleMaxRetriesExceeded(
    payment: Payment,
    invoice: Invoice,
    subscription: Subscription
  ): Promise<void> {
    // Mark invoice as uncollectible
    invoice.status = 'uncollectible' as any;

    // Update subscription status
    subscription.status = SubscriptionStatus.PAST_DUE;

    await this.emitEvent(WebhookEvent.INVOICE_PAYMENT_FAILED, {
      payment,
      invoice,
      subscription,
      maxRetriesExceeded: true,
    });
  }

  /**
   * Get retry attempts for a payment
   */
  getRetryAttempts(paymentId: string): RetryAttempt[] {
    return this.retryAttempts.get(paymentId) || [];
  }

  /**
   * Get retry statistics
   */
  getRetryStatistics(paymentId: string): {
    totalAttempts: number;
    successfulAttempts: number;
    failedAttempts: number;
    pendingAttempts: number;
    lastAttempt?: RetryAttempt;
  } {
    const attempts = this.getRetryAttempts(paymentId);

    return {
      totalAttempts: attempts.length,
      successfulAttempts: attempts.filter((a) => a.status === PaymentStatus.SUCCEEDED)
        .length,
      failedAttempts: attempts.filter((a) => a.status === PaymentStatus.FAILED)
        .length,
      pendingAttempts: attempts.filter((a) => a.status === PaymentStatus.PENDING)
        .length,
      lastAttempt: attempts[attempts.length - 1],
    };
  }

  /**
   * Cancel pending retries
   */
  cancelRetries(paymentId: string): void {
    this.retryAttempts.delete(paymentId);
  }

  /**
   * Smart retry decision based on error code
   */
  shouldRetry(errorCode: string): {
    retry: boolean;
    reason: string;
  } {
    // Don't retry these errors
    const doNotRetry = [
      'card_declined',
      'insufficient_funds',
      'invalid_card',
      'expired_card',
      'incorrect_cvc',
      'fraudulent',
      'do_not_honor',
    ];

    if (doNotRetry.includes(errorCode)) {
      return {
        retry: false,
        reason: `Error code ${errorCode} is not retryable`,
      };
    }

    // Retry these errors
    const shouldRetryErrors = [
      'processing_error',
      'rate_limit',
      'temporary_failure',
      'timeout',
      'network_error',
    ];

    if (shouldRetryErrors.includes(errorCode)) {
      return {
        retry: true,
        reason: `Error code ${errorCode} is retryable`,
      };
    }

    // Default: retry
    return {
      retry: true,
      reason: 'Unknown error, attempting retry',
    };
  }

  /**
   * Get dunning email schedule
   */
  getDunningSchedule(attemptNumber: number): {
    sendEmail: boolean;
    emailType: 'payment_failed' | 'payment_retry' | 'final_notice';
    daysUntilSuspension: number;
  } {
    switch (attemptNumber) {
      case 1:
        return {
          sendEmail: true,
          emailType: 'payment_failed',
          daysUntilSuspension: 14,
        };
      case 2:
      case 3:
        return {
          sendEmail: true,
          emailType: 'payment_retry',
          daysUntilSuspension: 7,
        };
      default:
        return {
          sendEmail: true,
          emailType: 'final_notice',
          daysUntilSuspension: 3,
        };
    }
  }

  /**
   * Update payment method and retry immediately
   */
  async retryWithNewPaymentMethod(
    payment: Payment,
    invoice: Invoice,
    newPaymentMethodId: string
  ): Promise<RetryAttempt> {
    const intent: PaymentIntent = {
      amount: invoice.amountDue,
      currency: invoice.currency,
      paymentMethodId: newPaymentMethodId,
      customerId: payment.tenantId,
      description: `Invoice ${invoice.number} - Retry with updated payment method`,
      metadata: {
        invoiceId: invoice.id,
        paymentId: payment.id,
        paymentMethodUpdated: true,
      },
    };

    const result = await this.paymentGateway.processPayment(intent);

    const attempt: RetryAttempt = {
      attemptNumber: (this.retryAttempts.get(payment.id) || []).length + 1,
      attemptedAt: new Date(),
      status: result.success ? PaymentStatus.SUCCEEDED : PaymentStatus.FAILED,
      errorCode: result.errorCode,
      errorMessage: result.errorMessage,
    };

    const attempts = this.retryAttempts.get(payment.id) || [];
    attempts.push(attempt);
    this.retryAttempts.set(payment.id, attempts);

    return attempt;
  }

  /**
   * Register event handler
   */
  on(event: WebhookEvent, handler: (data: any) => void): void {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, []);
    }
    this.eventHandlers.get(event)!.push(handler);
  }

  /**
   * Emit event to registered handlers
   */
  private async emitEvent(event: WebhookEvent, data: any): Promise<void> {
    const handlers = this.eventHandlers.get(event) || [];
    for (const handler of handlers) {
      try {
        await handler(data);
      } catch (error) {
        console.error(`Error in event handler for ${event}:`, error);
      }
    }
  }
}
