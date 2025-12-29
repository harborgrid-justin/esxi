/**
 * Runtime assertions
 * @module utils/assertions
 */

import { ValidationError } from '../errors/ValidationError';
import { SystemErrorCodes } from '../constants/errorCodes';
import * as typeGuards from './typeGuards';
import type { WCAGLevel, SeverityLevel } from '../types';

/**
 * Assert value is truthy
 */
export function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

/**
 * Assert value is defined
 */
export function assertDefined<T>(
  value: T | null | undefined,
  fieldName: string = 'value'
): asserts value is T {
  if (value === null || value === undefined) {
    throw ValidationError.missingRequired(fieldName);
  }
}

/**
 * Assert value is a string
 */
export function assertString(
  value: unknown,
  fieldName: string = 'value'
): asserts value is string {
  if (!typeGuards.isString(value)) {
    throw ValidationError.invalidType(fieldName, 'string', typeof value);
  }
}

/**
 * Assert value is a number
 */
export function assertNumber(
  value: unknown,
  fieldName: string = 'value'
): asserts value is number {
  if (!typeGuards.isNumber(value)) {
    throw ValidationError.invalidType(fieldName, 'number', typeof value);
  }
}

/**
 * Assert value is a boolean
 */
export function assertBoolean(
  value: unknown,
  fieldName: string = 'value'
): asserts value is boolean {
  if (!typeGuards.isBoolean(value)) {
    throw ValidationError.invalidType(fieldName, 'boolean', typeof value);
  }
}

/**
 * Assert value is an object
 */
export function assertObject(
  value: unknown,
  fieldName: string = 'value'
): asserts value is Record<string, unknown> {
  if (!typeGuards.isObject(value)) {
    throw ValidationError.invalidType(fieldName, 'object', typeof value);
  }
}

/**
 * Assert value is an array
 */
export function assertArray(
  value: unknown,
  fieldName: string = 'value'
): asserts value is unknown[] {
  if (!typeGuards.isArray(value)) {
    throw ValidationError.invalidType(fieldName, 'array', typeof value);
  }
}

/**
 * Assert value is a function
 */
export function assertFunction(
  value: unknown,
  fieldName: string = 'value'
): asserts value is (...args: any[]) => any {
  if (!typeGuards.isFunction(value)) {
    throw ValidationError.invalidType(fieldName, 'function', typeof value);
  }
}

/**
 * Assert value is a Date
 */
export function assertDate(
  value: unknown,
  fieldName: string = 'value'
): asserts value is Date {
  if (!typeGuards.isDate(value)) {
    throw ValidationError.invalidType(fieldName, 'Date', typeof value);
  }
}

/**
 * Assert value is an Error
 */
export function assertError(
  value: unknown,
  fieldName: string = 'value'
): asserts value is Error {
  if (!typeGuards.isError(value)) {
    throw ValidationError.invalidType(fieldName, 'Error', typeof value);
  }
}

/**
 * Assert value is a valid WCAG level
 */
export function assertWCAGLevel(
  value: unknown,
  fieldName: string = 'level'
): asserts value is WCAGLevel {
  if (!typeGuards.isWCAGLevel(value)) {
    throw ValidationError.invalidWcagLevel(value);
  }
}

/**
 * Assert value is a valid severity level
 */
export function assertSeverityLevel(
  value: unknown,
  fieldName: string = 'severity'
): asserts value is SeverityLevel {
  if (!typeGuards.isSeverityLevel(value)) {
    throw ValidationError.invalidFormat(
      fieldName,
      'critical, serious, moderate, minor, or info',
      value
    );
  }
}

/**
 * Assert value is a valid URL
 */
export function assertValidUrl(
  value: unknown,
  fieldName: string = 'url'
): asserts value is string {
  if (!typeGuards.isValidUrl(value)) {
    throw ValidationError.invalidUrl(fieldName, String(value));
  }
}

/**
 * Assert value is a valid email
 */
export function assertValidEmail(
  value: unknown,
  fieldName: string = 'email'
): asserts value is string {
  if (!typeGuards.isValidEmail(value)) {
    throw ValidationError.invalidEmail(fieldName, String(value));
  }
}

/**
 * Assert value is a non-empty string
 */
export function assertNonEmptyString(
  value: unknown,
  fieldName: string = 'value'
): asserts value is string {
  assertString(value, fieldName);
  if (value.length === 0) {
    throw ValidationError.missingRequired(fieldName);
  }
}

/**
 * Assert string meets length requirements
 */
export function assertStringLength(
  value: string,
  fieldName: string,
  min?: number,
  max?: number
): asserts value is string {
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
}

/**
 * Assert value is a positive number
 */
export function assertPositiveNumber(
  value: unknown,
  fieldName: string = 'value'
): asserts value is number {
  assertNumber(value, fieldName);
  if (value <= 0) {
    throw ValidationError.outOfRange(fieldName, 0, Infinity, value);
  }
}

/**
 * Assert value is a non-negative number
 */
export function assertNonNegativeNumber(
  value: unknown,
  fieldName: string = 'value'
): asserts value is number {
  assertNumber(value, fieldName);
  if (value < 0) {
    throw ValidationError.outOfRange(fieldName, 0, Infinity, value);
  }
}

