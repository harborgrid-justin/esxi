/**
 * Payment Gateway - Abstract payment gateway interface
 */

import { Decimal } from 'decimal.js';
import {
  PaymentIntent,
  PaymentResult,
  RefundRequest,
  PaymentMethod,
  PaymentMethodType,
  Tenant,
} from '../types';

export interface CreateCustomerParams {
  tenant: Tenant;
  email: string;
  metadata?: Record<string, any>;
}

export interface CreatePaymentMethodParams {
  customerId: string;
  type: PaymentMethodType;
  token?: string;
  billingDetails?: {
    name?: string;
    email?: string;
    phone?: string;
    address?: {
      line1?: string;
      line2?: string;
      city?: string;
      state?: string;
      postalCode?: string;
      country?: string;
    };
  };
}

export interface SetupIntentParams {
  customerId: string;
  paymentMethodTypes: string[];
  metadata?: Record<string, any>;
}

export abstract class PaymentGateway {
  protected apiKey: string;
  protected sandbox: boolean;

  constructor(apiKey: string, sandbox: boolean = false) {
    this.apiKey = apiKey;
    this.sandbox = sandbox;
  }

  /**
   * Create a customer in the payment gateway
   */
  abstract createCustomer(params: CreateCustomerParams): Promise<string>;

  /**
   * Update customer information
   */
  abstract updateCustomer(
    customerId: string,
    updates: Partial<CreateCustomerParams>
  ): Promise<void>;

  /**
   * Delete a customer
   */
  abstract deleteCustomer(customerId: string): Promise<void>;

  /**
   * Create a payment method
   */
  abstract createPaymentMethod(
    params: CreatePaymentMethodParams
  ): Promise<PaymentMethod>;

  /**
   * Attach payment method to customer
   */
  abstract attachPaymentMethod(
    paymentMethodId: string,
    customerId: string
  ): Promise<void>;

  /**
   * Detach payment method from customer
   */
  abstract detachPaymentMethod(paymentMethodId: string): Promise<void>;

  /**
   * Set default payment method
   */
  abstract setDefaultPaymentMethod(
    customerId: string,
    paymentMethodId: string
  ): Promise<void>;

  /**
   * List payment methods for a customer
   */
  abstract listPaymentMethods(customerId: string): Promise<PaymentMethod[]>;

  /**
   * Create a payment intent
   */
  abstract createPaymentIntent(intent: PaymentIntent): Promise<string>;

  /**
   * Confirm a payment
   */
  abstract confirmPayment(paymentIntentId: string): Promise<PaymentResult>;

  /**
   * Capture an authorized payment
   */
  abstract capturePayment(paymentIntentId: string): Promise<PaymentResult>;

  /**
   * Process a payment
   */
  abstract processPayment(intent: PaymentIntent): Promise<PaymentResult>;

  /**
   * Refund a payment
   */
  abstract refundPayment(request: RefundRequest): Promise<PaymentResult>;

  /**
   * Create a setup intent for saving payment method
   */
  abstract createSetupIntent(params: SetupIntentParams): Promise<string>;

  /**
   * Verify webhook signature
   */
  abstract verifyWebhookSignature(
    payload: string | Buffer,
    signature: string,
    secret: string
  ): boolean;

  /**
   * Parse webhook event
   */
  abstract parseWebhookEvent(payload: string | Buffer): any;

  /**
   * Get payment details
   */
  abstract getPaymentDetails(paymentId: string): Promise<any>;

  /**
   * Get customer details
   */
  abstract getCustomerDetails(customerId: string): Promise<any>;

  /**
   * Validate payment method
   */
  protected validatePaymentMethod(type: PaymentMethodType): void {
    const validTypes = Object.values(PaymentMethodType);
    if (!validTypes.includes(type)) {
      throw new Error(`Invalid payment method type: ${type}`);
    }
  }

  /**
   * Format amount for gateway (most gateways use cents)
   */
  protected formatAmount(amount: Decimal, currency: string): number {
    // Zero-decimal currencies (JPY, KRW, etc.)
    const zeroDecimalCurrencies = ['JPY', 'KRW', 'VND', 'CLP'];

    if (zeroDecimalCurrencies.includes(currency.toUpperCase())) {
      return amount.toNumber();
    }

    // Most currencies use 2 decimal places
    return amount.times(100).toNumber();
  }

  /**
   * Parse amount from gateway format
   */
  protected parseAmount(amount: number, currency: string): Decimal {
    const zeroDecimalCurrencies = ['JPY', 'KRW', 'VND', 'CLP'];

    if (zeroDecimalCurrencies.includes(currency.toUpperCase())) {
      return new Decimal(amount);
    }

    return new Decimal(amount).dividedBy(100);
  }
}
