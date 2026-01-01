/**
 * Query Optimization Engine
 * @module @harborgrid/enterprise-analytics/query
 */

import type { Query, Filter, FilterOperator } from '../types';

export interface OptimizationRule {
  name: string;
  description: string;
  apply: (query: Query) => Query;
  shouldApply: (query: Query) => boolean;
}

export class QueryOptimizer {
  private rules: OptimizationRule[] = [];

  constructor() {
    this.registerDefaultRules();
  }

  // ============================================================================
  // Rule Registration
  // ============================================================================

  registerRule(rule: OptimizationRule): void {
    this.rules.push(rule);
  }

  private registerDefaultRules(): void {
    this.registerRule(this.createFilterPushdownRule());
    this.registerRule(this.createRedundantFilterRule());
    this.registerRule(this.createFilterReorderRule());
    this.registerRule(this.createLimitOptimizationRule());
    this.registerRule(this.createDimensionDeduplicationRule());
    this.registerRule(this.createMetricOptimizationRule());
  }

  // ============================================================================
  // Main Optimization Method
  // ============================================================================

  optimize(query: Query): Query {
    let optimizedQuery = { ...query };
    let iterations = 0;
    const maxIterations = 10;

    // Apply rules iteratively until no more optimizations or max iterations
    while (iterations < maxIterations) {
      let changed = false;

      for (const rule of this.rules) {
        if (rule.shouldApply(optimizedQuery)) {
          const before = JSON.stringify(optimizedQuery);
          optimizedQuery = rule.apply(optimizedQuery);
          const after = JSON.stringify(optimizedQuery);

          if (before !== after) {
            changed = true;
          }
        }
      }

      if (!changed) break;
      iterations++;
    }

    return optimizedQuery;
  }

  // ============================================================================
  // Optimization Rules
  // ============================================================================

  private createFilterPushdownRule(): OptimizationRule {
    return {
      name: 'filter-pushdown',
      description: 'Push filters as early as possible in the query execution',
      shouldApply: (query) => query.filters.length > 0,
      apply: (query) => {
        // Sort filters by selectivity (more selective filters first)
        const optimizedFilters = [...query.filters].sort((a, b) => {
          return this.estimateFilterSelectivity(a) - this.estimateFilterSelectivity(b);
        });

        return {
          ...query,
          filters: optimizedFilters,
        };
      },
    };
  }

  private createRedundantFilterRule(): OptimizationRule {
    return {
      name: 'redundant-filter-removal',
      description: 'Remove redundant or contradictory filters',
      shouldApply: (query) => query.filters.length > 1,
      apply: (query) => {
        const uniqueFilters = new Map<string, Filter>();

        for (const filter of query.filters) {
          const key = `${filter.field}_${filter.operator}`;
          const existing = uniqueFilters.get(key);

          if (!existing) {
            uniqueFilters.set(key, filter);
          } else {
            // Keep the more restrictive filter
            if (this.isMoreRestrictive(filter, existing)) {
              uniqueFilters.set(key, filter);
            }
          }
        }

        return {
          ...query,
          filters: Array.from(uniqueFilters.values()),
        };
      },
    };
  }

  private createFilterReorderRule(): OptimizationRule {
    return {
      name: 'filter-reorder',
      description: 'Reorder filters for optimal execution',
      shouldApply: (query) => query.filters.length > 1,
      apply: (query) => {
        // Prioritize: equality > range > pattern matching > function-based
        const filterPriority = (filter: Filter): number => {
          switch (filter.operator) {
            case FilterOperator.EQUALS:
              return 1;
            case FilterOperator.IN:
              return 2;
            case FilterOperator.BETWEEN:
            case FilterOperator.GREATER_THAN:
            case FilterOperator.LESS_THAN:
              return 3;
            case FilterOperator.CONTAINS:
            case FilterOperator.STARTS_WITH:
              return 4;
            case FilterOperator.REGEX:
              return 5;
            default:
              return 6;
          }
        };

        const reorderedFilters = [...query.filters].sort((a, b) => {
          return filterPriority(a) - filterPriority(b);
        });

        return {
          ...query,
          filters: reorderedFilters,
        };
      },
    };
  }

  private createLimitOptimizationRule(): OptimizationRule {
    return {
      name: 'limit-optimization',
      description: 'Optimize queries with LIMIT clauses',
      shouldApply: (query) => query.limit !== undefined && query.limit > 0,
      apply: (query) => {
        // If we have a limit and no offset, we can optimize aggregations
        if (query.limit && !query.offset && query.metrics.length > 0) {
          return {
            ...query,
            metadata: {
              ...query.metadata,
              earlyLimit: true,
            },
          };
        }
        return query;
      },
    };
  }

  private createDimensionDeduplicationRule(): OptimizationRule {
    return {
      name: 'dimension-deduplication',
      description: 'Remove duplicate dimensions',
      shouldApply: (query) => query.dimensions.length > 1,
      apply: (query) => {
        const uniqueDimensions = new Map();

        for (const dim of query.dimensions) {
          if (!uniqueDimensions.has(dim.field)) {
            uniqueDimensions.set(dim.field, dim);
          }
        }

        return {
          ...query,
          dimensions: Array.from(uniqueDimensions.values()),
        };
      },
    };
  }

  private createMetricOptimizationRule(): OptimizationRule {
    return {
      name: 'metric-optimization',
      description: 'Optimize metric calculations',
      shouldApply: (query) => query.metrics.length > 0,
      apply: (query) => {
        // Combine multiple COUNT operations on the same field
        const metricMap = new Map();
        const optimizedMetrics = [];

        for (const metric of query.metrics) {
          const key = `${metric.field}_${metric.aggregation}`;

          if (!metricMap.has(key)) {
            metricMap.set(key, metric);
            optimizedMetrics.push(metric);
          }
        }

        return {
          ...query,
          metrics: optimizedMetrics,
        };
      },
    };
  }

