/**
 * Workflow Engine - Core execution engine for workflow orchestration
 */

import { EventEmitter } from 'eventemitter3';
import { v4 as uuidv4 } from 'uuid';
import {
  Workflow,
  WorkflowStep,
  Execution,
  ExecutionStatus,
  StepExecution,
  StepStatus,
  Context,
  ExecutionLog,
  ExecutionMetrics,
  ExecutionError,
  WorkflowEvent,
  WorkflowEventType
} from '../types';
import { StateManager } from './StateManager';
import { TransitionEngine } from './TransitionEngine';
import { ConditionEvaluator } from './ConditionEvaluator';
import { ActionExecutor } from './ActionExecutor';
import { ParallelExecutor } from './ParallelExecutor';

export interface WorkflowEngineConfig {
  maxConcurrentExecutions?: number;
  defaultTimeout?: number;
  enableCheckpoints?: boolean;
  logLevel?: 'debug' | 'info' | 'warn' | 'error';
}

export class WorkflowEngine extends EventEmitter {
  private stateManager: StateManager;
  private transitionEngine: TransitionEngine;
  private conditionEvaluator: ConditionEvaluator;
  private actionExecutor: ActionExecutor;
  private parallelExecutor: ParallelExecutor;
  private executions: Map<string, Execution>;
  private config: WorkflowEngineConfig;

  constructor(config: WorkflowEngineConfig = {}) {
    super();
    this.config = {
      maxConcurrentExecutions: 100,
      defaultTimeout: 3600000, // 1 hour
      enableCheckpoints: true,
      logLevel: 'info',
      ...config
    };

    this.stateManager = new StateManager();
    this.conditionEvaluator = new ConditionEvaluator();
    this.actionExecutor = new ActionExecutor();
    this.parallelExecutor = new ParallelExecutor();
    this.transitionEngine = new TransitionEngine(this.conditionEvaluator);
    this.executions = new Map();

    this.setupEventHandlers();
  }

  /**
   * Execute a workflow
   */
  async execute(
    workflow: Workflow,
    context: Partial<Context> = {},
    triggeredBy: string = 'system'
  ): Promise<Execution> {
    const executionId = uuidv4();

    // Create execution context
    const executionContext: Context = {
      workflowId: workflow.id,
      executionId,
      variables: new Map(workflow.variables.map(v => [v.name, v.value])),
      metadata: context.metadata || {},
      timestamp: new Date(),
      userId: context.userId,
      tenantId: context.tenantId,
      environment: context.environment || 'production'
    };

    // Create execution record
    const execution: Execution = {
      id: executionId,
      workflowId: workflow.id,
      workflowVersion: workflow.version,
      status: ExecutionStatus.PENDING,
      triggeredBy,
      context: executionContext,
      currentStepId: workflow.startStepId,
      stepExecutions: [],
      startedAt: new Date(),
      metrics: {
        totalSteps: workflow.steps.length,
        completedSteps: 0,
        failedSteps: 0,
        skippedSteps: 0,
        retryCount: 0
      }
    };

    this.executions.set(executionId, execution);
    this.emitEvent(WorkflowEventType.EXECUTION_STARTED, workflow.id, executionId, execution);

    try {
      // Initialize state
      const state = this.stateManager.createState(executionId, workflow.startStepId, executionContext);

      // Update execution status
      execution.status = ExecutionStatus.RUNNING;

      // Execute workflow
      await this.executeWorkflow(workflow, execution, state);

      // Finalize execution
      execution.completedAt = new Date();
      execution.duration = execution.completedAt.getTime() - execution.startedAt.getTime();

      if (execution.status === ExecutionStatus.RUNNING) {
        execution.status = ExecutionStatus.SUCCESS;
      }

      this.emitEvent(WorkflowEventType.EXECUTION_COMPLETED, workflow.id, executionId, execution);

    } catch (error) {
      execution.status = ExecutionStatus.FAILED;
      execution.error = this.createExecutionError(error as Error);
      execution.completedAt = new Date();
      execution.duration = execution.completedAt.getTime() - execution.startedAt.getTime();

      this.emitEvent(WorkflowEventType.EXECUTION_FAILED, workflow.id, executionId, {
        execution,
        error: execution.error
      });

      this.log(executionId, 'error', `Workflow execution failed: ${execution.error.message}`);
    }

    return execution;
  }

