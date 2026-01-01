# @esxi/enterprise-bridge

TypeScript-Rust WASM Bridge for the $983M Enterprise SaaS Platform v0.5.

## Overview

The Enterprise Bridge provides high-performance, type-safe bindings between TypeScript and Rust WebAssembly for critical enterprise services:

- **CAD Engine**: Geometry operations, spatial analysis, coordinate transformations
- **Compression**: Multi-algorithm data compression (gzip, brotli, zstd, lz4)
- **Query Optimizer**: SQL query optimization and execution planning
- **Collaboration**: Real-time collaboration with CRDT and Operational Transform
- **Security**: Input validation and sanitization for XSS, SQL injection, CSRF, etc.

## Features

- 🚀 **High Performance**: Zero-copy data transfer between JavaScript and Rust
- 🔒 **Type Safe**: Full TypeScript type definitions matching Rust types
- ⚡ **Async/Await**: Seamless async integration with JavaScript Promises
- 🎯 **Memory Efficient**: Advanced memory pooling and management
- 🔄 **Instance Pooling**: Concurrent WASM instance pool for parallel processing
- 📊 **Monitoring**: Built-in performance metrics and memory tracking
- 🛡️ **Enterprise Ready**: Production-grade error handling and logging

## Installation

```bash
npm install @esxi/enterprise-bridge
```

## Building WASM Module

Before using the bridge, you need to build the Rust WASM module:

```bash
# Build in release mode
npm run build:wasm

# Build in development mode (faster, larger)
npm run build:wasm:dev
```

