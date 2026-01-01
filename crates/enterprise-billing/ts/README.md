# Enterprise Billing System

A comprehensive, production-ready TypeScript library for multi-tenant subscription and billing management.

## Features

### Core Billing Engine
- **Subscription Management**: Complete lifecycle management (create, update, cancel, pause, resume)
- **Dynamic Pricing**: Support for flat-rate, per-seat, metered, tiered, and volume pricing models
- **Invoice Generation**: Automated invoice creation with line items, discounts, and tax calculation
- **Usage Tracking**: Metered billing with idempotency support and multiple aggregation methods
- **Proration Engine**: Intelligent proration for plan changes and quantity updates

### Payment Processing
- **Multi-Gateway Support**: Abstract payment gateway with Stripe, PayPal, and wire transfer adapters
- **Payment Retry Logic**: Smart retry with exponential backoff and dunning management
- **Payment Methods**: Support for cards, bank accounts, ACH, and PayPal
- **Refunds**: Full and partial refund support

### Tenant Management
- **Tenant Isolation**: Row-level, schema-level, and database-level isolation strategies
- **Auto-Provisioning**: Automated tenant setup with resource allocation
- **Plan Migration**: Seamless migration between plans with data migration support
- **Custom Quotas**: Feature quotas with usage tracking and enforcement

### React Components
- **PricingTable**: Beautiful plan comparison UI
- **CheckoutFlow**: Multi-step checkout with payment integration
- **BillingDashboard**: Overview of subscription and billing status
- **InvoiceList**: Invoice history with filtering and actions
- **PaymentMethodManager**: Manage payment methods
- **UsageMetrics**: Visualize usage and quotas
- **SubscriptionManager**: Subscription settings and plan changes

### Services
- **BillingService**: Orchestrates billing operations
- **QuotaService**: Feature quota management and enforcement
- **WebhookService**: Event-driven webhooks with retries
- **RevenueAnalytics**: MRR, ARR, churn, cohort analysis, and more

## Installation

```bash
npm install @harborgrid/enterprise-billing
```

## Quick Start

```typescript
import {
  BillingService,
  StripeAdapter,
  TenantManager,
  QuotaService,
} from '@harborgrid/enterprise-billing';

// Initialize payment gateway
const stripe = new StripeAdapter(process.env.STRIPE_SECRET_KEY!);

// Create billing service
const billingService = new BillingService(
  {
    defaultCurrency: 'USD',
    taxRate: 0.08,
    gracePeriodDays: 7,
    retryAttempts: 3,
    retryIntervalHours: [1, 6, 24],
    dunningEnabled: true,
    invoiceNumberPrefix: 'INV',
    timeZone: 'America/New_York',
  },
  stripe
);

// Create tenant
const tenantManager = new TenantManager();
const tenant = await tenantManager.createTenant({
  organizationName: 'Acme Corp',
  slug: 'acme-corp',
  billingEmail: 'billing@acme.com',
  currency: 'USD',
});

// Create subscription
const { subscription, invoice } = await billingService.createSubscription(
  tenant,
  plan,
  paymentMethodId,
  { quantity: 5 }
);

// Track usage for metered billing
await billingService.recordUsage(
  tenant.id,
  subscription.id,
  'api-calls',
  1000,
  'idempotency-key-123'
);

// Enforce quotas
const quotaService = new QuotaService();
const check = await quotaService.checkQuota(tenant.id, 'users', plan, 1);

if (check.allowed) {
  await quotaService.incrementUsage(tenant.id, 'users', 1);
}
```

## Architecture

### Pricing Models

1. **Flat Rate**: Fixed price per billing period
2. **Per Seat**: Price per user/seat
3. **Metered**: Usage-based pricing
4. **Tiered**: Graduated pricing tiers
5. **Volume**: All units at tier rate
6. **Hybrid**: Base fee + metered components

### Payment Flow

```
Subscription Created → Invoice Generated → Payment Attempted
                                               ↓
                                          Success / Fail
                                               ↓
                                         Update Status
                                               ↓
                                     Retry (if failed)
```

### Tenant Isolation

- **Shared Database, Row-Level**: Query filtering by tenant_id
- **Shared Database, Schema-Level**: Separate schema per tenant
- **Database-Level**: Dedicated database per tenant

## React Components Usage

```tsx
import { PricingTable, CheckoutFlow } from '@harborgrid/enterprise-billing';

function PricingPage() {
  return (
    <PricingTable
      plans={plans}
      currentPlanId={currentSubscription?.planId}
      onSelectPlan={handleSelectPlan}
      highlightPlanId="pro"
      showAnnualSavings={true}
    />
  );
}

function CheckoutPage() {
  return (
    <CheckoutFlow
      plan={selectedPlan}
      tenant={currentTenant}
      onComplete={handleCheckoutComplete}
      onCancel={handleCancel}
    />
  );
}
```

## Revenue Analytics

```typescript
import { RevenueAnalytics } from '@harborgrid/enterprise-billing';

const analytics = new RevenueAnalytics();
analytics.setData(subscriptions, plans);

// Calculate MRR
const mrr = analytics.calculateMRR();

// Get revenue metrics
const metrics = analytics.calculateRevenueMetrics(
  startOfMonth(new Date()),
  endOfMonth(new Date())
);

// Analyze churn
const churn = analytics.calculateChurnMetrics(
  startOfMonth(new Date()),
  endOfMonth(new Date())
);

// Cohort analysis
const cohorts = analytics.generateCohortAnalysis(new Date('2024-01-01'));
```

## Webhooks

```typescript
import { WebhookService, WebhookEvent } from '@harborgrid/enterprise-billing';

const webhookService = new WebhookService();

// Register endpoint
const endpoint = await webhookService.registerEndpoint(
  'https://api.example.com/webhooks',
  [
    WebhookEvent.SUBSCRIPTION_CREATED,
    WebhookEvent.INVOICE_PAID,
    WebhookEvent.PAYMENT_FAILED,
  ]
);

// Dispatch event
await webhookService.dispatchEvent(
  WebhookEvent.SUBSCRIPTION_CREATED,
  { subscriptionId: '123' }
);

// Verify signature (in webhook handler)
const isValid = webhookService.verifySignature(
  payload,
  signature,
  endpoint.secret
);
```

## Testing

```bash
npm test
npm run test:coverage
```

## TypeScript Support

Full TypeScript support with complete type definitions included.

## License

MIT

## Author

HarborGrid

## Support

For support, please visit https://github.com/harborgrid/esxi/issues
