/**
 * API Contract Type Definitions for Enterprise SaaS Platform
 * @module @harborgrid/enterprise-shared/types/api
 */

import { z } from 'zod';
import type { ApiResponse, PaginationParams } from './common';

// ============================================================================
// HTTP Types
// ============================================================================

export type HTTPMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS';

export interface HTTPHeaders {
  [key: string]: string | string[] | undefined;
}

export interface HTTPRequest {
  method: HTTPMethod;
  path: string;
  headers: HTTPHeaders;
  query: Record<string, string | string[]>;
  body?: unknown;
  params?: Record<string, string>;
}

export interface HTTPResponse<T = unknown> {
  statusCode: number;
  headers: HTTPHeaders;
  body: ApiResponse<T>;
}

// ============================================================================
// REST API Versioning
// ============================================================================

export interface APIVersion {
  version: string; // e.g., "1.0.0"
  path: string; // e.g., "/v1"
  status: APIVersionStatus;
  deprecatedAt?: Date;
  sunsetAt?: Date;
  migrationGuide?: string;
}

export enum APIVersionStatus {
  ACTIVE = 'active',
  DEPRECATED = 'deprecated',
  SUNSET = 'sunset',
}

export const APIVersionHeaders = {
  VERSION: 'X-API-Version',
  DEPRECATION: 'Deprecation',
  SUNSET: 'Sunset',
  LINK: 'Link',
} as const;

// ============================================================================
// Request/Response Middleware
// ============================================================================

export interface RequestContext {
  requestId: string;
  correlationId?: string;
  tenantId?: string;
  userId?: string;
  userAgent?: string;
  ipAddress?: string;
  startTime: Date;
  metadata: Record<string, unknown>;
}

export interface ResponseContext {
  statusCode: number;
  duration: number;
  cached: boolean;
  compressed: boolean;
  size: number;
}

// ============================================================================
// Rate Limiting
// ============================================================================

export interface RateLimitInfo {
  limit: number;
  remaining: number;
  reset: number; // Unix timestamp
  retryAfter?: number; // Seconds
}

export const RateLimitHeaders = {
  LIMIT: 'X-RateLimit-Limit',
  REMAINING: 'X-RateLimit-Remaining',
  RESET: 'X-RateLimit-Reset',
  RETRY_AFTER: 'Retry-After',
} as const;

// ============================================================================
// CORS Configuration
// ============================================================================

export interface CORSConfig {
  enabled: boolean;
  origins: string[];
  methods: HTTPMethod[];
  headers: string[];
  exposedHeaders: string[];
  credentials: boolean;
  maxAge?: number;
}

export const defaultCORSConfig: CORSConfig = {
  enabled: true,
  origins: ['*'],
  methods: ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS'],
  headers: ['Content-Type', 'Authorization', 'X-API-Key'],
  exposedHeaders: ['X-Request-Id', 'X-RateLimit-Remaining'],
  credentials: true,
  maxAge: 86400,
};

// ============================================================================
// Content Negotiation
// ============================================================================

export enum ContentType {
  JSON = 'application/json',
  XML = 'application/xml',
  CSV = 'text/csv',
  HTML = 'text/html',
  PLAIN = 'text/plain',
  FORM = 'application/x-www-form-urlencoded',
  MULTIPART = 'multipart/form-data',
  OCTET_STREAM = 'application/octet-stream',
}

export interface ContentNegotiation {
  accept: ContentType[];
  contentType?: ContentType;
  charset?: string;
  encoding?: string;
}

// ============================================================================
// API Error Codes
// ============================================================================

export enum APIErrorCode {
  // Client Errors (4xx)
  BAD_REQUEST = 'BAD_REQUEST',
  UNAUTHORIZED = 'UNAUTHORIZED',
  FORBIDDEN = 'FORBIDDEN',
  NOT_FOUND = 'NOT_FOUND',
  METHOD_NOT_ALLOWED = 'METHOD_NOT_ALLOWED',
  CONFLICT = 'CONFLICT',
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  RATE_LIMIT_EXCEEDED = 'RATE_LIMIT_EXCEEDED',
  PAYLOAD_TOO_LARGE = 'PAYLOAD_TOO_LARGE',