This requires [wasm-pack](https://rustwasm.github.io/wasm-pack/) to be installed:

```bash
cargo install wasm-pack
```

## Quick Start

### Basic Usage

```typescript
import { EnterpriseBridge } from '@esxi/enterprise-bridge';

// Create and initialize the bridge
const bridge = new EnterpriseBridge();
await bridge.initialize();

// Use CAD services
const geometry = {
  geometryType: 'Polygon',
  coordinates: [[
    [-122.4, 37.8],
    [-122.3, 37.8],
    [-122.3, 37.7],
    [-122.4, 37.7],
    [-122.4, 37.8],
  ]],
  properties: {},
};

const validationResult = await bridge.cad.validateGeometry(geometry);
console.log('Geometry valid:', validationResult.success);

// Use compression services
const data = new TextEncoder().encode('Hello, World!');
const compressed = await bridge.compression.compress(data, {
  algorithm: 'gzip',
  level: 6,
});

// Use security services
const xssCheck = await bridge.security.validateXss('<script>alert("xss")</script>');
console.log('XSS detected:', !xssCheck.data?.isSafe);

// Cleanup when done
await bridge.dispose();
```

### Using Individual Services

```typescript
import { WasmLoader, CadBridge, SecurityBridge } from '@esxi/enterprise-bridge';

// Initialize WASM loader
const loader = new WasmLoader();
await loader.initialize();

// Create service bridges
const cad = new CadBridge(loader);
const security = new SecurityBridge(loader, true); // strict mode

// Use services
const bbox = await cad.calculateBbox(geometry);
const sanitized = await security.sanitizeHtml('<div onclick="alert()">Safe</div>');

// Cleanup
cad.dispose();
security.dispose();
loader.dispose();
```

### Using WASM Instance Pool

For high-concurrency scenarios, use the instance pool:

```typescript
import { EnterpriseBridge } from '@esxi/enterprise-bridge';

const bridge = new EnterpriseBridge({
  usePool: true,
  poolOptions: {
    initialSize: 4,
    maxSize: 20,
    acquireTimeoutMs: 5000,
  },
});

await bridge.initialize();

// Services automatically use the pool for concurrent operations
const results = await Promise.all([
  bridge.compression.compress(data1, params),
  bridge.compression.compress(data2, params),
  bridge.compression.compress(data3, params),
  bridge.compression.compress(data4, params),
]);
```

## Service APIs

### CAD Bridge

```typescript
// Validate geometry
const errors = await bridge.cad.validateGeometry(geometry);

// Calculate bounding box
const bbox = await bridge.cad.calculateBbox(geometry);

// Simplify geometry
const simplified = await bridge.cad.simplify(geometry, 0.0001);

// Create buffer
const buffered = await bridge.cad.buffer(geometry, 100, 16);

// Boolean operations
const union = await bridge.cad.union(geom1, geom2);
const intersection = await bridge.cad.intersection(geom1, geom2);

// Coordinate transformation
const transformed = await bridge.cad.transform(
  geometry,
  'EPSG:4326',
  'EPSG:3857'
);
```

### Compression Bridge

```typescript
// Compress data
const compressed = await bridge.compression.compress(data, {
  algorithm: 'zstd',
  level: 9,
});

// Decompress data
const decompressed = await bridge.compression.decompress(compressed.data!, 'zstd');

// Get compression ratio estimate
const ratio = await bridge.compression.estimateRatio(data, params);

// Auto-select best algorithm
const recommendation = await bridge.compression.selectAlgorithm(data);

// Compress/decompress strings and JSON
const compressedJson = await bridge.compression.compressJson(obj, params);
const decompressedObj = await bridge.compression.decompressJson(compressedJson.data!, 'gzip');
```

### Query Bridge

```typescript
// Validate query
const validation = await bridge.query.validateQuery('SELECT * FROM users');

// Optimize query
const plan = await bridge.query.optimize({
  query: 'SELECT * FROM users WHERE age > ?',
  params: [18],
  optimize: true,
});

// Execute query
const result = await bridge.query.execute({
  query: 'SELECT * FROM users LIMIT 10',
  params: [],
});

// Explain query
const explanation = await bridge.query.explain('SELECT * FROM users');

// Get statistics
const stats = await bridge.query.getStats();
```

### Collaboration Bridge

```typescript
const collaboration = new CollaborationBridge(loader, 'user-123', 10);

// Apply local operation
const event = await collaboration.applyLocalOperation('insert', {
  position: 10,
  text: 'Hello',
});

// Apply remote operation
const transformed = await collaboration.applyRemoteOperation(remoteEvent);

// Merge concurrent operations
const merged = await collaboration.mergeOperations(event1, event2);

// CRDT text editing
await collaboration.insertText(0, 'Hello');
await collaboration.deleteText(5, 1);
const content = await collaboration.getTextContent();

// Presence tracking
const presence = await collaboration.createPresenceEvent({
  cursor: { line: 10, column: 5 },
  selection: { start: 10, end: 20 },
});
```

### Security Bridge

```typescript
// Validate for threats
const xssResult = await bridge.security.validateXss(input);
const sqlResult = await bridge.security.validateSqlInjection(input);
const csrfResult = await bridge.security.validateCsrf(token);

// Sanitize input
const safeHtml = await bridge.security.sanitizeHtml(dangerousHtml);
const safeSql = await bridge.security.sanitizeSql(userInput);
const safeUrl = await bridge.security.sanitizeUrl(url);
const safeFilename = await bridge.security.sanitizeFilename(filename);

// Password operations
const validation = await bridge.security.validatePassword('MyP@ssw0rd123!');
const hash = await bridge.security.hashPassword('password');
const isValid = await bridge.security.verifyPassword('password', hash);

// Token generation
const csrfToken = await bridge.security.generateToken(32);

// CSP validation
const cspResult = await bridge.security.validateCsp(cspHeader);
```

## Performance Monitoring

```typescript
// Get bridge statistics
const stats = await bridge.getStats();
console.log('Memory usage:', stats.bridge.memoryUsage);
console.log('Active operations:', stats.bridge.activeOperations);

// Pool statistics
if (stats.pool) {
  console.log('Total instances:', stats.pool.totalInstances);
  console.log('Available:', stats.pool.availableInstances);
  console.log('In use:', stats.pool.inUseInstances);
}

// Health check
const healthy = bridge.healthCheck();
```

## Memory Management

```typescript
import { MemoryMonitor, formatMemoryUsage } from '@esxi/enterprise-bridge';

// Monitor memory usage
const monitor = new MemoryMonitor(
  async () => {
    const stats = await bridge.getStats();
    return {
      totalAllocated: stats.bridge.memoryUsage,
      used: stats.bridge.memoryUsage,
      available: 0,
      allocations: 0,
    };
  },
  80 // Warn at 80% usage
);

monitor.start(5000); // Check every 5 seconds

// Stop monitoring
monitor.stop();
```

## Advanced Configuration

```typescript
const bridge = new EnterpriseBridge({
  wasmOptions: {
    wasmPath: './custom/path/to/wasm.wasm',
    config: {
      enablePerformanceMonitoring: true,
      memoryConfig: {
        initialPoolSize: 20 * 1024 * 1024, // 20 MB
        maxPoolSize: 200 * 1024 * 1024,    // 200 MB
        aggressiveGc: false,
      },
      maxConcurrentOperations: 200,
      debugMode: false,
    },
    verbose: true,
  },
  poolOptions: {
    initialSize: 4,
    maxSize: 20,
    acquireTimeoutMs: 5000,
  },
  usePool: true,
  userId: 'user-123',
  queryCache: true,
  strictSecurity: true,
});
```

## Error Handling

```typescript
import { BridgeError } from '@esxi/enterprise-bridge';

try {
  await bridge.cad.validateGeometry(invalidGeometry);
} catch (error) {
  if (error instanceof BridgeError) {
    console.error('Bridge error:', error.code, error.message);
    console.error('Details:', error.details);
  } else {
    console.error('Unknown error:', error);
  }
}
```

## TypeScript Types

All types are fully documented and exported:

```typescript
import type {
  BridgeConfig,
  CadGeometry,
  CompressionParams,
  QueryParams,
  SecurityParams,
  CollaborationEvent,
  OperationResult,
} from '@esxi/enterprise-bridge';
```

## Development

```bash
# Install dependencies
npm install

# Build TypeScript
npm run build

# Build WASM module
npm run build:wasm

# Watch mode
npm run watch

# Run tests
npm test

# Type check
npm run type-check

# Lint
npm run lint
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│           TypeScript Application                │
└─────────────────┬───────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────┐
│          Enterprise Bridge (TypeScript)         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │   CAD    │  │Compress  │  │  Query   │      │
│  │  Bridge  │  │  Bridge  │  │  Bridge  │      │
│  └──────────┘  └──────────┘  └──────────┘      │
│  ┌──────────┐  ┌──────────┐                    │
│  │  Collab  │  │ Security │                    │
│  │  Bridge  │  │  Bridge  │                    │
│  └──────────┘  └──────────┘                    │
└─────────────────┬───────────────────────────────┘
                  │
                  ▼
┌─────────────────────────────────────────────────┐
│          WASM Bridge Layer (Rust)               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
│  │   CAD    │  │Compress  │  │  Query   │      │
│  │ Engine   │  │  Engine  │  │Optimizer │      │
│  └──────────┘  └──────────┘  └──────────┘      │
│  ┌──────────┐  ┌──────────┐                    │
│  │  Collab  │  │ Security │                    │
│  │  Engine  │  │  Engine  │                    │
│  └──────────┘  └──────────┘                    │
└─────────────────────────────────────────────────┘
```

## License

MIT

## Contributing

This is part of the $983M Enterprise SaaS Platform. For contribution guidelines, see the main repository.

## Support

For enterprise support and licensing inquiries, contact: engineering@harborgrid.com
