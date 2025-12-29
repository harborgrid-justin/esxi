/**
 * Shared types for accessibility modules
 * @module types
 */

/**
 * WCAG conformance levels
 */
export type WCAGLevel = 'A' | 'AA' | 'AAA';

/**
 * WCAG principles
 */
export type WCAGPrinciple = 'perceivable' | 'operable' | 'understandable' | 'robust';

/**
 * Severity levels for accessibility issues
 */
export type SeverityLevel = 'critical' | 'serious' | 'moderate' | 'minor' | 'info';

/**
 * Error categories for platform errors
 */
export type ErrorCategory =
  | 'validation'
  | 'scan'
  | 'network'
  | 'auth'
  | 'permission'
  | 'accessibility'
  | 'system'
  | 'unknown';

/**
 * HTTP status codes
 */
export type HttpStatusCode =
  | 200 | 201 | 204
  | 400 | 401 | 403 | 404 | 409 | 422 | 429
  | 500 | 502 | 503 | 504;

/**
 * WCAG success criterion definition
 */
export interface WCAGCriterion {
  /** Criterion identifier (e.g., "1.1.1") */
  id: string;
  /** Criterion name */
  name: string;
  /** WCAG level */
  level: WCAGLevel;
  /** WCAG principle */
  principle: WCAGPrinciple;
  /** Description of the criterion */
  description: string;
  /** URL to WCAG documentation */
  url: string;
}

/**
 * Accessibility violation details
 */
export interface AccessibilityViolation {
  /** Unique identifier for the violation */
  id: string;
  /** WCAG criterion that was violated */
  criterion: WCAGCriterion;
  /** Severity of the violation */
  severity: SeverityLevel;
  /** Description of the violation */
  message: string;
  /** CSS selector or XPath to the problematic element */
  selector?: string;
  /** HTML snippet of the problematic element */
  html?: string;
  /** Suggested fix for the violation */
  suggestion?: string;
  /** Additional context about the violation */
  context?: Record<string, unknown>;
}

/**
 * Error metadata for enhanced error tracking
 */
export interface ErrorMetadata {
  /** Unique error identifier */
  errorId?: string;
  /** Timestamp when the error occurred */
  timestamp: Date;
  /** User ID if available */
  userId?: string;
  /** Session ID if available */
  sessionId?: string;
  /** Request ID for API calls */
  requestId?: string;
  /** URL or route where error occurred */
  url?: string;
  /** User agent string */
  userAgent?: string;
  /** Additional custom context */
  context?: Record<string, unknown>;
}

/**
 * Error recovery strategy
 */
export interface RecoveryStrategy {
  /** Strategy name */
  name: string;
  /** Strategy description */
  description: string;
  /** Function to execute recovery */
  execute: () => Promise<void> | void;
  /** Whether strategy can be retried */
  retriable: boolean;
}

/**
 * Validation error details
 */
export interface ValidationErrorDetail {
  /** Field that failed validation */
  field: string;
  /** Validation error message */
  message: string;
  /** Validation rule that failed */
  rule: string;
  /** Current value that failed validation */
  value?: unknown;
  /** Expected value or format */
  expected?: string;
}

/**
 * Scan configuration
 */
export interface ScanConfig {
  /** URL or URLs to scan */
  urls: string[];
  /** WCAG level to test against */
  level: WCAGLevel;
  /** Rules to include in scan */
  includeRules?: string[];
  /** Rules to exclude from scan */
  excludeRules?: string[];
  /** Maximum number of pages to scan */
  maxPages?: number;
  /** Timeout for each page scan (ms) */
  timeout?: number;
  /** Whether to capture screenshots */
  screenshots?: boolean;
}

/**
 * Scan result
 */
export interface ScanResult {
  /** Unique scan identifier */
  scanId: string;
  /** URL that was scanned */
  url: string;
  /** Timestamp of the scan */
  timestamp: Date;
  /** WCAG level used for scan */
  level: WCAGLevel;
  /** List of violations found */
  violations: AccessibilityViolation[];
  /** Number of elements tested */
  elementsTested: number;
  /** Scan duration in milliseconds */
  duration: number;
  /** Whether scan completed successfully */
  success: boolean;
  /** Error message if scan failed */
  error?: string;
}

/**
 * API response wrapper
 */
export interface ApiResponse<T = unknown> {
  /** Response data */
  data?: T;
  /** Error if request failed */
  error?: {
    message: string;
    code: string;
    details?: unknown;
  };
  /** Response metadata */
  meta?: {
    requestId: string;
    timestamp: Date;
    version: string;
  };
}

/**
 * Retry configuration
 */
export interface RetryConfig {
  /** Maximum number of retry attempts */
  maxRetries: number;
  /** Initial delay before first retry (ms) */
  initialDelay: number;
  /** Maximum delay between retries (ms) */
  maxDelay: number;
  /** Backoff multiplier */
  backoffMultiplier: number;
  /** Whether to use exponential backoff */
  exponentialBackoff: boolean;
}

/**
 * Logger interface
 */
export interface Logger {
  debug(message: string, context?: Record<string, unknown>): void;
  info(message: string, context?: Record<string, unknown>): void;
  warn(message: string, context?: Record<string, unknown>): void;
  error(message: string, error?: Error, context?: Record<string, unknown>): void;
}

/**
 * Error handler configuration
 */
export interface ErrorHandlerConfig {
  /** Whether to log errors */
  logErrors: boolean;
  /** Whether to report errors to external service */
  reportErrors: boolean;
  /** Whether to show user-friendly error messages */
  showUserMessages: boolean;
  /** Custom logger instance */
  logger?: Logger;
  /** Error reporting endpoint */
  reportingEndpoint?: string;
  /** Additional configuration */
  context?: Record<string, unknown>;
}
