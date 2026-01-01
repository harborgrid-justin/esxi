/**
 * Subscription Engine - Manages subscription lifecycle
 */

import { Decimal } from 'decimal.js';
import { v4 as uuidv4 } from 'uuid';
import { addDays, addMonths, addYears, isBefore, isAfter } from 'date-fns';
import {
  Subscription,
  SubscriptionStatus,
  Plan,
  Tenant,
  PlanInterval,
  Invoice,
  WebhookEvent,
  PlanChangeRequest,
} from '../types';

export class SubscriptionEngine {
  private subscriptions: Map<string, Subscription> = new Map();
  private eventHandlers: Map<WebhookEvent, ((data: any) => void)[]> = new Map();

  /**
   * Create a new subscription for a tenant
   */
  async createSubscription(
    tenant: Tenant,
    plan: Plan,
    options?: {
      quantity?: number;
      trialDays?: number;
      metadata?: Record<string, any>;
    }
  ): Promise<Subscription> {
    const now = new Date();
    const trialDays = options?.trialDays ?? plan.trialDays ?? 0;
    const quantity = options?.quantity ?? 1;

    const trialStart = trialDays > 0 ? now : undefined;
    const trialEnd = trialDays > 0 ? addDays(now, trialDays) : undefined;

    const subscription: Subscription = {
      id: uuidv4(),
      tenantId: tenant.id,
      planId: plan.id,
      status: trialDays > 0 ? SubscriptionStatus.TRIAL : SubscriptionStatus.ACTIVE,
      currentPeriodStart: trialEnd ?? now,
      currentPeriodEnd: this.calculatePeriodEnd(trialEnd ?? now, plan.interval),
      trialStart,
      trialEnd,
      cancelAtPeriodEnd: false,
      quantity,
      metadata: options?.metadata ?? {},
      createdAt: now,
      updatedAt: now,
    };

    this.subscriptions.set(subscription.id, subscription);
    await this.emitEvent(WebhookEvent.SUBSCRIPTION_CREATED, subscription);

    return subscription;
  }

  /**
   * Update subscription quantity (for per-seat pricing)
   */
  async updateQuantity(subscriptionId: string, newQuantity: number): Promise<Subscription> {
    const subscription = this.subscriptions.get(subscriptionId);
    if (!subscription) {
      throw new Error(`Subscription ${subscriptionId} not found`);
    }

    subscription.quantity = newQuantity;
    subscription.updatedAt = new Date();

    this.subscriptions.set(subscriptionId, subscription);
    await this.emitEvent(WebhookEvent.SUBSCRIPTION_UPDATED, subscription);

    return subscription;
  }

  /**
   * Cancel subscription
   */
  async cancelSubscription(
    subscriptionId: string,
    cancelAtPeriodEnd: boolean = true
  ): Promise<Subscription> {
    const subscription = this.subscriptions.get(subscriptionId);
    if (!subscription) {
      throw new Error(`Subscription ${subscriptionId} not found`);
    }

    const now = new Date();

    if (cancelAtPeriodEnd) {
      subscription.cancelAtPeriodEnd = true;
      subscription.status = SubscriptionStatus.ACTIVE;
    } else {
      subscription.status = SubscriptionStatus.CANCELED;
      subscription.canceledAt = now;
    }

    subscription.updatedAt = now;
    this.subscriptions.set(subscriptionId, subscription);
    await this.emitEvent(WebhookEvent.SUBSCRIPTION_CANCELED, subscription);

    return subscription;
  }

  /**
   * Reactivate a canceled subscription
   */
  async reactivateSubscription(subscriptionId: string): Promise<Subscription> {
    const subscription = this.subscriptions.get(subscriptionId);
    if (!subscription) {
      throw new Error(`Subscription ${subscriptionId} not found`);
    }

    if (subscription.status !== SubscriptionStatus.CANCELED) {
      throw new Error('Can only reactivate canceled subscriptions');
    }

    subscription.status = SubscriptionStatus.ACTIVE;
    subscription.cancelAtPeriodEnd = false;
    subscription.canceledAt = undefined;
    subscription.updatedAt = new Date();

    this.subscriptions.set(subscriptionId, subscription);
    await this.emitEvent(WebhookEvent.SUBSCRIPTION_UPDATED, subscription);

    return subscription;
  }

  /**
   * Pause subscription (useful for seasonal businesses)
   */
  async pauseSubscription(subscriptionId: string): Promise<Subscription> {
    const subscription = this.subscriptions.get(subscriptionId);
    if (!subscription) {
      throw new Error(`Subscription ${subscriptionId} not found`);
    }

    subscription.status = SubscriptionStatus.PAUSED;
    subscription.updatedAt = new Date();

    this.subscriptions.set(subscriptionId, subscription);
    await this.emitEvent(WebhookEvent.SUBSCRIPTION_UPDATED, subscription);

    return subscription;
  }

