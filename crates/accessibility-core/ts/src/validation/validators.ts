/**
 * Validation functions
 * @module validation/validators
 */

import { z, ZodError } from 'zod';
import { ValidationError } from '../errors/ValidationError';
import type { ValidationErrorDetail } from '../types';
import * as schemas from './schemas';

/**
 * Validation result
 */
export type ValidationResult<T> =
  | { success: true; data: T }
  | { success: false; error: ValidationError };

/**
 * Validate data against schema
 */
export function validate<T>(
  schema: z.ZodType<T>,
  data: unknown
): ValidationResult<T> {
  try {
    const validated = schema.parse(data);
    return { success: true, data: validated };
  } catch (error) {
    if (error instanceof ZodError) {
      return {
        success: false,
        error: convertZodError(error),
      };
    }
    throw error;
  }
}

/**
 * Validate data against schema (throws on error)
 */
export function validateOrThrow<T>(schema: z.ZodType<T>, data: unknown): T {
  try {
    return schema.parse(data);
  } catch (error) {
    if (error instanceof ZodError) {
      throw convertZodError(error);
    }
    throw error;
  }
}

/**
 * Validate data against schema asynchronously
 */
export async function validateAsync<T>(
  schema: z.ZodType<T>,
  data: unknown
): Promise<ValidationResult<T>> {
  try {
    const validated = await schema.parseAsync(data);
    return { success: true, data: validated };
  } catch (error) {
    if (error instanceof ZodError) {
      return {
        success: false,
        error: convertZodError(error),
      };
    }
    throw error;
  }
}

/**
 * Convert ZodError to ValidationError
 */
function convertZodError(error: ZodError): ValidationError {
  const details: ValidationErrorDetail[] = error.errors.map((err) => ({
    field: err.path.join('.') || 'unknown',
    message: err.message,
    rule: err.code,
    value: err.path.length > 0 ? getNestedValue(error, err.path) : undefined,
  }));

  return ValidationError.forFields(details);
}

/**
 * Get nested value from object by path
 */
function getNestedValue(obj: any, path: (string | number)[]): unknown {
  let current = obj;
  for (const key of path) {
    if (current && typeof current === 'object') {
      current = current[key];
    } else {
      return undefined;
    }
  }
  return current;
}

/**
 * Validate URL
 */
export function validateUrl(url: string): ValidationResult<string> {
  return validate(schemas.urlSchema, url);
}

/**
 * Validate email
 */
export function validateEmail(email: string): ValidationResult<string> {
  return validate(schemas.emailSchema, email);
}

/**
 * Validate WCAG level
 */
export function validateWcagLevel(level: string): ValidationResult<schemas.WCAGLevel> {
  return validate(schemas.wcagLevelSchema, level);
}

/**
 * Validate scan configuration
 */
export function validateScanConfig(config: unknown): ValidationResult<schemas.ScanConfig> {
  return validate(schemas.scanConfigSchema, config);
}

/**
 * Validate scan result
 */
export function validateScanResult(result: unknown): ValidationResult<schemas.ScanResult> {
  return validate(schemas.scanResultSchema, result);
}

/**
 * Validate accessibility violation
 */
export function validateAccessibilityViolation(
  violation: unknown
): ValidationResult<schemas.AccessibilityViolation> {
  return validate(schemas.accessibilityViolationSchema, violation);
}

/**
 * Check if value is valid URL
 */
export function isValidUrl(value: string): boolean {
  return validateUrl(value).success;
}

/**
 * Check if value is valid email
 */
export function isValidEmail(value: string): boolean {
  return validateEmail(value).success;
}

/**
 * Check if value is valid WCAG level
 */
export function isValidWcagLevel(value: string): value is schemas.WCAGLevel {
  return validateWcagLevel(value).success;
}

/**
 * Validate required field
 */
export function validateRequired<T>(
  value: T | null | undefined,
  fieldName: string
): T {
  if (value === null || value === undefined || value === '') {
    throw ValidationError.missingRequired(fieldName);
  }
  return value;
}

