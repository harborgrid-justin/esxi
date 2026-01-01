# Integration Tests

This directory contains integration tests for the Enterprise SaaS Platform v0.4.

## Structure

```
integration/
├── billing-security/        # Billing + Security integration tests
├── analytics-gateway/       # Analytics + Gateway integration tests
├── collaboration-cad/       # Collaboration + CAD Editor integration tests
├── workflow-notifications/  # Workflow + Notifications integration tests
├── gateway-all-services/    # Gateway with all services
└── shared/                  # Shared test utilities
```

## Running Tests

```bash
# Run all integration tests
npm run test:integration

# Run specific integration suite
npm run test:integration -- billing-security

# Run with coverage
npm run test:integration:coverage
```

## Writing Integration Tests

Integration tests should:
1. Test real module-to-module communication
2. Use actual service instances (not mocks)
3. Verify event bus integration
4. Test error handling and resilience
5. Validate data consistency across modules

## Test Environment

Integration tests run against:
- Docker Compose services (local)
- Test database (PostgreSQL)
- Test Redis instance
- Test event bus (Redis Pub/Sub)

## Example Test

```typescript
import { BillingService } from '@harborgrid/enterprise-billing';
import { SecurityService } from '@harborgrid/enterprise-security';
import { setupTestEnvironment, teardownTestEnvironment } from '../shared';

describe('Billing + Security Integration', () => {
  let billing: BillingService;
  let security: SecurityService;

  beforeAll(async () => {
    const env = await setupTestEnvironment();
    billing = env.billing;
    security = env.security;
  });

  afterAll(async () => {
    await teardownTestEnvironment();
  });

  it('should enforce permissions based on subscription plan', async () => {
    const tenant = await billing.createSubscription({
      planId: 'free',
      tenantId: 'test-tenant',
    });

    const hasAccess = await security.checkPermission({
      userId: 'test-user',
      tenantId: tenant.tenantId,
      resource: 'analytics.advanced_visualizations',
      action: 'read',
    });

    expect(hasAccess).toBe(false);
  });
});
```
