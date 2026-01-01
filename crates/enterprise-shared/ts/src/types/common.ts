/**
 * Common Shared Types for Enterprise SaaS Platform
 * @module @harborgrid/enterprise-shared/types/common
 */

import { z } from 'zod';

// ============================================================================
// Multi-Tenant Types
// ============================================================================

export interface Tenant {
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

export enum TenantStatus {
  ACTIVE = 'active',
  SUSPENDED = 'suspended',
  TRIAL = 'trial',
  CHURNED = 'churned',
}

export const TenantSchema = z.object({
  id: z.string().uuid(),
  name: z.string().min(1),
  slug: z.string().regex(/^[a-z0-9-]+$/),
  status: z.nativeEnum(TenantStatus),
  plan: z.string(),
  metadata: z.record(z.unknown()),
  createdAt: z.date(),
  updatedAt: z.date(),
  deletedAt: z.date().optional(),
});

// ============================================================================
// User & Identity Types
// ============================================================================

export interface User {
  id: string;
  tenantId: string;
  email: string;
  username?: string;
  firstName?: string;
  lastName?: string;
  displayName: string;
  avatarUrl?: string;
  role: UserRole;
  status: UserStatus;
  metadata: Record<string, unknown>;
  lastLoginAt?: Date;
  createdAt: Date;
  updatedAt: Date;
}

export enum UserRole {
  SUPER_ADMIN = 'super_admin',
  ADMIN = 'admin',
  MANAGER = 'manager',
  MEMBER = 'member',
  VIEWER = 'viewer',
  GUEST = 'guest',
}

export enum UserStatus {
  ACTIVE = 'active',
  INACTIVE = 'inactive',
  SUSPENDED = 'suspended',
  PENDING = 'pending',
}

export const UserSchema = z.object({
  id: z.string().uuid(),
  tenantId: z.string().uuid(),
  email: z.string().email(),
  username: z.string().optional(),
  firstName: z.string().optional(),
  lastName: z.string().optional(),
  displayName: z.string().min(1),
  avatarUrl: z.string().url().optional(),
  role: z.nativeEnum(UserRole),
  status: z.nativeEnum(UserStatus),
  metadata: z.record(z.unknown()),
  lastLoginAt: z.date().optional(),
  createdAt: z.date(),
  updatedAt: z.date(),
});

// ============================================================================
// Pagination Types
// ============================================================================

export interface PaginationParams {
  page: number;
  limit: number;
  offset?: number;
  sortBy?: string;
  sortOrder?: SortOrder;
}

export enum SortOrder {
  ASC = 'asc',
  DESC = 'desc',
}

export interface PaginatedResult<T> {
  data: T[];
  pagination: {
    total: number;
    page: number;
    limit: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
}

export const PaginationParamsSchema = z.object({
  page: z.number().int().positive().default(1),
  limit: z.number().int().positive().max(100).default(20),
  offset: z.number().int().nonnegative().optional(),
  sortBy: z.string().optional(),
  sortOrder: z.nativeEnum(SortOrder).optional(),
});

// ============================================================================
// API Response Types
// ============================================================================

export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: ApiError;
  metadata?: ApiMetadata;
}

export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
  stack?: string;
  requestId?: string;
}

export interface ApiMetadata {
  timestamp: Date;
  requestId: string;
  version: string;
  duration?: number;
  path?: string;
  method?: string;
}

export const ApiErrorSchema = z.object({
  code: z.string(),
  message: z.string(),
  details: z.record(z.unknown()).optional(),
  stack: z.string().optional(),
  requestId: z.string().optional(),
});

// ============================================================================
// Error Types
// ============================================================================

export class EnterpriseError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode: number = 500,
    public details?: Record<string, unknown>
  ) {
    super(message);
    this.name = 'EnterpriseError';
    Object.setPrototypeOf(this, EnterpriseError.prototype);
  }

  toJSON(): ApiError {
    return {
      code: this.code,
      message: this.message,
      details: this.details,
    };
  }
}

export class ValidationError extends EnterpriseError {
  constructor(message: string, details?: Record<string, unknown>) {
    super(message, 'VALIDATION_ERROR', 400, details);
    this.name = 'ValidationError';
  }
}

export class NotFoundError extends EnterpriseError {
  constructor(resource: string, id: string) {
    super(`${resource} with id '${id}' not found`, 'NOT_FOUND', 404, {
      resource,
      id,
    });
    this.name = 'NotFoundError';
  }
}

export class UnauthorizedError extends EnterpriseError {
  constructor(message: string = 'Unauthorized') {
    super(message, 'UNAUTHORIZED', 401);
    this.name = 'UnauthorizedError';
  }
}

export class ForbiddenError extends EnterpriseError {
  constructor(message: string = 'Forbidden') {
    super(message, 'FORBIDDEN', 403);
    this.name = 'ForbiddenError';
  }
}

export class ConflictError extends EnterpriseError {
  constructor(message: string, details?: Record<string, unknown>) {
    super(message, 'CONFLICT', 409, details);
    this.name = 'ConflictError';
  }
}

export class RateLimitError extends EnterpriseError {
  constructor(message: string, public retryAfter: number) {
    super(message, 'RATE_LIMIT_EXCEEDED', 429, { retryAfter });
    this.name = 'RateLimitError';
  }
}

// ============================================================================
// Audit & Logging Types
// ============================================================================

