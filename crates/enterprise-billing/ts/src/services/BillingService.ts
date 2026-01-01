/**
 * Billing Service - Core billing operations orchestration
 */

import { Decimal } from 'decimal.js';
import {
  Tenant,
  Subscription,
  Plan,
  Invoice,
  Payment,
  UsageRecord,
  BillingConfig,
  PlanChangeRequest,
  ProrationPreview,
} from '../types';
import { SubscriptionEngine } from '../engine/SubscriptionEngine';
import { InvoiceGenerator } from '../engine/InvoiceGenerator';
import { PricingEngine } from '../engine/PricingEngine';
import { ProrationEngine } from '../engine/ProrationEngine';
import { UsageTracker } from '../engine/UsageTracker';
import { PaymentGateway } from '../payments/PaymentGateway';

export class BillingService {
  private subscriptionEngine: SubscriptionEngine;
  private invoiceGenerator: InvoiceGenerator;
  private pricingEngine: PricingEngine;
  private prorationEngine: ProrationEngine;
  private usageTracker: UsageTracker;

  constructor(
    private config: BillingConfig,
    private paymentGateway: PaymentGateway
  ) {
    this.subscriptionEngine = new SubscriptionEngine();
    this.pricingEngine = new PricingEngine();
    this.prorationEngine = new ProrationEngine();
    this.usageTracker = new UsageTracker(this.pricingEngine);
    this.invoiceGenerator = new InvoiceGenerator(config, this.pricingEngine);
  }

  /**
   * Create new subscription and initial invoice
   */
  async createSubscription(
    tenant: Tenant,
    plan: Plan,
    paymentMethodId?: string,
    options?: {
      quantity?: number;
      trialDays?: number;
      metadata?: Record<string, any>;
    }
  ): Promise<{ subscription: Subscription; invoice?: Invoice }> {
    // Create subscription
    const subscription = await this.subscriptionEngine.createSubscription(
      tenant,
      plan,
      options
    );

    // Generate invoice if not in trial or trial requires payment method
    let invoice: Invoice | undefined;

    const inTrial = subscription.trialEnd && new Date() < subscription.trialEnd;

    if (!inTrial) {
      invoice = await this.invoiceGenerator.generateSubscriptionInvoice(
        tenant,
        subscription,
        plan
      );

      // Process payment if payment method provided
      if (paymentMethodId && invoice) {
        await this.processInvoicePayment(tenant, invoice, paymentMethodId);
      }
    }

    return { subscription, invoice };
  }

  /**
   * Process recurring billing for a subscription
   */
  async processRecurringBilling(
    tenant: Tenant,
    subscription: Subscription,
    plan: Plan
  ): Promise<Invoice> {
    // Collect usage records if metered billing
    let usageRecords: UsageRecord[] | undefined;

    if (plan.meteredComponents && plan.meteredComponents.length > 0) {
      usageRecords = this.usageTracker.getUsageRecords(
        subscription.id,
        subscription.currentPeriodStart,
        subscription.currentPeriodEnd
      );
    }

    // Generate invoice
    const invoice = await this.invoiceGenerator.generateSubscriptionInvoice(
      tenant,
      subscription,
      plan,
      usageRecords
    );

    // Attempt to charge default payment method
    // In production, this would fetch the default payment method
    // await this.processInvoicePayment(tenant, invoice, defaultPaymentMethodId);

    // Renew subscription period
    await this.subscriptionEngine.renewSubscription(subscription.id);

    return invoice;
  }

