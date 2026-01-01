# @harborgrid/enterprise-shared

Shared types, interfaces, and utilities for Enterprise SaaS Platform v0.4.

## Installation

```bash
npm install @harborgrid/enterprise-shared
```

## Features

- **Common Types**: Multi-tenant, user, pagination, API responses, errors
- **Event Bus Types**: Enterprise event definitions and event bus interfaces
- **API Types**: HTTP, REST, webhooks, versioning, rate limiting
- **Validation Utilities**: Zod-based validation with common validators
- **TypeScript First**: Full TypeScript support with type exports
- **Tree-Shakeable**: Modular exports for optimal bundle size

## Usage

### Common Types

```typescript
import {
  Tenant,
  User,
  PaginatedResult,
  ApiResponse,
  EnterpriseError,
} from '@harborgrid/enterprise-shared/types/common';

// Use tenant type
const tenant: Tenant = {
  id: '123',
  name: 'Acme Corp',
  slug: 'acme',
  status: TenantStatus.ACTIVE,
  plan: 'pro',
  metadata: {},
  createdAt: new Date(),
  updatedAt: new Date(),
};

// Create API response
const response: ApiResponse<User[]> = {
  success: true,
  data: users,
  metadata: {
    timestamp: new Date(),
    requestId: 'req-123',
    version: '1.0.0',
  },
};

// Throw enterprise error
throw new EnterpriseError('Resource not found', 'NOT_FOUND', 404);
```

### Event Types

```typescript
import {
  EnterpriseEvent,
  EventBuilder,
  BillingEventType,
} from '@harborgrid/enterprise-shared/types/events';

// Build and publish event
const event = new EventBuilder(
  BillingEventType.SUBSCRIPTION_CREATED,
  EventSource.BILLING
)
  .withTenant(tenantId)
  .withUser(userId)
  .withData({
    subscriptionId: 'sub-123',
    planId: 'pro',
    amount: 99.99,
  })
  .withCorrelationId(correlationId)
  .build();

await eventBus.publish(event);
```

### API Types

```typescript
import {
  HTTPRequest,
  HTTPResponse,
  APIVersion,
  RateLimitInfo,
} from '@harborgrid/enterprise-shared/types/api';

// Define API version
const apiV1: APIVersion = {
  version: '1.0.0',
  path: '/v1',
  status: APIVersionStatus.ACTIVE,
};

// Return rate limit info
const rateLimit: RateLimitInfo = {
  limit: 1000,
  remaining: 999,
  reset: Date.now() + 3600000,
};
```

### Validation

```typescript
import {
  validate,
  validateOrThrow,
  emailValidator,
  uuidValidator,
  z,
} from '@harborgrid/enterprise-shared/utils/validation';

// Validate email
const result = validate(emailValidator, 'user@example.com');
if (result.success) {
  console.log('Valid email:', result.data);
} else {
  console.error('Validation errors:', result.errors);
}

// Validate or throw
const userId = validateOrThrow(uuidValidator, input);

// Custom validation
const userSchema = z.object({
  email: emailValidator,
  name: z.string().min(1),
  age: z.number().positive(),
});

const user = validateOrThrow(userSchema, data);
```

### Custom Validators

```typescript
import {
  createTenantValidator,
  createFileSizeValidator,
  createDateRangeValidator,
} from '@harborgrid/enterprise-shared/utils/validation';

// Validate tenant access
const tenantValidator = createTenantValidator(['tenant-1', 'tenant-2']);
const tenantId = validateOrThrow(tenantValidator, 'tenant-1');

// Validate file size (max 10MB)
const fileSizeValidator = createFileSizeValidator(10 * 1024 * 1024);
const size = validateOrThrow(fileSizeValidator, fileSize);

// Validate date range (max 30 days)
const dateRangeValidator = createDateRangeValidator(30);
const range = validateOrThrow(dateRangeValidator, {
  start: new Date('2024-01-01'),
  end: new Date('2024-01-15'),
});
```

### Sanitization

```typescript
import {
  sanitizeHTML,
  sanitizeFilename,
  sanitizeSlug,
} from '@harborgrid/enterprise-shared/utils/validation';

// Sanitize HTML to prevent XSS
const safe = sanitizeHTML('<script>alert("xss")</script>');
// Output: &lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;

// Sanitize filename
const filename = sanitizeFilename('my file (1).txt');
// Output: my_file_1.txt

// Sanitize slug
const slug = sanitizeSlug('My Blog Post!');
// Output: my-blog-post
```

### Type Guards

```typescript
import {
  isUUID,
  isEmail,
  isURL,
  isValidDate,
} from '@harborgrid/enterprise-shared/utils/validation';

if (isUUID(value)) {
  // TypeScript knows value is a string
  console.log('Valid UUID:', value);
}

if (isEmail(value)) {
  // TypeScript knows value is a string
  sendEmail(value);
}
```