  /**
   * Resume a paused subscription
   */
  async resumeSubscription(subscriptionId: string): Promise<Subscription> {
    const subscription = this.subscriptions.get(subscriptionId);
    if (!subscription) {
      throw new Error(`Subscription ${subscriptionId} not found`);
    }

    if (subscription.status !== SubscriptionStatus.PAUSED) {
      throw new Error('Can only resume paused subscriptions');
    }

    subscription.status = SubscriptionStatus.ACTIVE;
    subscription.updatedAt = new Date();

    this.subscriptions.set(subscriptionId, subscription);
    await this.emitEvent(WebhookEvent.SUBSCRIPTION_UPDATED, subscription);

    return subscription;
  }

  /**
   * Renew subscription for next billing period
   */
  async renewSubscription(subscriptionId: string): Promise<Subscription> {
    const subscription = this.subscriptions.get(subscriptionId);
    if (!subscription) {
      throw new Error(`Subscription ${subscriptionId} not found`);
    }

    const plan = await this.getPlan(subscription.planId);

    subscription.currentPeriodStart = subscription.currentPeriodEnd;
    subscription.currentPeriodEnd = this.calculatePeriodEnd(
      subscription.currentPeriodEnd,
      plan.interval
    );
    subscription.updatedAt = new Date();

    // Check if subscription should be canceled
    if (subscription.cancelAtPeriodEnd) {
      subscription.status = SubscriptionStatus.CANCELED;
      subscription.canceledAt = new Date();
    }

    this.subscriptions.set(subscriptionId, subscription);
    await this.emitEvent(WebhookEvent.SUBSCRIPTION_UPDATED, subscription);

    return subscription;
  }

  /**
   * Check and update subscription statuses (run periodically)
   */
  async processSubscriptions(): Promise<void> {
    const now = new Date();

    for (const [id, subscription] of this.subscriptions) {
      let updated = false;

      // Check if trial has ended
      if (
        subscription.status === SubscriptionStatus.TRIAL &&
        subscription.trialEnd &&
        isBefore(subscription.trialEnd, now)
      ) {
        subscription.status = SubscriptionStatus.ACTIVE;
        updated = true;
      }

      // Check if subscription period has ended
      if (
        subscription.status === SubscriptionStatus.ACTIVE &&
        isBefore(subscription.currentPeriodEnd, now)
      ) {
        await this.renewSubscription(id);
        updated = true;
      }

      // Check if past due subscription should expire
      if (
        subscription.status === SubscriptionStatus.PAST_DUE &&
        isBefore(addDays(subscription.currentPeriodEnd, 30), now)
      ) {
        subscription.status = SubscriptionStatus.EXPIRED;
        updated = true;
      }

      if (updated) {
        subscription.updatedAt = now;
        this.subscriptions.set(id, subscription);
        await this.emitEvent(WebhookEvent.SUBSCRIPTION_UPDATED, subscription);
      }
    }
  }

  /**
   * Mark subscription as past due (after failed payment)
   */
  async markPastDue(subscriptionId: string): Promise<Subscription> {
    const subscription = this.subscriptions.get(subscriptionId);
    if (!subscription) {
      throw new Error(`Subscription ${subscriptionId} not found`);
    }

    subscription.status = SubscriptionStatus.PAST_DUE;
    subscription.updatedAt = new Date();

    this.subscriptions.set(subscriptionId, subscription);
    await this.emitEvent(WebhookEvent.SUBSCRIPTION_UPDATED, subscription);

    return subscription;
  }

  /**
   * Get subscription by ID
   */
  getSubscription(subscriptionId: string): Subscription | undefined {
    return this.subscriptions.get(subscriptionId);
  }

  /**
   * Get all subscriptions for a tenant
   */
  getTenantSubscriptions(tenantId: string): Subscription[] {
    return Array.from(this.subscriptions.values()).filter(
      (sub) => sub.tenantId === tenantId
    );
  }

  /**
   * Get active subscription for a tenant
   */
  getActiveSubscription(tenantId: string): Subscription | undefined {
    return Array.from(this.subscriptions.values()).find(
      (sub) =>
        sub.tenantId === tenantId &&
        (sub.status === SubscriptionStatus.ACTIVE ||
          sub.status === SubscriptionStatus.TRIAL)
    );
  }

  /**
   * Calculate period end date based on interval
   */
  private calculatePeriodEnd(start: Date, interval: PlanInterval): Date {
    switch (interval) {
      case PlanInterval.MONTHLY:
        return addMonths(start, 1);
      case PlanInterval.QUARTERLY:
        return addMonths(start, 3);
      case PlanInterval.YEARLY:
        return addYears(start, 1);
      case PlanInterval.LIFETIME:
        return addYears(start, 100); // Effectively forever
      default:
        throw new Error(`Unknown interval: ${interval}`);
    }
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

  /**
   * Mock method to get plan (should be replaced with actual data source)
   */
  private async getPlan(planId: string): Promise<Plan> {
    // This would typically fetch from a database
    throw new Error('getPlan must be implemented with actual data source');
  }
}
