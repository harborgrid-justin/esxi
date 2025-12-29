/**
 * Scan error class
 * @module errors/ScanError
 */

import { AccessibilityError } from './AccessibilityError';
import { ScanErrorCodes, ErrorCodeToHttpStatus } from '../constants/errorCodes';
import type { ErrorCode } from '../constants/errorCodes';

/**
 * Error thrown when accessibility scan fails
 */
export class ScanError extends AccessibilityError {
  /** URL being scanned */
  public readonly url?: string;

  /** Scan ID if available */
  public readonly scanId?: string;

  constructor(
    message: string,
    options: {
      code?: ErrorCode;
      url?: string;
      scanId?: string;
      retryable?: boolean;
      cause?: Error;
      metadata?: Record<string, unknown>;
    } = {}
  ) {
    const code = options.code ?? ScanErrorCodes.SCAN_FAILED;

    super(message, {
      code,
      category: 'scan',
      statusCode: ErrorCodeToHttpStatus[code] ?? 500,
      retryable: options.retryable ?? false,
      cause: options.cause,
      metadata: {
        ...options.metadata,
        url: options.url,
        scanId: options.scanId,
      },
    });

    this.url = options.url;
    this.scanId = options.scanId;
  }

  /**
   * Create error for scan timeout
   */
  public static timeout(url: string, timeout: number): ScanError {
    return new ScanError(`Scan timed out after ${timeout}ms`, {
      code: ScanErrorCodes.SCAN_TIMEOUT,
      url,
      retryable: true,
      metadata: {
        timeout,
      },
    });
  }

  /**
   * Create error for cancelled scan
   */
  public static cancelled(url: string, scanId?: string): ScanError {
    return new ScanError('Scan was cancelled', {
      code: ScanErrorCodes.SCAN_CANCELLED,
      url,
      scanId,
      retryable: false,
    });
  }

  /**
   * Create error for page load failure
   */
  public static pageLoadFailed(url: string, cause?: Error): ScanError {
    return new ScanError(`Failed to load page: ${url}`, {
      code: ScanErrorCodes.PAGE_LOAD_FAILED,
      url,
      retryable: true,
      cause,
    });
  }

  /**
   * Create error for invalid page
   */
  public static invalidPage(url: string, reason: string): ScanError {
    return new ScanError(`Invalid page: ${reason}`, {
      code: ScanErrorCodes.INVALID_PAGE,
      url,
      retryable: false,
      metadata: {
        reason,
      },
    });
  }

  /**
   * Create error for browser error
   */
  public static browserError(message: string, cause?: Error): ScanError {
    return new ScanError(`Browser error: ${message}`, {
      code: ScanErrorCodes.BROWSER_ERROR,
      retryable: true,
      cause,
    });
  }

  /**
   * Create error for navigation error
   */
  public static navigationError(url: string, cause?: Error): ScanError {
    return new ScanError(`Failed to navigate to ${url}`, {
      code: ScanErrorCodes.NAVIGATION_ERROR,
      url,
      retryable: true,
      cause,
    });
  }

  /**
   * Create error for script execution error
   */
  public static scriptExecutionError(message: string, cause?: Error): ScanError {
    return new ScanError(`Script execution failed: ${message}`, {
      code: ScanErrorCodes.SCRIPT_EXECUTION_ERROR,
      retryable: false,
      cause,
    });
  }

  /**
   * Create error for screenshot failure
   */
  public static screenshotFailed(url: string, cause?: Error): ScanError {
    return new ScanError('Failed to capture screenshot', {
      code: ScanErrorCodes.SCREENSHOT_FAILED,
      url,
      retryable: true,
      cause,
    });
  }

  /**
   * Create error for too many violations
   */
  public static tooManyViolations(url: string, count: number, limit: number): ScanError {
    return new ScanError(
      `Too many violations detected: ${count} (limit: ${limit})`,
      {
        code: ScanErrorCodes.TOO_MANY_VIOLATIONS,
        url,
        retryable: false,
        metadata: {
          violationCount: count,
          limit,
        },
      }
    );
  }

  /**
   * Create error for scan queue full
   */
  public static queueFull(): ScanError {
    return new ScanError('Scan queue is full. Please try again later.', {
      code: ScanErrorCodes.SCAN_QUEUE_FULL,
      retryable: true,
    });
  }

  /**
   * Convert error to plain object for serialization
   */
  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      url: this.url,
      scanId: this.scanId,
    };
  }
}
