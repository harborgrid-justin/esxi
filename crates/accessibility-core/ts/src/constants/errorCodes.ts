/**
 * Error codes for the accessibility platform
 * @module constants/errorCodes
 */

/**
 * Validation error codes
 */
export const ValidationErrorCodes = {
  INVALID_INPUT: 'VALIDATION_INVALID_INPUT',
  MISSING_REQUIRED: 'VALIDATION_MISSING_REQUIRED',
  INVALID_FORMAT: 'VALIDATION_INVALID_FORMAT',
  INVALID_TYPE: 'VALIDATION_INVALID_TYPE',
  OUT_OF_RANGE: 'VALIDATION_OUT_OF_RANGE',
  INVALID_URL: 'VALIDATION_INVALID_URL',
  INVALID_EMAIL: 'VALIDATION_INVALID_EMAIL',
  INVALID_WCAG_LEVEL: 'VALIDATION_INVALID_WCAG_LEVEL',
  INVALID_SELECTOR: 'VALIDATION_INVALID_SELECTOR',
  SCHEMA_VALIDATION_FAILED: 'VALIDATION_SCHEMA_FAILED',
} as const;

/**
 * Scan error codes
 */
export const ScanErrorCodes = {
  SCAN_FAILED: 'SCAN_FAILED',
  SCAN_TIMEOUT: 'SCAN_TIMEOUT',
  SCAN_CANCELLED: 'SCAN_CANCELLED',
  PAGE_LOAD_FAILED: 'SCAN_PAGE_LOAD_FAILED',
  INVALID_PAGE: 'SCAN_INVALID_PAGE',
  BROWSER_ERROR: 'SCAN_BROWSER_ERROR',
  NAVIGATION_ERROR: 'SCAN_NAVIGATION_ERROR',
  SCRIPT_EXECUTION_ERROR: 'SCAN_SCRIPT_EXECUTION_ERROR',
  SCREENSHOT_FAILED: 'SCAN_SCREENSHOT_FAILED',
  TOO_MANY_VIOLATIONS: 'SCAN_TOO_MANY_VIOLATIONS',
  SCAN_QUEUE_FULL: 'SCAN_QUEUE_FULL',
} as const;

/**
 * Network error codes
 */
export const NetworkErrorCodes = {
  NETWORK_ERROR: 'NETWORK_ERROR',
  REQUEST_TIMEOUT: 'NETWORK_REQUEST_TIMEOUT',
  CONNECTION_REFUSED: 'NETWORK_CONNECTION_REFUSED',
  DNS_ERROR: 'NETWORK_DNS_ERROR',
  SSL_ERROR: 'NETWORK_SSL_ERROR',
  RATE_LIMITED: 'NETWORK_RATE_LIMITED',
  SERVICE_UNAVAILABLE: 'NETWORK_SERVICE_UNAVAILABLE',
  BAD_GATEWAY: 'NETWORK_BAD_GATEWAY',
  GATEWAY_TIMEOUT: 'NETWORK_GATEWAY_TIMEOUT',
  CORS_ERROR: 'NETWORK_CORS_ERROR',
  ABORT_ERROR: 'NETWORK_ABORT_ERROR',
} as const;

/**
 * Authentication error codes
 */
export const AuthErrorCodes = {
  UNAUTHORIZED: 'AUTH_UNAUTHORIZED',
  INVALID_CREDENTIALS: 'AUTH_INVALID_CREDENTIALS',
  TOKEN_EXPIRED: 'AUTH_TOKEN_EXPIRED',
  TOKEN_INVALID: 'AUTH_TOKEN_INVALID',
  TOKEN_MISSING: 'AUTH_TOKEN_MISSING',
  SESSION_EXPIRED: 'AUTH_SESSION_EXPIRED',
  SESSION_INVALID: 'AUTH_SESSION_INVALID',
  MFA_REQUIRED: 'AUTH_MFA_REQUIRED',
  MFA_FAILED: 'AUTH_MFA_FAILED',
  ACCOUNT_LOCKED: 'AUTH_ACCOUNT_LOCKED',
  ACCOUNT_DISABLED: 'AUTH_ACCOUNT_DISABLED',
} as const;

/**
 * Permission error codes
 */