/**
 * Validate string length
 */
export function validateStringLength(
  value: string,
  fieldName: string,
  min?: number,
  max?: number
): string {
  if (min !== undefined && value.length < min) {
    throw ValidationError.invalidFormat(
      fieldName,
      `At least ${min} characters`,
      value
    );
  }

  if (max !== undefined && value.length > max) {
    throw ValidationError.invalidFormat(
      fieldName,
      `At most ${max} characters`,
      value
    );
  }

  return value;
}

/**
 * Validate number range
 */
export function validateNumberRange(
  value: number,
  fieldName: string,
  min?: number,
  max?: number
): number {
  if (min !== undefined && value < min) {
    throw ValidationError.outOfRange(fieldName, min, max ?? Infinity, value);
  }

  if (max !== undefined && value > max) {
    throw ValidationError.outOfRange(fieldName, min ?? -Infinity, max, value);
  }

  return value;
}

/**
 * Validate array length
 */
export function validateArrayLength<T>(
  value: T[],
  fieldName: string,
  min?: number,
  max?: number
): T[] {
  if (min !== undefined && value.length < min) {
    throw ValidationError.invalidFormat(
      fieldName,
      `At least ${min} items`,
      value
    );
  }

  if (max !== undefined && value.length > max) {
    throw ValidationError.invalidFormat(fieldName, `At most ${max} items`, value);
  }

  return value;
}

/**
 * Validate enum value
 */
export function validateEnum<T extends string>(
  value: string,
  fieldName: string,
  allowedValues: readonly T[]
): T {
  if (!allowedValues.includes(value as T)) {
    throw ValidationError.invalidFormat(
      fieldName,
      `One of: ${allowedValues.join(', ')}`,
      value
    );
  }
  return value as T;
}

/**
 * Validate pattern (regex)
 */
export function validatePattern(
  value: string,
  fieldName: string,
  pattern: RegExp,
  description?: string
): string {
  if (!pattern.test(value)) {
    throw ValidationError.invalidFormat(
      fieldName,
      description ?? `Must match pattern: ${pattern.source}`,
      value
    );
  }
  return value;
}

/**
 * Validate object has required keys
 */
export function validateRequiredKeys<T extends object>(
  value: T,
  requiredKeys: (keyof T)[]
): T {
  for (const key of requiredKeys) {
    if (!(key in value) || value[key] === undefined || value[key] === null) {
      throw ValidationError.missingRequired(String(key));
    }
  }
  return value;
}

/**
 * Validate custom condition
 */
export function validateCondition(
  condition: boolean,
  fieldName: string,
  message: string
): void {
  if (!condition) {
    throw ValidationError.forField(fieldName, message);
  }
}

/**
 * Combine multiple validators
 */
export function combine<T>(
  ...validators: Array<(value: T) => T>
): (value: T) => T {
  return (value: T) => {
    let result = value;
    for (const validator of validators) {
      result = validator(result);
    }
    return result;
  };
}

/**
 * Create custom validator
 */
export function createValidator<T>(
  schema: z.ZodType<T>
): (data: unknown) => ValidationResult<T> {
  return (data: unknown) => validate(schema, data);
}

/**
 * Safe parse with default value
 */
export function parseWithDefault<T>(
  schema: z.ZodType<T>,
  data: unknown,
  defaultValue: T
): T {
  const result = validate(schema, data);
  return result.success ? result.data : defaultValue;
}

/**
 * Partial validation (allows partial objects)
 */
export function validatePartial<T extends z.ZodObject<any>>(
  schema: T,
  data: unknown
): ValidationResult<Partial<z.infer<T>>> {
  return validate(schema.partial(), data);
}

/**
 * Deep partial validation
 */
export function validateDeepPartial<T extends z.ZodObject<any>>(
  schema: T,
  data: unknown
): ValidationResult<z.infer<T>> {
  return validate(schema.deepPartial() as z.ZodType<z.infer<T>>, data);
}
