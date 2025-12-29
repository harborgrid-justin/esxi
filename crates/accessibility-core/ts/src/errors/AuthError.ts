/**
 * Authentication error class
 * @module errors/AuthError
 */

import { AccessibilityError } from './AccessibilityError';
import { AuthErrorCodes, ErrorCodeToHttpStatus } from '../constants/errorCodes';
import type { ErrorCode } from '../constants/errorCodes';

/**
 * Error thrown when authentication fails
 */
export class AuthError extends AccessibilityError {
  /** User ID if available */
  public readonly userId?: string;

  /** Session ID if available */
  public readonly sessionId?: string;

  constructor(
    message: string,
    options: {
      code?: ErrorCode;
      userId?: string;
      sessionId?: string;
      retryable?: boolean;
      cause?: Error;
      metadata?: Record<string, unknown>;
    } = {}
  ) {
    const code = options.code ?? AuthErrorCodes.UNAUTHORIZED;

    super(message, {
      code,
      category: 'auth',
      statusCode: ErrorCodeToHttpStatus[code] ?? 401,
      retryable: options.retryable ?? false,
      cause: options.cause,
      metadata: {
        ...options.metadata,
        userId: options.userId,
        sessionId: options.sessionId,
      },
    });

    this.userId = options.userId;
    this.sessionId = options.sessionId;
  }

  /**
   * Create error for unauthorized access
   */
  public static unauthorized(message?: string): AuthError {
    return new AuthError(message ?? 'You are not authorized to access this resource', {
      code: AuthErrorCodes.UNAUTHORIZED,
      retryable: false,
    });
  }

  /**
   * Create error for invalid credentials
   */
  public static invalidCredentials(): AuthError {
    return new AuthError('Invalid username or password', {
      code: AuthErrorCodes.INVALID_CREDENTIALS,
      retryable: false,
    });
  }

  /**
   * Create error for expired token
   */
  public static tokenExpired(): AuthError {
    return new AuthError('Your session has expired. Please log in again', {
      code: AuthErrorCodes.TOKEN_EXPIRED,
      retryable: false,
    });
  }

  /**
   * Create error for invalid token
   */
  public static tokenInvalid(): AuthError {
    return new AuthError('Invalid authentication token', {
      code: AuthErrorCodes.TOKEN_INVALID,
      retryable: false,
    });
  }

  /**
   * Create error for missing token
   */
  public static tokenMissing(): AuthError {
    return new AuthError('Authentication token is missing', {
      code: AuthErrorCodes.TOKEN_MISSING,
      retryable: false,
    });
  }

  /**
   * Create error for expired session
   */
  public static sessionExpired(sessionId?: string): AuthError {
    return new AuthError('Your session has expired', {
      code: AuthErrorCodes.SESSION_EXPIRED,
      sessionId,
      retryable: false,
    });
  }

  /**
   * Create error for invalid session
   */
  public static sessionInvalid(sessionId?: string): AuthError {
    return new AuthError('Invalid session', {
      code: AuthErrorCodes.SESSION_INVALID,
      sessionId,
      retryable: false,
    });
  }

  /**
   * Create error for MFA required
   */
  public static mfaRequired(userId?: string): AuthError {
    return new AuthError('Multi-factor authentication is required', {
      code: AuthErrorCodes.MFA_REQUIRED,
      userId,
      retryable: false,
    });
  }

  /**
   * Create error for MFA failed
   */
  public static mfaFailed(userId?: string): AuthError {
    return new AuthError('Multi-factor authentication failed', {
      code: AuthErrorCodes.MFA_FAILED,
      userId,
      retryable: false,
    });
  }

  /**
   * Create error for locked account
   */
  public static accountLocked(userId?: string, reason?: string): AuthError {
    const message = reason
      ? `Your account has been locked: ${reason}`
      : 'Your account has been locked';

    return new AuthError(message, {
      code: AuthErrorCodes.ACCOUNT_LOCKED,
      userId,
      retryable: false,
      metadata: {
        reason,
      },
    });
  }

  /**
   * Create error for disabled account
   */
  public static accountDisabled(userId?: string, reason?: string): AuthError {
    const message = reason
      ? `Your account has been disabled: ${reason}`
      : 'Your account has been disabled';

    return new AuthError(message, {
      code: AuthErrorCodes.ACCOUNT_DISABLED,
      userId,
      retryable: false,
      metadata: {
        reason,
      },
    });
  }

  /**
   * Convert error to plain object for serialization
   */
  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      userId: this.userId,
      sessionId: this.sessionId,
    };
  }
}
