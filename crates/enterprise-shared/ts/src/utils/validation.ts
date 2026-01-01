/**
 * Validation Utilities for Enterprise SaaS Platform
 * @module @harborgrid/enterprise-shared/utils/validation
 */

import { z, ZodError, ZodSchema } from 'zod';
import { ValidationError } from '../types/common';

// ============================================================================
// Validation Result
// ============================================================================

export interface ValidationResult<T = unknown> {
  success: boolean;
  data?: T;
  errors?: ValidationErrorDetail[];
}

export interface ValidationErrorDetail {
  field: string;
  message: string;
  code: string;
  value?: unknown;
}

// ============================================================================
// Validation Functions
// ============================================================================

/**
 * Validate data against a Zod schema
 */
export function validate<T>(
  schema: ZodSchema<T>,
  data: unknown
): ValidationResult<T> {
  try {
    const validated = schema.parse(data);
    return {
      success: true,
      data: validated,
    };
  } catch (error) {
    if (error instanceof ZodError) {
      return {
        success: false,
        errors: error.errors.map((err) => ({
          field: err.path.join('.'),
          message: err.message,
          code: err.code,
          value: err.path.length > 0 ? getNestedValue(data, err.path) : data,
        })),
      };
    }
    throw error;
  }
}

/**
 * Validate data and throw ValidationError on failure
 */
export function validateOrThrow<T>(schema: ZodSchema<T>, data: unknown): T {
  const result = validate(schema, data);
  if (!result.success) {
    throw new ValidationError('Validation failed', {
      errors: result.errors,
    });
  }
  return result.data!;
}

/**
 * Get nested value from object using path
 */
function getNestedValue(obj: any, path: (string | number)[]): unknown {
  return path.reduce((current, key) => current?.[key], obj);
}

// ============================================================================
// Common Validators
// ============================================================================

/**
 * Email validation
 */
export const emailValidator = z.string().email();

/**
 * UUID validation
 */
export const uuidValidator = z.string().uuid();

/**
 * URL validation
 */
export const urlValidator = z.string().url();

/**
 * Phone number validation (simple)
 */
export const phoneValidator = z.string().regex(/^\+?[1-9]\d{1,14}$/);

/**
 * Strong password validation
 */
export const strongPasswordValidator = z
  .string()
  .min(8)
  .regex(/[a-z]/, 'Must contain lowercase letter')
  .regex(/[A-Z]/, 'Must contain uppercase letter')
  .regex(/[0-9]/, 'Must contain number')
  .regex(/[^a-zA-Z0-9]/, 'Must contain special character');

/**
 * Slug validation (URL-safe string)
 */
export const slugValidator = z.string().regex(/^[a-z0-9-]+$/);

/**
 * Color hex validation
 */
export const colorHexValidator = z.string().regex(/^#[0-9a-fA-F]{6}$/);

/**
 * Semantic version validation
 */
export const semverValidator = z
  .string()
  .regex(/^\d+\.\d+\.\d+(-[a-zA-Z0-9-]+)?(\+[a-zA-Z0-9-]+)?$/);

/**
 * ISO 8601 date string validation
 */
export const iso8601Validator = z.string().datetime();

/**
 * JSON validation
 */
export const jsonValidator = z.string().transform((str, ctx) => {
  try {
    return JSON.parse(str);
  } catch {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'Invalid JSON',
    });
    return z.NEVER;
  }
});

// ============================================================================
// Custom Validators
// ============================================================================

/**
 * Validate tenant ID belongs to current user
 */
export function createTenantValidator(allowedTenantIds: string[]) {
  return z.string().uuid().refine(
    (tenantId) => allowedTenantIds.includes(tenantId),
    {
      message: 'Tenant not accessible',
    }
  );
}

/**
 * Validate file size
 */
export function createFileSizeValidator(maxBytes: number) {
  return z.number().max(maxBytes, {
    message: `File size must not exceed ${formatBytes(maxBytes)}`,
  });
}

/**
 * Validate file type
 */
export function createFileTypeValidator(allowedTypes: string[]) {
  return z.string().refine(
    (mimeType) => allowedTypes.includes(mimeType),
    {
      message: `File type must be one of: ${allowedTypes.join(', ')}`,
    }
  );
}

/**
 * Validate date range
 */
