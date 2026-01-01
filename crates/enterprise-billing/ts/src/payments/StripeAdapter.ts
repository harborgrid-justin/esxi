/**
 * Stripe Adapter - Stripe payment gateway implementation
 */

import Stripe from 'stripe';
import { Decimal } from 'decimal.js';
import { v4 as uuidv4 } from 'uuid';
import {
  PaymentIntent,
  PaymentResult,
  RefundRequest,
  PaymentMethod,
  PaymentMethodType,
  CardDetails,
} from '../types';
import {
  PaymentGateway,
  CreateCustomerParams,
  CreatePaymentMethodParams,
  SetupIntentParams,
} from './PaymentGateway';

export class StripeAdapter extends PaymentGateway {
  private stripe: Stripe;
  private webhookSecret?: string;

  constructor(apiKey: string, webhookSecret?: string, sandbox: boolean = false) {
    super(apiKey, sandbox);
    this.stripe = new Stripe(apiKey, {
      apiVersion: '2024-11-20.acacia',
    });
    this.webhookSecret = webhookSecret;
  }

  /**
   * Create a customer in Stripe
   */
  async createCustomer(params: CreateCustomerParams): Promise<string> {
    const customer = await this.stripe.customers.create({
      email: params.email,
      name: params.tenant.organizationName,
      metadata: {
        tenantId: params.tenant.id,
        ...params.metadata,
      },
      address: params.tenant.billingAddress
        ? {
            line1: params.tenant.billingAddress.line1,
            line2: params.tenant.billingAddress.line2,
            city: params.tenant.billingAddress.city,
            state: params.tenant.billingAddress.state,
            postal_code: params.tenant.billingAddress.postalCode,
            country: params.tenant.billingAddress.country,
          }
        : undefined,
      tax_id_data: params.tenant.taxId
        ? [
            {
              type: 'eu_vat',
              value: params.tenant.taxId,
            },
          ]
        : undefined,
    });

    return customer.id;
  }

  /**
   * Update customer information
   */
  async updateCustomer(
    customerId: string,
    updates: Partial<CreateCustomerParams>
  ): Promise<void> {
    await this.stripe.customers.update(customerId, {
      email: updates.email,
      name: updates.tenant?.organizationName,
      metadata: updates.metadata,
      address: updates.tenant?.billingAddress
        ? {
            line1: updates.tenant.billingAddress.line1,
            line2: updates.tenant.billingAddress.line2,
            city: updates.tenant.billingAddress.city,
            state: updates.tenant.billingAddress.state,
            postal_code: updates.tenant.billingAddress.postalCode,
            country: updates.tenant.billingAddress.country,
          }
        : undefined,
    });
  }

  /**
   * Delete a customer
   */
  async deleteCustomer(customerId: string): Promise<void> {
    await this.stripe.customers.del(customerId);
  }

  /**
   * Create a payment method
   */
  async createPaymentMethod(
    params: CreatePaymentMethodParams
  ): Promise<PaymentMethod> {
    const stripePaymentMethod = await this.stripe.paymentMethods.create({
      type: this.mapPaymentMethodType(params.type),
      billing_details: params.billingDetails,
    });

    return this.mapStripePaymentMethod(stripePaymentMethod, params.customerId);
  }

  /**
   * Attach payment method to customer
   */
  async attachPaymentMethod(
    paymentMethodId: string,
    customerId: string
  ): Promise<void> {
    await this.stripe.paymentMethods.attach(paymentMethodId, {
      customer: customerId,
    });
  }

  /**
   * Detach payment method from customer
   */
  async detachPaymentMethod(paymentMethodId: string): Promise<void> {
    await this.stripe.paymentMethods.detach(paymentMethodId);
  }

  /**
   * Set default payment method
   */
  async setDefaultPaymentMethod(
    customerId: string,
    paymentMethodId: string
  ): Promise<void> {
    await this.stripe.customers.update(customerId, {
      invoice_settings: {
        default_payment_method: paymentMethodId,
      },
    });
  }

  /**
   * List payment methods for a customer
   */
  async listPaymentMethods(customerId: string): Promise<PaymentMethod[]> {
    const paymentMethods = await this.stripe.paymentMethods.list({
      customer: customerId,
      type: 'card',
    });

    return paymentMethods.data.map((pm) =>
      this.mapStripePaymentMethod(pm, customerId)
    );
  }

  /**
   * Create a payment intent
   */
  async createPaymentIntent(intent: PaymentIntent): Promise<string> {
    const paymentIntent = await this.stripe.paymentIntents.create({
      amount: this.formatAmount(intent.amount, intent.currency),
      currency: intent.currency.toLowerCase(),
      customer: intent.customerId,
      payment_method: intent.paymentMethodId,
      description: intent.description,
      metadata: intent.metadata,
      confirm: false,
    });

    return paymentIntent.id;
  }

  /**
   * Confirm a payment
   */
  async confirmPayment(paymentIntentId: string): Promise<PaymentResult> {
    try {
      const paymentIntent = await this.stripe.paymentIntents.confirm(paymentIntentId);

      if (paymentIntent.status === 'succeeded') {
        return {
          success: true,
          transactionId: paymentIntent.id,
          metadata: {
            status: paymentIntent.status,
            chargeId: paymentIntent.latest_charge,
          },
        };
      }

      return {
        success: false,
        errorCode: paymentIntent.status,
        errorMessage: `Payment status: ${paymentIntent.status}`,
      };
    } catch (error: any) {
      return {
        success: false,
        errorCode: error.code,
        errorMessage: error.message,
      };
    }
  }

