/**
 * Validation error class
 * @module errors/ValidationError
 */

import { AccessibilityError } from './AccessibilityError';
import { ValidationErrorCodes, ErrorCodeToHttpStatus } from '../constants/errorCodes';
import type { ErrorCode } from '../constants/errorCodes';
import type { ValidationErrorDetail } from '../types';

/**
 * Error thrown when validation fails
 */
export class ValidationError extends AccessibilityError {
  /** Validation error details */
  public readonly details: ValidationErrorDetail[];

  constructor(
    message: string,
    options: {
      code?: ErrorCode;
      details?: ValidationErrorDetail[];
      cause?: Error;
      metadata?: Record<string, unknown>;
    } = {}
  ) {
    const code = options.code ?? ValidationErrorCodes.SCHEMA_VALIDATION_FAILED;

    super(message, {
      code,
      category: 'validation',
      statusCode: ErrorCodeToHttpStatus[code] ?? 400,
      retryable: false,
      cause: options.cause,
      metadata: options.metadata,
    });

    this.details = options.details ?? [];
  }

  /**
   * Create validation error for a single field
   */
  public static forField(
    field: string,
    message: string,
    options: {
      rule?: string;
      value?: unknown;
      expected?: string;
    } = {}
  ): ValidationError {
    return new ValidationError(`Validation failed for field "${field}": ${message}`, {
      code: ValidationErrorCodes.INVALID_INPUT,
      details: [
        {
          field,
          message,
          rule: options.rule ?? 'unknown',
          value: options.value,
          expected: options.expected,
        },
      ],
    });
  }

  /**
   * Create validation error for multiple fields
   */
  public static forFields(details: ValidationErrorDetail[]): ValidationError {
    const fieldNames = details.map((d) => d.field).join(', ');
    return new ValidationError(`Validation failed for fields: ${fieldNames}`, {
      code: ValidationErrorCodes.SCHEMA_VALIDATION_FAILED,
      details,
    });
  }

  /**
   * Create validation error for missing required field
   */
  public static missingRequired(field: string): ValidationError {
    return new ValidationError(`Required field "${field}" is missing`, {
      code: ValidationErrorCodes.MISSING_REQUIRED,
      details: [
        {
          field,
          message: 'This field is required',
          rule: 'required',
        },
      ],
    });
  }

  /**
   * Create validation error for invalid format
   */
  public static invalidFormat(
    field: string,
    expected: string,
    value?: unknown
  ): ValidationError {
    return new ValidationError(`Field "${field}" has invalid format. Expected: ${expected}`, {
      code: ValidationErrorCodes.INVALID_FORMAT,
      details: [
        {
          field,
          message: `Invalid format. Expected: ${expected}`,
          rule: 'format',
          value,
          expected,
        },
      ],
    });
  }

  /**
   * Create validation error for invalid type
   */
  public static invalidType(
    field: string,
    expected: string,
    actual: string
  ): ValidationError {
    return new ValidationError(
      `Field "${field}" has invalid type. Expected: ${expected}, got: ${actual}`,
      {
        code: ValidationErrorCodes.INVALID_TYPE,
        details: [
          {
            field,
            message: `Invalid type. Expected: ${expected}, got: ${actual}`,
            rule: 'type',
            expected,
          },
        ],
      }
    );
  }

  /**
   * Create validation error for out of range value
   */
  public static outOfRange(
    field: string,
    min: number | string,
    max: number | string,
    value?: unknown
  ): ValidationError {
    return new ValidationError(
      `Field "${field}" is out of range. Must be between ${min} and ${max}`,
      {
        code: ValidationErrorCodes.OUT_OF_RANGE,
        details: [
          {
            field,
            message: `Value is out of range. Must be between ${min} and ${max}`,
            rule: 'range',
            value,
            expected: `${min} to ${max}`,
          },
        ],
      }
    );
  }

  /**
   * Create validation error for invalid URL
   */
  public static invalidUrl(field: string, value?: string): ValidationError {
    return new ValidationError(`Field "${field}" contains an invalid URL`, {
      code: ValidationErrorCodes.INVALID_URL,
      details: [
        {
          field,
          message: 'Invalid URL format',
          rule: 'url',
          value,
          expected: 'Valid URL (e.g., https://example.com)',
        },
      ],
    });
  }

  /**
   * Create validation error for invalid email
   */
  public static invalidEmail(field: string, value?: string): ValidationError {
    return new ValidationError(`Field "${field}" contains an invalid email address`, {
      code: ValidationErrorCodes.INVALID_EMAIL,
      details: [
        {
          field,
          message: 'Invalid email format',
          rule: 'email',
          value,
          expected: 'Valid email address (e.g., user@example.com)',
        },
      ],
    });
  }

  /**
   * Create validation error for invalid WCAG level
   */
  public static invalidWcagLevel(value: unknown): ValidationError {
    return new ValidationError('Invalid WCAG conformance level', {
      code: ValidationErrorCodes.INVALID_WCAG_LEVEL,
      details: [
        {
          field: 'level',
          message: 'Invalid WCAG conformance level',
          rule: 'wcag_level',
          value,
          expected: 'A, AA, or AAA',
        },
      ],
    });
  }

  /**
   * Convert error to user-friendly message
   */
  public override toUserMessage(): string {
    if (this.details.length === 0) {
      return this.message;
    }

    if (this.details.length === 1) {
      return this.details[0]?.message ?? this.message;
    }

    const errorList = this.details
      .map((d) => `- ${d.field}: ${d.message}`)
      .join('\n');

    return `Validation errors:\n${errorList}`;
  }

  /**
   * Convert error to plain object for serialization
   */
  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      details: this.details,
    };
  }
}
