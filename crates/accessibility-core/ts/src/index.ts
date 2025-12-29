/**
 * Accessibility Core - Error handling and types for Enterprise Web-Accessibility SaaS
 * @module @harborgrid/accessibility-core
 */

// Types
export * from './types';

// Error classes
export { AccessibilityError } from './errors/AccessibilityError';
export { ValidationError } from './errors/ValidationError';
export { ScanError } from './errors/ScanError';
export { NetworkError } from './errors/NetworkError';
export { AuthError } from './errors/AuthError';
export { PermissionError } from './errors/PermissionError';

// Error handling infrastructure
export { ErrorBoundary, withErrorBoundary } from './errors/ErrorBoundary';
export type { ErrorBoundaryProps } from './errors/ErrorBoundary';

export { ErrorHandler, getErrorHandler, initializeErrorHandler } from './errors/ErrorHandler';

export { ErrorLogger, createLogger } from './errors/ErrorLogger';
export type { LogLevel, LogEntry, LoggerConfig } from './errors/ErrorLogger';

export {
  ErrorRecovery,
  createErrorRecovery,
  retry,
  retryWithFallback,
} from './errors/ErrorRecovery';

// Constants
export {
  ErrorCodes,
  ValidationErrorCodes,
  ScanErrorCodes,
  NetworkErrorCodes,
  AuthErrorCodes,
  PermissionErrorCodes,
  AccessibilityErrorCodes,
  SystemErrorCodes,
  ErrorCodeToHttpStatus,
  ErrorMessages,
} from './constants/errorCodes';
export type { ErrorCode } from './constants/errorCodes';

export {
  WCAG_CRITERIA,
  WCAG_CRITERIA_BY_LEVEL,
  WCAG_CRITERIA_BY_PRINCIPLE,
  getWCAGCriterion,
  getWCAGCriteriaForLevel,
  getWCAGCriteriaForPrinciple,
} from './constants/wcagCriteria';

export {
  SEVERITY_LEVELS,
  SEVERITY_PRIORITY,
  SEVERITY_DESCRIPTIONS,
  SEVERITY_COLORS,
  compareSeverity,
  isSeverityAtLeast,
  getSeverityFromPriority,
  sortSeverities,
  getSeverityIcon,
  getSeverityLabel,
} from './constants/severityLevels';

// Validation
export * from './validation/schemas';
export * from './validation/validators';

// Utilities
export * as typeGuards from './utils/typeGuards';
export * as assertions from './utils/assertions';

// React hooks
export {
  useErrorHandler,
  useAsyncError,
  useRetry,
} from './hooks/useErrorHandler';
export type {
  UseErrorHandlerOptions,
  UseErrorHandlerReturn,
} from './hooks/useErrorHandler';

// Re-export commonly used utilities
export {
  isAccessibilityError,
  isValidationError,
  isScanError,
  isNetworkError,
  isAuthError,
  isPermissionError,
  isWCAGLevel,
  isSeverityLevel,
  isValidUrl,
  isValidEmail,
} from './utils/typeGuards';

export {
  assertDefined,
  assertString,
  assertNumber,
  assertBoolean,
  assertObject,
  assertArray,
  assertWCAGLevel,
  assertValidUrl,
  assertValidEmail,
  assertNonEmptyString,
  assertPositiveNumber,
  assertNonNegativeNumber,
  assertInteger,
} from './utils/assertions';
