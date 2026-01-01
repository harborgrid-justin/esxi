/**
 * Enterprise Billing System - Main Entry Point
 *
 * A comprehensive multi-tenant subscription and billing system with
 * payment processing, usage tracking, and revenue analytics.
 *
 * @packageDocumentation
 */

// Core Types
export * from './types';

// Billing Engine
export { SubscriptionEngine } from './engine/SubscriptionEngine';
export { PricingEngine } from './engine/PricingEngine';
export { InvoiceGenerator } from './engine/InvoiceGenerator';
export { UsageTracker } from './engine/UsageTracker';
export { ProrationEngine } from './engine/ProrationEngine';

// Payment Processing
export { PaymentGateway } from './payments/PaymentGateway';
export { StripeAdapter } from './payments/StripeAdapter';
export { PayPalAdapter } from './payments/PayPalAdapter';
export { WireTransfer } from './payments/WireTransfer';
export { PaymentRetry } from './payments/PaymentRetry';

// Tenant Management
export { TenantManager } from './tenant/TenantManager';
export { TenantIsolation } from './tenant/TenantIsolation';
export { TenantProvisioning } from './tenant/TenantProvisioning';
export { TenantMigrationManager } from './tenant/TenantMigration';

// Services
export { BillingService } from './services/BillingService';
export { QuotaService } from './services/QuotaService';
export { WebhookService } from './services/WebhookService';
export { RevenueAnalytics } from './services/RevenueAnalytics';

// React Components
export { PricingTable } from './components/PricingTable';
export { CheckoutFlow } from './components/CheckoutFlow';
export { BillingDashboard } from './components/BillingDashboard';
export { InvoiceList } from './components/InvoiceList';
export { PaymentMethodManager } from './components/PaymentMethodManager';
export { UsageMetrics } from './components/UsageMetrics';
export { SubscriptionManager } from './components/SubscriptionManager';

// Re-export types for convenience
export type {
  Tenant,
  Subscription,
  Plan,
  Invoice,
  Payment,
  PaymentMethod,
  UsageRecord,
  BillingCycle,
  Discount,
  QuotaUsage,
  UsageSummary,
  RevenueMetrics,
  ChurnMetrics,
} from './types';