  /**
   * Capture an authorized payment
   */
  async capturePayment(paymentIntentId: string): Promise<PaymentResult> {
    try {
      const paymentIntent = await this.stripe.paymentIntents.capture(paymentIntentId);

      return {
        success: true,
        transactionId: paymentIntent.id,
        metadata: {
          status: paymentIntent.status,
          chargeId: paymentIntent.latest_charge,
        },
      };
    } catch (error: any) {
      return {
        success: false,
        errorCode: error.code,
        errorMessage: error.message,
      };
    }
  }

  /**
   * Process a payment
   */
  async processPayment(intent: PaymentIntent): Promise<PaymentResult> {
    try {
      const paymentIntent = await this.stripe.paymentIntents.create({
        amount: this.formatAmount(intent.amount, intent.currency),
        currency: intent.currency.toLowerCase(),
        customer: intent.customerId,
        payment_method: intent.paymentMethodId,
        description: intent.description,
        metadata: intent.metadata,
        confirm: true,
      });

      if (paymentIntent.status === 'succeeded') {
        return {
          success: true,
          transactionId: paymentIntent.id,
          metadata: {
            status: paymentIntent.status,
            chargeId: paymentIntent.latest_charge,
          },
        };
      }

      return {
        success: false,
        errorCode: paymentIntent.status,
        errorMessage: `Payment status: ${paymentIntent.status}`,
      };
    } catch (error: any) {
      return {
        success: false,
        errorCode: error.code,
        errorMessage: error.message,
      };
    }
  }

  /**
   * Refund a payment
   */
  async refundPayment(request: RefundRequest): Promise<PaymentResult> {
    try {
      const paymentIntent = await this.stripe.paymentIntents.retrieve(
        request.paymentId
      );

      if (!paymentIntent.latest_charge) {
        throw new Error('No charge found for this payment');
      }

      const refund = await this.stripe.refunds.create({
        charge: paymentIntent.latest_charge as string,
        amount: request.amount
          ? this.formatAmount(request.amount, paymentIntent.currency)
          : undefined,
        reason: request.reason as Stripe.RefundCreateParams.Reason,
      });

      return {
        success: true,
        transactionId: refund.id,
        metadata: {
          status: refund.status,
          amount: refund.amount,
        },
      };
    } catch (error: any) {
      return {
        success: false,
        errorCode: error.code,
        errorMessage: error.message,
      };
    }
  }

  /**
   * Create a setup intent for saving payment method
   */
  async createSetupIntent(params: SetupIntentParams): Promise<string> {
    const setupIntent = await this.stripe.setupIntents.create({
      customer: params.customerId,
      payment_method_types: params.paymentMethodTypes as Stripe.SetupIntentCreateParams.PaymentMethodType[],
      metadata: params.metadata,
    });

    return setupIntent.client_secret!;
  }

  /**
   * Verify webhook signature
   */
  verifyWebhookSignature(
    payload: string | Buffer,
    signature: string,
    secret: string
  ): boolean {
    try {
      this.stripe.webhooks.constructEvent(payload, signature, secret);
      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * Parse webhook event
   */
  parseWebhookEvent(payload: string | Buffer): any {
    if (!this.webhookSecret) {
      throw new Error('Webhook secret not configured');
    }

    return JSON.parse(payload.toString());
  }

  /**
   * Get payment details
   */
  async getPaymentDetails(paymentId: string): Promise<any> {
    return await this.stripe.paymentIntents.retrieve(paymentId);
  }

  /**
   * Get customer details
   */
  async getCustomerDetails(customerId: string): Promise<any> {
    return await this.stripe.customers.retrieve(customerId);
  }

  /**
   * Map our payment method type to Stripe's
   */
  private mapPaymentMethodType(type: PaymentMethodType): string {
    switch (type) {
      case PaymentMethodType.CREDIT_CARD:
      case PaymentMethodType.DEBIT_CARD:
        return 'card';
      case PaymentMethodType.BANK_ACCOUNT:
      case PaymentMethodType.ACH:
        return 'us_bank_account';
      default:
        return 'card';
    }
  }

  /**
   * Map Stripe payment method to our format
   */
  private mapStripePaymentMethod(
    pm: Stripe.PaymentMethod,
    customerId: string
  ): PaymentMethod {
    const paymentMethod: PaymentMethod = {
      id: uuidv4(),
      tenantId: '', // Should be set by caller
      type: this.mapStripeTypeToOurs(pm.type),
      isDefault: false,
      gatewayCustomerId: customerId,
      gatewayPaymentMethodId: pm.id,
      metadata: {},
      createdAt: new Date(pm.created * 1000),
      updatedAt: new Date(),
    };

    if (pm.card) {
      paymentMethod.card = {
        brand: pm.card.brand,
        last4: pm.card.last4,
        expiryMonth: pm.card.exp_month,
        expiryYear: pm.card.exp_year,
        fingerprint: pm.card.fingerprint,
      };
    }

    if (pm.us_bank_account) {
      paymentMethod.bankAccount = {
        accountHolderName: pm.billing_details.name || '',
        accountType: pm.us_bank_account.account_type || '',
        bankName: pm.us_bank_account.bank_name || '',
        last4: pm.us_bank_account.last4,
        routingNumber: pm.us_bank_account.routing_number,
      };
    }

    return paymentMethod;
  }

  /**
   * Map Stripe type to our enum
   */
  private mapStripeTypeToOurs(type: string): PaymentMethodType {
    switch (type) {
      case 'card':
        return PaymentMethodType.CREDIT_CARD;
      case 'us_bank_account':
        return PaymentMethodType.ACH;
      default:
        return PaymentMethodType.CREDIT_CARD;
    }
  }
}