export enum LogLevel {
  DEBUG = 'debug',
  INFO = 'info',
  WARN = 'warn',
  ERROR = 'error',
  FATAL = 'fatal',
}

export interface LogEntry {
  level: LogLevel;
  message: string;
  timestamp: Date;
  service: string;
  tenantId?: string;
  userId?: string;
  requestId?: string;
  correlationId?: string;
  metadata?: Record<string, unknown>;
  error?: Error;
}

export interface AuditLog {
  id: string;
  tenantId: string;
  userId?: string;
  action: string;
  resource: string;
  resourceId?: string;
  result: AuditResult;
  ipAddress?: string;
  userAgent?: string;
  details?: Record<string, unknown>;
  timestamp: Date;
}

export enum AuditResult {
  SUCCESS = 'success',
  FAILURE = 'failure',
  PARTIAL = 'partial',
}

// ============================================================================
// Time & Date Types
// ============================================================================

export interface TimeRange {
  start: Date;
  end: Date;
  timezone?: string;
}

export interface Schedule {
  cron: string;
  timezone?: string;
  enabled: boolean;
  nextRun?: Date;
  lastRun?: Date;
}

export const TimeRangeSchema = z.object({
  start: z.date(),
  end: z.date(),
  timezone: z.string().optional(),
});

// ============================================================================
// Feature Flag Types
// ============================================================================

export interface FeatureFlag {
  key: string;
  enabled: boolean;
  description?: string;
  tenants?: string[];
  users?: string[];
  percentage?: number;
  metadata?: Record<string, unknown>;
}

export interface FeatureFlagContext {
  tenantId?: string;
  userId?: string;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// Health Check Types
// ============================================================================

export interface HealthCheck {
  status: HealthStatus;
  timestamp: Date;
  version: string;
  uptime: number;
  checks: HealthCheckItem[];
}

export enum HealthStatus {
  HEALTHY = 'healthy',
  DEGRADED = 'degraded',
  UNHEALTHY = 'unhealthy',
}

export interface HealthCheckItem {
  name: string;
  status: HealthStatus;
  message?: string;
  duration?: number;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// Metrics Types
// ============================================================================

export interface Metric {
  name: string;
  value: number;
  unit: MetricUnit;
  timestamp: Date;
  tags?: Record<string, string>;
}

export enum MetricUnit {
  COUNT = 'count',
  BYTES = 'bytes',
  MILLISECONDS = 'milliseconds',
  PERCENTAGE = 'percentage',
  REQUESTS_PER_SECOND = 'rps',
}

export interface TimeSeries {
  metric: string;
  points: TimeSeriesPoint[];
  tags?: Record<string, string>;
}

export interface TimeSeriesPoint {
  timestamp: Date;
  value: number;
}

// ============================================================================
// Configuration Types
// ============================================================================

export interface AppConfig {
  environment: Environment;
  service: {
    name: string;
    version: string;
    port: number;
    host: string;
  };
  database?: DatabaseConfig;
  redis?: RedisConfig;
  logging?: LoggingConfig;
  features?: Record<string, boolean>;
}

export enum Environment {
  DEVELOPMENT = 'development',
  STAGING = 'staging',
  PRODUCTION = 'production',
  TEST = 'test',
}

export interface DatabaseConfig {
  host: string;
  port: number;
  database: string;
  username: string;
  password: string;
  ssl?: boolean;
  poolSize?: number;
  timeout?: number;
}

export interface RedisConfig {
  host: string;
  port: number;
  password?: string;
  db?: number;
  keyPrefix?: string;
  tls?: boolean;
}

export interface LoggingConfig {
  level: LogLevel;
  format: 'json' | 'text';
  outputs: ('stdout' | 'file' | 'external')[];
}

// ============================================================================
// File & Upload Types
// ============================================================================

export interface FileMetadata {
  id: string;
  filename: string;
  originalFilename: string;
  mimeType: string;
  size: number;
  path: string;
  url?: string;
  uploadedBy: string;
  tenantId: string;
  metadata?: Record<string, unknown>;
  createdAt: Date;
}

export interface UploadProgress {
  loaded: number;
  total: number;
  percentage: number;
  bytesPerSecond: number;
  estimatedTimeRemaining: number;
}

// ============================================================================
// Utility Types
// ============================================================================

export type UUID = string;
export type ISODateString = string;
export type Timestamp = number;

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export type RequireAtLeastOne<T, Keys extends keyof T = keyof T> = Pick<
  T,
  Exclude<keyof T, Keys>
> &
  {
    [K in Keys]-?: Required<Pick<T, K>> & Partial<Pick<T, Exclude<Keys, K>>>;
  }[Keys];

export type Nullable<T> = T | null;
export type Optional<T> = T | undefined;

// ============================================================================
// Export all types
// ============================================================================

export type {
  Tenant,
  User,
  PaginationParams,
  PaginatedResult,
  ApiResponse,
  ApiError,
  ApiMetadata,
  LogEntry,
  AuditLog,
  TimeRange,
  Schedule,
  FeatureFlag,
  FeatureFlagContext,
  HealthCheck,
  HealthCheckItem,
  Metric,
  TimeSeries,
  TimeSeriesPoint,
  AppConfig,
  DatabaseConfig,
  RedisConfig,
  LoggingConfig,
  FileMetadata,
  UploadProgress,
};
