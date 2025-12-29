/**
 * Base error class for accessibility platform errors
 * @module errors/AccessibilityError
 */

import type { ErrorCategory, ErrorMetadata } from '../types';
import type { ErrorCode } from '../constants/errorCodes';

/**
 * Base error class for all accessibility platform errors
 */
export class AccessibilityError extends Error {
  /** Error code */
  public readonly code: ErrorCode;

  /** Error category */
  public readonly category: ErrorCategory;

  /** HTTP status code */
  public readonly statusCode: number;

  /** Error metadata */
  public readonly metadata: ErrorMetadata;

  /** Whether error is retryable */
  public readonly retryable: boolean;

  /** Original error if this wraps another error */
  public readonly cause?: Error;

  /**
   * Create a new AccessibilityError
   */
  constructor(
    message: string,
    options: {
      code: ErrorCode;
      category?: ErrorCategory;
      statusCode?: number;
      retryable?: boolean;
      cause?: Error;
      metadata?: Partial<ErrorMetadata>;
    }
  ) {
    super(message);

    // Maintains proper stack trace for where error was thrown (V8 only)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }

    this.name = this.constructor.name;
    this.code = options.code;
    this.category = options.category ?? 'accessibility';
    this.statusCode = options.statusCode ?? 500;
    this.retryable = options.retryable ?? false;
    this.cause = options.cause;

    this.metadata = {
      timestamp: new Date(),
      errorId: this.generateErrorId(),
      ...options.metadata,
    };

    // Set the prototype explicitly (needed for extending built-in classes in TypeScript)
    Object.setPrototypeOf(this, new.target.prototype);
  }

  /**
   * Generate a unique error ID
   */
  private generateErrorId(): string {
    return `err_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
  }

  /**
   * Convert error to plain object for serialization
   */
  public toJSON(): Record<string, unknown> {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      category: this.category,
      statusCode: this.statusCode,
      retryable: this.retryable,
      metadata: {
        ...this.metadata,
        timestamp: this.metadata.timestamp.toISOString(),
      },
      stack: this.stack,
      cause: this.cause
        ? {
            name: this.cause.name,
            message: this.cause.message,
            stack: this.cause.stack,
          }
        : undefined,
    };
  }

  /**
   * Convert error to user-friendly message
   */
  public toUserMessage(): string {
    return this.message;
  }

  /**
   * Check if error is of a specific category
   */
  public isCategory(category: ErrorCategory): boolean {
    return this.category === category;
  }

  /**
   * Check if error is retryable
   */
  public isRetryable(): boolean {
    return this.retryable;
  }

  /**
   * Create error from unknown error type
   */
  public static fromUnknown(error: unknown, defaultCode: ErrorCode): AccessibilityError {
    if (error instanceof AccessibilityError) {
      return error;
    }

    if (error instanceof Error) {
      return new AccessibilityError(error.message, {
        code: defaultCode,
        cause: error,
        metadata: {
          context: {
            originalError: {
              name: error.name,
              message: error.message,
            },
          },
        },
      });
    }

    if (typeof error === 'string') {
      return new AccessibilityError(error, {
        code: defaultCode,
      });
    }

    return new AccessibilityError('An unknown error occurred', {
      code: defaultCode,
      metadata: {
        context: {
          originalError: error,
        },
      },
    });
  }

  /**
   * Create error from HTTP response
   */
  public static fromHttpResponse(
    response: {
      status: number;
      statusText: string;
      data?: { message?: string; code?: string };
    },
    defaultCode: ErrorCode
  ): AccessibilityError {
    const message = response.data?.message ?? response.statusText ?? 'HTTP request failed';
    const code = (response.data?.code as ErrorCode) ?? defaultCode;

    return new AccessibilityError(message, {
      code,
      statusCode: response.status,
      metadata: {
        context: {
          httpStatus: response.status,
          httpStatusText: response.statusText,
          responseData: response.data,
        },
      },
    });
  }
}
