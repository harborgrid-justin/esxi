/**
 * TypeScript type guards
 * @module utils/typeGuards
 */

import { AccessibilityError } from '../errors/AccessibilityError';
import { ValidationError } from '../errors/ValidationError';
import { ScanError } from '../errors/ScanError';
import { NetworkError } from '../errors/NetworkError';
import { AuthError } from '../errors/AuthError';
import { PermissionError } from '../errors/PermissionError';
import type {
  WCAGLevel,
  WCAGPrinciple,
  SeverityLevel,
  ErrorCategory,
  AccessibilityViolation,
  ScanResult,
  ErrorMetadata,
} from '../types';

/**
 * Check if value is a string
 */
export function isString(value: unknown): value is string {
  return typeof value === 'string';
}

/**
 * Check if value is a number
 */
export function isNumber(value: unknown): value is number {
  return typeof value === 'number' && !Number.isNaN(value);
}

/**
 * Check if value is a boolean
 */
export function isBoolean(value: unknown): value is boolean {
  return typeof value === 'boolean';
}

/**
 * Check if value is an object
 */
export function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/**
 * Check if value is an array
 */
export function isArray(value: unknown): value is unknown[] {
  return Array.isArray(value);
}

/**
 * Check if value is null
 */
export function isNull(value: unknown): value is null {
  return value === null;
}

/**
 * Check if value is undefined
 */
export function isUndefined(value: unknown): value is undefined {
  return value === undefined;
}

/**
 * Check if value is null or undefined
 */
export function isNullOrUndefined(value: unknown): value is null | undefined {
  return value === null || value === undefined;
}

/**
 * Check if value is a function
 */
export function isFunction(value: unknown): value is (...args: any[]) => any {
  return typeof value === 'function';
}

/**
 * Check if value is a Date
 */
export function isDate(value: unknown): value is Date {
  return value instanceof Date && !Number.isNaN(value.getTime());
}

/**
 * Check if value is an Error
 */
export function isError(value: unknown): value is Error {
  return value instanceof Error;
}

/**
 * Check if value is AccessibilityError
 */
export function isAccessibilityError(value: unknown): value is AccessibilityError {
  return value instanceof AccessibilityError;
}

/**
 * Check if value is ValidationError
 */
export function isValidationError(value: unknown): value is ValidationError {
  return value instanceof ValidationError;
}

/**
 * Check if value is ScanError
 */
export function isScanError(value: unknown): value is ScanError {
  return value instanceof ScanError;
}

/**
 * Check if value is NetworkError
 */
export function isNetworkError(value: unknown): value is NetworkError {
  return value instanceof NetworkError;
}

/**
 * Check if value is AuthError
 */
export function isAuthError(value: unknown): value is AuthError {
  return value instanceof AuthError;
}

/**
 * Check if value is PermissionError
 */
export function isPermissionError(value: unknown): value is PermissionError {
  return value instanceof PermissionError;
}

/**
 * Check if value is a valid WCAG level
 */
export function isWCAGLevel(value: unknown): value is WCAGLevel {
  return isString(value) && ['A', 'AA', 'AAA'].includes(value);
}

/**
 * Check if value is a valid WCAG principle
 */
export function isWCAGPrinciple(value: unknown): value is WCAGPrinciple {
  return (
    isString(value) &&
    ['perceivable', 'operable', 'understandable', 'robust'].includes(value)
  );
}

/**
 * Check if value is a valid severity level
 */
export function isSeverityLevel(value: unknown): value is SeverityLevel {
  return (
    isString(value) &&
    ['critical', 'serious', 'moderate', 'minor', 'info'].includes(value)
  );
}

/**
 * Check if value is a valid error category
 */
export function isErrorCategory(value: unknown): value is ErrorCategory {
  return (
    isString(value) &&
    [
      'validation',
      'scan',
      'network',
      'auth',
      'permission',
      'accessibility',
      'system',
      'unknown',
    ].includes(value)
  );
}

/**
 * Check if value is an AccessibilityViolation
 */
export function isAccessibilityViolation(
  value: unknown
): value is AccessibilityViolation {
  if (!isObject(value)) return false;

  return (
    isString(value.id) &&
    isObject(value.criterion) &&
    isSeverityLevel(value.severity) &&
    isString(value.message)
  );
}

/**
 * Check if value is a ScanResult
 */
