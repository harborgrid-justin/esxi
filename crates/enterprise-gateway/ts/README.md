# @harborgrid/enterprise-gateway

Enterprise API Gateway & Rate Limiting System with Load Balancing, Circuit Breaking, and Advanced Security.

## Features

### Core Gateway
- **Request Routing** - Advanced route matching (exact, prefix, regex)
- **Load Balancing** - Multiple algorithms (round-robin, weighted, least-connections, IP hash, consistent hash)
- **Circuit Breaking** - Fault tolerance and cascade failure prevention
- **Health Checking** - Active and passive health checks for upstream targets
- **Service Discovery** - Dynamic service discovery (DNS, Consul, Eureka, Kubernetes)

### Rate Limiting
- **Multiple Algorithms**:
  - Token Bucket
  - Sliding Window
  - Fixed Window
  - Adaptive Rate Limiting
- **Distributed Limiting** - Redis-backed for multi-instance deployments
- **Flexible Scoping** - Global, per-consumer, per-route, per-IP
- **Burst Support** - Handle traffic spikes gracefully

### Security
- **API Key Management** - Generate, validate, and manage API keys
- **OAuth2 Validation** - Token introspection and validation
- **JWT Validation** - JSON Web Token verification with multiple algorithms
- **IP Filtering** - Whitelist/blacklist with CIDR support
- **WAF Engine** - Web Application Firewall with threat detection
- **Request Sanitization** - Input sanitization and normalization

### Transformation
- **Request/Response Transformation** - Modify headers, query params, body, path
- **Body Parsing** - Support for JSON, form-urlencoded, XML, multipart
- **Header Management** - Advanced header manipulation
- **Protocol Translation** - Convert between REST, GraphQL, gRPC

### Services
- **Gateway Service** - Core gateway operations
- **Metrics Service** - Request metrics and analytics
- **Logging Service** - Comprehensive logging and audit trails
- **Cache Service** - Response caching with LRU/LFU strategies

### React Components
- **GatewayDashboard** - Real-time metrics and monitoring
- **RouteManager** - Route configuration UI
- **ConsumerManager** - Consumer and API key management
- **AnalyticsDashboard** - Traffic analytics and visualization
- **HealthStatus** - Backend health monitoring
- **RateLimitConfig** - Rate limit configuration
- **PluginManager** - Plugin management

## Installation

```bash
npm install @harborgrid/enterprise-gateway
```

## Quick Start

```typescript
import {
  GatewayEngine,
  GatewayService,
  RateLimiter,
  APIKeyManager,
  JWTValidator,
} from '@harborgrid/enterprise-gateway';

// Configure gateway
const config = {
  port: 8080,
  host: '0.0.0.0',
  workers: 4,
  redis: {
    host: 'localhost',
    port: 6379,
  },
  cors: {
    enabled: true,
    origins: ['*'],
    methods: ['GET', 'POST', 'PUT', 'DELETE'],
    headers: ['Content-Type', 'Authorization'],
    credentials: true,
  },
};

// Create gateway service
const gateway = new GatewayService(config);

// Register a route
gateway.registerRoute({
  id: 'users-api',
  name: 'Users API',
  methods: ['GET', 'POST', 'PUT', 'DELETE'],
  paths: ['/api/users', '/api/users/:id'],
  matchType: 'prefix',
  upstream: {
    id: 'users-upstream',
    name: 'Users Service',
    targets: [
      {
        id: 'users-1',
        url: 'http://localhost:3001',
        weight: 1,
        healthy: true,
        activeConnections: 0,
      },
      {
        id: 'users-2',
        url: 'http://localhost:3002',
        weight: 1,
        healthy: true,
        activeConnections: 0,
      },
    ],
    algorithm: 'round-robin',
    retries: 3,
    timeout: 30000,
    connectTimeout: 5000,
    sendTimeout: 30000,
    readTimeout: 30000,
    createdAt: Date.now(),
    updatedAt: Date.now(),
  },
  enabled: true,
  createdAt: Date.now(),
  updatedAt: Date.now(),
});

// Handle requests
const request = {
  id: 'req-123',
  method: 'GET' as const,
  path: '/api/users',
  headers: {},
  query: {},
  ip: '192.168.1.1',
  timestamp: Date.now(),
};

const response = await gateway.handleRequest(request);
console.log(response);
```

## Rate Limiting Example

```typescript
import { RateLimiter, TokenBucket, SlidingWindow } from '@harborgrid/enterprise-gateway';

// Create rate limiter
const limiter = new RateLimiter();

// Define rate limit
const rateLimit = {
  id: 'api-limit',
  name: 'API Rate Limit',
  algorithm: 'token-bucket' as const,
  limit: 100,
  window: 60000, // 1 minute
  refillRate: 100 / 60, // 100 tokens per 60 seconds
  burstSize: 150,
  scope: 'consumer' as const,
  enabled: true,
};

// Check rate limit
const result = await limiter.checkLimit(rateLimit, request);

if (!result.allowed) {
  console.log(`Rate limit exceeded. Retry after ${result.retryAfter}ms`);
}
```

## Security Example

```typescript
import { APIKeyManager, JWTValidator, WAFEngine } from '@harborgrid/enterprise-gateway';

// API Key Authentication
const apiKeyManager = new APIKeyManager();

// Generate API key
const apiKey = apiKeyManager.generateKey('consumer-123', 'Production Key', ['read', 'write']);
console.log('Generated API key:', apiKey.key);

// Validate API key
const validatedKey = await apiKeyManager.validate(request, ['read']);

// JWT Authentication
const jwtValidator = new JWTValidator({
  secret: 'your-secret-key',
  algorithm: 'HS256',
  issuer: 'your-issuer',
  audience: 'your-audience',
});

const payload = await jwtValidator.validate(request);

// WAF Protection
const waf = new WAFEngine();
const wafResult = waf.analyze(request);

if (!wafResult.allowed) {
  console.log('WAF blocked request:', wafResult.matchedRules);
}
```

## React Components Example

```typescript
import { GatewayDashboard, RouteManager } from '@harborgrid/enterprise-gateway';

function App() {
  return (
    <div>
      <GatewayDashboard
        config={config}
        metrics={metrics}
        onRefresh={() => console.log('Refreshing...')}
      />

      <RouteManager
        routes={routes}
        upstreams={upstreams}
        onAddRoute={(route) => console.log('Adding route:', route)}
        onUpdateRoute={(id, updates) => console.log('Updating route:', id, updates)}
        onDeleteRoute={(id) => console.log('Deleting route:', id)}
      />
    </div>
  );
}
```

## Architecture

The gateway is built with a modular architecture:

1. **Core Layer** - Request routing, load balancing, circuit breaking
2. **Rate Limiting Layer** - Multiple algorithms with distributed support
3. **Security Layer** - Authentication, authorization, WAF, sanitization
4. **Transformation Layer** - Request/response modification
5. **Service Layer** - High-level gateway operations
6. **UI Layer** - React components for management

## Performance

- **High Throughput** - Handles thousands of requests per second
- **Low Latency** - Sub-millisecond routing decisions
- **Efficient Memory** - LRU/LFU caching with configurable limits
- **Scalable** - Horizontal scaling with Redis-backed rate limiting

## License

MIT

## Contributing

Contributions are welcome! Please read our contributing guidelines.

## Support

For support, please open an issue on GitHub.
