/**
 * Enterprise API Gateway - Response Transformer
 *
 * Response modification and transformation
 */

import type { GatewayResponse, TransformRule } from '../types';

export class ResponseTransformer {
  private rules: TransformRule[] = [];

  constructor(rules: TransformRule[] = []) {
    this.rules = rules;
  }

  /**
   * Transform response based on rules
   */
  public transform(response: GatewayResponse): GatewayResponse {
    let transformed = { ...response };

    for (const rule of this.rules) {
      if (this.shouldApplyRule(rule, transformed)) {
        transformed = this.applyRule(rule, transformed);
      }
    }

    return transformed;
  }

  /**
   * Check if rule should be applied
   */
  private shouldApplyRule(rule: TransformRule, response: GatewayResponse): boolean {
    if (!rule.condition) {
      return true;
    }

    try {
      const conditionFunc = new Function('response', `return ${rule.condition}`);
      return conditionFunc(response);
    } catch {
      return false;
    }
  }

  /**
   * Apply transformation rule
   */
  private applyRule(rule: TransformRule, response: GatewayResponse): GatewayResponse {
    switch (rule.target) {
      case 'header':
        return this.transformHeader(rule, response);
      case 'body':
        return this.transformBody(rule, response);
      default:
        return response;
    }
  }

  /**
   * Transform header
   */
  private transformHeader(rule: TransformRule, response: GatewayResponse): GatewayResponse {
    const headers = { ...response.headers };

    switch (rule.type) {
      case 'add':
        if (rule.value !== undefined) {
          headers[rule.field] = rule.value;
        }
        break;

      case 'remove':
        delete headers[rule.field];
        break;

      case 'replace':
        if (rule.value !== undefined && headers[rule.field] !== undefined) {
          headers[rule.field] = rule.value;
        }
        break;

      case 'rename':
        if (rule.newField && headers[rule.field] !== undefined) {
          headers[rule.newField] = headers[rule.field];
          delete headers[rule.field];
        }
        break;
    }

    return { ...response, headers };
  }

  /**
   * Transform body
   */
  private transformBody(rule: TransformRule, response: GatewayResponse): GatewayResponse {
    if (!response.body || typeof response.body !== 'object') {
      return response;
    }

    const body = { ...response.body as Record<string, unknown> };

    switch (rule.type) {
      case 'add':
        if (rule.value !== undefined) {
          body[rule.field] = rule.value;
        }
        break;

      case 'remove':
        delete body[rule.field];
        break;

      case 'replace':
        if (rule.value !== undefined && body[rule.field] !== undefined) {
          body[rule.field] = rule.value;
        }
        break;

      case 'rename':
        if (rule.newField && body[rule.field] !== undefined) {
          body[rule.newField] = body[rule.field];
          delete body[rule.field];
        }
        break;
    }

    return { ...response, body };
  }

  /**
   * Add CORS headers
   */
  public addCORSHeaders(
    response: GatewayResponse,
    origin: string,
    methods: string[] = ['GET', 'POST', 'PUT', 'DELETE'],
    headers: string[] = ['Content-Type', 'Authorization']
  ): GatewayResponse {
    const corsHeaders = {
      ...response.headers,
      'Access-Control-Allow-Origin': origin,
      'Access-Control-Allow-Methods': methods.join(', '),
      'Access-Control-Allow-Headers': headers.join(', '),
      'Access-Control-Allow-Credentials': 'true',
    };

    return { ...response, headers: corsHeaders };
  }

  /**
   * Add cache headers
   */
  public addCacheHeaders(
    response: GatewayResponse,
    maxAge: number,
    isPublic = true
  ): GatewayResponse {
    const cacheHeaders = {
      ...response.headers,
      'Cache-Control': `${isPublic ? 'public' : 'private'}, max-age=${maxAge}`,
      'Expires': new Date(Date.now() + maxAge * 1000).toUTCString(),
    };

    return { ...response, headers: cacheHeaders };
  }

  /**
   * Add security headers
   */
  public addSecurityHeaders(response: GatewayResponse): GatewayResponse {
    const securityHeaders = {
      ...response.headers,
      'X-Content-Type-Options': 'nosniff',
      'X-Frame-Options': 'DENY',
      'X-XSS-Protection': '1; mode=block',
      'Strict-Transport-Security': 'max-age=31536000; includeSubDomains',
      'Content-Security-Policy': "default-src 'self'",
    };

    return { ...response, headers: securityHeaders };
  }

  /**
   * Add transformation rule
   */
  public addRule(rule: TransformRule): void {
    this.rules.push(rule);
  }

  /**
   * Remove transformation rule
   */
  public removeRule(ruleId: string): void {
    this.rules = this.rules.filter((r) => r.id !== ruleId);
  }

  /**
   * Get all rules
   */
  public getRules(): TransformRule[] {
    return [...this.rules];
  }

  /**
   * Clear all rules
   */
  public clearRules(): void {
    this.rules = [];
  }
}
