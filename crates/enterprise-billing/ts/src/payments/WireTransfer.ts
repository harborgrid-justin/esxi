/**
 * Wire Transfer - Manual payment processing for enterprise customers
 */

import { Decimal } from 'decimal.js';
import { v4 as uuidv4 } from 'uuid';
import {
  PaymentIntent,
  PaymentResult,
  RefundRequest,
  PaymentMethod,
  PaymentMethodType,
  Invoice,
  Payment,
  PaymentStatus,
} from '../types';
import {
  PaymentGateway,
  CreateCustomerParams,
  CreatePaymentMethodParams,
  SetupIntentParams,
} from './PaymentGateway';

export interface WireTransferInstructions {
  bankName: string;
  accountName: string;
  accountNumber: string;
  routingNumber?: string;
  swiftCode?: string;
  iban?: string;
  referenceNumber: string;
  currency: string;
  amount: Decimal;
}

export interface WireTransferReconciliation {
  paymentId: string;
  invoiceId: string;
  referenceNumber: string;
  receivedAmount: Decimal;
  receivedDate: Date;
  bankTransactionId?: string;
  notes?: string;
}

export class WireTransfer extends PaymentGateway {
  private pendingPayments: Map<string, Payment> = new Map();
  private instructions: WireTransferInstructions;

  constructor(instructions: WireTransferInstructions) {
    super('wire_transfer', false);
    this.instructions = instructions;
  }

  /**
   * Create a customer (no-op for wire transfers)
   */
  async createCustomer(params: CreateCustomerParams): Promise<string> {
    return `wire_customer_${params.tenant.id}`;
  }

  /**
   * Update customer information
   */
  async updateCustomer(
    customerId: string,
    updates: Partial<CreateCustomerParams>
  ): Promise<void> {
    // No-op for wire transfers
  }

  /**
   * Delete a customer
   */
  async deleteCustomer(customerId: string): Promise<void> {
    // No-op for wire transfers
  }