/**
 * Assert value is an integer
 */
export function assertInteger(
  value: unknown,
  fieldName: string = 'value'
): asserts value is number {
  assertNumber(value, fieldName);
  if (!Number.isInteger(value)) {
    throw ValidationError.invalidType(fieldName, 'integer', 'number');
  }
}

/**
 * Assert value is a positive integer
 */
export function assertPositiveInteger(
  value: unknown,
  fieldName: string = 'value'
): asserts value is number {
  assertInteger(value, fieldName);
  if (value <= 0) {
    throw ValidationError.outOfRange(fieldName, 1, Infinity, value);
  }
}

/**
 * Assert value is a non-negative integer
 */
export function assertNonNegativeInteger(
  value: unknown,
  fieldName: string = 'value'
): asserts value is number {
  assertInteger(value, fieldName);
  if (value < 0) {
    throw ValidationError.outOfRange(fieldName, 0, Infinity, value);
  }
}

/**
 * Assert value is in range
 */
export function assertInRange(
  value: number,
  fieldName: string,
  min: number,
  max: number
): asserts value is number {
  if (value < min || value > max) {
    throw ValidationError.outOfRange(fieldName, min, max, value);
  }
}

/**
 * Assert value is a non-empty array
 */
export function assertNonEmptyArray(
  value: unknown,
  fieldName: string = 'value'
): asserts value is unknown[] {
  assertArray(value, fieldName);
  if (value.length === 0) {
    throw ValidationError.missingRequired(fieldName);
  }
}

/**
 * Assert array meets length requirements
 */
export function assertArrayLength(
  value: unknown[],
  fieldName: string,
  min?: number,
  max?: number
): void {
  if (min !== undefined && value.length < min) {
    throw ValidationError.invalidFormat(fieldName, `At least ${min} items`, value);
  }

  if (max !== undefined && value.length > max) {
    throw ValidationError.invalidFormat(fieldName, `At most ${max} items`, value);
  }
}

/**
 * Assert object has property
 */
export function assertHasProperty<K extends string>(
  value: unknown,
  property: K,
  fieldName: string = 'value'
): asserts value is Record<K, unknown> {
  assertObject(value, fieldName);
  if (!(property in value)) {
    throw ValidationError.missingRequired(`${fieldName}.${property}`);
  }
}

/**
 * Assert object has all properties
 */
export function assertHasProperties<K extends string>(
  value: unknown,
  properties: K[],
  fieldName: string = 'value'
): asserts value is Record<K, unknown> {
  assertObject(value, fieldName);
  for (const property of properties) {
    if (!(property in value)) {
      throw ValidationError.missingRequired(`${fieldName}.${property}`);
    }
  }
}

/**
 * Assert value is one of allowed values
 */
export function assertOneOf<T extends string | number | boolean>(
  value: unknown,
  allowedValues: readonly T[],
  fieldName: string = 'value'
): asserts value is T {
  if (!allowedValues.includes(value as T)) {
    throw ValidationError.invalidFormat(
      fieldName,
      `One of: ${allowedValues.join(', ')}`,
      value
    );
  }
}

/**
 * Assert value matches pattern
 */
export function assertPattern(
  value: string,
  pattern: RegExp,
  fieldName: string = 'value',
  description?: string
): void {
  if (!pattern.test(value)) {
    throw ValidationError.invalidFormat(
      fieldName,
      description ?? `Must match pattern: ${pattern.source}`,
      value
    );
  }
}

/**
 * Assert condition is true
 */
export function assertCondition(
  condition: boolean,
  message: string,
  fieldName: string = 'value'
): void {
  if (!condition) {
    throw ValidationError.forField(fieldName, message);
  }
}

/**
 * Assert code is unreachable (for exhaustive type checking)
 */
export function assertNever(value: never, message?: string): never {
  throw new Error(
    message ?? `Unexpected value: ${JSON.stringify(value)}`
  );
}

/**
 * Assert that code should not be reached
 */
export function assertNotReached(message: string = 'This code should not be reached'): never {
  throw new Error(message);
}

/**
 * Assert value equals expected
 */
export function assertEqual<T>(
  value: T,
  expected: T,
  fieldName: string = 'value'
): void {
  if (value !== expected) {
    throw ValidationError.forField(
      fieldName,
      `Expected ${expected}, got ${value}`
    );
  }
}

/**
 * Assert value does not equal unexpected
 */
export function assertNotEqual<T>(
  value: T,
  unexpected: T,
  fieldName: string = 'value'
): void {
  if (value === unexpected) {
    throw ValidationError.forField(
      fieldName,
      `Value should not equal ${unexpected}`
    );
  }
}

/**
 * Assert deep equality
 */
export function assertDeepEqual<T>(
  value: T,
  expected: T,
  fieldName: string = 'value'
): void {
  if (JSON.stringify(value) !== JSON.stringify(expected)) {
    throw ValidationError.forField(
      fieldName,
      `Expected ${JSON.stringify(expected)}, got ${JSON.stringify(value)}`
    );
  }
}
