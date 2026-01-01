/**
 * RuleEvaluator - Alert rule evaluation engine
 * Evaluates conditions and triggers alerts based on rules
 */

import { EventEmitter } from 'events';
import { AlertRule, RuleCondition, RuleConditionOperator, Threshold } from '../types';

export interface EvaluationContext {
  data: Record<string, unknown>;
  metrics?: Record<string, number>;
  metadata?: Record<string, unknown>;
  timestamp?: Date;
}

export interface EvaluationResult {
  ruleId: string;
  matched: boolean;
  conditions: ConditionResult[];
  thresholds: ThresholdResult[];
  metadata?: Record<string, unknown>;
}

export interface ConditionResult {
  condition: RuleCondition;
  matched: boolean;
  actualValue?: unknown;
  expectedValue?: unknown;
}

export interface ThresholdResult {
  threshold: Threshold;
  exceeded: boolean;
  currentValue?: number;
  thresholdValue?: number;
}

export class RuleEvaluator extends EventEmitter {
  private rules: Map<string, AlertRule>;
  private evaluationHistory: Map<string, EvaluationResult[]>;

  constructor() {
    super();
    this.rules = new Map();
    this.evaluationHistory = new Map();
  }

  /**
   * Register alert rule
   */
  registerRule(rule: AlertRule): void {
    this.rules.set(rule.id, rule);
    this.emit('rule:registered', rule);
  }

  /**
   * Unregister alert rule
   */
  unregisterRule(ruleId: string): void {
    this.rules.delete(ruleId);
    this.evaluationHistory.delete(ruleId);
    this.emit('rule:unregistered', ruleId);
  }

  /**
   * Evaluate all rules against context
   */
  evaluateAll(context: EvaluationContext): EvaluationResult[] {
    const results: EvaluationResult[] = [];

    for (const rule of this.rules.values()) {
      if (!rule.enabled) {
        continue;
      }

      const result = this.evaluateRule(rule, context);
      results.push(result);

      if (result.matched) {
        this.emit('rule:matched', rule, result);
      }
    }

    return results;
  }

  /**
   * Evaluate single rule
   */
  evaluateRule(rule: AlertRule, context: EvaluationContext): EvaluationResult {
    const conditionResults = rule.conditions.map(condition =>
      this.evaluateCondition(condition, context)
    );

    const thresholdResults = rule.thresholds.map(threshold =>
      this.evaluateThreshold(threshold, context)
    );

    // Combine condition results based on operator
    let conditionsMatched = false;
    if (rule.conditionOperator === 'AND') {
      conditionsMatched = conditionResults.every(r => r.matched);
    } else {
      conditionsMatched = conditionResults.some(r => r.matched);
    }

    // Check if any thresholds exceeded
    const thresholdsExceeded = thresholdResults.some(r => r.exceeded);

    const result: EvaluationResult = {
      ruleId: rule.id,
      matched: conditionsMatched && (thresholdResults.length === 0 || thresholdsExceeded),
      conditions: conditionResults,
      thresholds: thresholdResults,
      metadata: {
        evaluatedAt: new Date(),
        conditionOperator: rule.conditionOperator,
      },
    };

    // Store in history
    this.addToHistory(rule.id, result);

    return result;
  }

  /**
   * Evaluate condition
   */
  private evaluateCondition(
    condition: RuleCondition,
    context: EvaluationContext
  ): ConditionResult {
    const actualValue = this.resolveValue(condition.field, context);
    const expectedValue =
      condition.valueType === 'dynamic'
        ? this.resolveValue(String(condition.value), context)
        : condition.value;

    const matched = this.compareValues(actualValue, condition.operator, expectedValue);

    return {
      condition,
      matched,
      actualValue,
      expectedValue,
    };
  }

  /**
   * Evaluate threshold
   */
  private evaluateThreshold(
    threshold: Threshold,
    context: EvaluationContext
  ): ThresholdResult {
    const currentValue = context.metrics?.[threshold.metric];

    if (currentValue === undefined) {
      return {
        threshold,
        exceeded: false,
      };
    }

    let thresholdValue = threshold.value;

    // Handle dynamic thresholds
    if (threshold.type === 'dynamic' && threshold.baselineWindow && threshold.deviationMultiplier) {
      const baseline = this.calculateBaseline(threshold.metric, threshold.baselineWindow);
      thresholdValue = baseline * threshold.deviationMultiplier;
    }

    // Handle percentage thresholds
    if (threshold.type === 'percentage' && threshold.percentageOf) {
      const baseValue = context.metrics?.[threshold.percentageOf] ?? 0;
      thresholdValue = (baseValue * threshold.value) / 100;
    }

    const exceeded = this.compareValues(currentValue, threshold.operator, thresholdValue);

    return {
      threshold,
      exceeded,
      currentValue,
      thresholdValue,
    };
  }

