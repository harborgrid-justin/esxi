/**
 * Optimization Pipeline
 * Asset processing pipeline with multiple optimization stages
 */

import {
  OptimizationPipeline as IPipeline,
  OptimizationStage,
  PipelineStatus,
  PipelineStats,
  StageStats,
  StageContext,
} from '../types';

export class OptimizationPipeline implements IPipeline {
  id: string;
  name: string;
  stages: OptimizationStage[];
  status: PipelineStatus;
  stats: PipelineStats;

  constructor(name: string, stages: OptimizationStage[]) {
    this.id = this.generateId();
    this.name = name;
    this.stages = stages.sort((a, b) => a.order - b.order);
    this.status = PipelineStatus.PENDING;
    this.stats = this.initializeStats();
  }

  /**
   * Execute pipeline
   */
  async execute(input: Buffer): Promise<Buffer> {
    this.status = PipelineStatus.RUNNING;
    this.stats.totalProcessed = 0;
    this.stats.totalSavings = 0;

    const startTime = performance.now();
    let current = input;
    const originalSize = input.length;

    try {
      for (const stage of this.stages) {
        if (!stage.enabled) continue;

        const context: StageContext = {
          pipeline: this.id,
          stage: stage.name,
          metadata: {},
          stats: new Map(),
        };

        const stageStart = performance.now();
        current = await stage.execute(current, context);
        const stageDuration = performance.now() - stageStart;

        // Update stage stats
        const stageStats: StageStats = {
          processed: 1,
          savings: originalSize - current.length,
          duration: stageDuration,
          errors: 0,
        };

        this.stats.stageStats.set(stage.name, stageStats);
      }

      this.stats.totalProcessed = 1;
      this.stats.totalSavings = originalSize - current.length;
      this.stats.duration = performance.now() - startTime;
      this.stats.throughput = (originalSize / this.stats.duration) * 1000;
      this.stats.averageRatio = originalSize / current.length;

      this.status = PipelineStatus.COMPLETED;
      return current;
    } catch (error) {
      this.status = PipelineStatus.FAILED;
      this.stats.errors++;
      throw error;
    }
  }

  /**
   * Execute pipeline on batch
   */
  async executeBatch(
    inputs: Buffer[],
    concurrency: number = 4
  ): Promise<Buffer[]> {
    const results: Buffer[] = [];

    for (let i = 0; i < inputs.length; i += concurrency) {
      const batch = inputs.slice(i, i + concurrency);
      const batchResults = await Promise.all(
        batch.map(input => this.execute(input))
      );
      results.push(...batchResults);
    }

    return results;
  }

  /**
   * Add stage to pipeline
   */
  addStage(stage: OptimizationStage): void {
    this.stages.push(stage);
    this.stages.sort((a, b) => a.order - b.order);
  }

  /**
   * Remove stage from pipeline
   */
  removeStage(name: string): void {
    this.stages = this.stages.filter(s => s.name !== name);
  }

  /**
   * Enable/disable stage
   */
  toggleStage(name: string, enabled: boolean): void {
    const stage = this.stages.find(s => s.name === name);
    if (stage) {
      stage.enabled = enabled;
    }
  }

  /**
   * Get stage by name
   */
  getStage(name: string): OptimizationStage | undefined {
    return this.stages.find(s => s.name === name);
  }

  /**
   * Initialize statistics
   */
  private initializeStats(): PipelineStats {
    return {
      totalProcessed: 0,
      totalSavings: 0,
      averageRatio: 0,
      duration: 0,
      throughput: 0,
      errors: 0,
      stageStats: new Map(),
    };
  }

  /**
   * Generate unique ID
   */
  private generateId(): string {
    return `pipeline_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Reset pipeline
   */
  reset(): void {
    this.status = PipelineStatus.PENDING;
    this.stats = this.initializeStats();
  }

  /**
   * Cancel pipeline
   */
  cancel(): void {
    this.status = PipelineStatus.CANCELLED;
  }

  /**
   * Get statistics
   */
  getStats(): PipelineStats {
    return { ...this.stats };
  }

  /**
   * Get stage statistics
   */
  getStageStats(name: string): StageStats | undefined {
    return this.stats.stageStats.get(name);
  }

  /**
   * Clone pipeline
   */
  clone(name?: string): OptimizationPipeline {
    return new OptimizationPipeline(
      name || `${this.name} (Copy)`,
      [...this.stages]
    );
  }
}

/**
 * Pipeline Builder
 * Fluent API for creating pipelines
 */
export class PipelineBuilder {
  private stages: OptimizationStage[] = [];

  /**
   * Add compression stage
   */
  compression(
    compressor: (data: Buffer, context: StageContext) => Promise<Buffer>
  ): this {
    this.stages.push({
      name: 'compression',
      type: 'compression',
      config: {},
      enabled: true,
      order: this.stages.length,
      execute: compressor,
    });
    return this;
  }

  /**
   * Add image optimization stage
   */
  imageOptimization(
    optimizer: (data: Buffer, context: StageContext) => Promise<Buffer>
  ): this {
    this.stages.push({
      name: 'image-optimization',
      type: 'image',
      config: {},
      enabled: true,
      order: this.stages.length,
      execute: optimizer,
    });
    return this;
  }

  /**
   * Add data optimization stage
   */
  dataOptimization(
    optimizer: (data: Buffer, context: StageContext) => Promise<Buffer>
  ): this {
    this.stages.push({
      name: 'data-optimization',
      type: 'data',
      config: {},
      enabled: true,
      order: this.stages.length,
      execute: optimizer,
    });
    return this;
  }

  /**
   * Add custom stage
   */
  custom(
    name: string,
    executor: (data: Buffer, context: StageContext) => Promise<Buffer>
  ): this {
    this.stages.push({
      name,
      type: 'compression',
      config: {},
      enabled: true,
      order: this.stages.length,
      execute: executor,
    });
    return this;
  }

  /**
   * Build pipeline
   */
  build(name: string): OptimizationPipeline {
    return new OptimizationPipeline(name, this.stages);
  }
}