export const PermissionErrorCodes = {
  FORBIDDEN: 'PERMISSION_FORBIDDEN',
  INSUFFICIENT_PERMISSIONS: 'PERMISSION_INSUFFICIENT',
  RESOURCE_NOT_FOUND: 'PERMISSION_RESOURCE_NOT_FOUND',
  OPERATION_NOT_ALLOWED: 'PERMISSION_OPERATION_NOT_ALLOWED',
  QUOTA_EXCEEDED: 'PERMISSION_QUOTA_EXCEEDED',
  PLAN_LIMIT_REACHED: 'PERMISSION_PLAN_LIMIT_REACHED',
  FEATURE_NOT_AVAILABLE: 'PERMISSION_FEATURE_NOT_AVAILABLE',
} as const;

/**
 * Accessibility error codes
 */
export const AccessibilityErrorCodes = {
  WCAG_VIOLATION: 'A11Y_WCAG_VIOLATION',
  ARIA_ERROR: 'A11Y_ARIA_ERROR',
  KEYBOARD_NAVIGATION: 'A11Y_KEYBOARD_NAVIGATION',
  COLOR_CONTRAST: 'A11Y_COLOR_CONTRAST',
  MISSING_ALT_TEXT: 'A11Y_MISSING_ALT_TEXT',
  INVALID_HEADING_ORDER: 'A11Y_INVALID_HEADING_ORDER',
  MISSING_LABEL: 'A11Y_MISSING_LABEL',
  INVALID_ROLE: 'A11Y_INVALID_ROLE',
  MISSING_LANDMARK: 'A11Y_MISSING_LANDMARK',
  FOCUS_ORDER: 'A11Y_FOCUS_ORDER',
} as const;

/**
 * System error codes
 */
export const SystemErrorCodes = {
  INTERNAL_ERROR: 'SYSTEM_INTERNAL_ERROR',
  NOT_IMPLEMENTED: 'SYSTEM_NOT_IMPLEMENTED',
  CONFIGURATION_ERROR: 'SYSTEM_CONFIGURATION_ERROR',
  DATABASE_ERROR: 'SYSTEM_DATABASE_ERROR',
  FILE_SYSTEM_ERROR: 'SYSTEM_FILE_SYSTEM_ERROR',
  MEMORY_ERROR: 'SYSTEM_MEMORY_ERROR',
  DEPENDENCY_ERROR: 'SYSTEM_DEPENDENCY_ERROR',
  INITIALIZATION_ERROR: 'SYSTEM_INITIALIZATION_ERROR',
} as const;

/**
 * All error codes combined
 */
export const ErrorCodes = {
  ...ValidationErrorCodes,
  ...ScanErrorCodes,
  ...NetworkErrorCodes,
  ...AuthErrorCodes,
  ...PermissionErrorCodes,
  ...AccessibilityErrorCodes,
  ...SystemErrorCodes,
} as const;

/**
 * Error code type
 */
export type ErrorCode = (typeof ErrorCodes)[keyof typeof ErrorCodes];

/**
 * Error code to HTTP status mapping
 */