  /**
   * Process invoice payment
   */
  async processInvoicePayment(
    tenant: Tenant,
    invoice: Invoice,
    paymentMethodId: string
  ): Promise<Payment> {
    const result = await this.paymentGateway.processPayment({
      amount: invoice.amountDue,
      currency: invoice.currency,
      paymentMethodId,
      customerId: tenant.id,
      description: `Invoice ${invoice.number}`,
      metadata: {
        invoiceId: invoice.id,
        tenantId: tenant.id,
      },
    });

    const payment: Payment = {
      id: result.transactionId || '',
      tenantId: tenant.id,
      invoiceId: invoice.id,
      amount: invoice.amountDue,
      currency: invoice.currency,
      status: result.success ? 'succeeded' : 'failed',
      paymentMethodId,
      gatewayTransactionId: result.transactionId,
      failureCode: result.errorCode,
      failureMessage: result.errorMessage,
      refundedAmount: new Decimal(0),
      metadata: result.metadata || {},
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    if (result.success) {
      await this.invoiceGenerator.markPaid(invoice.id, invoice.amountDue);
    }

    return payment;
  }

  /**
   * Change subscription plan
   */
  async changePlan(
    request: PlanChangeRequest,
    currentPlan: Plan,
    newPlan: Plan,
    subscription: Subscription,
    tenant: Tenant
  ): Promise<{
    subscription: Subscription;
    prorationInvoice?: Invoice;
  }> {
    // Calculate proration
    const prorationPreview = this.prorationEngine.calculatePlanChangeProration(
      request,
      currentPlan,
      newPlan,
      subscription
    );

    // Update subscription
    subscription.planId = newPlan.id;
    if (request.quantity) {
      subscription.quantity = request.quantity;
    }

    // Generate proration invoice if immediate charge
    let prorationInvoice: Invoice | undefined;

    if (
      request.prorate &&
      prorationPreview.immediateCharge.greaterThan(0)
    ) {
      prorationInvoice = await this.invoiceGenerator.generateCustomInvoice(
        tenant,
        prorationPreview.proratedItems.map((item) => ({
          description: item.description,
          quantity: 1,
          unitAmount: item.amount,
          amount: item.amount,
          proration: true,
          periodStart: item.periodStart,
          periodEnd: item.periodEnd,
          metadata: {},
        }))
      );
    }

    return { subscription, prorationInvoice };
  }

  /**
   * Update subscription quantity
   */
  async updateQuantity(
    subscription: Subscription,
    plan: Plan,
    tenant: Tenant,
    newQuantity: number
  ): Promise<{
    subscription: Subscription;
    prorationInvoice?: Invoice;
  }> {
    const prorationPreview = this.prorationEngine.calculateQuantityChangeProration(
      plan,
      subscription,
      newQuantity
    );

    const updatedSubscription = await this.subscriptionEngine.updateQuantity(
      subscription.id,
      newQuantity
    );

    let prorationInvoice: Invoice | undefined;

    if (prorationPreview.immediateCharge.greaterThan(0)) {
      prorationInvoice = await this.invoiceGenerator.generateCustomInvoice(
        tenant,
        prorationPreview.proratedItems.map((item) => ({
          description: item.description,
          quantity: 1,
          unitAmount: item.amount,
          amount: item.amount,
          proration: true,
          periodStart: item.periodStart,
          periodEnd: item.periodEnd,
          metadata: {},
        }))
      );
    }

    return { subscription: updatedSubscription, prorationInvoice };
  }

  /**
   * Cancel subscription
   */
  async cancelSubscription(
    subscriptionId: string,
    cancelAtPeriodEnd: boolean = true
  ): Promise<Subscription> {
    return await this.subscriptionEngine.cancelSubscription(
      subscriptionId,
      cancelAtPeriodEnd
    );
  }

  /**
   * Get proration preview for plan change
   */
  getProrationPreview(
    request: PlanChangeRequest,
    currentPlan: Plan,
    newPlan: Plan,
    subscription: Subscription
  ): ProrationPreview {
    return this.prorationEngine.calculatePlanChangeProration(
      request,
      currentPlan,
      newPlan,
      subscription
    );
  }

  /**
   * Record usage for metered billing
   */
  async recordUsage(
    tenantId: string,
    subscriptionId: string,
    meteredComponentId: string,
    quantity: number,
    idempotencyKey?: string
  ): Promise<UsageRecord> {
    return await this.usageTracker.recordUsage(
      tenantId,
      subscriptionId,
      meteredComponentId,
      quantity,
      idempotencyKey
    );
  }

  /**
   * Get upcoming invoice preview
   */
  async previewUpcomingInvoice(
    tenant: Tenant,
    subscription: Subscription,
    plan: Plan
  ): Promise<Invoice> {
    const usageRecords = plan.meteredComponents
      ? this.usageTracker.getUsageRecords(
          subscription.id,
          subscription.currentPeriodStart,
          subscription.currentPeriodEnd
        )
      : undefined;

    return await this.invoiceGenerator.previewInvoice(
      tenant,
      subscription,
      plan,
      usageRecords
    );
  }

  /**
   * Process all due subscriptions (run periodically)
   */
  async processDueSubscriptions(): Promise<void> {
    await this.subscriptionEngine.processSubscriptions();
  }
}
