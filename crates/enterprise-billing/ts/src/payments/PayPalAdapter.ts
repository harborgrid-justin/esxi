/**
 * PayPal Adapter - PayPal payment gateway implementation
 */

import paypal from '@paypal/checkout-server-sdk';
import { Decimal } from 'decimal.js';
import { v4 as uuidv4 } from 'uuid';
import {
  PaymentIntent,
  PaymentResult,
  RefundRequest,
  PaymentMethod,
  PaymentMethodType,
  PayPalDetails,
} from '../types';
import {
  PaymentGateway,
  CreateCustomerParams,
  CreatePaymentMethodParams,
  SetupIntentParams,
} from './PaymentGateway';

export class PayPalAdapter extends PaymentGateway {
  private client: paypal.core.PayPalHttpClient;

  constructor(clientId: string, clientSecret: string, sandbox: boolean = false) {
    super(clientId, sandbox);

    const environment = sandbox
      ? new paypal.core.SandboxEnvironment(clientId, clientSecret)
      : new paypal.core.LiveEnvironment(clientId, clientSecret);

    this.client = new paypal.core.PayPalHttpClient(environment);
  }

  /**
   * Create a customer (PayPal doesn't have a direct customer concept)
   */
  async createCustomer(params: CreateCustomerParams): Promise<string> {
    // PayPal doesn't have a customer object like Stripe
    // We'll return a generated ID and store the mapping
    return `paypal_customer_${params.tenant.id}`;
  }

  /**
   * Update customer information
   */
  async updateCustomer(
    customerId: string,
    updates: Partial<CreateCustomerParams>
  ): Promise<void> {
    // PayPal doesn't support customer updates in the same way
    // Customer info is managed through billing agreements
  }

  /**
   * Delete a customer
   */
  async deleteCustomer(customerId: string): Promise<void> {
    // No-op for PayPal
  }

