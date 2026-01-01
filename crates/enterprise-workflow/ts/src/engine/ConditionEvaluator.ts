/**
 * Condition Evaluator - Boolean logic evaluation for workflow conditions
 */

import {
  Condition,
  ConditionOperator,
  LogicalOperator,
  Context,
  ConditionResult,
  Variable
} from '../types';

export class ConditionEvaluator {
  /**
   * Evaluate a condition against a context
   */
  async evaluate(condition: Condition, context: Context): Promise<boolean> {
    if (condition.type === 'simple') {
      return this.evaluateSimpleCondition(condition, context);
    } else if (condition.type === 'composite') {
      return this.evaluateCompositeCondition(condition, context);
    }

    throw new Error(`Unknown condition type: ${condition.type}`);
  }

  /**
   * Evaluate a simple condition
   */
  private async evaluateSimpleCondition(
    condition: Condition,
    context: Context
  ): Promise<boolean> {
    if (!condition.operator) {
      throw new Error('Simple condition must have an operator');
    }

    const left = this.resolveValue(condition.left, context);
    const right = this.resolveValue(condition.right, context);

    switch (condition.operator) {
      case ConditionOperator.EQUALS:
        return left === right;

      case ConditionOperator.NOT_EQUALS:
        return left !== right;

      case ConditionOperator.GREATER_THAN:
        return left > right;

      case ConditionOperator.LESS_THAN:
        return left < right;

      case ConditionOperator.GREATER_THAN_OR_EQUAL:
        return left >= right;

      case ConditionOperator.LESS_THAN_OR_EQUAL:
        return left <= right;

      case ConditionOperator.CONTAINS:
        return this.contains(left, right);

      case ConditionOperator.NOT_CONTAINS:
        return !this.contains(left, right);

      case ConditionOperator.STARTS_WITH:
        return String(left).startsWith(String(right));

      case ConditionOperator.ENDS_WITH:
        return String(left).endsWith(String(right));

      case ConditionOperator.MATCHES_REGEX:
        return new RegExp(String(right)).test(String(left));

      case ConditionOperator.IN:
        return Array.isArray(right) && right.includes(left);

      case ConditionOperator.NOT_IN:
        return Array.isArray(right) && !right.includes(left);

      case ConditionOperator.IS_NULL:
        return left === null || left === undefined;

      case ConditionOperator.IS_NOT_NULL:
        return left !== null && left !== undefined;

      default:
        throw new Error(`Unknown operator: ${condition.operator}`);
    }
  }

  /**
   * Evaluate a composite condition
   */
  private async evaluateCompositeCondition(
    condition: Condition,
    context: Context
  ): Promise<boolean> {
    if (!condition.logicalOperator) {
      throw new Error('Composite condition must have a logical operator');
    }

    if (!condition.conditions || condition.conditions.length === 0) {
      throw new Error('Composite condition must have sub-conditions');
    }

    const results = await Promise.all(
      condition.conditions.map(c => this.evaluate(c, context))
    );

    switch (condition.logicalOperator) {
      case LogicalOperator.AND:
        return results.every(r => r === true);

      case LogicalOperator.OR:
        return results.some(r => r === true);

      case LogicalOperator.NOT:
        return !results[0];

      default:
        throw new Error(`Unknown logical operator: ${condition.logicalOperator}`);
    }
  }

  /**
   * Resolve a value (variable or literal)
   */
  private resolveValue(value: any, context: Context): any {
    if (value === null || value === undefined) {
      return value;
    }

    // If it's a Variable object, get its value from context
    if (typeof value === 'object' && 'id' in value && 'name' in value) {
      const variable = value as Variable;
      return context.variables.get(variable.name) ?? variable.value;
    }

    // If it's a string that looks like a variable reference (e.g., "${varName}")
    if (typeof value === 'string' && value.startsWith('${') && value.endsWith('}')) {
      const varName = value.slice(2, -1);
      return context.variables.get(varName);
    }

    // If it's a string that looks like a context path (e.g., "context.metadata.userId")
    if (typeof value === 'string' && value.includes('.')) {
      return this.resolveContextPath(value, context);
    }

    return value;
  }

  /**
   * Resolve a context path (e.g., "context.metadata.userId")
   */
  private resolveContextPath(path: string, context: Context): any {
    const parts = path.split('.');
    let current: any = context;

    for (const part of parts) {
      if (current === null || current === undefined) {
        return undefined;
      }

      if (part === 'variables' && current.variables instanceof Map) {
        // Special handling for variables Map
        const nextPart = parts[parts.indexOf(part) + 1];
        if (nextPart) {
          return current.variables.get(nextPart);
        }
        return current.variables;
      }

      current = current[part];
    }

    return current;
  }

  /**
   * Check if a value contains another value
   */
  private contains(haystack: any, needle: any): boolean {
    if (typeof haystack === 'string') {
      return haystack.includes(String(needle));
    }

    if (Array.isArray(haystack)) {
      return haystack.includes(needle);
    }

    if (typeof haystack === 'object' && haystack !== null) {
      return needle in haystack;
    }

    return false;
  }

