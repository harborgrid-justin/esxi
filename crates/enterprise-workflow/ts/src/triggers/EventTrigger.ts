/**
 * Event Trigger - Event-based workflow triggering
 */

import { EventEmitter } from 'eventemitter3';
import { Trigger, EventTriggerConfig, Context } from '../types';
import { ConditionEvaluator } from '../engine/ConditionEvaluator';

export interface WorkflowEventPayload {
  eventName: string;
  source?: string;
  data: any;
  timestamp: Date;
  metadata?: Record<string, any>;
}

export class EventTrigger extends EventEmitter {
  private triggers: Map<string, Trigger>;
  private conditionEvaluator: ConditionEvaluator;
  private debounceTimers: Map<string, NodeJS.Timeout>;

  constructor() {
    super();
    this.triggers = new Map();
    this.conditionEvaluator = new ConditionEvaluator();
    this.debounceTimers = new Map();
  }

  /**
   * Register an event trigger
   */
  register(trigger: Trigger): void {
    const config = trigger.config as EventTriggerConfig;

    if (!config.eventName) {
      throw new Error('Event trigger must have an event name');
    }

    this.triggers.set(trigger.id, trigger);
    this.emit('event_trigger:registered', {
      triggerId: trigger.id,
      eventName: config.eventName
    });
  }

  /**
   * Unregister an event trigger
   */
  unregister(triggerId: string): void {
    // Clear any pending debounce timers
    const timer = this.debounceTimers.get(triggerId);
    if (timer) {
      clearTimeout(timer);
      this.debounceTimers.delete(triggerId);
    }

    this.triggers.delete(triggerId);
    this.emit('event_trigger:unregistered', { triggerId });
  }

  /**
   * Handle incoming event
   */
  async handleEvent(event: WorkflowEventPayload): Promise<string[]> {
    const triggeredIds: string[] = [];

    // Find matching triggers
    const matchingTriggers = Array.from(this.triggers.values()).filter(trigger => {
      if (!trigger.enabled) return false;

      const config = trigger.config as EventTriggerConfig;

      // Check event name match
      if (config.eventName !== event.eventName) return false;

      // Check source match if specified
      if (config.source && config.source !== event.source) return false;

      return true;
    });

    // Process each matching trigger
    for (const trigger of matchingTriggers) {
      const config = trigger.config as EventTriggerConfig;

      // Evaluate filter condition if present
      if (config.filter) {
        const context: Context = this.createContext(event);
        const matches = await this.conditionEvaluator.evaluate(config.filter, context);

        if (!matches) {
          this.emit('event_trigger:filtered', {
            triggerId: trigger.id,
            event: event.eventName
          });
          continue;
        }
      }

      // Handle debouncing if configured
      if (config.debounce && config.debounce > 0) {
        this.handleDebounce(trigger, event);
      } else {
        await this.triggerWorkflow(trigger, event);
        triggeredIds.push(trigger.id);
      }
    }

    return triggeredIds;
  }

  /**
   * Handle debounced event
   */
  private handleDebounce(trigger: Trigger, event: WorkflowEventPayload): void {
    const config = trigger.config as EventTriggerConfig;

    // Clear existing timer if any
    const existingTimer = this.debounceTimers.get(trigger.id);
    if (existingTimer) {
      clearTimeout(existingTimer);
    }

    // Set new timer
    const timer = setTimeout(() => {
      this.triggerWorkflow(trigger, event);
      this.debounceTimers.delete(trigger.id);
    }, config.debounce);

    this.debounceTimers.set(trigger.id, timer);

    this.emit('event_trigger:debounced', {
      triggerId: trigger.id,
      debounceMs: config.debounce
    });
  }

  /**
   * Trigger workflow execution
   */
  private async triggerWorkflow(
    trigger: Trigger,
    event: WorkflowEventPayload
  ): Promise<void> {
    const context: Partial<Context> = {
      variables: new Map([
        ['event_name', event.eventName],
        ['event_source', event.source],
        ['event_data', event.data],
        ['event_timestamp', event.timestamp]
      ]),
      metadata: {
        source: 'event',
        eventName: event.eventName,
        ...event.metadata
      }
    };

    this.emit('event_trigger:triggered', {
      triggerId: trigger.id,
      event: event.eventName,
      context
    });
  }

  /**
   * Create context from event
   */
  private createContext(event: WorkflowEventPayload): Context {
    return {
      workflowId: '',
      executionId: '',
      variables: new Map([
        ['event_name', event.eventName],
        ['event_source', event.source],
        ['event_data', event.data],
        ['event_timestamp', event.timestamp]
      ]),
      metadata: event.metadata || {},
      timestamp: new Date(),
      environment: 'production'
    };
  }

  /**
   * Subscribe to event
   */
  subscribe(eventName: string, callback: (event: WorkflowEventPayload) => void): void {
    this.on(eventName, callback);
  }

  /**
   * Unsubscribe from event
   */
  unsubscribe(eventName: string, callback?: (event: WorkflowEventPayload) => void): void {
    if (callback) {
      this.off(eventName, callback);
    } else {
      this.removeAllListeners(eventName);
    }
  }

  /**
   * Publish event
   */
  async publish(event: WorkflowEventPayload): Promise<string[]> {
    this.emit(event.eventName, event);
    return this.handleEvent(event);
  }

  /**
   * Get all registered event triggers
   */
  getRegistered(): Trigger[] {
    return Array.from(this.triggers.values());
  }

  /**
   * Get triggers for specific event
   */
  getTriggersForEvent(eventName: string): Trigger[] {
    return Array.from(this.triggers.values()).filter(trigger => {
      const config = trigger.config as EventTriggerConfig;
      return config.eventName === eventName;
    });
  }

  /**
   * Get pending debounced triggers
   */
  getPendingDebounced(): string[] {
    return Array.from(this.debounceTimers.keys());
  }

  /**
   * Validate event trigger configuration
   */
  validate(config: EventTriggerConfig): string[] {
    const errors: string[] = [];

    if (!config.eventName) {
      errors.push('Event name is required');
    }

    if (config.debounce !== undefined && config.debounce < 0) {
      errors.push('Debounce must be a positive number');
    }

    return errors;
  }

  /**
   * Clear all debounce timers
   */
  clearDebounceTimers(): void {
    this.debounceTimers.forEach(timer => clearTimeout(timer));
    this.debounceTimers.clear();
  }
}