export function createDateRangeValidator(maxDays: number) {
  return z
    .object({
      start: z.date(),
      end: z.date(),
    })
    .refine(
      (range) => {
        const diffMs = range.end.getTime() - range.start.getTime();
        const diffDays = diffMs / (1000 * 60 * 60 * 24);
        return diffDays <= maxDays;
      },
      {
        message: `Date range must not exceed ${maxDays} days`,
      }
    );
}

/**
 * Validate array length
 */
export function createArrayLengthValidator<T>(
  min: number,
  max: number,
  itemSchema?: ZodSchema<T>
) {
  let schema = z.array(itemSchema || z.unknown()).min(min).max(max);
  return schema;
}

/**
 * Validate enum value
 */
export function createEnumValidator<T extends string>(values: readonly T[]) {
  return z.enum(values as [T, ...T[]]);
}

// ============================================================================
// Sanitization
// ============================================================================

/**
 * Sanitize HTML string (basic XSS prevention)
 */
export function sanitizeHTML(html: string): string {
  return html
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;')
    .replace(/\//g, '&#x2F;');
}

/**
 * Sanitize SQL string (basic SQL injection prevention)
 */
export function sanitizeSQL(sql: string): string {
  return sql.replace(/['";\\]/g, '');
}

/**
 * Sanitize filename
 */
export function sanitizeFilename(filename: string): string {
  return filename
    .replace(/[^a-zA-Z0-9.-]/g, '_')
    .replace(/_{2,}/g, '_')
    .replace(/^_+|_+$/g, '');
}

/**
 * Sanitize slug
 */
export function sanitizeSlug(slug: string): string {
  return slug
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9-]/g, '-')
    .replace(/-{2,}/g, '-')
    .replace(/^-+|-+$/g, '');
}

// ============================================================================
// Format Utilities
// ============================================================================

/**
 * Format bytes to human-readable string
 */
export function formatBytes(bytes: number, decimals: number = 2): string {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB'];

  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

/**
 * Format duration to human-readable string
 */
export function formatDuration(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days}d ${hours % 24}h`;
  if (hours > 0) return `${hours}h ${minutes % 60}m`;
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
  if (seconds > 0) return `${seconds}s`;
  return `${milliseconds}ms`;
}

// ============================================================================
// Type Guards
// ============================================================================

/**
 * Check if value is a valid UUID
 */
export function isUUID(value: unknown): value is string {
  if (typeof value !== 'string') return false;
  const uuidRegex =
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
  return uuidRegex.test(value);
}

/**
 * Check if value is a valid email
 */
export function isEmail(value: unknown): value is string {
  if (typeof value !== 'string') return false;
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(value);
}

/**
 * Check if value is a valid URL
 */
export function isURL(value: unknown): value is string {
  if (typeof value !== 'string') return false;
  try {
    new URL(value);
    return true;
  } catch {
    return false;
  }
}

/**
 * Check if value is a valid date
 */
export function isValidDate(value: unknown): value is Date {
  return value instanceof Date && !isNaN(value.getTime());
}

/**
 * Check if value is a plain object
 */
export function isPlainObject(value: unknown): value is Record<string, unknown> {
  return (
    typeof value === 'object' &&
    value !== null &&
    !Array.isArray(value) &&
    !(value instanceof Date)
  );
}

// ============================================================================
// Assertion Functions
// ============================================================================

/**
 * Assert value is defined (not null or undefined)
 */
export function assertDefined<T>(
  value: T | null | undefined,
  message: string = 'Value must be defined'
): asserts value is T {
  if (value === null || value === undefined) {
    throw new ValidationError(message);
  }
}

/**
 * Assert value is non-empty string
 */
export function assertNonEmptyString(
  value: unknown,
  message: string = 'Value must be a non-empty string'
): asserts value is string {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new ValidationError(message);
  }
}

/**
 * Assert value is positive number
 */
export function assertPositiveNumber(
  value: unknown,
  message: string = 'Value must be a positive number'
): asserts value is number {
  if (typeof value !== 'number' || value <= 0 || isNaN(value)) {
    throw new ValidationError(message);
  }
}

/**
 * Assert array has items
 */
export function assertNonEmptyArray<T>(
  value: unknown,
  message: string = 'Array must not be empty'
): asserts value is T[] {
  if (!Array.isArray(value) || value.length === 0) {
    throw new ValidationError(message);
  }
}

// ============================================================================
// Export all validators and utilities
// ============================================================================

export {
  // Zod re-exports
  z,
  type ZodSchema,
  type ZodError,
};
