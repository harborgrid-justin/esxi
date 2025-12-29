/**
 * Network error class
 * @module errors/NetworkError
 */

import { AccessibilityError } from './AccessibilityError';
import { NetworkErrorCodes, ErrorCodeToHttpStatus } from '../constants/errorCodes';
import type { ErrorCode } from '../constants/errorCodes';

/**
 * Error thrown when network/API request fails
 */
export class NetworkError extends AccessibilityError {
  /** Request URL */
  public readonly url?: string;

  /** HTTP method */
  public readonly method?: string;

  /** Request ID if available */
  public readonly requestId?: string;

  /** Response status code if available */
  public readonly responseStatus?: number;

  constructor(
    message: string,
    options: {
      code?: ErrorCode;
      url?: string;
      method?: string;
      requestId?: string;
      responseStatus?: number;
      retryable?: boolean;
      cause?: Error;
      metadata?: Record<string, unknown>;
    } = {}
  ) {
    const code = options.code ?? NetworkErrorCodes.NETWORK_ERROR;

    super(message, {
      code,
      category: 'network',
      statusCode: options.responseStatus ?? ErrorCodeToHttpStatus[code] ?? 500,
      retryable: options.retryable ?? true,
      cause: options.cause,
      metadata: {
        ...options.metadata,
        url: options.url,
        method: options.method,
        requestId: options.requestId,
        responseStatus: options.responseStatus,
      },
    });

    this.url = options.url;
    this.method = options.method;
    this.requestId = options.requestId;
    this.responseStatus = options.responseStatus;
  }

  /**
   * Create error for request timeout
   */
  public static timeout(url: string, timeout: number): NetworkError {
    return new NetworkError(`Request timed out after ${timeout}ms`, {
      code: NetworkErrorCodes.REQUEST_TIMEOUT,
      url,
      retryable: true,
      metadata: {
        timeout,
      },
    });
  }

  /**
   * Create error for connection refused
   */
  public static connectionRefused(url: string): NetworkError {
    return new NetworkError('Connection refused', {
      code: NetworkErrorCodes.CONNECTION_REFUSED,
      url,
      retryable: true,
    });
  }

  /**
   * Create error for DNS resolution failure
   */
  public static dnsError(url: string, cause?: Error): NetworkError {
    return new NetworkError('DNS resolution failed', {
      code: NetworkErrorCodes.DNS_ERROR,
      url,
      retryable: true,
      cause,
    });
  }

  /**
   * Create error for SSL/TLS error
   */
  public static sslError(url: string, cause?: Error): NetworkError {
    return new NetworkError('SSL/TLS error', {
      code: NetworkErrorCodes.SSL_ERROR,
      url,
      retryable: false,
      cause,
    });
  }

  /**
   * Create error for rate limiting
   */
  public static rateLimited(retryAfter?: number): NetworkError {
    const message = retryAfter
      ? `Rate limited. Retry after ${retryAfter} seconds`
      : 'Rate limited. Please try again later';

    return new NetworkError(message, {
      code: NetworkErrorCodes.RATE_LIMITED,
      retryable: true,
      metadata: {
        retryAfter,
      },
    });
  }

  /**
   * Create error for service unavailable
   */
  public static serviceUnavailable(url?: string): NetworkError {
    return new NetworkError('Service is temporarily unavailable', {
      code: NetworkErrorCodes.SERVICE_UNAVAILABLE,
      url,
      retryable: true,
      responseStatus: 503,
    });
  }

  /**
   * Create error for bad gateway
   */
  public static badGateway(url?: string): NetworkError {
    return new NetworkError('Bad gateway', {
      code: NetworkErrorCodes.BAD_GATEWAY,
      url,
      retryable: true,
      responseStatus: 502,
    });
  }

  /**
   * Create error for gateway timeout
   */
  public static gatewayTimeout(url?: string): NetworkError {
    return new NetworkError('Gateway timeout', {
      code: NetworkErrorCodes.GATEWAY_TIMEOUT,
      url,
      retryable: true,
      responseStatus: 504,
    });
  }

  /**
   * Create error for CORS
   */
  public static corsError(url: string): NetworkError {
    return new NetworkError('Cross-origin request blocked', {
      code: NetworkErrorCodes.CORS_ERROR,
      url,
      retryable: false,
    });
  }

  /**
   * Create error for aborted request
   */
  public static aborted(url?: string): NetworkError {
    return new NetworkError('Request was aborted', {
      code: NetworkErrorCodes.ABORT_ERROR,
      url,
      retryable: false,
    });
  }

  /**
   * Create error from fetch/axios error
   */
  public static fromFetchError(error: unknown, url?: string): NetworkError {
    if (error instanceof NetworkError) {
      return error;
    }

    if (error instanceof Error) {
      // Check for common network error patterns
      if (error.name === 'AbortError') {
        return NetworkError.aborted(url);
      }

      if (error.message.includes('timeout')) {
        return NetworkError.timeout(url ?? '', 0);
      }

      if (error.message.includes('CORS')) {
        return NetworkError.corsError(url ?? '');
      }

      if (error.message.includes('DNS')) {
        return NetworkError.dnsError(url ?? '', error);
      }

      return new NetworkError(error.message, {
        url,
        cause: error,
      });
    }

    return new NetworkError('Network request failed', {
      url,
      metadata: {
        originalError: error,
      },
    });
  }

  /**
   * Convert error to plain object for serialization
   */
  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      url: this.url,
      method: this.method,
      requestId: this.requestId,
      responseStatus: this.responseStatus,
    };
  }
}
