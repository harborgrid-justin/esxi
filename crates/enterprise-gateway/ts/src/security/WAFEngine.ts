/**
 * Enterprise API Gateway - WAF Engine
 *
 * Web Application Firewall for threat detection and prevention
 */

import type { WAFRule, WAFResult, GatewayRequest, WAFRuleType, WAFAction } from '../types';
import { AuthorizationError } from '../types';

export class WAFEngine {
  private rules: Map<string, WAFRule> = new Map();
  private readonly builtInPatterns: Map<WAFRuleType, RegExp[]>;

  constructor() {
    this.builtInPatterns = new Map([
      [
        'sql-injection',
        [
          /(\b(SELECT|INSERT|UPDATE|DELETE|DROP|CREATE|ALTER|EXEC|EXECUTE)\b)/i,
          /(UNION\s+SELECT)/i,
          /('|"|;|--|\bOR\b|\bAND\b).*?(\bOR\b|\bAND\b).*?=/i,
          /(;|\-\-|\/\*|\*\/|xp_|sp_)/i,
        ],
      ],
      [
        'xss',
        [
          /<script[^>]*>.*?<\/script>/i,
          /javascript:/i,
          /on\w+\s*=/i,
          /<iframe[^>]*>/i,
          /<object[^>]*>/i,
          /<embed[^>]*>/i,
          /eval\s*\(/i,
        ],
      ],
      [
        'path-traversal',
        [
          /\.\.[\/\\]/,
          /%2e%2e[\/\\]/i,
          /\.\.[%2f%5c]/i,
          /(\.\.\\|\.\.\/)/,
        ],
      ],
      [
        'command-injection',
        [
          /[;&|`$()]/,
          /(;|\||&|\$\(|\`)/,
          /(bash|sh|cmd|powershell|exec)/i,
        ],
      ],
    ]);

    this.initializeDefaultRules();
  }

  /**
   * Initialize default WAF rules
   */
  private initializeDefaultRules(): void {
    this.addRule({
      id: 'sql-injection-1',
      type: 'sql-injection',
      action: 'block',
      message: 'Potential SQL injection detected',
      enabled: true,
      severity: 'high',
    });

    this.addRule({
      id: 'xss-1',
      type: 'xss',
      action: 'block',
      message: 'Potential XSS attack detected',
      enabled: true,
      severity: 'high',
    });

    this.addRule({
      id: 'path-traversal-1',
      type: 'path-traversal',
      action: 'block',
      message: 'Potential path traversal attack detected',
      enabled: true,
      severity: 'high',
    });

    this.addRule({
      id: 'command-injection-1',
      type: 'command-injection',
      action: 'block',
      message: 'Potential command injection detected',
      enabled: true,
      severity: 'critical',
    });
  }

  /**
   * Add a WAF rule
   */
  public addRule(rule: WAFRule): void {
    this.rules.set(rule.id, rule);
  }

  /**
   * Remove a WAF rule
   */
  public removeRule(ruleId: string): void {
    this.rules.delete(ruleId);
  }

  /**
   * Enable/disable a rule
   */
  public setRuleEnabled(ruleId: string, enabled: boolean): void {
    const rule = this.rules.get(ruleId);
    if (rule) {
      rule.enabled = enabled;
      this.rules.set(ruleId, rule);
    }
  }

  /**
   * Analyze request for threats
   */
  public analyze(request: GatewayRequest): WAFResult {
    const matchedRules: WAFRule[] = [];
    let action: WAFAction = 'log';

    for (const rule of this.rules.values()) {
      if (!rule.enabled) continue;

      if (this.checkRule(rule, request)) {
        matchedRules.push(rule);

        // Determine most restrictive action
        if (rule.action === 'block' || action === 'log') {
          action = rule.action;
        }
      }
    }

    return {
      allowed: action !== 'block',
      matchedRules,
      action,
    };
  }

  /**
   * Check if request matches a rule
   */
  private checkRule(rule: WAFRule, request: GatewayRequest): boolean {
    // Get patterns for rule type
    const patterns = rule.pattern
      ? [rule.pattern]
      : this.builtInPatterns.get(rule.type) || [];

    // Check path
    if (this.matchPatterns(patterns, request.path)) {
      return true;
    }

    // Check query parameters
    for (const value of Object.values(request.query)) {
      const queryValue = Array.isArray(value) ? value.join(' ') : value;
      if (this.matchPatterns(patterns, queryValue)) {
        return true;
      }
    }

    // Check headers
    for (const value of Object.values(request.headers)) {
      const headerValue = Array.isArray(value) ? value.join(' ') : value?.toString() || '';
      if (this.matchPatterns(patterns, headerValue)) {
        return true;
      }
    }

    // Check body
    if (request.body) {
      const bodyStr = JSON.stringify(request.body);
      if (this.matchPatterns(patterns, bodyStr)) {
        return true;
      }
    }

    return false;
  }

  /**
   * Check if text matches any pattern
   */
  private matchPatterns(patterns: Array<string | RegExp>, text: string): boolean {
    for (const pattern of patterns) {
      if (pattern instanceof RegExp) {
        if (pattern.test(text)) {
          return true;
        }
      } else {
        if (text.includes(pattern)) {
          return true;
        }
      }
    }

    return false;
  }

  /**
   * Validate request or throw error
   */
  public validate(request: GatewayRequest): void {
    const result = this.analyze(request);

    if (!result.allowed) {
      const messages = result.matchedRules.map((r) => r.message).join(', ');
      throw new AuthorizationError(`WAF blocked request: ${messages}`);
    }
  }

  /**
   * Get all rules
   */
  public getRules(): WAFRule[] {
    return Array.from(this.rules.values());
  }

  /**
   * Get rule by ID
   */
  public getRule(id: string): WAFRule | undefined {
    return this.rules.get(id);
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    totalRules: number;
    enabledRules: number;
    rulesBySeverity: Record<string, number>;
    rulesByType: Record<string, number>;
  } {
    let enabled = 0;
    const bySeverity: Record<string, number> = {};
    const byType: Record<string, number> = {};

    for (const rule of this.rules.values()) {
      if (rule.enabled) enabled++;

      bySeverity[rule.severity] = (bySeverity[rule.severity] || 0) + 1;
      byType[rule.type] = (byType[rule.type] || 0) + 1;
    }

    return {
      totalRules: this.rules.size,
      enabledRules: enabled,
      rulesBySeverity: bySeverity,
      rulesByType: byType,
    };
  }
}
