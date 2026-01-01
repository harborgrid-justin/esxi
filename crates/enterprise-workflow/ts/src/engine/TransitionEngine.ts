/**
 * Transition Engine - Manages workflow step transitions
 */

import { WorkflowStep, Transition, Context, StepExecution, StepId } from '../types';
import { ConditionEvaluator } from './ConditionEvaluator';

export class TransitionEngine {
  private conditionEvaluator: ConditionEvaluator;

  constructor(conditionEvaluator: ConditionEvaluator) {
    this.conditionEvaluator = conditionEvaluator;
  }

  /**
   * Determine the next step based on current step and execution results
   */
  async determineNextStep(
    currentStep: WorkflowStep,
    context: Context,
    stepExecution: StepExecution
  ): Promise<StepId | null> {
    const transitions = currentStep.transitions;

    if (!transitions || transitions.length === 0) {
      return null;
    }

    // Handle success/failure transitions from actions
    if (currentStep.action) {
      if (stepExecution.error && currentStep.action.onFailure) {
        return this.evaluateTransitions(currentStep.action.onFailure, context);
      } else if (!stepExecution.error && currentStep.action.onSuccess) {
        return this.evaluateTransitions(currentStep.action.onSuccess, context);
      }
    }

    // Handle conditional transitions
    if (currentStep.type === 'condition' && stepExecution.output !== undefined) {
      const conditionResult = stepExecution.output as boolean;
      return this.findTransitionByConditionResult(transitions, conditionResult, context);
    }

    // Evaluate all transitions and pick the first matching one
    return this.evaluateTransitions(transitions, context);
  }

  /**
   * Evaluate transitions and return the first matching one
   */
  private async evaluateTransitions(
    transitions: Transition[],
    context: Context
  ): Promise<StepId | null> {
    // Sort transitions by priority (transitions with conditions first)
    const sortedTransitions = [...transitions].sort((a, b) => {
      if (a.condition && !b.condition) return -1;
      if (!a.condition && b.condition) return 1;
      return 0;
    });

    for (const transition of sortedTransitions) {
      if (await this.evaluateTransition(transition, context)) {
        return transition.to;
      }
    }

    // If no conditional transition matched, return the first unconditional one
    const unconditionalTransition = transitions.find(t => !t.condition);
    return unconditionalTransition?.to || null;
  }

  /**
   * Evaluate a single transition
   */
  private async evaluateTransition(transition: Transition, context: Context): Promise<boolean> {
    if (!transition.condition) {
      return true; // Unconditional transition
    }

    try {
      return await this.conditionEvaluator.evaluate(transition.condition, context);
    } catch (error) {
      console.error(`Error evaluating transition condition: ${error}`);
      return false;
    }
  }

  /**
   * Find transition based on condition result (true/false branch)
   */
  private async findTransitionByConditionResult(
    transitions: Transition[],
    conditionResult: boolean,
    context: Context
  ): Promise<StepId | null> {
    // Look for transition with matching label
    const labeledTransition = transitions.find(t => {
      const label = t.label?.toLowerCase();
      return conditionResult
        ? (label === 'true' || label === 'yes' || label === 'success')
        : (label === 'false' || label === 'no' || label === 'failure');
    });

    if (labeledTransition) {
      return labeledTransition.to;
    }

    // Fall back to evaluating all transitions
    return this.evaluateTransitions(transitions, context);
  }

