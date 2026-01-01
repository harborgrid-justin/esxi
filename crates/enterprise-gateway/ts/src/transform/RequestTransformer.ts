/**
 * Enterprise API Gateway - Request Transformer
 *
 * Request modification and transformation
 */

import type { GatewayRequest, TransformRule } from '../types';

export class RequestTransformer {
  private rules: TransformRule[] = [];

  constructor(rules: TransformRule[] = []) {
    this.rules = rules;
  }

  /**
   * Transform request based on rules
   */
  public transform(request: GatewayRequest): GatewayRequest {
    let transformed = { ...request };

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
  private shouldApplyRule(rule: TransformRule, request: GatewayRequest): boolean {
    if (!rule.condition) {
      return true;
    }

    try {
      // Evaluate condition as JavaScript expression
      // In production, use a safe expression evaluator
      const conditionFunc = new Function('request', `return ${rule.condition}`);
      return conditionFunc(request);
    } catch {
      return false;
    }
  }

  /**
   * Apply transformation rule
   */
  private applyRule(rule: TransformRule, request: GatewayRequest): GatewayRequest {
    switch (rule.target) {
      case 'header':
        return this.transformHeader(rule, request);
      case 'query':
        return this.transformQuery(rule, request);
      case 'body':
        return this.transformBody(rule, request);
      case 'path':
        return this.transformPath(rule, request);
      default:
        return request;
    }
  }

  /**
   * Transform header
   */
  private transformHeader(rule: TransformRule, request: GatewayRequest): GatewayRequest {
    const headers = { ...request.headers };

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

    return { ...request, headers };
  }

  /**
   * Transform query parameters
   */
  private transformQuery(rule: TransformRule, request: GatewayRequest): GatewayRequest {
    const query = { ...request.query };

    switch (rule.type) {
      case 'add':
        if (rule.value !== undefined) {
          query[rule.field] = rule.value;
        }
        break;

      case 'remove':
        delete query[rule.field];
        break;

      case 'replace':
        if (rule.value !== undefined && query[rule.field] !== undefined) {
          query[rule.field] = rule.value;
        }
        break;

      case 'rename':
        if (rule.newField && query[rule.field] !== undefined) {
          query[rule.newField] = query[rule.field];
          delete query[rule.field];
        }
        break;
    }

    return { ...request, query };
  }

  /**
   * Transform body
   */
  private transformBody(rule: TransformRule, request: GatewayRequest): GatewayRequest {
    if (!request.body || typeof request.body !== 'object') {
      return request;
    }

    const body = { ...request.body as Record<string, unknown> };

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

    return { ...request, body };
  }

  /**
   * Transform path
   */
  private transformPath(rule: TransformRule, request: GatewayRequest): GatewayRequest {
    let path = request.path;

    switch (rule.type) {
      case 'replace':
        if (rule.value !== undefined) {
          path = path.replace(new RegExp(rule.field, 'g'), rule.value);
        }
        break;

      case 'add':
        if (rule.value !== undefined) {
          path = rule.field === 'prefix' ? rule.value + path : path + rule.value;
        }
        break;

      case 'remove':
        path = path.replace(new RegExp(rule.field, 'g'), '');
        break;
    }

    return { ...request, path };
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
