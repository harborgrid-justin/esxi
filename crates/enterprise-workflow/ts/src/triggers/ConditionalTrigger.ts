/**
 * Conditional Trigger - Condition-based workflow triggering
 */

import { EventEmitter } from 'eventemitter3';
import { Trigger, ConditionalTriggerConfig, Context } from '../types';
import { ConditionEvaluator } from '../engine/ConditionEvaluator';

export interface ConditionalTriggerState {
  triggerId: string;
  lastEvaluation: Date;
  lastResult: boolean;
  evaluationCount: number;
  triggeredCount: number;
}

export class ConditionalTrigger extends EventEmitter {
  private triggers: Map<string, Trigger>;
  private states: Map<string, ConditionalTriggerState>;
  private intervals: Map<string, NodeJS.Timeout>;
  private conditionEvaluator: ConditionEvaluator;

  constructor() {
    super();
    this.triggers = new Map();
    this.states = new Map();
    this.intervals = new Map();
    this.conditionEvaluator = new ConditionEvaluator();
  }

  /**
   * Register a conditional trigger
   */
  register(trigger: Trigger): void {
    const config = trigger.config as ConditionalTriggerConfig;

    if (!config.condition) {
      throw new Error('Conditional trigger must have a condition');
    }

    this.triggers.set(trigger.id, trigger);

    // Initialize state
    this.states.set(trigger.id, {
      triggerId: trigger.id,
      lastEvaluation: new Date(),
      lastResult: false,
      evaluationCount: 0,
      triggeredCount: 0
    });

    // Start evaluation interval if configured
    if (config.evaluationInterval && trigger.enabled) {
      this.startEvaluation(trigger);
    }

    this.emit('conditional_trigger:registered', {
      triggerId: trigger.id
    });
  }

  /**
   * Unregister a conditional trigger
   */
  unregister(triggerId: string): void {
    this.stopEvaluation(triggerId);
    this.triggers.delete(triggerId);
    this.states.delete(triggerId);

    this.emit('conditional_trigger:unregistered', { triggerId });
  }

  /**
   * Start periodic condition evaluation
   */
  private startEvaluation(trigger: Trigger): void {
    const config = trigger.config as ConditionalTriggerConfig;
    const interval = config.evaluationInterval || 60000; // Default 1 minute

    // Clear existing interval if any
    this.stopEvaluation(trigger.id);

    // Set up periodic evaluation
    const timer = setInterval(async () => {
      await this.evaluateCondition(trigger);
    }, interval);

    this.intervals.set(trigger.id, timer);

    this.emit('conditional_trigger:started', {
      triggerId: trigger.id,
      interval
    });
  }

  /**
   * Stop periodic condition evaluation
   */
  private stopEvaluation(triggerId: string): void {
    const interval = this.intervals.get(triggerId);
    if (interval) {
      clearInterval(interval);
      this.intervals.delete(triggerId);

      this.emit('conditional_trigger:stopped', { triggerId });
    }
  }

  /**
   * Evaluate trigger condition
   */
  async evaluateCondition(trigger: Trigger): Promise<boolean> {
    const config = trigger.config as ConditionalTriggerConfig;
    const state = this.states.get(trigger.id);

    if (!state) {
      throw new Error(`State not found for trigger ${trigger.id}`);
    }

    // Create context for evaluation
    const context = await this.createEvaluationContext(trigger, config);

    try {
      // Evaluate condition
      const result = await this.conditionEvaluator.evaluate(config.condition, context);

      // Update state
      state.lastEvaluation = new Date();
      state.evaluationCount++;
      const previousResult = state.lastResult;
      state.lastResult = result;

      this.emit('conditional_trigger:evaluated', {
        triggerId: trigger.id,
        result,
        previousResult,
        context
      });

      // Trigger workflow if condition changed from false to true
      if (result && !previousResult) {
        state.triggeredCount++;

        const executionContext: Partial<Context> = {
          variables: new Map([
            ['condition_result', result],
            ['evaluation_count', state.evaluationCount],
            ['trigger_count', state.triggeredCount],
            ...context.variables.entries()
          ]),
          metadata: {
            source: 'conditional',
            evaluationCount: state.evaluationCount,
            ...context.metadata
          }
        };

        this.emit('conditional_trigger:triggered', {
          triggerId: trigger.id,
          context: executionContext
        });

        return true;
      }

      return false;

    } catch (error) {
      this.emit('conditional_trigger:error', {
        triggerId: trigger.id,
        error: error instanceof Error ? error.message : String(error)
      });

      return false;
    }
  }