  /**
   * Create a payment method (PayPal account)
   */
  async createPaymentMethod(
    params: CreatePaymentMethodParams
  ): Promise<PaymentMethod> {
    // PayPal payment methods are created through the checkout flow
    // This is a placeholder implementation
    return {
      id: uuidv4(),
      tenantId: '', // Should be set by caller
      type: PaymentMethodType.PAYPAL,
      isDefault: false,
      paypal: {
        email: params.billingDetails?.email || '',
        payerId: '', // Set during checkout
      },
      gatewayCustomerId: params.customerId,
      metadata: {},
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }

  /**
   * Attach payment method to customer
   */
  async attachPaymentMethod(
    paymentMethodId: string,
    customerId: string
  ): Promise<void> {
    // PayPal doesn't support attaching payment methods
  }

  /**
   * Detach payment method from customer
   */
  async detachPaymentMethod(paymentMethodId: string): Promise<void> {
    // PayPal doesn't support detaching payment methods
  }

  /**
   * Set default payment method
   */
  async setDefaultPaymentMethod(
    customerId: string,
    paymentMethodId: string
  ): Promise<void> {
    // Would be handled at the application level
  }

  /**
   * List payment methods for a customer
   */
  async listPaymentMethods(customerId: string): Promise<PaymentMethod[]> {
    // PayPal doesn't provide a way to list saved payment methods
    return [];
  }

  /**
   * Create a payment intent (order in PayPal)
   */
  async createPaymentIntent(intent: PaymentIntent): Promise<string> {
    const request = new paypal.orders.OrdersCreateRequest();
    request.prefer('return=representation');
    request.requestBody({
      intent: 'CAPTURE',
      purchase_units: [
        {
          amount: {
            currency_code: intent.currency.toUpperCase(),
            value: intent.amount.toFixed(2),
          },
          description: intent.description,
          custom_id: intent.customerId,
        },
      ],
      application_context: {
        brand_name: 'Your Company',
        landing_page: 'BILLING',
        user_action: 'PAY_NOW',
      },
    });

    const response = await this.client.execute(request);
    return response.result.id;
  }

  /**
   * Confirm a payment (not used in PayPal flow)
   */
  async confirmPayment(paymentIntentId: string): Promise<PaymentResult> {
    // PayPal uses capture instead of confirm
    return this.capturePayment(paymentIntentId);
  }

  /**
   * Capture an authorized payment
   */
  async capturePayment(orderId: string): Promise<PaymentResult> {
    try {
      const request = new paypal.orders.OrdersCaptureRequest(orderId);
      request.requestBody({});

      const response = await this.client.execute(request);
      const order = response.result;

      if (order.status === 'COMPLETED') {
        return {
          success: true,
          transactionId: order.id,
          metadata: {
            status: order.status,
            payerId: order.payer?.payer_id,
            captureId: order.purchase_units[0]?.payments?.captures?.[0]?.id,
          },
        };
      }

      return {
        success: false,
        errorCode: order.status,
        errorMessage: `Order status: ${order.status}`,
      };
    } catch (error: any) {
      return {
        success: false,
        errorCode: error.statusCode?.toString() || 'UNKNOWN',
        errorMessage: error.message,
      };
    }
  }

  /**
   * Process a payment
   */
  async processPayment(intent: PaymentIntent): Promise<PaymentResult> {
    try {
      // Create and immediately capture the order
      const orderId = await this.createPaymentIntent(intent);
      return await this.capturePayment(orderId);
    } catch (error: any) {
      return {
        success: false,
        errorCode: error.statusCode?.toString() || 'UNKNOWN',
        errorMessage: error.message,
      };
    }
  }

  /**
   * Refund a payment
   */
  async refundPayment(request: RefundRequest): Promise<PaymentResult> {
    try {
      // Get the capture ID from the order
      const orderRequest = new paypal.orders.OrdersGetRequest(request.paymentId);
      const orderResponse = await this.client.execute(orderRequest);
      const captureId =
        orderResponse.result.purchase_units[0]?.payments?.captures?.[0]?.id;

      if (!captureId) {
        throw new Error('No capture found for this order');
      }

      const refundRequest = new paypal.payments.CapturesRefundRequest(captureId);
      refundRequest.requestBody({
        amount: request.amount
          ? {
              value: request.amount.toFixed(2),
              currency_code: 'USD', // Should be dynamic
            }
          : undefined,
        note_to_payer: request.reason,
      });

      const response = await this.client.execute(refundRequest);
      const refund = response.result;

      return {
        success: true,
        transactionId: refund.id,
        metadata: {
          status: refund.status,
          amount: refund.amount?.value,
        },
      };
    } catch (error: any) {
      return {
        success: false,
        errorCode: error.statusCode?.toString() || 'UNKNOWN',
        errorMessage: error.message,
      };
    }
  }

  /**
   * Create a setup intent (billing agreement in PayPal)
   */
  async createSetupIntent(params: SetupIntentParams): Promise<string> {
    // Would create a billing agreement for recurring payments
    // This is a simplified implementation
    return `setup_${uuidv4()}`;
  }

  /**
   * Verify webhook signature
   */
  verifyWebhookSignature(
    payload: string | Buffer,
    signature: string,
    secret: string
  ): boolean {
    // PayPal webhook verification would go here
    // Requires additional PayPal SDK methods
    return true;
  }

  /**
   * Parse webhook event
   */
  parseWebhookEvent(payload: string | Buffer): any {
    return JSON.parse(payload.toString());
  }

  /**
   * Get payment details
   */
  async getPaymentDetails(orderId: string): Promise<any> {
    const request = new paypal.orders.OrdersGetRequest(orderId);
    const response = await this.client.execute(request);
    return response.result;
  }

  /**
   * Get customer details
   */
  async getCustomerDetails(customerId: string): Promise<any> {
    // PayPal doesn't have customer objects
    return { id: customerId };
  }

  /**
   * Create subscription (PayPal native subscriptions)
   */
  async createSubscription(planId: string, customerId: string): Promise<string> {
    const request = new paypal.subscriptions.SubscriptionsCreateRequest();
    request.requestBody({
      plan_id: planId,
      application_context: {
        brand_name: 'Your Company',
        locale: 'en-US',
        user_action: 'SUBSCRIBE_NOW',
      },
    });

    const response = await this.client.execute(request);
    return response.result.id;
  }

  /**
   * Cancel subscription
   */
  async cancelSubscription(subscriptionId: string, reason?: string): Promise<void> {
    const request = new paypal.subscriptions.SubscriptionsCancelRequest(
      subscriptionId
    );
    request.requestBody({
      reason: reason || 'Customer requested cancellation',
    });

    await this.client.execute(request);
  }
}
