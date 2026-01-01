/**
 * ABAC Engine - Attribute-Based Access Control
 * Fine-grained access control based on attributes and policies
 */

import { nanoid } from 'nanoid';

// ============================================================================
// Types
// ============================================================================

export interface AttributePolicy {
  id: string;
  name: string;
  description: string;
  effect: 'ALLOW' | 'DENY';
  rules: AttributeRule[];
  priority: number;
  enabled: boolean;
  metadata: Record<string, unknown>;
}

export interface AttributeRule {
  id: string;
  subject: AttributeCondition[];
  resource: AttributeCondition[];
  action: string[];
  environment?: AttributeCondition[];
  operator: 'AND' | 'OR';
}

export interface AttributeCondition {
  attribute: string;
  operator: ComparisonOperator;
  value: AttributeValue;
}

export type ComparisonOperator =
  | 'EQUALS'
  | 'NOT_EQUALS'
  | 'IN'
  | 'NOT_IN'
  | 'GREATER_THAN'
  | 'LESS_THAN'
  | 'GREATER_THAN_OR_EQUAL'
  | 'LESS_THAN_OR_EQUAL'
  | 'CONTAINS'
  | 'NOT_CONTAINS'
  | 'STARTS_WITH'
  | 'ENDS_WITH'
  | 'MATCHES'; // Regex

export type AttributeValue = string | number | boolean | string[] | number[];

export interface AttributeContext {
  subject: Record<string, AttributeValue>;
  resource: Record<string, AttributeValue>;
  action: string;
  environment: Record<string, AttributeValue>;
}

export interface AccessDecision {
  allowed: boolean;
  effect: 'ALLOW' | 'DENY' | 'NOT_APPLICABLE';
  matchedPolicies: string[];
  reason: string;
}

// ============================================================================
// ABAC Engine Implementation
// ============================================================================

export class ABACEngine {
  private policies: Map<string, AttributePolicy> = new Map();

  /**
   * Create attribute policy
   */
  async createPolicy(
    name: string,
    description: string,
    effect: 'ALLOW' | 'DENY',
    rules: AttributeRule[],
    priority: number = 0
  ): Promise<AttributePolicy> {
    const policy: AttributePolicy = {
      id: nanoid(),
      name,
      description,
      effect,
      rules,
      priority,
      enabled: true,
      metadata: {},
    };

    this.policies.set(policy.id, policy);
    return policy;
  }

  /**
   * Update policy
   */
  async updatePolicy(
    policyId: string,
    updates: Partial<Omit<AttributePolicy, 'id'>>
  ): Promise<AttributePolicy> {
    const policy = this.policies.get(policyId);
    if (!policy) {
      throw new Error('Policy not found');
    }

    const updatedPolicy: AttributePolicy = {
      ...policy,
      ...updates,
      id: policy.id,
    };

    this.policies.set(policyId, updatedPolicy);
    return updatedPolicy;
  }

  /**
   * Delete policy
   */
  async deletePolicy(policyId: string): Promise<void> {
    this.policies.delete(policyId);
  }

  /**
   * Evaluate access request
   */
  async evaluate(context: AttributeContext): Promise<AccessDecision> {
    const matchedPolicies: string[] = [];
    let finalEffect: 'ALLOW' | 'DENY' | 'NOT_APPLICABLE' = 'NOT_APPLICABLE';
    let highestPriority = -Infinity;

    // Get all enabled policies
    const enabledPolicies = Array.from(this.policies.values())
      .filter((p) => p.enabled)
      .sort((a, b) => b.priority - a.priority); // Higher priority first

    // Evaluate each policy
    for (const policy of enabledPolicies) {
      const matches = this.evaluatePolicy(policy, context);

      if (matches) {
        matchedPolicies.push(policy.id);

        // Higher priority policy wins
        if (policy.priority >= highestPriority) {
          highestPriority = policy.priority;
          finalEffect = policy.effect;

          // DENY always wins at same priority
          if (policy.effect === 'DENY') {
            break;
          }
        }
      }
    }

    const allowed = finalEffect === 'ALLOW';
    const reason = this.generateReason(finalEffect, matchedPolicies);

    return {
      allowed,
      effect: finalEffect,
      matchedPolicies,
      reason,
    };
  }

  /**
   * Check if action is allowed
   */
  async isAllowed(context: AttributeContext): Promise<boolean> {
    const decision = await this.evaluate(context);
    return decision.allowed;
  }

  /**
   * Get all policies
   */
  getAllPolicies(): AttributePolicy[] {
    return Array.from(this.policies.values());
  }

  /**
   * Get policy by ID
   */
  getPolicy(policyId: string): AttributePolicy | undefined {
    return this.policies.get(policyId);
  }

  /**
   * Enable policy
   */
  async enablePolicy(policyId: string): Promise<void> {
    const policy = this.policies.get(policyId);
    if (policy) {
      policy.enabled = true;
    }
  }

  /**
   * Disable policy
   */
  async disablePolicy(policyId: string): Promise<void> {
    const policy = this.policies.get(policyId);
    if (policy) {
      policy.enabled = false;
    }
  }

