/**
 * Enterprise API Gateway - Request Sanitizer
 *
 * Input sanitization and normalization
 */

import type { GatewayRequest } from '../types';

export class RequestSanitizer {
  /**
   * Sanitize entire request
   */
  public sanitize(request: GatewayRequest): GatewayRequest {
    return {
      ...request,
      path: this.sanitizePath(request.path),
      query: this.sanitizeQuery(request.query),
      headers: this.sanitizeHeaders(request.headers),
      body: this.sanitizeBody(request.body),
    };
  }

  /**
   * Sanitize path
   */
  private sanitizePath(path: string): string {
    // Remove dangerous characters
    let sanitized = path.replace(/[<>'"]/g, '');

    // Normalize path separators
    sanitized = sanitized.replace(/\\/g, '/');

    // Remove double slashes
    sanitized = sanitized.replace(/\/+/g, '/');

    // Remove path traversal attempts
    sanitized = sanitized.replace(/\.\.\/*/g, '');

    // Decode URL encoding
    try {
      sanitized = decodeURIComponent(sanitized);
    } catch {
      // Invalid encoding, use as-is
    }

    return sanitized;
  }

  /**
   * Sanitize query parameters
   */
  private sanitizeQuery(query: Record<string, string | string[]>): Record<string, string | string[]> {
    const sanitized: Record<string, string | string[]> = {};

    for (const [key, value] of Object.entries(query)) {
      const sanitizedKey = this.sanitizeString(key);

      if (Array.isArray(value)) {
        sanitized[sanitizedKey] = value.map((v) => this.sanitizeString(v));
      } else {
        sanitized[sanitizedKey] = this.sanitizeString(value);
      }
    }

    return sanitized;
  }

  /**
   * Sanitize headers
   */
  private sanitizeHeaders(headers: Record<string, string | string[] | undefined>): Record<string, string | string[] | undefined> {
    const sanitized: Record<string, string | string[] | undefined> = {};

    for (const [key, value] of Object.entries(headers)) {
      // Header names are case-insensitive, convert to lowercase
      const sanitizedKey = key.toLowerCase();

      if (value === undefined) {
        sanitized[sanitizedKey] = undefined;
      } else if (Array.isArray(value)) {
        sanitized[sanitizedKey] = value.map((v) => this.sanitizeHeaderValue(v));
      } else {
        sanitized[sanitizedKey] = this.sanitizeHeaderValue(value);
      }
    }

    return sanitized;
  }

  /**
   * Sanitize header value
   */
  private sanitizeHeaderValue(value: string): string {
    // Remove control characters and newlines (prevent header injection)
    return value.replace(/[\r\n\x00-\x1f\x7f]/g, '');
  }

  /**
   * Sanitize body
   */
  private sanitizeBody(body: unknown): unknown {
    if (!body) return body;

    if (typeof body === 'string') {
      return this.sanitizeString(body);
    }

    if (Array.isArray(body)) {
      return body.map((item) => this.sanitizeBody(item));
    }

    if (typeof body === 'object') {
      const sanitized: Record<string, unknown> = {};

      for (const [key, value] of Object.entries(body)) {
        const sanitizedKey = this.sanitizeString(key);
        sanitized[sanitizedKey] = this.sanitizeBody(value);
      }

      return sanitized;
    }

    return body;
  }

  /**
   * Sanitize string value
   */
  private sanitizeString(value: string): string {
    // Remove null bytes
    let sanitized = value.replace(/\x00/g, '');

    // Trim whitespace
    sanitized = sanitized.trim();

    // Normalize unicode
    try {
      sanitized = sanitized.normalize('NFC');
    } catch {
      // Normalization failed, use as-is
    }

    return sanitized;
  }

  /**
   * Sanitize HTML (escape dangerous characters)
   */
  public sanitizeHTML(html: string): string {
    return html
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#x27;')
      .replace(/\//g, '&#x2F;');
  }

  /**
   * Sanitize SQL (escape dangerous characters)
   */
  public sanitizeSQL(sql: string): string {
    return sql.replace(/'/g, "''");
  }

  /**
   * Sanitize for JSON
   */
  public sanitizeJSON(value: string): string {
    return value
      .replace(/\\/g, '\\\\')
      .replace(/"/g, '\\"')
      .replace(/\n/g, '\\n')
      .replace(/\r/g, '\\r')
      .replace(/\t/g, '\\t');
  }

  /**
   * Remove all HTML tags
   */
  public stripHTML(html: string): string {
    return html.replace(/<[^>]*>/g, '');
  }

  /**
   * Validate email format
   */
  public isValidEmail(email: string): boolean {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  }

  /**
   * Validate URL format
   */
  public isValidURL(url: string): boolean {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Sanitize filename (remove path traversal)
   */
  public sanitizeFilename(filename: string): string {
    return filename
      .replace(/[\/\\]/g, '')
      .replace(/\.\./g, '')
      .replace(/[<>:"|?*\x00-\x1f]/g, '')
      .trim();
  }

  /**
   * Truncate string to max length
   */
  public truncate(value: string, maxLength: number): string {
    if (value.length <= maxLength) {
      return value;
    }

    return value.substring(0, maxLength);
  }
}