  /**
   * Execute workflow steps
   */
  private async executeWorkflow(
    workflow: Workflow,
    execution: Execution,
    state: any
  ): Promise<void> {
    let currentStepId = workflow.startStepId;

    while (currentStepId && !workflow.endStepIds.includes(currentStepId)) {
      const step = workflow.steps.find(s => s.id === currentStepId);
      if (!step) {
        throw new Error(`Step ${currentStepId} not found in workflow`);
      }

      // Check if execution was cancelled
      if (execution.status === ExecutionStatus.CANCELLED) {
        break;
      }

      // Execute step
      const stepExecution = await this.executeStep(step, execution, state);
      execution.stepExecutions.push(stepExecution);

      // Update metrics
      this.updateMetrics(execution, stepExecution);

      // Handle step failure
      if (stepExecution.status === StepStatus.FAILED) {
        const errorHandling = workflow.settings.errorHandling || 'fail';

        if (errorHandling === 'fail') {
          execution.status = ExecutionStatus.FAILED;
          throw new Error(`Step ${step.name} failed: ${stepExecution.error?.message}`);
        } else if (errorHandling === 'continue') {
          this.log(execution.id, 'warn', `Step ${step.name} failed but continuing execution`);
        }
      }

      // Determine next step
      const nextStepId = await this.transitionEngine.determineNextStep(
        step,
        execution.context,
        stepExecution
      );

      // Update state
      if (nextStepId) {
        state.visitedSteps.add(currentStepId);
        state.currentStep = nextStepId;
        state.history.push({
          from: currentStepId,
          to: nextStepId,
          timestamp: new Date()
        });

        // Create checkpoint if enabled
        if (this.config.enableCheckpoints) {
          this.stateManager.createCheckpoint(state, nextStepId);
        }
      }

      currentStepId = nextStepId;
      execution.currentStepId = nextStepId;
    }
  }

  /**
   * Execute a single workflow step
   */
  private async executeStep(
    step: WorkflowStep,
    execution: Execution,
    state: any
  ): Promise<StepExecution> {
    const stepExecution: StepExecution = {
      stepId: step.id,
      status: StepStatus.PENDING,
      startedAt: new Date(),
      attempt: 1,
      logs: []
    };

    this.log(execution.id, 'info', `Starting step: ${step.name}`, { stepId: step.id });
    this.emitEvent(WorkflowEventType.STEP_STARTED, execution.workflowId, execution.id, { step, stepExecution });

    try {
      stepExecution.status = StepStatus.RUNNING;

      // Execute based on step type
      switch (step.type) {
        case 'action':
          stepExecution.output = await this.executeAction(step, execution.context);
          break;

        case 'condition':
          stepExecution.output = await this.evaluateCondition(step, execution.context);
          break;

        case 'parallel':
          stepExecution.output = await this.executeParallel(step, execution, state);
          break;

        case 'loop':
          stepExecution.output = await this.executeLoop(step, execution, state);
          break;

        case 'wait':
          stepExecution.output = await this.executeWait(step);
          break;

        case 'subworkflow':
          stepExecution.output = await this.executeSubworkflow(step, execution.context);
          break;

        default:
          throw new Error(`Unknown step type: ${step.type}`);
      }

      stepExecution.status = StepStatus.SUCCESS;
      stepExecution.completedAt = new Date();
      stepExecution.duration = stepExecution.completedAt.getTime() - stepExecution.startedAt.getTime();

      this.log(execution.id, 'info', `Step completed: ${step.name}`, {
        stepId: step.id,
        duration: stepExecution.duration
      });

      this.emitEvent(WorkflowEventType.STEP_COMPLETED, execution.workflowId, execution.id, {
        step,
        stepExecution
      });

    } catch (error) {
      stepExecution.status = StepStatus.FAILED;
      stepExecution.error = this.createExecutionError(error as Error, step.id);
      stepExecution.completedAt = new Date();
      stepExecution.duration = stepExecution.completedAt.getTime() - stepExecution.startedAt.getTime();

      this.log(execution.id, 'error', `Step failed: ${step.name}`, {
        stepId: step.id,
        error: stepExecution.error
      });

      this.emitEvent(WorkflowEventType.STEP_FAILED, execution.workflowId, execution.id, {
        step,
        stepExecution,
        error: stepExecution.error
      });

      // Retry logic
      if (step.action?.retryPolicy && stepExecution.attempt < step.action.retryPolicy.maxAttempts) {
        stepExecution.attempt++;
        execution.metrics.retryCount++;
        const delay = this.calculateRetryDelay(step.action.retryPolicy, stepExecution.attempt);

        this.log(execution.id, 'info', `Retrying step ${step.name} in ${delay}ms`, {
          stepId: step.id,
          attempt: stepExecution.attempt
        });

        await this.sleep(delay);
        return this.executeStep(step, execution, state);
      }
    }

    return stepExecution;
  }