export const ErrorCodeToHttpStatus: Record<string, number> = {
  // Validation errors -> 400 Bad Request
  [ValidationErrorCodes.INVALID_INPUT]: 400,
  [ValidationErrorCodes.MISSING_REQUIRED]: 400,
  [ValidationErrorCodes.INVALID_FORMAT]: 400,
  [ValidationErrorCodes.INVALID_TYPE]: 400,
  [ValidationErrorCodes.OUT_OF_RANGE]: 400,
  [ValidationErrorCodes.INVALID_URL]: 400,
  [ValidationErrorCodes.INVALID_EMAIL]: 400,
  [ValidationErrorCodes.INVALID_WCAG_LEVEL]: 400,
  [ValidationErrorCodes.INVALID_SELECTOR]: 400,
  [ValidationErrorCodes.SCHEMA_VALIDATION_FAILED]: 422,

  // Authentication errors -> 401 Unauthorized
  [AuthErrorCodes.UNAUTHORIZED]: 401,
  [AuthErrorCodes.INVALID_CREDENTIALS]: 401,
  [AuthErrorCodes.TOKEN_EXPIRED]: 401,
  [AuthErrorCodes.TOKEN_INVALID]: 401,
  [AuthErrorCodes.TOKEN_MISSING]: 401,
  [AuthErrorCodes.SESSION_EXPIRED]: 401,
  [AuthErrorCodes.SESSION_INVALID]: 401,
  [AuthErrorCodes.MFA_REQUIRED]: 401,
  [AuthErrorCodes.MFA_FAILED]: 401,
  [AuthErrorCodes.ACCOUNT_LOCKED]: 403,
  [AuthErrorCodes.ACCOUNT_DISABLED]: 403,

  // Permission errors -> 403 Forbidden
  [PermissionErrorCodes.FORBIDDEN]: 403,
  [PermissionErrorCodes.INSUFFICIENT_PERMISSIONS]: 403,
  [PermissionErrorCodes.RESOURCE_NOT_FOUND]: 404,
  [PermissionErrorCodes.OPERATION_NOT_ALLOWED]: 403,
  [PermissionErrorCodes.QUOTA_EXCEEDED]: 429,
  [PermissionErrorCodes.PLAN_LIMIT_REACHED]: 402,
  [PermissionErrorCodes.FEATURE_NOT_AVAILABLE]: 403,

  // Network errors -> various
  [NetworkErrorCodes.RATE_LIMITED]: 429,
  [NetworkErrorCodes.SERVICE_UNAVAILABLE]: 503,
  [NetworkErrorCodes.BAD_GATEWAY]: 502,
  [NetworkErrorCodes.GATEWAY_TIMEOUT]: 504,
  [NetworkErrorCodes.REQUEST_TIMEOUT]: 408,

  // System errors -> 500 Internal Server Error
  [SystemErrorCodes.INTERNAL_ERROR]: 500,
  [SystemErrorCodes.NOT_IMPLEMENTED]: 501,
  [SystemErrorCodes.CONFIGURATION_ERROR]: 500,
  [SystemErrorCodes.DATABASE_ERROR]: 500,
  [SystemErrorCodes.FILE_SYSTEM_ERROR]: 500,
  [SystemErrorCodes.MEMORY_ERROR]: 500,
  [SystemErrorCodes.DEPENDENCY_ERROR]: 500,
  [SystemErrorCodes.INITIALIZATION_ERROR]: 500,

  // Scan errors -> 500 or 400 depending on context
  [ScanErrorCodes.SCAN_FAILED]: 500,
  [ScanErrorCodes.SCAN_TIMEOUT]: 504,
  [ScanErrorCodes.SCAN_CANCELLED]: 499,
  [ScanErrorCodes.PAGE_LOAD_FAILED]: 422,
  [ScanErrorCodes.INVALID_PAGE]: 400,
  [ScanErrorCodes.BROWSER_ERROR]: 500,
  [ScanErrorCodes.NAVIGATION_ERROR]: 422,
  [ScanErrorCodes.SCRIPT_EXECUTION_ERROR]: 500,
  [ScanErrorCodes.SCREENSHOT_FAILED]: 500,
  [ScanErrorCodes.TOO_MANY_VIOLATIONS]: 422,
  [ScanErrorCodes.SCAN_QUEUE_FULL]: 503,
};

/**
 * User-friendly error messages
 */
