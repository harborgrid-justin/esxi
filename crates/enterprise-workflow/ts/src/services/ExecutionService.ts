/**
 * Execution Service - Run and manage workflow executions
 */

import { v4 as uuidv4 } from 'uuid';
import {
  Workflow,
  Execution,
  ExecutionId,
  ExecutionStatus,
  Context,
  ApiResponse,
  PaginatedResponse
} from '../types';
import { WorkflowEngine } from '../engine/WorkflowEngine';

export class ExecutionService {
  private executions: Map<ExecutionId, Execution>;
  private engine: WorkflowEngine;

  constructor() {
    this.executions = new Map();
    this.engine = new WorkflowEngine();

    // Listen to engine events
    this.setupEventListeners();
  }

  /**
   * Execute a workflow
   */
  async execute(
    workflow: Workflow,
    context?: Partial<Context>,
    triggeredBy: string = 'manual'
  ): Promise<ApiResponse<Execution>> {
    try {
      const execution = await this.engine.execute(workflow, context, triggeredBy);
      this.executions.set(execution.id, execution);

      return {
        success: true,
        data: execution
      };
    } catch (error) {
      return {
        success: false,
        error: {
          code: 'EXECUTION_ERROR',
          message: error instanceof Error ? error.message : 'Unknown error'
        }
      };
    }
  }

  /**
   * Get execution by ID
   */
  async getById(id: ExecutionId): Promise<ApiResponse<Execution>> {
    const execution = this.executions.get(id) || this.engine.getExecution(id);

    if (!execution) {
      return {
        success: false,
        error: {
          code: 'NOT_FOUND',
          message: `Execution ${id} not found`
        }
      };
    }

    return {
      success: true,
      data: execution
    };
  }

  /**
   * List executions with pagination
   */
  async list(
    workflowId?: string,
    page: number = 1,
    pageSize: number = 20,
    filters?: {
      status?: ExecutionStatus;
      startDate?: Date;
      endDate?: Date;
    }
  ): Promise<ApiResponse<PaginatedResponse<Execution>>> {
    try {
      let executions = Array.from(this.executions.values());

      // Filter by workflow
      if (workflowId) {
        executions = executions.filter(e => e.workflowId === workflowId);
      }

      // Filter by status
      if (filters?.status) {
        executions = executions.filter(e => e.status === filters.status);
      }

      // Filter by date range
      if (filters?.startDate) {
        executions = executions.filter(
          e => e.startedAt >= filters.startDate!
        );
      }
      if (filters?.endDate) {
        executions = executions.filter(
          e => e.startedAt <= filters.endDate!
        );
      }

      // Sort by date (newest first)
      executions.sort((a, b) => b.startedAt.getTime() - a.startedAt.getTime());

      // Pagination
      const total = executions.length;
      const start = (page - 1) * pageSize;
      const end = start + pageSize;
      const items = executions.slice(start, end);

      return {
        success: true,
        data: {
          items,
          total,
          page,
          pageSize,
          hasMore: end < total
        }
      };
    } catch (error) {
      return {
        success: false,
        error: {
          code: 'LIST_ERROR',
          message: error instanceof Error ? error.message : 'Unknown error'
        }
      };
    }
  }

  /**
   * Cancel execution
   */
  async cancel(id: ExecutionId): Promise<ApiResponse<Execution>> {
    try {
      await this.engine.cancelExecution(id);
      const execution = this.engine.getExecution(id);

      if (!execution) {
        return {
          success: false,
          error: {
            code: 'NOT_FOUND',
            message: `Execution ${id} not found`
          }
        };
      }

      return {
        success: true,
        data: execution
      };
    } catch (error) {
      return {
        success: false,
        error: {
          code: 'CANCEL_ERROR',
          message: error instanceof Error ? error.message : 'Unknown error'
        }
      };
    }
  }

  /**
   * Retry failed execution
   */
  async retry(id: ExecutionId): Promise<ApiResponse<Execution>> {
    const execution = this.executions.get(id);

    if (!execution) {
      return {
        success: false,
        error: {
          code: 'NOT_FOUND',
          message: `Execution ${id} not found`
        }
      };
    }

    // Get original workflow (placeholder - would fetch from WorkflowService)
    const workflow = {
      id: execution.workflowId
    } as Workflow;

    // Execute with same context
    return this.execute(workflow, execution.context, execution.triggeredBy);
  }

  /**
   * Get execution statistics
   */
  async getStatistics(workflowId?: string): Promise<ApiResponse<{
    total: number;
    successful: number;
    failed: number;
    running: number;
    avgDuration: number;
  }>> {
    try {
      let executions = Array.from(this.executions.values());

      if (workflowId) {
        executions = executions.filter(e => e.workflowId === workflowId);
      }

      const total = executions.length;
      const successful = executions.filter(
        e => e.status === ExecutionStatus.SUCCESS
      ).length;
      const failed = executions.filter(
        e => e.status === ExecutionStatus.FAILED
      ).length;
      const running = executions.filter(
        e => e.status === ExecutionStatus.RUNNING
      ).length;

      const completedExecutions = executions.filter(e => e.duration);
      const avgDuration = completedExecutions.length > 0
        ? completedExecutions.reduce((sum, e) => sum + (e.duration || 0), 0) /
          completedExecutions.length
        : 0;

      return {
        success: true,
        data: {
          total,
          successful,
          failed,
          running,
          avgDuration
        }
      };
    } catch (error) {
      return {
        success: false,
        error: {
          code: 'STATISTICS_ERROR',
          message: error instanceof Error ? error.message : 'Unknown error'
        }
      };
    }
  }

  /**
   * Setup event listeners
   */
  private setupEventListeners(): void {
    this.engine.on('event', (event) => {
      // Handle workflow events
      console.log('Workflow event:', event.type);
    });

    this.engine.on('log', (log) => {
      // Handle execution logs
      console.log('Execution log:', log);
    });
  }

  /**
   * Clean up old executions
   */
  async cleanup(maxAge: number = 2592000000): Promise<void> {
    // Default 30 days
    const now = Date.now();

    this.executions.forEach((execution, id) => {
      if (execution.completedAt) {
        const age = now - execution.completedAt.getTime();
        if (age > maxAge) {
          this.executions.delete(id);
        }
      }
    });
  }
}
