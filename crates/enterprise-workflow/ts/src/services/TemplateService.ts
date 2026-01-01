/**
 * Template Service - Workflow templates management
 */

import { v4 as uuidv4 } from 'uuid';
import { WorkflowTemplate, Workflow, Variable, ApiResponse } from '../types';

export class TemplateService {
  private templates: Map<string, WorkflowTemplate>;

  constructor() {
    this.templates = new Map();
    this.loadDefaultTemplates();
  }

  /**
   * Load default workflow templates
   */
  private loadDefaultTemplates(): void {
    const defaultTemplates: WorkflowTemplate[] = [
      {
        id: 'approval-workflow',
        name: 'Approval Workflow',
        description: 'Simple approval workflow with notifications',
        category: 'Business Process',
        workflow: {
          name: 'Approval Workflow',
          version: '1.0.0',
          status: 'draft' as any,
          triggers: [],
          steps: [],
          variables: [],
          startStepId: 'start',
          endStepIds: ['end'],
          createdBy: 'system',
          settings: {}
        },
        parameters: [],
        tags: ['approval', 'business'],
        version: '1.0.0'
      }
    ];

    defaultTemplates.forEach(template => {
      this.templates.set(template.id, template);
    });
  }

  /**
   * Create a new template
   */
  async create(template: Omit<WorkflowTemplate, 'id'>): Promise<ApiResponse<WorkflowTemplate>> {
    try {
      const newTemplate: WorkflowTemplate = {
        ...template,
        id: uuidv4()
      };

      this.templates.set(newTemplate.id, newTemplate);

      return {
        success: true,
        data: newTemplate
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
   * Get template by ID
   */
  async getById(id: string): Promise<ApiResponse<WorkflowTemplate>> {
    const template = this.templates.get(id);

    if (!template) {
      return {
        success: false,
        error: {
          code: 'NOT_FOUND',
          message: `Template ${id} not found`
        }
      };
    }

    return {
      success: true,
      data: template
    };
  }

  /**
   * List all templates
   */
  async list(category?: string): Promise<ApiResponse<WorkflowTemplate[]>> {
    try {
      let templates = Array.from(this.templates.values());

      if (category) {
        templates = templates.filter(t => t.category === category);
      }

      // Sort by popularity
      templates.sort((a, b) => (b.popularity || 0) - (a.popularity || 0));

      return {
        success: true,
        data: templates
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
   * Instantiate workflow from template
   */
  async instantiate(
    templateId: string,
    parameters: Record<string, any>,
    workflowName?: string
  ): Promise<ApiResponse<Workflow>> {
    const template = this.templates.get(templateId);

    if (!template) {
      return {
        success: false,
        error: {
          code: 'NOT_FOUND',
          message: `Template ${templateId} not found`
        }
      };
    }

    try {
      // Apply parameters to template
      const workflow: Workflow = {
        ...template.workflow,
        id: uuidv4(),
        name: workflowName || template.name,
        createdAt: new Date(),
        updatedAt: new Date(),
        createdBy: 'user'
      };

      // Substitute parameter values
      template.parameters.forEach(param => {
        const value = parameters[param.name] ?? param.defaultValue;
        if (value !== undefined) {
          workflow.variables.push({
            ...param,
            value
          });
        }
      });

      // Increment popularity
      if (template.popularity !== undefined) {
        template.popularity++;
      } else {
        template.popularity = 1;
      }

      return {
        success: true,
        data: workflow
      };
    } catch (error) {
      return {
        success: false,
        error: {
          code: 'INSTANTIATE_ERROR',
          message: error instanceof Error ? error.message : 'Unknown error'
        }
      };
    }
  }

  /**
   * Search templates
   */
  async search(query: string): Promise<ApiResponse<WorkflowTemplate[]>> {
    try {
      const results = Array.from(this.templates.values()).filter(template =>
        template.name.toLowerCase().includes(query.toLowerCase()) ||
        template.description.toLowerCase().includes(query.toLowerCase()) ||
        template.tags.some(tag => tag.toLowerCase().includes(query.toLowerCase()))
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
   * Get categories
   */
  async getCategories(): Promise<ApiResponse<string[]>> {
    const categories = Array.from(
      new Set(Array.from(this.templates.values()).map(t => t.category))
    ).sort();

    return {
      success: true,
      data: categories
    };
  }
}