  /**
   * Evaluate a condition and return detailed result
   */
  async evaluateWithDetails(
    condition: Condition,
    context: Context
  ): Promise<ConditionResult> {
    const startTime = new Date();

    try {
      const result = await this.evaluate(condition, context);

      return {
        conditionId: condition.id,
        result,
        evaluatedAt: startTime,
        context: {
          variables: Object.fromEntries(context.variables),
          metadata: context.metadata
        }
      };
    } catch (error) {
      return {
        conditionId: condition.id,
        result: false,
        evaluatedAt: startTime,
        context: {
          variables: Object.fromEntries(context.variables),
          metadata: context.metadata
        },
        error: error instanceof Error ? error.message : String(error)
      };
    }
  }

  /**
   * Build a condition from expression string
   */
  buildCondition(expression: string): Condition {
    // Simple expression parser
    // Supports expressions like: "x > 5", "name == 'John'", etc.

    const operators: { [key: string]: ConditionOperator } = {
      '==': ConditionOperator.EQUALS,
      '!=': ConditionOperator.NOT_EQUALS,
      '>': ConditionOperator.GREATER_THAN,
      '<': ConditionOperator.LESS_THAN,
      '>=': ConditionOperator.GREATER_THAN_OR_EQUAL,
      '<=': ConditionOperator.LESS_THAN_OR_EQUAL,
      'contains': ConditionOperator.CONTAINS,
      'startsWith': ConditionOperator.STARTS_WITH,
      'endsWith': ConditionOperator.ENDS_WITH,
      'matches': ConditionOperator.MATCHES_REGEX,
      'in': ConditionOperator.IN
    };

    // Find operator in expression
    let operator: ConditionOperator | undefined;
    let operatorStr: string | undefined;

    for (const [op, condOp] of Object.entries(operators)) {
      if (expression.includes(op)) {
        operator = condOp;
        operatorStr = op;
        break;
      }
    }

    if (!operator || !operatorStr) {
      throw new Error(`Invalid expression: ${expression}`);
    }

    const [left, right] = expression.split(operatorStr).map(s => s.trim());

    return {
      id: `cond_${Date.now()}`,
      type: 'simple',
      operator,
      left: this.parseValue(left),
      right: this.parseValue(right)
    };
  }

  /**
   * Parse a value from string
   */
  private parseValue(value: string): any {
    // Remove quotes if present
    if ((value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'"))) {
      return value.slice(1, -1);
    }

    // Parse number
    if (!isNaN(Number(value))) {
      return Number(value);
    }

    // Parse boolean
    if (value === 'true') return true;
    if (value === 'false') return false;

    // Parse null
    if (value === 'null') return null;

    // Parse array
    if (value.startsWith('[') && value.endsWith(']')) {
      return JSON.parse(value);
    }

    // Otherwise treat as variable reference
    return value;
  }

  /**
   * Create AND composite condition
   */
  and(...conditions: Condition[]): Condition {
    return {
      id: `cond_and_${Date.now()}`,
      type: 'composite',
      logicalOperator: LogicalOperator.AND,
      conditions
    };
  }

  /**
   * Create OR composite condition
   */
  or(...conditions: Condition[]): Condition {
    return {
      id: `cond_or_${Date.now()}`,
      type: 'composite',
      logicalOperator: LogicalOperator.OR,
      conditions
    };
  }

  /**
   * Create NOT composite condition
   */
  not(condition: Condition): Condition {
    return {
      id: `cond_not_${Date.now()}`,
      type: 'composite',
      logicalOperator: LogicalOperator.NOT,
      conditions: [condition]
    };
  }

  /**
   * Validate a condition structure
   */
  validate(condition: Condition): string[] {
    const errors: string[] = [];

    if (!condition.type) {
      errors.push('Condition must have a type');
      return errors;
    }

    if (condition.type === 'simple') {
      if (!condition.operator) {
        errors.push('Simple condition must have an operator');
      }
      if (condition.left === undefined) {
        errors.push('Simple condition must have a left operand');
      }
      // Right operand is optional for some operators (IS_NULL, IS_NOT_NULL)
      if (condition.right === undefined &&
          condition.operator !== ConditionOperator.IS_NULL &&
          condition.operator !== ConditionOperator.IS_NOT_NULL) {
        errors.push('Simple condition must have a right operand');
      }
    } else if (condition.type === 'composite') {
      if (!condition.logicalOperator) {
        errors.push('Composite condition must have a logical operator');
      }
      if (!condition.conditions || condition.conditions.length === 0) {
        errors.push('Composite condition must have sub-conditions');
      } else {
        condition.conditions.forEach((subCondition, index) => {
          const subErrors = this.validate(subCondition);
          errors.push(...subErrors.map(e => `Sub-condition ${index}: ${e}`));
        });
      }
    }

    return errors;
  }
}
