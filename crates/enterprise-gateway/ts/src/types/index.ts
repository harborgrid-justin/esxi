/**
 * Enterprise API Gateway - Core Types
 *
 * Complete type definitions for API Gateway, Rate Limiting, and Security
 */

import { z } from 'zod';

// ============================================================================
// HTTP & Request Types
// ============================================================================

export type HTTPMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS';

export interface HTTPHeaders {
  [key: string]: string | string[] | undefined;
}

export interface GatewayRequest {
  id: string;
  method: HTTPMethod;
  path: string;
  headers: HTTPHeaders;
  query: Record<string, string | string[]>;
  body?: unknown;
  ip: string;
  timestamp: number;
  consumer?: Consumer;
}

export interface GatewayResponse {
  statusCode: number;
  headers: HTTPHeaders;
  body?: unknown;
  duration: number;
  upstream?: string;
}

// ============================================================================
// Route Configuration
// ============================================================================

export type RouteMatchType = 'exact' | 'prefix' | 'regex';

export interface Route {
  id: string;
  name: string;
  methods: HTTPMethod[];
  paths: string[];
  matchType: RouteMatchType;
  stripPath?: boolean;
  preserveHost?: boolean;
  upstream: Upstream;
  plugins?: Plugin[];
  metadata?: Record<string, unknown>;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export const RouteSchema = z.object({
  id: z.string(),
  name: z.string(),
  methods: z.array(z.enum(['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS'])),
  paths: z.array(z.string()),
  matchType: z.enum(['exact', 'prefix', 'regex']),
  stripPath: z.boolean().optional(),
  preserveHost: z.boolean().optional(),
  upstream: z.object({
    id: z.string(),
    targets: z.array(z.string()),
  }),
  plugins: z.array(z.any()).optional(),
  metadata: z.record(z.unknown()).optional(),
  enabled: z.boolean(),
  createdAt: z.number(),
  updatedAt: z.number(),
});

// ============================================================================
// Upstream & Load Balancing
// ============================================================================

export type LoadBalancingAlgorithm =
  | 'round-robin'
  | 'weighted-round-robin'
  | 'least-connections'
  | 'ip-hash'
  | 'random'
  | 'consistent-hash';

export interface Target {
  id: string;
  url: string;
  weight: number;
  metadata?: Record<string, unknown>;
  healthy: boolean;
  activeConnections: number;
  lastChecked?: number;
}

export interface Upstream {
  id: string;
  name: string;
  targets: Target[];
  algorithm: LoadBalancingAlgorithm;
  healthChecks?: HealthCheck;
  retries: number;
  timeout: number;
  connectTimeout: number;
  sendTimeout: number;
  readTimeout: number;
  metadata?: Record<string, unknown>;
  createdAt: number;
  updatedAt: number;
}

// ============================================================================
// Health Checks
// ============================================================================

export type HealthCheckType = 'http' | 'https' | 'tcp';

export interface HealthCheck {
  type: HealthCheckType;
  interval: number; // milliseconds
  timeout: number;
  healthyThreshold: number;
  unhealthyThreshold: number;
  httpPath?: string;
  httpMethod?: HTTPMethod;
  expectedStatus?: number[];
  expectedBody?: string;
}

export interface HealthStatus {
  targetId: string;
  healthy: boolean;
  consecutiveSuccesses: number;
  consecutiveFailures: number;
  lastCheck: number;
  lastError?: string;
  responseTime?: number;
}

// ============================================================================
// Circuit Breaker
// ============================================================================

export type CircuitState = 'CLOSED' | 'OPEN' | 'HALF_OPEN';

export interface CircuitBreakerConfig {
  failureThreshold: number;
  successThreshold: number;
  timeout: number; // milliseconds to wait before transitioning to half-open
  monitoringPeriod: number;
  volumeThreshold: number; // minimum number of requests before circuit can open
}

export interface CircuitBreakerState {
  state: CircuitState;
  failures: number;
  successes: number;
  lastFailureTime?: number;
  nextAttemptTime?: number;
  totalRequests: number;
}

// ============================================================================
// Rate Limiting
// ============================================================================

export type RateLimitAlgorithm = 'token-bucket' | 'sliding-window' | 'fixed-window' | 'adaptive';

export interface RateLimit {
  id: string;
  name: string;
  algorithm: RateLimitAlgorithm;
  limit: number;
  window: number; // milliseconds
  burstSize?: number; // for token bucket
  refillRate?: number; // tokens per second
  scope: 'global' | 'consumer' | 'route' | 'ip';
  key?: string;
  enabled: boolean;
}

export interface RateLimitState {
  tokens: number;
  lastRefill: number;
  requestCount: number;
  windowStart: number;
}

export interface RateLimitResult {
  allowed: boolean;
  limit: number;
  remaining: number;
  resetAt: number;
  retryAfter?: number;
}

// ============================================================================
// Quota Management
// ============================================================================

export type QuotaPeriod = 'second' | 'minute' | 'hour' | 'day' | 'month';

export interface Quota {
  id: string;
  limit: number;
  period: QuotaPeriod;
  scope: 'global' | 'consumer' | 'route';
  resetOn?: number; // specific time of day/month
}

export interface QuotaUsage {
  used: number;
  limit: number;
  period: QuotaPeriod;
  resetAt: number;
}

// ============================================================================
// Throttling
// ============================================================================

export interface ThrottleConfig {
  requestsPerSecond: number;
  burstSize: number;
  delayAfter?: number;
  delayMs?: number;
}

// ============================================================================
// Authentication & Authorization
// ============================================================================

export type AuthType = 'api-key' | 'oauth2' | 'jwt' | 'basic' | 'mtls';

export interface APIKey {
  id: string;
  key: string;
  name: string;
  consumerId: string;
  scopes: string[];
  rateLimit?: RateLimit;
  quota?: Quota;
  enabled: boolean;
  expiresAt?: number;
  createdAt: number;
  metadata?: Record<string, unknown>;
}

export interface Consumer {
  id: string;
  username: string;
  customId?: string;
  apiKeys: APIKey[];
  rateLimit?: RateLimit;
  quota?: Quota;
  tags: string[];
  metadata?: Record<string, unknown>;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

// ============================================================================
// OAuth2
// ============================================================================

export interface OAuthToken {
  accessToken: string;
  tokenType: string;
  expiresIn: number;
  refreshToken?: string;
  scope: string[];
  issuedAt: number;
}

export interface OAuthConfig {
  tokenEndpoint: string;
  authorizationEndpoint: string;
  introspectionEndpoint?: string;
  clientId: string;
  clientSecret: string;
  scopes: string[];
}

// ============================================================================
// JWT
// ============================================================================

export interface JWTConfig {
  secret?: string;
  publicKey?: string;
  algorithm: 'HS256' | 'HS384' | 'HS512' | 'RS256' | 'RS384' | 'RS512' | 'ES256' | 'ES384' | 'ES512';
  issuer?: string;
  audience?: string;
  clockTolerance?: number;
}

export interface JWTPayload {
  sub?: string;
  iss?: string;
  aud?: string | string[];
  exp?: number;
  nbf?: number;
  iat?: number;
  jti?: string;
  [key: string]: unknown;
}

// ============================================================================
// IP Filtering
// ============================================================================

export type IPFilterMode = 'whitelist' | 'blacklist';

export interface IPFilter {
  mode: IPFilterMode;
  addresses: string[]; // Supports CIDR notation
  enabled: boolean;
}

// ============================================================================
// WAF (Web Application Firewall)
// ============================================================================

export type WAFRuleType = 'sql-injection' | 'xss' | 'path-traversal' | 'command-injection' | 'custom';
export type WAFAction = 'block' | 'log' | 'challenge';

export interface WAFRule {
  id: string;
  type: WAFRuleType;
  pattern?: string | RegExp;
  action: WAFAction;
  message: string;
  enabled: boolean;
  severity: 'low' | 'medium' | 'high' | 'critical';
}

export interface WAFResult {
  allowed: boolean;
  matchedRules: WAFRule[];
  action: WAFAction;
}

// ============================================================================
// Request/Response Transformation
// ============================================================================

export interface TransformRule {
  id: string;
  type: 'add' | 'remove' | 'replace' | 'rename';
  target: 'header' | 'query' | 'body' | 'path';
  field: string;
  value?: string;
  newField?: string;
  condition?: string; // JavaScript expression
}

export interface BodyTransform {
  contentType: string;
  schema?: unknown; // JSON Schema
  transforms: TransformRule[];
}

// ============================================================================
// Plugins
// ============================================================================

export type PluginPhase = 'pre-route' | 'route' | 'post-route' | 'error';

export interface Plugin {
  id: string;
  name: string;
  enabled: boolean;
  phase: PluginPhase;
  priority: number;
  config: Record<string, unknown>;
}

export interface PluginContext {
  request: GatewayRequest;
  response?: GatewayResponse;
  route?: Route;
  consumer?: Consumer;
  state: Map<string, unknown>;
}

export type PluginHandler = (context: PluginContext) => Promise<void | GatewayResponse>;

// ============================================================================
// Middleware
// ============================================================================

export type NextFunction = () => Promise<void>;

export type MiddlewareHandler = (
  request: GatewayRequest,
  response?: GatewayResponse,
  next?: NextFunction
) => Promise<void | GatewayResponse>;

// ============================================================================
// Service Discovery
// ============================================================================

export type ServiceDiscoveryType = 'static' | 'dns' | 'consul' | 'eureka' | 'kubernetes';

export interface ServiceDiscoveryConfig {
  type: ServiceDiscoveryType;
  endpoints?: string[];
  namespace?: string;
  interval?: number; // polling interval in ms
  metadata?: Record<string, unknown>;
}

export interface ServiceInstance {
  id: string;
  name: string;
  address: string;
  port: number;
  metadata?: Record<string, unknown>;
  healthy: boolean;
  weight: number;
}

// ============================================================================
// Caching
// ============================================================================

export type CacheStrategy = 'time-based' | 'lru' | 'lfu';

export interface CacheConfig {
  enabled: boolean;
  strategy: CacheStrategy;
  ttl: number; // milliseconds
  maxSize: number; // bytes
  varyHeaders?: string[];
  cacheableStatusCodes: number[];
  cacheMethods: HTTPMethod[];
}

export interface CacheEntry {
  key: string;
  value: GatewayResponse;
  size: number;
  createdAt: number;
  expiresAt: number;
  hits: number;
}

// ============================================================================
// Metrics & Analytics
// ============================================================================

export interface RequestMetrics {
  routeId: string;
  consumerId?: string;
  method: HTTPMethod;
  path: string;
  statusCode: number;
  duration: number;
  upstream?: string;
  cached: boolean;
  rateLimited: boolean;
  timestamp: number;
}

export interface AggregatedMetrics {
  totalRequests: number;
  successRate: number;
  averageLatency: number;
  p50Latency: number;
  p95Latency: number;
  p99Latency: number;
  requestsPerSecond: number;
  errorRate: number;
  cacheHitRate: number;
  rateLimitRate: number;
}

export interface TrafficMetrics {
  route: string;
  requests: number;
  bandwidth: number;
  errors: number;
  latency: {
    avg: number;
    p50: number;
    p95: number;
    p99: number;
  };
}

// ============================================================================
// Logging
// ============================================================================

export type LogLevel = 'debug' | 'info' | 'warn' | 'error' | 'fatal';

export interface LogEntry {
  timestamp: number;
  level: LogLevel;
  message: string;
  requestId?: string;
  routeId?: string;
  consumerId?: string;
  metadata?: Record<string, unknown>;
  error?: Error;
}

// ============================================================================
// Gateway Configuration
// ============================================================================

export interface GatewayConfig {
  port: number;
  host: string;
  workers?: number;
  redis?: {
    host: string;
    port: number;
    password?: string;
    db?: number;
  };
  cors?: {
    enabled: boolean;
    origins: string[];
    methods: HTTPMethod[];
    headers: string[];
    credentials: boolean;
  };
  compression?: {
    enabled: boolean;
    threshold: number; // bytes
    level: number; // 1-9
  };
  ssl?: {
    enabled: boolean;
    cert: string;
    key: string;
    ca?: string;
  };
  admin?: {
    enabled: boolean;
    port: number;
    apiKey?: string;
  };
}

// ============================================================================
// Error Types
// ============================================================================

export class GatewayError extends Error {
  constructor(
    message: string,
    public statusCode: number = 500,
    public code?: string,
    public metadata?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'GatewayError';
  }
}

export class RateLimitError extends GatewayError {
  constructor(message: string, public retryAfter: number) {
    super(message, 429, 'RATE_LIMIT_EXCEEDED');
    this.name = 'RateLimitError';
  }
}

export class AuthenticationError extends GatewayError {
  constructor(message: string) {
    super(message, 401, 'AUTHENTICATION_FAILED');
    this.name = 'AuthenticationError';
  }
}

export class AuthorizationError extends GatewayError {
  constructor(message: string) {
    super(message, 403, 'AUTHORIZATION_FAILED');
    this.name = 'AuthorizationError';
  }
}

export class UpstreamError extends GatewayError {
  constructor(message: string, public upstream?: string) {
    super(message, 502, 'UPSTREAM_ERROR');
    this.name = 'UpstreamError';
  }
}

export class CircuitBreakerError extends GatewayError {
  constructor(message: string) {
    super(message, 503, 'CIRCUIT_BREAKER_OPEN');
    this.name = 'CircuitBreakerError';
  }
}