  /**
   * Execute action step
   */
  private async executeAction(step: WorkflowStep, context: Context): Promise<any> {
    if (!step.action) {
      throw new Error('Action step must have an action defined');
    }
    return this.actionExecutor.execute(step.action, context);
  }

  /**
   * Evaluate condition step
   */
  private async evaluateCondition(step: WorkflowStep, context: Context): Promise<boolean> {
    if (!step.condition) {
      throw new Error('Condition step must have a condition defined');
    }
    return this.conditionEvaluator.evaluate(step.condition, context);
  }

  /**
   * Execute parallel branches
   */
  private async executeParallel(step: WorkflowStep, execution: Execution, state: any): Promise<any[]> {
    if (!step.parallelBranches) {
      throw new Error('Parallel step must have branches defined');
    }
    return this.parallelExecutor.execute(step.parallelBranches, execution, state);
  }

  /**
   * Execute loop
   */
  private async executeLoop(step: WorkflowStep, execution: Execution, state: any): Promise<any[]> {
    if (!step.loopConfig) {
      throw new Error('Loop step must have loop configuration');
    }

    const results: any[] = [];
    let iteration = 0;
    const maxIterations = step.loopConfig.maxIterations || 1000;

    while (iteration < maxIterations) {
      // Evaluate loop condition
      if (step.loopConfig.condition) {
        const shouldContinue = await this.conditionEvaluator.evaluate(
          step.loopConfig.condition,
          execution.context
        );
        if (!shouldContinue) break;
      }

      // Check break condition
      if (step.loopConfig.breakOn) {
        const shouldBreak = await this.conditionEvaluator.evaluate(
          step.loopConfig.breakOn,
          execution.context
        );
        if (shouldBreak) break;
      }

      // Execute loop body (implement based on requirements)
      results.push(await this.executeAction(step, execution.context));
      iteration++;
    }

    return results;
  }

  /**
   * Execute wait step
   */
  private async executeWait(step: WorkflowStep): Promise<void> {
    if (!step.waitConfig) {
      throw new Error('Wait step must have wait configuration');
    }

    const { type, duration, until } = step.waitConfig;

    switch (type) {
      case 'duration':
        if (duration) await this.sleep(duration);
        break;
      case 'until':
        if (until) {
          const delay = until.getTime() - Date.now();
          if (delay > 0) await this.sleep(delay);
        }
        break;
      case 'event':
        // Event-based waiting would be implemented here
        break;
    }
  }