  /**
   * Validate transitions for a workflow step
   */
  validateTransitions(step: WorkflowStep, allStepIds: Set<StepId>): string[] {
    const errors: string[] = [];

    if (!step.transitions || step.transitions.length === 0) {
      // End steps don't need transitions
      return errors;
    }

    step.transitions.forEach((transition, index) => {
      // Validate target step exists
      if (!allStepIds.has(transition.to)) {
        errors.push(
          `Transition ${index} from step ${step.id} references non-existent step ${transition.to}`
        );
      }

      // Validate no self-loops (optional, depending on requirements)
      if (transition.from === transition.to) {
        errors.push(
          `Transition ${index} from step ${step.id} creates a self-loop`
        );
      }

      // Validate condition structure if present
      if (transition.condition) {
        const conditionErrors = this.validateCondition(transition.condition);
        errors.push(...conditionErrors.map(e => `Transition ${index}: ${e}`));
      }
    });

    // Validate conditional steps have appropriate transitions
    if (step.type === 'condition') {
      const hasTrueTransition = step.transitions.some(t =>
        t.label?.toLowerCase() === 'true' || !t.condition
      );
      const hasFalseTransition = step.transitions.some(t =>
        t.label?.toLowerCase() === 'false'
      );

      if (!hasTrueTransition) {
        errors.push(`Condition step ${step.id} missing 'true' transition`);
      }
      if (!hasFalseTransition) {
        errors.push(`Condition step ${step.id} missing 'false' transition`);
      }
    }

    return errors;
  }

  /**
   * Validate condition structure
   */
  private validateCondition(condition: any): string[] {
    const errors: string[] = [];

    if (!condition.type) {
      errors.push('Condition missing type');
      return errors;
    }

    if (condition.type === 'simple') {
      if (!condition.operator) {
        errors.push('Simple condition missing operator');
      }
      if (condition.left === undefined) {
        errors.push('Simple condition missing left operand');
      }
    } else if (condition.type === 'composite') {
      if (!condition.logicalOperator) {
        errors.push('Composite condition missing logical operator');
      }
      if (!condition.conditions || condition.conditions.length === 0) {
        errors.push('Composite condition missing sub-conditions');
      } else {
        condition.conditions.forEach((subCondition: any, index: number) => {
          const subErrors = this.validateCondition(subCondition);
          errors.push(...subErrors.map(e => `Sub-condition ${index}: ${e}`));
        });
      }
    }

    return errors;
  }

  /**
   * Find all possible paths from a step
   */
  findPossiblePaths(
    step: WorkflowStep,
    visitedSteps: Set<StepId> = new Set()
  ): StepId[][] {
    if (visitedSteps.has(step.id)) {
      return []; // Prevent infinite loops
    }

    const newVisited = new Set(visitedSteps);
    newVisited.add(step.id);

    if (!step.transitions || step.transitions.length === 0) {
      return [[step.id]]; // End of path
    }

    const paths: StepId[][] = [];

    step.transitions.forEach(transition => {
      const path = [step.id, transition.to];
      paths.push(path);
    });

    return paths;
  }

  /**
   * Check if a transition creates a cycle
   */
  createsCycle(
    fromStepId: StepId,
    toStepId: StepId,
    allSteps: WorkflowStep[]
  ): boolean {
    const visited = new Set<StepId>();
    const queue: StepId[] = [toStepId];

    while (queue.length > 0) {
      const currentId = queue.shift()!;

      if (currentId === fromStepId) {
        return true; // Cycle detected
      }

      if (visited.has(currentId)) {
        continue;
      }

      visited.add(currentId);

      const currentStep = allSteps.find(s => s.id === currentId);
      if (currentStep?.transitions) {
        currentStep.transitions.forEach(t => {
          if (!visited.has(t.to)) {
            queue.push(t.to);
          }
        });
      }
    }

    return false;
  }

  /**
   * Get all reachable steps from a given step
   */
  getReachableSteps(
    startStepId: StepId,
    allSteps: WorkflowStep[]
  ): Set<StepId> {
    const reachable = new Set<StepId>();
    const queue: StepId[] = [startStepId];

    while (queue.length > 0) {
      const currentId = queue.shift()!;

      if (reachable.has(currentId)) {
        continue;
      }

      reachable.add(currentId);

      const currentStep = allSteps.find(s => s.id === currentId);
      if (currentStep?.transitions) {
        currentStep.transitions.forEach(t => {
          if (!reachable.has(t.to)) {
            queue.push(t.to);
          }
        });
      }
    }

    return reachable;
  }

  /**
   * Find unreachable steps in a workflow
   */
  findUnreachableSteps(
    startStepId: StepId,
    allSteps: WorkflowStep[]
  ): WorkflowStep[] {
    const reachable = this.getReachableSteps(startStepId, allSteps);
    return allSteps.filter(step => !reachable.has(step.id));
  }
}