  // ============================================================================
  // Private Helper Methods
  // ============================================================================

  private evaluatePolicy(policy: AttributePolicy, context: AttributeContext): boolean {
    // Policy matches if any rule matches
    return policy.rules.some((rule) => this.evaluateRule(rule, context));
  }

  private evaluateRule(rule: AttributeRule, context: AttributeContext): boolean {
    // Check action
    if (!rule.action.includes(context.action)) {
      return false;
    }

    // Evaluate subject conditions
    const subjectMatch = this.evaluateConditions(
      rule.subject,
      context.subject,
      rule.operator
    );

    // Evaluate resource conditions
    const resourceMatch = this.evaluateConditions(
      rule.resource,
      context.resource,
      rule.operator
    );

    // Evaluate environment conditions
    const environmentMatch =
      !rule.environment ||
      rule.environment.length === 0 ||
      this.evaluateConditions(rule.environment, context.environment, rule.operator);

    return subjectMatch && resourceMatch && environmentMatch;
  }

  private evaluateConditions(
    conditions: AttributeCondition[],
    attributes: Record<string, AttributeValue>,
    operator: 'AND' | 'OR'
  ): boolean {
    if (conditions.length === 0) {
      return true;
    }

    const results = conditions.map((condition) =>
      this.evaluateCondition(condition, attributes)
    );

    if (operator === 'AND') {
      return results.every((r) => r);
    } else {
      return results.some((r) => r);
    }
  }

  private evaluateCondition(
    condition: AttributeCondition,
    attributes: Record<string, AttributeValue>
  ): boolean {
    const attrValue = attributes[condition.attribute];

    switch (condition.operator) {
      case 'EQUALS':
        return attrValue === condition.value;

      case 'NOT_EQUALS':
        return attrValue !== condition.value;

      case 'IN':
        if (Array.isArray(condition.value)) {
          return condition.value.includes(attrValue as string | number);
        }
        return false;

      case 'NOT_IN':
        if (Array.isArray(condition.value)) {
          return !condition.value.includes(attrValue as string | number);
        }
        return true;

      case 'GREATER_THAN':
        return typeof attrValue === 'number' &&
          typeof condition.value === 'number' &&
          attrValue > condition.value;

      case 'LESS_THAN':
        return typeof attrValue === 'number' &&
          typeof condition.value === 'number' &&
          attrValue < condition.value;

      case 'GREATER_THAN_OR_EQUAL':
        return typeof attrValue === 'number' &&
          typeof condition.value === 'number' &&
          attrValue >= condition.value;

      case 'LESS_THAN_OR_EQUAL':
        return typeof attrValue === 'number' &&
          typeof condition.value === 'number' &&
          attrValue <= condition.value;

      case 'CONTAINS':
        if (typeof attrValue === 'string' && typeof condition.value === 'string') {
          return attrValue.includes(condition.value);
        }
        if (Array.isArray(attrValue)) {
          return attrValue.includes(condition.value as string | number);
        }
        return false;

      case 'NOT_CONTAINS':
        if (typeof attrValue === 'string' && typeof condition.value === 'string') {
          return !attrValue.includes(condition.value);
        }
        if (Array.isArray(attrValue)) {
          return !attrValue.includes(condition.value as string | number);
        }
        return true;

      case 'STARTS_WITH':
        return typeof attrValue === 'string' &&
          typeof condition.value === 'string' &&
          attrValue.startsWith(condition.value);

      case 'ENDS_WITH':
        return typeof attrValue === 'string' &&
          typeof condition.value === 'string' &&
          attrValue.endsWith(condition.value);

      case 'MATCHES':
        if (typeof attrValue === 'string' && typeof condition.value === 'string') {
          try {
            const regex = new RegExp(condition.value);
            return regex.test(attrValue);
          } catch {
            return false;
          }
        }
        return false;

      default:
        return false;
    }
  }

  private generateReason(
    effect: 'ALLOW' | 'DENY' | 'NOT_APPLICABLE',
    matchedPolicies: string[]
  ): string {
    if (effect === 'NOT_APPLICABLE') {
      return 'No applicable policies found';
    }

    if (matchedPolicies.length === 0) {
      return `Access ${effect.toLowerCase()}ed by default policy`;
    }

    return `Access ${effect.toLowerCase()}ed by ${matchedPolicies.length} policy(ies)`;
  }

  /**
   * Test policy against context
   */
  async testPolicy(policyId: string, context: AttributeContext): Promise<boolean> {
    const policy = this.policies.get(policyId);
    if (!policy) {
      throw new Error('Policy not found');
    }

    return this.evaluatePolicy(policy, context);
  }

  /**
   * Get matching policies for context
   */
  async getMatchingPolicies(context: AttributeContext): Promise<AttributePolicy[]> {
    const matching: AttributePolicy[] = [];

    for (const policy of this.policies.values()) {
      if (policy.enabled && this.evaluatePolicy(policy, context)) {
        matching.push(policy);
      }
    }

    return matching.sort((a, b) => b.priority - a.priority);
  }
}

// Export singleton instance
export const abacEngine = new ABACEngine();