export function isScanResult(value: unknown): value is ScanResult {
  if (!isObject(value)) return false;

  return (
    isString(value.scanId) &&
    isString(value.url) &&
    isDate(value.timestamp) &&
    isWCAGLevel(value.level) &&
    isArray(value.violations) &&
    isNumber(value.elementsTested) &&
    isNumber(value.duration) &&
    isBoolean(value.success)
  );
}

/**
 * Check if value is ErrorMetadata
 */
export function isErrorMetadata(value: unknown): value is ErrorMetadata {
  if (!isObject(value)) return false;

  return isDate(value.timestamp);
}

/**
 * Check if value is a Promise
 */
export function isPromise<T = unknown>(value: unknown): value is Promise<T> {
  return value instanceof Promise || (isObject(value) && isFunction(value.then));
}

/**
 * Check if value is a valid URL string
 */
export function isValidUrl(value: unknown): value is string {
  if (!isString(value)) return false;

  try {
    const url = new URL(value);
    return url.protocol === 'http:' || url.protocol === 'https:';
  } catch {
    return false;
  }
}

/**
 * Check if value is a valid email string
 */
export function isValidEmail(value: unknown): value is string {
  if (!isString(value)) return false;

  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(value);
}

/**
 * Check if value is a valid hex color
 */
export function isValidHexColor(value: unknown): value is string {
  if (!isString(value)) return false;

  return /^#[0-9A-Fa-f]{6}$/.test(value);
}

/**
 * Check if value is a valid CSS selector
 */
export function isValidCSSSelector(value: unknown): value is string {
  if (!isString(value)) return false;

  try {
    document.querySelector(value);
    return true;
  } catch {
    return false;
  }
}

/**
 * Check if value is a non-empty string
 */
export function isNonEmptyString(value: unknown): value is string {
  return isString(value) && value.length > 0;
}

/**
 * Check if value is a positive number
 */
export function isPositiveNumber(value: unknown): value is number {
  return isNumber(value) && value > 0;
}

/**
 * Check if value is a non-negative number
 */
export function isNonNegativeNumber(value: unknown): value is number {
  return isNumber(value) && value >= 0;
}

/**
 * Check if value is an integer
 */
export function isInteger(value: unknown): value is number {
  return isNumber(value) && Number.isInteger(value);
}

/**
 * Check if value is a positive integer
 */
export function isPositiveInteger(value: unknown): value is number {
  return isInteger(value) && value > 0;
}

/**
 * Check if value is a non-negative integer
 */
export function isNonNegativeInteger(value: unknown): value is number {
  return isInteger(value) && value >= 0;
}

/**
 * Check if value is in range
 */
export function isInRange(
  value: unknown,
  min: number,
  max: number
): value is number {
  return isNumber(value) && value >= min && value <= max;
}

/**
 * Check if value is a non-empty array
 */
export function isNonEmptyArray<T>(value: unknown): value is T[] {
  return isArray(value) && value.length > 0;
}

/**
 * Check if value has property
 */
export function hasProperty<K extends string>(
  value: unknown,
  property: K
): value is Record<K, unknown> {
  return isObject(value) && property in value;
}

/**
 * Check if value has all properties
 */
export function hasProperties<K extends string>(
  value: unknown,
  properties: K[]
): value is Record<K, unknown> {
  if (!isObject(value)) return false;
  return properties.every((prop) => prop in value);
}

/**
 * Assert value is defined (type narrowing)
 */
export function isDefined<T>(value: T | null | undefined): value is T {
  return value !== null && value !== undefined;
}

/**
 * Filter out null and undefined values
 */
export function filterDefined<T>(array: (T | null | undefined)[]): T[] {
  return array.filter(isDefined);
}

/**
 * Check if all values are defined
 */
export function areAllDefined<T>(values: (T | null | undefined)[]): values is T[] {
  return values.every(isDefined);
}

/**
 * Type guard factory for literal types
 */
export function isLiteral<T extends string | number | boolean>(
  expected: T
): (value: unknown) => value is T {
  return (value: unknown): value is T => value === expected;
}

/**
 * Type guard factory for union types
 */
export function isOneOf<T extends readonly (string | number | boolean)[]>(
  values: T
): (value: unknown) => value is T[number] {
  return (value: unknown): value is T[number] => values.includes(value as any);
}