  /**
   * Execute subworkflow
   */
  private async executeSubworkflow(step: WorkflowStep, context: Context): Promise<any> {
    if (!step.subWorkflowId) {
      throw new Error('Subworkflow step must have a workflow ID');
    }
    // This would load and execute the subworkflow
    // Implementation depends on workflow storage mechanism
    throw new Error('Subworkflow execution not yet implemented');
  }

  /**
   * Calculate retry delay based on retry policy
   */
  private calculateRetryDelay(retryPolicy: any, attempt: number): number {
    const { backoffType, initialDelay, maxDelay = 60000, multiplier = 2 } = retryPolicy;

    let delay = initialDelay;

    switch (backoffType) {
      case 'exponential':
        delay = initialDelay * Math.pow(multiplier, attempt - 1);
        break;
      case 'linear':
        delay = initialDelay * attempt;
        break;
      case 'fixed':
      default:
        delay = initialDelay;
    }

    return Math.min(delay, maxDelay);
  }

  /**
   * Update execution metrics
   */
  private updateMetrics(execution: Execution, stepExecution: StepExecution): void {
    const metrics = execution.metrics;

    switch (stepExecution.status) {
      case StepStatus.SUCCESS:
        metrics.completedSteps++;
        break;
      case StepStatus.FAILED:
        metrics.failedSteps++;
        break;
      case StepStatus.SKIPPED:
        metrics.skippedSteps++;
        break;
    }

    // Calculate average step duration
    const completedSteps = execution.stepExecutions.filter(
      se => se.completedAt && se.duration
    );
    if (completedSteps.length > 0) {
      const totalDuration = completedSteps.reduce((sum, se) => sum + (se.duration || 0), 0);
      metrics.avgStepDuration = totalDuration / completedSteps.length;
    }
  }

  /**
   * Create execution error
   */
  private createExecutionError(error: Error, stepId?: string): ExecutionError {
    return {
      code: 'EXECUTION_ERROR',
      message: error.message,
      stepId,
      timestamp: new Date(),
      stackTrace: error.stack,
      recoverable: false
    };
  }

  /**
   * Log execution message
   */
  private log(
    executionId: string,
    level: 'debug' | 'info' | 'warn' | 'error',
    message: string,
    data?: Record<string, any>
  ): void {
    const log: ExecutionLog = {
      id: uuidv4(),
      executionId,
      level,
      message,
      timestamp: new Date(),
      data
    };

    const execution = this.executions.get(executionId);
    if (execution && execution.currentStepId) {
      const currentStep = execution.stepExecutions.find(
        se => se.stepId === execution.currentStepId
      );
      if (currentStep) {
        currentStep.logs.push(log);
      }
    }

    this.emit('log', log);
  }

  /**
   * Emit workflow event
   */
  private emitEvent(
    type: WorkflowEventType,
    workflowId: string,
    executionId: string,
    payload: any
  ): void {
    const event: WorkflowEvent = {
      id: uuidv4(),
      type,
      workflowId,
      executionId,
      timestamp: new Date(),
      payload
    };

    this.emit('event', event);
  }

  /**
   * Setup event handlers
   */
  private setupEventHandlers(): void {
    this.actionExecutor.on('action:started', (data) => {
      this.emit('action:started', data);
    });

    this.actionExecutor.on('action:completed', (data) => {
      this.emit('action:completed', data);
    });

    this.actionExecutor.on('action:failed', (data) => {
      this.emit('action:failed', data);
    });
  }

  /**
   * Get execution by ID
   */
  getExecution(executionId: string): Execution | undefined {
    return this.executions.get(executionId);
  }

  /**
   * Cancel execution
   */
  async cancelExecution(executionId: string): Promise<void> {
    const execution = this.executions.get(executionId);
    if (execution) {
      execution.status = ExecutionStatus.CANCELLED;
      execution.completedAt = new Date();
      execution.duration = execution.completedAt.getTime() - execution.startedAt.getTime();
    }
  }

  /**
   * Sleep utility
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
