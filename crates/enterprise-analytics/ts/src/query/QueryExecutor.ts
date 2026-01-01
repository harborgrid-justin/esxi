/**
 * Async Query Execution Engine
 * @module @harborgrid/enterprise-analytics/query
 */

import type { Query, QueryResult, QueryMetadata, DataSource, ColumnMetadata } from '../types';
import { QueryOptimizer } from './QueryOptimizer';
import { CacheManager } from './CacheManager';

export interface ExecutionContext {
  userId?: string;
  tenantId?: string;
  sessionId?: string;
  timeout?: number;
  priority?: 'low' | 'normal' | 'high';
  tracing?: boolean;
}

export interface ExecutionPlan {
  steps: ExecutionStep[];
  estimatedDuration: number;
  cacheHit: boolean;
  optimized: boolean;
}

export interface ExecutionStep {
  name: string;
  type: 'fetch' | 'filter' | 'aggregate' | 'sort' | 'transform';
  cost: number;
  parallel?: boolean;
}

export class QueryExecutor {
  private optimizer: QueryOptimizer;
  private cache: CacheManager;
  private activeQueries: Map<string, AbortController>;
  private executionHistory: ExecutionRecord[];

  constructor() {
    this.optimizer = new QueryOptimizer();
    this.cache = new CacheManager();
    this.activeQueries = new Map();
    this.executionHistory = [];
  }

  // ============================================================================
  // Main Execution Methods
  // ============================================================================