  // Server Errors (5xx)
  INTERNAL_SERVER_ERROR = 'INTERNAL_SERVER_ERROR',
  NOT_IMPLEMENTED = 'NOT_IMPLEMENTED',
  BAD_GATEWAY = 'BAD_GATEWAY',
  SERVICE_UNAVAILABLE = 'SERVICE_UNAVAILABLE',
  GATEWAY_TIMEOUT = 'GATEWAY_TIMEOUT',

  // Business Logic Errors
  INSUFFICIENT_CREDITS = 'INSUFFICIENT_CREDITS',
  QUOTA_EXCEEDED = 'QUOTA_EXCEEDED',
  SUBSCRIPTION_REQUIRED = 'SUBSCRIPTION_REQUIRED',
  FEATURE_NOT_AVAILABLE = 'FEATURE_NOT_AVAILABLE',
  DUPLICATE_RESOURCE = 'DUPLICATE_RESOURCE',
}

export const HTTPStatusCodes: Record<APIErrorCode, number> = {
  [APIErrorCode.BAD_REQUEST]: 400,
  [APIErrorCode.UNAUTHORIZED]: 401,
  [APIErrorCode.FORBIDDEN]: 403,
  [APIErrorCode.NOT_FOUND]: 404,
  [APIErrorCode.METHOD_NOT_ALLOWED]: 405,
  [APIErrorCode.CONFLICT]: 409,
  [APIErrorCode.VALIDATION_ERROR]: 422,
  [APIErrorCode.RATE_LIMIT_EXCEEDED]: 429,
  [APIErrorCode.PAYLOAD_TOO_LARGE]: 413,
  [APIErrorCode.INTERNAL_SERVER_ERROR]: 500,
  [APIErrorCode.NOT_IMPLEMENTED]: 501,
  [APIErrorCode.BAD_GATEWAY]: 502,
  [APIErrorCode.SERVICE_UNAVAILABLE]: 503,
  [APIErrorCode.GATEWAY_TIMEOUT]: 504,
  [APIErrorCode.INSUFFICIENT_CREDITS]: 402,
  [APIErrorCode.QUOTA_EXCEEDED]: 429,
  [APIErrorCode.SUBSCRIPTION_REQUIRED]: 402,
  [APIErrorCode.FEATURE_NOT_AVAILABLE]: 403,
  [APIErrorCode.DUPLICATE_RESOURCE]: 409,
};

// ============================================================================
// Query Parameters
// ============================================================================

export interface QueryParams extends PaginationParams {
  search?: string;
  filter?: Record<string, unknown>;
  include?: string[];
  exclude?: string[];
  fields?: string[];
}

export const QueryParamsSchema = z.object({
  page: z.coerce.number().int().positive().default(1),
  limit: z.coerce.number().int().positive().max(100).default(20),
  sortBy: z.string().optional(),
  sortOrder: z.enum(['asc', 'desc']).optional(),
  search: z.string().optional(),
  filter: z.record(z.unknown()).optional(),
  include: z.array(z.string()).or(z.string()).optional(),
  exclude: z.array(z.string()).or(z.string()).optional(),
  fields: z.array(z.string()).or(z.string()).optional(),
});

// ============================================================================
// Webhooks
// ============================================================================

export interface WebhookConfig {
  url: string;
  events: string[];
  secret?: string;
  headers?: Record<string, string>;
  enabled: boolean;
  retryConfig?: WebhookRetryConfig;
}

export interface WebhookRetryConfig {
  maxRetries: number;
  retryDelay: number; // milliseconds
  backoffMultiplier: number;
  maxDelay: number; // milliseconds
}

export interface WebhookPayload<T = unknown> {
  id: string;
  event: string;
  timestamp: Date;
  data: T;
  signature: string;
}

export interface WebhookDelivery {
  id: string;
  webhookId: string;
  event: string;
  url: string;
  attempt: number;
  statusCode?: number;
  response?: string;
  error?: string;
  deliveredAt?: Date;
  createdAt: Date;
}

// ============================================================================
// API Documentation
// ============================================================================

export interface APIEndpoint {
  method: HTTPMethod;
  path: string;
  summary: string;
  description?: string;
  tags: string[];
  parameters?: APIParameter[];
  requestBody?: APIRequestBody;
  responses: Record<number, APIResponse>;
  security?: APISecurity[];
  deprecated?: boolean;
}

