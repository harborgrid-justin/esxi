/**
 * Enterprise Billing System - Core Type Definitions
 * Comprehensive types for multi-tenant subscription and billing
 */

import { Decimal } from 'decimal.js';

// ============================================================================
// Enumerations
// ============================================================================

export enum SubscriptionStatus {
  TRIAL = 'trial',
  ACTIVE = 'active',
  PAST_DUE = 'past_due',
  CANCELED = 'canceled',
  PAUSED = 'paused',
  EXPIRED = 'expired',
}

export enum PlanInterval {
  MONTHLY = 'monthly',
  QUARTERLY = 'quarterly',
  YEARLY = 'yearly',
  LIFETIME = 'lifetime',
}

export enum PricingModel {
  FLAT_RATE = 'flat_rate',
  PER_SEAT = 'per_seat',
  METERED = 'metered',
  TIERED = 'tiered',
  VOLUME = 'volume',
  HYBRID = 'hybrid',
}

export enum PaymentStatus {
  PENDING = 'pending',
  PROCESSING = 'processing',
  SUCCEEDED = 'succeeded',
  FAILED = 'failed',
  REFUNDED = 'refunded',
  PARTIALLY_REFUNDED = 'partially_refunded',
}

export enum PaymentMethodType {
  CREDIT_CARD = 'credit_card',
  DEBIT_CARD = 'debit_card',
  BANK_ACCOUNT = 'bank_account',
  PAYPAL = 'paypal',
  WIRE_TRANSFER = 'wire_transfer',
  ACH = 'ach',
}

export enum InvoiceStatus {
  DRAFT = 'draft',
  OPEN = 'open',
  PAID = 'paid',
  VOID = 'void',
  UNCOLLECTIBLE = 'uncollectible',
}

export enum TenantStatus {
  ACTIVE = 'active',
  SUSPENDED = 'suspended',
  TRIAL = 'trial',
  CHURNED = 'churned',
}

export enum DiscountType {
  PERCENTAGE = 'percentage',
  FIXED_AMOUNT = 'fixed_amount',
  FREE_TRIAL = 'free_trial',
}

export enum UsageAggregation {
  SUM = 'sum',
  MAX = 'max',
  LAST_DURING_PERIOD = 'last_during_period',
}

// ============================================================================
// Core Entities
// ============================================================================

export interface Tenant {
  id: string;
  organizationName: string;
  slug: string;
  status: TenantStatus;
  subscriptionId?: string;
  billingEmail: string;
  billingAddress?: Address;
  taxId?: string;
  currency: string;
  locale: string;
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
  deletedAt?: Date;
}

export interface Address {
  line1: string;
  line2?: string;
  city: string;
  state?: string;
  postalCode: string;
  country: string;
}

