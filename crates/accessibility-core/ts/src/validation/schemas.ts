/**
 * Zod validation schemas
 * @module validation/schemas
 */

import { z } from 'zod';

/**
 * WCAG level schema
 */
export const wcagLevelSchema = z.enum(['A', 'AA', 'AAA'], {
  errorMap: () => ({ message: 'WCAG level must be A, AA, or AAA' }),
});

/**
 * WCAG principle schema
 */
export const wcagPrincipleSchema = z.enum([
  'perceivable',
  'operable',
  'understandable',
  'robust',
]);

/**
 * Severity level schema
 */
export const severityLevelSchema = z.enum([
  'critical',
  'serious',
  'moderate',
  'minor',
  'info',
]);

/**
 * URL schema
 */
export const urlSchema = z
  .string()
  .url({ message: 'Invalid URL format' })
  .refine(
    (url) => {
      try {
        const parsed = new URL(url);
        return ['http:', 'https:'].includes(parsed.protocol);
      } catch {
        return false;
      }
    },
    { message: 'URL must use HTTP or HTTPS protocol' }
  );

/**
 * Email schema
 */
export const emailSchema = z
  .string()
  .email({ message: 'Invalid email address' })
  .toLowerCase();

/**
 * CSS selector schema
 */
export const cssSelectorSchema = z
  .string()
  .min(1, 'CSS selector cannot be empty')
  .refine(
    (selector) => {
      try {
        document.querySelector(selector);
        return true;
      } catch {
        return false;
      }
    },
    { message: 'Invalid CSS selector' }
  );

/**
 * WCAG criterion schema
 */
export const wcagCriterionSchema = z.object({
  id: z.string().regex(/^\d+\.\d+\.\d+$/, 'Invalid WCAG criterion ID format'),
  name: z.string().min(1, 'Criterion name is required'),
  level: wcagLevelSchema,
  principle: wcagPrincipleSchema,
  description: z.string().min(1, 'Description is required'),
  url: urlSchema,
});

/**
 * Accessibility violation schema
 */
export const accessibilityViolationSchema = z.object({
  id: z.string().min(1, 'Violation ID is required'),
  criterion: wcagCriterionSchema,
  severity: severityLevelSchema,
  message: z.string().min(1, 'Violation message is required'),
  selector: z.string().optional(),
  html: z.string().optional(),
  suggestion: z.string().optional(),
  context: z.record(z.unknown()).optional(),
});

/**
 * Scan configuration schema
 */
export const scanConfigSchema = z.object({
  urls: z
    .array(urlSchema)
    .min(1, 'At least one URL is required')
    .max(100, 'Cannot scan more than 100 URLs at once'),
  level: wcagLevelSchema.default('AA'),
  includeRules: z.array(z.string()).optional(),
  excludeRules: z.array(z.string()).optional(),
  maxPages: z.number().int().positive().max(1000).optional(),
  timeout: z
    .number()
    .int()
    .positive()
    .max(300000)
    .default(30000)
    .describe('Timeout in milliseconds'),
  screenshots: z.boolean().default(false),
});

/**
 * Scan result schema
 */
export const scanResultSchema = z.object({
  scanId: z.string().min(1, 'Scan ID is required'),
  url: urlSchema,
  timestamp: z.date(),
  level: wcagLevelSchema,
  violations: z.array(accessibilityViolationSchema),
  elementsTested: z.number().int().nonnegative(),
  duration: z.number().positive(),
  success: z.boolean(),
  error: z.string().optional(),
});

/**
 * Error metadata schema
 */
export const errorMetadataSchema = z.object({
  errorId: z.string().optional(),
  timestamp: z.date(),
  userId: z.string().optional(),
  sessionId: z.string().optional(),
  requestId: z.string().optional(),
  url: z.string().optional(),
  userAgent: z.string().optional(),
  context: z.record(z.unknown()).optional(),
});

/**
 * Validation error detail schema
 */
