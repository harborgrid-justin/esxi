/**
 * Invoice Generator - Creates and manages invoices
 */

import { Decimal } from 'decimal.js';
import { v4 as uuidv4 } from 'uuid';
import { format, addDays } from 'date-fns';
import {
  Invoice,
  InvoiceStatus,
  InvoiceLineItem,
  InvoiceDiscount,
  Subscription,
  Plan,
  Tenant,
  Discount,
  UsageRecord,
  BillingConfig,
  ComponentUsage,
} from '../types';
import { PricingEngine } from './PricingEngine';

export class InvoiceGenerator {
  private invoices: Map<string, Invoice> = new Map();
  private pricingEngine: PricingEngine;
  private invoiceCounter: number = 1000;

  constructor(
    private config: BillingConfig,
    pricingEngine?: PricingEngine
  ) {
    this.pricingEngine = pricingEngine ?? new PricingEngine();
  }

  /**
   * Generate invoice for subscription
   */
  async generateSubscriptionInvoice(
    tenant: Tenant,
    subscription: Subscription,
    plan: Plan,
    usageRecords?: UsageRecord[]
  ): Promise<Invoice> {
    const lineItems: InvoiceLineItem[] = [];

    // Add subscription line item
    const subscriptionAmount = this.pricingEngine.calculateSubscriptionPrice(
      plan,
      subscription.quantity,
      usageRecords
    );

    lineItems.push({
      id: uuidv4(),
      description: `${plan.name} - ${format(subscription.currentPeriodStart, 'MMM d, yyyy')} to ${format(subscription.currentPeriodEnd, 'MMM d, yyyy')}`,
      quantity: subscription.quantity,
      unitAmount: plan.amount,
      amount: subscriptionAmount,
      proration: false,
      periodStart: subscription.currentPeriodStart,
      periodEnd: subscription.currentPeriodEnd,
      metadata: {},
    });

    // Add metered usage line items
    if (usageRecords && plan.meteredComponents) {
      for (const component of plan.meteredComponents) {
        const componentUsage = this.pricingEngine.calculateComponentUsage(
          component,
          usageRecords
        );

        if (componentUsage.totalUsage > 0) {
          lineItems.push({
            id: uuidv4(),
            description: `${component.name} usage (${componentUsage.totalUsage} ${component.unit})`,
            quantity: componentUsage.totalUsage,
            unitAmount: component.unitAmount ?? new Decimal(0),
            amount: componentUsage.cost,
            proration: false,
            periodStart: subscription.currentPeriodStart,
            periodEnd: subscription.currentPeriodEnd,
            metadata: { componentId: component.id },
          });
        }
      }
    }

    const subtotal = lineItems.reduce((sum, item) => sum.plus(item.amount), new Decimal(0));
    const tax = this.pricingEngine.calculateTax(subtotal, this.config.taxRate);
    const total = subtotal.plus(tax);

    const invoice: Invoice = {
      id: uuidv4(),
      tenantId: tenant.id,
      subscriptionId: subscription.id,
      number: this.generateInvoiceNumber(),
      status: InvoiceStatus.OPEN,
      currency: tenant.currency,
      subtotal,
      tax,
      total,
      amountDue: total,
      amountPaid: new Decimal(0),
      lineItems,
      discounts: [],
      periodStart: subscription.currentPeriodStart,
      periodEnd: subscription.currentPeriodEnd,
      dueDate: addDays(new Date(), this.config.gracePeriodDays),
      metadata: {},
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    this.invoices.set(invoice.id, invoice);
    return invoice;
  }

  /**
   * Generate invoice with custom line items
   */
  async generateCustomInvoice(
    tenant: Tenant,
    lineItems: Omit<InvoiceLineItem, 'id'>[],
    dueDate?: Date
  ): Promise<Invoice> {
    const items: InvoiceLineItem[] = lineItems.map((item) => ({
      ...item,
      id: uuidv4(),
    }));

    const subtotal = items.reduce((sum, item) => sum.plus(item.amount), new Decimal(0));
    const tax = this.pricingEngine.calculateTax(subtotal, this.config.taxRate);
    const total = subtotal.plus(tax);

    const invoice: Invoice = {
      id: uuidv4(),
      tenantId: tenant.id,
      number: this.generateInvoiceNumber(),
      status: InvoiceStatus.OPEN,
      currency: tenant.currency,
      subtotal,
      tax,
      total,
      amountDue: total,
      amountPaid: new Decimal(0),
      lineItems: items,
      discounts: [],
      periodStart: new Date(),
      periodEnd: new Date(),
      dueDate: dueDate ?? addDays(new Date(), this.config.gracePeriodDays),
      metadata: {},
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    this.invoices.set(invoice.id, invoice);
    return invoice;
  }

  /**
   * Add discount to invoice
   */
  async applyDiscount(invoiceId: string, discount: Discount): Promise<Invoice> {
    const invoice = this.invoices.get(invoiceId);
    if (!invoice) {
      throw new Error(`Invoice ${invoiceId} not found`);
    }

    if (invoice.status !== InvoiceStatus.DRAFT && invoice.status !== InvoiceStatus.OPEN) {
      throw new Error('Can only apply discounts to draft or open invoices');
    }

    // Calculate discount amount
    const discountAmount = this.calculateDiscountAmount(invoice.subtotal, discount);

    const invoiceDiscount: InvoiceDiscount = {
      id: uuidv4(),
      discountId: discount.id,
      amount: discountAmount,
      description: discount.name,
    };

    invoice.discounts.push(invoiceDiscount);

    // Recalculate totals
    const totalDiscounts = invoice.discounts.reduce(
      (sum, d) => sum.plus(d.amount),
      new Decimal(0)
    );
    const discountedSubtotal = invoice.subtotal.minus(totalDiscounts);
    invoice.tax = this.pricingEngine.calculateTax(discountedSubtotal, this.config.taxRate);
    invoice.total = discountedSubtotal.plus(invoice.tax);
    invoice.amountDue = invoice.total.minus(invoice.amountPaid);
    invoice.updatedAt = new Date();

    this.invoices.set(invoiceId, invoice);
    return invoice;
  }

  /**
   * Mark invoice as paid
   */
  async markPaid(invoiceId: string, paymentAmount: Decimal): Promise<Invoice> {
    const invoice = this.invoices.get(invoiceId);
    if (!invoice) {
      throw new Error(`Invoice ${invoiceId} not found`);
    }

    invoice.amountPaid = invoice.amountPaid.plus(paymentAmount);
    invoice.amountDue = invoice.total.minus(invoice.amountPaid);

    if (invoice.amountDue.lessThanOrEqualTo(0)) {
      invoice.status = InvoiceStatus.PAID;
      invoice.paidAt = new Date();
    }

    invoice.updatedAt = new Date();
    this.invoices.set(invoiceId, invoice);

    return invoice;
  }

  /**
   * Void an invoice
   */
  async voidInvoice(invoiceId: string): Promise<Invoice> {
    const invoice = this.invoices.get(invoiceId);
    if (!invoice) {
      throw new Error(`Invoice ${invoiceId} not found`);
    }

    if (invoice.status === InvoiceStatus.PAID) {
      throw new Error('Cannot void a paid invoice');
    }

    invoice.status = InvoiceStatus.VOID;
    invoice.updatedAt = new Date();

    this.invoices.set(invoiceId, invoice);
    return invoice;
  }

  /**
   * Mark invoice as uncollectible
   */
  async markUncollectible(invoiceId: string): Promise<Invoice> {
    const invoice = this.invoices.get(invoiceId);
    if (!invoice) {
      throw new Error(`Invoice ${invoiceId} not found`);
    }

    invoice.status = InvoiceStatus.UNCOLLECTIBLE;
    invoice.updatedAt = new Date();

    this.invoices.set(invoiceId, invoice);
    return invoice;
  }

  /**
   * Get invoice by ID
   */
  getInvoice(invoiceId: string): Invoice | undefined {
    return this.invoices.get(invoiceId);
  }

  /**
   * Get all invoices for a tenant
   */
  getTenantInvoices(tenantId: string): Invoice[] {
    return Array.from(this.invoices.values())
      .filter((inv) => inv.tenantId === tenantId)
      .sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime());
  }

  /**
   * Get unpaid invoices for a tenant
   */
  getUnpaidInvoices(tenantId: string): Invoice[] {
    return Array.from(this.invoices.values()).filter(
      (inv) =>
        inv.tenantId === tenantId &&
        inv.status === InvoiceStatus.OPEN &&
        inv.amountDue.greaterThan(0)
    );
  }

  /**
   * Get overdue invoices
   */
  getOverdueInvoices(tenantId?: string): Invoice[] {
    const now = new Date();
    return Array.from(this.invoices.values()).filter(
      (inv) =>
        inv.status === InvoiceStatus.OPEN &&
        inv.dueDate < now &&
        inv.amountDue.greaterThan(0) &&
        (!tenantId || inv.tenantId === tenantId)
    );
  }

  /**
   * Add proration line item
   */
  addProrationLineItem(
    invoice: Invoice,
    description: string,
    amount: Decimal,
    periodStart: Date,
    periodEnd: Date
  ): Invoice {
    const lineItem: InvoiceLineItem = {
      id: uuidv4(),
      description,
      quantity: 1,
      unitAmount: amount,
      amount,
      proration: true,
      periodStart,
      periodEnd,
      metadata: {},
    };

    invoice.lineItems.push(lineItem);

    // Recalculate totals
    invoice.subtotal = invoice.lineItems.reduce(
      (sum, item) => sum.plus(item.amount),
      new Decimal(0)
    );

    const totalDiscounts = invoice.discounts.reduce(
      (sum, d) => sum.plus(d.amount),
      new Decimal(0)
    );
    const discountedSubtotal = invoice.subtotal.minus(totalDiscounts);
    invoice.tax = this.pricingEngine.calculateTax(discountedSubtotal, this.config.taxRate);
    invoice.total = discountedSubtotal.plus(invoice.tax);
    invoice.amountDue = invoice.total.minus(invoice.amountPaid);
    invoice.updatedAt = new Date();

    return invoice;
  }

  /**
   * Generate unique invoice number
   */
  private generateInvoiceNumber(): string {
    const number = this.invoiceCounter++;
    const year = new Date().getFullYear();
    return `${this.config.invoiceNumberPrefix}-${year}-${number.toString().padStart(6, '0')}`;
  }

  /**
   * Calculate discount amount
   */
  private calculateDiscountAmount(subtotal: Decimal, discount: Discount): Decimal {
    return subtotal.minus(this.pricingEngine.applyDiscount(subtotal, discount));
  }

  /**
   * Generate invoice preview (without saving)
   */
  async previewInvoice(
    tenant: Tenant,
    subscription: Subscription,
    plan: Plan,
    usageRecords?: UsageRecord[],
    discounts?: Discount[]
  ): Promise<Invoice> {
    let invoice = await this.generateSubscriptionInvoice(
      tenant,
      subscription,
      plan,
      usageRecords
    );

    // Apply discounts to preview
    if (discounts && discounts.length > 0) {
      for (const discount of discounts) {
        invoice = await this.applyDiscount(invoice.id, discount);
      }
    }

    // Remove from storage (it's just a preview)
    this.invoices.delete(invoice.id);

    return invoice;
  }
}
