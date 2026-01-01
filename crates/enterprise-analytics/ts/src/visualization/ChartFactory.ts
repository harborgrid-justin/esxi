/**
 * Chart Type Factory
 * @module @harborgrid/enterprise-analytics/visualization
 */

import type { VisualizationType, VisualizationConfig, QueryResult } from '../types';

export interface ChartSpec {
  type: VisualizationType;
  data: unknown[];
  config: VisualizationConfig;
  width?: number;
  height?: number;
}

export interface ChartInstance {
  render(container: HTMLElement): void;
  update(data: unknown[]): void;
  destroy(): void;
  export(format: 'png' | 'svg'): Promise<Blob>;
}

export type ChartConstructor = new (spec: ChartSpec) => ChartInstance;

export class ChartFactory {
  private registry: Map<VisualizationType, ChartConstructor>;
  private instances: Map<string, ChartInstance>;

  constructor() {
    this.registry = new Map();
    this.instances = new Map();
    this.registerDefaultCharts();
  }

  // ============================================================================
  // Registration
  // ============================================================================

  register(type: VisualizationType, constructor: ChartConstructor): void {
    this.registry.set(type, constructor);
  }

  private registerDefaultCharts(): void {
    // Chart constructors would be registered here
    // For now, we'll use a simplified implementation
  }

  // ============================================================================
  // Chart Creation
  // ============================================================================

  create(spec: ChartSpec, id?: string): ChartInstance {
    const Constructor = this.registry.get(spec.type);

    if (!Constructor) {
      throw new Error(`Unknown chart type: ${spec.type}`);
    }

    const instance = new Constructor(spec);

    if (id) {
      this.instances.set(id, instance);
    }

    return instance;
  }

  createFromQueryResult<T = Record<string, unknown>>(
    type: VisualizationType,
    result: QueryResult<T>,
    config: VisualizationConfig,
    id?: string
  ): ChartInstance {
    const spec: ChartSpec = {
      type,
      data: result.data,
      config,
    };

    return this.create(spec, id);
  }

  // ============================================================================
  // Chart Management
  // ============================================================================

  get(id: string): ChartInstance | undefined {
    return this.instances.get(id);
  }

  destroy(id: string): void {
    const instance = this.instances.get(id);
    if (instance) {
      instance.destroy();
      this.instances.delete(id);
    }
  }

  destroyAll(): void {
    for (const [id, instance] of this.instances.entries()) {
      instance.destroy();
      this.instances.delete(id);
    }
  }

  // ============================================================================
  // Chart Recommendations
  // ============================================================================

  recommendChartType(data: unknown[], context?: ChartContext): VisualizationType[] {
    const recommendations: VisualizationType[] = [];

    if (!data || data.length === 0) {
      return recommendations;
    }

    const firstRow = data[0] as Record<string, unknown>;
    const fields = Object.keys(firstRow);
    const numericFields = fields.filter((f) => typeof firstRow[f] === 'number');
    const dateFields = fields.filter(
      (f) => firstRow[f] instanceof Date || this.isDateString(firstRow[f])
    );
    const categoricalFields = fields.filter(
      (f) => !numericFields.includes(f) && !dateFields.includes(f)
    );

    // Time series data
    if (dateFields.length > 0 && numericFields.length > 0) {
      recommendations.push(VisualizationType.LINE_CHART);
      recommendations.push(VisualizationType.AREA_CHART);
    }

    // Single categorical dimension with numeric values
    if (categoricalFields.length === 1 && numericFields.length >= 1) {
      recommendations.push(VisualizationType.BAR_CHART);
      recommendations.push(VisualizationType.PIE_CHART);
    }

    // Multiple categorical dimensions
    if (categoricalFields.length >= 2 && numericFields.length >= 1) {
      recommendations.push(VisualizationType.HEAT_MAP);
      recommendations.push(VisualizationType.TREE_MAP);
    }

    // Two numeric dimensions (correlation analysis)
    if (numericFields.length >= 2) {
      recommendations.push(VisualizationType.SCATTER_PLOT);
    }

    // Geographic data
    if (context?.hasGeoData || this.hasGeoFields(fields)) {
      recommendations.push(VisualizationType.GEO_MAP);
    }

    // Flow/hierarchy data
    if (context?.isHierarchical) {
      recommendations.push(VisualizationType.SANKEY_DIAGRAM);
      recommendations.push(VisualizationType.TREE_MAP);
    }

    // Single metric
    if (numericFields.length === 1 && categoricalFields.length === 0) {
      recommendations.push(VisualizationType.METRIC_CARD);
      recommendations.push(VisualizationType.GAUGE);
    }

    // Default to table for complex data
    if (recommendations.length === 0) {
      recommendations.push(VisualizationType.TABLE);
    }

    return recommendations;
  }