export const validationErrorDetailSchema = z.object({
  field: z.string().min(1, 'Field name is required'),
  message: z.string().min(1, 'Error message is required'),
  rule: z.string().min(1, 'Validation rule is required'),
  value: z.unknown().optional(),
  expected: z.string().optional(),
});

/**
 * Retry configuration schema
 */
export const retryConfigSchema = z.object({
  maxRetries: z.number().int().nonnegative().max(10).default(3),
  initialDelay: z.number().positive().max(60000).default(1000),
  maxDelay: z.number().positive().max(300000).default(30000),
  backoffMultiplier: z.number().positive().max(10).default(2),
  exponentialBackoff: z.boolean().default(true),
});

/**
 * API response schema
 */
export const apiResponseSchema = <T extends z.ZodTypeAny>(dataSchema: T) =>
  z.object({
    data: dataSchema.optional(),
    error: z
      .object({
        message: z.string(),
        code: z.string(),
        details: z.unknown().optional(),
      })
      .optional(),
    meta: z
      .object({
        requestId: z.string(),
        timestamp: z.date(),
        version: z.string(),
      })
      .optional(),
  });

/**
 * Pagination schema
 */
export const paginationSchema = z.object({
  page: z.number().int().positive().default(1),
  pageSize: z.number().int().positive().max(100).default(20),
  total: z.number().int().nonnegative().optional(),
});

/**
 * Sort order schema
 */
export const sortOrderSchema = z.enum(['asc', 'desc']);

/**
 * Date range schema
 */
export const dateRangeSchema = z
  .object({
    start: z.date(),
    end: z.date(),
  })
  .refine((data) => data.end >= data.start, {
    message: 'End date must be after or equal to start date',
  });

/**
 * ID schema
 */
export const idSchema = z.string().min(1, 'ID is required');

/**
 * Positive integer schema
 */
export const positiveIntSchema = z.number().int().positive();

/**
 * Non-negative integer schema
 */
export const nonNegativeIntSchema = z.number().int().nonnegative();

/**
 * Percentage schema (0-100)
 */
export const percentageSchema = z.number().min(0).max(100);

/**
 * Color schema (hex)
 */
export const hexColorSchema = z
  .string()
  .regex(/^#[0-9A-Fa-f]{6}$/, 'Invalid hex color format');

/**
 * ISO date string schema
 */
export const isoDateStringSchema = z
  .string()
  .datetime({ message: 'Invalid ISO date string' });

/**
 * User agent schema
 */
export const userAgentSchema = z.string().min(1);

/**
 * HTTP status code schema
 */
export const httpStatusCodeSchema = z.union([
  z.literal(200),
  z.literal(201),
  z.literal(204),
  z.literal(400),
  z.literal(401),
  z.literal(403),
  z.literal(404),
  z.literal(409),
  z.literal(422),
  z.literal(429),
  z.literal(500),
  z.literal(502),
  z.literal(503),
  z.literal(504),
]);

/**
 * Export inferred types
 */
export type WCAGLevel = z.infer<typeof wcagLevelSchema>;
export type WCAGPrinciple = z.infer<typeof wcagPrincipleSchema>;
export type SeverityLevel = z.infer<typeof severityLevelSchema>;
export type WCAGCriterion = z.infer<typeof wcagCriterionSchema>;
export type AccessibilityViolation = z.infer<typeof accessibilityViolationSchema>;
export type ScanConfig = z.infer<typeof scanConfigSchema>;
export type ScanResult = z.infer<typeof scanResultSchema>;
export type ErrorMetadata = z.infer<typeof errorMetadataSchema>;
export type ValidationErrorDetail = z.infer<typeof validationErrorDetailSchema>;
export type RetryConfig = z.infer<typeof retryConfigSchema>;
export type Pagination = z.infer<typeof paginationSchema>;
export type SortOrder = z.infer<typeof sortOrderSchema>;
export type DateRange = z.infer<typeof dateRangeSchema>;