export const ErrorMessages: Record<string, string> = {
  // Validation
  [ValidationErrorCodes.INVALID_INPUT]: 'The provided input is invalid.',
  [ValidationErrorCodes.MISSING_REQUIRED]: 'Required field is missing.',
  [ValidationErrorCodes.INVALID_FORMAT]: 'The format of the input is incorrect.',
  [ValidationErrorCodes.INVALID_TYPE]: 'The type of the input is incorrect.',
  [ValidationErrorCodes.OUT_OF_RANGE]: 'The value is out of the acceptable range.',
  [ValidationErrorCodes.INVALID_URL]: 'The URL format is invalid.',
  [ValidationErrorCodes.INVALID_EMAIL]: 'The email address format is invalid.',
  [ValidationErrorCodes.INVALID_WCAG_LEVEL]: 'Invalid WCAG conformance level. Must be A, AA, or AAA.',
  [ValidationErrorCodes.INVALID_SELECTOR]: 'The CSS selector is invalid.',
  [ValidationErrorCodes.SCHEMA_VALIDATION_FAILED]: 'Data validation failed.',

  // Authentication
  [AuthErrorCodes.UNAUTHORIZED]: 'You are not authorized to access this resource.',
  [AuthErrorCodes.INVALID_CREDENTIALS]: 'Invalid username or password.',
  [AuthErrorCodes.TOKEN_EXPIRED]: 'Your session has expired. Please log in again.',
  [AuthErrorCodes.TOKEN_INVALID]: 'Invalid authentication token.',
  [AuthErrorCodes.TOKEN_MISSING]: 'Authentication token is missing.',
  [AuthErrorCodes.SESSION_EXPIRED]: 'Your session has expired.',
  [AuthErrorCodes.SESSION_INVALID]: 'Invalid session.',
  [AuthErrorCodes.MFA_REQUIRED]: 'Multi-factor authentication is required.',
  [AuthErrorCodes.MFA_FAILED]: 'Multi-factor authentication failed.',
  [AuthErrorCodes.ACCOUNT_LOCKED]: 'Your account has been locked.',
  [AuthErrorCodes.ACCOUNT_DISABLED]: 'Your account has been disabled.',

  // Permission
  [PermissionErrorCodes.FORBIDDEN]: 'You do not have permission to access this resource.',
  [PermissionErrorCodes.INSUFFICIENT_PERMISSIONS]: 'Insufficient permissions to perform this action.',
  [PermissionErrorCodes.RESOURCE_NOT_FOUND]: 'The requested resource was not found.',
  [PermissionErrorCodes.OPERATION_NOT_ALLOWED]: 'This operation is not allowed.',
  [PermissionErrorCodes.QUOTA_EXCEEDED]: 'You have exceeded your usage quota.',
  [PermissionErrorCodes.PLAN_LIMIT_REACHED]: 'You have reached your plan limit.',
  [PermissionErrorCodes.FEATURE_NOT_AVAILABLE]: 'This feature is not available on your current plan.',

  // Network
  [NetworkErrorCodes.NETWORK_ERROR]: 'A network error occurred.',
  [NetworkErrorCodes.REQUEST_TIMEOUT]: 'The request timed out.',
  [NetworkErrorCodes.CONNECTION_REFUSED]: 'Connection refused.',
  [NetworkErrorCodes.DNS_ERROR]: 'DNS resolution failed.',
  [NetworkErrorCodes.SSL_ERROR]: 'SSL/TLS error occurred.',
  [NetworkErrorCodes.RATE_LIMITED]: 'Too many requests. Please try again later.',
  [NetworkErrorCodes.SERVICE_UNAVAILABLE]: 'Service is temporarily unavailable.',
  [NetworkErrorCodes.BAD_GATEWAY]: 'Bad gateway.',
  [NetworkErrorCodes.GATEWAY_TIMEOUT]: 'Gateway timeout.',
  [NetworkErrorCodes.CORS_ERROR]: 'Cross-origin request blocked.',
  [NetworkErrorCodes.ABORT_ERROR]: 'Request was aborted.',

  // Scan
  [ScanErrorCodes.SCAN_FAILED]: 'The accessibility scan failed.',
  [ScanErrorCodes.SCAN_TIMEOUT]: 'The scan timed out.',
  [ScanErrorCodes.SCAN_CANCELLED]: 'The scan was cancelled.',
  [ScanErrorCodes.PAGE_LOAD_FAILED]: 'Failed to load the page.',
  [ScanErrorCodes.INVALID_PAGE]: 'The page is invalid or inaccessible.',
  [ScanErrorCodes.BROWSER_ERROR]: 'Browser error occurred during scan.',
  [ScanErrorCodes.NAVIGATION_ERROR]: 'Failed to navigate to the page.',
  [ScanErrorCodes.SCRIPT_EXECUTION_ERROR]: 'Script execution failed during scan.',
  [ScanErrorCodes.SCREENSHOT_FAILED]: 'Failed to capture screenshot.',
  [ScanErrorCodes.TOO_MANY_VIOLATIONS]: 'Too many violations detected.',
  [ScanErrorCodes.SCAN_QUEUE_FULL]: 'Scan queue is full. Please try again later.',

  // System
  [SystemErrorCodes.INTERNAL_ERROR]: 'An internal error occurred.',
  [SystemErrorCodes.NOT_IMPLEMENTED]: 'This feature is not yet implemented.',
  [SystemErrorCodes.CONFIGURATION_ERROR]: 'Configuration error.',
  [SystemErrorCodes.DATABASE_ERROR]: 'Database error occurred.',
  [SystemErrorCodes.FILE_SYSTEM_ERROR]: 'File system error occurred.',
  [SystemErrorCodes.MEMORY_ERROR]: 'Memory error occurred.',
  [SystemErrorCodes.DEPENDENCY_ERROR]: 'Dependency error occurred.',
  [SystemErrorCodes.INITIALIZATION_ERROR]: 'Initialization failed.',
};