export interface Subscription {
  id: string;
  tenantId: string;
  planId: string;
  status: SubscriptionStatus;
  currentPeriodStart: Date;
  currentPeriodEnd: Date;
  trialStart?: Date;
  trialEnd?: Date;
  canceledAt?: Date;
  cancelAtPeriodEnd: boolean;
  quantity: number;
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface Plan {
  id: string;
  name: string;
  description: string;
  pricingModel: PricingModel;
  interval: PlanInterval;
  currency: string;
  amount: Decimal;
  trialDays?: number;
  features: PlanFeature[];
  tiers?: PricingTier[];
  meteredComponents?: MeteredComponent[];
  isActive: boolean;
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface PlanFeature {
  id: string;
  name: string;
  description: string;
  quota?: number;
  unlimited: boolean;
  enabled: boolean;
}

export interface PricingTier {
  id: string;
  upTo?: number; // undefined means infinity
  unitAmount: Decimal;
  flatAmount?: Decimal;
}

export interface MeteredComponent {
  id: string;
  name: string;
  unit: string;
  aggregation: UsageAggregation;
  tiers?: PricingTier[];
  unitAmount?: Decimal;
}

export interface Invoice {
  id: string;
  tenantId: string;
  subscriptionId?: string;
  number: string;
  status: InvoiceStatus;
  currency: string;
  subtotal: Decimal;
  tax: Decimal;
  total: Decimal;
  amountDue: Decimal;
  amountPaid: Decimal;
  lineItems: InvoiceLineItem[];
  discounts: InvoiceDiscount[];
  periodStart: Date;
  periodEnd: Date;
  dueDate: Date;
  paidAt?: Date;
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface InvoiceLineItem {
  id: string;
  description: string;
  quantity: number;
  unitAmount: Decimal;
  amount: Decimal;
  proration: boolean;
  periodStart?: Date;
  periodEnd?: Date;
  metadata: Record<string, any>;
}

export interface InvoiceDiscount {
  id: string;
  discountId: string;
  amount: Decimal;
  description: string;
}

export interface Payment {
  id: string;
  tenantId: string;
  invoiceId?: string;
  amount: Decimal;
  currency: string;
  status: PaymentStatus;
  paymentMethodId: string;
  gatewayTransactionId?: string;
  failureCode?: string;
  failureMessage?: string;
  refundedAmount: Decimal;
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface PaymentMethod {
  id: string;
  tenantId: string;
  type: PaymentMethodType;
  isDefault: boolean;
  card?: CardDetails;
  bankAccount?: BankAccountDetails;
  paypal?: PayPalDetails;
  gatewayCustomerId?: string;
  gatewayPaymentMethodId?: string;
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

export interface CardDetails {
  brand: string;
  last4: string;
  expiryMonth: number;
  expiryYear: number;
  fingerprint?: string;
}

export interface BankAccountDetails {
  accountHolderName: string;
  accountType: string;
  bankName: string;
  last4: string;
  routingNumber?: string;
}

export interface PayPalDetails {
  email: string;
  payerId: string;
}

export interface UsageRecord {
  id: string;
  tenantId: string;
  subscriptionId: string;
  meteredComponentId: string;
  quantity: number;
  timestamp: Date;
  idempotencyKey?: string;
  metadata: Record<string, any>;
}

export interface BillingCycle {
  id: string;
  tenantId: string;
  subscriptionId: string;
  periodStart: Date;
  periodEnd: Date;
  usageRecords: UsageRecord[];
  invoiceId?: string;
  status: 'open' | 'closed' | 'invoiced';
  createdAt: Date;
  updatedAt: Date;
}

export interface Discount {
  id: string;
  code: string;
  name: string;
  type: DiscountType;
  value: Decimal;
  duration: 'once' | 'repeating' | 'forever';
  durationInMonths?: number;
  maxRedemptions?: number;
  redemptionCount: number;
  validFrom: Date;
  validTo?: Date;
  appliesTo?: string[]; // plan IDs
  isActive: boolean;
  metadata: Record<string, any>;
  createdAt: Date;
  updatedAt: Date;
}

// ============================================================================
// Usage and Quotas
// ============================================================================

export interface QuotaUsage {
  tenantId: string;
  featureId: string;
  used: number;
  limit: number;
  unlimited: boolean;
  periodStart: Date;
  periodEnd: Date;
}

export interface UsageSummary {
  tenantId: string;
  subscriptionId: string;
  periodStart: Date;
  periodEnd: Date;
  components: ComponentUsage[];
  estimatedCost: Decimal;
}

export interface ComponentUsage {
  meteredComponentId: string;
  componentName: string;
  totalUsage: number;
  unit: string;
  cost: Decimal;
}

// ============================================================================
// Revenue Analytics
// ============================================================================

export interface RevenueMetrics {
  mrr: Decimal; // Monthly Recurring Revenue
  arr: Decimal; // Annual Recurring Revenue
  newMrr: Decimal;
  expansionMrr: Decimal;
  contractionMrr: Decimal;
  churnedMrr: Decimal;
  netMrr: Decimal;
  period: string;
  timestamp: Date;
}

export interface ChurnMetrics {
  customerChurnRate: number;
  revenueChurnRate: number;
  period: string;
  churnedCustomers: number;
  totalCustomers: number;
  churnedRevenue: Decimal;
  totalRevenue: Decimal;
}

export interface CohortAnalysis {
  cohort: string; // e.g., "2024-01"
  month: number;
  customersRetained: number;
  retentionRate: number;
  revenue: Decimal;
}

// ============================================================================
// Webhooks and Events
// ============================================================================

export enum WebhookEvent {
  SUBSCRIPTION_CREATED = 'subscription.created',
  SUBSCRIPTION_UPDATED = 'subscription.updated',
  SUBSCRIPTION_CANCELED = 'subscription.canceled',
  INVOICE_CREATED = 'invoice.created',
  INVOICE_PAID = 'invoice.paid',
  INVOICE_PAYMENT_FAILED = 'invoice.payment_failed',
  PAYMENT_SUCCEEDED = 'payment.succeeded',
  PAYMENT_FAILED = 'payment.failed',
  CUSTOMER_CREATED = 'customer.created',
  CUSTOMER_UPDATED = 'customer.updated',
}

export interface WebhookPayload {
  event: WebhookEvent;
  data: any;
  timestamp: Date;
  signature?: string;
}

// ============================================================================
// Payment Gateway Interfaces
// ============================================================================

export interface PaymentGatewayConfig {
  apiKey: string;
  apiSecret?: string;
  webhookSecret?: string;
  sandbox?: boolean;
}

export interface PaymentIntent {
  amount: Decimal;
  currency: string;
  paymentMethodId: string;
  customerId?: string;
  description?: string;
  metadata?: Record<string, any>;
}

export interface PaymentResult {
  success: boolean;
  transactionId?: string;
  errorCode?: string;
  errorMessage?: string;
  metadata?: Record<string, any>;
}

export interface RefundRequest {
  paymentId: string;
  amount?: Decimal; // undefined means full refund
  reason?: string;
}

// ============================================================================
// Proration and Plan Changes
// ============================================================================

export interface ProrationPreview {
  immediateCharge: Decimal;
  creditApplied: Decimal;
  nextInvoiceAmount: Decimal;
  proratedItems: ProrationItem[];
}

export interface ProrationItem {
  description: string;
  amount: Decimal;
  periodStart: Date;
  periodEnd: Date;
}

export interface PlanChangeRequest {
  tenantId: string;
  currentSubscriptionId: string;
  newPlanId: string;
  quantity?: number;
  prorate: boolean;
  effectiveDate?: Date;
}

// ============================================================================
// Tenant Provisioning
// ============================================================================

export interface ProvisioningConfig {
  tenantId: string;
  planId: string;
  resources: ResourceAllocation[];
  features: string[];
  isolationLevel: 'shared' | 'dedicated' | 'hybrid';
}

export interface ResourceAllocation {
  resourceType: string;
  quota: number;
  unit: string;
}

export interface TenantMigration {
  id: string;
  tenantId: string;
  fromPlanId: string;
  toPlanId: string;
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  startedAt: Date;
  completedAt?: Date;
  error?: string;
}

// ============================================================================
// Utility Types
// ============================================================================

export interface PaginationParams {
  page: number;
  limit: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface PaginatedResult<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
  totalPages: number;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: ApiError;
}

export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, any>;
}

// ============================================================================
// Configuration
// ============================================================================

export interface BillingConfig {
  defaultCurrency: string;
  taxRate: number;
  gracePeriodDays: number;
  retryAttempts: number;
  retryIntervalHours: number[];
  dunningEnabled: boolean;
  invoiceNumberPrefix: string;
  timeZone: string;
}

export interface StripeConfig extends PaymentGatewayConfig {
  publishableKey: string;
}

export interface PayPalConfig extends PaymentGatewayConfig {
  clientId: string;
  clientSecret: string;
  mode: 'sandbox' | 'live';
}
