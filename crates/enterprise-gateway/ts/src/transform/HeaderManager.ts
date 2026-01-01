/**
 * Enterprise API Gateway - Header Manager
 *
 * Advanced header manipulation and management
 */

import type { HTTPHeaders, GatewayRequest, GatewayResponse } from '../types';

export class HeaderManager {
  /**
   * Add header to request
   */
  public addRequestHeader(request: GatewayRequest, name: string, value: string): GatewayRequest {
    return {
      ...request,
      headers: {
        ...request.headers,
        [name.toLowerCase()]: value,
      },
    };
  }

  /**
   * Add header to response
   */
  public addResponseHeader(response: GatewayResponse, name: string, value: string): GatewayResponse {
    return {
      ...response,
      headers: {
        ...response.headers,
        [name.toLowerCase()]: value,
      },
    };
  }

  /**
   * Remove header from request
   */
  public removeRequestHeader(request: GatewayRequest, name: string): GatewayRequest {
    const headers = { ...request.headers };
    delete headers[name.toLowerCase()];
    return { ...request, headers };
  }

  /**
   * Remove header from response
   */
  public removeResponseHeader(response: GatewayResponse, name: string): GatewayResponse {
    const headers = { ...response.headers };
    delete headers[name.toLowerCase()];
    return { ...response, headers };
  }

  /**
   * Get header value
   */
  public getHeader(headers: HTTPHeaders, name: string): string | string[] | undefined {
    return headers[name.toLowerCase()];
  }

  /**
   * Check if header exists
   */
  public hasHeader(headers: HTTPHeaders, name: string): boolean {
    return headers[name.toLowerCase()] !== undefined;
  }

  /**
   * Set multiple headers
   */
  public setHeaders(
    headers: HTTPHeaders,
    newHeaders: Record<string, string>
  ): HTTPHeaders {
    const result = { ...headers };

    for (const [name, value] of Object.entries(newHeaders)) {
      result[name.toLowerCase()] = value;
    }

    return result;
  }

  /**
   * Add trace headers for distributed tracing
   */
  public addTraceHeaders(request: GatewayRequest, traceId?: string, spanId?: string): GatewayRequest {
    const headers = { ...request.headers };

    headers['x-request-id'] = request.id;
    if (traceId) headers['x-trace-id'] = traceId;
    if (spanId) headers['x-span-id'] = spanId;

    return { ...request, headers };
  }

  /**
   * Add forwarding headers
   */
  public addForwardHeaders(request: GatewayRequest, clientIp: string): GatewayRequest {
    const headers = { ...request.headers };

    // Add or append to X-Forwarded-For
    const existingForwardedFor = headers['x-forwarded-for'];
    if (existingForwardedFor) {
      const current = Array.isArray(existingForwardedFor)
        ? existingForwardedFor.join(', ')
        : existingForwardedFor;
      headers['x-forwarded-for'] = `${current}, ${clientIp}`;
    } else {
      headers['x-forwarded-for'] = clientIp;
    }

    // Add X-Real-IP
    if (!headers['x-real-ip']) {
      headers['x-real-ip'] = clientIp;
    }

    // Add X-Forwarded-Proto
    const isHttps = request.headers['x-forwarded-proto'] === 'https';
    headers['x-forwarded-proto'] = isHttps ? 'https' : 'http';

    // Add X-Forwarded-Host
    const host = request.headers['host'];
    if (host) {
      headers['x-forwarded-host'] = Array.isArray(host) ? host[0]! : host;
    }

    return { ...request, headers };
  }

  /**
   * Remove hop-by-hop headers
   */
  public removeHopByHopHeaders(headers: HTTPHeaders): HTTPHeaders {
    const hopByHopHeaders = [
      'connection',
      'keep-alive',
      'proxy-authenticate',
      'proxy-authorization',
      'te',
      'trailers',
      'transfer-encoding',
      'upgrade',
    ];

    const result = { ...headers };

    for (const header of hopByHopHeaders) {
      delete result[header];
    }

    return result;
  }

  /**
   * Normalize headers (lowercase keys)
   */
  public normalize(headers: HTTPHeaders): HTTPHeaders {
    const result: HTTPHeaders = {};

    for (const [key, value] of Object.entries(headers)) {
      result[key.toLowerCase()] = value;
    }

    return result;
  }

  /**
   * Merge headers (second takes precedence)
   */
  public merge(headers1: HTTPHeaders, headers2: HTTPHeaders): HTTPHeaders {
    return {
      ...this.normalize(headers1),
      ...this.normalize(headers2),
    };
  }

  /**
   * Get authorization token
   */
  public getAuthToken(headers: HTTPHeaders): string | null {
    const authHeader = headers['authorization'];

    if (!authHeader) {
      return null;
    }

    const authValue = Array.isArray(authHeader) ? authHeader[0] : authHeader;
    if (!authValue) {
      return null;
    }

    const match = authValue.match(/^Bearer (.+)$/);
    return match ? match[1]! : null;
  }

  /**
   * Set authorization header
   */
  public setAuthToken(headers: HTTPHeaders, token: string, type = 'Bearer'): HTTPHeaders {
    return {
      ...headers,
      'authorization': `${type} ${token}`,
    };
  }

  /**
   * Parse content type
   */
  public parseContentType(headers: HTTPHeaders): {
    type: string;
    charset?: string;
    boundary?: string;
  } | null {
    const contentType = headers['content-type'];

    if (!contentType) {
      return null;
    }

    const value = Array.isArray(contentType) ? contentType[0] : contentType;
    if (!value) {
      return null;
    }

    const parts = value.split(';').map((p) => p.trim());
    const type = parts[0]!;
    const result: { type: string; charset?: string; boundary?: string } = { type };

    for (let i = 1; i < parts.length; i++) {
      const part = parts[i];
      if (!part) continue;

      const [key, val] = part.split('=');
      if (key && val) {
        if (key.trim() === 'charset') {
          result.charset = val.trim();
        } else if (key.trim() === 'boundary') {
          result.boundary = val.trim();
        }
      }
    }

    return result;
  }

  /**
   * Convert headers to HTTP/1.1 format
   */
  public toHTTP1(headers: HTTPHeaders): string {
    const lines: string[] = [];

    for (const [key, value] of Object.entries(headers)) {
      if (value === undefined) continue;

      const values = Array.isArray(value) ? value : [value];

      for (const val of values) {
        lines.push(`${key}: ${val}`);
      }
    }

    return lines.join('\r\n');
  }
}