### Assertions

```typescript
import {
  assertDefined,
  assertNonEmptyString,
  assertPositiveNumber,
} from '@harborgrid/enterprise-shared/utils/validation';

// Assert value is defined
assertDefined(user, 'User not found');
// TypeScript knows user is not null/undefined after this

// Assert non-empty string
assertNonEmptyString(name, 'Name is required');
// TypeScript knows name is a non-empty string

// Assert positive number
assertPositiveNumber(amount, 'Amount must be positive');
// TypeScript knows amount is a positive number
```

## Module Exports

### Types

- `@harborgrid/enterprise-shared/types` - All types
- `@harborgrid/enterprise-shared/types/common` - Common types only
- `@harborgrid/enterprise-shared/types/events` - Event types only
- `@harborgrid/enterprise-shared/types/api` - API types only

### Utils

- `@harborgrid/enterprise-shared/utils` - All utilities
- `@harborgrid/enterprise-shared/utils/validation` - Validation utilities only

## Type Definitions

### Tenant

Multi-tenant isolation types for SaaS platforms.

```typescript
interface Tenant {
  id: string;
  name: string;
  slug: string;
  status: TenantStatus;
  plan: string;
  metadata: Record<string, unknown>;
  createdAt: Date;
  updatedAt: Date;
  deletedAt?: Date;
}
```

### User

User identity and authentication types.

```typescript
interface User {
  id: string;
  tenantId: string;
  email: string;
  displayName: string;
  role: UserRole;
  status: UserStatus;
  metadata: Record<string, unknown>;
  createdAt: Date;
  updatedAt: Date;
}
```

### EnterpriseEvent

Event bus event definition.

```typescript
interface EnterpriseEvent<T = unknown> {
  id: string;
  type: EventType;
  version: string;
  timestamp: Date;
  source: EventSource;
  tenantId?: string;
  userId?: string;
  data: T;
  metadata: EventMetadata;
}
```

### ApiResponse

Standardized API response format.

```typescript
interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: ApiError;
  metadata?: ApiMetadata;
}
```

## Integration with Enterprise Modules

This shared library is used by all 10 enterprise modules:

- `@harborgrid/enterprise-analytics` - Business Intelligence & Analytics
- `@harborgrid/enterprise-billing` - Subscription & Billing Management
- `@harborgrid/enterprise-cad-editor` - CAD/Vector Editor
- `@esxi/enterprise-collaboration` - Real-time Collaboration
- `@enterprise/compression` - Data Compression & Optimization
- `@harborgrid/enterprise-gateway` - API Gateway & Rate Limiting
- `@harborgrid/enterprise-notifications` - Multi-channel Notifications
- `@harborgrid/enterprise-security` - Security & Compliance
- `@harborgrid/enterprise-spatial` - GIS Spatial Analysis
- `@enterprise/workflow` - Workflow Automation

## Event Types Reference

### Billing Events

- `billing.subscription.created`
- `billing.subscription.updated`
- `billing.invoice.paid`
- `billing.payment.failed`

### Security Events

- `security.login.success`
- `security.threat.detected`
- `security.audit.logged`

### Analytics Events

- `analytics.query.executed`
- `analytics.threshold.exceeded`
- `analytics.report.generated`

### Workflow Events

- `workflow.execution.started`
- `workflow.execution.completed`
- `workflow.approval.requested`

See full event catalog in [INTEGRATION_REPORT_v0.4.md](../../../INTEGRATION_REPORT_v0.4.md).

## Error Handling

```typescript
import { EnterpriseError, ValidationError, NotFoundError } from '@harborgrid/enterprise-shared';

try {
  // Operation that might fail
} catch (error) {
  if (error instanceof ValidationError) {
    // Handle validation error
    return res.status(400).json({
      success: false,
      error: error.toJSON(),
    });
  }

  if (error instanceof NotFoundError) {
    // Handle not found
    return res.status(404).json({
      success: false,
      error: error.toJSON(),
    });
  }

  // Generic error handling
  throw error;
}
```

## Best Practices

1. **Always use shared types** for cross-module communication
2. **Validate input** using Zod schemas before processing
3. **Use EventBuilder** for creating well-formed events
4. **Follow API response format** for consistency
5. **Include correlation IDs** for distributed tracing
6. **Sanitize user input** to prevent XSS and injection attacks
7. **Use type guards** for runtime type checking

## Contributing

See [CONTRIBUTING.md](../../../CONTRIBUTING.md) for contribution guidelines.

## License

MIT License - see [LICENSE](../../../LICENSE) for details.

## Related Documentation

- [Integration Report](../../../INTEGRATION_REPORT_v0.4.md)
- [API Documentation](../../../docs/api/)
- [Architecture Guide](../../../docs/architecture/)