  // ============================================================================
  // Validation
  // ============================================================================

  validateChartSpec(spec: ChartSpec): ValidationResult {
    const errors: string[] = [];
    const warnings: string[] = [];

    // Check if chart type is registered
    if (!this.registry.has(spec.type)) {
      errors.push(`Chart type ${spec.type} is not registered`);
    }

    // Check data
    if (!spec.data || spec.data.length === 0) {
      warnings.push('No data provided for chart');
    }

    // Type-specific validation
    switch (spec.type) {
      case VisualizationType.LINE_CHART:
      case VisualizationType.AREA_CHART:
        if (!spec.config.xAxis || !spec.config.yAxis) {
          errors.push('Line/Area charts require xAxis and yAxis configuration');
        }
        break;

      case VisualizationType.PIE_CHART:
      case VisualizationType.DONUT_CHART:
        if (spec.data.length > 20) {
          warnings.push('Pie charts work best with fewer than 20 categories');
        }
        break;

      case VisualizationType.HEAT_MAP:
        if (!spec.config.xAxis || !spec.config.yAxis) {
          errors.push('Heat maps require xAxis and yAxis configuration');
        }
        break;

      case VisualizationType.GEO_MAP:
        if (!spec.config.geoField) {
          errors.push('Geographic maps require geoField configuration');
        }
        break;
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
    };
  }

  // ============================================================================
  // Helper Methods
  // ============================================================================

  private isDateString(value: unknown): boolean {
    if (typeof value !== 'string') return false;
    const date = new Date(value);
    return !isNaN(date.getTime());
  }

  private hasGeoFields(fields: string[]): boolean {
    const geoKeywords = ['lat', 'lon', 'lng', 'latitude', 'longitude', 'country', 'city', 'state'];
    return fields.some((field) =>
      geoKeywords.some((keyword) => field.toLowerCase().includes(keyword))
    );
  }

  // ============================================================================
  // Chart Templates
  // ============================================================================

  getTemplate(type: VisualizationType): Partial<VisualizationConfig> {
    const templates: Record<VisualizationType, Partial<VisualizationConfig>> = {
      [VisualizationType.LINE_CHART]: {
        showGrid: true,
        smooth: true,
        legend: { show: true, position: 'top' },
        tooltip: { show: true },
      },
      [VisualizationType.BAR_CHART]: {
        showGrid: true,
        showValues: false,
        legend: { show: true, position: 'top' },
        tooltip: { show: true },
      },
      [VisualizationType.PIE_CHART]: {
        legend: { show: true, position: 'right' },
        tooltip: { show: true },
        showValues: true,
      },
      [VisualizationType.SCATTER_PLOT]: {
        showGrid: true,
        legend: { show: true, position: 'top' },
        tooltip: { show: true },
      },
      [VisualizationType.HEAT_MAP]: {
        showGrid: false,
        legend: { show: true, position: 'right' },
        tooltip: { show: true },
      },
      [VisualizationType.TABLE]: {
        pagination: { pageSize: 25, pageSizeOptions: [10, 25, 50, 100] },
      },
      [VisualizationType.METRIC_CARD]: {
        showValues: true,
      },
      // Add more templates as needed
    } as Record<VisualizationType, Partial<VisualizationConfig>>;

    return templates[type] || {};
  }

  // ============================================================================
  // Batch Operations
  // ============================================================================

  createMany(specs: Array<ChartSpec & { id: string }>): Map<string, ChartInstance> {
    const instances = new Map<string, ChartInstance>();

    for (const spec of specs) {
      try {
        const instance = this.create(spec, spec.id);
        instances.set(spec.id, instance);
      } catch (error) {
        console.error(`Failed to create chart ${spec.id}:`, error);
      }
    }

    return instances;
  }

  updateMany(updates: Array<{ id: string; data: unknown[] }>): void {
    for (const update of updates) {
      const instance = this.instances.get(update.id);
      if (instance) {
        try {
          instance.update(update.data);
        } catch (error) {
          console.error(`Failed to update chart ${update.id}:`, error);
        }
      }
    }
  }
}

// ============================================================================
// Types
// ============================================================================

export interface ChartContext {
  hasGeoData?: boolean;
  isHierarchical?: boolean;
  isTimeSeries?: boolean;
  purpose?: 'comparison' | 'distribution' | 'composition' | 'relationship' | 'trend';
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

// Factory function
export function createChartFactory(): ChartFactory {
  return new ChartFactory();
}