  async execute<T = Record<string, unknown>>(
    query: Query,
    dataSource: DataSource,
    context?: ExecutionContext
  ): Promise<QueryResult<T>> {
    const startTime = Date.now();
    const queryId = query.id || this.generateQueryId();

    try {
      // Check cache first
      if (query.cache) {
        const cached = await this.cache.get<T>(queryId);
        if (cached) {
          return {
            ...cached,
            cached: true,
          };
        }
      }

      // Create abort controller for this query
      const abortController = new AbortController();
      this.activeQueries.set(queryId, abortController);

      // Optimize query
      const optimizedQuery = this.optimizer.optimize(query);

      // Create execution plan
      const plan = this.createExecutionPlan(optimizedQuery);

      // Execute query
      const result = await this.executeQuery<T>(
        optimizedQuery,
        dataSource,
        plan,
        abortController.signal,
        context
      );

      // Calculate execution time
      const executionTime = Date.now() - startTime;

      // Add execution metadata
      const finalResult: QueryResult<T> = {
        ...result,
        executionTime,
        cached: false,
      };

      // Cache result if enabled
      if (query.cache && query.cacheTTL) {
        await this.cache.set(queryId, finalResult, query.cacheTTL);
      }

      // Record execution
      this.recordExecution(queryId, query, executionTime, result.metadata.rowCount);

      return finalResult;
    } catch (error) {
      this.recordExecution(queryId, query, Date.now() - startTime, 0, error);
      throw new QueryExecutionError(
        `Query execution failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        queryId,
        error
      );
    } finally {
      this.activeQueries.delete(queryId);
    }
  }

  async executeParallel<T = Record<string, unknown>>(
    queries: Query[],
    dataSource: DataSource,
    context?: ExecutionContext
  ): Promise<QueryResult<T>[]> {
    return Promise.all(queries.map((q) => this.execute<T>(q, dataSource, context)));
  }

  async executePaginated<T = Record<string, unknown>>(
    query: Query,
    dataSource: DataSource,
    page: number,
    pageSize: number,
    context?: ExecutionContext
  ): Promise<PaginatedResult<T>> {
    // Execute count query
    const countQuery = {
      ...query,
      metrics: [{ id: 'count', field: '*', aggregation: 'count' as any }],
      dimensions: [],
      sort: undefined,
      limit: undefined,
      offset: undefined,
    };

    const [dataResult, countResult] = await Promise.all([
      this.execute<T>(
        {
          ...query,
          limit: pageSize,
          offset: (page - 1) * pageSize,
        },
        dataSource,
        context
      ),
      this.execute(countQuery, dataSource, context),
    ]);

    const total = countResult.data[0]?.count as number || 0;

    return {
      data: dataResult.data,
      metadata: dataResult.metadata,
      pagination: {
        page,
        pageSize,
        total,
        totalPages: Math.ceil(total / pageSize),
        hasNext: page * pageSize < total,
        hasPrev: page > 1,
      },
      executionTime: dataResult.executionTime,
    };
  }

  // ============================================================================
  // Execution Planning
  // ============================================================================

  private createExecutionPlan(query: Query): ExecutionPlan {
    const steps: ExecutionStep[] = [];

    // Data fetching step
    steps.push({
      name: 'fetch',
      type: 'fetch',
      cost: 100,
    });

    // Filtering steps
    if (query.filters.length > 0) {
      steps.push({
        name: 'filter',
        type: 'filter',
        cost: query.filters.length * 10,
      });
    }

    // Aggregation steps
    if (query.metrics.length > 0) {
      steps.push({
        name: 'aggregate',
        type: 'aggregate',
        cost: query.metrics.length * 50,
      });
    }

    // Sorting steps
    if (query.sort && query.sort.length > 0) {
      steps.push({
        name: 'sort',
        type: 'sort',
        cost: 30,
      });
    }

    const estimatedDuration = steps.reduce((sum, step) => sum + step.cost, 0);

    return {
      steps,
      estimatedDuration,
      cacheHit: false,
      optimized: true,
    };
  }

  // ============================================================================
  // Query Execution Implementation
  // ============================================================================

  private async executeQuery<T>(
    query: Query,
    dataSource: DataSource,
    plan: ExecutionPlan,
    signal: AbortSignal,
    context?: ExecutionContext
  ): Promise<QueryResult<T>> {
    // This is a simplified implementation
    // In a real system, this would delegate to appropriate data source handlers

    let data: T[] = [];
    const columns: ColumnMetadata[] = [];

    // Simulate data fetching based on data source type
    switch (dataSource.type) {
      case 'sql':
        data = await this.executeSQLQuery<T>(query, dataSource, signal);
        break;
      case 'rest_api':
        data = await this.executeRESTQuery<T>(query, dataSource, signal);
        break;
      case 'json':
        data = await this.executeJSONQuery<T>(query, dataSource, signal);
        break;
      default:
        throw new Error(`Unsupported data source type: ${dataSource.type}`);
    }

    // Extract column metadata from first row
    if (data.length > 0) {
      const firstRow = data[0] as Record<string, unknown>;
      Object.keys(firstRow).forEach((key) => {
        columns.push({
          name: key,
          type: this.inferType(firstRow[key]),
          nullable: true,
        });
      });
    }

    const metadata: QueryMetadata = {
      rowCount: data.length,
      columnCount: columns.length,
      columns,
      queryId: query.id,
      executedAt: new Date(),
    };

    return { data, metadata };
  }

  private async executeSQLQuery<T>(
    query: Query,
    dataSource: DataSource,
    signal: AbortSignal
  ): Promise<T[]> {
    // In a real implementation, this would use a SQL driver
    // For now, return mock data
    return this.generateMockData<T>(query);
  }

  private async executeRESTQuery<T>(
    query: Query,
    dataSource: DataSource,
    signal: AbortSignal
  ): Promise<T[]> {
    if (!dataSource.connectionString) {
      throw new Error('REST API connection string is required');
    }

    const response = await fetch(dataSource.connectionString, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...dataSource.headers,
      },
      body: JSON.stringify(query),
      signal,
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const result = await response.json();
    return result.data || result;
  }

  private async executeJSONQuery<T>(
    query: Query,
    dataSource: DataSource,
    signal: AbortSignal
  ): Promise<T[]> {
    // Load JSON data and apply filters/transformations
    return this.generateMockData<T>(query);
  }

  // ============================================================================
  // Query Control
  // ============================================================================

  cancelQuery(queryId: string): void {
    const controller = this.activeQueries.get(queryId);
    if (controller) {
      controller.abort();
      this.activeQueries.delete(queryId);
    }
  }

  cancelAllQueries(): void {
    this.activeQueries.forEach((controller) => controller.abort());
    this.activeQueries.clear();
  }

  getActiveQueries(): string[] {
    return Array.from(this.activeQueries.keys());
  }

  // ============================================================================
  // Execution History
  // ============================================================================

  getExecutionHistory(limit: number = 100): ExecutionRecord[] {
    return this.executionHistory.slice(-limit);
  }

  clearExecutionHistory(): void {
    this.executionHistory = [];
  }

  private recordExecution(
    queryId: string,
    query: Query,
    executionTime: number,
    rowCount: number,
    error?: unknown
  ): void {
    this.executionHistory.push({
      queryId,
      query,
      executionTime,
      rowCount,
      timestamp: new Date(),
      success: !error,
      error: error instanceof Error ? error.message : undefined,
    });

    // Keep only last 1000 records
    if (this.executionHistory.length > 1000) {
      this.executionHistory = this.executionHistory.slice(-1000);
    }
  }

  // ============================================================================
  // Helper Methods
  // ============================================================================

  private generateQueryId(): string {
    return `query_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
  }

  private inferType(value: unknown): any {
    if (typeof value === 'string') return 'string';
    if (typeof value === 'number') {
      return Number.isInteger(value) ? 'integer' : 'float';
    }
    if (typeof value === 'boolean') return 'boolean';
    if (value instanceof Date) return 'date';
    if (Array.isArray(value)) return 'array';
    if (typeof value === 'object') return 'object';
    return 'unknown';
  }

  private generateMockData<T>(query: Query): T[] {
    // Generate mock data for testing
    const mockData: T[] = [];
    const limit = query.limit || 100;

    for (let i = 0; i < limit; i++) {
      const row: Record<string, unknown> = {};

      query.dimensions.forEach((dim) => {
        row[dim.alias || dim.field] = `value_${i}`;
      });

      query.metrics.forEach((metric) => {
        row[metric.alias || metric.field] = Math.random() * 1000;
      });

      mockData.push(row as T);
    }

    return mockData;
  }
}

// ============================================================================
// Types
// ============================================================================

export interface PaginatedResult<T> {
  data: T[];
  metadata: QueryMetadata;
  pagination: {
    page: number;
    pageSize: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  executionTime?: number;
}

export interface ExecutionRecord {
  queryId: string;
  query: Query;
  executionTime: number;
  rowCount: number;
  timestamp: Date;
  success: boolean;
  error?: string;
}

export class QueryExecutionError extends Error {
  constructor(
    message: string,
    public queryId: string,
    public cause?: unknown
  ) {
    super(message);
    this.name = 'QueryExecutionError';
  }
}

// Factory function
export function createQueryExecutor(): QueryExecutor {
  return new QueryExecutor();
}
