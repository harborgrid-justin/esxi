/**
 * Workflow Service - CRUD operations for workflows
 */

import { v4 as uuidv4 } from 'uuid';
import {
  Workflow,
  WorkflowId,
  WorkflowStatus,
  WorkflowStep,
  Variable,
  Trigger,
  ApiResponse,
  PaginatedResponse
} from '../types';

export class WorkflowService {
  private workflows: Map<WorkflowId, Workflow>;

  constructor() {
    this.workflows = new Map();
  }

  /**
   * Create a new workflow
   */
  async create(
    workflow: Omit<Workflow, 'id' | 'createdAt' | 'updatedAt'>
  ): Promise<ApiResponse<Workflow>> {
    try {
      const newWorkflow: Workflow = {
        ...workflow,
        id: uuidv4(),
        createdAt: new Date(),
        updatedAt: new Date()
      };

      this.workflows.set(newWorkflow.id, newWorkflow);

      return {
        success: true,
        data: newWorkflow,
        metadata: {
          timestamp: new Date(),
          requestId: uuidv4()
        }
      };
    } catch (error) {
      return {
        success: false,
        error: {
          code: 'CREATE_ERROR',
          message: error instanceof Error ? error.message : 'Unknown error'
        }
      };
    }
  }

  /**
   * Get workflow by ID
   */
  async getById(id: WorkflowId): Promise<ApiResponse<Workflow>> {
    const workflow = this.workflows.get(id);

    if (!workflow) {
      return {
        success: false,
        error: {
          code: 'NOT_FOUND',
          message: `Workflow ${id} not found`
        }
      };
    }

    return {
      success: true,
      data: workflow
    };
  }

  /**
   * List workflows with pagination
   */
  async list(
    page: number = 1,
    pageSize: number = 20,
    filters?: {
      status?: WorkflowStatus;
      category?: string;
      tags?: string[];
    }
  ): Promise<ApiResponse<PaginatedResponse<Workflow>>> {
    try {
      let workflows = Array.from(this.workflows.values());

      // Apply filters
      if (filters?.status) {
        workflows = workflows.filter(w => w.status === filters.status);
      }
      if (filters?.category) {
        workflows = workflows.filter(w => w.category === filters.category);
      }
      if (filters?.tags && filters.tags.length > 0) {
        workflows = workflows.filter(w =>
          w.tags?.some(tag => filters.tags!.includes(tag))
        );
      }

      // Pagination
      const total = workflows.length;
      const start = (page - 1) * pageSize;
      const end = start + pageSize;
      const items = workflows.slice(start, end);

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
   * Update workflow
   */
  async update(
    id: WorkflowId,
    updates: Partial<Workflow>
  ): Promise<ApiResponse<Workflow>> {
    const workflow = this.workflows.get(id);

    if (!workflow) {
      return {
        success: false,
        error: {
          code: 'NOT_FOUND',
          message: `Workflow ${id} not found`
        }
      };
    }

    const updatedWorkflow: Workflow = {
      ...workflow,
      ...updates,
      id: workflow.id, // Prevent ID changes
      createdAt: workflow.createdAt,
      updatedAt: new Date()
    };

    this.workflows.set(id, updatedWorkflow);

    return {
      success: true,
      data: updatedWorkflow
    };
  }

  /**
   * Delete workflow
   */
  async delete(id: WorkflowId): Promise<ApiResponse<void>> {
    const workflow = this.workflows.get(id);

    if (!workflow) {
      return {
        success: false,
        error: {
          code: 'NOT_FOUND',
          message: `Workflow ${id} not found`
        }
      };
    }

    this.workflows.delete(id);

    return {
      success: true
    };
  }

  /**
   * Publish workflow (change status to active)
   */
  async publish(id: WorkflowId): Promise<ApiResponse<Workflow>> {
    return this.update(id, { status: WorkflowStatus.ACTIVE });
  }

  /**
   * Archive workflow
   */
  async archive(id: WorkflowId): Promise<ApiResponse<Workflow>> {
    return this.update(id, { status: WorkflowStatus.ARCHIVED });
  }

  /**
   * Clone workflow
   */
  async clone(id: WorkflowId, name?: string): Promise<ApiResponse<Workflow>> {
    const workflow = this.workflows.get(id);

    if (!workflow) {
      return {
        success: false,
        error: {
          code: 'NOT_FOUND',
          message: `Workflow ${id} not found`
        }
      };
    }

    const clonedWorkflow: Omit<Workflow, 'id' | 'createdAt' | 'updatedAt'> = {
      ...workflow,
      name: name || `${workflow.name} (Copy)`,
      status: WorkflowStatus.DRAFT
    };

    return this.create(clonedWorkflow);
  }

  /**
   * Validate workflow
   */
  async validate(workflow: Workflow): Promise<ApiResponse<{ valid: boolean; errors: string[] }>> {
    const errors: string[] = [];

    // Validate basic fields
    if (!workflow.name) {
      errors.push('Workflow name is required');
    }

    if (!workflow.startStepId) {
      errors.push('Start step is required');
    }

    if (!workflow.steps || workflow.steps.length === 0) {
      errors.push('Workflow must have at least one step');
    }

    // Validate steps
    const stepIds = new Set(workflow.steps.map(s => s.id));

    if (!stepIds.has(workflow.startStepId)) {
      errors.push('Start step does not exist in workflow');
    }

    workflow.endStepIds.forEach(endId => {
      if (!stepIds.has(endId)) {
        errors.push(`End step ${endId} does not exist in workflow`);
      }
    });

    // Validate transitions
    workflow.steps.forEach(step => {
      step.transitions.forEach(transition => {
        if (!stepIds.has(transition.to)) {
          errors.push(`Step ${step.id} has invalid transition to ${transition.to}`);
        }
      });
    });

    return {
      success: true,
      data: {
        valid: errors.length === 0,
        errors
      }
    };
  }

  /**
   * Search workflows
   */
  async search(query: string): Promise<ApiResponse<Workflow[]>> {
    try {
      const results = Array.from(this.workflows.values()).filter(workflow =>
        workflow.name.toLowerCase().includes(query.toLowerCase()) ||
        workflow.description?.toLowerCase().includes(query.toLowerCase()) ||
        workflow.tags?.some(tag => tag.toLowerCase().includes(query.toLowerCase()))
      );

      return {
        success: true,
        data: results
      };
    } catch (error) {
      return {
        success: false,
        error: {
          code: 'SEARCH_ERROR',
          message: error instanceof Error ? error.message : 'Unknown error'
        }
      };
    }
  }

  /**
   * Export workflow to JSON
   */
  async export(id: WorkflowId): Promise<ApiResponse<string>> {
    const workflow = this.workflows.get(id);

    if (!workflow) {
      return {
        success: false,
        error: {
          code: 'NOT_FOUND',
          message: `Workflow ${id} not found`
        }
      };
    }

    return {
      success: true,
      data: JSON.stringify(workflow, null, 2)
    };
  }

  /**
   * Import workflow from JSON
   */
  async import(json: string): Promise<ApiResponse<Workflow>> {
    try {
      const workflow = JSON.parse(json) as Workflow;

      // Generate new ID for imported workflow
      const imported: Omit<Workflow, 'id' | 'createdAt' | 'updatedAt'> = {
        ...workflow,
        status: WorkflowStatus.DRAFT
      };

      return this.create(imported);
    } catch (error) {
      return {
        success: false,
        error: {
          code: 'IMPORT_ERROR',
          message: error instanceof Error ? error.message : 'Invalid JSON'
        }
      };
    }
  }
}
