/**
 * Parallel Executor - Executes workflow branches in parallel
 */

import { WorkflowStep, Execution, StepExecution, StepStatus } from '../types';

export interface ParallelExecutionResult {
  branchIndex: number;
  results: any[];
  success: boolean;
  error?: Error;
}

export class ParallelExecutor {
  /**
   * Execute multiple workflow branches in parallel
   */
  async execute(
    branches: WorkflowStep[][],
    execution: Execution,
    state: any
  ): Promise<ParallelExecutionResult[]> {
    const branchPromises = branches.map((branch, index) =>
      this.executeBranch(branch, index, execution, state)
    );

    return Promise.all(branchPromises);
  }

  /**
   * Execute a single branch
   */
  private async executeBranch(
    steps: WorkflowStep[],
    branchIndex: number,
    execution: Execution,
    state: any
  ): Promise<ParallelExecutionResult> {
    const results: any[] = [];

    try {
      for (const step of steps) {
        // Create a step execution record
        const stepExecution: StepExecution = {
          stepId: step.id,
          status: StepStatus.PENDING,
          startedAt: new Date(),
          attempt: 1,
          logs: []
        };

        // Execute step (simplified - in real implementation would use WorkflowEngine)
        stepExecution.status = StepStatus.RUNNING;

        // This is a simplified version - in production, this would delegate to
        // the actual step execution logic from WorkflowEngine
        const result = await this.executeStep(step, execution);

        stepExecution.output = result;
        stepExecution.status = StepStatus.SUCCESS;
        stepExecution.completedAt = new Date();
        stepExecution.duration = stepExecution.completedAt.getTime() - stepExecution.startedAt.getTime();

        results.push(result);
        execution.stepExecutions.push(stepExecution);
      }

      return {
        branchIndex,
        results,
        success: true
      };

    } catch (error) {
      return {
        branchIndex,
        results,
        success: false,
        error: error as Error
      };
    }
  }

  /**
   * Execute a single step (simplified version)
   */
  private async executeStep(step: WorkflowStep, execution: Execution): Promise<any> {
    // This is a placeholder - in production, this would use the full
    // WorkflowEngine step execution logic
    return { stepId: step.id, completed: true };
  }

  /**
   * Execute branches with a race strategy (first to complete wins)
   */
  async executeRace(
    branches: WorkflowStep[][],
    execution: Execution,
    state: any
  ): Promise<ParallelExecutionResult> {
    const branchPromises = branches.map((branch, index) =>
      this.executeBranch(branch, index, execution, state)
    );

    return Promise.race(branchPromises);
  }

  /**
   * Execute branches with all or nothing strategy
   */
  async executeAllOrNothing(
    branches: WorkflowStep[][],
    execution: Execution,
    state: any
  ): Promise<ParallelExecutionResult[]> {
    const results = await this.execute(branches, execution, state);

    const allSucceeded = results.every(r => r.success);

    if (!allSucceeded) {
      throw new Error('One or more parallel branches failed');
    }

    return results;
  }

  /**
   * Execute branches with a minimum success threshold
   */
  async executeWithThreshold(
    branches: WorkflowStep[][],
    execution: Execution,
    state: any,
    threshold: number
  ): Promise<ParallelExecutionResult[]> {
    const results = await this.execute(branches, execution, state);

    const successCount = results.filter(r => r.success).length;
    const successRate = successCount / results.length;

    if (successRate < threshold) {
      throw new Error(
        `Parallel execution failed to meet threshold: ${successRate} < ${threshold}`
      );
    }

    return results;
  }

  /**
   * Execute branches with concurrency limit
   */
  async executeWithConcurrencyLimit(
    branches: WorkflowStep[][],
    execution: Execution,
    state: any,
    limit: number
  ): Promise<ParallelExecutionResult[]> {
    const results: ParallelExecutionResult[] = [];
    const queue = [...branches];
    const executing: Promise<ParallelExecutionResult>[] = [];

    while (queue.length > 0 || executing.length > 0) {
      // Fill up to the concurrency limit
      while (executing.length < limit && queue.length > 0) {
        const branch = queue.shift()!;
        const index = branches.indexOf(branch);
        const promise = this.executeBranch(branch, index, execution, state);
        executing.push(promise);
      }

      // Wait for at least one to complete
      if (executing.length > 0) {
        const result = await Promise.race(executing);
        results.push(result);

        // Remove completed promise from executing array
        const completedIndex = executing.findIndex(async p => (await p) === result);
        if (completedIndex !== -1) {
          executing.splice(completedIndex, 1);
        }
      }
    }

    return results;
  }

  /**
   * Merge results from parallel branches
   */
  mergeResults(results: ParallelExecutionResult[]): any {
    const merged: any = {
      branches: results.length,
      successful: results.filter(r => r.success).length,
      failed: results.filter(r => !r.success).length,
      results: results.map(r => r.results),
      errors: results.filter(r => r.error).map(r => r.error)
    };

    return merged;
  }

  /**
   * Check if all branches succeeded
   */
  allSucceeded(results: ParallelExecutionResult[]): boolean {
    return results.every(r => r.success);
  }

  /**
   * Check if any branch succeeded
   */
  anySucceeded(results: ParallelExecutionResult[]): boolean {
    return results.some(r => r.success);
  }

  /**
   * Get successful branch results
   */
  getSuccessfulResults(results: ParallelExecutionResult[]): any[][] {
    return results.filter(r => r.success).map(r => r.results);
  }

  /**
   * Get failed branch errors
   */
  getFailedBranches(results: ParallelExecutionResult[]): ParallelExecutionResult[] {
    return results.filter(r => !r.success);
  }
}
