/**
 * ETL (Extract, Transform, Load) Pipeline Service
 * @module @harborgrid/enterprise-analytics/services
 */

export interface ETLStep {
  name: string;
  type: 'extract' | 'transform' | 'load';
  execute: (data: unknown) => Promise<unknown>;
  onError?: (error: Error) => void;
}

export interface ETLPipelineConfig {
  steps: ETLStep[];
  parallel?: boolean;
  continueOnError?: boolean;
  retryAttempts?: number;
  timeout?: number;
}

export interface ETLResult {
  success: boolean;
  data?: unknown;
  errors: Array<{ step: string; error: Error }>;
  executionTime: number;
  stepsCompleted: number;
  stepsTotal: number;
}

export class ETLPipeline {
  private config: ETLPipelineConfig;
  private executionHistory: ETLExecution[];

  constructor(config: ETLPipelineConfig) {
    this.config = config;
    this.executionHistory = [];
  }

  // ============================================================================
  // Pipeline Execution
  // ============================================================================

  async execute(initialData: unknown): Promise<ETLResult> {
    const startTime = Date.now();
    const errors: Array<{ step: string; error: Error }> = [];
    let currentData = initialData;
    let stepsCompleted = 0;

    try {
      if (this.config.parallel) {
        currentData = await this.executeParallel(currentData, errors);
      } else {
        currentData = await this.executeSequential(currentData, errors);
      }

      stepsCompleted = this.config.steps.length - errors.length;
    } catch (error) {
      errors.push({
        step: 'pipeline',
        error: error instanceof Error ? error : new Error(String(error)),
      });
    }

    const executionTime = Date.now() - startTime;
    const result: ETLResult = {
      success: errors.length === 0,
      data: currentData,
      errors,
      executionTime,
      stepsCompleted,
      stepsTotal: this.config.steps.length,
    };

    this.recordExecution(result);
    return result;
  }

  private async executeSequential(
    data: unknown,
    errors: Array<{ step: string; error: Error }>
  ): Promise<unknown> {
    let currentData = data;

    for (const step of this.config.steps) {
      try {
        currentData = await this.executeStep(step, currentData);
      } catch (error) {
        const err = error instanceof Error ? error : new Error(String(error));
        errors.push({ step: step.name, error: err });

        if (step.onError) {
          step.onError(err);
        }

        if (!this.config.continueOnError) {
          throw err;
        }
      }
    }

    return currentData;
  }

  private async executeParallel(
    data: unknown,
    errors: Array<{ step: string; error: Error }>
  ): Promise<unknown> {
    const results = await Promise.allSettled(
      this.config.steps.map((step) => this.executeStep(step, data))
    );

    const successfulResults = results
      .filter((r): r is PromiseFulfilledResult<unknown> => r.status === 'fulfilled')
      .map((r) => r.value);

    results.forEach((result, index) => {
      if (result.status === 'rejected') {
        const step = this.config.steps[index]!;
        errors.push({ step: step.name, error: result.reason });

        if (step.onError) {
          step.onError(result.reason);
        }
      }
    });

    return successfulResults.length > 0 ? successfulResults : data;
  }

  private async executeStep(step: ETLStep, data: unknown): Promise<unknown> {
    const timeout = this.config.timeout;

    if (timeout) {
      return Promise.race([
        step.execute(data),
        new Promise((_, reject) =>
          setTimeout(() => reject(new Error(`Step ${step.name} timed out`)), timeout)
        ),
      ]);
    }

    return step.execute(data);
  }

  // ============================================================================
  // Transformation Functions
  // ============================================================================

  static createExtractStep(name: string, extractor: (data: unknown) => Promise<unknown>): ETLStep {
    return {
      name,
      type: 'extract',
      execute: extractor,
    };
  }

  static createTransformStep(
    name: string,
    transformer: (data: unknown) => Promise<unknown>
  ): ETLStep {
    return {
      name,
      type: 'transform',
      execute: transformer,
    };
  }

  static createLoadStep(name: string, loader: (data: unknown) => Promise<unknown>): ETLStep {
    return {
      name,
      type: 'load',
      execute: loader,
    };
  }

  // ============================================================================
  // Common Transformations
  // ============================================================================

  static filterRows<T>(predicate: (row: T) => boolean): (data: T[]) => Promise<T[]> {
    return async (data: T[]) => data.filter(predicate);
  }

  static mapRows<T, R>(mapper: (row: T) => R): (data: T[]) => Promise<R[]> {
    return async (data: T[]) => data.map(mapper);
  }

  static groupBy<T>(key: keyof T): (data: T[]) => Promise<Map<unknown, T[]>> {
    return async (data: T[]) => {
      const groups = new Map<unknown, T[]>();
      for (const row of data) {
        const groupKey = row[key];
        if (!groups.has(groupKey)) {
          groups.set(groupKey, []);
        }
        groups.get(groupKey)!.push(row);
      }
      return groups;
    };
  }

  static deduplicate<T>(keyFn: (row: T) => unknown): (data: T[]) => Promise<T[]> {
    return async (data: T[]) => {
      const seen = new Set();
      return data.filter((row) => {
        const key = keyFn(row);
        if (seen.has(key)) {
          return false;
        }
        seen.add(key);
        return true;
      });
    };
  }

  static sort<T>(compareFn: (a: T, b: T) => number): (data: T[]) => Promise<T[]> {
    return async (data: T[]) => [...data].sort(compareFn);
  }

  // ============================================================================
  // Pipeline Management
  // ============================================================================

  getHistory(): ETLExecution[] {
    return this.executionHistory;
  }

  clearHistory(): void {
    this.executionHistory = [];
  }

  private recordExecution(result: ETLResult): void {
    this.executionHistory.push({
      timestamp: new Date(),
      result,
    });

    // Keep only last 100 executions
    if (this.executionHistory.length > 100) {
      this.executionHistory = this.executionHistory.slice(-100);
    }
  }
}

interface ETLExecution {
  timestamp: Date;
  result: ETLResult;
}

// Factory function
export function createETLPipeline(config: ETLPipelineConfig): ETLPipeline {
  return new ETLPipeline(config);
}
