/**
 * SQL-like Query Construction Engine
 * @module @harborgrid/enterprise-analytics/query
 */

import type {
  Query,
  Filter,
  FilterOperator,
  Dimension,
  Metric,
  SortClause,
  Aggregation,
  TimeRange,
} from '../types';

export class QueryBuilder {
  private query: Partial<Query>;

  constructor(dataSourceId: string) {
    this.query = {
      id: this.generateId(),
      dataSourceId,
      dimensions: [],
      metrics: [],
      filters: [],
      sort: [],
      cache: true,
      cacheTTL: 300000, // 5 minutes default
    };
  }

  private generateId(): string {
    return `query_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
  }

  // ============================================================================
  // Dimension Methods
  // ============================================================================

  addDimension(field: string, alias?: string): this {
    const dimension: Dimension = {
      id: this.generateId(),
      field,
      alias,
      type: this.inferType(field),
    };
    this.query.dimensions?.push(dimension);
    return this;
  }

  addTimeDimension(field: string, interval: string, alias?: string): this {
    const dimension: Dimension = {
      id: this.generateId(),
      field,
      alias,
      type: 'date' as any,
      bucket: {
        type: 'date',
        interval,
      },
    };
    this.query.dimensions?.push(dimension);
    return this;
  }

  addBucketedDimension(
    field: string,
    ranges: Array<{ from: number; to: number; label: string }>,
    alias?: string
  ): this {
    const dimension: Dimension = {
      id: this.generateId(),
      field,
      alias,
      type: 'number' as any,
      bucket: {
        type: 'numeric',
        ranges,
      },
    };
    this.query.dimensions?.push(dimension);
    return this;
  }

  // ============================================================================
  // Metric Methods
  // ============================================================================

  addMetric(
    field: string,
    aggregation: Aggregation,
    alias?: string,
    format?: string
  ): this {
    const metric: Metric = {
      id: this.generateId(),
      field,
      aggregation,
      alias,
      format,
    };
    this.query.metrics?.push(metric);
    return this;
  }

  count(field: string = '*', alias: string = 'count'): this {
    return this.addMetric(field, Aggregation.COUNT, alias);
  }

  countDistinct(field: string, alias?: string): this {
    return this.addMetric(field, Aggregation.COUNT_DISTINCT, alias || `${field}_distinct`);
  }

  sum(field: string, alias?: string): this {
    return this.addMetric(field, Aggregation.SUM, alias || `${field}_sum`);
  }

  avg(field: string, alias?: string): this {
    return this.addMetric(field, Aggregation.AVG, alias || `${field}_avg`);
  }

  min(field: string, alias?: string): this {
    return this.addMetric(field, Aggregation.MIN, alias || `${field}_min`);
  }

  max(field: string, alias?: string): this {
    return this.addMetric(field, Aggregation.MAX, alias || `${field}_max`);
  }

  median(field: string, alias?: string): this {
    return this.addMetric(field, Aggregation.MEDIAN, alias || `${field}_median`);
  }

  // Calculated metrics
  addCalculatedMetric(formula: string, alias: string, format?: string): this {
    const metric: Metric = {
      id: this.generateId(),
      field: formula,
      aggregation: Aggregation.CUSTOM,
      alias,
      format,
      calculation: formula,
    };
    this.query.metrics?.push(metric);
    return this;
  }

  // ============================================================================
  // Filter Methods
  // ============================================================================

  where(field: string, operator: FilterOperator, value: unknown): this {
    const filter: Filter = {
      id: this.generateId(),
      field,
      operator,
      value,
      condition: 'and',
    };
    this.query.filters?.push(filter);
    return this;
  }

  orWhere(field: string, operator: FilterOperator, value: unknown): this {
    const filter: Filter = {
      id: this.generateId(),
      field,
      operator,
      value,
      condition: 'or',
    };
    this.query.filters?.push(filter);
    return this;
  }

  whereIn(field: string, values: unknown[]): this {
    return this.where(field, FilterOperator.IN, values);
  }

  whereNotIn(field: string, values: unknown[]): this {
    return this.where(field, FilterOperator.NOT_IN, values);
  }

  whereBetween(field: string, min: number, max: number): this {
    return this.where(field, FilterOperator.BETWEEN, [min, max]);
  }

  whereNull(field: string): this {
    return this.where(field, FilterOperator.IS_NULL, null);
  }

  whereNotNull(field: string): this {
    return this.where(field, FilterOperator.IS_NOT_NULL, null);
  }

  whereLike(field: string, pattern: string, caseSensitive: boolean = false): this {
    const filter: Filter = {
      id: this.generateId(),
      field,
      operator: FilterOperator.CONTAINS,
      value: pattern,
      caseSensitive,
      condition: 'and',
    };
    this.query.filters?.push(filter);
    return this;
  }

  // ============================================================================
  // Time Range Methods
  // ============================================================================

  timeRange(start: Date | string, end: Date | string, timezone?: string): this {
    this.query.timeRange = { start, end, timezone };
    return this;
  }

  last24Hours(): this {
    const end = new Date();
    const start = new Date(end.getTime() - 24 * 60 * 60 * 1000);
    return this.timeRange(start, end);
  }

  last7Days(): this {
    const end = new Date();
    const start = new Date(end.getTime() - 7 * 24 * 60 * 60 * 1000);
    return this.timeRange(start, end);
  }

  last30Days(): this {
    const end = new Date();
    const start = new Date(end.getTime() - 30 * 24 * 60 * 60 * 1000);
    return this.timeRange(start, end);
  }

  thisMonth(): this {
    const now = new Date();
    const start = new Date(now.getFullYear(), now.getMonth(), 1);
    const end = new Date(now.getFullYear(), now.getMonth() + 1, 0);
    return this.timeRange(start, end);
  }

  thisYear(): this {
    const now = new Date();
    const start = new Date(now.getFullYear(), 0, 1);
    const end = new Date(now.getFullYear(), 11, 31);
    return this.timeRange(start, end);
  }

  // ============================================================================
  // Sorting Methods
  // ============================================================================

  orderBy(field: string, direction: 'asc' | 'desc' = 'asc'): this {
    const sort: SortClause = { field, direction };
    if (!this.query.sort) {
      this.query.sort = [];
    }
    this.query.sort.push(sort);
    return this;
  }

  orderByAsc(field: string): this {
    return this.orderBy(field, 'asc');
  }

  orderByDesc(field: string): this {
    return this.orderBy(field, 'desc');
  }

  // ============================================================================
  // Grouping Methods
  // ============================================================================

  groupBy(...fields: string[]): this {
    this.query.groupBy = fields;
    return this;
  }

  having(field: string, operator: FilterOperator, value: unknown): this {
    const filter: Filter = {
      id: this.generateId(),
      field,
      operator,
      value,
      condition: 'and',
    };
    if (!this.query.having) {
      this.query.having = [];
    }
    this.query.having.push(filter);
    return this;
  }

  // ============================================================================
  // Pagination Methods
  // ============================================================================

  limit(limit: number): this {
    this.query.limit = limit;
    return this;
  }

  offset(offset: number): this {
    this.query.offset = offset;
    return this;
  }

  paginate(page: number, pageSize: number): this {
    this.query.limit = pageSize;
    this.query.offset = (page - 1) * pageSize;
    return this;
  }

  // ============================================================================
  // Cache Methods
  // ============================================================================

  cache(enabled: boolean = true, ttl?: number): this {
    this.query.cache = enabled;
    if (ttl !== undefined) {
      this.query.cacheTTL = ttl;
    }
    return this;
  }

  noCache(): this {
    return this.cache(false);
  }

  // ============================================================================
  // Build Methods
  // ============================================================================

  build(): Query {
    if (!this.query.name) {
      this.query.name = `Query ${new Date().toISOString()}`;
    }
    return this.query as Query;
  }

  toSQL(tableName: string): string {
    const parts: string[] = [];

    // SELECT clause
    const selectFields: string[] = [];

    // Add dimensions
    this.query.dimensions?.forEach((dim) => {
      selectFields.push(dim.alias ? `${dim.field} AS ${dim.alias}` : dim.field);
    });

    // Add metrics
    this.query.metrics?.forEach((metric) => {
      const aggFunc = this.getAggregationFunction(metric.aggregation);
      const fieldExpr = metric.calculation || `${aggFunc}(${metric.field})`;
      selectFields.push(metric.alias ? `${fieldExpr} AS ${metric.alias}` : fieldExpr);
    });

    parts.push(`SELECT ${selectFields.join(', ')}`);
    parts.push(`FROM ${tableName}`);

    // WHERE clause
    if (this.query.filters && this.query.filters.length > 0) {
      const whereClauses = this.query.filters.map((f) => this.buildFilterClause(f));
      parts.push(`WHERE ${whereClauses.join(' AND ')}`);
    }

    // GROUP BY clause
    if (this.query.groupBy && this.query.groupBy.length > 0) {
      parts.push(`GROUP BY ${this.query.groupBy.join(', ')}`);
    }

    // HAVING clause
    if (this.query.having && this.query.having.length > 0) {
      const havingClauses = this.query.having.map((f) => this.buildFilterClause(f));
      parts.push(`HAVING ${havingClauses.join(' AND ')}`);
    }

    // ORDER BY clause
    if (this.query.sort && this.query.sort.length > 0) {
      const orderClauses = this.query.sort.map(
        (s) => `${s.field} ${s.direction.toUpperCase()}`
      );
      parts.push(`ORDER BY ${orderClauses.join(', ')}`);
    }

    // LIMIT and OFFSET
    if (this.query.limit !== undefined) {
      parts.push(`LIMIT ${this.query.limit}`);
    }
    if (this.query.offset !== undefined) {
      parts.push(`OFFSET ${this.query.offset}`);
    }

    return parts.join('\n');
  }

  // ============================================================================
  // Helper Methods
  // ============================================================================

  private inferType(field: string): any {
    // Simple type inference based on field name
    if (field.includes('date') || field.includes('time')) return 'date';
    if (field.includes('count') || field.includes('amount')) return 'number';
    return 'string';
  }

  private getAggregationFunction(aggregation: Aggregation): string {
    const mapping: Record<Aggregation, string> = {
      [Aggregation.COUNT]: 'COUNT',
      [Aggregation.COUNT_DISTINCT]: 'COUNT(DISTINCT',
      [Aggregation.SUM]: 'SUM',
      [Aggregation.AVG]: 'AVG',
      [Aggregation.MIN]: 'MIN',
      [Aggregation.MAX]: 'MAX',
      [Aggregation.MEDIAN]: 'MEDIAN',
      [Aggregation.PERCENTILE]: 'PERCENTILE',
      [Aggregation.STDDEV]: 'STDDEV',
      [Aggregation.VARIANCE]: 'VARIANCE',
      [Aggregation.FIRST]: 'FIRST',
      [Aggregation.LAST]: 'LAST',
      [Aggregation.CUSTOM]: '',
    };
    return mapping[aggregation] || aggregation.toUpperCase();
  }

  private buildFilterClause(filter: Filter): string {
    const { field, operator, value } = filter;

    switch (operator) {
      case FilterOperator.EQUALS:
        return `${field} = ${this.formatValue(value)}`;
      case FilterOperator.NOT_EQUALS:
        return `${field} != ${this.formatValue(value)}`;
      case FilterOperator.GREATER_THAN:
        return `${field} > ${this.formatValue(value)}`;
      case FilterOperator.GREATER_THAN_OR_EQUAL:
        return `${field} >= ${this.formatValue(value)}`;
      case FilterOperator.LESS_THAN:
        return `${field} < ${this.formatValue(value)}`;
      case FilterOperator.LESS_THAN_OR_EQUAL:
        return `${field} <= ${this.formatValue(value)}`;
      case FilterOperator.IN:
        return `${field} IN (${this.formatArray(value as unknown[])})`;
      case FilterOperator.NOT_IN:
        return `${field} NOT IN (${this.formatArray(value as unknown[])})`;
      case FilterOperator.CONTAINS:
        return `${field} LIKE '%${value}%'`;
      case FilterOperator.STARTS_WITH:
        return `${field} LIKE '${value}%'`;
      case FilterOperator.ENDS_WITH:
        return `${field} LIKE '%${value}'`;
      case FilterOperator.IS_NULL:
        return `${field} IS NULL`;
      case FilterOperator.IS_NOT_NULL:
        return `${field} IS NOT NULL`;
      case FilterOperator.BETWEEN:
        const [min, max] = value as [number, number];
        return `${field} BETWEEN ${min} AND ${max}`;
      default:
        return `${field} = ${this.formatValue(value)}`;
    }
  }

  private formatValue(value: unknown): string {
    if (value === null) return 'NULL';
    if (typeof value === 'string') return `'${value.replace(/'/g, "''")}'`;
    if (value instanceof Date) return `'${value.toISOString()}'`;
    return String(value);
  }

  private formatArray(values: unknown[]): string {
    return values.map((v) => this.formatValue(v)).join(', ');
  }

  // ============================================================================
  // Chainable Utility Methods
  // ============================================================================

  clone(): QueryBuilder {
    const cloned = new QueryBuilder(this.query.dataSourceId || '');
    cloned.query = JSON.parse(JSON.stringify(this.query));
    return cloned;
  }

  reset(): this {
    this.query = {
      id: this.generateId(),
      dataSourceId: this.query.dataSourceId,
      dimensions: [],
      metrics: [],
      filters: [],
      sort: [],
      cache: true,
      cacheTTL: 300000,
    };
    return this;
  }

  setName(name: string): this {
    this.query.name = name;
    return this;
  }

  setRefreshInterval(interval: number): this {
    this.query.refreshInterval = interval;
    return this;
  }
}

// Factory function
export function createQueryBuilder(dataSourceId: string): QueryBuilder {
  return new QueryBuilder(dataSourceId);
}