export interface APIParameter {
  name: string;
  in: 'path' | 'query' | 'header' | 'cookie';
  description?: string;
  required: boolean;
  schema: unknown; // JSON Schema
  example?: unknown;
}

export interface APIRequestBody {
  description?: string;
  required: boolean;
  content: Record<ContentType, APIMediaType>;
}

export interface APIMediaType {
  schema: unknown; // JSON Schema
  example?: unknown;
  examples?: Record<string, unknown>;
}

export interface APIResponse {
  description: string;
  content?: Record<ContentType, APIMediaType>;
  headers?: Record<string, APIHeader>;
}

export interface APIHeader {
  description?: string;
  schema: unknown; // JSON Schema
  required?: boolean;
}

export interface APISecurity {
  type: 'apiKey' | 'http' | 'oauth2' | 'openIdConnect';
  name?: string;
  in?: 'header' | 'query' | 'cookie';
  scheme?: string;
  bearerFormat?: string;
}

// ============================================================================
// Batch Operations
// ============================================================================

export interface BatchRequest<T = unknown> {
  operations: BatchOperation<T>[];
  atomic?: boolean; // All or nothing
  continueOnError?: boolean;
}

export interface BatchOperation<T = unknown> {
  id: string;
  method: HTTPMethod;
  path: string;
  body?: T;
  headers?: HTTPHeaders;
}

export interface BatchResponse<T = unknown> {
  results: BatchResult<T>[];
  errors: number;
  successes: number;
}

export interface BatchResult<T = unknown> {
  id: string;
  statusCode: number;
  body?: T;
  error?: string;
}

// ============================================================================
// Caching
// ============================================================================

export interface CacheControl {
  maxAge?: number; // seconds
  sMaxAge?: number; // seconds
  public?: boolean;
  private?: boolean;
  noCache?: boolean;
  noStore?: boolean;
  mustRevalidate?: boolean;
  proxyRevalidate?: boolean;
  staleWhileRevalidate?: number; // seconds
  staleIfError?: number; // seconds
}

export function buildCacheControlHeader(config: CacheControl): string {
  const directives: string[] = [];

  if (config.maxAge !== undefined) {
    directives.push(`max-age=${config.maxAge}`);
  }
  if (config.sMaxAge !== undefined) {
    directives.push(`s-maxage=${config.sMaxAge}`);
  }
  if (config.public) {
    directives.push('public');
  }
  if (config.private) {
    directives.push('private');
  }
  if (config.noCache) {
    directives.push('no-cache');
  }
  if (config.noStore) {
    directives.push('no-store');
  }
  if (config.mustRevalidate) {
    directives.push('must-revalidate');
  }
  if (config.proxyRevalidate) {
    directives.push('proxy-revalidate');
  }
  if (config.staleWhileRevalidate !== undefined) {
    directives.push(`stale-while-revalidate=${config.staleWhileRevalidate}`);
  }
  if (config.staleIfError !== undefined) {
    directives.push(`stale-if-error=${config.staleIfError}`);
  }

  return directives.join(', ');
}

// ============================================================================
// ETag & Conditional Requests
// ============================================================================

export interface ETagConfig {
  etag: string;
  weak?: boolean;
}

export interface ConditionalRequest {
  ifMatch?: string;
  ifNoneMatch?: string;
  ifModifiedSince?: Date;
  ifUnmodifiedSince?: Date;
}

export function generateETag(data: unknown, weak: boolean = false): string {
  // Simple hash-based ETag generation
  const hash = JSON.stringify(data);
  const etag = Buffer.from(hash).toString('base64');
  return weak ? `W/"${etag}"` : `"${etag}"`;
}

// ============================================================================
// Export all types
// ============================================================================

export type {
  HTTPMethod,
  HTTPHeaders,
  HTTPRequest,
  HTTPResponse,
  APIVersion,
  RequestContext,
  ResponseContext,
  RateLimitInfo,
  CORSConfig,
  ContentNegotiation,
  QueryParams,
  WebhookConfig,
  WebhookRetryConfig,
  WebhookPayload,
  WebhookDelivery,
  APIEndpoint,
  APIParameter,
  APIRequestBody,
  APIResponse,
  BatchRequest,
  BatchOperation,
  BatchResponse,
  BatchResult,
  CacheControl,
  ETagConfig,
  ConditionalRequest,
};
