/**
 * Permission error class
 * @module errors/PermissionError
 */

import { AccessibilityError } from './AccessibilityError';
import { PermissionErrorCodes, ErrorCodeToHttpStatus } from '../constants/errorCodes';
import type { ErrorCode } from '../constants/errorCodes';

/**
 * Error thrown when user lacks required permissions
 */
export class PermissionError extends AccessibilityError {
  /** Required permission */
  public readonly requiredPermission?: string;

  /** Resource being accessed */
  public readonly resource?: string;

  /** Action being attempted */
  public readonly action?: string;

  constructor(
    message: string,
    options: {
      code?: ErrorCode;
      requiredPermission?: string;
      resource?: string;
      action?: string;
      retryable?: boolean;
      cause?: Error;
      metadata?: Record<string, unknown>;
    } = {}
  ) {
    const code = options.code ?? PermissionErrorCodes.FORBIDDEN;

    super(message, {
      code,
      category: 'permission',
      statusCode: ErrorCodeToHttpStatus[code] ?? 403,
      retryable: options.retryable ?? false,
      cause: options.cause,
      metadata: {
        ...options.metadata,
        requiredPermission: options.requiredPermission,
        resource: options.resource,
        action: options.action,
      },
    });

    this.requiredPermission = options.requiredPermission;
    this.resource = options.resource;
    this.action = options.action;
  }

  /**
   * Create error for forbidden access
   */
  public static forbidden(resource?: string, action?: string): PermissionError {
    let message = 'You do not have permission to access this resource';
    if (resource && action) {
      message = `You do not have permission to ${action} ${resource}`;
    } else if (resource) {
      message = `You do not have permission to access ${resource}`;
    }

    return new PermissionError(message, {
      code: PermissionErrorCodes.FORBIDDEN,
      resource,
      action,
      retryable: false,
    });
  }

  /**
   * Create error for insufficient permissions
   */
  public static insufficientPermissions(
    requiredPermission: string,
    resource?: string
  ): PermissionError {
    const message = resource
      ? `Insufficient permissions to access ${resource}. Required: ${requiredPermission}`
      : `Insufficient permissions. Required: ${requiredPermission}`;

    return new PermissionError(message, {
      code: PermissionErrorCodes.INSUFFICIENT_PERMISSIONS,
      requiredPermission,
      resource,
      retryable: false,
    });
  }

  /**
   * Create error for resource not found
   */
  public static resourceNotFound(resource: string): PermissionError {
    return new PermissionError(`The requested resource was not found: ${resource}`, {
      code: PermissionErrorCodes.RESOURCE_NOT_FOUND,
      resource,
      retryable: false,
    });
  }

  /**
   * Create error for operation not allowed
   */
  public static operationNotAllowed(operation: string, reason?: string): PermissionError {
    const message = reason
      ? `Operation "${operation}" is not allowed: ${reason}`
      : `Operation "${operation}" is not allowed`;

    return new PermissionError(message, {
      code: PermissionErrorCodes.OPERATION_NOT_ALLOWED,
      action: operation,
      retryable: false,
      metadata: {
        reason,
      },
    });
  }

  /**
   * Create error for quota exceeded
   */
  public static quotaExceeded(
    quotaName: string,
    limit: number,
    current: number
  ): PermissionError {
    return new PermissionError(
      `You have exceeded your ${quotaName} quota (${current}/${limit})`,
      {
        code: PermissionErrorCodes.QUOTA_EXCEEDED,
        retryable: false,
        metadata: {
          quotaName,
          limit,
          current,
        },
      }
    );
  }

  /**
   * Create error for plan limit reached
   */
  public static planLimitReached(feature: string, planName: string): PermissionError {
    return new PermissionError(
      `You have reached the ${feature} limit for your ${planName} plan`,
      {
        code: PermissionErrorCodes.PLAN_LIMIT_REACHED,
        retryable: false,
        metadata: {
          feature,
          planName,
        },
      }
    );
  }

  /**
   * Create error for feature not available
   */
  public static featureNotAvailable(feature: string, planName?: string): PermissionError {
    const message = planName
      ? `Feature "${feature}" is not available on your ${planName} plan`
      : `Feature "${feature}" is not available`;

    return new PermissionError(message, {
      code: PermissionErrorCodes.FEATURE_NOT_AVAILABLE,
      retryable: false,
      metadata: {
        feature,
        planName,
      },
    });
  }

  /**
   * Convert error to plain object for serialization
   */
  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      requiredPermission: this.requiredPermission,
      resource: this.resource,
      action: this.action,
    };
  }
}