  /**
   * Create a payment method (wire transfer details)
   */
  async createPaymentMethod(
    params: CreatePaymentMethodParams
  ): Promise<PaymentMethod> {
    return {
      id: uuidv4(),
      tenantId: '', // Should be set by caller
      type: PaymentMethodType.WIRE_TRANSFER,
      isDefault: false,
      metadata: {
        instructions: this.instructions,
      },
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
    // No-op for wire transfers
  }

  /**
   * Detach payment method from customer
   */
  async detachPaymentMethod(paymentMethodId: string): Promise<void> {
    // No-op for wire transfers
  }

  /**
   * Set default payment method
   */
  async setDefaultPaymentMethod(
    customerId: string,
    paymentMethodId: string
  ): Promise<void> {
    // No-op for wire transfers
  }

  /**
   * List payment methods for a customer
   */
  async listPaymentMethods(customerId: string): Promise<PaymentMethod[]> {
    return [];
  }

  /**
   * Create a payment intent (generate wire transfer instructions)
   */
  async createPaymentIntent(intent: PaymentIntent): Promise<string> {
    const referenceNumber = this.generateReferenceNumber(intent.customerId);

    const payment: Payment = {
      id: uuidv4(),
      tenantId: intent.customerId || '',
      amount: intent.amount,
      currency: intent.currency,
      status: PaymentStatus.PENDING,
      paymentMethodId: intent.paymentMethodId,
      refundedAmount: new Decimal(0),
      metadata: {
        ...intent.metadata,
        referenceNumber,
        instructions: {
          ...this.instructions,
          referenceNumber,
          amount: intent.amount,
          currency: intent.currency,
        },
      },
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    this.pendingPayments.set(payment.id, payment);
    return payment.id;
  }

  /**
   * Confirm a payment (manual reconciliation required)
   */
  async confirmPayment(paymentIntentId: string): Promise<PaymentResult> {
    const payment = this.pendingPayments.get(paymentIntentId);

    if (!payment) {
      return {
        success: false,
        errorCode: 'PAYMENT_NOT_FOUND',
        errorMessage: 'Payment intent not found',
      };
    }

    return {
      success: false,
      errorCode: 'MANUAL_CONFIRMATION_REQUIRED',
      errorMessage:
        'Wire transfer requires manual confirmation. Use reconcilePayment() when funds are received.',
      metadata: {
        instructions: payment.metadata.instructions,
      },
    };
  }

  /**
   * Capture an authorized payment
   */
  async capturePayment(paymentIntentId: string): Promise<PaymentResult> {
    return this.confirmPayment(paymentIntentId);
  }

  /**
   * Process a payment (creates pending payment awaiting reconciliation)
   */
  async processPayment(intent: PaymentIntent): Promise<PaymentResult> {
    const paymentId = await this.createPaymentIntent(intent);
    const payment = this.pendingPayments.get(paymentId)!;

    return {
      success: true,
      transactionId: paymentId,
      metadata: {
        status: 'PENDING_WIRE_TRANSFER',
        instructions: payment.metadata.instructions,
        message:
          'Payment instructions generated. Awaiting wire transfer and manual reconciliation.',
      },
    };
  }

  /**
   * Reconcile a wire transfer payment
   */
  async reconcilePayment(
    reconciliation: WireTransferReconciliation
  ): Promise<PaymentResult> {
    const payment = this.pendingPayments.get(reconciliation.paymentId);

    if (!payment) {
      return {
        success: false,
        errorCode: 'PAYMENT_NOT_FOUND',
        errorMessage: 'Payment not found',
      };
    }

    // Check if amounts match (allowing for small differences due to fees)
    const amountDifference = payment.amount.minus(reconciliation.receivedAmount).abs();
    const tolerance = payment.amount.times(0.01); // 1% tolerance

    if (amountDifference.greaterThan(tolerance)) {
      return {
        success: false,
        errorCode: 'AMOUNT_MISMATCH',
        errorMessage: `Received amount (${reconciliation.receivedAmount}) does not match expected amount (${payment.amount})`,
      };
    }

    payment.status = PaymentStatus.SUCCEEDED;
    payment.gatewayTransactionId = reconciliation.bankTransactionId;
    payment.metadata = {
      ...payment.metadata,
      reconciled: true,
      reconciledAt: new Date(),
      receivedAmount: reconciliation.receivedAmount,
      receivedDate: reconciliation.receivedDate,
      notes: reconciliation.notes,
    };
    payment.updatedAt = new Date();

    this.pendingPayments.set(payment.id, payment);

    return {
      success: true,
      transactionId: payment.id,
      metadata: {
        status: 'RECONCILED',
        receivedAmount: reconciliation.receivedAmount,
        receivedDate: reconciliation.receivedDate,
      },
    };
  }

  /**
   * Mark payment as failed (wire transfer not received)
   */
  async markPaymentFailed(
    paymentId: string,
    reason: string
  ): Promise<PaymentResult> {
    const payment = this.pendingPayments.get(paymentId);

    if (!payment) {
      return {
        success: false,
        errorCode: 'PAYMENT_NOT_FOUND',
        errorMessage: 'Payment not found',
      };
    }

    payment.status = PaymentStatus.FAILED;
    payment.failureCode = 'WIRE_TRANSFER_NOT_RECEIVED';
    payment.failureMessage = reason;
    payment.updatedAt = new Date();

    this.pendingPayments.set(paymentId, payment);

    return {
      success: true,
      transactionId: paymentId,
      metadata: {
        status: 'FAILED',
        reason,
      },
    };
  }

  /**
   * Refund a payment (manual process for wire transfers)
   */
  async refundPayment(request: RefundRequest): Promise<PaymentResult> {
    const payment = this.pendingPayments.get(request.paymentId);

    if (!payment) {
      return {
        success: false,
        errorCode: 'PAYMENT_NOT_FOUND',
        errorMessage: 'Payment not found',
      };
    }

    if (payment.status !== PaymentStatus.SUCCEEDED) {
      return {
        success: false,
        errorCode: 'INVALID_PAYMENT_STATUS',
        errorMessage: 'Can only refund successful payments',
      };
    }

    const refundAmount = request.amount ?? payment.amount;

    if (refundAmount.greaterThan(payment.amount.minus(payment.refundedAmount))) {
      return {
        success: false,
        errorCode: 'REFUND_AMOUNT_TOO_HIGH',
        errorMessage: 'Refund amount exceeds available amount',
      };
    }

    payment.refundedAmount = payment.refundedAmount.plus(refundAmount);

    if (payment.refundedAmount.equals(payment.amount)) {
      payment.status = PaymentStatus.REFUNDED;
    } else {
      payment.status = PaymentStatus.PARTIALLY_REFUNDED;
    }

    payment.metadata = {
      ...payment.metadata,
      refunds: [
        ...(payment.metadata.refunds || []),
        {
          amount: refundAmount,
          reason: request.reason,
          date: new Date(),
        },
      ],
    };
    payment.updatedAt = new Date();

    this.pendingPayments.set(payment.id, payment);

    return {
      success: true,
      transactionId: `refund_${uuidv4()}`,
      metadata: {
        status: 'REFUND_INITIATED',
        amount: refundAmount,
        message:
          'Refund initiated. Please process wire transfer refund manually.',
      },
    };
  }

  /**
   * Create a setup intent
   */
  async createSetupIntent(params: SetupIntentParams): Promise<string> {
    // Wire transfers don't require setup
    return `setup_wire_${uuidv4()}`;
  }

  /**
   * Verify webhook signature
   */
  verifyWebhookSignature(
    payload: string | Buffer,
    signature: string,
    secret: string
  ): boolean {
    // Wire transfers don't use webhooks
    return true;
  }

  /**
   * Parse webhook event
   */
  parseWebhookEvent(payload: string | Buffer): any {
    return {};
  }

  /**
   * Get payment details
   */
  async getPaymentDetails(paymentId: string): Promise<any> {
    return this.pendingPayments.get(paymentId);
  }

  /**
   * Get customer details
   */
  async getCustomerDetails(customerId: string): Promise<any> {
    return { id: customerId };
  }

  /**
   * Get wire transfer instructions for an invoice
   */
  getInstructions(invoice: Invoice): WireTransferInstructions {
    return {
      ...this.instructions,
      referenceNumber: this.generateReferenceNumber(invoice.tenantId),
      currency: invoice.currency,
      amount: invoice.total,
    };
  }

  /**
   * Generate unique reference number
   */
  private generateReferenceNumber(customerId: string): string {
    const timestamp = Date.now().toString(36).toUpperCase();
    const random = Math.random().toString(36).substring(2, 8).toUpperCase();
    return `WIRE-${timestamp}-${random}`;
  }

  /**
   * Get pending payments awaiting reconciliation
   */
  getPendingPayments(): Payment[] {
    return Array.from(this.pendingPayments.values()).filter(
      (p) => p.status === PaymentStatus.PENDING
    );
  }

  /**
   * Get payments by reference number
   */
  getPaymentByReference(referenceNumber: string): Payment | undefined {
    return Array.from(this.pendingPayments.values()).find(
      (p) => p.metadata.referenceNumber === referenceNumber
    );
  }
}