  // ============================================================================
  // Helper Methods
  // ============================================================================

  private estimateFilterSelectivity(filter: Filter): number {
    // Lower number = more selective (fewer results)
    // This is a heuristic estimation
    switch (filter.operator) {
      case FilterOperator.EQUALS:
        return 0.1;
      case FilterOperator.IN:
        return 0.3;
      case FilterOperator.BETWEEN:
        return 0.5;
      case FilterOperator.GREATER_THAN:
      case FilterOperator.LESS_THAN:
        return 0.4;
      case FilterOperator.CONTAINS:
        return 0.6;
      case FilterOperator.STARTS_WITH:
        return 0.5;
      case FilterOperator.IS_NULL:
      case FilterOperator.IS_NOT_NULL:
        return 0.2;
      default:
        return 0.7;
    }
  }

  private isMoreRestrictive(filter1: Filter, filter2: Filter): boolean {
    // Simple heuristic: smaller selectivity is more restrictive
    return this.estimateFilterSelectivity(filter1) < this.estimateFilterSelectivity(filter2);
  }

  // ============================================================================
  // Query Analysis
  // ============================================================================

  analyzeQuery(query: Query): QueryAnalysis {
    const analysis: QueryAnalysis = {
      complexity: this.calculateComplexity(query),
      estimatedCost: this.estimateCost(query),
      canUseIndex: this.canUseIndex(query),
      suggestedIndexes: this.suggestIndexes(query),
      warnings: this.generateWarnings(query),
      optimizationOpportunities: this.findOptimizationOpportunities(query),
    };

    return analysis;
  }

  private calculateComplexity(query: Query): number {
    let complexity = 0;

    // Base complexity
    complexity += query.dimensions.length * 2;
    complexity += query.metrics.length * 3;
    complexity += query.filters.length * 2;
    complexity += (query.sort?.length || 0) * 1;
    complexity += query.groupBy ? query.groupBy.length * 3 : 0;
    complexity += query.having ? query.having.length * 2 : 0;

    return complexity;
  }

  private estimateCost(query: Query): number {
    // Simplified cost estimation
    let cost = 100; // Base cost

    // Aggregations are expensive
    cost += query.metrics.length * 50;

    // Joins (if we had them) would be very expensive
    // Filters reduce cost
    cost -= query.filters.length * 10;

    // Sorting adds cost
    cost += (query.sort?.length || 0) * 20;

    // Grouping is expensive
    cost += (query.groupBy?.length || 0) * 30;

    return Math.max(cost, 0);
  }

  private canUseIndex(query: Query): boolean {
    // Check if query has filters on indexed fields
    // This is simplified; in reality, would check against actual schema
    return query.filters.some(
      (f) => f.operator === FilterOperator.EQUALS || f.operator === FilterOperator.IN
    );
  }

  private suggestIndexes(query: Query): string[] {
    const indexes = new Set<string>();

    // Suggest indexes for frequently filtered fields
    query.filters.forEach((filter) => {
      if (
        filter.operator === FilterOperator.EQUALS ||
        filter.operator === FilterOperator.IN
      ) {
        indexes.add(filter.field);
      }
    });

    // Suggest indexes for grouped fields
    query.groupBy?.forEach((field) => {
      indexes.add(field);
    });

    // Suggest indexes for sorted fields
    query.sort?.forEach((sort) => {
      indexes.add(sort.field);
    });

    return Array.from(indexes);
  }

  private generateWarnings(query: Query): string[] {
    const warnings: string[] = [];

    // Check for missing filters
    if (query.filters.length === 0 && !query.limit) {
      warnings.push('Query has no filters and no limit; may return large result set');
    }

    // Check for expensive operations
    if (query.metrics.length > 10) {
      warnings.push('Query has many aggregations; consider splitting into multiple queries');
    }

    // Check for inefficient patterns
    const hasRegexFilter = query.filters.some((f) => f.operator === FilterOperator.REGEX);
    if (hasRegexFilter) {
      warnings.push('Regex filters can be slow; consider using simpler patterns');
    }

    // Check for missing group by with aggregations
    if (query.metrics.length > 0 && query.dimensions.length > 0 && !query.groupBy) {
      warnings.push('Aggregations without GROUP BY; may not produce expected results');
    }

    return warnings;
  }

  private findOptimizationOpportunities(query: Query): string[] {
    const opportunities: string[] = [];

    // Check if caching is disabled but could be useful
    if (!query.cache && query.metrics.length > 0) {
      opportunities.push('Enable caching for expensive aggregation queries');
    }

    // Check if filters could be more selective
    const hasWildcardFilters = query.filters.some(
      (f) => f.operator === FilterOperator.CONTAINS
    );
    if (hasWildcardFilters) {
      opportunities.push('Use more specific filters instead of wildcard searches');
    }

    // Check if we can use materialized views
    if (query.groupBy && query.groupBy.length > 0 && query.metrics.length > 0) {
      opportunities.push('Consider creating a materialized view for this aggregation');
    }

    return opportunities;
  }
}

export interface QueryAnalysis {
  complexity: number;
  estimatedCost: number;
  canUseIndex: boolean;
  suggestedIndexes: string[];
  warnings: string[];
  optimizationOpportunities: string[];
}

// Factory function
export function createQueryOptimizer(): QueryOptimizer {
  return new QueryOptimizer();
}
