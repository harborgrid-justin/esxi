/**
 * EscalationManager - Manages alert escalation policies
 * Handles multi-level escalation and notification routing
 */

import { EventEmitter } from 'events';
import { Alert, EscalationPolicy, EscalationLevel, NotificationRecipient } from '../types';

export interface EscalationState {
  alertId: string;
  policyId: string;
  currentLevel: number;
  startedAt: Date;
  lastEscalatedAt?: Date;
  repeatCount: number;
  nextEscalationAt?: Date;
}

export class EscalationManager extends EventEmitter {
  private policies: Map<string, EscalationPolicy>;
  private escalations: Map<string, EscalationState>;
  private timers: Map<string, NodeJS.Timeout>;

  constructor() {
    super();
    this.policies = new Map();
    this.escalations = new Map();
    this.timers = new Map();
  }

  /**
   * Register escalation policy
   */
  registerPolicy(policy: EscalationPolicy): void {
    this.policies.set(policy.id, policy);
    this.emit('policy:registered', policy);
  }

  /**
   * Start escalation for alert
   */
  startEscalation(alert: Alert, policyId: string): EscalationState {
    const policy = this.policies.get(policyId);
    if (!policy || !policy.enabled) {
      throw new Error(`Escalation policy ${policyId} not found or disabled`);
    }

    const state: EscalationState = {
      alertId: alert.id,
      policyId,
      currentLevel: 0,
      startedAt: new Date(),
      repeatCount: 0,
    };

    this.escalations.set(alert.id, state);

    // Start escalation chain
    this.escalateToLevel(state, 0);

    return state;
  }

  /**
   * Escalate to specific level
   */
  private escalateToLevel(state: EscalationState, level: number): void {
    const policy = this.policies.get(state.policyId);
    if (!policy) {
      return;
    }

    const escalationLevel = policy.levels.find(l => l.level === level);
    if (!escalationLevel) {
      // Reached end of escalation chain
      if (policy.repeatInterval && policy.maxRepeats) {
        if (state.repeatCount < policy.maxRepeats) {
          // Repeat from beginning
          state.repeatCount++;
          this.scheduleEscalation(state, 0, policy.repeatInterval * 60 * 1000);
        }
      }
      return;
    }

    state.currentLevel = level;
    state.lastEscalatedAt = new Date();

    this.emit('escalation:triggered', {
      alertId: state.alertId,
      level,
      recipients: escalationLevel.recipients,
      actions: escalationLevel.actions,
    });

    // Execute actions
    for (const action of escalationLevel.actions) {
      this.emit('action:execute', {
        alertId: state.alertId,
        action: action.type,
        config: action.config,
      });
    }

    // Schedule next level
    const nextLevel = policy.levels.find(l => l.level === level + 1);
    if (nextLevel) {
      this.scheduleEscalation(state, level + 1, nextLevel.delayMinutes * 60 * 1000);
    }
  }

  /**
   * Schedule escalation to next level
   */
  private scheduleEscalation(state: EscalationState, level: number, delayMs: number): void {
    // Clear existing timer
    const existingTimer = this.timers.get(state.alertId);
    if (existingTimer) {
      clearTimeout(existingTimer);
    }

    state.nextEscalationAt = new Date(Date.now() + delayMs);

    const timer = setTimeout(() => {
      this.escalateToLevel(state, level);
      this.timers.delete(state.alertId);
    }, delayMs);

    this.timers.set(state.alertId, timer);
  }

  /**
   * Stop escalation
   */
  stopEscalation(alertId: string): boolean {
    const state = this.escalations.get(alertId);
    if (!state) {
      return false;
    }

    // Clear timer
    const timer = this.timers.get(alertId);
    if (timer) {
      clearTimeout(timer);
      this.timers.delete(alertId);
    }

    this.escalations.delete(alertId);
    this.emit('escalation:stopped', alertId);

    return true;
  }

  /**
   * Get escalation state
   */
  getState(alertId: string): EscalationState | undefined {
    return this.escalations.get(alertId);
  }

  /**
   * Get all active escalations
   */
  getActiveEscalations(): EscalationState[] {
    return Array.from(this.escalations.values());
  }
}

export default EscalationManager;
