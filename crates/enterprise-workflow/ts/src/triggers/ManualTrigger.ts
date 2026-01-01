/**
 * Manual Trigger - Human-initiated workflow triggering
 */

import { EventEmitter } from 'eventemitter3';
import { Trigger, ManualTriggerConfig, Context, Variable } from '../types';

export interface ManualTriggerRequest {
  userId: string;
  userRoles: string[];
  inputs?: Record<string, any>;
  reason?: string;
  metadata?: Record<string, any>;
}

export class ManualTrigger extends EventEmitter {
  private triggers: Map<string, Trigger>;

  constructor() {
    super();
    this.triggers = new Map();
  }

  /**
   * Register a manual trigger
   */
  register(trigger: Trigger): void {
    this.triggers.set(trigger.id, trigger);
    this.emit('manual_trigger:registered', { triggerId: trigger.id });
  }

  /**
   * Unregister a manual trigger
   */
  unregister(triggerId: string): void {
    this.triggers.delete(triggerId);
    this.emit('manual_trigger:unregistered', { triggerId });
  }

  /**
   * Execute manual trigger
   */
  async execute(
    triggerId: string,
    request: ManualTriggerRequest
  ): Promise<{ authorized: boolean; context?: Partial<Context>; error?: string }> {
    const trigger = this.triggers.get(triggerId);

    if (!trigger) {
      return {
        authorized: false,
        error: 'Trigger not found'
      };
    }

    if (!trigger.enabled) {
      return {
        authorized: false,
        error: 'Trigger is disabled'
      };
    }

    const config = trigger.config as ManualTriggerConfig;

    // Check role-based authorization
    if (!this.isAuthorized(request, config)) {
      this.emit('manual_trigger:unauthorized', {
        triggerId,
        userId: request.userId,
        userRoles: request.userRoles
      });

      return {
        authorized: false,
        error: 'User not authorized to execute this trigger'
      };
    }

    // Validate custom inputs
    if (config.customInputs) {
      const validationErrors = this.validateInputs(request.inputs || {}, config.customInputs);
      if (validationErrors.length > 0) {
        return {
          authorized: false,
          error: `Invalid inputs: ${validationErrors.join(', ')}`
        };
      }
    }

    // Request confirmation if required
    if (config.confirmationRequired) {
      this.emit('manual_trigger:confirmation_required', {
        triggerId,
        userId: request.userId,
        inputs: request.inputs
      });

      // In a real implementation, this would wait for user confirmation
      // For now, we'll assume confirmation is given
    }

    // Create execution context
    const context: Partial<Context> = {
      userId: request.userId,
      variables: new Map([
        ['trigger_user_id', request.userId],
        ['trigger_user_roles', request.userRoles],
        ['trigger_reason', request.reason],
        ['trigger_timestamp', new Date()],
        ...(request.inputs ? Object.entries(request.inputs) : [])
      ]),
      metadata: {
        source: 'manual',
        triggeredBy: request.userId,
        reason: request.reason,
        ...request.metadata
      }
    };

    this.emit('manual_trigger:executed', {
      triggerId,
      userId: request.userId,
      context
    });

    return {
      authorized: true,
      context
    };
  }

  /**
   * Check if user is authorized to execute trigger
   */
  private isAuthorized(request: ManualTriggerRequest, config: ManualTriggerConfig): boolean {
    if (!config.requiredRoles || config.requiredRoles.length === 0) {
      return true; // No role restrictions
    }

    // Check if user has at least one required role
    return config.requiredRoles.some(role => request.userRoles.includes(role));
  }

  /**
   * Validate custom inputs
   */
  private validateInputs(
    inputs: Record<string, any>,
    customInputs: Variable[]
  ): string[] {
    const errors: string[] = [];

    customInputs.forEach(input => {
      const value = inputs[input.name];

      // Check required fields
      if (input.required && (value === undefined || value === null)) {
        errors.push(`${input.name} is required`);
        return;
      }

      // Skip validation if value is not provided and not required
      if (value === undefined || value === null) {
        return;
      }

      // Validate type
      if (!this.validateType(value, input.type)) {
        errors.push(`${input.name} must be of type ${input.type}`);
      }

      // Run custom validation if provided
      if (input.validation) {
        const validationErrors = this.runValidation(value, input);
        errors.push(...validationErrors);
      }
    });

    return errors;
  }

  /**
   * Validate value type
   */
  private validateType(value: any, type: string): boolean {
    switch (type) {
      case 'string':
        return typeof value === 'string';
      case 'number':
        return typeof value === 'number';
      case 'boolean':
        return typeof value === 'boolean';
      case 'object':
        return typeof value === 'object' && !Array.isArray(value);
      case 'array':
        return Array.isArray(value);
      case 'date':
        return value instanceof Date || !isNaN(Date.parse(value));
      default:
        return true;
    }
  }

  /**
   * Run validation rules
   */
  private runValidation(value: any, input: Variable): string[] {
    const errors: string[] = [];
    const validation = input.validation;

    if (!validation) return errors;

    // Pattern validation
    if (validation.pattern && typeof value === 'string') {
      const regex = new RegExp(validation.pattern);
      if (!regex.test(value)) {
        errors.push(`${input.name} does not match required pattern`);
      }
    }

    // Length validation for strings
    if (typeof value === 'string') {
      if (validation.minLength !== undefined && value.length < validation.minLength) {
        errors.push(`${input.name} must be at least ${validation.minLength} characters`);
      }
      if (validation.maxLength !== undefined && value.length > validation.maxLength) {
        errors.push(`${input.name} must be at most ${validation.maxLength} characters`);
      }
    }

    // Range validation for numbers
    if (typeof value === 'number') {
      if (validation.min !== undefined && value < validation.min) {
        errors.push(`${input.name} must be at least ${validation.min}`);
      }
      if (validation.max !== undefined && value > validation.max) {
        errors.push(`${input.name} must be at most ${validation.max}`);
      }
    }

    // Enum validation
    if (validation.enum && !validation.enum.includes(value)) {
      errors.push(`${input.name} must be one of: ${validation.enum.join(', ')}`);
    }

    return errors;
  }

  /**
   * Get trigger by ID
   */
  getById(triggerId: string): Trigger | undefined {
    return this.triggers.get(triggerId);
  }

  /**
   * Get all registered manual triggers
   */
  getRegistered(): Trigger[] {
    return Array.from(this.triggers.values());
  }

  /**
   * Get triggers accessible by user
   */
  getAccessibleTriggers(userRoles: string[]): Trigger[] {
    return Array.from(this.triggers.values()).filter(trigger => {
      const config = trigger.config as ManualTriggerConfig;

      // If no role restrictions, accessible to all
      if (!config.requiredRoles || config.requiredRoles.length === 0) {
        return true;
      }

      // Check if user has required role
      return config.requiredRoles.some(role => userRoles.includes(role));
    });
  }

  /**
   * Validate manual trigger configuration
   */
  validate(config: ManualTriggerConfig): string[] {
    const errors: string[] = [];

    if (config.customInputs) {
      config.customInputs.forEach((input, index) => {
        if (!input.name) {
          errors.push(`Custom input ${index} must have a name`);
        }
        if (!input.type) {
          errors.push(`Custom input ${input.name || index} must have a type`);
        }
      });
    }

    return errors;
  }
}
