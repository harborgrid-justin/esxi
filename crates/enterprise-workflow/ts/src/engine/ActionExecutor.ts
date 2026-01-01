/**
 * Action Executor - Executes workflow actions with proper error handling and retries
 */

import { EventEmitter } from 'eventemitter3';
import { Action, ActionType, Context } from '../types';
import { HTTPAction } from '../actions/HTTPAction';
import { EmailAction } from '../actions/EmailAction';
import { NotificationAction } from '../actions/NotificationAction';
import { DatabaseAction } from '../actions/DatabaseAction';
import { TransformAction } from '../actions/TransformAction';
import { ApprovalAction } from '../actions/ApprovalAction';

export class ActionExecutor extends EventEmitter {
  private actions: Map<ActionType, any>;

  constructor() {
    super();
    this.actions = new Map();
    this.registerDefaultActions();
  }

  /**
   * Register default action handlers
   */
  private registerDefaultActions(): void {
    this.registerAction(ActionType.HTTP, new HTTPAction());
    this.registerAction(ActionType.EMAIL, new EmailAction());
    this.registerAction(ActionType.NOTIFICATION, new NotificationAction());
    this.registerAction(ActionType.DATABASE, new DatabaseAction());
    this.registerAction(ActionType.TRANSFORM, new TransformAction());
    this.registerAction(ActionType.APPROVAL, new ApprovalAction());
  }

  /**
   * Register a custom action handler
   */
  registerAction(type: ActionType, handler: any): void {
    this.actions.set(type, handler);
  }

  /**
   * Execute an action
   */
  async execute(action: Action, context: Context): Promise<any> {
    this.emit('action:started', { action, context });

    const handler = this.actions.get(action.type);
    if (!handler) {
      throw new Error(`No handler registered for action type: ${action.type}`);
    }

    try {
      // Apply timeout if specified
      const timeout = action.timeout || 30000; // Default 30 seconds
      const result = await this.executeWithTimeout(
        () => handler.execute(action.config, context),
        timeout
      );

      this.emit('action:completed', { action, context, result });
      return result;

    } catch (error) {
      this.emit('action:failed', { action, context, error });
      throw error;
    }
  }

  /**
   * Execute action with timeout
   */
  private async executeWithTimeout<T>(
    fn: () => Promise<T>,
    timeout: number
  ): Promise<T> {
    return Promise.race([
      fn(),
      new Promise<T>((_, reject) =>
        setTimeout(() => reject(new Error('Action timeout')), timeout)
      )
    ]);
  }

  /**
   * Validate action configuration
   */
  validate(action: Action): string[] {
    const errors: string[] = [];

    if (!action.type) {
      errors.push('Action must have a type');
    }

    if (!action.config) {
      errors.push('Action must have a configuration');
    }

    const handler = this.actions.get(action.type);
    if (handler && handler.validate) {
      const handlerErrors = handler.validate(action.config);
      errors.push(...handlerErrors);
    }

    return errors;
  }

  /**
   * Get registered action types
   */
  getRegisteredTypes(): ActionType[] {
    return Array.from(this.actions.keys());
  }
}
