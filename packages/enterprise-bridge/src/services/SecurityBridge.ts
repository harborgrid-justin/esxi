/**
 * Security service bridge for validation and sanitization.
 *
 * Provides TypeScript wrapper around WASM security engine.
 */

import type {
  SecurityParams,
  SecurityResult,
  CspValidationResult,
  PasswordValidationResult,
  OperationResult,
} from '../types';
import { BridgeError } from '../types';
import { WasmLoader } from '../loader/WasmLoader';

/**
 * Security service bridge.
 */
export class SecurityBridge {
  private securityEngine: any = null;

  constructor(
    private readonly loader: WasmLoader,
    private readonly strictMode = true
  ) {}

  /**
   * Initialize the security engine.
   */
  private async ensureInitialized(): Promise<void> {
    if (!this.securityEngine) {
      const instance = this.loader.getInstance();
      // In production: this.securityEngine = new instance.SecurityEngine(this.strictMode);
      throw new BridgeError(
        'Security engine not available. Build WASM module first.',
        'SECURITY_NOT_AVAILABLE'
      );
    }
  }

  /**
   * Validate input for security threats.
   *
   * @param params - Security validation parameters
   * @returns Validation result with detected threats
   */
  async validate(params: SecurityParams): Promise<OperationResult<SecurityResult>> {
    await this.ensureInitialized();

    try {
      const result = await this.securityEngine.validate(params);
      return result as OperationResult<SecurityResult>;
    } catch (error) {
      throw new BridgeError(
        `Security validation failed: ${error instanceof Error ? error.message : String(error)}`,
        'VALIDATE_ERROR',
        error
      );
    }
  }

  /**
   * Sanitize input by removing or escaping dangerous content.
   *
   * @param input - Input to sanitize
   * @param sanitizeType - Type of sanitization (html, sql, url, filename)
   * @returns Sanitized string
   */
  async sanitize(input: string, sanitizeType: string): Promise<string> {
    await this.ensureInitialized();

    try {
      const result = await this.securityEngine.sanitize(input, sanitizeType);
      return result as string;
    } catch (error) {
      throw new BridgeError(
        `Sanitization failed: ${error instanceof Error ? error.message : String(error)}`,
        'SANITIZE_ERROR',
        error
      );
    }
  }

  /**
   * Validate a Content Security Policy header.
   *
   * @param csp - CSP header value
   * @returns CSP validation result
   */
  async validateCsp(csp: string): Promise<CspValidationResult> {
    await this.ensureInitialized();

    try {
      const result = await this.securityEngine.validate_csp(csp);
      return result as CspValidationResult;
    } catch (error) {
      throw new BridgeError(
        `CSP validation failed: ${error instanceof Error ? error.message : String(error)}`,
        'CSP_ERROR',
        error
      );
    }
  }

  /**
   * Check if a password meets security requirements.
   *
   * @param password - Password to validate
   * @returns Password validation result
   */
  async validatePassword(password: string): Promise<PasswordValidationResult> {
    await this.ensureInitialized();

    try {
      const result = await this.securityEngine.validate_password(password);
      return result as PasswordValidationResult;
    } catch (error) {
      throw new BridgeError(
        `Password validation failed: ${error instanceof Error ? error.message : String(error)}`,
        'PASSWORD_ERROR',
        error
      );
    }
  }

  /**
   * Generate a secure random token.
   *
   * @param length - Token length
   * @returns Secure random token
   */
  async generateToken(length = 32): Promise<string> {
    await this.ensureInitialized();

    try {
      return this.securityEngine.generate_token(length);
    } catch (error) {
      throw new BridgeError(
        `Token generation failed: ${error instanceof Error ? error.message : String(error)}`,
        'TOKEN_ERROR',
        error
      );
    }
  }

  /**
   * Hash a password using a secure algorithm.
   *
   * @param password - Password to hash
   * @returns Hashed password
   */
  async hashPassword(password: string): Promise<string> {
    await this.ensureInitialized();

    try {
      const result = await this.securityEngine.hash_password(password);
      return result as string;
    } catch (error) {
      throw new BridgeError(
        `Password hashing failed: ${error instanceof Error ? error.message : String(error)}`,
        'HASH_ERROR',
        error
      );
    }
  }

  /**
   * Verify a password against a hash.
   *
   * @param password - Password to verify
   * @param hash - Password hash
   * @returns True if password matches hash
   */
  async verifyPassword(password: string, hash: string): Promise<boolean> {
    await this.ensureInitialized();

    try {
      const result = await this.securityEngine.verify_password(password, hash);
      return result as boolean;
    } catch (error) {
      throw new BridgeError(
        `Password verification failed: ${error instanceof Error ? error.message : String(error)}`,
        'VERIFY_ERROR',
        error
      );
    }
  }

  /**
   * Convenience methods for common validation types
   */

  /**
   * Validate input for XSS attacks.
   *
   * @param input - Input to validate
   * @returns Validation result
   */
  async validateXss(input: string): Promise<OperationResult<SecurityResult>> {
    return this.validate({
      checkType: 'xss',
      input,
    });
  }

  /**
   * Validate input for SQL injection.
   *
   * @param input - Input to validate
   * @returns Validation result
   */
  async validateSqlInjection(input: string): Promise<OperationResult<SecurityResult>> {
    return this.validate({
      checkType: 'sql_injection',
      input,
    });
  }

  /**
   * Validate CSRF token.
   *
   * @param token - Token to validate
   * @returns Validation result
   */
  async validateCsrf(token: string): Promise<OperationResult<SecurityResult>> {
    return this.validate({
      checkType: 'csrf',
      input: token,
    });
  }

  /**
   * Validate path for path traversal attacks.
   *
   * @param path - Path to validate
   * @returns Validation result
   */
  async validatePathTraversal(path: string): Promise<OperationResult<SecurityResult>> {
    return this.validate({
      checkType: 'path_traversal',
      input: path,
    });
  }

  /**
   * Validate input for command injection.
   *
   * @param input - Input to validate
   * @returns Validation result
   */
  async validateCommandInjection(input: string): Promise<OperationResult<SecurityResult>> {
    return this.validate({
      checkType: 'command_injection',
      input,
    });
  }

  /**
   * Convenience methods for common sanitization types
   */

  /**
   * Sanitize HTML input.
   *
   * @param html - HTML to sanitize
   * @returns Sanitized HTML
   */
  async sanitizeHtml(html: string): Promise<string> {
    return this.sanitize(html, 'html');
  }

  /**
   * Sanitize SQL input.
   *
   * @param sql - SQL to sanitize
   * @returns Sanitized SQL
   */
  async sanitizeSql(sql: string): Promise<string> {
    return this.sanitize(sql, 'sql');
  }

  /**
   * Sanitize URL.
   *
   * @param url - URL to sanitize
   * @returns Sanitized URL
   */
  async sanitizeUrl(url: string): Promise<string> {
    return this.sanitize(url, 'url');
  }

  /**
   * Sanitize filename.
   *
   * @param filename - Filename to sanitize
   * @returns Sanitized filename
   */
  async sanitizeFilename(filename: string): Promise<string> {
    return this.sanitize(filename, 'filename');
  }

  /**
   * Clean up resources.
   */
  dispose(): void {
    this.securityEngine = null;
  }
}