  /**
   * Resolve value from context
   */
  private resolveValue(field: string, context: EvaluationContext): unknown {
    const parts = field.split('.');
    let value: unknown = context.data;

    for (const part of parts) {
      if (value && typeof value === 'object' && part in value) {
        value = (value as Record<string, unknown>)[part];
      } else {
        return undefined;
      }
    }

    return value;
  }

  /**
   * Compare values based on operator
   */
  private compareValues(
    actual: unknown,
    operator: RuleConditionOperator,
    expected: unknown
  ): boolean {
    switch (operator) {
      case RuleConditionOperator.EQUALS:
        return actual === expected;

      case RuleConditionOperator.NOT_EQUALS:
        return actual !== expected;

      case RuleConditionOperator.GREATER_THAN:
        return typeof actual === 'number' && typeof expected === 'number' && actual > expected;

      case RuleConditionOperator.GREATER_THAN_OR_EQUAL:
        return typeof actual === 'number' && typeof expected === 'number' && actual >= expected;

      case RuleConditionOperator.LESS_THAN:
        return typeof actual === 'number' && typeof expected === 'number' && actual < expected;

      case RuleConditionOperator.LESS_THAN_OR_EQUAL:
        return typeof actual === 'number' && typeof expected === 'number' && actual <= expected;

      case RuleConditionOperator.CONTAINS:
        return (
          typeof actual === 'string' &&
          typeof expected === 'string' &&
          actual.includes(expected)
        );

      case RuleConditionOperator.NOT_CONTAINS:
        return (
          typeof actual === 'string' &&
          typeof expected === 'string' &&
          !actual.includes(expected)
        );

      case RuleConditionOperator.MATCHES:
        if (typeof actual === 'string' && typeof expected === 'string') {
          try {
            const regex = new RegExp(expected);
            return regex.test(actual);
          } catch {
            return false;
          }
        }
        return false;

      case RuleConditionOperator.IN:
        return Array.isArray(expected) && expected.includes(actual);

      case RuleConditionOperator.NOT_IN:
        return Array.isArray(expected) && !expected.includes(actual);

      default:
        return false;
    }
  }

  /**
   * Calculate baseline for metric
   */
  private calculateBaseline(metric: string, windowMs: number): number {
    // Mock implementation - would query historical data
    return 100;
  }

  /**
   * Add result to history
   */
  private addToHistory(ruleId: string, result: EvaluationResult): void {
    let history = this.evaluationHistory.get(ruleId);
    if (!history) {
      history = [];
      this.evaluationHistory.set(ruleId, history);
    }

    history.push(result);

    // Keep only last 100 results
    if (history.length > 100) {
      history.shift();
    }
  }

  /**
   * Get evaluation history
   */
  getHistory(ruleId: string, limit: number = 100): EvaluationResult[] {
    const history = this.evaluationHistory.get(ruleId) ?? [];
    return history.slice(-limit);
  }

  /**
   * Get rule statistics
   */
  getRuleStats(ruleId: string): {
    totalEvaluations: number;
    matchedCount: number;
    matchRate: number;
    lastMatched?: Date;
  } {
    const history = this.evaluationHistory.get(ruleId) ?? [];
    const matchedCount = history.filter(r => r.matched).length;
    const lastMatched = history
      .filter(r => r.matched)
      .reverse()
      .find(r => r.metadata?.evaluatedAt);

    return {
      totalEvaluations: history.length,
      matchedCount,
      matchRate: history.length > 0 ? matchedCount / history.length : 0,
      lastMatched: lastMatched?.metadata?.evaluatedAt as Date | undefined,
    };
  }

  /**
   * Clear history
   */
  clearHistory(ruleId?: string): void {
    if (ruleId) {
      this.evaluationHistory.delete(ruleId);
    } else {
      this.evaluationHistory.clear();
    }
  }
}

export default RuleEvaluator;
