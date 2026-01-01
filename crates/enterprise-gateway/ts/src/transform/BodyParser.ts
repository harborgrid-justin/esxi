/**
 * Enterprise API Gateway - Body Parser
 *
 * Parse and validate request/response bodies
 */

import type { GatewayRequest, GatewayResponse } from '../types';

export type ContentType =
  | 'application/json'
  | 'application/xml'
  | 'application/x-www-form-urlencoded'
  | 'multipart/form-data'
  | 'text/plain'
  | 'text/html';

export class BodyParser {
  /**
   * Parse request body based on Content-Type
   */
  public parseRequest(request: GatewayRequest): GatewayRequest {
    const contentType = this.getContentType(request.headers['content-type']);

    if (!contentType || !request.body) {
      return request;
    }

    let parsedBody: unknown;

    try {
      switch (contentType) {
        case 'application/json':
          parsedBody = this.parseJSON(request.body);
          break;

        case 'application/x-www-form-urlencoded':
          parsedBody = this.parseFormUrlEncoded(request.body);
          break;

        case 'text/plain':
        case 'text/html':
          parsedBody = this.parseText(request.body);
          break;

        default:
          parsedBody = request.body;
      }

      return { ...request, body: parsedBody };
    } catch (error) {
      throw new Error(`Failed to parse request body: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Parse response body based on Content-Type
   */
  public parseResponse(response: GatewayResponse): GatewayResponse {
    const contentType = this.getContentType(response.headers['content-type']);

    if (!contentType || !response.body) {
      return response;
    }

    let parsedBody: unknown;

    try {
      switch (contentType) {
        case 'application/json':
          parsedBody = this.parseJSON(response.body);
          break;

        case 'text/plain':
        case 'text/html':
          parsedBody = this.parseText(response.body);
          break;

        default:
          parsedBody = response.body;
      }

      return { ...response, body: parsedBody };
    } catch (error) {
      throw new Error(`Failed to parse response body: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Parse JSON body
   */
  private parseJSON(body: unknown): unknown {
    if (typeof body === 'string') {
      return JSON.parse(body);
    }
    return body;
  }

  /**
   * Parse form-urlencoded body
   */
  private parseFormUrlEncoded(body: unknown): Record<string, string> {
    if (typeof body !== 'string') {
      return {};
    }

    const params = new URLSearchParams(body);
    const result: Record<string, string> = {};

    for (const [key, value] of params.entries()) {
      result[key] = value;
    }

    return result;
  }

  /**
   * Parse text body
   */
  private parseText(body: unknown): string {
    if (typeof body === 'string') {
      return body;
    }
    return String(body);
  }

  /**
   * Stringify body for transmission
   */
  public stringify(body: unknown, contentType: ContentType = 'application/json'): string {
    switch (contentType) {
      case 'application/json':
        return JSON.stringify(body);

      case 'application/x-www-form-urlencoded':
        return this.stringifyFormUrlEncoded(body);

      case 'text/plain':
      case 'text/html':
        return String(body);

      default:
        return String(body);
    }
  }

  /**
   * Stringify to form-urlencoded
   */
  private stringifyFormUrlEncoded(body: unknown): string {
    if (typeof body !== 'object' || body === null) {
      return '';
    }

    const params = new URLSearchParams();

    for (const [key, value] of Object.entries(body)) {
      params.append(key, String(value));
    }

    return params.toString();
  }

  /**
   * Get content type from header
   */
  private getContentType(header: string | string[] | undefined): ContentType | null {
    if (!header) {
      return null;
    }

    const contentTypeStr = Array.isArray(header) ? header[0] : header;
    if (!contentTypeStr) {
      return null;
    }

    const [type] = contentTypeStr.toLowerCase().split(';');

    switch (type?.trim()) {
      case 'application/json':
        return 'application/json';
      case 'application/xml':
        return 'application/xml';
      case 'application/x-www-form-urlencoded':
        return 'application/x-www-form-urlencoded';
      case 'multipart/form-data':
        return 'multipart/form-data';
      case 'text/plain':
        return 'text/plain';
      case 'text/html':
        return 'text/html';
      default:
        return null;
    }
  }

  /**
   * Validate body against schema (basic validation)
   */
  public validate(body: unknown, schema: Record<string, any>): boolean {
    // This is a simplified validation - in production, use a library like Zod or Joi
    if (typeof body !== 'object' || body === null) {
      return false;
    }

    for (const [key, validator] of Object.entries(schema)) {
      const value = (body as Record<string, unknown>)[key];

      if (validator.required && value === undefined) {
        return false;
      }

      if (value !== undefined && validator.type && typeof value !== validator.type) {
        return false;
      }
    }

    return true;
  }

  /**
   * Calculate body size in bytes
   */
  public getSize(body: unknown): number {
    const str = typeof body === 'string' ? body : JSON.stringify(body);
    return new Blob([str]).size;
  }

  /**
   * Check if body is empty
   */
  public isEmpty(body: unknown): boolean {
    if (body === null || body === undefined) {
      return true;
    }

    if (typeof body === 'string') {
      return body.trim().length === 0;
    }

    if (typeof body === 'object') {
      return Object.keys(body).length === 0;
    }

    return false;
  }
}