  /**
   * Create evaluation context
   */
  private async createEvaluationContext(
    trigger: Trigger,
    config: ConditionalTriggerConfig
  ): Promise<Context> {
    const variables = new Map<string, any>();

    // Add watched variables if configured
    if (config.watchVariables) {
      for (const varName of config.watchVariables) {
        // In a real implementation, this would fetch the actual variable value
        // from a data source or state management system
        variables.set(varName, await this.getWatchedVariable(varName));
      }
    }

    return {
      workflowId: '',
      executionId: '',
      variables,
      metadata: {
        triggerId: trigger.id,
        evaluationTime: new Date()
      },
      timestamp: new Date(),
      environment: 'production'
    };
  }

  /**
   * Get watched variable value
   * This is a placeholder - in production, would integrate with data sources
   */
  private async getWatchedVariable(varName: string): Promise<any> {
    // Placeholder implementation
    // In production, this would fetch from database, API, or other data source
    return undefined;
  }

  /**
   * Manually evaluate a trigger
   */
  async manualEvaluate(triggerId: string): Promise<boolean> {
    const trigger = this.triggers.get(triggerId);
    if (!trigger) {
      throw new Error(`Trigger ${triggerId} not found`);
    }

    return this.evaluateCondition(trigger);
  }

  /**
   * Start trigger
   */
  start(triggerId: string): void {
    const trigger = this.triggers.get(triggerId);
    if (trigger) {
      this.startEvaluation(trigger);
    }
  }

  /**
   * Stop trigger
   */
  stop(triggerId: string): void {
    this.stopEvaluation(triggerId);
  }

  /**
   * Get trigger state
   */
  getState(triggerId: string): ConditionalTriggerState | undefined {
    return this.states.get(triggerId);
  }

  /**
   * Get all registered conditional triggers
   */
  getRegistered(): Trigger[] {
    return Array.from(this.triggers.values());
  }

  /**
   * Get all trigger states
   */
  getAllStates(): ConditionalTriggerState[] {
    return Array.from(this.states.values());
  }

  /**
   * Get active triggers
   */
  getActive(): Trigger[] {
    return Array.from(this.triggers.values()).filter(trigger =>
      this.intervals.has(trigger.id)
    );
  }

  /**
   * Reset trigger state
   */
  resetState(triggerId: string): void {
    const state = this.states.get(triggerId);
    if (state) {
      state.lastResult = false;
      state.evaluationCount = 0;
      state.triggeredCount = 0;
      state.lastEvaluation = new Date();

      this.emit('conditional_trigger:reset', { triggerId });
    }
  }

  /**
   * Validate conditional trigger configuration
   */
  validate(config: ConditionalTriggerConfig): string[] {
    const errors: string[] = [];

    if (!config.condition) {
      errors.push('Condition is required');
    } else {
      const conditionErrors = this.conditionEvaluator.validate(config.condition);
      errors.push(...conditionErrors);
    }

    if (config.evaluationInterval !== undefined && config.evaluationInterval < 1000) {
      errors.push('Evaluation interval must be at least 1000ms');
    }

    return errors;
  }

  /**
   * Stop all triggers
   */
  stopAll(): void {
    this.triggers.forEach((_, triggerId) => {
      this.stopEvaluation(triggerId);
    });

    this.emit('conditional_trigger:all_stopped');
  }

  /**
   * Clear all triggers
   */
  clear(): void {
    this.stopAll();
    this.triggers.clear();
    this.states.clear();

    this.emit('conditional_trigger:cleared');
  }
}
