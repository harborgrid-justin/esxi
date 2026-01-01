/**
 * Transform Action - Data transformation operations
 */

import { TransformActionConfig, Context } from '../types';

export class TransformAction {
  /**
   * Execute data transformation
   */
  async execute(config: TransformActionConfig, context: Context): Promise<any> {
    // Get input data
    const input = context.variables.get(config.inputVariable);

    if (input === undefined) {
      throw new Error(`Input variable '${config.inputVariable}' not found`);
    }

    // Apply transformation
    let output: any;

    switch (config.transformType) {
      case 'map':
        output = await this.applyMap(input, config.transformation, context);
        break;
      case 'filter':
        output = await this.applyFilter(input, config.transformation, context);
        break;
      case 'reduce':
        output = await this.applyReduce(input, config.transformation, context);
        break;
      case 'custom':
        output = await this.applyCustom(input, config.transformation, context);
        break;
      default:
        throw new Error(`Unsupported transform type: ${config.transformType}`);
    }

    // Store output
    context.variables.set(config.outputVariable, output);

    return {
      success: true,
      inputVariable: config.inputVariable,
      outputVariable: config.outputVariable,
      transformType: config.transformType,
      inputSize: Array.isArray(input) ? input.length : 1,
      outputSize: Array.isArray(output) ? output.length : 1
    };
  }

  /**
   * Apply map transformation
   */
  private async applyMap(
    input: any,
    transformation: string | ((input: any) => any),
    context: Context
  ): Promise<any> {
    if (!Array.isArray(input)) {
      throw new Error('Map transformation requires array input');
    }

    const transformFn = this.getTransformFunction(transformation);

    return input.map((item, index) => {
      return transformFn(item, index, context);
    });
  }

  /**
   * Apply filter transformation
   */
  private async applyFilter(
    input: any,
    transformation: string | ((input: any) => any),
    context: Context
  ): Promise<any> {
    if (!Array.isArray(input)) {
      throw new Error('Filter transformation requires array input');
    }

    const transformFn = this.getTransformFunction(transformation);

    return input.filter((item, index) => {
      return transformFn(item, index, context);
    });
  }

  /**
   * Apply reduce transformation
   */
  private async applyReduce(
    input: any,
    transformation: string | ((input: any) => any),
    context: Context
  ): Promise<any> {
    if (!Array.isArray(input)) {
      throw new Error('Reduce transformation requires array input');
    }

    const transformFn = this.getTransformFunction(transformation);

    return input.reduce((acc, item, index) => {
      return transformFn({ accumulator: acc, item, index }, index, context);
    }, null);
  }

  /**
   * Apply custom transformation
   */
  private async applyCustom(
    input: any,
    transformation: string | ((input: any) => any),
    context: Context
  ): Promise<any> {
    const transformFn = this.getTransformFunction(transformation);
    return transformFn(input, 0, context);
  }

  /**
   * Get transformation function
   */
  private getTransformFunction(
    transformation: string | ((input: any) => any)
  ): (item: any, index: number, context: Context) => any {
    if (typeof transformation === 'function') {
      return (item, index, context) => transformation(item);
    }

    if (typeof transformation === 'string') {
      // Parse string transformation
      return this.parseTransformation(transformation);
    }

    throw new Error('Invalid transformation type');
  }

  /**
   * Parse string transformation into function
   */
  private parseTransformation(
    transformation: string
  ): (item: any, index: number, context: Context) => any {
    // Common transformation patterns
    const patterns = {
      // Extract property: "item.propertyName"
      property: /^item\.(\w+)$/,
      // Math operation: "item * 2", "item + 10"
      math: /^item\s*([\+\-\*\/])\s*(.+)$/,
      // Comparison: "item > 10"
      comparison: /^item\s*([><=!]+)\s*(.+)$/,
      // Template string: "Hello ${item.name}"
      template: /\${([^}]+)}/g
    };

    // Property extraction
    const propertyMatch = transformation.match(patterns.property);
    if (propertyMatch) {
      const prop = propertyMatch[1];
      return (item) => item?.[prop];
    }

    // Math operation
    const mathMatch = transformation.match(patterns.math);
    if (mathMatch) {
      const operator = mathMatch[1];
      const operand = parseFloat(mathMatch[2]);

      return (item) => {
        const value = typeof item === 'number' ? item : parseFloat(item);
        switch (operator) {
          case '+': return value + operand;
          case '-': return value - operand;
          case '*': return value * operand;
          case '/': return value / operand;
          default: return item;
        }
      };
    }

    // Comparison
    const comparisonMatch = transformation.match(patterns.comparison);
    if (comparisonMatch) {
      const operator = comparisonMatch[1];
      const value = this.parseValue(comparisonMatch[2]);

      return (item) => {
        switch (operator) {
          case '>': return item > value;
          case '<': return item < value;
          case '>=': return item >= value;
          case '<=': return item <= value;
          case '==': return item == value;
          case '===': return item === value;
          case '!=': return item != value;
          case '!==': return item !== value;
          default: return false;
        }
      };
    }

    // Template string
    if (patterns.template.test(transformation)) {
      return (item, index, context) => {
        return transformation.replace(patterns.template, (match, expr) => {
          // Evaluate expression
          if (expr.startsWith('item.')) {
            const prop = expr.slice(5);
            return item?.[prop] ?? '';
          }
          if (expr === 'item') {
            return String(item);
          }
          if (expr === 'index') {
            return String(index);
          }
          // Try to get from context variables
          return String(context.variables.get(expr) ?? match);
        });
      };
    }

    // Default: try to create function from string
    try {
      // eslint-disable-next-line no-new-func
      const fn = new Function('item', 'index', 'context', `return ${transformation}`);
      return fn as any;
    } catch {
      throw new Error(`Invalid transformation: ${transformation}`);
    }
  }

  /**
   * Parse value from string
   */
  private parseValue(value: string): any {
    value = value.trim();

    // Boolean
    if (value === 'true') return true;
    if (value === 'false') return false;

    // Null
    if (value === 'null') return null;

    // Number
    if (!isNaN(Number(value))) return Number(value);

    // String (remove quotes)
    if ((value.startsWith('"') && value.endsWith('"')) ||
        (value.startsWith("'") && value.endsWith("'"))) {
      return value.slice(1, -1);
    }

    return value;
  }

  /**
   * Validate transform action configuration
   */
  validate(config: TransformActionConfig): string[] {
    const errors: string[] = [];

    if (!config.inputVariable) {
      errors.push('Input variable is required');
    }

    if (!config.outputVariable) {
      errors.push('Output variable is required');
    }

    if (!config.transformType) {
      errors.push('Transform type is required');
    } else if (!['map', 'filter', 'reduce', 'custom'].includes(config.transformType)) {
      errors.push('Invalid transform type');
    }

    if (!config.transformation) {
      errors.push('Transformation is required');
    }

    return errors;
  }
}
