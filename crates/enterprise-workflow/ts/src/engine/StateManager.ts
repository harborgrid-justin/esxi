/**
 * State Manager - Manages workflow execution state and checkpoints
 */

import { v4 as uuidv4 } from 'uuid';
import {
  State,
  StepId,
  ExecutionId,
  ExecutionStatus,
  Context,
  StateTransition,
  Checkpoint
} from '../types';

export class StateManager {
  private states: Map<ExecutionId, State>;
  private checkpointStore: Map<string, Checkpoint>;

  constructor() {
    this.states = new Map();
    this.checkpointStore = new Map();
  }

  /**
   * Create a new workflow execution state
   */
  createState(executionId: ExecutionId, startStepId: StepId, context: Context): State {
    const state: State = {
      executionId,
      currentStep: startStepId,
      visitedSteps: new Set(),
      variables: new Map(context.variables),
      status: ExecutionStatus.RUNNING,
      history: [],
      checkpoints: []
    };

    this.states.set(executionId, state);
    return state;
  }

  /**
   * Get state by execution ID
   */
  getState(executionId: ExecutionId): State | undefined {
    return this.states.get(executionId);
  }

  /**
   * Update state
   */
  updateState(executionId: ExecutionId, updates: Partial<State>): State {
    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    Object.assign(state, updates);
    return state;
  }

  /**
   * Transition to next step
   */
  transitionTo(executionId: ExecutionId, nextStepId: StepId, reason?: string): void {
    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    const transition: StateTransition = {
      from: state.currentStep,
      to: nextStepId,
      timestamp: new Date(),
      reason
    };

    state.visitedSteps.add(state.currentStep);
    state.currentStep = nextStepId;
    state.history.push(transition);
  }

  /**
   * Create checkpoint
   */
  createCheckpoint(state: State, stepId: StepId, metadata?: Record<string, any>): Checkpoint {
    const checkpoint: Checkpoint = {
      id: uuidv4(),
      timestamp: new Date(),
      stepId,
      variables: new Map(state.variables),
      metadata
    };

    state.checkpoints.push(checkpoint);
    this.checkpointStore.set(checkpoint.id, checkpoint);

    return checkpoint;
  }

  /**
   * Restore from checkpoint
   */
  restoreCheckpoint(executionId: ExecutionId, checkpointId: string): State {
    const checkpoint = this.checkpointStore.get(checkpointId);
    if (!checkpoint) {
      throw new Error(`Checkpoint ${checkpointId} not found`);
    }

    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    // Restore state from checkpoint
    state.currentStep = checkpoint.stepId;
    state.variables = new Map(checkpoint.variables);

    return state;
  }

  /**
   * Get variable value
   */
  getVariable(executionId: ExecutionId, variableName: string): any {
    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    return state.variables.get(variableName);
  }

  /**
   * Set variable value
   */
  setVariable(executionId: ExecutionId, variableName: string, value: any): void {
    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    state.variables.set(variableName, value);
  }

  /**
   * Get all variables
   */
  getVariables(executionId: ExecutionId): Map<string, any> {
    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    return new Map(state.variables);
  }

  /**
   * Update multiple variables
   */
  updateVariables(executionId: ExecutionId, variables: Record<string, any>): void {
    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    Object.entries(variables).forEach(([key, value]) => {
      state.variables.set(key, value);
    });
  }

  /**
   * Check if step has been visited
   */
  hasVisitedStep(executionId: ExecutionId, stepId: StepId): boolean {
    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    return state.visitedSteps.has(stepId);
  }

  /**
   * Get transition history
   */
  getHistory(executionId: ExecutionId): StateTransition[] {
    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    return [...state.history];
  }

  /**
   * Get all checkpoints
   */
  getCheckpoints(executionId: ExecutionId): Checkpoint[] {
    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    return [...state.checkpoints];
  }

  /**
   * Update execution status
   */
  updateStatus(executionId: ExecutionId, status: ExecutionStatus): void {
    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    state.status = status;
  }

  /**
   * Clear state
   */
  clearState(executionId: ExecutionId): void {
    const state = this.states.get(executionId);
    if (state) {
      // Clean up checkpoints
      state.checkpoints.forEach(cp => {
        this.checkpointStore.delete(cp.id);
      });
    }

    this.states.delete(executionId);
  }

  /**
   * Get all active states
   */
  getActiveStates(): State[] {
    return Array.from(this.states.values()).filter(
      state => state.status === ExecutionStatus.RUNNING || state.status === ExecutionStatus.WAITING
    );
  }

  /**
   * Cleanup old states
   */
  cleanup(maxAge: number = 86400000): void { // Default 24 hours
    const now = Date.now();

    this.states.forEach((state, executionId) => {
      const lastTransition = state.history[state.history.length - 1];
      if (lastTransition) {
        const age = now - lastTransition.timestamp.getTime();
        if (age > maxAge && state.status !== ExecutionStatus.RUNNING) {
          this.clearState(executionId);
        }
      }
    });
  }

  /**
   * Export state for persistence
   */
  exportState(executionId: ExecutionId): any {
    const state = this.states.get(executionId);
    if (!state) {
      throw new Error(`State not found for execution ${executionId}`);
    }

    return {
      executionId: state.executionId,
      currentStep: state.currentStep,
      visitedSteps: Array.from(state.visitedSteps),
      variables: Object.fromEntries(state.variables),
      status: state.status,
      history: state.history,
      checkpoints: state.checkpoints.map(cp => ({
        ...cp,
        variables: Object.fromEntries(cp.variables)
      }))
    };
  }

  /**
   * Import state from persistence
   */
  importState(stateData: any): State {
    const state: State = {
      executionId: stateData.executionId,
      currentStep: stateData.currentStep,
      visitedSteps: new Set(stateData.visitedSteps),
      variables: new Map(Object.entries(stateData.variables)),
      status: stateData.status,
      history: stateData.history,
      checkpoints: stateData.checkpoints.map((cp: any) => ({
        ...cp,
        variables: new Map(Object.entries(cp.variables))
      }))
    };

    this.states.set(state.executionId, state);

    // Restore checkpoints to store
    state.checkpoints.forEach(cp => {
      this.checkpointStore.set(cp.id, cp);
    });

    return state;
  }
}
